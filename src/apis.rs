/*!
## External data sources
- Contract ABI is pulled from https://www.sourcify.dev
- Event signatures are pulled from https://4byte.directory

IPFS would ideally replace these sources, not done here to proceed with
proof of concept.

Some ideas for both would be to have sourcify and 4byte both publish
annual immutable "editions" where volumes of their data could
be downloaded and pinned more readily, without CIDs changing. This
might improve data availability on IPFS by allowing more participants.
*/
use std::str::FromStr;

use anyhow::{bail, Result};
use reqwest::{header::CONTENT_TYPE, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web3::types::H160;

use crate::parsing::{as_checksummed, summary_of_abi_from_json};

const FOURBYTE: &str = "https://www.4byte.directory/api/v1/event-signatures/";
const SOURCIFY_FULL: &str = "https://repo.sourcify.dev/contracts/full_match/1/";
const SOURCIFY_PARTIAL: &str = "https://repo.sourcify.dev/contracts/partial_match/1/";

#[derive(Serialize, Deserialize, Debug)]
/// Response for a match query on event signatures at 4byte.directory.
pub struct FourBytePage {
    next: Option<String>,
    previous: Option<u32>,
    count: Option<u32>,
    results: Vec<FourByteResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
/// Content for a single match at 4byte.directory
pub struct FourByteResponse {
    id: u32,
    created_at: String,
    text_signature: String,
    hex_signature: String,
    bytes_signature: String,
}

/// Returns the first match from 4byte api for an event/topic hash.
///
/// Example endpoint:
///
/// https://www.4byte.directory/api/v1/event-signatures/?hex_signature=0xe1fffcc4
///
/// ## Hash collisions
/// Each decoded candidate response is hashed and compared to the full 32 byte signature
/// (present in the transaction log).
pub async fn method_from_fourbyte_api(topic: &str) -> Result<Option<String>> {
    let hex_sig = format!("0x{}", topic);
    let url = Url::from_str(FOURBYTE)?;
    let client = reqwest::Client::new();
    let response: FourBytePage = client
        .get(url)
        .query(&[("hex_signature", hex_sig)])
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?
        .json()
        .await?;
    // Hash to check each decoded response.
    for r in response.results {
        let target = hex::encode(topic);
        let candidate_full_hash = r.hex_signature.trim_start_matches("0x");
        if candidate_full_hash == target {
            return Ok(Some(r.text_signature));
        }
    }
    Ok(None)
}

/// Returns the sourcify url target for a given contract address.
pub async fn abi_from_sourcify_api(address: &H160) -> Result<Option<String>> {
    let client = reqwest::Client::new();
    let a = format!("{}/{}", as_checksummed(address), "metadata.json");

    let url = Url::from_str(SOURCIFY_FULL)?.join(&a)?;
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await;
    let Ok(r) = response else {bail!("The request failed for {}", a)};
    if let StatusCode::OK = r.status() {
        let v: Value = r.json().await?;
        let contract_summary = summary_of_abi_from_json(v).unwrap();
        return Ok(Some(contract_summary));
    }

    // May not match on full
    let url = Url::from_str(SOURCIFY_PARTIAL)?.join(&a)?;
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await;
    let Ok(r) = response else {bail!("The request failed for {}", a)};
    if let StatusCode::OK = r.status() {
        let v: Value = r.json().await?;
        let contract_summary = summary_of_abi_from_json(v).unwrap();
        Ok(Some(contract_summary))
    } else {
        // println!("Status code: {} for request for partial match", r.status());
        Ok(None)
    }
}
