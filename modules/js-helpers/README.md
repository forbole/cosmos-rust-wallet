## Testing

This repository implements a set of small test crates.
Running these tests requieres to [install](https://rustwasm.github.io/wasm-pack/installer/)
`wasm-pack`.

To run the tests run:

```bash
wasm-pack test --firefox --headless client-test
```

To test in chrome, run:

```bash
wasm-pack test --chrome --headless client-test
```

To test in safari, run:

```bash
wasm-pack test --safari --headless client-test
```

You can omit the `--headless` flag to let the test open a web browser page showing the tests' result.