use std::env;
use std::str::FromStr;

use web3::transports::WebSocket;
use web3::types::{Address, H160, U256};
use web3::Web3;

enum Networks {
    Mainnet,
    Polygon,
    ZkSync,
    Arbitrum,
    Optimism,
    Base,
    Gnosis
}

struct Config {
    network: Networks,
    target_address: H160,
}

async fn load_web3_socket(config: &Config) -> Web3<WebSocket> {
    dotenv::dotenv().ok();
    let rpc_url: String = match config.network {
        Networks::Mainnet => dotenv::var("MAINNET_WSS").expect("MAINNET_WSS environment variable not set"),
        Networks::Polygon => dotenv::var("POLYGON_WSS").expect("POLYGON_WSS environment variable not set"),
        Networks::ZkSync => dotenv::var("ZKSYNC_WSS").expect("ZKSYNC_WSS environment variable not set"),
        Networks::Arbitrum => dotenv::var("ARBITRUM_WSS").expect("ARBITRUM_WSS environment variable not set"),
        Networks::Optimism => dotenv::var("OPTIMISM_WSS").expect("OPTIMISM_WSS environment variable not set"),
        Networks::Base => dotenv::var("BASE_WSS").expect("BASE_WSS environment variable not set"),
        Networks::Gnosis => dotenv::var("GNOSIS_WSS").expect("GNOSIS_WSS environment variable not set"),
    };
    let websocket = WebSocket::new(&rpc_url).await.unwrap();
    let web3_socket: Web3<WebSocket> = web3::Web3::new(websocket);

    return web3_socket;
}

fn get_command_args() -> Config {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Invalid number of arguments");
    }
    let network = match args[1].as_str() {
        "mainnet" => Networks::Mainnet,
        "polygon" => Networks::Polygon,
        "zksync" => Networks::ZkSync,
        "arbitrum" => Networks::Arbitrum,
        "optimism" => Networks::Optimism,
        "base" => Networks::Base,
        "gnosis" => Networks::Gnosis,
        _ => panic!("Invalid network"),
    };
    let address: H160 = Address::from_str(&args[2]).unwrap();
    let config: Config = Config {
        network: network,
        target_address: address,
    };
    return config;
}

async fn query_proxy(
    web3_socket: Web3<WebSocket>,
    storage_slot: &str,
    proxy_address: &H160,
) -> web3::types::H256 {
    let res: web3::types::H256 = web3_socket
        .eth()
        .storage(*proxy_address, U256::from_str(storage_slot).unwrap(), None)
        .await
        .unwrap();

    return res;
}

fn convert_storage_query_to_address(res: web3::types::H256) -> H160 {
    let res_as_bytes = res.as_bytes();
    let admin = H160::from_slice(&res_as_bytes[12..]);
    return admin;
}

async fn get_proxy_admin(web3_socket: &Web3<WebSocket>, proxy_address: &H160) -> H160 {
    // Storage Slot for Proxy Admin per EIP-1967 (https://eips.ethereum.org/EIPS/eip-1967)
    // bytes32(uint256(keccak256('eip1967.proxy.admin')) - 1) = 0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103
    let proxy_admin_slot =
        "0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103".to_string();
    let res: web3::types::H256 =
        query_proxy(web3_socket.clone(), &proxy_admin_slot, proxy_address).await;
    return convert_storage_query_to_address(res);
}

async fn get_proxy_implementation(web3_socket: &Web3<WebSocket>, proxy_address: &H160) -> Address {
    // Storage Slot for Proxy Admin per EIP-1967 (https://eips.ethereum.org/EIPS/eip-1967)
    // bytes32(uint256(keccak256('eip1967.proxy.implementation')) - 1) = 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc
    let proxy_implementation_slot: String =
        "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc".to_string();

    let res: web3::types::H256 = query_proxy(
        web3_socket.clone(),
        &proxy_implementation_slot,
        proxy_address,
    )
    .await;

    return convert_storage_query_to_address(res);
}

async fn get_proxy_beacon(web3_socket: &Web3<WebSocket>, proxy_address: &H160) -> Address {
    // Storage Slot for Proxy Beacon per EIP-1967 (https://eips.ethereum.org/EIPS/eip-1967)
    //bytes32(uint256(keccak256('eip1967.proxy.beacon')) - 1) = 0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50
    // The fallback contract for the proxy. Used only if the implementation is not set.
    let proxy_beacon_slot: String =
        "0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50".to_string();
    let res: web3::types::H256 =
        query_proxy(web3_socket.clone(), &proxy_beacon_slot, proxy_address).await;
    return convert_storage_query_to_address(res);
}

#[tokio::main]
async fn main() {
    let config: Config = get_command_args();
    let web3s: Web3<WebSocket> = load_web3_socket(&config).await;
    let admin: Address = get_proxy_admin(&web3s, &config.target_address).await;
    let implementation: Address = get_proxy_implementation(&web3s, &config.target_address).await;

    println!("EIP-1967 Proxy Scanner - {:?}", &config.target_address);
    println!("========================================================================");

    if implementation == Address::zero() {
        println!(
            "Proxy Beacon: {:?}",
            get_proxy_beacon(&web3s, &config.target_address).await
        );
    } else {
        println!("Implementation: {:?}", implementation);
    }

    println!("Proxy Admin: {:?}", admin);
    println!("========================================================================");
}
