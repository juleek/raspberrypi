use anyhow::{anyhow, Context, Result};


struct State {
   thread_rx:    common::Rx,
   measurements: Vec<common::pb::MeasurementReq>,
   counter:      i64,
}
impl State {
   fn new(thread_rx: common::Rx) -> Self {
      Self { thread_rx,
             measurements: Default::default(),
             counter: chrono::Utc::now().timestamp_nanos_opt().unwrap() }
   }
   fn on_new_measurement(&mut self, measurement: common::Measurement) -> common::pb::MeasurementReq {
      self.counter += 1;
      let req = common::pb::MeasurementReq { measurement: Some(measurement.into()),
                                             counter:     self.counter, };
      self.measurements.push(req.clone());
      if self.measurements.len() > 100 {
         log::warn!("Measurements size: {}", self.measurements.len());
      }
      req
   }
   fn pull_from_rx(&mut self) {
      while let Ok(measurement) = self.thread_rx.try_recv() {
         self.on_new_measurement(measurement);
      }
   }
   fn remove_confirmed(&mut self, confirmed: i64) {
      let upper_bound = self.measurements.partition_point(|req| req.counter <= confirmed);
      self.measurements.drain(0..upper_bound);
   }
}



async fn one_iteration(ct: &tokio_util::sync::CancellationToken,
                       server_host_port: &str,
                       state: &mut State)
                       -> Result<()> {
   state.pull_from_rx();
   let mut client = common::pb::agg_client::AggClient::connect(server_host_port.to_string())
      .await.with_context(|| anyhow!("Failed to connect to {server_host_port}"))?;

   let (tx_outbound, rx_outbound) = tokio::sync::mpsc::channel(10);
   let outbound = tokio_stream::wrappers::ReceiverStream::new(rx_outbound);
   let mut inbound_stream = client.store_measurement(outbound).await?.into_inner();

   for i in 0..state.measurements.len() {
      state.pull_from_rx();
      log::info!("Sending: {:?} to {server_host_port}", state.measurements[i]);
      tokio::select! {
         _ = ct.cancelled() => {
            return Ok(());
         },
         res = tx_outbound.send(state.measurements[i].clone()) => {
            res.with_context(|| anyhow!("Failed to send measurement at index {i} from {:?}", state.measurements))?;
         }
      }
   }
   loop {
      tokio::select! {
         _ = ct.cancelled() => {
            return Ok(());
         },
         Some(measurement) = state.thread_rx.recv() => {
            let req = state.on_new_measurement(measurement);
            log::info!("Sending: {req:?} to {server_host_port}");
            tx_outbound.send(req.clone()).await.with_context(|| anyhow!("Failed to send measurement {req:?}"))?;
         },
         confirmed = inbound_stream.message() => {
            match confirmed {
               Ok(Some(confirmed)) => state.remove_confirmed(confirmed.counter),
               Ok(None) => { return Err(anyhow!("Received an empty response from the stream"));   },
               Err(e) => {   return Err(anyhow!("Got error from grpc: {e:?}"));  }
            }
         }
      }
   }
}



pub async fn poll_and_publish_forever(ct: &tokio_util::sync::CancellationToken,
                                      thread_rx: common::Rx,
                                      server_host_port: &str)
                                      -> Result<()> {
   let mut state = State::new(thread_rx);
   loop {
      let res = one_iteration(ct, server_host_port, &mut state).await;
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

   fn ts_ymd(year: i32, month: u32, day: u32) -> common::MicroSecTs {
      use chrono::TimeZone;
      let ts = chrono::Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).earliest().unwrap();
      common::MicroSecTs(ts)
   }

   fn measurement() -> common::Measurement {
      common::Measurement { sensor:      "ambient".to_string(),
                            temperature: Some(26.8),
                            read_ts: ts_ymd(2024, 1, 1),
                            error:      "error1".to_string(), }
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

   fn create_state(measurements: Vec<common::pb::MeasurementReq>) -> State {
      let (_tx, rx) = tokio::sync::mpsc::channel(100);
      State { counter: 0,
              measurements,
              thread_rx: rx }
   }


   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_between() {
      let measurement = measurement();
      let mut state = create_state(populate_measurements(&measurement, &[1, 2, 3, 4, 10]));

      state.remove_confirmed(7);

      let expected = populate_measurements(&measurement, &[10]);

      assert_eq!(state.measurements, expected);
   }
   #[test]
   fn test_remove_confirmed_removes_all_elements_before_if_ts_in_middle() {
      let measurement = measurement();
      let mut state = create_state(populate_measurements(&measurement, &[1, 2, 3, 4, 10]));

      state.remove_confirmed(3);
      let expected = populate_measurements(&measurement, &[4, 10]);

      assert_eq!(state.measurements, expected);
   }


   #[test]
   fn test_remove_confirmed_removes_all_elements_if_ts_is_max() {
      let mut state = create_state(populate_measurements(&measurement(), &[1, 2, 3, 4, 10]));

      state.remove_confirmed(10);

      let expected = Vec::<common::pb::MeasurementReq>::default();

      assert_eq!(state.measurements, expected);
   }
   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_strictly_larger_than_max() {
      let measurement = measurement();
      let mut state = create_state(populate_measurements(&measurement, &[1, 2, 3, 4, 10]));

      state.remove_confirmed(20);

      let expected: Vec<common::pb::MeasurementReq> = Vec::new();

      assert_eq!(state.measurements, expected);
   }




   #[test]
   fn test_remove_confirmed_removes_only_first_if_ts_is_min() {
      let measurement = measurement();
      let mut state = create_state(populate_measurements(&measurement, &[1, 2, 3, 4, 10]));

      state.remove_confirmed(1);

      let expected = populate_measurements(&measurement, &[2, 3, 4, 10]);

      assert_eq!(state.measurements, expected);
   }
   #[test]
   fn test_remove_confirmed_keeps_all_elements_if_ts_is_strictly_smaller_than_min() {
      let measurement = measurement();
      let mut state = create_state(populate_measurements(&measurement, &[1, 2, 3, 4, 10]));

      state.remove_confirmed(0);

      let expected = populate_measurements(&measurement, &[1, 2, 3, 4, 10]);

      assert_eq!(state.measurements, expected);
   }



   #[test]
   fn test_remove_confirmed_if_measurements_is_empty() {
      let mut state = create_state(Vec::new());

      state.remove_confirmed(1);

      let expected: Vec<common::pb::MeasurementReq> = Vec::new();

      assert_eq!(state.measurements, expected);
   }
}
