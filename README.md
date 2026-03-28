# logos-inspector

A CLI tool to inspect Logos blockchain state in real-time.

## Installation

    git clone https://github.com/ygd58/logos-inspector
    cd logos-inspector
    cargo build --release

## Usage

    logos-inspector latest          # latest block height
    logos-inspector block 42        # block by height
    logos-inspector account <addr>  # account info
    logos-inspector tx <hash>       # transaction lookup
    logos-inspector programs        # list deployed programs
    logos-inspector watch           # watch chain in real-time

## Custom RPC

    logos-inspector --rpc http://localhost:3040 latest

## Requirements

- Logos sequencer running (via logos-scaffold localnet start)
- Default RPC: http://localhost:3040

## License

MIT OR Apache-2.0
