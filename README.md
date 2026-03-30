# lez-inspector

A CLI tool to inspect LEZ sequencer state in real-time.

## Installation

    git clone https://github.com/ygd58/logos-inspector
    cd logos-inspector
    cargo build --release

## Usage

    lez-inspector latest          # latest block height
    lez-inspector block 42        # block by height
    lez-inspector account <addr>  # account info
    lez-inspector tx <hash>       # transaction lookup
    lez-inspector programs        # list deployed programs
    lez-inspector watch           # watch chain in real-time

## Custom RPC

    lez-inspector --rpc http://localhost:3040 latest

## Requirements

- LEZ sequencer running (via logos-scaffold localnet start)
- Default RPC: http://localhost:3040

## License

MIT OR Apache-2.0
