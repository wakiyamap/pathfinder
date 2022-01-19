//! generates trees from a seedable random number generator, suitable for ingestion to
//! py/src/generate_test_global_tree.py and examples/merkle_global_tree.rs OR the storage variant.

use num_bigint::RandBigInt;
use rand::{Rng, SeedableRng};
use std::fmt;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
#[structopt(
    name = "tree_tool",
    about = "Generates input files for differential testing between pathfinder and cairo-lang."
)]
struct Options {
    /// The seed to reproduce. Default is to generate a new seed, and produce a new document.
    /// Seed is unprefixed 64 bytes of hex.
    #[structopt(long = "seed", parse(try_from_str = parse_seed))]
    seed: Option<[u8; 32]>,

    /// The kind of the document generated; "contract" for two columns, or "global" for three
    /// columns.
    kind: DocumentKind,
}

fn parse_seed(s: &str) -> Result<[u8; 32], hex::FromHexError> {
    let mut out = [0u8; 32];
    hex::decode_to_slice(s, &mut out)?;
    Ok(out)
}

fn main() {
    let opts = Options::from_args();

    let seed = opts.seed.unwrap_or_else(|| {
        // thread_rng algorithm isn't documented, so it cannot be trusted to stable seedable outputs
        // we use it to get a next seed for the actual document generation.
        let mut rng = rand::thread_rng();

        let mut seed = <<rand_chacha::ChaCha8Rng as SeedableRng>::Seed>::default();
        rng.fill(&mut seed);
        seed
    });

    let rng = rand_chacha::ChaCha8Rng::from_seed(seed);

    generate_doc(opts.kind, rng, &seed);
}

fn generate_doc<R: Rng>(kind: DocumentKind, mut rng: R, seed: &[u8]) {
    // always include the seed, in case you find a non-working one
    println!("# chacha8 seed: {:?}", Hex(&seed));

    let count = rng.gen_range(1..=1024);

    println!("# count: {}", count);

    // this is the field modulus
    let modulus = num_bigint::BigUint::from_str(
        "3618502788666131213697322783095070105623107215331596699973092056135872020481",
    )
    .unwrap();

    let columns = usize::from(kind);

    for _ in 0..=count {
        let mut first = true;

        // two for storage trees, three for global trees
        for _ in 0..columns {
            if first {
                first = false;
            } else {
                print!(" ");
            }

            let num = rng.gen_biguint_below(&modulus);
            let num = num.to_bytes_be();
            print!("0x{:?}", Hex(&num));
        }

        println!();
    }
}

#[derive(StructOpt)]
enum DocumentKind {
    ContractStorage,
    GlobalTree,
}

impl From<DocumentKind> for usize {
    fn from(d: DocumentKind) -> usize {
        use DocumentKind::*;
        match d {
            ContractStorage => 2,
            GlobalTree => 3,
        }
    }
}

impl std::str::FromStr for DocumentKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "contract" => DocumentKind::ContractStorage,
            "global" => DocumentKind::GlobalTree,
            _ => return Err("invalid document kind, either 'contract' or 'global'"),
        })
    }
}

struct Hex<'a>(&'a [u8]);

impl fmt::Debug for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in self.0.len()..32 {
            write!(f, "00")?;
        }
        self.0.iter().try_for_each(|&b| write!(f, "{:02x}", b))
    }
}
