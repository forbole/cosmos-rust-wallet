# Types Package
The types package contains all the common types used by `modules` and `packages`.

## Types

### Msg
Message type is the representation of a transaction's message already encoded from protobuf.
It's a wrapper of the `Any` types and can be converted from and to it.

```rust
pub struct Msg(pub Any);

/// From protobuf definition
pub struct Any {
    pub type_url: String,
    pub value: Vec<u8>,
}

fn example() {
    let proto_msg = Msg(Any {
        type_url: "/cosmos.bank.v1beta1.Msg/Send".to_string(),
        value: msg_bytes,
    });
}
```

### Error
Error is the representation of any kind of error that could happen during the execution of wallet's
operations.

Any function that return a ```Result<T,E>``` can return an error to the above function as follow:
```rust
fn example() {
    let mnemonic = Mnemonic::from_phrase(mnemonic_words, Language::English)
    .map_err(|err| Error::Mnemonic(err.to_string()))?;
}
```