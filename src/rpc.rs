use serde::{Deserialize, Serialize};
use serde_json::{from_str};
use reqwest::{StatusCode, blocking::get};
use cosmos_sdk_proto::cosmos::auth::v1beta1::{QueryAccountRequest, BaseAccount};
use cosmos_sdk_proto::cosmos::auth::v1beta1::QueryAccountResponse;
use cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::{QueryClient};
use std::any::Any;
use tonic::transport::Error;
use tonic::codegen::http::Uri;

#[derive(Serialize, Deserialize)]
pub struct NodeInfoResponse {
    pub node_info: NodeInfo
}

#[derive(Serialize, Deserialize)]
pub struct NodeInfo {
    pub network: String
}

#[doc = r"Returns the node info such as network moniker. Currently using LCD endpoint"]
pub fn get_node_info(lcd_address: String) -> Result<NodeInfoResponse, String> {
    let endpoint = format!("{}{}", lcd_address, "/node_info");
    let result = get(&endpoint);

    // checking if the response is good
    let response = match result {
        Ok(res) => res,
        Err(err) => return Err(err.to_string())
    };

    let node_info_response: NodeInfoResponse;
    match response.status() {
        StatusCode::OK => {
            // unwrap here is safe since we already knew that the response is good
            node_info_response = response.json::<NodeInfoResponse>().unwrap()
        },
        status_code => return Err(status_code.to_string())
    }

    Ok(node_info_response)
}

pub async fn get_account_data(rpc_endpoint: String, address: String) -> Result<QueryAccountResponse, Error>  {
    // Creating channel connection to the gRPC server
    let channel = tonic::transport::Channel::builder(rpc_endpoint.parse::<Uri>().unwrap())
        .connect()
        .await?;

    // creating gRPC client from channel
    let mut client = QueryClient::new(channel);

    // creating a new request
    let request = tonic::Request::new(
        QueryAccountRequest {
            address
        },
    );

    // sending request and waiting for response
    let response = client.account(request).await.unwrap().into_inner();
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_node_info_works() {
        let endpoint = "http://localhost:1317";
        let res = get_node_info(endpoint.to_string());
        match res {
            Ok(node_response) => assert_ne!(node_response.node_info.network.len(), 0),
            Err(err) => assert_eq!(err, "error sending request for url (http://localhost:1317/node_info): error trying to connect: tcp connect error: Connection refused (os error 61)")
        }
    }

    #[actix_rt::test]
    async fn get_account_data_works() {
        let rpc_endpoint = "http://localhost:9090";
        let address = "desmos1d9plufhcj8aumwlfc6vu00mm7n89mvfepevhsy";
        let result = get_account_data(rpc_endpoint.to_string(), address.to_string()).await;
        match result {
            Ok(response) => assert_eq!(response, QueryAccountResponse{ account: None }),
            Err(err) => assert_eq!(err.to_string(), "")
        };

    }
}