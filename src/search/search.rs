use std::collections::BTreeMap;

use axum::Json;
use serde_json::Value;
use levenshtein;
use crate::db_client::db_client;

pub async fn search(keyword:Json<String>)->Json<Value>{
let keyword=keyword.0;


let g= db_client().await;
let mut location:Vec<String>=Vec::new();
let mut model:Vec<String>=Vec::new();
let mut description:Vec<String>=Vec::new();
for row in g.query("SELECT * FROM car",&[]).await.unwrap_or_else(|_| panic!("Error on query")){
location.push(row.get::<_, String>("location"));
model.push(row.get::<_, String>("model"));
description.push(row.get::<_, String>("description"));
}
let mut name_result:Vec<String>=Vec::new();
let mut model_result:Vec<String>=Vec::new();
let mut description_result:Vec<String>=Vec::new();
for i in 0..location.len(){
    if levenshtein::levenshtein(&location[i],&keyword)<=2{
        name_result.push(location[i].clone());
    }
    if levenshtein::levenshtein(&model[i],&keyword)<=2{
        model_result.push(model[i].clone());
    }
    if levenshtein::levenshtein(&description[i],&keyword)<=2{
        description_result.push(description[i].clone());
    }
}
let mut result:BTreeMap<String,String>=BTreeMap::new();
for i in 0..name_result.len(){
    result.insert(name_result[i].clone(),"location".to_string());
}
for i in 0..model_result.len(){
    result.insert(model_result[i].clone(),"model".to_string());
}
for i in 0..description_result.len(){
    result.insert(description_result[i].clone(),"description".to_string());
}
Json(serde_json::json!(result))
}