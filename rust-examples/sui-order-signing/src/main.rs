use ed25519_dalek::*;
use web3_unit_converter::Unit;
use blake2b_simd::Params;
use sha256::digest;
use std::time::Instant;

mod order;


#[tokio::main]
async fn main() {
    let wallet_key = "c501312ca9eb1aaac6344edbe160e41d3d8d79570e6440f2a84f7d9abf462270";

    // JWT Token obtained from onboarding signer corresponding to the same wallet
    let jwt_token = "<JWT from onboarding signer example>";

    // Market: ETH-PERP or BTC-PERP
    let market = "ETH-PERP";

    // Decode Private Key Hex String to Bytes
    let bytes = hex::decode(wallet_key).expect("Decoding failed");
    let mut private_key_bytes: [u8; 32] = [0; 32];
    private_key_bytes.copy_from_slice(&bytes[0..32]);

    // Convert to Signing Key
    let signingkey = SigningKey::from_bytes(&private_key_bytes);

    // Generate the corresponding public key
    let public_key: VerifyingKey = (&signingkey).into();
    // println!("Public key bytes: {:?}", public_key.to_bytes());

    // Generate the b64 of the public key
    let public_key_b64 = base64::encode(&public_key.to_bytes());
    // println!("Public Key Base64: {}", public_key_b64);

    // Append 0x00 to public key due to BIP32
    let public_key_array = public_key.to_bytes();
    let mut public_key_array_bip32 = [0; 33];
    public_key_array_bip32[0] = 0;
    public_key_array_bip32[1..].copy_from_slice(&public_key_array);
    // println!("PUBLIC KEY ARRAY BIP32 {:?}", public_key_array_bip32);
    
    // Generate Wallet Address for BIP32 Public Key
    let hash = Params::new()
        .hash_length(32)
        .to_state()
        .update(&public_key_array_bip32)
        .finalize();
    let wallet_address = "0x".to_string() + &hash.to_hex().to_ascii_lowercase();
    println!("Wallet Address: {}", wallet_address);

    // Create an Order
    let order = order::Order{
        market: market.to_string(),
        isBuy: true,
        price: (Unit::Ether(&"0").to_wei_str().unwrap()).parse().unwrap() ,
        quantity: (Unit::Ether(&"0.01").to_wei_str().unwrap()).parse().unwrap(),
        leverage: (Unit::Ether(&"3").to_wei_str().unwrap()).parse().unwrap(),
        maker: wallet_address.to_string(),
        reduceOnly: false,
        postOnly: false,
        orderbookOnly: true,
        expiration: 1696489993397,
        salt: 1695466663327515,
        ioc: false,
        orderType: "MARKET".to_string(),
        timeInForce: "GTT".to_string()
    };

    // Generate Order Hash, Sign, append "1" and append the base64 of the public key
    let serialized_msg = order::get_serialized_order(&order).await;
    let order_hash = digest(hex::decode(&serialized_msg).expect("Decoding failed"));
    println!("Order Hash: {}", order_hash);

    let msg_hash_decoded = hex::decode(digest(&serialized_msg)).expect("Decoding failed");
    let msg_hash_sig  = signingkey.sign(&msg_hash_decoded);
    // let msg_hash_sig = msg_hash_sig.to_string().to_ascii_lowercase() + "1" + &public_key_b64;

    println!("Signature: {}", msg_hash_sig);


    let msg: [u8; 32] = msg_hash_decoded.try_into().unwrap();

    let now = Instant::now();
    let verified = signingkey.verify(&msg, &msg_hash_sig).is_ok();
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    println!("verified: {}", verified);

    // Post Order and return the order hash
    // let returned_order_hash = order::post_signed_order(&order,msg_hash_sig, jwt_token).await;
    // println!("Returned Order Hash: {}", returned_order_hash);

    // // order hash generated should be the same as the one returned
    // assert_eq!(returned_order_hash, order_hash);

    // // Hash the order hashes that require cancellation and sign them
    // let hash = order::create_signed_cancel_order(&order_hash);
    // let cancel_sig_temp  = signingkey.sign(&hash.as_bytes());
    // let cancel_sig = cancel_sig_temp.to_string().to_ascii_lowercase() + "1";
    // println!("Signature: {}", cancel_sig);

    // // Combine Onboarding Signature and base64 of Public Key
    // let cancel_sig_full = cancel_sig + &public_key_b64;
    // println!("Full Signature: {}", cancel_sig_full);

    // let cancel_order = order::OrderCancellationJSONRequest {
    //     symbol : market.to_string(),
    //     orderHashes : [order_hash],
    //     cancelSignature : cancel_sig_full,
    //     parentAddress: "".to_string()
    // };

    // let response = order::post_cancel_order(cancel_order, jwt_token).await;
    // println!("Response: {}", response);
}