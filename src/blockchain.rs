use serde;
use serde_json;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use std::fmt::Write;

#[derive(Debug, Clone, Serialize)]
pub struct Transaction {
    sender: String,
    receiver: String,
    amount: f32,
}

#[derive(Debug, Serialize)]
pub struct BlockHeader {
    timestamp: i32,
    nonce: u32,
    pre_hash: String,
    merkle: String,
    difficulty: u32,
}

#[derive(Debug, Serialize)]
pub struct Block {
    header: BlockHeader,
    count: u32,
    transactions: Vec<Transaction>,
}

pub struct Chain {
    chain: Vec<Block>,
    curr_transaction: Vec<Transaction>,
    difficulty: u32,
    miner_addr: String,
    reward: f32,
}

impl Chain {
    pub fn new(miner_addr: String, difficulty: u32) -> Self {
        let mut chain = Self {
            chain: Vec::new(),
            curr_transaction: Vec::new(),
            difficulty,
            miner_addr,
            reward: 100.0,
        };

        chain.generate_new_block();

        return chain;
    }

    pub fn generate_new_block(&mut self) -> bool {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        let header = BlockHeader {
            timestamp: since_the_epoch.as_secs() as i32,
            nonce: 0,
            pre_hash: self.last_hash(),
            merkle: String::new(),
            difficulty: self.difficulty,
        };

        let reward_trans = Transaction {
            sender: String::from("Root"),
            receiver: self.miner_addr.clone(),
            amount: self.reward,
        };

        let mut block = Block {
            header,
            count: 0,
            transactions: vec![],
        };

        block.transactions.push(reward_trans);
        block.transactions.append(&mut self.curr_transaction);
        block.count = block.transactions.len() as u32;
        block.header.merkle = Chain::get_merkle(block.transactions.clone());
        Chain::proof_of_work(&mut block.header);

        println!("{:#?}", block);
        self.chain.push(block);

        return true;
    }

    pub fn new_transaction(&mut self, sender: String, receiver: String, amount: f32) -> bool {
        self.curr_transaction.push(Transaction {
            sender,
            receiver,
            amount,
        });

        return true;
    }

    pub fn last_hash(&self) -> String {
        let block = match self.chain.last() {
            Some(block) => block,
            None => return String::from_utf8(vec![48; 64]).unwrap(),
        };

        return Chain::hash(&block.header);
    }

    pub fn update_difficulty(&mut self, new_difficulty: u32) -> bool {
        self.difficulty = new_difficulty;
        return true;
    }

    pub fn update_reward(&mut self, new_reward: f32) -> bool {
        self.reward = new_reward;
        return true;
    }

    pub fn get_merkle(curr_transaction: Vec<Transaction>) -> String {
        let mut merkle = Vec::new();

        for t in curr_transaction {
            let hash = Chain::hash(&t);
            merkle.push(hash);
        }

        if merkle.len() % 2 == 1 {
            let last = merkle.last().cloned().unwrap();
            merkle.push(last)
        }

        while merkle.len() > 1 {
            let mut h1 = merkle.remove(0);
            let mut h2 = merkle.remove(0);
            h1.push_str(&mut h2);
            let nh = Chain::hash(&h1);
            merkle.push(nh);
        }

        return merkle.pop().unwrap();
    }

    pub fn proof_of_work(header: &mut BlockHeader) {
        loop {
            let hash = Chain::hash(header);
            let slice: &str = &hash[..header.difficulty as usize];

            match slice.parse::<u32>() {
                Ok(value) => {
                    if value != 0 {
                        header.nonce += 1;
                    } else {
                        println!("Block hash: {}", hash);
                        break;
                    }
                }
                Err(_) => {
                    header.nonce += 1;
                    continue;
                }
            }
        }
    }

    pub fn hash<T: serde::Serialize>(item: &T) -> String {
        let input = serde_json::to_string(&item).unwrap();
        let mut hasher = Sha256::default();
        hasher.update(input.as_bytes());

        let res = hasher.finalize();
        let vec_res = res.to_vec();

        return Chain::hex_to_string(vec_res.as_slice());
    }

    pub fn hex_to_string(vec_res: &[u8]) -> String {
        let mut s = String::new();
        for b in vec_res {
            write!(&mut s, "{:x}", b).expect("Unable to write");
        }

        return s;
    }
}
