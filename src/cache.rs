use std::collections::HashMap;

use anyhow::Result;
use heimdall::decompile::DecompileBuilder;
use log::{debug, error, warn};
use web3::types::H160;

use crate::{
    apis::{abi_from_sourcify_api, method_from_fourbyte_api},
    types::{address_nametags, sig_to_text, Config, Mode, VisitNote},
};

#[derive(Debug, Default, Clone, PartialEq)]
/// A store of things that have been obtained externally, that may arise more than once.
///
/// Each value has a bool
pub struct Cache {
    /// Maps (keccak) signatures to names text names.
    ///
    /// 4 byte signatures "abcd1234" -> "Withdraw()"
    pub signatures: HashMap<String, (VisitNote, String)>,
    /// Maps addresses to text names and tags.
    ///
    /// 20 byte addresses "abcd...1234" -> ("SomeContractName", "Special tag")
    pub nametags: HashMap<String, (VisitNote, Vec<String>)>,
    /// Maps addresses to JSON encoded text ABIs.
    ///
    /// 20 byte addresses "abcd...1234" -> ("{...}")
    pub abis: HashMap<String, (VisitNote, String)>,
}

impl Cache {
    /// Attempt to look up abi if not in cache.
    pub async fn try_abi(
        &mut self,
        address: &H160,
        mode: &Mode,
        config: &Config,
        bytecode: &[u8],
    ) -> Option<String> {
        let address_string = hex::encode(address);
        let address_string = address_string.trim_start_matches("0x");
        match self.abis.get(address_string) {
            Some((VisitNote::PriorSuccess, abi)) => {
                debug!("Using cached ABI: {} {}", address_string, abi);
                return Some(abi.to_owned());
            }
            Some((VisitNote::PriorFailure, _)) => {
                debug!(
                    "(skipping) Prior ABI fetch failure for address: {}",
                    address
                );
                return None;
            }
            _ => {}
        }

        let abi_result = get_abi(address, mode, bytecode).await;

        let abi = match abi_result {
            Ok(a) => a,
            Err(e) => {
                error!("Couldn't get ABI for address: {} ({})", &address_string, e);
                self.abis.insert(
                    address_string.to_owned(),
                    (VisitNote::PriorFailure, String::from("")),
                );
                return None;
            }
        };

        match abi {
            Some(a) => {
                self.abis.insert(
                    address_string.to_owned(),
                    (VisitNote::PriorSuccess, a.to_owned()),
                );
                Some(a)
            }
            None => {
                error!("No ABI found for address: {}", &address_string);
                self.abis.insert(
                    address_string.to_owned(),
                    (VisitNote::PriorFailure, String::from("")),
                );
                return None;
            }
        }
    }

    /// Attempt to look up a signature if not in cache.
    pub async fn try_sig(&mut self, sig: &str, mode: &Mode, config: &Config) -> Option<String> {
        match self.signatures.get(sig) {
            Some((VisitNote::PriorSuccess, value)) => {
                debug!("Using cached signature: {} {}", sig, value);
                return Some(value.to_owned());
            }
            Some((VisitNote::PriorFailure, _)) => {
                debug!("(skipping) Prior text fetch failure for signature: {}", sig);
                return None;
            }
            _ => {}
        }

        let text_result = match mode {
            Mode::AvoidApis => sig_to_text(&sig, config),
            Mode::UseApis => method_from_fourbyte_api(&sig).await,
        };

        let text = match text_result {
            Ok(t) => t,
            Err(e) => {
                error!("Couldn't get text for signature: {} ({})", &sig, e);
                self.signatures
                    .insert(sig.to_owned(), (VisitNote::PriorFailure, String::from("")));
                return None;
            }
        };

        match text {
            Some(t) => {
                self.signatures
                    .insert(sig.to_owned(), (VisitNote::PriorSuccess, t.to_owned()));
                Some(t)
            }
            None => {
                error!("No text found for signature: {}", &sig);
                self.signatures
                    .insert(sig.to_owned(), (VisitNote::PriorFailure, String::from("")));
                return None;
            }
        }
    }
    /// Attempt to look up nametags if not in cache.
    pub fn try_nametags(&mut self, address: &H160, config: &Config) -> Option<Vec<String>> {
        let addr_hex = hex::encode(address);
        match self.nametags.get(&addr_hex) {
            Some((VisitNote::PriorSuccess, value)) => {
                debug!("Using cached nametag: {} {:?}", address, value);
                return Some(value.to_owned());
            }
            Some((VisitNote::PriorFailure, _)) => {
                debug!(
                    "(skipping) Prior nametag fetch failure for nametag: {}",
                    address
                );
                return None;
            }
            _ => {}
        }

        match address_nametags(&addr_hex, config) {
            Ok(n) => {
                self.nametags
                    .insert(addr_hex.to_owned(), (VisitNote::PriorSuccess, n.to_owned()));
                Some(n)
            }
            Err(e) => {
                error!("Couldn't get nametag for address: {} ({})", &address, e);
                self.nametags.insert(
                    addr_hex.to_owned(),
                    (VisitNote::PriorFailure, vec![String::from("")]),
                );
                None
            }
        }
    }
}

/// Gets the ABI for a contract.
///
/// This may take two forms:
/// - `Mode::UseApis` First tries Sourcify then Heimdall (which relies on third party API for
/// four byte signatures)
/// - `Mode::AvoidApis`
pub async fn get_abi(address: &H160, mode: &Mode, bytecode: &[u8]) -> Result<Option<String>> {
    Ok(match mode {
        Mode::UseApis => {
            let abi = abi_from_sourcify_api(address).await?;
            // If no ABI is found at the API, decompile.
            match abi {
                Some(x) => Some(x),
                None => {
                    let bytecode_string = hex::encode(&bytecode);
                    DecompileBuilder::new(&bytecode_string)
                        .output(&format!("decompiled/{}", address))
                        .decompile();
                    warn!("Did not check if decompilation fails.");
                    Some(String::from("TODO: Pull decompiled-ABI from file"))
                }
            }
        }
        Mode::AvoidApis => {
            warn!(
                "ABI not fetched for address {}. Pending integration with TODD-ABI (IPFS) database.",
                address
            );
            Some(String::from("TODO, get TODD-ABIs"))
        }
    })
}
