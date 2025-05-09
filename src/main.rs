mod args;
mod derive;
mod diagram;
mod encrypt;
mod input;
mod unicode;

use args::{DeriveCommand, DiagramCommand, EncryptCommand};
use artimonist::{ComplexDiagram, Matrix, SimpleDiagram};
use clap::{Parser, Subcommand};
use diagram::{DiagramOutput, MatrixInput};
use encrypt::Encryptor;
use input::Input;

const ABOUT_LONG: &str = "
Artimonist
A tool for generating mnemonics and wallets.

Project location: <https://github.com/artimonist/artimonist-cli>
Web version: <https://www.artimonist.org>";

/// Artimonist - A tool for generating mnemonics and wallets.   
#[derive(Parser)]
#[command(version, long_about=ABOUT_LONG)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Use simple diagram of 7 * 7 chars
    Simple(DiagramCommand),
    /// Use complex diagram of 7 * 7 strings
    Complex(DiagramCommand),
    /// Encrypt private key by bip38
    Encrypt(EncryptCommand),
    /// Decrypt private key by bip38
    Decrypt(EncryptCommand),
    /// Derive from master key or mnemonic
    Derive(DeriveCommand),
}

macro_rules! confirm_overwrite {
    ($output: expr) => {
        if let Some(path) = $output {
            if std::fs::exists(path)? && !Input::confirm_overwrite("File exists.")? {
                return Ok(());
            }
        }
    };
}

fn main() -> Result<(), CommandError> {
    let args = Cli::parse();
    match args.command {
        Commands::Simple(mut cmd) => {
            confirm_overwrite!(&cmd.output);
            let mx = match &cmd.file {
                Some(file) => Matrix::<char>::from_file(file)?,
                None => Matrix::<char>::by_inquire()?,
            };
            if cmd.has_mnemonic() {
                cmd.language = Input::choice_language()?;
            }
            cmd.password = Input::password(true)?;
            match &cmd.output {
                Some(path) => SimpleDiagram(mx).to_file(&cmd, path)?,
                None => SimpleDiagram(mx).display(&cmd)?,
            }
        }
        Commands::Complex(mut cmd) => {
            confirm_overwrite!(&cmd.output);
            let mx = match &cmd.file {
                Some(file) => Matrix::<String>::from_file(file)?,
                None => Matrix::<String>::by_inquire()?,
            };
            if cmd.has_mnemonic() {
                cmd.language = Input::choice_language()?;
            }
            cmd.password = Input::password(true)?;
            match &cmd.output {
                Some(path) => ComplexDiagram(mx).to_file(&cmd, path)?,
                None => ComplexDiagram(mx).display(&cmd)?,
            }
        }
        Commands::Encrypt(mut cmd) => {
            if !artimonist::NETWORK.is_mainnet() {
                return Ok(());
            }
            if !check_private_key(&cmd.key, false) {
                println!("Invalid private key");
                return Ok(());
            }
            cmd.password = Input::password(false)?;
            cmd.execute(true)?;
        }
        Commands::Decrypt(mut cmd) => {
            if !artimonist::NETWORK.is_mainnet() {
                return Ok(());
            }
            if !check_private_key(&cmd.key, true) {
                println!("Invalid private key");
                return Ok(());
            }
            cmd.password = Input::password(false)?;
            cmd.execute(false)?;
        }
        Commands::Derive(mut cmd) => {
            if !check_master_key(&cmd.key) && !check_mnemonic(&cmd.key) {
                println!("Invalid master key or mnemonic");
                return Ok(());
            }
            confirm_overwrite!(&cmd.output);
            if artimonist::NETWORK.is_mainnet() && !cmd.is_multisig() {
                cmd.password = Input::password(true)?;
            }
            cmd.execute()?;
        }
    }
    Ok(())
}

