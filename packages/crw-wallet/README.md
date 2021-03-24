# Wallet Package
The Wallet package contains all the things needed to create a `wallet` from a mnemonic phrase and use it 
to sign a tx.

## Wallet
```rust
/// Keychain contains a pair of Secp256k1 keys.
pub struct Keychain {
    pub ext_public_key: ExtendedPubKey,
    pub ext_private_key: ExtendedPrivKey,
}

/// Wallet is a facility used to manipulate private and public keys associated
/// to a BIP-32 mnemonic.
pub struct Wallet {
    pub keychain: Keychain,
    pub bech32_address: String,
}
```

### Import wallet from mnemonic
````rust
fn import_wallet_from_mnemonic_example() {
    let wallet = Wallet::from_mnemonic(
        "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel",
        "m/44'/852'/0'/0/0".to_string(),
        "desmos".to_string(),
    ).unwrap();
}
````
### Sign tx
````rust
fn sign_tx_example() {
    let wallet = Wallet::from_mnemonic(...).unwrap();

    let lcd_endpoint = "http://localhost:1317";
    let node_info = get_node_info(lcd_endpoint.to_string())
        .await
        .unwrap()
        .node_info;
    let grpc_endpoint = "http://localhost:9090";
    let chain_client = ChainClient::new(
        node_info,
        lcd_endpoint.to_string(),
        grpc_endpoint.to_string(),
    );

    let account = chain_client
        .get_account_data(wallet.bech32_address.clone())
        .await
        .unwrap();

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

    let amount = Coin {
        denom: "stake".to_string(),
        amount: "100000".to_string(),
    };
    let msg = MsgSend {
        from_address: wallet.bech32_address.clone(),
        to_address: "desmos16kjmymxuxjns7usuke2604arqm9222gjgp9d56".to_string(),
        amount: vec![amount],
    };

    let mut msg_bytes = Vec::new();
    prost::Message::encode(&msg, &mut msg_bytes).unwrap();

    let proto_msg = Msg(Any {
        type_url: "/cosmos.bank.v1beta1.Msg/Send".to_string(),
        value: msg_bytes,
    });

    let msgs = vec![proto_msg];

    let tx_signed_bytes = wallet
        .sign_tx(account, chain_client.node_info.network, msgs, fee, None, 0)
        .unwrap();
}
````