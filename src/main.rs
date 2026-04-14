use dashmap::DashMap;
use futures::{FutureExt, SinkExt as _, StreamExt as _};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, sync::atomic::{AtomicU64, Ordering}};
use yellowstone_grpc_client::GeyserGrpcBuilder;
use yellowstone_grpc_proto::{geyser::{subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts, SubscribeRequestFilterBlocks}, tonic::transport::Endpoint};

mod lut;
mod non_votes;

const JITOTIP_1: Pubkey = Pubkey::from_str_const("96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5");
const JITOTIP_2: Pubkey = Pubkey::from_str_const("HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe");
const JITOTIP_3: Pubkey = Pubkey::from_str_const("Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY");
const JITOTIP_4: Pubkey = Pubkey::from_str_const("ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49");
const JITOTIP_5: Pubkey = Pubkey::from_str_const("DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh");
const JITOTIP_6: Pubkey = Pubkey::from_str_const("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt");
const JITOTIP_7: Pubkey = Pubkey::from_str_const("DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL");
const JITOTIP_8: Pubkey = Pubkey::from_str_const("3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT");
const JITOTIPS: [Pubkey; 8] = [JITOTIP_1, JITOTIP_2, JITOTIP_3, JITOTIP_4, JITOTIP_5, JITOTIP_6, JITOTIP_7, JITOTIP_8];

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let grpc_url = std::env::var("GRPC_URL").expect("GRPC_URL must be set");
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");
    let mysql_url = std::env::var("MYSQL").expect("MYSQL must be set");
    
    let handle = tokio::spawn(async move {
        let lut = lut::Lut::new(rpc_url);
        let non_votes = non_votes::NonVotes::new(&lut);

        println!("connecting to grpc server: {}", grpc_url);
        let mut grpc_client = GeyserGrpcBuilder{
            endpoint: Endpoint::from_shared(grpc_url.to_string()).unwrap(),
            x_token: None,
            x_request_snapshot: false,
            send_compressed: None,
            accept_compressed: None,
            max_decoding_message_size: Some(128 * 1024 * 1024),
            max_encoding_message_size: None,
        }.connect().await.expect("cannon connect to grpc server");
        println!("connected to grpc server!");
        
        let mut blocks = HashMap::new();
        blocks.insert("blocks".to_string(), SubscribeRequestFilterBlocks {
            account_include: vec![],
            include_transactions: Some(true),
            include_accounts: Some(true),
            include_entries: Some(false),
        });
        let mut accounts = HashMap::new();
        accounts.insert("lut".to_string(), SubscribeRequestFilterAccounts {
            account: vec![],
            owner: vec!["AddressLookupTab1e1111111111111111111111111".to_string()],
            filters: vec![],
            nonempty_txn_signature: Some(true),
        });
        let (mut _sink, mut stream) = grpc_client.subscribe_with_request(Some(SubscribeRequest {
            accounts,
            blocks,
            commitment: Some(CommitmentLevel::Confirmed as i32),
            ..Default::default()
        })).await.expect("unable to subscribe");

        while let Some(msg) = stream.next().await {
            if msg.is_err() {
                println!("grpc error: {:?}", msg.err());
                break;
            }
            let msg = msg.unwrap();
            match msg.update_oneof {
                Some(UpdateOneof::Account(account)) => {
                    lut.process_account_update(account);
                },
                Some(UpdateOneof::Block(block)) => {
                    println!("got block: {}", block.slot);
                    let tip_changes = DashMap::new();
                    let futs = block.transactions.iter().filter_map(|tx| {
                        if tx.is_vote {
                            None
                        } else {
                            Some(non_votes.process_transaction(tx).map(|changes| {
                                changes.into_iter().for_each(|(key, change)| {
                                    for tip in JITOTIPS {
                                        if key == tip {
                                            if change > 0 {
                                                let change = change as u64;
                                                tip_changes.entry(key).or_insert(AtomicU64::new(0)).fetch_add(change, Ordering::Relaxed);
                                            }
                                        }
                                    }
                                });
                            }))
                        }
                    }).collect::<Vec<_>>();
                    println!("processing {} non-vote transactions in block {}", futs.len(), block.slot);
                    futures::future::join_all(futs).await;
                    println!("finished processing non-vote transactions in block {}", block.slot);
                    lut.print_stats();
                    for (tip, change) in tip_changes {
                        println!("tip {}: {}", tip, change.load(Ordering::Relaxed));
                    }
                },
                _ => {},
            }
        }
    });

    handle.await.unwrap();
}
