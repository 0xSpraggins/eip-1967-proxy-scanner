use std::env;
use std::str::FromStr;

use web3::transports::WebSocket;
use web3::types::{Address, H160, U256};
use web3::Web3;

async fn load_web3_socket() -> Web3<WebSocket> {
    dotenv::dotenv().ok();

    let rpc_url = dotenv::var("MAINNET_WSS").expect("MAINNET_WSS environment variable not set");
    let websocket = WebSocket::new(&rpc_url).await.unwrap();
    let web3_socket: Web3<WebSocket> = web3::Web3::new(websocket);

    return web3_socket;
}

fn load_proxy_address() -> H160 {
    let str_address: &str = &env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS not set");
    return Address::from_str(str_address).unwrap();
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
    // bytes32(uint256(keccak256('eip1967.proxy.admin')) - 1) = 0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103
    let proxy_implementation_slot: String =
        "0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103".to_string();

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
    let web3s: Web3<WebSocket> = load_web3_socket().await;
    let proxy_addr: Address = load_proxy_address();
    let admin: Address = get_proxy_admin(&web3s, &proxy_addr).await;
    let implementation: Address = get_proxy_implementation(&web3s, &proxy_addr).await;

    println!("EIP-1967 Proxy Scanner - {:?}", proxy_addr);
    println!("========================================================================");

    if implementation == Address::zero() {
        println!(
            "Proxy Beacon: {:?}",
            get_proxy_beacon(&web3s, &proxy_addr).await
        );
    } else {
        println!("Implementation: {:?}", implementation);
    }

    println!("Proxy Admin: {:?}", admin);
    println!("========================================================================");
}
