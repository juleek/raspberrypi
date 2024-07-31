

use anyhow::{anyhow, Context, Result};

fn read_exactly_ignoring_early_eof(reader: &mut impl std::io::Read, max_size: usize) -> Result<Vec<u8>> {
   let mut buffer = vec![0; max_size];
   let mut total_read = 0;
   while total_read < max_size {
      let bytes_read_res = reader.read(&mut buffer[total_read..]);
      // Failed to read bytes? => return Err()
      let bytes_read = bytes_read_res.with_context(|| {
                                        anyhow!("Successfully read: {total_read} bytes. Failed to read more.")
                                     })?;
      total_read += bytes_read;
      if bytes_read == 0 {
         // Read 0 bytes? => eof => return what has been read so far:
         break;
      }
   }
   buffer.truncate(total_read);
   Ok(buffer)
}
fn parse(reader: &mut impl std::io::Read) -> Result<f64> {
   const MAX: usize = 2 * 1024;
   let data = read_exactly_ignoring_early_eof(reader, MAX)?;

   let second_line = {
      let data = &data[..std::cmp::min(MAX, data.len())];
      let start =
         data.iter()
             .position(|x| *x == b'\n')
             .with_context(|| {
                anyhow!("Failed to find start of second line in: {:?}", std::str::from_utf8(data))
             })?;
      let data = &data[start + 1..];
      let end = data.iter().position(|x| *x == b'\n').unwrap_or(data.len());
      &data[..end]
   };
   let temperature = {
      let marker = b"t=";
      let start = second_line.windows(marker.len())
                             .position(|window| window == marker)
                             .with_context(|| anyhow!("Failed to find t= in: {:?}", std::str::from_utf8(second_line)))?;
      let temp = &second_line[start + marker.len()..];
      let temp = std::str::from_utf8(temp).with_context(|| anyhow!("Failed to convert {:?} to string", temp))?;
      let temp: i32 = temp.parse().with_context(|| anyhow!("Failed to parse {temp} as integer"))?;
      temp as f64 / 1000.0
   };
   Ok(temperature)
}


#[derive(Debug, Clone)]
pub struct Measurement {
   pub sensor:      String,
   pub temperature: Option<f64>,
   pub errors:      Vec<String>,
}

pub type Rx = tokio::sync::mpsc::Receiver<Measurement>;
pub type Tx = tokio::sync::mpsc::Sender<Measurement>;

impl From<Measurement> for agg_proto::Measurement {
   fn from(value: Measurement) -> Self {
       todo!()
   }
}

pub struct Sensor {
   pub name: String,
   pub path: String,
}


pub struct Waiter {
   start:    std::time::Instant,
   interval: std::time::Duration,
}

impl Waiter {
   fn new(interval: std::time::Duration) -> Self {
      Waiter { start: std::time::Instant::now(),
               interval }
   }
   fn wait(&mut self, stop_requested: &std::sync::Arc<std::sync::atomic::AtomicBool>) {
      let end = self.start + self.interval;
      while std::time::Instant::now() < end && !stop_requested.load(std::sync::atomic::Ordering::Relaxed) {
         std::thread::sleep(self.interval)
      }
      self.start = std::time::Instant::now();
   }
}

pub fn poll_sensor(tx: tokio::sync::mpsc::Sender<Measurement>,
                   sensor: Sensor,
                   stop_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,
                   interval: std::time::Duration) {
   let mut waiter = Waiter::new(interval);
   while !stop_requested.load(std::sync::atomic::Ordering::Relaxed) {
      let (temperature, error) = match std::fs::File::open(&sensor.path) {
         Ok(mut file) => match parse(&mut file) {
            Ok(temperature) => (Some(temperature), vec![]),
            Err(e) => (None, vec![format!("Failed to parse file: {}", e)]),
         },
         Err(e) => (None, vec![format!("Failed to open file: {}", e)]),
      };
      let measurement = Measurement { sensor: sensor.name.clone(),
                                      temperature,
                                      errors: error };
      tx.blocking_send(measurement.clone())
        .with_context(|| anyhow!("Failed to send measurement {:?} in channel", measurement))
        .unwrap();
      waiter.wait(&stop_requested);
   }
}




#[cfg(test)]
mod tests {
   use super::*;
   use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

   type Chunk = Vec<u8>;
   struct FakeRead {
      chunks: Vec<Chunk>,
   }
   impl FakeRead {
      pub fn from(chunks: Vec<Chunk>) -> Self { FakeRead { chunks } }
   }
   impl std::io::Read for FakeRead {
      fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
         let chunk = match self.chunks.first_mut() {
            None => return Ok(0),
            Some(x) => x,
         };

         let to_put = std::cmp::min(buf.len(), chunk.len());
         buf[..to_put].copy_from_slice(&chunk[..to_put]);
         chunk.drain(..to_put);
         if chunk.is_empty() {
            self.chunks.remove(0);
         }

