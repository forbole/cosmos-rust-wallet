use std::{any::Any, io::Bytes};
use cosmos_sdk_proto::cosmos::{
    auth::v1beta1::{
        query_client::QueryClient, BaseAccount, QueryAccountRequest,
    },
    base::abci::v1beta1::TxResponse,
    tx::v1beta1::{
        service_client::ServiceClient,
        BroadcastMode, BroadcastTxRequest, BroadcastTxResponse, TxRaw,
    }
};
use reqwest::{get, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use tonic::{codegen::http::Uri, transport::Channel, Request};
use crate::error::Error;

#[derive(Clone, Serialize, Deserialize)]
/// Response of /node_info query
pub struct NodeInfoResponse {
    pub node_info: NodeInfo,
}

#[derive(Clone, Serialize, Deserialize)]
/// NodeInfo represent some basics full node info
pub struct NodeInfo {
    pub id: String,
    pub network: String,
}

/// ChainConfig represent the configuration of a full node
#[derive(Clone)]
pub struct ChainClient {
    pub node_info: NodeInfo,
    pub grpc_addr: String,
    pub lcd_addr: String,
}

impl ChainClient {
    pub fn new(node_info: NodeInfo, lcd_addr: String, grpc_addr: String) -> ChainClient {
        ChainClient {
            node_info,
            grpc_addr,
            lcd_addr,
        }
    }

    /// Returns the account data associated with the given address
    pub async fn get_account_data(&self, address: String) -> Result<BaseAccount, Error> {
        /// TODO move this externally to create it one time only
        // Create channel connection to the gRPC server
        let channel = Channel::builder(self.grpc_addr.parse::<Uri>().unwrap())
            .connect()
            .await
            .map_err(|err| Error::Grpc(err.to_string()))?;

        // Create gRPC query auth client from channel
        /// TODO move this externally to create it one time only
        let mut client = QueryClient::new(channel);

        // Build a new request
        let request = Request::new(QueryAccountRequest { address });

        // Send request and wait for response
        let response = client
            .account(request)
            .await
            .map_err(|err| Error::Grpc(err.to_string()))?
            .into_inner();

        // Decode response body into BaseAccount
        let base_account: BaseAccount =
            prost::Message::decode(response.account.unwrap().value.as_slice())
                .map_err(|err| Error::Decode(err.to_string()))?;

        Ok(base_account)
    }

    /// Broadcast a tx through gRPC client
    pub async fn broadcast_tx_gRPC(
        &self,
        tx_bytes: Vec<u8>,
        broadcast_mode: BroadcastMode,
    ) -> Result<TxResponse, Error> {
        // Create channel connection to the gRPC server
        let channel = Channel::builder(self.grpc_addr.parse::<Uri>().unwrap())
            .connect()
            .await
            .map_err(|err| Error::Grpc(err.to_string()))?;

        // Create gRPC tx client from channel
        let mut tx_client = ServiceClient::new(channel);

        let mode = match broadcast_mode {
            BroadcastMode::Unspecified => 0,
            BroadcastMode::Block => 1,
            BroadcastMode::Sync => 2,
            BroadcastMode::Async => 3,
        };

        let request = Request::new(BroadcastTxRequest { tx_bytes, mode });

        let tx_response = tx_client
            .broadcast_tx(request)
            .await
            .map_err(|err| Error::Grpc(err.to_string()))?
            .into_inner()
            .tx_response
            .unwrap();

        Ok(tx_response)
    }
}

/// Returns the node info such as network moniker. Currently using LCD endpoint
pub async fn get_node_info(lcd_address: String) -> Result<NodeInfoResponse, Error> {
    let endpoint = format!("{}{}", lcd_address, "/node_info");
    let response = get(&endpoint)
        .await
        .map_err(|err| Error::Lcd(err.to_string()))?;

    let node_info_response: NodeInfoResponse;
    match response.status() {
        StatusCode::OK => {
            // Unwrap here is safe since we already knew that the response is good
            node_info_response = response.json::<NodeInfoResponse>().await.unwrap()
        }
        status_code => return Err(Error::Lcd(status_code.to_string())),
    }

    Ok(node_info_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::wallet::Wallet;
    use cosmos_sdk_proto::cosmos::{
        base::v1beta1::Coin,
        tx::v1beta1::Fee,
        bank::v1beta1::MsgSend
    };
    use crate::msg::Msg;

    struct TestData {
        chain_client: ChainClient,
        proto_msg: Msg,
        fee: Fee,
    }

    async fn setup_test(lcd_endpoint: &str, grpc_endpoint: &str, address: String) -> TestData {
        let node_info = get_node_info(lcd_endpoint.to_string())
            .await
            .unwrap()
            .node_info;

        let chain_client = ChainClient::new(
            node_info,
            lcd_endpoint.to_string(),
            grpc_endpoint.to_string()
        );

        // Gas Fee
        let coin = Coin {
            denom: "stake".to_string(),
            amount: "5000".to_string(),
        };

        let fee = Fee {
            amount: vec![coin],
            gas_limit: 300000,
            payer: "".to_string(),
            granter: "".to_string(),
        };

        let amount = Coin{ denom: "stake".to_string(), amount: "100000".to_string() };
        let msg = MsgSend{
            from_address: address,
            to_address: "desmos1gvd8j8w986qey68s6trc3h9zkzxest20zs5g0w".to_string(),
            amount: vec![amount]
        };

        let mut msg_bytes =  Vec::new();
        prost::Message::encode(&msg, &mut msg_bytes).unwrap();

        let proto_msg = Msg::new(
            "/cosmos.bank.v1beta1.Msg/Send",
            msg_bytes
        );

        TestData{ chain_client, proto_msg, fee }
    }

    #[actix_rt::test]
    async fn get_node_info_works() {
        let endpoint = "http://localhost:1317";
        let res = get_node_info(endpoint.to_string()).await;
        let exp_err = Error::Lcd("error sending request for url (http://localhost:1317/node_info): error trying to connect: tcp connect error: Connection refused (os error 61)".to_string());
        match res {
            Ok(node_response) => assert_ne!(node_response.node_info.network.len(), 0),
            Err(err) => assert_eq!(err, exp_err),
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

        let chain_config = ChainClient::new(
            node_info,
            lcd_endpoint.to_string(),
            grpc_endpoint.to_string(),
        );

        let result = chain_config.get_account_data(address.to_string()).await;
        let exp_err = Error::Grpc("GRPC error: transport error: error trying to connect: tcp connect error: Connection refused (os error 61)".to_string());
        match result {
            Ok(response) => assert_eq!(response.address, address),
            Err(err) => assert_eq!(err, exp_err),
        };
    }

    #[actix_rt::test]
    async fn broadcast_tx_works() {
        let wallet = Wallet::from_mnemonic(
            "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel",
            "m/44'/852'/0'/0/0".to_string(),
            "desmos".to_string(),
        ).unwrap();

        let test_data = setup_test(
            "http://localhost:1317",
            "http://localhost:9090",
            wallet.bech32_address.clone()
        ).await;

        let account = test_data.chain_client
            .get_account_data(wallet.bech32_address.clone())
            .await
            .unwrap();

        let tx_signed_bytes = wallet.sign_tx(
            account,
            test_data.chain_client.clone(),
            &[test_data.proto_msg], test_data.fee,
            None,
            0
        ).unwrap();

        let res = test_data.chain_client.broadcast_tx_gRPC(tx_signed_bytes, BroadcastMode::Block)
            .await.unwrap();

        let tx_height = res.height;
        let code = res.code;
        let code_space = res.codespace;
        let raw_log = res.raw_log;

        assert_eq!(code, 0)
    }
}
