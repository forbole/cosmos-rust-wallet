# Wallet Package
The Wallet package contains all the things needed to create a `wallet` from a mnemonic phrase and use it 
to sign a tx.

This package can also be compiled to WASM and used on the browser.

### Import wallet from mnemonic
````rust
let cosmos_dp = "m/44'/118'/0'/0/0";
let mnemonic = "battle call once stool three mammal hybrid list sign field athlete amateur cinnamon eagle shell erupt voyage hero assist maple matrix maximum able barrel";

let wallet = MnemonicWallet::new(mnemonic, cosmos_dp).unwrap();
````

### Wallet from random mnemonic
````rust
let cosmos_dp = "m/44'/118'/0'/0/0";

let (wallet, mnemonic) = MnemonicWallet::random(cosmos_dp).unwrap();
````

### Sign tx
````rust
fn sign_tx_example() {
    let cosmos_dp = "m/44'/118'/0'/0/0";

    let (wallet, mnemonic) = MnemonicWallet::random(cosmos_dp).unwrap();

    // ... Prepare the transaction
    
    let serialized_transaction: Vec<u8> = ...; 
    
    let signature = wallet.sign(&serialized_transaction).unwrap();
}
````