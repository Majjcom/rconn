use aes::cipher::{generic_array::GenericArray, Block, BlockDecrypt, BlockEncrypt, Key, KeyInit};
use aes::Aes128;
use rand::prelude::*;
use rand::rng;
use sha2::{Digest, Sha256};
use std::io::Write;

pub struct RCipher {
    cipher: Aes128,
}

impl RCipher {
    pub fn new(key: &[u8; 16], const_key: &str) -> std::io::Result<Self> {
        let mut hasher = Sha256::new();
        hasher.write(key)?;
        hasher.write(const_key.as_bytes())?;
        let key = hasher.finalize();
        let key = key.as_slice();
        let key = Key::<Aes128>::from(GenericArray::clone_from_slice(&key[0..16]));
        let cipher = Aes128::new(&key);
        Ok(RCipher { cipher })
    }

    pub fn gen_key() -> [u8; 16] {
        let mut key = [0u8; 16];
        let mut r = rng();
        for i in 0..16 {
            key[i] = r.random_range(('A' as u8)..=('Z' as u8));
        }
        key
    }

    pub fn encript_data(&self, data: &[u8]) -> Vec<u8> {
        let mut data = data.to_vec();
        let padding_size = 16 - data.len() % 16;
        for _ in 0..padding_size {
            data.push(padding_size as u8);
        }
        for i in 0..(data.len() / 16) {
            let from = i * 16;
            self.cipher
                .encrypt_block(Block::<Aes128>::from_mut_slice(&mut data[from..from + 16]));
        }
        data
    }

    pub fn decript_data(&self, data: &[u8]) -> std::io::Result<Vec<u8>> {
        if data.len() % 16 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid data length",
            ));
        }
        let mut data = data.to_vec();
        for i in 0..(data.len() / 16) {
            let from = i * 16;
            self.cipher
                .decrypt_block(Block::<Aes128>::from_mut_slice(&mut data[from..from + 16]));
        }
        let padding_size = data.last().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid padding size",
        ))?;
        let data = data.split_off(data.len() - *padding_size as usize);
        Ok(data)
    }
}
