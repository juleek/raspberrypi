pub mod cli;
pub mod message;
pub mod plot;
// pub mod alerting;
// pub mod grpc;
pub mod db;
pub mod sensor;


fn generate_random_id(prefix: &str, len: usize) -> String {
   use rand::Rng;
   let mut rng = rand::rng();
   let mut res = prefix.to_owned();
   for _ in 0..len {
      res.push(rng.sample(rand::distr::Alphanumeric) as char);
   }
   res
}
