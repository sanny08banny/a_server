use base64::{self, Engine};

use crate::{r::r_tokens::R, ecryption_engine::CUSTOM_ENGINE};
struct Client{
    user_name:String,
    phone_number:String,
    email:String,
    password:String,
    tokens:R,
    client_id:String,
    verified:bool
}

impl Client{
   pub fn new(user_name:String,phone_number:String,email:String,password:String)->Self{
    let input=format!("{}{}{}{}",user_name,phone_number,email,password);
    let c=CUSTOM_ENGINE.encode(input);
    Client { user_name, phone_number, email, password, 
        tokens: R::new(c.clone()), 
        client_id: c,
        verified: false }
    }
}