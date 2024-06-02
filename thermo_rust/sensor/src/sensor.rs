use anyhow::{anyhow, Context, Result};
// use std::io::Read;

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




fn parse(data: &str) -> Result<f64> {
   const MAX: usize = 2 * 1024;
   let second_line = {
      let data = &data[..std::cmp::min(MAX, data.len())];
      let start = data.find('\n')
                      .with_context(|| anyhow!("Failed to find start of second line in: {}", &data))?;
      let data = &data[start + 1..];
      let end = data.find('\n').unwrap_or(data.len());
      &data[..end]
   };
   let temperature = {
      let start = &second_line.find("t=").with_context(|| anyhow!("Failed to find t= in: {second_line}"))?;
      let temp = &second_line[start + 2..];
      let temp: i32 =
         temp.parse().with_context(|| anyhow!("Failed to parse temperature as integer: {temp}"))?;
      temp as f64 / 1000.0
   };
   Ok(temperature)
}


// fn read_from_file(file_path: &std::path::Path, max_size:usize) -> Result<String> {
//    let mut file = File::open(file_path)?;
//    let mut buffer = vec![0; max_size];
//    let bytes_read = file.read(mut buffer)?;
//    buffer.truncate(bytes_read);
//    let result= String::from_utf8(buffer).expect("Invalid UTF-8 data");
//    Ok(result)

// let message: String = std::fs::read_to_string(file_path)?;
// Ok(message)
// }
// fn main() {
//    let path = std::path::PathBuf::from("1.txt");
//    let text = read_from_file(&path, 2*1024)?;
//    let temperature = parse(&text);
// }


#[cfg(test)]
mod tests {
   use super::*;
   use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

   #[derive(Debug)]
   struct OkParseTC {
      data:     String,
      expected: f64,
   }

   #[test]
   fn test_parse_returns_ok() {
      let test_cases = [
         //
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=18375"), expected: 18.375, },      // normal
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=0375"), expected: 0.375, },        // leading zeros 0100 => 0.1
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=-18375"), expected: -18.375, },    // negative
         OkParseTC { data:   String::from("26: crc=64 YES\n 26 t=18375\n 26"), expected: 18.375, }, // more than 2 lines, 2nd line has correct data
         //
      ];

      for (i, tc) in test_cases.iter().enumerate() {
         let res = parse(&tc.data);
         assert!(res.is_ok());
         assert_eq!(res.unwrap(), tc.expected, "Test-case #{i}: {tc:?}");
      }
   }

   struct ErrorParseTC {
      data: String,
   }

   #[test]
   fn test_parse_returns_error() {
      let test_cases = [
         //
         ErrorParseTC { data: String::new() },                                 // empty
         ErrorParseTC { data: String::from("26: crc=64 YES"), },               // one line
         ErrorParseTC { data: String::from("26: crc=64 YES t=1\n18325"), },    // t= in 1st line only
         ErrorParseTC { data: String::from("26: crc=64 YES\n 26 t=375ABC"), }, // letters after number
         ErrorParseTC { data: String::from("26: crc=64 YES\n 26 t=ABC375"), }, // letters before number
         ErrorParseTC { data: String::from("26: crc=64 YES\n 26 t=1 t=2"), },  // multiple t=
         //
      ];

      for tc in test_cases {
         let res = parse(&tc.data);
         println!("{res:?}");
         assert!(res.is_err());
      }
   }


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
