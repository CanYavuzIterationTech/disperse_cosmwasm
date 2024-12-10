# Disperse CosmWasm Contract

A CosmWasm smart contract that enables efficient distribution of native and CW20 tokens to multiple recipients in a single transaction. This contract is particularly useful for batch payments, airdrops, or any scenario requiring multiple token transfers.

## Features

- Disperse native tokens (e.g., ATOM, OSMO) to multiple recipients
- Disperse CW20 tokens to multiple recipients
- Validation of total amounts and recipient data
- Gas-efficient batch processing

## Prerequisites

- Rust 1.60.0 or later
- [rustup](https://rustup.rs/)
- wasm32-unknown-unknown target
- [cargo-generate](https://github.com/ashleygwilliams/cargo-generate)

## Installation

First, make sure you have Rust and the wasm32 target installed:

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

## Building

To build the contract:

```sh
cargo wasm
```

To run tests:

```sh
cargo unit-test
cargo integration-test
```

To generate schema:

```sh
cargo schema
```

## Usage

### Native Token Disperse

To disperse native tokens, send a message with the following format:

```json
{
  "disperse": {
    "recipients": [
      {
        "address": "recipient1_address",
        "amount": [
          {
            "denom": "uatom",
            "amount": "1000000"
          }
        ]
      },
      {
        "address": "recipient2_address",
        "amount": [
          {
            "denom": "uatom",
            "amount": "2000000"
          }
        ]
      }
    ]
  }
}
```

### CW20 Token Disperse

For CW20 tokens, first approve the disperse contract to spend your tokens, then send them with the following message format:

```json
{
  "send": {
    "contract": "disperse_contract_address",
    "amount": "3000000",
    "msg": {
      "disperse_cw20": {
        "recipients": [
          {
            "address": "recipient1_address",
            "amount": "1000000"
          },
          {
            "address": "recipient2_address",
            "amount": "2000000"
          }
        ]
      }
    }
  }
}
```

## Testing

The contract includes both unit tests and integration tests. The integration tests demonstrate how to use the contract with CW20 tokens in a multi-contract environment.

To run all tests:

```sh
cargo test
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Security

This contract has not been audited and is provided as-is. Use at your own risk.
