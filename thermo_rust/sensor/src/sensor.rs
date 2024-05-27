use anyhow::{anyhow, Context, Result};
use std::io::Read;

fn get_temp() -> Result<(f64, f64)> {
   let mut res = (0.0, 0.0);
   let sens1 = std::path::PathBuf::from("/sys/bus/w1/devices/28-000005eaddc2/w1_slave");
   let sens2 = std::path::PathBuf::from("/sys/bus/w1/devices/28-000005eac50a/w1_slave");

   for path in [sens1, sens2] {
      let mut read_file = std::fs::File::open(path)?;
      let mut contents = String::new();
      read_file.read_to_string(&mut contents);
      let lines: Vec<&str> = contents.lines().collect();
      let start_index = lines[1].find("t=").unwrap();
      let temp = &lines[1][start_index + 2..];
      if res.0 == 0.0 {
         res.0 = temp.parse().unwrap();
      } else {
         res.1 = temp.parse().unwrap();
      }
   }
   Ok(res)

}
