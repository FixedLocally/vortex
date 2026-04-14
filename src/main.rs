use futures::{FutureExt, StreamExt as _};
use mysql::{Pool, prelude::Queryable};
use std::{collections::HashMap, sync::atomic::{AtomicU64, Ordering}};
use yellowstone_grpc_client::GeyserGrpcBuilder;
use yellowstone_grpc_proto::{geyser::{subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest, SubscribeRequestFilterAccounts, SubscribeRequestFilterBlocks}, tonic::transport::Endpoint};

use crate::votes::process_vote;

mod lut;
mod non_votes;
mod utils;
mod votes;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let grpc_url = std::env::var("GRPC_URL").expect("GRPC_URL must be set");
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");
    let mysql_url = std::env::var("MYSQL").expect("MYSQL must be set");
    
    let handle = tokio::spawn(async move {
        let lut = lut::Lut::new(rpc_url);
        let non_votes = non_votes::NonVotes::new(&lut);
        let pool = Pool::new(mysql_url.as_str()).unwrap();
        let mut conn = pool.get_conn().unwrap();
        let latency_stmt = conn.prep("insert ignore into vote_latencies (vote_account, slot, latency) values (?, ?, ?)").unwrap();
        let revenue_stmt = conn.prep("insert into block_revenue (slot, fee, tips) values (?, ?, ?)").unwrap();

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
                    let total_tips = AtomicU64::new(0);
                    let mut latencies = vec![];
                    let futs = block.transactions.iter().filter_map(|tx| {
                        if tx.is_vote {
                            latencies.extend(process_vote(block.slot, tx));
                            None
                        } else {
                            Some(non_votes.tally_tips(tx).map(|tips| {
                                total_tips.fetch_add(tips, Ordering::Relaxed);
                            }))
                        }
                    }).collect::<Vec<_>>();
                    println!("processing {} non-vote transactions in block {}", futs.len(), block.slot);
                    futures::future::join_all(futs).await;
                    println!("finished processing non-vote transactions in block {}", block.slot);
                    lut.print_stats();
                    let mut total_fees = 0;
                    if let Some(rewards) = block.rewards {
                        for reward in rewards.rewards {
                            println!("reward: {:?}, {:?}, {:?}", reward.pubkey, reward.lamports, reward.reward_type);
                            if reward.reward_type == 1 {
                                total_fees = reward.lamports;
                                break;
                            }
                        }
                    }
                    println!("total tips in block {}: {}", block.slot, total_tips.load(Ordering::Relaxed));
                    println!("total fees in block {}: {}", block.slot, total_fees);

                    conn.exec_batch(&latency_stmt, latencies);
                    conn.exec_drop(&revenue_stmt, (block.slot, total_fees, total_tips.load(Ordering::Relaxed)));
                },
                _ => {},
            }
        }
    });

    handle.await.unwrap();
}
