use std::{fmt::Display, path::PathBuf};

use anyhow::{anyhow, Result};
use log::debug;
use min_know::{
    config::{
        address_appearance_index::Network,
        choices::{DataKind, DirNature},
    },
    database::types::Todd,
    specs::{
        address_appearance_index::{AAIAppearanceTx, AAISpec},
        nametags::NameTagsSpec,
        signatures::SignaturesSpec,
    },
};

use web3::{
    transports::Http,
    types::{BlockNumber, Log, H160},
    Web3,
};

use crate::{
    cache::Cache,
    contract::cid_from_runtime_bytecode,
    data::{Contract, LoggedEvent, TxInfo},
    parsing::h160_to_string,
};

/// Selected mode of operation. APIs are used as temporary stop-gaps.
///
/// Available APIs: Sourcify and 4byte.directory.
#[allow(dead_code)]
pub enum Mode {
    AvoidApis,
    UseApis,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Database that contains the indexed transaction appearances.
    pub appearances_db: Todd<AAISpec>,
    /// Database that contains the indexed transaction appearances.
    pub signatures_db: Todd<SignaturesSpec>,
    /// Database that contains the indexed transaction appearances.
    pub nametags_db: Todd<NameTagsSpec>,
    /// RPC URL of local node.
    pub rpc_url: &'static str,
}

/// Represents historical activity data for a single address.
#[derive(Debug, Clone, PartialEq)]
pub struct AddressHistory {
    /// Address that a user wants to explore.
    pub address: &'static str,
    /// Holds information for all transactions relevant to the address.
    pub transactions: Vec<TxInfo>,
    /// Settings and configurations.
    pub config: Config,
    /// A Cache of things looked up.
    pub cache: Cache,
}

/// A resource may have been looked up before. This stores the result of that attempt.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum VisitNote {
    #[default]
    NotVisited,
    PriorSuccess,
    PriorFailure,
}

impl Config {
    /// Sets up TODD databases with the option for Sample, Default or Custom directories.
    pub fn new(directory_nature: DirNature, rpc_url: &'static str) -> Result<Self> {
        Ok(Config {
            appearances_db: Todd::init(
                DataKind::AddressAppearanceIndex(Network::default()),
                directory_nature.clone(),
            )?,
            signatures_db: Todd::init(DataKind::Signatures, directory_nature.clone())?,
            nametags_db: Todd::init(DataKind::NameTags, directory_nature)?,
            rpc_url,
        })
    }
}

