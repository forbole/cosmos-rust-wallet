# Browser example
This folder contains an example on how the `crw-wallet` can be used from a javascript application.

# Setup 
In order to use the `crw-wallet` you need to build it and generate the js glue code that interact with WASM.  

To do so you need to install the following tools:
* [Rust](https://www.rust-lang.org/tools/install)
* [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

After installing these tools move inside the `cosmos-rust-wallet/packages/crw-wallet` directory and run the following command: 
`wasm-pack build --release -- --features wasm-bindgen` this will build the wallet and prepare a node module 
inside a new directory **pkg**.

Now go back to the example directory, install the required dependencies with `npm install` and 
finally launch the demo with `npm start`.
