use serde::{Deserialize, Serialize};

use crate::datasets::dataset::DataSet;
use crate::datasets::dataset_config::DataSetConfig;
use crate::models::bert::data::BertData;
use crate::provider::provider_config::ProviderConfig;
use crate::tokenizer::tokenizer_config::{TokenizerInternalConfig};
use crate::batcher::BatchConfig;
use crate::transport::TransportConfig;
use crate::transport::zmq_receive::NodeConfig;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum TaskType {
    #[serde(rename="masking")]
    Mlm,
    #[serde(rename="causal")]
    Causal,
    #[serde(rename="squad")]
    Squad,
    #[serde(rename="multi-label")]
    MultiLabel,
    #[serde(rename="single-class")]
    SingleClass,
    #[serde(rename="t5")]
    T5,
    Python,
    Context
}

#[derive(Deserialize, Serialize, Debug, Clone)]

pub enum ModelType {
    Bert,
    Roberta,
    Gpt2,
    T5
}

impl ModelType {
    pub fn create_dataset(&self, dataset_config:DataSetConfig, batch_config:BatchConfig) -> DataSet{
        match self {
            ModelType::Bert =>  {
                BertData::new(batch_config, dataset_config).into()
            }
            _ => todo!()
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]

pub struct TrainingConfig {
    pub model_config:ModelType,
    pub model:TaskType,
    pub source:ProviderConfig,
    pub tokenizer:TokenizerInternalConfig,
    pub batch:BatchConfig,
    pub transport:TransportConfig,
    pub node:NodeConfig,
    pub dataset_config:DataSetConfig
}