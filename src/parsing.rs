use anyhow::{anyhow, Result};
use eip55::checksum;
use serde_json::Value;
use web3::types::H160;

/// Gets a human readable summary of contract metadata.
///
/// Parses a JSON string representing contract metadata and returns name of contract and
/// information about functions as a printable string.
pub fn summary_of_abi_from_json(metadata: Value) -> Result<String> {
    let contract_name = &metadata["settings"]["compilationTarget"];
    let mut summary = format!("Contract: {}", contract_name);
    let n_funcs = match &metadata["output"]["abi"] {
        Value::Array(a) => a.len(),
        _ => 0,
    };
    for n in 0..n_funcs {
        let loc = format!("/output/abi/{}", n);
        let func = metadata
            .pointer(&loc)
            .ok_or_else(|| anyhow!("Could not read abi from json at loc: {}", &loc))?;
        let f = format!(
            "\n\t{} {} {}.\n\t\tInputs: {}\n\t\tOutputs: {}",
            &func["type"],
            &func["stateMutability"],
            &func["name"],
            &func["inputs"],
            &func["outputs"]
        );
        summary.push_str(&f);
    }
    Ok(summary)
}

/// Takes a web3.rs address and returns checksummed String.
///
/// E.g., "0xabCd...1234"
pub fn as_checksummed(address: &H160) -> String {
    let s = h160_to_string(address);
    checksum(&s)
}

/// Converts H160 to String.
pub fn h160_to_string(address: &H160) -> String {
    //format!("0x{:0>20}", hex::encode(address))
    hex::encode(address)
}

#[test]
fn parse_metadata() {
    let metadata_str = r#"
    {"compiler":{"version":"0.4.19+commit.c4cbbb05"},"language":"Solidity","output":{"abi":[{"constant":true,"inputs":[],"name":"name","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"guy","type":"address"},{"name":"wad","type":"uint256"}],"name":"approve","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"src","type":"address"},{"name":"dst","type":"address"},{"name":"wad","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"wad","type":"uint256"}],"name":"withdraw","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"","type":"address"}],"name":"balanceOf","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"dst","type":"address"},{"name":"wad","type":"uint256"}],"name":"transfer","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[],"name":"deposit","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"constant":true,"inputs":[{"name":"","type":"address"},{"name":"","type":"address"}],"name":"allowance","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"payable":true,"stateMutability":"payable","type":"fallback"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":true,"name":"guy","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":true,"name":"dst","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Transfer","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"dst","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Deposit","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Withdrawal","type":"event"}],"devdoc":{"methods":{}},"userdoc":{"methods":{}}},"settings":{"compilationTarget":{"WETH9.sol":"WETH9"},"libraries":{},"optimizer":{"enabled":false,"runs":200},"remappings":[]},"sources":{"WETH9.sol":{"keccak256":"0x4f98b4d0620142d8bea339d134eecd64cbd578b042cf6bc88cb3f23a13a4c893","urls":["bzzr://8f5718790b18ad332003e9f8386333ce182399563925546c3130699d4932de3e"]}},"version":1
    }"#;
    let metadata_json: Value = serde_json::from_str(metadata_str).unwrap();
    let summary = summary_of_abi_from_json(metadata_json).unwrap();
    println!("Summary: {}", summary);
}
