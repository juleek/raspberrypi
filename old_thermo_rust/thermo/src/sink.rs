
#[allow(non_snake_case)]
#[serde_with::serde_as]
#[derive(Debug, serde::Serialize)]
pub struct Item {
   pub NameToTemp:  std::collections::HashMap<String, sensors::TempType>,
   pub ErrorString: String,
   #[serde_as(as = "time::format_description::well_known::Iso8601")]
   pub Time:        time::OffsetDateTime,
}
impl Item {
   pub fn new_with_curr_time() -> Self {
      Item { NameToTemp:  Default::default(),
             ErrorString: Default::default(),
             Time:        time::OffsetDateTime::now_utc(), }
   }
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

#[derive(Debug, Default)]
pub struct FakeSink {
   pub items: Vec<Item>,
}
impl Sink for FakeSink {
   fn publish(&mut self, item: Item) { self.items.push(item); }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn to_json() {
      let mut NameToTemp = std::collections::HashMap::new();
      NameToTemp.insert(String::from("Blue"), 10.);
      let ErrorString = String::new();
      let result = super::to_json(&Item { NameToTemp,
                                          ErrorString,
                                          Time: time::OffsetDateTime::now_utc() });
      assert!(result.contains("Blue"));
      assert!(result.contains("NameToTemp"));
      assert!(result.contains("10."));
   }
}
