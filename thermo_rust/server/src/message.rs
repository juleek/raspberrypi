use anyhow::{anyhow, Context, Result};

#[async_trait::async_trait]
pub trait Sender: Send {
   async fn send_with_pic(&self, text: &str, pic: Vec<u8>) -> Result<()>;
   async fn send_text(&self, text: String, is_markdown: bool) -> Result<()>;
}




pub struct Telegram {
   pub chat_id: i64,
   pub bot_id:  String,
}

fn handle_response(body: Result<String, reqwest::Error>) -> Result<()> {
   let body = body.with_context(|| anyhow!("Failed to obtain body"))?;
   let json: serde_json::Value =
      serde_json::from_str(&body).with_context(|| anyhow!("Failed to parse the body: {body}"))?;

   if let Some(ok) = json.get("ok").and_then(|v| v.as_bool()) {
      if ok {
         Ok(())
      } else {
         Err(anyhow!("ok exists but is false: {body}"))
      }
   } else {
      Err(anyhow!("ok is not in json body: {body}"))
   }
}

impl Telegram {
   async fn try_sending(&self, get_req: impl Fn() -> (reqwest::Request, usize)) -> Result<()> {
      let (mut request, body_len) = get_req();
      log::info!("Sending request of size: {body_len}, request headers: {:?}", request.headers());
      let mut error_description = String::new();
      for i in 0..3 {
         let request_headers = request.headers().clone();
         let resp = reqwest::Client::new().execute(request).await;
         match resp {
            Err(why) => {
               error_description.push_str(&format!(", Attempt: {i}, failed to send request of size: {body_len}, request headers: {:?}: {why:?}", request_headers));
               request = get_req().0;
               continue;
            }
            Ok(resp) => match handle_response(resp.text().await) {
               Err(why) => {
                  error_description.push_str(&format!(
                     "Attempt: {i}, failed to handle response of request of size: {body_len}, request headers: {:?}: {why:?}",
                     request_headers));
                  request = get_req().0;
                  continue;
               }
               Ok(_) => {
                  return Ok(());
               }
            },
         }
      }
      Err(anyhow!(error_description))
   }
}

#[async_trait::async_trait]
impl Sender for Telegram {
   async fn send_with_pic(&self, text: &str, pic: Vec<u8>) -> Result<()> {
      let url = format!("https://api.telegram.org/bot{}/sendPhoto", self.bot_id);

      let get_req = || {
         let len = pic.len();



         // let headers = http::HeaderMap::from_iter([(http::header::CONTENT_TYPE,
         //                                            http::HeaderValue::from_static("image/png"))]);
         // headers.insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("image/png"));
         // let pic = reqwest::multipart::Part::bytes(pic.clone()).mime_str("image/png")
         //                                                       .unwrap()
         //                                                       .file_name("file.png")
         //                                                       .headers(headers);

         let headers =
            reqwest::header::HeaderMap::from_iter([(reqwest::header::CONTENT_TYPE,
                                                    reqwest::header::HeaderValue::from_static("image/png"))]);
         let pic = reqwest::multipart::Part::bytes(pic.clone()).mime_str("image/png")
                                                               .unwrap()
                                                               .file_name("file.png")
                                                               .headers(headers);
         let form =
            reqwest::multipart::Form::new().part("chat_id",
                                                 reqwest::multipart::Part::text(self.chat_id.to_string()))
                                           .part("caption", reqwest::multipart::Part::text(text.to_string()))
                                           .part("photo", pic);
         let request = reqwest::Client::new().post(&url)
                                             .multipart(form)
                                             .timeout(std::time::Duration::from_secs(10))
                                             .build()
                                             .unwrap();
         (request, len)
      };

      self.try_sending(get_req).await
   }

   async fn send_text(&self, mut text: String, is_markdown: bool) -> Result<()> {
      let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_id);
      if is_markdown {
         text = text.replace('.', "\\.");
      }
      let mut data = serde_json::json!({
          "chat_id": self.chat_id,
          "text": text,
      });
      if is_markdown {
         data["parse_mode"] = serde_json::json!("MarkdownV2");
      }
      let request = reqwest::Client::new().post(&url)
                                          .json(&data)
                                          .timeout(std::time::Duration::from_secs(10))
                                          .build()?;
      let body_len = request.body()
                            .map(|b| b.as_bytes().map(|b| b.len()).unwrap_or_default())
                            .unwrap_or_default();

      self.try_sending(|| (request.try_clone().unwrap(), body_len)).await
   }
}


//
// ===========================================================================================================
// Tests


#[cfg(test)]
mod tests {
   use super::*;
   // use pretty_assertions::assert_eq;

   // static TELEGRAM: Telegram = once_cell::sync::Lazy::new(|| Telegram {chat_id: -4609542105, bot_id: "7575784506:AAFIFywDLlLNtIR6qBPY6m9E4z7KBdTfx3c".to_string()});

   #[ignore]
   #[tokio::test]
   async fn test_send_text_without_markdown() {
      let sender: Telegram = Telegram { chat_id: -4609542105,
                                        bot_id:  "7575784506:AAFIFywDLlLNtIR6qBPY6m9E4z7KBdTfx3c".to_string(), };
      let text: String = String::from("Hello Test");
      let result = sender.send_text(text, false).await;
      assert!(result.is_ok());
   }

   #[ignore]
   #[tokio::test]
   async fn test_send_text_with_markdown() {
      let sender: Telegram = Telegram { chat_id: -4609542105,
                                        bot_id:  "7575784506:AAFIFywDLlLNtIR6qBPY6m9E4z7KBdTfx3c".to_string(), };
      let text = String::from(
                                          "Authenticating has not been implemented yet, so insert your chat id into Google BigQuery manually by issuing:\n\n\
                                          ```\n\
                                          MERGE INTO {self.table} AS Dst\n\
                                          ```\n at https://console.cloudgoogle.com/bigquery"
                                      );
      let result = sender.send_text(text, true).await;
      assert!(result.is_ok(), "{result:?}");
   }

   #[ignore]
   #[tokio::test]
   async fn test_send_with_pic() {
      common::init_logger("debug");
      let pic: Vec<u8> = std::fs::read("/home/yulia/devel/log.png").expect("Failed to read the image file");
      let text = "hello pic";
      let sender: Telegram = Telegram { chat_id: -4609542105,
                                        bot_id:  "7575784506:AAFIFywDLlLNtIR6qBPY6m9E4z7KBdTfx3c".to_string(), };
      let result = sender.send_with_pic(text, pic).await;
      log::info!("result: {result:?}");
      // assert!(result.is_ok());
   }
}
