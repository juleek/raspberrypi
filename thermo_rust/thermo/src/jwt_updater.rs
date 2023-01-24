use anyhow::{anyhow, Context, Result};
use crossbeam_channel as channel;
use stdext::function_name;

pub mod ReqResp {
   #[derive(Debug)]
   pub struct JWT(pub String);
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Claims<'a> {
   aud:             &'a str,
   target_audience: &'a str,
   sub:             &'a str,
   iss:             &'a str,
   iat:             u64,
   exp:             u64,
}

#[derive(serde::Deserialize)]
struct ExchangedJwt<'a> {
   id_token: &'a str, // Serde will use zero-copy parsing here! https://serde.rs/lifetimes.html
}

pub struct JwtUpdater {
   func_http_end_point: String,
   account_email:       String,
   private_key:         jsonwebtoken::EncodingKey,
   raid:                channel::Sender<ReqResp::JWT>,
   client:              reqwest::blocking::Client,
}

const URL: &str = "https://www.googleapis.com/oauth2/v4/token";

impl JwtUpdater {
   pub fn new(raid: channel::Sender<ReqResp::JWT>,
              func_http_end_point: &str,
              account_email: &str,
              private_key: &str)
              -> Self {
      JwtUpdater {
         func_http_end_point: func_http_end_point.to_owned(),
         account_email: account_email.to_owned(),
         private_key: jsonwebtoken::EncodingKey::from_rsa_pem(private_key.as_bytes())
            .expect("Must be possible to parse the key as PEM RSA in JwtUpdater"),
         raid,
         client: reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Must be possible to create HTTP Client"),
      }
   }

   pub fn start(self) { std::thread::spawn(move || self.event_loop()); }

   fn create_jwt(&self) -> String {
      let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                                                     .unwrap();
      let claims = Claims { aud:             URL,
                            target_audience: &self.func_http_end_point,
                            sub:             &self.account_email,
                            iss:             &self.account_email,
                            iat:             current_time.as_secs(),
                            exp:             (current_time
                                              + std::time::Duration::from_secs(10 * 60)).as_secs(), };
      let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
      jsonwebtoken::encode(&header, &claims, &self.private_key)
         .expect("Must be possible to encode a JWT")
   }

   fn try_to_parse_resp(&self, body: &bytes::Bytes) {
      let exchanged_jwt: ExchangedJwt = match serde_json::from_slice(&body) {
         Ok(v) => v,
         Err(ref why) => {
            println!("{}: Failed to deserialise response as json: {why:?}",
                     function_name!());
            return;
         }
      };
      let result = ReqResp::JWT(String::from(exchanged_jwt.id_token));
      // println!("{}: Parsed JWT: {result:?}", function_name!());
      let _ = self.raid.send(result);
   }

   fn event_loop(&self) {
      let mut counter: i32 = 0;
      loop {
         counter += 1;

         let signed_token = self.create_jwt();

         let mut params = std::collections::HashMap::new();
         params.insert("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer");
         params.insert("assertion", &signed_token);

         let req = self.client
                       .post(URL)
                       .header(reqwest::header::AUTHORIZATION,
                               format!("Bearer {signed_token}"))
                       .form(&params)
                       .build()
                       .expect("Must be possible to create a request");

         println!("{}: Making request : {req:#?},data: {:#?}",
                  function_name!(),
                  req.body());

         let resp = self.client.execute(req);

         let resp_for_logs = format!("{resp:?}");
         let mut body = bytes::Bytes::new();
         if let Ok(r) = resp {
            if let Ok(bytes) = r.bytes() {
               body = bytes;
            }
         }
         println!("{}: Got response: {resp_for_logs}, data: {:#?}",
                  function_name!(),
                  body);

         self.try_to_parse_resp(&body);

         std::thread::sleep(std::time::Duration::from_secs(10 * 60));
      }
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   const FUNC_HTTP_END_POINT: &str = "";
   const ACCOUNT_EMAIL: &str = "";
   const PRIVATE_KEY: &str = "";

   #[test]
   #[ignore = "Integration test: Uses real service, requires file"]
   fn test_updater() {
      let file: std::fs::File = std::fs::File::open("/home/dimanne/devel/thermo-app-priv.pem")
         .expect("Must be possible to read key with PEM RSA key for JWT");

      let buf_reader = std::io::BufReader::new(file);
      let contents = std::io::read_to_string(buf_reader)
         .expect("Must be possible to read file with private key in the test");

      let (tx, rx) = channel::bounded(100);
      let poller = JwtUpdater::new(tx, FUNC_HTTP_END_POINT, ACCOUNT_EMAIL, &contents);

      poller.start();

      for received in rx {
         println!("Got: {:?}", received);
      }
   }
}
