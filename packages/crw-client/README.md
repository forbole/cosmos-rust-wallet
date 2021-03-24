# Client Package
The client package takes care of query and broadcast operations towards a cosmos-sdk based chain (updated to stargate).

## Chain Client
```rust
pub struct ChainClient {
    pub node_info: NodeInfo,
    pub grpc_addr: String,
    pub lcd_addr: String,
}

pub struct NodeInfo {
    pub id: String,
    pub network: String,
}
```

### Get Account data
```rust
fn get_account_data_example() {
    let grpc_endpoint = "http://localhost:9090";
    let lcd_endpoint = "http://localhost:1317";
    let address = "desmos1rc4jrjyxyq0qpv7sn5ex9tr0kt0chrdq3x66ah";
    let node_info = get_node_info(lcd_endpoint.to_string())
        .await
        .unwrap()
        .node_info;

    let chain_client = ChainClient::new(
        node_info,
        lcd_endpoint.to_string(),
        grpc_endpoint.to_string(),
    );

    let account = chain_client.get_account_data(address.to_string()).await.unwrap();
}
```
### Broadcast transaction
```rust
fn broadcast_tx_example() {
    let wallet = Wallet::from_mnemonic(...).unwrap();
    let chain_client = ChainClient::new(...);
    let msg = MsgSend {...};
    let fee = Fee {...};

    let mut msg_bytes = Vec::new();
    prost::Message::encode(&msg, &mut msg_bytes).unwrap();

    let proto_msg = Msg(Any {
        type_url: "/cosmos.bank.v1beta1.Msg/Send".to_string(),
        value: msg_bytes,
    });
    
    let msgs = vec![proto_msg];

    let account = chain_client
        .get_account_data(wallet.bech32_address.clone())
        .await
        .unwrap();

    let tx_signed_bytes = wallet
        .sign_tx(
            account,
            chain_client.node_info.network.clone(),
            msgs,
            fee,
            None,
            0,
        )
        .unwrap();

    let transaction_response = chain_client
        .broadcast_tx(tx_signed_bytes, BroadcastMode::Block)
        .await
        .unwrap();
}
```