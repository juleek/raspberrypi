use crate::sensor;

pub trait Entity {
   fn take(&self, data: sensor::Measurement);
}

pub struct Alerting {}
impl Entity for Alerting {
   fn take(&self, _data: sensor::Measurement) {}
}

pub struct DataBase {}
impl Entity for DataBase {
   fn take(&self, _data: sensor::Measurement) {}
}

pub fn f(data: sensor::Measurement, entities: Vec<Box<dyn Entity>>) {
   for entity in entities {
      entity.take(data.clone());
   }
}
