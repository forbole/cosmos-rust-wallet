use serde::{Deserialize, Serialize};
use serde_json::{from_str};
use reqwest::{StatusCode, get};
use cosmos_sdk_proto::cosmos::auth::v1beta1::{QueryAccountRequest, BaseAccount};
use cosmos_sdk_proto::cosmos::auth::v1beta1::QueryAccountResponse;
use cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::{QueryClient};
use std::any::Any;
use tonic::codegen::http::Uri;
use std::io::Bytes;
use crate::error::{Error};
use prost::DecodeError;

#[derive(Serialize, Deserialize)]
 struct NodeInfoResponse {
    pub node_info: NodeInfo
}

#[derive(Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub network: String
}

pub struct ChainConfig {
    pub node_info: NodeInfo,
    pub grpc_addr: String,
    pub lcd_addr: String,
}

impl ChainConfig {
    pub fn new(node_info: NodeInfo, lcd_addr: String, grpc_addr: String) -> ChainConfig {
        ChainConfig{
            node_info,
            grpc_addr,
            lcd_addr,
        }
    }

    #[doc = r"Returns the account data associated with the given address"]
    pub async fn get_account_data(&self, address: String) -> Result<BaseAccount, Error>  {
        /// TODO move this externally to create it one time only
        // creating channel connection to the gRPC server
        let channel = tonic::transport::Channel::builder(self.grpc_addr.parse::<Uri>().unwrap())
            .connect()
            .await
            .map_err(| err | Error::Grpc(err.to_string()))?;

        // creating gRPC client from channel
        /// TODO move this externally to create it one time only
        let mut client = QueryClient::new(channel);

        // creating a new request
        let request = tonic::Request::new(
            QueryAccountRequest {
                address
            },
        );

        // sending request and waiting for response
        let response = client
            .account(request)
            .await
            .map_err(| err| Error::Grpc(err.to_string()))?
            .into_inner();

        // decoding response body into BaseAccount
        let base_account: BaseAccount = prost::Message::decode(
            response
                .account
                .unwrap()
                .value
                .as_slice()
        ).map_err(| err | Error::Decode(err.to_string()))?;

        Ok(base_account)
    }

}

#[doc = r"Returns the node info such as network moniker. Currently using LCD endpoint"]
async fn get_node_info(lcd_address: String) -> Result<NodeInfoResponse, Error> {
    let endpoint = format!("{}{}", lcd_address, "/node_info");
    let response = get(&endpoint)
        .await
        .map_err(| err | Error::Lcd(err.to_string()))?;

    let node_info_response: NodeInfoResponse;
    match response.status() {
        StatusCode::OK => {
            // unwrap here is safe since we already knew that the response is good
            node_info_response = response.json::<NodeInfoResponse>().await.unwrap()
        },
        status_code => return Err(Error::Lcd(status_code.to_string()))
    }

    Ok(node_info_response)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Error};

    #[actix_rt::test]
    async fn get_node_info_works() {
        let endpoint = "http://localhost:1317";
        let res = get_node_info(endpoint.to_string()).await;
        let exp_err = Error::Lcd("error sending request for url (http://localhost:1317/node_info): error trying to connect: tcp connect error: Connection refused (os error 61)".to_string());
        match res {
            Ok(node_response) => assert_ne!(node_response.node_info.network.len(), 0),
            Err(err) => assert_eq!(err, exp_err)
        }
    }

    #[actix_rt::test]
    async fn get_account_data_works() {
        let grpc_endpoint = "http://localhost:9090";
        let lcd_endpoint = "http://localhost:1317";
        let address = "desmos1jgta2lsjq9zln4jgv8hxslg3hdghmvrx9dq3e6";
        let node_info = get_node_info(lcd_endpoint.to_string())
            .await
            .unwrap()
            .node_info;

        let chain_config = ChainConfig::new(
            node_info,
            lcd_endpoint.to_string(),
            grpc_endpoint.to_string(),
        );

        let result = chain_config.get_account_data(address.to_string()).await;
        let exp_err = Error::Grpc("GRPC error: transport error: error trying to connect: tcp connect error: Connection refused (os error 61)".to_string());
        match result {
            Ok(response) => assert_eq!(response.address, address),
            Err(err) => assert_eq!(err, exp_err)
        };

    }
}