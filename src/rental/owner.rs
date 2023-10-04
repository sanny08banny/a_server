use crate::r::r_tokens::R;

use super::car::Car;
struct Owner{
    user_name:String,
    phone_number:String,
    email:String,
    password:String,
    owner_id:String,
    verified:bool,
    tokens:R,
    cars:Vec<Car>
}