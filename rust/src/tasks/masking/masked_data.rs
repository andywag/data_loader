
use abomonation_derive::Abomonation;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Abomonation)]
pub struct MaskedData {
    pub input_ids:Vec<Vec<u32>>,
    pub attention_mask:Vec<Vec<u32>>,
    pub labels:Vec<Vec<i32>>,
    pub original:Option<Vec<Vec<u32>>>
}

impl MaskedData {
    pub fn new(batch_size:u32, sequence_length:u32, _masked_length:u32, pad_token:u32) -> Self{
        Self {
            input_ids: vec![vec![pad_token;sequence_length as usize];batch_size as usize],
            attention_mask: vec![vec![1;sequence_length as usize];batch_size as usize],
            labels:vec![vec![-100;sequence_length as usize]; batch_size as usize],
            original:None
        }
    }
}