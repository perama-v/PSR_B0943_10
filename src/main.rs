use std::env;

use anyhow::{anyhow, Result};
use heimdall::decompile::DecompileBuilder;
use min_know::{
    config::choices::{DataKind, DirNature},
    database::types::Todd,
    specs::address_appearance_index::{AAIAppearanceTx, AAISpec},
    utils::contract::cid_from_runtime_bytecode,
};
use web3::types::BlockNumber;

/// Uses index data and a theoretical local Ethereum portal node to
/// decode information for a user.
///
/// A transaction is inspected for logs, which contain event
/// signatures and the contract from which they were emitted.
///
/// The contract runtime bytecode may also be feched and decompiled
/// with Heimdall.
///
/// Additionally, the contract code can be inspected and the metadata
/// extracted, which may contain a link to the contract ABI.
#[tokio::main]
async fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample)?;
    println!("DB is {:#?}", db);

    // A random address.
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9";
    //let address = "0xcb776c47291b55bf02b159810712f6897874f1cc"; // 7
    //let address = "0x691e27c4c24cf8a5700563e42dadf66b557f372c"; // 44
    //let address = "0x00d83bf7cec1f97489cf324aa8d159bae6aa4df5"; // 1
    //let address = "0xebfd902f83d8ec838ad24259b5bf9617e1b774fc"; // 1
    //let address = "0x029f388ac4d5c8bff490550ce0853221030e822b"; // 339
    //let address = "0xae32371368e500c01068f4fe444aa3cedb48fab4"; // 1
    //let address = "0x00bdb5699745f5b860228c8f939abf1b9ae374ed"; // 1504
    //let address = "0xbf705e134a86c67b703a601c8d5a6caab06cbfd0"; // 7

    let values = db.find(address)?;
    let mut appearances: Vec<AAIAppearanceTx> = vec![];
    for v in values {
        appearances.extend(v.value.to_vec());
    }

    let portal_node = "http://localhost:8545";
    let transport = web3::transports::Http::new(portal_node)?;
    let web3 = web3::Web3::new(transport);

    let Some(tx) = appearances.get(0)
        else {return Err(anyhow!("No data for this transaction id."))};

    // portal node eth_getTransactionByBlockNumberAndIndex
    let tx_data = web3
        .eth()
        .transaction(tx.as_web3_tx_id())
        .await?
        .ok_or_else(|| anyhow!("No data for this transaction id."))?;


    // portal node eth_getTransactionReceipt
    let tx_receipt = web3
        .eth()
        .transaction_receipt(tx_data.hash)
        .await?
        .ok_or_else(|| anyhow!("No receipt for this transaction hash."))?;

    println!(
        "Tx {:?} has {:#?} logs:\n",
        tx_receipt.transaction_hash,
        tx_receipt.logs.len()
    );

    for log in tx_receipt.logs {
        println!(
            "Contract: {:?}\n\tTopics logged: {:?}",
            log.address,
            log.topics,
        );
        // portal node eth_getCode
        let bytecode = web3
            .eth()
            .code(log.address, Some(BlockNumber::Latest))
            .await?
            .0;

        match cid_from_runtime_bytecode(bytecode.as_ref()) {
            Ok(None) => {}
            Ok(cid) => {
                println!("\tMetadata CID: {:?}", cid.unwrap());
            }
            Err(e) => return Err(e),
        };
        let bytecode_string = hex::encode(bytecode);
        DecompileBuilder::new(&bytecode_string)
            .output(&format!("decompiled/{}", log.address))
            .decompile();
    }
    Ok(())
}