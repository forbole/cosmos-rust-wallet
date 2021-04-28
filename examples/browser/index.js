import {randomMnemonic, MnemonicWallet} from "crw-wallet";

const cosmos_dp = "m/44'/118'/0'/0/0";
const generate_btn = document.getElementById("generate_btn");
const mnemonic_container = document.getElementById("mnemonic");
const address_container = document.getElementById("address");

generate_btn.addEventListener("click", () => {
    const mnemonic = randomMnemonic();
    const wallet = new MnemonicWallet(mnemonic, cosmos_dp);
    const address = wallet.getBech32Address("cosmos");
    mnemonic_container.textContent = mnemonic;
    address_container.textContent = address;
    wallet.free();
});