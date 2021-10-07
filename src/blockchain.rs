use std::collections::HashSet;
use super::*;

#[derive(Debug)]
pub enum BlockValidationErr {
    MismatchedIndex,
    InvalideHash,
    AchronologicalTimestamp,
    MismatchedPreviousHash,
    InvalidGenesisBlockFormat,
    InvalidInput,
    InsufficientInputValue,
    InvalidCoinbaseTransaction,
}
pub struct Blockchain {
    pub blocks: Vec<Block>,
    unspent_outputs: HashSet<Hash>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            blocks: vec![],
            unspent_outputs: HashSet::new(),
        }
    }
    pub fn update_with_block(&mut self, block: Block) -> Result<(), BlockValidationErr> {
            let i = self.blocks.len();

            if block.index != i as u32 {
                return Err(BlockValidationErr::MismatchedIndex);
            } else if !block::check_difficulty(&block.hash(), block.difficulty) {
                return Err(BlockValidationErr::InvalideHash);
            } else if i != 0 {
                // Note Genesis Block
                let prev_block = &self.blocks[i - 1];
                if block.timestamp <= prev_block.timestamp {
                    return Err(BlockValidationErr::AchronologicalTimestamp);
                } else if block.prev_block_hash != prev_block.hash {
                   return Err(BlockValidationErr::MismatchedPreviousHash);
                }
            } else {
                // Genesis Block
                if block.prev_block_hash != vec![0; 32] {
                    return Err(BlockValidationErr::InvalidGenesisBlockFormat);
                }
            }

            if let Some((conibase, transactions)) = block.transactions.split_first() {
                if !conibase.is_coinbase(){
                    return Err(BlockValidationErr::InvalidCoinbaseTransaction);
                }
                let mut block_spent: HashSet<Hash> = HashSet::new();
                let mut block_creatd: HashSet<Hash> = HashSet::new();
                let mut total_fee = 0;

                for transaction in transactions {
                    let input_hashes =  transaction.input_hashes();

                    if !(&input_hashes - &self.unspent_outputs).is_empty() || 
                        !(&input_hashes & &block_spent).is_empty()
                    {
                        return Err(BlockValidationErr::InvalidInput);
                    }

                    
                    let input_value = transaction.input_value();
                    let output_value = transaction.output_value();

                    if output_value > input_value {
                        return Err(BlockValidationErr::InsufficientInputValue);
                    }

                    let fee = input_value - output_value;
                    total_fee += fee;

                    block_spent.extend(input_hashes);
                    block_creatd.extend(transaction.output_hashes()); 
                }
            
                if conibase.output_value() < total_fee {
                    return Err(BlockValidationErr::InvalidCoinbaseTransaction);
                } else {
                    block_creatd.extend(conibase.output_hashes());
                }

                self.unspent_outputs.retain(|output| !block_spent.contains(output));
                self.unspent_outputs.extend(block_creatd);
            }

            self.blocks.push(block);

        Ok(())
    }
}
