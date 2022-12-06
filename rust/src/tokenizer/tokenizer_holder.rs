

use tokenizers::{Tokenizer};

use crate::tasks::python::{python_top::{PythonParserTop, PythonContextCreator}, python_top_new::PythonParserNew};

use super::{tokenizer_config::{TokenizerType}, tokenizer_data::TokenizedData};
use std::thread;

//#[derive(Deserialize, Serialize, Debug)]

pub enum TokenizerHolder {
    HuggingFace(Tokenizer),
    PythonNew(PythonParserNew),
    Python(PythonParserTop),
    PythonContext(PythonContextCreator)
}

impl TokenizerHolder {
    pub fn get_ids(&mut self, data:String) -> Vec<u32> {
        match self {
            TokenizerHolder::HuggingFace(x) => {
                let result = x.encode(data, true);
                result.unwrap().get_ids().to_vec()    
            }
            TokenizerHolder::Python(x) => {
                x.encode(data)
            },
            TokenizerHolder::PythonContext(x) => {
                let result = x.encode(data);
                result
            }
            TokenizerHolder::PythonNew(_x) => {
                vec![]
            },
        }
    }

    pub fn encode(&self, data:tokenizers::EncodeInput) -> Option<tokenizers::Encoding> {
        match self {
            TokenizerHolder::HuggingFace(x) => {
                Some(x.encode(data, true).unwrap()) 
            }
            _ => None
            
        }
    }

    pub fn encode_split(&self, data:String) -> Option<TokenizedData> {
        match self {
            TokenizerHolder::PythonNew(x) => {
                x.encode(data) 
            }
            _ => None
            
        }
    }

    pub fn token_to_id(&self, token:&str) -> Option<u32> {
        match self {
            TokenizerHolder::HuggingFace(x) => {
                x.token_to_id(token) 
            }
            // TODO : Need to add extra ids
            TokenizerHolder::Python(_) | TokenizerHolder::PythonContext(_) | TokenizerHolder::PythonNew(_) => {
                match token {
                    "<pad>" | "[PAD]" => Some(0),
                    "<s>" | "[CLS]" => Some(1),
                    "</s>" | "[SEP]" => Some(2),
                    "<|endoftext|>" => Some(3),
                    "<unk" => Some(4),
                    "<mask>" | "[MASK]" => Some(5),
                    _ => None
                }
            }
        }
    }

}

fn get_hugging_tokenizer(location:String) -> Option<Tokenizer> {
    let (tx,rx)= std::sync::mpsc::channel::<Tokenizer>();
    //let location_clone = location.clone();
    thread::spawn(move || {
        let base = Tokenizer::from_pretrained(location, None);
        let _ =tx.send(base.unwrap());
    });
    match rx.recv() {
        Ok(x) => {
            Some(x)
        },
        Err(e) => {
            log::error!("Couldn't Open Tokenizer {:?}", e);
            None
        },
    }    
}


pub fn create_tokenizer_holder(config:TokenizerType) -> TokenizerHolder {
    match config {
        TokenizerType::HuggingFace(name) => {
            let tokenizer = get_hugging_tokenizer(name).unwrap();
            TokenizerHolder::HuggingFace(tokenizer)
        }
        TokenizerType::Python => {
            let tokenizer = PythonParserTop::new();
            TokenizerHolder::Python(tokenizer)
        },
        TokenizerType::PythonContext => {
            let tokenizer = PythonContextCreator::new(2048);
            TokenizerHolder::PythonContext(tokenizer)
        }
    }
}