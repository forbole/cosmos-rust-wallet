/*use crate::wallet::{
 Wallet
};

use crate::rpc::{
 ChainConfig,
 get_node_info
};

/// Import a wallet from the given mnemonic
pub fn import_wallet(grpc_address: String, lcd_address: String, mnemonic: String, derivation_path: String, hrp: String) -> Wallet {
 let node_info = get_node_info(lcd_address).unwrap().node_info;
 let chain_config = ChainConfig::new(
  node_info,
  lcd_address.clone(),
  grpc_address
 );

 let wallet = Wallet::from_mnemonic(mnemonic.as_str(), derivation_path, hrp).unwrap();

 let account_data = chain_config.get_account_data(wallet.bech32_address).await.unwrap();


}*/
