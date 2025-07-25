use artimonist::bitcoin;

#[derive(clap::Parser)]
pub struct EncryptCommand<const ENCRYPT: bool> {
    /// Private key in WIF format or encrypted key, or a file containing keys
    #[clap(name = "KEY|FILE_NAME")]
    pub source: EncryptSource,

    /// Password
    #[clap(hide = true, long)]
    pub password: Option<String>,
}

/// Source of encryption/decryption
#[derive(Clone, Debug)]
pub enum EncryptSource {
    /// Private key in WIF format or encrypted key
    Key(String),
    /// Text file containing private keys or encrypted keys
    File(String),
}

impl std::str::FromStr for EncryptSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if is_private_key(s) || is_encrypted_key(s) {
            Ok(EncryptSource::Key(s.to_string()))
        } else if std::path::Path::new(s).exists() {
            Ok(EncryptSource::File(s.to_string()))
        } else {
            Err(format!("Invalid input: {s}"))
        }
    }
}

#[inline(always)]
fn is_private_key(s: &str) -> bool {
    s.starts_with(['K', 'L', '5']) && s.len() == 52 && bitcoin::base58::decode(s).is_ok()
}

#[inline(always)]
fn is_encrypted_key(s: &str) -> bool {
    s.starts_with("6P") && s.len() == 58 && bitcoin::base58::decode(s).is_ok()
}
