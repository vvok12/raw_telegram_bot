use std::{str::FromStr, collections::HashMap, thread, time, fs};

use serde_json::value::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tocken = &fs::read_to_string("../raw_telegram_bot_tocken.txt").unwrap();
    let mut offset = 738654966u64;

    loop {

        let updates = get_updates(tocken, &offset.to_string());

        if !updates.is_empty() {
            for update in updates {
                if let Some(message) = update.maybe_message() {
                    push_ok(tocken, &message);
                }
                
                let last_update_id = update.id() + 1;
                if last_update_id > offset {
                    offset = last_update_id;
                }
            }
        } else {
            println!("no new data found");
        }

        
        thread::sleep(time::Duration::from_secs(2));
    }

    //Ok(())
}

fn push_ok(tocken: &str, message:&UpdateMessage) {
    //let link = format!("https://api.telegram.org/bot{tocken}/sendMessage?chat_id={}&text=OK&reply_to_message_id={}", message.chat_id(), message.id());
    let mut params = HashMap::new();
    params.insert("chat_id", message.chat_id().to_string());
    params.insert("text", "Thanx!".to_string());
    params.insert("reply_to_message_id", message.id().to_string());

    let _ = reqwest::blocking::Client::new()
        .post(format!("https://api.telegram.org/bot{tocken}/sendMessage"))
        .form(&params)
        .send();
    //let _ = reqwest::blocking::get(link);
}

fn get_updates<'a>(tocken: &'a str, offset: &'a str) -> Vec<TelegramDataUpdate> {
    let link = format!("https://api.telegram.org/bot{tocken}/getUpdates?offset={offset}");
    let resp = reqwest::blocking::get(link).unwrap().json::<Value>().unwrap();

    extract_json_updates_data(resp)
        .into_iter()
        .map(|val| TelegramDataUpdate(val))
        .collect::<Vec<TelegramDataUpdate>>()
}

fn extract_json_updates_data(resp: Value) -> Vec<Value> {
    if let Value::Bool(is_ok) = resp.get("ok").unwrap() {
        if is_ok == &true {
            
            if let Value::Array(updates) = resp.get("result").unwrap() {
                return updates.clone();
            }

        }
    }

    return Vec::new();
}

#[derive(Debug)]
struct TelegramDataUpdate(Value);

impl TelegramDataUpdate {
    fn id(&self)-> u64 {
        self.0.get("update_id").unwrap().as_u64().unwrap()
    }

    fn maybe_message(&self) -> Option<UpdateMessage> {
        
        let mess = {
            let m = self.0.get("message");
            let em = self.0.get("edited_message");

            m.or(em)
        };
        
        match mess {
            Some(mess) => Some(UpdateMessage(mess)),
            None => None
        }
    }
}

#[derive(Debug)]
struct UpdateMessage<'a>(&'a Value);

impl<'a> UpdateMessage<'a> {
    
    fn chat_type(&self) -> &'a str {
        self.0.get("chat").unwrap()
            .get("type").unwrap().as_str().unwrap()
    }

    fn chat_id(&self) -> i64 {
        self.0.get("chat").unwrap()
            .get("id").unwrap().as_i64().unwrap()
    }

    fn id(&self) -> u64 {
        self.0.get("message_id").unwrap().as_u64().unwrap()
    }

    fn maybe_text(&self) -> Option<String> {
        self.0.get("text").and_then(|t| t.as_str().and_then(|s| Some(String::from_str(s).unwrap())))
    }
}