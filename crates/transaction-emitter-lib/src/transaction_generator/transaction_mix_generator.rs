// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0
use crate::transaction_generator::{TransactionGenerator, TransactionGeneratorCreator};
use aptos_sdk::types::{transaction::SignedTransaction, LocalAccount};
use async_trait::async_trait;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct TxnMixGenerator {
    rng: StdRng,
    txn_mix: Vec<(Box<dyn TransactionGenerator>, usize)>,
    total_weight: usize,
}

impl TxnMixGenerator {
    pub fn new(rng: StdRng, txn_mix: Vec<(Box<dyn TransactionGenerator>, usize)>) -> Self {
        let total_weight = txn_mix.iter().map(|(_, weight)| weight).sum();
        Self {
            rng,
            txn_mix,
            total_weight,
        }
    }
}

impl TransactionGenerator for TxnMixGenerator {
    fn generate_transactions(
        &mut self,
        accounts: Vec<&mut LocalAccount>,
        transactions_per_account: usize,
    ) -> Vec<SignedTransaction> {
        let mut picked = self.rng.gen_range(0, self.total_weight);
        for (gen, weight) in &mut self.txn_mix {
            if picked < *weight {
                return gen.generate_transactions(accounts, transactions_per_account);
            }
            picked -= *weight;
        }
        panic!(
            "Picked {} out of {}, couldn't find correct generator",
            picked, self.total_weight
        );
    }
}

pub struct TxnMixGeneratorCreator {
    txn_mix_creators: Vec<(Box<dyn TransactionGeneratorCreator>, usize)>,
}

impl TxnMixGeneratorCreator {
    pub fn new(txn_mix_creators: Vec<(Box<dyn TransactionGeneratorCreator>, usize)>) -> Self {
        Self { txn_mix_creators }
    }
}

#[async_trait]
impl TransactionGeneratorCreator for TxnMixGeneratorCreator {
    async fn create_transaction_generator(&mut self) -> Box<dyn TransactionGenerator> {
        let mut txn_mix = Vec::<(Box<dyn TransactionGenerator>, usize)>::new();
        for (generator_creator, weight) in self.txn_mix_creators.iter_mut() {
            txn_mix.push((
                generator_creator.create_transaction_generator().await,
                *weight,
            ));
        }

        Box::new(TxnMixGenerator::new(StdRng::from_entropy(), txn_mix))
    }
}
