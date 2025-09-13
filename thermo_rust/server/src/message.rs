use anyhow::{Context, Result, anyhow};

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

#[derive(clap::Parser, Debug, Clone)]
pub struct TelegramArgs {
   #[arg(long)]
   tg_bot_id: String,

   #[arg(long)]
   tg_chat_id: String,
}

#[derive(Debug, Clone)]
pub struct Telegram {
   pub chat_id: String,
   pub bot_id: String,
}


impl Telegram {
   pub fn from_args(args: TelegramArgs) -> Self {
      Self {
         bot_id: args.tg_bot_id,
         chat_id: args.tg_chat_id,
      }
   }
   // pub fn new(bot_id: String, chat_id: String) -> Self { Self { bot_id, chat_id } }


   async fn try_sending(&self, len: usize, get_req: impl Fn() -> reqwest::Request) -> Result<()> {
      let mut descriptions = Vec::new();
      for i in 0..3 {
         let request = get_req();
         if i == 0 {
            log::info!("Sending request of size: {len}, request headers: {:?}", request.headers());
         }
         let request_headers = request.headers().clone();
         let resp = reqwest::Client::new().execute(request).await;
         match resp {
            Err(why) => {
               descriptions.push(format!(
                  ", Attempt: {i}, failed to send request of size: {len}, request headers: {:?}: {why:?}",
                  request_headers
               ));
               tokio::time::sleep(std::time::Duration::from_millis(100)).await;
               continue;
            }
            Ok(resp) => match handle_response(resp.text().await) {
               Err(why) => {
                  descriptions.push(format!(
                     "Attempt: {i}, failed to handle response of request of size: {len}, request headers: {:?}: {why:?}",
                     request_headers));
                  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                  continue;
               }
               Ok(_) => {
                  return Ok(());
               }
            },
         }
      }
      Err(anyhow!(descriptions.join("\n")))
   }

   pub async fn send_with_pic(&self, text: &str, pic: Vec<u8>) -> Result<()> {
      let url = format!("https://api.telegram.org/bot{}/sendPhoto", self.bot_id);

      let len = pic.len();

      let get_req = || {
         let headers = reqwest::header::HeaderMap::from_iter([(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("image/png"),
         )]);
         let pic = reqwest::multipart::Part::bytes(pic.clone())
            .mime_str("image/png")
            .unwrap()
            .file_name("file.png")
            .headers(headers);
         let form = reqwest::multipart::Form::new()
            .part("chat_id", reqwest::multipart::Part::text(self.chat_id.to_string()))
            .part("caption", reqwest::multipart::Part::text(text.to_string()))
            .part("photo", pic);
         let request = reqwest::Client::new()
            .post(&url)
            .multipart(form)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
         request
      };

      self.try_sending(len, get_req).await
   }

   pub async fn send_text(&self, mut text: String, is_markdown: bool) -> Result<()> {
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
      let request = reqwest::Client::new()
         .post(&url)
         .json(&data)
         .timeout(std::time::Duration::from_secs(10))
         .build()?;
      let body_len = request
         .body()
         .map(|b| b.as_bytes().map(|b| b.len()).unwrap_or_default())
         .unwrap_or_default();

      self.try_sending(body_len, || request.try_clone().unwrap()).await
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
      let sender: Telegram = Telegram {
         chat_id: "-4609542105".to_string(),
         bot_id: "7575784506:AAFIFywDLlLNtIR6qBPY6m9E4z7KBdTfx3c".to_string(),
      };
      let text: String = String::from("Hello Test");
      let result = sender.send_text(text, false).await;
      assert!(result.is_ok());
   }

   #[ignore]
   #[tokio::test]
   async fn test_send_text_with_markdown() {
      let sender: Telegram = Telegram {
         chat_id: "-4609542105".to_string(),
         bot_id: "7575784506:AAFIFywDLlLNtIR6qBPY6m9E4z7KBdTfx3c".to_string(),
      };
      let text = String::from(
         "Authenticating has not been implemented yet, so insert your chat id into Google BigQuery manually by issuing:\n\n\
                                          ```\n\
                                          MERGE INTO {self.table} AS Dst\n\
                                          ```\n at https://console.cloudgoogle.com/bigquery",
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
      let sender: Telegram = Telegram {
         chat_id: "-4609542105".to_string(),
         bot_id: "7575784506:AAFIFywDLlLNtIR6qBPY6m9E4z7KBdTfx3c".to_string(),
      };
      let result = sender.send_with_pic(text, pic).await;
      log::info!("result: {result:?}");
      // assert!(result.is_ok());
   }
}
