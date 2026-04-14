use std::collections::HashMap;

use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo;
use crate::lut;

pub struct NonVotes<'a> {
    lut: &'a lut::Lut,
}

impl<'a> NonVotes<'a> {
    pub fn new(lut: &'a lut::Lut) -> Self {
        Self { lut }
    }

    pub async fn process_transaction(&self, tx: &SubscribeUpdateTransactionInfo) -> HashMap<Pubkey, i64> {
        // Implement the logic to process transactions and identify non-vote transactions
        // You can use the LUT to resolve account keys and analyze transaction instructions
        let changes = self.lut.decompile_changes(tx).await;
        changes
    }
}