         Ok(to_put)
      }
   }

   // --------------------------------------------------------------------------------------------------------
   // parse

   // #[derive(Debug)]
   struct OkParseTC {
      reader:   FakeRead,
      expected: f64,
   }

   #[test]
   fn test_parse_returns_ok() {
      let mut test_cases = [
         //
         OkParseTC{ reader: FakeRead::from(vec!["26: crc=64 YES\n 26 t=18375".as_bytes().to_vec()]), expected: 18.375},        // normal
         OkParseTC{ reader: FakeRead::from(vec!["26: crc=64 YES\n 26 t=0375".as_bytes().to_vec()]), expected: 0.375},          // leading zeros 0100 => 0.1
         OkParseTC{ reader: FakeRead::from(vec!["26: crc=64 YES\n 26 t=-18375".as_bytes().to_vec()]), expected: -18.375},      // negative
         OkParseTC{ reader: FakeRead::from(vec!["26: crc=64 YES\n 26 t=18375\n 26".as_bytes().to_vec()]), expected: 18.375},   // more than 2 lines, 2nd line has correct data
         //
      ];

      for (i, tc) in test_cases.iter_mut().enumerate() {
         let res = parse(&mut tc.reader);
         assert!(res.is_ok());
         assert_eq!(res.unwrap(), tc.expected, "Test-case #{i}");
      }
   }

   struct ErrorParseTC {
      reader: FakeRead,
   }

   #[test]
   fn test_parse_returns_error() {
      let mut test_cases = [
         //
         ErrorParseTC { reader: FakeRead::from(vec![String::new().as_bytes().to_vec()])},                   // empty
         ErrorParseTC { reader: FakeRead::from(vec!["26: crc=64 YES".as_bytes().to_vec()])},                // one line
         ErrorParseTC { reader: FakeRead::from(vec!["26: crc=64 YES t=1\n18325".as_bytes().to_vec()])},     // t= in 1st line only
         ErrorParseTC { reader: FakeRead::from(vec!["26: crc=64 YES\n 26 t=375ABC".as_bytes().to_vec()])},  // letters after number
         ErrorParseTC { reader: FakeRead::from(vec!["26: crc=64 YES\n 26 t=ABC375".as_bytes().to_vec()])},  // letters before number
         ErrorParseTC { reader: FakeRead::from(vec!["26: crc=64 YES\n 26 t=1 t=2".as_bytes().to_vec()])},   // multiple t=
         //
      ];

      for tc in test_cases.iter_mut() {
         let res = parse(&mut tc.reader);
         println!("{res:?}");
         assert!(res.is_err());
      }
   }


   // --------------------------------------------------------------------------------------------------------
   // read_exactly_ignoring_early_eof


   #[test]
   fn test_read_exactly_ignoring_early_eof_reader_has_less_than_buffer_size_single_chunk() -> Result<()> {
      let mut read = FakeRead::from(vec!["123".as_bytes().to_vec()]);
      let res = read_exactly_ignoring_early_eof(&mut read, 5)?;
      assert_eq!(res, "123".as_bytes());
      Ok(())
   }

   #[test]
   fn test_read_exactly_ignoring_early_eof_reader_has_less_than_buffer_size_multiple_chunks() -> Result<()> {
      let mut read = FakeRead::from(vec!["123".as_bytes().to_vec(), "3".as_bytes().to_vec()]);
      let res = read_exactly_ignoring_early_eof(&mut read, 5)?;
      assert_eq!(res, "1233".as_bytes());
      Ok(())
   }

   #[test]
   fn test_read_exactly_ignoring_early_eof_reader_has_more_than_buffer_size_single_chunk() -> Result<()> {
      let mut read = FakeRead::from(vec!["12345".as_bytes().to_vec()]);
      let res = read_exactly_ignoring_early_eof(&mut read, 2)?;
      assert_eq!(res, "12".as_bytes());
      Ok(())
   }

   #[test]
   fn test_read_exactly_ignoring_early_eof_reader_has_more_than_buffer_size_multiple_chunks() -> Result<()> {
      let mut read = FakeRead::from(vec!["1".as_bytes().to_vec(), "2345".as_bytes().to_vec()]);
      let res = read_exactly_ignoring_early_eof(&mut read, 2)?;
      assert_eq!(res, "12".as_bytes());
      Ok(())
   }

   #[test]
   fn test_read_exactly_ignoring_early_eof_reader_returns_buffer_size_single_chunk() -> Result<()> {
      let mut read = FakeRead::from(vec!["1234".as_bytes().to_vec()]);
      let res = read_exactly_ignoring_early_eof(&mut read, 4)?;
      assert_eq!(res, "1234".as_bytes());
      Ok(())
   }

   #[test]
   fn test_read_exactly_ignoring_early_eof_reader_returns_buffer_size_multiple_chunks() -> Result<()> {
      let mut read = FakeRead::from(vec!["123".as_bytes().to_vec(), "45".as_bytes().to_vec()]);
      let res = read_exactly_ignoring_early_eof(&mut read, 5)?;
      assert_eq!(res, "12345".as_bytes());
      Ok(())
   }
}

// println!()
