use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_proto::prelude::SubscribeUpdateTransactionInfo;
use crate::lut;

const JITOTIP_1: Pubkey = Pubkey::from_str_const("96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5");
const JITOTIP_2: Pubkey = Pubkey::from_str_const("HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe");
const JITOTIP_3: Pubkey = Pubkey::from_str_const("Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY");
const JITOTIP_4: Pubkey = Pubkey::from_str_const("ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49");
const JITOTIP_5: Pubkey = Pubkey::from_str_const("DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh");
const JITOTIP_6: Pubkey = Pubkey::from_str_const("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt");
const JITOTIP_7: Pubkey = Pubkey::from_str_const("DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL");
const JITOTIP_8: Pubkey = Pubkey::from_str_const("3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT");
const JITOTIPS: [Pubkey; 8] = [JITOTIP_1, JITOTIP_2, JITOTIP_3, JITOTIP_4, JITOTIP_5, JITOTIP_6, JITOTIP_7, JITOTIP_8];

pub struct NonVotes<'a> {
    lut: &'a lut::Lut,
}

impl<'a> NonVotes<'a> {
    pub fn new(lut: &'a lut::Lut) -> Self {
        Self { lut }
    }

    pub async fn tally_tips(&self, tx: &SubscribeUpdateTransactionInfo) -> u64 {
        // Implement the logic to process transactions and identify non-vote transactions
        // You can use the LUT to resolve account keys and analyze transaction instructions
        let changes = self.lut.decompile_changes(tx).await;
        let mut total_tips = 0;
        for key in JITOTIPS {
            if let Some(change) = changes.get(&key) && change > &0 {
                total_tips += *change as u64;
            }
        }
        total_tips
    }
}