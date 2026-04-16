use mysql::{prelude::Queryable, Pool};

fn main() {
    dotenv::dotenv().ok();

    let mysql_url = std::env::var("MYSQL").expect("MYSQL must be set");
    let pool = Pool::new(mysql_url.as_str()).unwrap();
    let mut conn = pool.get_conn().unwrap();

    // basically look at all slots including and after the highest cranked slot, so we know the optimal latency for the min uncranked slot and everything after that
    conn.exec_drop("insert into vote_latencies (SELECT 'OPTIMAL', slot, slot-prev_slot FROM (SELECT slot, lag(slot) OVER(ORDER BY slot) AS prev_slot FROM `block_revenue` WHERE slot >= (SELECT max(slot) FROM `vote_latencies` WHERE `vote_account` LIKE 'OPTIMAL') ORDER BY `slot`) t where prev_slot is not null)", ());
}