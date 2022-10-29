


use std::sync::Arc;

use serde_yaml::Value;
use tokio::task::{self};


use super::squad_arrow_sync::SquadArrowGenerator;
use super::squad_data::{SquadGeneral, SquadData};
use super::squad_endpoint::SquadEnpoint;
use super::{SquadConfig, squad_tokenizer};
use crate::batcher::{self};
use crate::endpoint;
use crate::provider::arrow_transfer::{ArrowTransfer};
use crate::provider::{ProviderChannel, arrow_provider};
use crate::transport::{self, ZmqChannel};


// TODO : The squad implementation has quite a few flaws and is not fully functional

pub fn create_provider() -> ArrowTransfer<SquadGeneral>{
    let locations = arrow_provider::download_huggingface_dataset("squad".to_string(), None, "training".to_string());
    let mut loader = ArrowTransfer::new(locations.unwrap()[0].to_owned());
    let generator = Box::new(SquadArrowGenerator::new(&loader.schema)) ;
    loader.generator = Some(generator);
    return loader;
}


pub async fn run_main(value:Arc<Value>) -> bool {

    type ProviderType = SquadGeneral;
    type DataType = SquadData;

    let (tx, rx) = tokio::sync::mpsc::channel::<ProviderChannel<ProviderType>>(2);
    let (tx_trans, rx_trans) = tokio::sync::mpsc::channel::<ZmqChannel<DataType>>(1);

    let tokenizer = &value["tokenizer"]["config"];
    let config:SquadConfig = serde_yaml::from_value(tokenizer.to_owned()).unwrap();


    let config_clone = config.clone();

    let iterations = value["source"]["iterations"].as_u64().unwrap().to_owned();

 
    // Create the Data Provider
    let join_provider = task::spawn(async move {

        let mut loader = create_provider();

        let load_result = loader.load_data(iterations, tx);
        load_result.await;
            
        
    });

    // Create the tokenizer
    let join_tokenizer = task::spawn(async move {
        //let batch = 
        let generator = Box::new(squad_tokenizer::SquadTokenizer::new(&config_clone));
        let result = batcher::create_batch(rx, tx_trans, generator);
        //let tok = squad_tokenizer::create_tokenizer(&config_clone, rx, tx_trans);
        result.await;
    });



    // Create the Receiver : Either a test endpoint for local testing or a ZMQ transport for external Operation
    let rx_select = value["sink"]["type"].as_str().map(|e| e.to_string());
    let join_rx = if rx_select.unwrap() == "test" { // Local Test Point
        let endpoint = Box::new(SquadEnpoint::new(config));
        task::spawn(async move {
            let result = endpoint::receive(rx_trans, endpoint);
            result.await
            
        })   
    }
    else { // Send to Processing Node
        let address = value["sink"]["config"]["address"].as_str().unwrap().to_string();        
        task::spawn(async move {
            let result = transport::zmq_transmit::receive_transport(address, rx_trans);
            result.await
        })
    };


    let node_select = value["node"]["type"].as_str().unwrap();
    
    // Option for no processing element for test
    if node_select == "none" { // Option where node point
        println!("Creating without Sink Node");
        let result = tokio::join!(join_rx, join_tokenizer, join_provider);
        println!("Finished {:?}", result.0);
        return true;
    }
    else {
        let join_node = {
            if node_select == "rust" {
                let address = value["sink"]["config"]["address"].as_str().unwrap().to_string();
                let batch_size = value["tokenizer"]["config"]["batch_size"].as_u64().unwrap();

                task::spawn(async move {
                    let result = transport::zmq_receive::rust_node_transport::<SquadData>(address, batch_size);
                    result.await
                })
            }
            else if node_select == "python" {
                let command = value["node"]["config"]["cmd"].as_str().unwrap().to_string();
                let cwd = value["node"]["config"]["cwd"].as_str().unwrap().to_string();
                let args:Vec<String> = value["node"]["config"]["args"].as_sequence().unwrap().into_iter().map(|e|e.as_str().unwrap().to_string()).collect();
    
                task::spawn(async move {
                    let result = transport::zmq_receive::python_node_transport(command,cwd,args);
                    result.await
                })
            }
            else {
                let address = value["sink"]["config"]["address"].as_str().unwrap().to_string();
                let batch_size = value["tokenizer"]["config"]["batch_size"].as_u64().unwrap();

                task::spawn(async move {
                    let result = transport::zmq_receive::rust_node_transport::<SquadData>(address, batch_size);
                    result.await
                })
            }

        };
        let result = tokio::join!(join_rx, join_tokenizer, join_provider, join_node);
        println!("Finished {:?} {:?}", result.0, result.3);
        return result.0.unwrap();
    }

    
    //let total = tokio::join!(join_rx, join_tokenizer, join_provider);
    
}