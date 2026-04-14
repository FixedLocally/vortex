use std::{collections::HashMap, sync::atomic::{AtomicU64, Ordering}};

use crate::utils::pubkey_from_slice;
use dashmap::DashMap;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use yellowstone_grpc_proto::geyser::{SubscribeUpdateAccount, SubscribeUpdateTransactionInfo};

pub struct Lut {
    rpc_client: RpcClient,
    lut_cache: DashMap<Pubkey, Vec<Pubkey>>,

    cache_hit_cnt: AtomicU64,
    cache_miss_cnt: AtomicU64,
    external_update_cnt: AtomicU64,
    fetch_fail_cnt: AtomicU64,
}

impl Lut {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url.to_string()),
            lut_cache: DashMap::new(),
            cache_hit_cnt: AtomicU64::new(0),
            cache_miss_cnt: AtomicU64::new(0),
            external_update_cnt: AtomicU64::new(0),
            fetch_fail_cnt: AtomicU64::new(0),
        }
    }

    pub async fn ensure_luts(&self, lut_address: Vec<Pubkey>) {
        // find keys that are not in the cache
        let len = lut_address.len() as u64;
        let missing_keys: Vec<Pubkey> = lut_address.into_iter()
            .filter(|key| !self.lut_cache.contains_key(key))
            .collect();
        let miss_count = missing_keys.len() as u64;
        let hit_count = len - miss_count;
        self.cache_hit_cnt.fetch_add(hit_count, Ordering::Relaxed);
        self.cache_miss_cnt.fetch_add(miss_count, Ordering::Relaxed);
        if !missing_keys.is_empty() {
            // fetch missing LUTs from RPC and update the cache
            self.rpc_client.get_multiple_accounts(&missing_keys[..]).await.unwrap().into_iter().enumerate().for_each(|(i, account)| {
                self.cache_miss_cnt.fetch_add(missing_keys.len() as u64, Ordering::Relaxed);
                if let Some(account) = account && let Ok(lut) = AddressLookupTable::deserialize(&account.data) {
                    self.lut_cache.insert(missing_keys[i].clone(), lut.addresses.to_vec());
                } else {
                    self.fetch_fail_cnt.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    }

    pub fn process_account_update(&self, update: SubscribeUpdateAccount) {
        // Implement the logic to process account updates and update the LUT cache accordingly
        if let Some(account) = update.account {
            let pubkey = pubkey_from_slice(&account.pubkey[0..32]);
            if let Ok(lut) = AddressLookupTable::deserialize(&account.data) {
                self.lut_cache.insert(pubkey, lut.addresses.to_vec());
                self.external_update_cnt.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub async fn decompile_changes(&self, tx: &SubscribeUpdateTransactionInfo) -> HashMap<Pubkey, i64> {
        let mut changes = HashMap::new();
        // ensure that we can resolve all LUTs before processing the transaction
        if let Some(meta) = &tx.meta && let Some(tx) = &tx.transaction && let Some(msg) = &tx.message {
            // first part of the accounts referenced by this tx
            let mut account_keys: Vec<Pubkey> = msg.account_keys.iter().map(|key| pubkey_from_slice(&key[0..32])).collect();

            // the rest are in LUTs, make sure we have them cached
            let lut_keys: Vec<Pubkey> = msg.address_table_lookups.iter().map(|lookup| pubkey_from_slice(&lookup.account_key[0..32])).collect();
            self.ensure_luts(lut_keys.clone()).await;

            // second part of the accounts
            let mut writable: Vec<Pubkey> = Vec::new();
            let mut readonly: Vec<Pubkey> = Vec::new();
            msg.address_table_lookups.iter().enumerate().for_each(|(i, lookup)| {
                let lut_key = lut_keys[i];
                let lut = self.lut_cache.get(&lut_key).expect("lut not found");

                lookup.writable_indexes.iter().for_each(|index| {
                    writable.push(lut.get(*index as usize).unwrap_or(&Pubkey::default()).clone());
                });

                lookup.readonly_indexes.iter().for_each(|index| {
                    readonly.push(lut.get(*index as usize).unwrap_or(&Pubkey::default()).clone());
                });
            });
            account_keys.extend(writable);
            account_keys.extend(readonly);

            // assert that len(account_keys) matches meta's
            assert_eq!(account_keys.len(), meta.post_balances.len());
            assert_eq!(account_keys.len(), meta.pre_balances.len());

            // now we can decompile the changes
            for i in 0..account_keys.len() {
                let pre_balance = meta.pre_balances[i];
                let post_balance = meta.post_balances[i];
                if pre_balance != post_balance {
                    changes.insert(account_keys[i], post_balance.saturating_sub(pre_balance).try_into().unwrap());
                }
            }
        }
        changes
    }

    pub fn print_stats(&self) {
        println!("LUT cache size: {}", self.lut_cache.len());
        println!("LUT cache hits: {}", self.cache_hit_cnt.load(Ordering::Relaxed));
        println!("LUT cache misses: {}", self.cache_miss_cnt.load(Ordering::Relaxed));
        println!("LUT external updates: {}", self.external_update_cnt.load(Ordering::Relaxed));
        println!("LUT fetch failures: {}", self.fetch_fail_cnt.load(Ordering::Relaxed));
    }
}