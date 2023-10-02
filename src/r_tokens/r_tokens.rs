use chrono::{DateTime, Local};
struct R{
    user_id: String,
    token_value: f32,
    amount: f32,
    expiry_date: DateTime<Local>,
}
/*
in the database:::
 userId | amount | expiry_date |
NB::the saf api uses the callback url (this is where the code below should be)
  if transaction_is_successful{
 let token_value=150;
 let rs=amount/token_value;
 //*fetch the amount of r from the database */
 let new_r_amount =rs+fetched_amount;
 let new_expiry=...;
//^update the database with the new amount and expiry
//*  send a 200 Ok with the new amount */
 }
*/