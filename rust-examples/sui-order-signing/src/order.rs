use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use blake2b_simd::Params;

#[derive(Debug, Clone)]
pub struct Order {
    pub market: String,
    pub price: u128,
    pub isBuy: bool,
    pub reduceOnly: bool,
    pub quantity: u128,
    pub postOnly: bool,
    pub orderbookOnly: bool,
    pub leverage: u128,
    pub expiration: u128,
    pub salt: u128,
    pub maker: String,
    pub ioc: bool,
    pub orderType: String,
    pub timeInForce: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderJSONRequest {
    pub orderbookOnly: bool,
    pub symbol: String,
    pub price: String,
    pub quantity: String,
    pub triggerPrice: String,
    pub leverage: String,
    pub userAddress: String,
    pub orderType: String,
    pub side: String,
    pub reduceOnly: bool,
    pub salt: u128,
    pub expiration: u128,
    pub orderSignature: String,
    pub timeInForce: String,
    pub postOnly: bool,
    pub cancelOnRevert: bool,
    pub clientId: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderCancellationJSONRequest {
    pub symbol: String,
    pub orderHashes: [String; 1],
    pub parentAddress: String,
    pub cancelSignature: String,
}


/**
 * Encodes order flags and returns a 16 bit hex
 */
pub fn get_order_flags (order: &Order) -> u32{
    
    let mut flag = 0;

    if order.ioc {
        flag += 1;
    };
    if order.postOnly{
        flag += 2;
    }
    if order.reduceOnly{
        flag += 4;
    }
    if order.isBuy{
        flag += 8
    }
    if order.orderbookOnly{
        flag += 16
    }

    return flag;
}

/**
 * POSTS the Cancellation Order
 */
pub async fn post_cancel_order(order_cancel: OrderCancellationJSONRequest, jwt_token: &str) -> String {
        
    // POST Request and obtain JWT Token
    let client = reqwest::Client::new();
    let res = client.delete("https://dapi.api.sui-staging.bluefin.io/orders/hash")
        .header("Authorization", "Bearer ".to_owned() + &jwt_token.to_owned())
        .json(&order_cancel)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    
    return res.to_string();
}

/**
 * Given an order hash, returns a cancel order hash
 */
pub fn create_signed_cancel_orders(order_hash : &str) -> blake2b_simd::Hash{
    let mut msg_dict = HashMap::new();
    msg_dict.insert("orderHashes", [order_hash]);

    let msg_str = serde_json::to_string(&msg_dict).unwrap();
    let mut intent: Vec<u8> = vec![3, 0, 0, msg_str.len() as u8];
    intent.extend_from_slice(msg_str.as_bytes());

    let hash = Params::new()
        .hash_length(32)
        .to_state()
        .update(&intent)
        .finalize();
    return hash;
}


/**
 * POSTS the Order
 */
pub async fn post_signed_order(order: &Order, order_hash_sig:String, jwt_token: &str) -> String {
    // POST Request and obtain JWT Token
    let order_request = OrderJSONRequest{
        orderbookOnly: order.orderbookOnly,
        symbol: order.market.to_string().into(),
        price: order.price.to_string().into(),
        quantity: order.quantity.to_string().into(),
        triggerPrice: "0".to_string().into(),
        leverage: order.leverage.to_string().into(),
        userAddress: order.maker.to_string().into(),
        orderType: order.orderType.to_string().into(),
        side: if order.isBuy == true {"BUY".to_string().into()} else {"SELL".to_string().into()},
        reduceOnly: order.reduceOnly,
        salt: order.salt,
        expiration: order.expiration,
        orderSignature: order_hash_sig,
        timeInForce: order.timeInForce.to_string().into(),
        postOnly: order.postOnly,
        cancelOnRevert: false,
        clientId: "bluefin-v2-client-python".to_string().into(),
    };
    
    let client = reqwest::Client::new();
    let res = client.post("https://dapi.api.sui-staging.bluefin.io/orders")
        .header("Authorization", "Bearer ".to_owned() + &jwt_token.to_owned()) 
        .json(&order_request)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    
    println!("{}", res);
    
    let v: Value = serde_json::from_str(&res).expect("JSON Decoding failed");
    let hash : &str = v["hash"].as_str().unwrap();
    return hash.to_string();
}


/**
 * Given a market ("ETH-PERP" or "BTC-PERP"), returns the perpetual address
 */
pub async fn get_market_id (market: &str) -> String{
    let client = reqwest::Client::new();
    let res = client.get("https://dapi.api.sui-staging.bluefin.io/meta?symbol=".to_owned() + &market.to_string().to_owned() )
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    
    let v: Value = serde_json::from_str(&res).expect("JSON Decoding failed");
    let v1: Value = serde_json::from_str(&v["perpetualAddress"].to_string()).expect("JSON Decoding failed");
    let market_id_value: Value = serde_json::from_str(&v1["id"].to_string()).expect("JSON Decoding failed");
    let market_id =  market_id_value.as_str().unwrap();

    return market_id.to_string();
}

/**
 * Given an order, returns hash of the order
 */
pub async fn get_serialized_order(order: &Order) -> String {

    let flags = get_order_flags(&order);
    let flags_array = format!("{:0>2x}", flags);

    let order_price_hex = format!("{:0>32x}", order.price);
    let order_quantity_hex = format!("{:0>32x}", order.quantity);
    let order_leverage_hex = format!("{:0>32x}", order.leverage);
    let order_salt = format!("{:0>32x}", order.salt);
    let order_expiration = format!("{:0>16x}", order.expiration);
    let order_maker = &order.maker;
    let order_market = get_market_id(&order.market).await;
    let bluefin_string = hex::encode("Bluefin");

    let order_buffer = order_price_hex 
        + &order_quantity_hex 
        + &order_leverage_hex 
        + &order_salt
        + &order_expiration
        + &order_maker[2..]
        + &order_market[2..]
        + &flags_array
        + &bluefin_string;

    return order_buffer;
}