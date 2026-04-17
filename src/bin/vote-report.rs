use mysql::{Pool, prelude::Queryable};

fn main() {
    // select count(*), sum(latency), sum(least(16,17-latency)) from vote_latencies where vote_account="mintrNtxN3PhAB45Pt41XqyKghTTpqcoBkQTZqh96iR" and slot between 957*432000 and 957*432000+999;
    dotenv::dotenv().ok();

    let mysql_url = std::env::var("MYSQL").expect("MYSQL must be set");
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        println!("usage: {} <vote> <epoch> [bucket]", args[0]);
        return;
    }
    let epoch: u64 = args[2].parse().unwrap();
    let bucket: Option<u64> = if let Some(bucket_) = args.get(3) {
        Some(args[3].parse().unwrap())
    } else {
        None
    };
    if let Some(bucket) = bucket && bucket >= 432 {
        println!("max bucket is 431");
        return;
    }
    let pool = Pool::new(mysql_url.as_str()).unwrap();
    let mut conn = pool.get_conn().unwrap();

    let (start_slot, end_slot) = if let Some(bucket) = bucket {
        (epoch * 432000 + bucket * 1000, epoch * 432000 + bucket * 1000 + 999)
    } else {
        (epoch * 432000, epoch * 432000 + 431999)
    };
    conn.exec_map("select floor(slot/1000)%432, count(*), sum(latency), sum(least(16,18-latency)) from vote_latencies where vote_account=? and slot between ? and ? group by floor(slot/1000)", (args[1].clone(), start_slot, end_slot), |(bucket, count, lat_sum, credits): (u64, u64, u64, u64)|{
        println!("{bucket:3} {count:4}/1000 {lat_sum} {credits}");
    });
}