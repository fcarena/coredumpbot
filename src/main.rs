extern crate telegram_bot;
extern crate hyper;
extern crate rustc_serialize;

use telegram_bot::{Api, ListeningMethod, MessageType, ListeningAction};
use hyper::{Client};
use rustc_serialize::json::Json;
use std::io::{Read};

fn main() {
    loop {
        // Create bot, test simple API call and print bot information
        let api = Api::from_env("TELEGRAM_BOT_TOKEN").unwrap();
        println!("getMe: {:?}", api.get_me());
        let mut listener = api.listener(ListeningMethod::LongPoll(None));
        
        let path_to_picture :String = "rust-logo-blk.png".to_string();
        let caption_to_picture :String = "Rust Logo".to_string();

        // Fetch new updates via long poll method
        let res = listener.listen(|u| {
            // If the received update contains a message...
            if let Some(m) = u.message {
                let name = m.from.first_name;

                // Match message type
                match m.msg {
                    MessageType::Text(t) => {
                        // Print received text message to stdout
                        println!("<{}> {}", name, t);
                        let t = t.replace("@CoreDumpBot", "");
                        let ts:&str = t.trim();
                        
                        match ts {
                        "/getPicture" | "/get_picture" => { 
                        try!(api.send_photo(
                                    m.chat.id(),
                                    path_to_picture.clone(), // Path
                                    Some(caption_to_picture.clone()), // caption
                                    None, // reply_to_message_id
                                    None  // reply_markup
                            ));
                        },
                        "/help" => {
                            try!(api.send_message(
                                    m.chat.id(),
                                    format!("No such help 😜\nuse /getPicture for a snapshot of the 3d printer.\nuse /crowd for an update on people now present"),
                                    None, None, None
                            ));
                        },
                        "/crowd"| "/status" => {
                            let s = match fetch_people_now_present() {
                            Ok(people_now_present) if people_now_present > 1 =>  format!("Coredump is open\n{} people are present!", people_now_present),
                            Ok(people_now_present) if people_now_present == 1 => format!("Coredump is open\nOne person is present!"),
                            Ok(_) => format!("Coredump is closed\nobody here right now."),
                            Err(e) => format!("An error occured 😕\n{}", e),
                            };
                            try!(api.send_message(
                                    m.chat.id(),
                                    s,
                                    None, None, None
                            ));
                        },
                        "/start" => {
                            try!(api.send_message(
                                    m.chat.id(),
                                    format!("Welcome to CoredumpBot\nuse /getPicture for a snapshot of the 3d printer."),
                                    None, None, None
                            ));
                        },
                        _ => { /* ignore */ }, 
                        }
                    },
                    _ => {
                        try!(
                            api.send_message(
                                m.chat.id(),
                                format!("Unknown Command ..."),
                                None, None, None)
                        );
                    }
                }
            }

            // If none of the "try!" statements returned an error: It's Ok!
            Ok(ListeningAction::Continue)
        });

        if let Err(e) = res {
            println!("An error occured: {}", e);
            // Rest for 10 Seconds
            std::thread::sleep_ms(10 * 1000);
        }
    }
}

fn fetch_people_now_present() -> std::result::Result<i64, String> {
  let client = Client::new();

  match client.get("https://status.coredump.ch/").send() {
    Err(e) => Err(format!("client.get() error:\n{}", e)),
    Ok(mut res) => {
      
      let mut body = String::new();
      match res.read_to_string(&mut body) {
        Err(e) => { Err(format!("unable to connect to server, try again later:\n{}\n{}", e, body)) },
        Ok(_/*len*/) => {
          
          match Json::from_str( &*body ) {
            Err(e) => Err(format!("unable to parse server response: {:?}", e)),
            Ok(data) => {
              
              match data.as_object() {
                None => Err(format!("response must be a Json Object!")),
                Some(obj) => {
                  
                  match obj.get("sensors") {
                    None => Err(format!("response contains no sensors")),
                    Some(sensors) => {
                      match sensors.as_object() {
                        None => Err(format!("response.sensors must be an Object")),
                        Some(sensors) => {
                          match sensors.get("people_now_present") {
                            None => Err(format!("response contains no sensors.people_now_present")),
                            Some(people_now_present) => match people_now_present.as_array() {
                              None => Err(format!("response.sensors.people_now_present is not an Array")),
                              Some(people_now_present) => {
                                
                                match people_now_present[0].as_object() {
                                  None => Err(format!("response.sensors.people_now_present[0] is not an Object")),
                                  Some(people_now_present) => {
                                    
                                    match people_now_present.get("value") {
                                      None => Err(format!("response.sensors.people_now_present[0] has no Member calles 'value'")),
                                      Some(people_now_present) =>
                                        
                                        match people_now_present.as_i64() {
                                          None => Err(format!("response.sensors.people_now_present[0].value is no Integer")),
                                          Some(people_now_present) => Ok(people_now_present)
                                        }
                                    }
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
