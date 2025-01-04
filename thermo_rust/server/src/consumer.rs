pub trait Consumer {
   fn consume(&self, measurement: helpers::helpers::Measurement);
}
