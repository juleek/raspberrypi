use anyhow::{anyhow, Context, Result};


fn remove_confirmed(confirmed: i64, measurements: &mut Vec<common::pb::MeasurementReq>) {
   let upper_bound = measurements.partition_point(|req| req.counter <= confirmed);
   measurements.drain(0..upper_bound);
}

fn populate_measurements_from_rx(thread_rx: &mut common::Rx,
                                 measurements: &mut Vec<common::pb::MeasurementReq>,
                                 counter: &mut i64) {
   while let Ok(measurement) = thread_rx.try_recv() {
      *counter += 1;
      let req = common::pb::MeasurementReq { measurement: Some(measurement.into()),
                                             counter:     *counter, };
      measurements.push(req);
   }
}

async fn one_iteration(ct: &tokio_util::sync::CancellationToken,
                       server_host_port: &str,
                       thread_rx: &mut common::Rx,
                       measurements: &mut Vec<common::pb::MeasurementReq>,
                       counter: &mut i64)
                       -> Result<()> {
   populate_measurements_from_rx(thread_rx, measurements, counter);
   let mut client = common::pb::agg_client::AggClient::connect(server_host_port.to_string())
      .await.with_context(|| anyhow!("Failed to connect to {server_host_port}"))?;

   let (tx_outbound, rx_outbound) = tokio::sync::mpsc::channel(10);
   let outbound = tokio_stream::wrappers::ReceiverStream::new(rx_outbound);
   let mut inbound_stream = client.send_measurement(outbound).await?.into_inner();

   for i in 0..measurements.len() {
      populate_measurements_from_rx(thread_rx, measurements, counter);
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
            let req = common::pb::MeasurementReq { measurement: Some(measurement.into()), counter: *counter, };
            log::info!("Sending: {req:?} to {server_host_port}");
            measurements.push(req.clone());
            tx_outbound.send(req.clone()).await.with_context(|| anyhow!("Failed to send measurement {req:?}"))?;
         },
         confirmed = inbound_stream.message() => {
            match confirmed {
               Ok(Some(confirmed)) => remove_confirmed(confirmed.counter, measurements),
               Ok(None) => { return Err(anyhow!("Received an empty response from the stream"));   },
               Err(e) => {   return Err(anyhow!("Got error from grpc: {e:?}"));  }
            }
         }
      }
   }
}


pub async fn poll_and_publish_forever(ct: &tokio_util::sync::CancellationToken,
                                      mut thread_rx: common::Rx,
                                      server_host_port: &str)
                                      -> Result<()> {
   let mut measurements: Vec<common::pb::MeasurementReq> = Vec::new();
   let mut counter = chrono::Utc::now().timestamp_nanos_opt().unwrap();

   loop {
      let res = one_iteration(ct, server_host_port, &mut thread_rx, &mut measurements, &mut counter).await;
      if let Err(e) = res {
         log::warn!("Failed to do one iteration: {e:?}");
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

   fn measurement() -> common::Measurement {
      common::Measurement { sensor:      "ambient".to_string(),
                            temperature: Some(26.8),
                            errors:      vec!["error1".to_string(), "error2".to_string()], }
   }

   fn populate_measurements(measurement: &common::Measurement,
                            counters: &[i64])
                            -> Vec<common::pb::MeasurementReq> {
      let mut measurements = Vec::new();
      for counter in counters {
         let req = common::pb::MeasurementReq { measurement: Some(measurement.clone().into()),
                                                counter:     *counter, };
         measurements.push(req);
      }
      measurements
   }


   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_between() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, &[1, 2, 3, 4, 10]);

      remove_confirmed(7, &mut measurements);

      let expected = populate_measurements(&measurement, &[10]);

      assert_eq!(expected, measurements);
   }
   #[test]
   fn test_remove_confirmed_removes_all_elements_before_if_ts_in_middle() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, &[1, 2, 3, 4, 10]);

      remove_confirmed(3, &mut measurements);
      let expected = populate_measurements(&measurement, &[4, 10]);

      assert_eq!(expected, measurements);
   }


   #[test]
   fn test_remove_confirmed_removes_all_elements_if_ts_is_max() {
      let mut measurements = populate_measurements(&measurement(), &[1, 2, 3, 4, 10]);

      remove_confirmed(10, &mut measurements);

      let expected = Vec::<common::pb::MeasurementReq>::default();

      assert_eq!(expected, measurements);
   }
   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_strictly_larger_than_max() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, &[1, 2, 3, 4, 10]);

      remove_confirmed(20, &mut measurements);

      let expected: Vec<common::pb::MeasurementReq> = Vec::new();

      assert_eq!(expected, measurements);
   }




   #[test]
   fn test_remove_confirmed_removes_only_first_if_ts_is_min() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, &[1, 2, 3, 4, 10]);

      remove_confirmed(1, &mut measurements);

      let expected = populate_measurements(&measurement, &[2, 3, 4, 10]);

      assert_eq!(expected, measurements);
   }
   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_strictly_smaller_than_min() {
      let measurement = measurement();
      let mut measurements = populate_measurements(&measurement, &[1, 2, 3, 4, 10]);

      remove_confirmed(0, &mut measurements);

      let expected = populate_measurements(&measurement, &[1, 2, 3, 4, 10]);

      assert_eq!(expected, measurements);
   }



   #[test]
   fn test_remove_confirmed_if_measurements_is_empty() {
      let mut measurements = Vec::new();

      remove_confirmed(1, &mut measurements);

      let expected: Vec<common::pb::MeasurementReq> = Vec::new();

      assert_eq!(expected, measurements);
   }
}
