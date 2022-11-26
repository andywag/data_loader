use std::sync::Arc;

use serde_yaml::Value;
use tokio::task::{JoinHandle, self};

use crate::{provider::{ProviderChannel, ProviderConfig, general_file_provider,  SourceDescription, pile_datasets}, tasks::{runner_simple}, tokenizer_wrapper::{TokenizerWrapper}, datasets::DataSet};
use tokio::sync::mpsc::Sender;

use super::{MaskingConfig, gpt2_test_endpoint::Gpt2Endpoint, gpt_data::GptData, masked_data::MaskedData, masking_test_endpoint::MaskingEndpoint, T5Config, t5_data::T5Data, t5_test_endpoint::T5Endpoint};
use crate::tasks::gen_tokenizer::GenTokenizer;




// Create the Dataset Provider for Squad
fn create_provider(value:&Arc<Value>, tx:Sender<ProviderChannel<String>>) -> JoinHandle<()> {


    let provider_config:ProviderConfig = serde_yaml::from_value(value["source"].to_owned()).unwrap();
    let handle = task::spawn(
        async move {
            match provider_config.source {
                SourceDescription::DataList(datasets) => {
                    //log::info!("Datasets {:?}", datasets);
                    general_file_provider::load_data_sets(datasets, provider_config.length, tx).await;
                },
                SourceDescription::Pile{typ} => {
                    let datasets = pile_datasets::get_datasets(typ);
                    match datasets {
                        Some(x) => {
                            general_file_provider::load_data_sets(x, provider_config.length, tx).await;
                        }
                        None => {
                            log::error!("Data Set Not Supported");
                            std::process::exit(0);
                        }
                    }
                    
                },
         
                _ => {
                    log::error!("Can't support Input Type");
                }
            }
        });
    handle

}

fn get_config(value:&Arc<serde_yaml::Value>) -> MaskingConfig {
    let tokenizer = &value["tokenizer"]["config"];
    serde_yaml::from_value(tokenizer.to_owned()).unwrap()
}



fn create_generator(value:&Arc<serde_yaml::Value>, tokenizer:TokenizerWrapper)-> Box<dyn crate::batcher::Batcher<S=String,T=DataSet> + Send> {
    
    let config = get_config(&value);
    let dataset = MaskedData::new(config.clone(),tokenizer.mask_token().unwrap());
    let wrap = GenTokenizer::new(crate::datasets::DataSet::Mask(dataset), config.batch_size as usize, config.sequence_length as usize, tokenizer, true);
    Box::new(wrap)
}

fn create_causal_generator(value:&Arc<serde_yaml::Value>, tokenizer:TokenizerWrapper)-> Box<dyn crate::batcher::Batcher<S=String,T=DataSet> + Send> {
    
    let config = get_config(&value);
    let dataset = GptData::new(config.batch_size as usize, config.sequence_length as usize);
    let wrap = GenTokenizer::new(crate::datasets::DataSet::Gpt2(dataset), config.batch_size as usize, config.sequence_length as usize, tokenizer, true);
    Box::new(wrap)
}
 
fn create_t5_generator(value:&Arc<serde_yaml::Value>, tokenizer:TokenizerWrapper)-> Box<dyn crate::batcher::Batcher<S=String,T=DataSet> + Send> {
    let value = &value["tokenizer"]["config"];
    let config:T5Config = serde_yaml::from_value(value.to_owned()).unwrap();
    let batch_size = config.batch_size;
    let sequence_length = config.sequence_length;
    let dataset = T5Data::new(config, tokenizer.get_extra_ids());
    let wrap = GenTokenizer::new(crate::datasets::DataSet::T5(dataset), batch_size, sequence_length, tokenizer, false);
    Box::new(wrap)
}


fn create_causal_endpoint(value:&Arc<serde_yaml::Value>) -> Box<dyn crate::transport::test_endpoint::EndPoint<DataSet> + Send> { 
    Box::new(Gpt2Endpoint::new(get_config(value)))
}
fn create_endpoint(value:&Arc<serde_yaml::Value>) -> Box<dyn crate::transport::test_endpoint::EndPoint<DataSet> + Send> {
    Box::new(MaskingEndpoint::new(get_config(value)))
}
fn create_t5_endpoint(value:&Arc<serde_yaml::Value>) -> Box<dyn crate::transport::test_endpoint::EndPoint<DataSet> + Send> {
    let tokenizer = &value["tokenizer"]["config"];
    let config:T5Config = serde_yaml::from_value(tokenizer.to_owned()).unwrap();
    Box::new(T5Endpoint::new(config))
}

pub enum MaskType {
    Mlm,
    Causal, 
    Span
}

pub async fn run(value:Arc<Value>, mask_type:MaskType) -> bool{

    let result = match mask_type {
        MaskType::Mlm => {
            runner_simple::run_main(value, 
                runner_simple::Either::Left(Box::new(create_provider)), 
                Box::new(create_generator), 
                Box::new(create_endpoint))
        },
        MaskType::Causal => {
            runner_simple::run_main(value, 
                runner_simple::Either::Left(Box::new(create_provider)), 
                Box::new(create_causal_generator) , 
                Box::new(create_causal_endpoint))
        },
        MaskType::Span => {
            runner_simple::run_main(value, 
                runner_simple::Either::Left(Box::new(create_provider)), 
                Box::new(create_t5_generator) , 
                Box::new(create_t5_endpoint)) 
        },
    };
    result.await

    
}



