#[derive(Debug, serde::Serialize, Default)]
pub struct Item {
   pub NameToTemp: std::collections::HashMap<String, sensors::TempType>,
   pub ErrorString: String,
   // pub Time: i64,
}

pub fn to_json(item: &Item) -> String {
   serde_json::to_string(&item)
      .expect("Assume that Item must be always possible to serealise to JSON")
}

pub trait Sink {
   fn publish(&mut self, item: Item);
}

pub struct StdOutSink;
impl Sink for StdOutSink {
   fn publish(&mut self, item: Item) {
      println!("{item:?}");
   }
}

struct FakeSink {
   items: Vec<Item>,
}
impl FakeSink {
   pub fn new() -> Self {
      FakeSink { items: Vec::new() }
   }
}
impl Sink for FakeSink {
   fn publish(&mut self, item: Item) {
      self.items.push(item);
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn to_json() {
      let mut NameToTemp = std::collections::HashMap::new();
      NameToTemp.insert(String::from("Blue"), 10.);
      let ErrorString = String::new();
      let result = super::to_json(&Item {
         NameToTemp,
         ErrorString,
      });
      assert!(result.contains("Blue"));
      assert!(result.contains("NameToTemp"));
      assert!(result.contains("10."));
   }
}
