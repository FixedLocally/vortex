use std::{collections::{HashMap, HashSet}, env};

use mysql::{prelude::Queryable, Pool};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let rpc_client = RpcClient::new(env::var("RPC_URL").unwrap());
    let args = std::env::args().collect::<Vec<_>>();
    let epoch = if args.len() >= 2 {
        args[1].parse::<u64>().expect("Invalid epoch")
    } else {
        rpc_client.get_epoch_info().await.unwrap().epoch
    };
    let leader_schedule = rpc_client.get_leader_schedule(Some(epoch * 432000)).await.expect("cannot get leader schedule").expect("cannot get leader schedule");
    let mysql_url = env::var("MYSQL").unwrap();
    let pool = Pool::new(mysql_url.as_str()).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let leader_set: HashSet<_> = leader_schedule.keys().collect();
    let stmt = format!("insert ignore into address_lookup_table (address) values {}", leader_set.iter().map(|_| "(?)").collect::<Vec<_>>().join(","));
    conn.exec_drop(stmt, leader_set.iter().collect::<Vec<_>>()).unwrap();
    let stmt = format!("select id, address from address_lookup_table where address in ({})", leader_set.iter().map(|_| "(?)").collect::<Vec<_>>().join(","));
    let leader_map: HashMap<String, u64> = HashMap::from_iter(conn.exec_map(stmt, leader_set.iter().collect::<Vec<_>>(), |(id, leader)| (leader, id)).unwrap());

    let rev_leader_schedule: Vec<(u64, u64)> = leader_schedule.iter().fold(vec![], |mut acc, (k, v)| {
        v.iter().for_each(|v| {
            acc.push((*v as u64 + 432000 * epoch, *leader_map.get(k).unwrap()));
        });
        acc
    });
    let stmt = conn.prep("INSERT ignore INTO leader_schedule (slot, leader_id) VALUES (?, ?)").unwrap();
    conn.exec_batch(stmt, rev_leader_schedule.iter());
    println!("populated leader schedule for epoch {epoch}");
}