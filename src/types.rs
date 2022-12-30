use std::path::PathBuf;

use anyhow::{anyhow, Result};
use heimdall::decompile::DecompileBuilder;
use min_know::{
    database::types::Todd,
    specs::address_appearance_index::{AAIAppearanceTx, AAISpec},
};
use serde::{Deserialize, Serialize};
use web3::{
    transports::Http,
    types::{BlockNumber, Log, Transaction, TransactionReceipt, H256},
    Web3,
};

use crate::{
    apis::{abi_from_sourcify_api, method_from_fourbyte_api},
    contract::{cid_from_runtime_bytecode, MetadataSource},
    parsing::h160_to_string,
};

/// Selected mode of operation. APIs are used as temporary stop-gaps.
///
/// Available APIs: Sourcify and 4byte.directory.
pub enum Mode {
    AvoidApis,
    UseApis,
}

/// Represents historical activity data for a single address.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct AddressHistory {
    /// Address that a user wants to explore.
    pub address: String,
    /// Holds information for all transactions relevant to the address.
    pub transactions: Vec<TxInfo>,
    /// Database that contains the indexed transaction appearances.
    pub appearances_db: Todd<AAISpec>,
    /// RPC URL of local node.
    pub rpc_url: String,
}

/// Information about a particular transaction.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct TxInfo {
    /// A block number and index.
    pub location: AAIAppearanceTx,
    /// Data from eth_getTransactionByBlockNumberAndIndex.
    pub description: Option<Transaction>,
    /// Receipt from eth_getTransactionReceipt.
    pub receipt: Option<TransactionReceipt>,
    /// Events extracted from the Transaction.
    pub events: Option<Vec<LoggedEvent>>,
}

/// Information about a particular logged event.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LoggedEvent {
    /// Unmodified Transaction.log.
    pub raw: Log,
    /// The signature of the first topic (raw event name).
    pub topic_zero: H256,
    /// Address of the contract that emitted the event.
    pub contract: Contract,
    /// Decoded 4 byte log signature.
    pub name: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Contract {
    /// The address of the contract
    pub address: String,
    /// Link extracted from the CBOR encoded metadata on deployed bytecode (usually IFPS or swarm)
    pub source_code_metadata_link: Option<MetadataSource>,
    /// The bytecode of the contract.
    pub bytecode: Vec<u8>,
    /// Path to the source code (original or decompiled).
    pub source_code: PathBuf,
    /// The contract ABI (original or decompiled).
    pub abi: String,
    /// Flag for whether the contract data is from the source or is decompiled.
    pub decompiled: bool,
}

