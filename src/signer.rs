use crate::{
    rpc::{
        ChainClient,
        get_node_info
    },
    wallet::{
        Wallet
    }
};

/// Import a wallet from the given mnemonic
pub fn import_wallet(grpc_address: String, lcd_address: String, mnemonic: String, derivation_path: String, hrp: String) -> Wallet {
 let response = get_node_info(lcd_address).await?;
 let chain_config = ChainClient::new(
     response.node_info,
     lcd_address.clone(),
     grpc_address
 );

 let wallet = Wallet::from_mnemonic(mnemonic.as_str(), derivation_path, hrp).unwrap();
}
