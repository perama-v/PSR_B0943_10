use std::collections::HashMap;

use log::{debug, error};

use crate::{
    apis::method_from_fourbyte_api,
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
    /// Attempt to look up a signature if not in cache.
    pub async fn try_sig(&mut self, sig: &str, mode: &Mode, config: &Config) -> Option<String> {
        match self.signatures.get(sig) {
            Some((VisitNote::PriorSuccess, value)) => {
                debug!("Using cached signature: {} {}", sig, value);
                return Some(value.to_owned());
            }
            Some((VisitNote::PriorFailure, _)) => {
                debug!("(skipping) Prior failure for signature: {}", sig);
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
    pub fn try_nametags(&mut self, address: &str, config: &Config) -> Option<Vec<String>> {
        match self.nametags.get(address) {
            Some((VisitNote::PriorSuccess, value)) => {
                debug!("Using cached nametag: {} {:?}", address, value);
                return Some(value.to_owned());
            }
            Some((VisitNote::PriorFailure, _)) => {
                debug!("(skipping) Prior failure for nametag: {}", address);
                return None;
            }
            _ => {}
        }

        match address_nametags(&address, config) {
            Ok(n) => {
                self.nametags
                    .insert(address.to_owned(), (VisitNote::PriorSuccess, n.to_owned()));
                Some(n)
            }
            Err(e) => {
                error!("Couldn't get nametag for address: {} ({})", &address, e);
                self.nametags.insert(
                    address.to_owned(),
                    (VisitNote::PriorFailure, vec![String::from("")]),
                );
                None
            }
        }
    }
}