impl AddressHistory {
    /// Initialise
    pub fn new(address: &str, appearances_db: Todd<AAISpec>, rpc_url: &str) -> Self {
        AddressHistory {
            address: address.to_string(),
            transactions: vec![],
            appearances_db,
            rpc_url: rpc_url.to_string(),
        }
    }
    /// Find the appearances for this address.
    pub fn find_transactions(&mut self) -> Result<&mut Self> {
        let values = self.appearances_db.find(&self.address)?;
        let mut appearances: Vec<AAIAppearanceTx> = vec![];
        for record_value in values {
            // Join together the SSZ vectors in to one Vector.
            appearances.extend(record_value.value.to_vec());
        }
        for appearance in appearances {
            let info = TxInfo {
                location: appearance,
                description: None,
                receipt: None,
                events: None,
            };
            self.transactions.push(info)
        }
        Ok(self)
    }
    /// Get the basic transaction data from a node.
    ///
    /// Number of transactions to get data for can be capped.
    pub async fn get_transactions(&mut self, cap_num: Option<u32>) -> Result<&mut Self> {
        let transport = Http::new(&self.rpc_url)?;
        let web3 = Web3::new(transport);
        let txs = self.transactions.clone();
        for (i, mut tx) in txs.into_iter().enumerate() {
            if let Some(cap) = cap_num {
                if i > cap as usize {
                    break;
                }
            }
            // eth_getTransactionByBlockNumberAndIndex
            let tx_data = web3
                .eth()
                .transaction(tx.location.as_web3_tx_id())
                .await?
                .ok_or_else(|| anyhow!("No data for this transaction id."))?;

            tx.description = Some(tx_data);
        }
        Ok(self)
    }
    /// Get the receipts of transactions from a node.
    ///
    /// Number of transactions to get receipts for can be capped.
    pub async fn get_receipts(&mut self, cap_num: Option<u32>) -> Result<&mut Self> {
        let transport = Http::new(&self.rpc_url)?;
        let web3 = Web3::new(transport);
        let txs = self.transactions.clone();
        for (i, mut tx) in txs.into_iter().enumerate() {
            if let Some(cap) = cap_num {
                if i > cap as usize {
                    break;
                }
            }
            let Some(description) = tx.description else {continue};
            // eth_getTransactionReceipt
            let tx_receipt = web3
                .eth()
                .transaction_receipt(description.hash)
                .await?
                .ok_or_else(|| anyhow!("No receipt for this transaction hash."))?;

            tx.receipt = Some(tx_receipt);
        }
        Ok(self)
    }
    /// Decodes the event signatures of the logs for each transaction
    ///
    /// Every logged event originates from a contract. That contract
    /// is obtained with ethGetCode and useful information is stored
    /// alongside the event.
    pub async fn decode_logs(&mut self, cap_num: Option<u32>, mode: Mode) -> Result<&mut Self> {
        let transport = Http::new(&self.rpc_url)?;
        let web3 = Web3::new(transport);
        let txs = self.transactions.clone();
        for (i, mut tx) in txs.into_iter().enumerate() {
            if let Some(cap) = cap_num {
                if i > cap as usize {
                    break;
                }
            }
            let Some(receipt) = tx.receipt else {continue};
            let mut events: Vec<LoggedEvent> = vec![];
            for log in receipt.logs {
                let event = examine_log(log, &mode, &web3).await?;
                let Some(e) = event else {continue};
                events.push(e)
            }
            tx.events = Some(events)
        }
        Ok(self)
    }
}

/// Extracts the information about a given log.
async fn examine_log(log: Log, mode: &Mode, web3: &Web3<Http>) -> Result<Option<LoggedEvent>> {
    let Some(topic_zero) = log.topics.get(0) else {return Ok(None)};
    let raw = log.clone();
    // eth_getCode
    let bytecode = web3
        .eth()
        .code(log.address, Some(BlockNumber::Latest))
        .await?
        .0;

    let cid = cid_from_runtime_bytecode(bytecode.as_ref())?;

    let abi = match mode {
        Mode::UseApis => {
            let abi = abi_from_sourcify_api(&log.address).await?;
            // If no ABI is found at the API, decompile.
            match abi {
                Some(x) => x,
                None => {
                    let bytecode_string = hex::encode(&bytecode);
                    DecompileBuilder::new(&bytecode_string)
                        .output(&format!("decompiled/{}", log.address))
                        .decompile();
                    String::from("TODO: Pull from file")
                }
            }
        }
        Mode::AvoidApis => {
            let bytecode_string = hex::encode(&bytecode);
            DecompileBuilder::new(&bytecode_string)
                .output(&format!("decompiled/{}", log.address))
                .decompile();
            String::from("TODO: Pull from file")
        }
    };

    let contract = Contract {
        address: h160_to_string(&log.address),
        source_code_metadata_link: cid,
        bytecode,
        source_code: PathBuf::from("TODO: Path to source code."),
        abi,
        decompiled: false,
    };

    let event: LoggedEvent = LoggedEvent {
        raw,
        contract,
        topic_zero: topic_zero.to_owned(),
        name: match mode {
            Mode::AvoidApis => None,
            Mode::UseApis => method_from_fourbyte_api(topic_zero).await?,
        },
    };
    Ok(Some(event))
}
