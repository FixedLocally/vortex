use crate::utils::pubkey_from_slice;
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;

// (vote account, slot, latency)
pub fn process_vote(landed_slot: u64, tx: &SubscribeUpdateTransactionInfo) -> Vec<(String, u64, u8)> {
    if !tx.is_vote {
        return vec![];
    }
    // deserialise TowerSync
    if let Some(meta) = &tx.meta && let Some(tx) = &tx.transaction && let Some(msg) = &tx.message {
        if meta.err.is_some() {
            // this vote failed and didn't modify state
            return vec![];
        }
        let ix = &msg.instructions[0];
        let vote_account = pubkey_from_slice(&msg.account_keys[ix.accounts[0] as usize]);
        let mut latencies = vec![];
        
        if ix.data[0..4] != [0x0e, 0x00, 0x00, 0x00] {
            // not a tower sync
            return vec![];
        }
        
        let mut offset = 4;
        let root = u64::from_le_bytes(ix.data[offset..offset + 8].try_into().unwrap());
        offset += 8;
        let lockout_len = u8::from_le_bytes(ix.data[offset..offset + 1].try_into().unwrap());
        offset += 1;
        let mut slot = root;
        for _ in 0..lockout_len {
            slot += ix.data[offset] as u64;
            offset += 1;
            // confirmation count
            offset += 1;
            latencies.push((vote_account.to_string(), slot, (landed_slot - slot) as u8));
        }

        return latencies;
    }

    vec![]
}