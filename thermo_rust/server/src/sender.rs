use anyhow::{anyhow, Context, Result};

pub trait Sender: Send + Sync {
   fn send_with_pic(&self, text: &str, pic: Vec<u8>) -> Result<(), String>;
   fn send_text(&self, text: String, is_markdown: bool) -> Result<(), String>;

   // fn send(&self, text: &str, is_markdown: bool, pic: Option<Vec<u8>>) -> Result<()>;
}


pub struct TelegramSender {
   pub chat_id: i64,
   pub bot_id:  String,
}

impl crate::sender::Sender for TelegramSender {
   fn send_with_pic(&self, text: &str, pic: Vec<u8>) -> Result<(), String> { Ok(()) }

   fn send_text(&self, mut text: String, is_markdown: bool) -> Result<(), String> {
      println!("text: {}, is_markdown: {}", text, is_markdown);
      let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_id);
      if is_markdown {
         text = text.replace(".", "\\.").replace("-", "\\-").replace("`", "\\`");
      };
      let mut data = serde_json::json!({
          "chat_id": self.chat_id,
          "text": text,
      });

      if is_markdown {
         data["parse_mode"] = serde_json::json!("MarkdownV2");
      }
      let client = reqwest::Client::new();
      let result = client.post(&url).json(&data).send();
      println!("Result: {:?}", result);



      Ok(())
   }
}
