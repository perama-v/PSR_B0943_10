mod apis;
mod cache;
mod contract;
mod data;
mod parsing;
mod history;

use std::env;

use anyhow::Result;
use min_know::config::choices::DirNature;
use history::Mode;

use crate::history::{AddressHistory, Config};

const PORTAL_NODE: &str = "http://localhost:8545";

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

    let config = Config::new(DirNature::Sample, PORTAL_NODE)?;
    let mut history = AddressHistory::new(SAMPLE_ADDRESS[1], config);

    history
        .get_transaction_ids()?
        .get_transaction_data(Some(1))
        .await?
        .get_receipts(Some(1))
        .await?
        .decode_logs(Some(1), Mode::AvoidApis)
        .await?;

    println!("{}", history);
    Ok(())
}


const SAMPLE_ADDRESS: [&str; 10] = [
    "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae", // an EF wallet
    "0x846be97d3bf1e3865f3caf55d749864d39e54cb9",
    "0xcb776c47291b55bf02b159810712f6897874f1cc", // 7 transactions
    "0x691e27c4c24cf8a5700563e42dadf66b557f372c", // 44 transactions
    "0x00d83bf7cec1f97489cf324aa8d159bae6aa4df5", // 1
    "0xebfd902f83d8ec838ad24259b5bf9617e1b774fc", // 1
    "0x029f388ac4d5c8bff490550ce0853221030e822b", // 339
    "0xae32371368e500c01068f4fe444aa3cedb48fab4", // 1
    "0x00bdb5699745f5b860228c8f939abf1b9ae374ed", // 1504
    "0xbf705e134a86c67b703a601c8d5a6caab06cbfd0", // 7
];