use std::{path::PathBuf, fmt::Display};

use min_know::specs::address_appearance_index::AAIAppearanceTx;
use serde::{Deserialize, Serialize};
use web3::types::{Transaction, TransactionReceipt};

use crate::contract::MetadataSource;


/// Information about a particular logged event.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LoggedEvent {
    /// Unmodified Transaction.log.
    pub raw: web3::types::Log,
    /// The signature of the first topic (raw event name).
    pub topic_zero: String,
    /// Address of the contract that emitted the event.
    pub contract: Contract,
    /// Decoded 4 byte log signature.
    pub name: Option<String>,
    /// Associated names or tags for the emitting contract.
    pub nametags: Option<Vec<String>>,
}


/// Information about a particular transaction.
#[derive(Debug, Default, Clone, PartialEq)]
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
    pub abi: Option<String>,
    /// Flag for whether the contract data is from the source or is decompiled.
    pub decompiled: bool,
}


impl Display for LoggedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\t\tEmitted from: {} (nametags: {:?})", self.contract, self.nametags)?;
        write!(f, "\n\t\tEvent name: {:?} (signature: {})", self.name, self.topic_zero)?;
        write!(f, "\n\t\tLog type: {:?}", self.raw.log_type)?;
        write!(f, "\n\t\tTopic count: {}. Data: {} bytes.", self.raw.topics.len(), self.raw.data.0.len())?;
        write!(f, "")
    }
}

impl Display for Contract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let abi = match &self.abi {
            Some(a) => a,
            None => "Absent",
        };
        write!(f, "contract address {}, (abi sample: '{}', decomplied status: {})", self.address, abi, self.decompiled)
    }
}