#[inline]
fn check_private_key(s: &Option<String>, encrypted: bool) -> bool {
    let s = s.as_ref().map_or("".to_owned(), |v| v.to_string());
    match encrypted {
        true => s.starts_with("6P") && s.len() == 58,
        false => s.starts_with(['K', 'L', '5']) && s.len() == 52,
    }
}
#[inline]
fn check_master_key(s: &str) -> bool {
    s.starts_with("xprv") && s.len() == 111
}
#[inline]
fn check_mnemonic(s: &str) -> bool {
    matches!(s.split_whitespace().count(), 12 | 15 | 18 | 21 | 24)
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum CommandError {
    #[error("file error")]
    File(#[from] std::io::Error),
    #[error("input error")]
    Inquire(#[from] inquire::InquireError),
    #[error("encrypt error")]
    Encrypt(#[from] encrypt::EncryptError),
    #[error("diagram error")]
    Diagram(#[from] diagram::DiagramError),
    #[error("derive error")]
    Derive(#[from] derive::DeriveError),
}

#[cfg(test)]
mod diagram_test {
    use artimonist::{GenericDiagram, SimpleDiagram, Wif, BIP85};

    /// Test compatible with old version data
    #[test]
    fn test_simple() {
        const CHARS: &str = "【1$≈⅞£】";
        static INDICES: [(usize, usize); 7] =
            [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        const MNEMONIC: &str = "face shoot relax patch verify six lion proud income copy strategy primary person sign hint mango bargain soldier lobster change follow vehicle material harvest";
        const WIFS: [&str; 5] = [
            "3Cp9s5u2e2Y4mWEDQKnjn7XidkFqwCAR16, Kxnp8CMBWth5yBZHURj4qiHoQZbiu2vsppbFMGAWv6c3hajtmMor",
            "3MDfN9tXdozXKRiGbDpgWujk6haJXXVXSS, KzUjZbdPGN8UqJTE9UXzpQugKWRMZwRqE3vCqhwJJs1dJ3qXSz3z",
            "35mY6LGhApUhgqd5xw3FR4ngZhjGvZjHMq, L4KcnHRnJFdRjHDuLHoGjQ1Lf82Fs2WUanGtRuZsYQChKXN9cs1t",
            "3EgqQwGyeYBtZTdbaposrRuszsaPju3oBK, KxLnnzRK3hdfJ7kfkE6kHsyLEMMoWLypchyJw92dFRG6z6fvNqL5",
            "3QhuuovyzenmJfyjL257AgDK2n7CG3DJSi, KygF68fiRUuk8W2c7nf3iA5Mxzi4rdijz49MKAp1aZ2nkLHkWJ3J",
        ];
        const XPRIV: &str = "xprv9s21ZrQH143K2NbNten7yUnUKHWKgmqC51sNJYJMhrvyxXcxD6bDk8W33ZGw3nBezrVVLsfaoFC2SuBRCkgX1Hpyn4er6XCGf1L9uTWmpH9";
        const PWDS: [&str; 10] = [
            "sLVP2EgoUWu#8khAuN4F",
            "yo%r9stqLShHW8EXbS1A",
            "7xT5kfHDyqrGQkrV9kku",
            "aBj1kp7Wus&eyZh3Y%g5",
            "pBnRfSRt9FM*rmhmvBkg",
            "j@fEyGzSGF5o#38%H#86",
            "1@oYSzj5DR7cvXHavHHX",
            "$vfj#S3WjQ4vkn4iPrXf",
            "f7mKae76xBMMdKNN3Yt7",
            "zVJMgcxXEUZDwYvayXb*",
        ];

        // Simple diagram compatible with older serializations
        // Matrix use generic serializations
        let diagram = SimpleDiagram::from_values(&CHARS.chars().collect::<Vec<_>>(), &INDICES);
        // let mx = &diagram.0;
        // assert_ne!(diagram.to_bytes().unwrap(), mx.to_bytes().unwrap());

        // simple diagram compatible with older results
        let master = diagram.bip32_master(Default::default()).unwrap();
        let mnemonic = master.bip85_mnemonic(Default::default(), 24, 0).unwrap();
        assert_eq!(mnemonic, MNEMONIC);
        WIFS.into_iter().enumerate().for_each(|(i, s)| {
            let Wif { addr, pk } = master.bip85_wif(i as u32).unwrap();
            assert_eq!(format!("{addr}, {pk}"), s);
        });
        let salt_master = diagram.bip32_master("artimonist".as_bytes()).unwrap();
        assert_eq!(salt_master.bip85_xpriv(0).unwrap(), XPRIV);
        PWDS.into_iter().enumerate().for_each(|(i, s)| {
            let pwd = master.bip85_pwd(Default::default(), 20, i as u32).unwrap();
            assert_eq!(pwd, s);
        });
    }
}
