use std::{fmt::Display, path::PathBuf};

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

impl LoggedEvent {
    fn nametag_string(&self) -> String {
        let mut nametags = String::new();
        match &self.nametags {
            Some(tags) => {
                if tags.is_empty() {
                    nametags.push_str("|unlabelled")
                }
                nametags.push('|');
                for tag in tags {
                    nametags.push_str(tag);
                    nametags.push('|');
                }
            }
            None => nametags.push_str("|unlabelled"),
        }
        nametags.push(' ');
        nametags.push_str(&self.contract.address);
        nametags
    }
    fn event_string(&self) -> String {
        let mut event = String::new();
        match &self.name {
            Some(n) => event.push_str(n),
            None => event.push_str("Unknown"),
        }
        let sig = format!(" event ({})", self.topic_zero);
        event.push_str(&sig);
        event.to_owned()
    }
    fn topics_string(&self) -> String {
        let mut t = format!("{}", self.raw.topics.len());
        for (i, topic) in self.raw.topics.iter().enumerate() {
            t.push_str(&format!(", topic {} {}", i + 1, topic));
        }
        t
    }
}

impl Display for LoggedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.event_string())?;
        write!(f, "\n\t\t{} contract", self.nametag_string())?;
        write!(f, "\n\t\t\tTopic values: {}", self.topics_string())?;
        write!(f, "\n\t\t\tData: {} bytes.", self.raw.data.0.len())?;
        write!(f, "")
    }
}

impl Display for Contract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let abi = match &self.abi {
            Some(a) => a,
            None => "Absent",
        };
        write!(
            f,
            "contract address {}, (abi sample: '{}', decomplied status: {})",
            self.address, abi, self.decompiled
        )
    }
}
