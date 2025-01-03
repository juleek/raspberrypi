use anyhow::{anyhow, Context, Result};


fn remove_confirmed(measurement: &agg_proto::MeasurementResp,
                    measurements: &mut Vec<agg_proto::MeasurementReq>) {
   {
      let upper_bound = measurements.partition_point(|req| req.counter <= measurement.counter);
      measurements.drain(0..upper_bound);
   }
}

fn populate_measurements(thread_rx: &mut crate::sensor::Rx,
                         measurements: &mut Vec<agg_proto::MeasurementReq>,
                         counter: &mut i64) {
   while let Ok(measurement) = thread_rx.try_recv() {
      *counter += 1;
      let req = agg_proto::MeasurementReq { measurement: Some(measurement.into()),
                                            counter:     *counter, };
      measurements.push(req);
   }
}

async fn one_iteration(ct: &tokio_util::sync::CancellationToken,
                       server_host_port: &str,
                       thread_rx: &mut crate::sensor::Rx,
                       measurements: &mut Vec<agg_proto::MeasurementReq>,
                       counter: &mut i64)
                       -> Result<()> {
   populate_measurements(thread_rx, measurements, counter);
   let mut client = agg_proto::agg_client::AggClient::connect(server_host_port.to_string())
      .await.with_context(|| anyhow!("Failed to connect to {server_host_port}"))?;

   let (tx_outbound, rx_outbound) = tokio::sync::mpsc::channel(10);
   let outbound = tokio_stream::wrappers::ReceiverStream::new(rx_outbound);
   let mut inbound_stream = client.send_measurement(outbound).await?.into_inner();

   for i in 0..measurements.len() {
      populate_measurements(thread_rx, measurements, counter);
      tokio::select! {
         _ = ct.cancelled() => {
            return Ok(());
         },
         res = tx_outbound.send(measurements[i].clone()) => {
            res.with_context(|| anyhow!("Failed to send measurement at index {i} from {measurements:?}"))?;
         }
      }
   }
   loop {
      tokio::select! {
         _ = ct.cancelled() => {
            return Ok(());
         },
         Some(measurement) = thread_rx.recv() => {
            *counter += 1;
            let req = agg_proto::MeasurementReq { measurement: Some(measurement.into()), counter: *counter, };
            println!("Client sending: {:?}", &req);
            measurements.push(req.clone());
            tx_outbound.send(req.clone()).await.with_context(|| anyhow!("Failed to send measurement {req:?}"))?;
         },
         response = inbound_stream.message() => {
            match response {
               Ok(Some(measurement)) => remove_confirmed(&measurement, measurements),
               Ok(None) => { return Err(anyhow!("Received an empty response from the stream"));   },
               Err(e) => {   return Err(anyhow!("Got error from grpc: {e:?}"));  }
            }
         }
      }
   }
}


// TODO: move to publisher.rs with the name: poll_and_publish_forever()
pub async fn poll_and_publish_forever(ct: &tokio_util::sync::CancellationToken,
                                      mut thread_rx: crate::sensor::Rx,
                                      server_host_port: &str)
                                      -> Result<()> {
   let mut measurements: Vec<agg_proto::MeasurementReq> = Vec::new();
   let mut counter = chrono::Utc::now().timestamp_nanos_opt().unwrap();

   loop {
      let res = one_iteration(ct, server_host_port, &mut thread_rx, &mut measurements, &mut counter).await;
      if let Err(e) = res {
         println!("Failed to do one iteration: {e:?}");
      }
      tokio::time::sleep(std::time::Duration::from_millis(290)).await;
   }
}



//
// ===========================================================================================================
// Tests


#[cfg(test)]
mod tests {
   use super::*;
   use pretty_assertions::assert_eq;

   fn measurement() -> crate::sensor::Measurement {
      crate::sensor::Measurement { sensor:      "ambient".to_string(),
                                   temperature: Some(26.8),
                                   errors:      vec!["error1".to_string(), "error2".to_string()], }
   }

   fn populate_measurements(measurement: &crate::sensor::Measurement,
                            counters: Vec<i64>)
                            -> Vec<agg_proto::MeasurementReq> {
      let mut measurements = Vec::new();
      for counter in counters {
         let req = agg_proto::MeasurementReq { measurement: Some(measurement.clone().into()),
                                               counter };
         measurements.push(req);
      }
      measurements
   }


   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_between() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, vec![1, 2, 3, 4, 10]);
      let agg_resp = agg_proto::MeasurementResp { counter: 7 };

      remove_confirmed(&agg_resp, &mut measurements);

      let expected = populate_measurements(&measurement, vec![10]);

      assert_eq!(expected, measurements);
   }


   #[test]
   fn test_remove_confirmed_removes_all_elements_before_if_ts_in_middle() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, vec![1, 2, 3, 4, 10]);
      let agg_resp = agg_proto::MeasurementResp { counter: 3 };

      remove_confirmed(&agg_resp, &mut measurements);
      let expected = populate_measurements(&measurement, vec![4, 10]);

      assert_eq!(expected, measurements);
   }


   #[test]
   fn test_remove_confirmed_removes_all_elements_if_ts_is_max() {
      let mut measurements = populate_measurements(&measurement(), vec![1, 2, 3, 4, 10]);
      let agg_resp = agg_proto::MeasurementResp { counter: 10 };

      remove_confirmed(&agg_resp, &mut measurements);

      let expected: Vec<agg_proto::MeasurementReq> = vec![];

      assert_eq!(expected, measurements);
   }
   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_strictly_larger_than_max() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, vec![1, 2, 3, 4, 10]);
      let agg_resp = agg_proto::MeasurementResp { counter: 20 };

      remove_confirmed(&agg_resp, &mut measurements);

      let expected: Vec<agg_proto::MeasurementReq> = Vec::new();

      assert_eq!(expected, measurements);
   }




   #[test]
   fn test_remove_confirmed_removes_only_first_if_ts_is_min() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, vec![1, 2, 3, 4, 10]);
      let agg_resp = agg_proto::MeasurementResp { counter: 1 };

      remove_confirmed(&agg_resp, &mut measurements);

      let expected = populate_measurements(&measurement, vec![2, 3, 4, 10]);

      assert_eq!(expected, measurements);
   }
   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_strictly_smaller_than_min() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, vec![1, 2, 3, 4, 10]);
      let agg_resp = agg_proto::MeasurementResp { counter: 0 };

      remove_confirmed(&agg_resp, &mut measurements);

      let expected = populate_measurements(&measurement, vec![1, 2, 3, 4, 10]);

      assert_eq!(expected, measurements);
   }



   #[test]
   fn test_remove_confirmed_if_measurements_is_empty() {
      let mut measurements = Vec::new();
      let agg_resp = agg_proto::MeasurementResp { counter: 1 };

      remove_confirmed(&agg_resp, &mut measurements);

      let expected: Vec<agg_proto::MeasurementReq> = Vec::new();

      assert_eq!(expected, measurements);
   }
}
