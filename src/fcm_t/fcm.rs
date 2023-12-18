use axum::Json;
use fcm;
use hyper::{client, StatusCode};
use serde_json::{json, Value};

pub async fn req_ride(det:Json<Value>)->Result<Json<Value>,StatusCode>
{
   let det=det.0;
   // deserializing the json data
   let client_id=det["client_id"].as_str().unwrap();
   let driver = det["driver_regId"].as_str().unwrap();
   // let phone = det["phone"].as_str().unwrap();
   let client = fcm::Client::new();
   let mut details=  json!(
       {
           "ride_id": "1",
           "user_name": "User 1",
           "user_phone": "0707676913",
           "client_id": client_id,
       }
   );

   let mut notification_builder = fcm::NotificationBuilder::new();
   notification_builder.title("Ride request!");
   notification_builder.body("User 1 has requested a ride!");
   notification_builder.tag("Ride req");
   notification_builder.icon("ic_launcher");


   let notification = notification_builder.finalize();
   let mut message_builder = fcm::MessageBuilder::new("AAAA1rzD5J4:APA91bEVDhK6QTL835XVuXPEEA0mbtV1q37zzZeTd0R7w2wHwyh-QyEjYP1CqZ2Jv6GKiSbuOrdLVi62TAThdyy4uPK4rYuphOLQPX_pfsx-l98jUmNPp6l_H7zCD_Jlq2i2-UZVlSXm",
   driver);
      //  "c4JWkJpESg6I1q2irDFAbQ:APA91bGm01z1FVqLvwKr9qhkFauhSlBsN7PNbwnuQ7hjL_-yWTNnBffB5vs-IHePAW9UMQ7KNXl3T4KxxPs2JYKPK8SOa51N9wXPsNHX_Zvm-fB62r0A91x8eCbkxrcWBj3KR0Y0QKpv");
   message_builder.notification(notification);
   message_builder.data(&mut details);

   let response = client.send(message_builder.finalize()).await.unwrap();
   println!("Sent: {:?}", response);  
   Ok(Json(json!({"status":"success"})))
}