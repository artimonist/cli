mod matrix;
use artimonist::{ComplexDiagram, Error, GenericDiagram, SimpleDiagram, Xpriv, BIP85};
use clap::{Parser, Subcommand, ValueEnum};
use matrix::{FmtTable, Matrix, ToMatrix};

/// Artimonist - A tool for generating mnemonics based on diagrams.   
/// Web version: https://www.artimonist.org
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Use simple diagram of 7 * 7 chars
    Simple {
        /// Target
        #[arg(short, long, default_value = "mnemonic")]
        target: Target,

        /// Start serial number
        #[arg(short, long, default_value_t = 0)]
        serial: u16,

        /// Amount to generate
        #[arg(short = 'm', long, default_value_t = 1)]
        amount: u16,

        /// Salt
        #[arg(short, long)]
        salt: Option<String>,
        // /// Encrypt private key of wif
        // #[arg(short, long, required="wif")]
        // encrypt: Option<String>,
    },
    /// Use complex diagram of 7 * 7 strings
    Complex {
        /// Target
        #[arg(short, long, default_value = "mnemonic")]
        target: Target,

        /// Start serial number
        #[arg(short, long, default_value_t = 0)]
        serial: u16,

        /// Amount to generate
        #[arg(short = 'm', long, default_value_t = 1)]
        amount: u16,

        /// Salt
        #[arg(short, long)]
        salt: Option<String>,
        // /// Encrypt private key of wif
        // #[arg(short, long, required="wif")]
        // encrypt: Option<String>,
    },
    // /// Encrypt private key by bip38
    // Encrypt { key: String, password: String },
    // /// Decrypt private key by bip38
    // Decrypt { key: String, password: String },
}

#[derive(ValueEnum, Clone, Copy, Default, Debug)]
enum Target {
    #[default]
    Mnemonic,
    #[value(alias = "wif")]
    Wallet,
    Xpriv,
    #[value(alias = "pwd")]
    Password,
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    match args.command {
        Commands::Simple {
            target,
            serial,
            amount,
            salt,
        } => {
            let mx = input_simple();
            println!();
            println!("Simple diagram: ");
            println!("{}", mx.fmt_table());
            let master = SimpleDiagram(mx).bip32_master(salt.unwrap_or_default().as_bytes())?;
            println!();
            println!("{:?} results: ", target);
            generate(&master, target, serial as u32, amount as u32)
                .iter()
                .enumerate()
                .for_each(|(i, s)| println!("{}: {s}", i + serial as usize));
        }
        Commands::Complex {
            target,
            serial,
            amount,
            salt,
        } => {
            let mx = input_complex();
            println!();
            println!("Complex diagram: ");
            println!("{}", mx.fmt_table());
            let master = ComplexDiagram(mx).bip32_master(salt.unwrap_or_default().as_bytes())?;
            println!();
            println!("{:?} results: ", target);
            generate(&master, target, serial as u32, amount as u32)
                .iter()
                .enumerate()
                .for_each(|(i, s)| println!("{}: {s}", i + serial as usize));
        } // Commands::Encrypt { key } => {}
          // Commands::Decrypt { key } => {}
    }
    Ok(())
}

/// Input simple diagram
fn input_simple() -> Matrix<7, 7, char> {
    // input
    let lns: Vec<String> = (1..=7)
        .map(|i| {
            inquire::Text::new(&format!("row ({i})"))
                .with_initial_value(&"\"\" ".repeat(7))
                .with_help_message("Fill characters in quotes.")
                .prompt()
                .unwrap()
        })
        .collect();
    // parse
    lns.into_iter()
        .map(|s| {
            s.split_whitespace()
                .map(|v| v.trim_matches('\"').chars().next())
                .collect()
        })
        .collect::<Vec<Vec<_>>>()
        .to_matrix::<7, 7>()
}

/// Input complex diagram
fn input_complex() -> Matrix<7, 7, String> {
    // input
    let lns: Vec<String> = (1..=7)
        .map(|i| {
            inquire::Text::new(&format!("row ({i})"))
                .with_initial_value(&"\"\"  ".repeat(7))
                .with_help_message("Fill characters in quotes.")
                .prompt()
                .unwrap()
        })
        .collect();
    // parse
    lns.into_iter()
        .map(|s| {
            s.split_whitespace()
                .map(|v| match v.trim_matches('\"') {
                    "" => None,
                    s => Some(s.to_owned()),
                })
                .collect()
        })
        .collect::<Vec<Vec<_>>>()
        .to_matrix::<7, 7>()
}

/// Generate target results
fn generate(master: &Xpriv, target: Target, serial: u32, amount: u32) -> Vec<String> {
    (serial..serial + amount)
        .filter_map(|i| {
            match target {
                Target::Mnemonic => master.bip85_mnemonic(Default::default(), 24, i),
                Target::Xpriv => master.bip85_xpriv(i),
                Target::Wallet => master
                    .bip85_wif(i)
                    .map(|v| v.extra_address())
                    .map(|(addr, pk)| format!("{addr}, {pk}")),
                Target::Password => master.bip85_pwd(Default::default(), 20, i),
            }
            .ok()
        })
        .collect()
}

#[cfg(test)]
mod diagram_test {
    use super::*;
    use artimonist::GenericDiagram;

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
        let mx = &diagram.0;
        assert_ne!(diagram.to_bytes().unwrap(), mx.to_bytes().unwrap());

        // simple diagram compatible with older results
        let master = diagram.bip32_master(Default::default()).unwrap();
        let mnemonic = master.bip85_mnemonic(Default::default(), 24, 0).unwrap();
        assert_eq!(mnemonic, MNEMONIC);
        WIFS.into_iter().enumerate().for_each(|(i, s)| {
            let (addr, pk) = master.bip85_wif(i as u32).unwrap().extra_address();
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
