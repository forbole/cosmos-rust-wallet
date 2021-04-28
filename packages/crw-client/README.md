# Client Package
The client package takes care of query and broadcast operations towards a cosmos-sdk based chain (updated to stargate).

### Get Account data
```rust
fn get_account_data_example() {
    let grpc_endpoint = "http://localhost:9090";
    let lcd_endpoint = "http://localhost:1317";
    let address = "desmos1rc4jrjyxyq0qpv7sn5ex9tr0kt0chrdq3x66ah";
    let client = CosmosClient::new(lcd_endpoint, grpc_endpoint).unwrap();

    let account = client.get_account_data(address).await.unwrap();
}
```
### Broadcast transaction
```rust
fn broadcast_tx_example() {
    let wallet = MnemonicWallet::random("m/44'/118'/0'/0/0").unwrap();

    let cosmos_client =
        CosmosClient::new("http://localhost:1317", "http://localhost:9090").unwrap();

    let address = wallet.get_bech32_address("cosmos").unwrap();
    let account_data = cosmos_client.get_account_data(&address).await.unwrap();

    let amount = Coin {
        denom: "stake".to_string(),
        amount: "10".to_string(),
    };

    let msg_snd = MsgSend {
        from_address: address,
        to_address: "cosmos18ek6mnlxj8sysrtvu60k5zj0re7s5n42yncner".to_string(),
        amount: vec![amount],
    };

    let tx = TxBuilder::new("cosmoshub-5")
        .memo("Test memo")
        .account_info(account_data.sequence, account_data.account_number)
        .timeout_height(0)
        .fee("stake", "5000", 300_000)
        .add_message("/cosmos.bank.v1beta1.Msg/Send", msg_snd)
        .unwrap()
        .sign(&wallet)
        .unwrap();

    let res = cosmos_client
        .broadcast_tx(&tx, BroadcastMode::Block)
        .await
        .unwrap()
        .unwrap();
}
```