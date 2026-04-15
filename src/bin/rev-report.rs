use std::collections::HashMap;

use mysql::{Pool, prelude::Queryable};

fn median(v: &Vec<u64>) -> u64 {
    let len = v.len();
    if len % 2 == 0 {
        (v[len / 2] + v[len / 2 - 1]) / 2
    } else {
        v[len / 2]
    }
}

fn pct_of(first: u64, second: u64) -> f64 {
    return first as f64 * 100.0 / second as f64;
}

// slots, fee/tip median, fee/tip sum
// leader, slots, (med, avg, total) (fees, tips)
fn print_row(identity: String, global: (u64, u64, u64, u64, u64), leader: (u64, u64, u64, u64, u64)) {
    let global_avg_fee = global.3 / global.0;
    let global_avg_tip = global.4 / global.0;
    let leader_avg_fee = leader.3 / leader.0;
    let leader_avg_tip = leader.4 / leader.0;
    print!("{identity:45} ");
    print!("{:6} ", leader.0);
    // median
    print!("{:12.9} {:7.2}% ", leader.1 as f64 / 1e9, pct_of(leader.1, global.1));
    print!("{:12.9} {:7.2}% ", leader.2 as f64 / 1e9, pct_of(leader.2, global.2));
    // average
    print!("{:12.9} {:7.2}% ", leader_avg_fee as f64 / 1e9, pct_of(leader_avg_fee, global_avg_fee));
    print!("{:12.9} {:7.2}% ", leader_avg_tip as f64 / 1e9, pct_of(leader_avg_tip, global_avg_tip));
    // total
    print!("{:15.9} ", leader.3 as f64 / 1e9);
    print!("{:15.9} ", leader.4 as f64 / 1e9);
    println!("");
}

fn main() {
    dotenv::dotenv().ok();

    let mysql_url = std::env::var("MYSQL").expect("MYSQL must be set");
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} <epoch>\n", args[0]);
        return;
    }
    let epoch: u64 = args[1].parse().unwrap();
    let pool = Pool::new(mysql_url.as_str()).unwrap();
    let mut conn = pool.get_conn().unwrap();
    if let Ok(res) = conn.exec("SELECT b.slot, ls.leader, b.fee, b.tips FROM `block_revenue` b, leader_schedule_view ls where b.slot=ls.slot and b.slot between ? and ?", (epoch * 432000, epoch * 432000 + 431999)) {
        let mut rev_by_leader = HashMap::new();
        let mut fee_global = vec![];
        let mut tip_global = vec![];
        let slot_count = res.len();
        res.into_iter().for_each(|(slot, leader, fee, tip): (u64, String, u64, u64)| {
            // println!("{slot} {leader} {fee} {tip}");
            let mut rev = rev_by_leader.entry(leader.clone()).or_insert((vec![], vec![]));
            rev.0.push(fee);
            rev.1.push(tip);
            fee_global.push(fee);
            tip_global.push(tip);
        });
        fee_global.sort();
        tip_global.sort();
        let fee_median = median(&fee_global);
        let tip_median = median(&tip_global);
        let fee_total = fee_global.into_iter().reduce(|acc, e| acc + e).unwrap_or(0u64);
        let tip_total = tip_global.into_iter().reduce(|acc, e| acc + e).unwrap_or(0u64);
        
        // table header
        print!("{:45} ", "Leader");
        print!("{:>6} ", "Slots");
        print!("{:>12} {:7}  ", "Median Fee", "");
        print!("{:>12} {:7}  ", "Median Tip", "");
        print!("{:>12} {:7}  ", "Average Fee", "");
        print!("{:>12} {:7}  ", "Average Tip", "");
        print!("{:>15} ", "Total Fee");
        print!("{:>15} ", "Total Tip");
        println!("");

        print_row("(global)".to_string(), (slot_count as u64, fee_median, tip_median, fee_total, tip_total), (slot_count as u64, fee_median, tip_median, fee_total, tip_total));
        rev_by_leader.into_iter().for_each(|(leader, (mut fees, mut tips))| {
            fees.sort();
            tips.sort();
            let leader_slot_count = fees.len();
            let fee_med = median(&fees);
            let tip_med = median(&tips);
            let fee_sum = fees.into_iter().reduce(|acc, e| acc + e).unwrap_or(0u64);
            let tip_sum = tips.into_iter().reduce(|acc, e| acc + e).unwrap_or(0u64);
            print_row(leader, (slot_count as u64, fee_median, tip_median, fee_total, tip_total), (leader_slot_count as u64, fee_med, tip_med, fee_sum, tip_sum));
        });
    }
}