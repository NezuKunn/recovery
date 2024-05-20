use reqwest;
use bincode;

use std::fs::File;
use std::io::BufReader;

use serde_json::json;

use std::collections::HashMap;

use jupiter_swap_api_client::{
    quote::QuoteRequest, swap::SwapRequest, transaction_config::TransactionConfig,
    JupiterSwapApiClient,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{pubkey, transaction::VersionedTransaction};
use std::str::FromStr;
use serde_json::Value;

pub async fn start(owner: String, private_key_str: &str) -> bool {

    let jupiter_swap_api_client: JupiterSwapApiClient = JupiterSwapApiClient::new(format!("https://quote-api.jup.ag/v6"));

    let temp: Vec<(String, f64)> = get_token_accounts(owner.clone()).await;

    let bps: u16 = 200;

    for item in temp.iter() {

        let (mint_, kapital_) = item;

        let mint = mint_.clone();
        let kapital = *kapital_ as u64;

        let wallet: Pubkey = solana_sdk::pubkey::Pubkey::from_str(&owner as &str).unwrap();

        let input_mint_: Pubkey = solana_sdk::pubkey::Pubkey::from_str(&mint as &str).unwrap();
        let output_mint_: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

        let quote_resp: jupiter_swap_api_client::quote::QuoteResponse = quote_response(jupiter_swap_api_client.clone(), kapital, input_mint_, output_mint_, bps).await;

        let versioned_transaction_in: VersionedTransaction = swap_response(jupiter_swap_api_client.clone(), wallet, quote_resp.clone()).await;

        let mut b: bool = sign(versioned_transaction_in.clone(), private_key_str).await;

        if b == false {
            while b == false {
                b = sign(versioned_transaction_in.clone(), private_key_str).await;
            }
        }

        println!("Победа")
    }



    // let input_mint_: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
    // let output_mint_: Pubkey = solana_sdk::pubkey::Pubkey::from_str(address).unwrap();

    // let amount: u64 = kapital - 105000;

    // let quote_resp: jupiter_swap_api_client::quote::QuoteResponse = quote_response(jupiter_swap_api_client.clone(), amount, input_mint_, output_mint_, bps).await;

    // let versioned_transaction_in: VersionedTransaction = swap_response(jupiter_swap_api_client.clone(), wallet, quote_resp.clone()).await;

    // let b: bool = sign(versioned_transaction_in, private_key_str).await;

    // if b == false {
    //     return false; // Не смог купить токены, процесс сначала начинается
    // }



    // let result: bool = generator(address).await;

    // if result == false {
    //     println!("А всё, токен {} всё . _.", address);
    //     return false; // Прошло 48 часов и цена не поднялась
    // }

    // let (token_amount, decimals): (f64, u64) = get_amount_to_account(address).await;

    // let amount: u64 = (token_amount * ((10 as u64).pow(decimals as u32)) as f64) as u64;

    // let input_mint_: Pubkey = solana_sdk::pubkey::Pubkey::from_str(address).unwrap();
    // let output_mint_: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

    // let quote_resp: jupiter_swap_api_client::quote::QuoteResponse = quote_response(jupiter_swap_api_client.clone(), amount, input_mint_, output_mint_, bps).await;

    // let versioned_transaction_out: VersionedTransaction = swap_response(jupiter_swap_api_client.clone(), wallet, quote_resp.clone()).await;

    // let mut b: bool = sign(versioned_transaction_out.clone(), private_key_str).await;

    // if b == false {
    //     while b == false {
    //         b = sign(versioned_transaction_out.clone(), private_key_str).await;
    //     }
    // }

    // println!("Победа, получается?");

    return true
}

pub async fn get_token_accounts(owner: String) -> Vec<(String, f64)> {

    let rpc = "http://localhost:8899";
    let client = reqwest::Client::new();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let params = vec![
        json!(owner),
        json!({
            "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }),
        json!({
            "encoding": "jsonParsed"
        })
    ];

    let json_data = json!({
        "jsonrpc": "2.0",
        "id": 13,
        "method": "getTokenAccountsByOwner",
        "params": params
    });

    let res = client.post(rpc)
        .headers(headers)
        .json(&json_data)
        .send()
        .await.unwrap();

    let response: HashMap<String, serde_json::Value> = res.json().await.unwrap();
    let data = &response["result"]["value"];

    let mut out: Vec<(String, f64)> = Vec::new();

    if let Value::Array(array) = &data {
        for item in array {
            let mint: String = item["account"]["data"]["parsed"]["info"]["mint"].to_string();
            let amount: f64 = item["account"]["data"]["parsed"]["info"]["tokenAmount"]["uiAmount"].as_f64().unwrap();
            out.push((mint, amount));
        }
    }

    out
}


pub async fn quote_response(jupiter_swap_api_client: JupiterSwapApiClient, amount: u64, input_mint_: Pubkey, output_mint_: Pubkey, bps: u16) -> jupiter_swap_api_client::quote::QuoteResponse {
    let quote_request = QuoteRequest {
        amount: amount,
        input_mint: input_mint_,
        output_mint: output_mint_,
        slippage_bps: bps,
        ..QuoteRequest::default()
    };

    let quote_response: jupiter_swap_api_client::quote::QuoteResponse = jupiter_swap_api_client.quote(&quote_request).await.unwrap();

    quote_response
}


pub async fn swap_response(jupiter_swap_api_client: JupiterSwapApiClient, wallet: Pubkey, quote_response: jupiter_swap_api_client::quote::QuoteResponse) -> VersionedTransaction {
    let swap_response_in = jupiter_swap_api_client
        .swap(&SwapRequest {
            user_public_key: wallet,
            quote_response: quote_response.clone(),
            config: TransactionConfig::default(),
        })
        .await
        .unwrap();

    let versioned_transaction_in: VersionedTransaction = bincode::deserialize(&swap_response_in.swap_transaction).unwrap();

    versioned_transaction_in
}

pub async fn sign(mut versioned_transaction: VersionedTransaction, private_key_str: &str) -> bool {
    let rpc_client: RpcClient = RpcClient::new("http://localhost:8899".into());

    let file = File::open(private_key_str).unwrap();
    let reader = BufReader::new(file);
    let v: Value = serde_json::from_reader(reader).unwrap();
    let contents: Vec<u8> = v.as_array().unwrap().iter().map(|x| {
        match x.as_u64() {
            Some(num) if num <= 255 => Ok(num as u8),
            _ => Err("Invalid number in JSON"),
        }
    }).collect::<Result<Vec<u8>, &str>>().unwrap();

    let keypair = match Keypair::from_bytes(&contents) {
        Ok(keypair) => keypair,
        Err(e) => panic!("{}", e),
    };

    let latest_blockhash = rpc_client.get_latest_blockhash().await.unwrap();

    versioned_transaction
        .message
        .set_recent_blockhash(latest_blockhash);

    let signed_versioned_transaction = VersionedTransaction::try_new(versioned_transaction.message, &[&keypair]).unwrap();

    let flag: bool;

    let result = rpc_client.send_and_confirm_transaction(&signed_versioned_transaction).await;
    match result {
        Ok(signature) => {
            println!("Tx | {}", signature);
            flag = true;
        },
        Err(_err) => {
            // eprintln!("Error | {:?}", err);
            eprintln!("|[ Error ]|");
            flag = false;
        }
    }

    flag
}