impl AddressHistory {
    pub fn new(address: &'static str, config: Config) -> Self {
        AddressHistory {
            address,
            transactions: vec![],
            config,
            cache: Cache::default(),
        }
    }
    /// Find the appearances for this address.
    ///
    /// Uses an index of address appearances.
    pub fn get_transaction_ids(&mut self) -> Result<&mut Self> {
        let values = self.config.appearances_db.find(self.address)?;
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
    /// Uses eth_getTransactionByBlockNumberAndIndex on local node.
    ///
    /// Number of transactions to get data for can be capped.
    pub async fn get_transaction_data(&mut self, cap_num: Option<u32>) -> Result<&mut Self> {
        let transport = Http::new(self.config.rpc_url)?;
        let web3 = Web3::new(transport);
        let mut txs_with_data = vec![];
        for (i, tx) in self.transactions.iter().enumerate() {
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

            let tx = TxInfo {
                location: tx.location.clone(),
                description: Some(tx_data),
                receipt: None,
                events: None,
            };
            txs_with_data.push(tx);
        }
        self.transactions = txs_with_data;
        for t in &self.transactions {
            debug!("{:?}", t.description);
        }
        Ok(self)
    }
    /// Get the receipts of transactions from a node.
    ///
    /// Uses eth_getTransactionReceipt on local node.
    ///
    /// Number of transactions to get receipts for can be capped.
    pub async fn get_receipts(&mut self, cap_num: Option<u32>) -> Result<&mut Self> {
        let transport = Http::new(self.config.rpc_url)?;
        let web3 = Web3::new(transport);
        let mut txs_with_data: Vec<TxInfo> = vec![];
        for (i, tx) in self.transactions.iter().enumerate() {
            if let Some(cap) = cap_num {
                if i > cap as usize {
                    break;
                }
            }
            let Some(description) = &tx.description else {
                continue
            };
            // eth_getTransactionReceipt
            let tx_receipt = web3
                .eth()
                .transaction_receipt(description.hash)
                .await?
                .ok_or_else(|| anyhow!("No receipt for this transaction hash."))?;
            let mut tx_new = tx.clone();
            tx_new.receipt = Some(tx_receipt);
            txs_with_data.push(tx_new);
        }
        self.transactions = txs_with_data;
        for t in &self.transactions {
            debug!("{:?}", t.receipt);
        }
        Ok(self)
    }
    /// Decodes the event signatures of the logs for each transaction
    ///
    /// Every logged event originates from a contract. That contract
    /// is obtained with ethGetCode and useful information is stored
    /// alongside the event.
    pub async fn decode_logs(&mut self, cap_num: Option<u32>, mode: Mode) -> Result<&mut Self> {
        let transport = Http::new(self.config.rpc_url)?;
        let web3 = Web3::new(transport);
        let mut txs_with_data: Vec<TxInfo> = vec![];
        for (i, tx) in self.transactions.iter().enumerate() {
            if let Some(cap) = cap_num {
                if i > cap as usize {
                    break;
                }
            }
            let Some(receipt) = &tx.receipt else {continue};
            let mut events: Vec<LoggedEvent> = vec![];
            for log in receipt.logs.clone() {
                let event = examine_log(&log, &mode, &web3, &self.config, &mut self.cache).await?;
                let Some(e) = event else {continue};
                events.push(e)
            }
            let mut tx_new = tx.clone();
            tx_new.events = Some(events);
            txs_with_data.push(tx_new);
        }
        self.transactions = txs_with_data;
        for t in &self.transactions {
            debug!("{:?}", t.events);
        }
        Ok(self)
    }
}

impl Display for AddressHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let a = self.address;
        write!(
            f,
            "There are {} txs for address: {}",
            self.transactions.len(),
            a
        )?;
        for (i, tx) in self.transactions.iter().enumerate() {
            write!(f, "\n\nTransaction {}:", i)?;
            let Some(desc) = &tx.description else {continue};
            let Some(receipt) = &tx.receipt else {continue};
            let Some(events) = &tx.events else {continue};
            write!(f, "\n\tSender: {}", nice_address(desc.from, a))?;
            write!(f, "\n\tRecipient: {}", nice_address(receipt.to, a))?;
            write!(
                f,
                "\n\tContract: {}",
                nice_address(receipt.contract_address, a)
            )?;
            write!(f, "\n\tTx Hash: {}", hex::encode(desc.hash))?;
            let event_count = events.len();
            write!(f, "\n\tEvents emitted: {}", event_count)?;
            for (i, e) in events.iter().enumerate() {
                if i > 10 {
                    write!(
                        f,
                        "\n\t\tSkipping remaining {} events...",
                        event_count - i - 1
                    )?;
                    break;
                }
                write!(f, "\n\n\t\t{}. Event {}/{}", e, i, event_count)?;
            }
        }
        write!(f, "")
    }
}

/// Makes an address option nice to read and detects if it is the owner.
fn nice_address(address: Option<H160>, owner_address: &str) -> String {
    let owner_address = owner_address.trim_start_matches("0x");
    match address {
        Some(a) => {
            let a = hex::encode(a);
            if a == owner_address {
                String::from("Self")
            } else {
                format!("0x{}", a)
            }
        }
        None => String::from("None"),
    }
}

/// Extracts the information about a given log.
async fn examine_log(
    log: &Log,
    mode: &Mode,
    web3: &Web3<Http>,
    config: &Config,
    cache: &mut Cache,
) -> Result<Option<LoggedEvent>> {
    let topic_zero = match log.topics.get(0) {
        Some(t) => {
            let s = hex::encode(t);
            s[..8].to_owned()
        }
        None => return Ok(None),
    };
    let raw = log.clone();

    // eth_getCode
    let bytecode = web3
        .eth()
        .code(log.address, Some(BlockNumber::Latest))
        .await?
        .0;

    let cid = match cid_from_runtime_bytecode(bytecode.as_ref()) {
        Ok(c) => c,
        Err(e) => {
            log::error!(
                "The metadata CID was not able to be extracted from bytecode
for contract 0x{}. ({})",
                hex::encode(log.address),
                e
            );
            None
        }
    };
    let address = h160_to_string(&log.address);

    let abi = cache.try_abi(&log.address, mode, &bytecode).await;
    let sig_text = cache.try_sig(&topic_zero, mode, config).await;
    let nametags = cache.try_nametags(&log.address, config);

    let contract = Contract {
        address: address.to_owned(),
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
        name: sig_text,
        nametags,
    };
    Ok(Some(event))
}

/// Uses TODD Signatures database to convert hex string to text string.
///
/// Input: "abcd1234",  no leading "0x".
pub fn sig_to_text(sig: &str, config: &Config) -> Result<Option<String>> {
    let val = config.signatures_db.find(sig)?;
    let mut s = String::new();
    for v in &val {
        s.extend(v.texts_as_strings()?);
    }
    if val.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

/// Uses TODD nametags database to convert address to names and tags.
pub fn address_nametags(address: &str, config: &Config) -> Result<Vec<String>> {
    let val = config.nametags_db.find(address)?;
    let mut s = vec![];
    for v in val {
        s.extend(v.names_as_strings()?);
        s.extend(v.tags_as_strings()?)
    }
    Ok(s)
}
