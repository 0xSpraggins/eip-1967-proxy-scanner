# EIP-1967 Proxy Scanner

## Description
A script, written in Rust using the web3 crate, that finds the queries EIP-1967 compliant smart contract proxies for the corresponding Implementation and Proxy Admin addresses.

More details about EIIP-1967 can be found [here](https://eips.ethereum.org/EIPS/eip-1967).

## Repo Setup
Create a dotenv file in the root directory of the repo with the following variables:

```env
MAINNET_WSS=<mainnet wss url>
SEPOLIA_WSS=<sepolia wss url>
POLYGON_WSS=<polygon wss url>
ZKSYNC_WSS=<zksync wss url>
ARBITRUM_WSS=<arbitrum wss url>
OPTIMISM_WSS=<optimism wss url>
BASE_WSS=<base wss url>
GNOSIS_WSS=<gnosis wss url>
```

```bash
source .env
cargo build
cargo run -- <network-name> <target-address>
```