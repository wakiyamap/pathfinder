//! generates trees from a seedable random number generator, suitable for ingestion to
//! py/src/generate_test_global_tree.py and examples/merkle_global_tree.rs OR the storage variant.

use num_bigint::RandBigInt;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
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

    /// The kind of the document generated; "storage" for two columns, or "global" for three
    /// columns.
    kind: DocumentKind,

    /// Applies for "storage"; the probability of deletion of previously set values. 0 is default.
    #[structopt(long = "deletion-probability")]
    deletion_probability: Option<u8>,
}

fn parse_seed(s: &str) -> Result<[u8; 32], hex::FromHexError> {
    let mut out = [0u8; 32];
    hex::decode_to_slice(s, &mut out)?;
    Ok(out)
}

fn main() {
    let opts = Options::from_args();

    if opts.kind == DocumentKind::GlobalTree {
        assert_eq!(
            opts.deletion_probability, None,
            "deletion_probability doesn't apply to global trees"
        );
    } else if let Some(deletion_probability) = opts.deletion_probability {
        assert!(
            deletion_probability < 100,
            "deletion_probability needs to be under 100"
        );
    }

    let seed = opts.seed.unwrap_or_else(|| {
        // thread_rng algorithm isn't documented, so it cannot be trusted to stable seedable outputs
        // we use it to get a next seed for the actual document generation.
        let mut rng = rand::thread_rng();

        let mut seed = <<rand_chacha::ChaCha8Rng as SeedableRng>::Seed>::default();
        rng.fill(&mut seed);
        seed
    });

    let rng = rand_chacha::ChaCha8Rng::from_seed(seed);

    generate_doc(
        opts.kind,
        rng,
        &seed,
        opts.deletion_probability.map(|x| x as f64 / 100.0),
    );
}

fn generate_doc<R: Rng>(
    kind: DocumentKind,
    mut rng: R,
    seed: &[u8],
    deletion_probability: Option<f64>,
) {
    // always include the seed, in case you find a non-working one
    println!("# chacha8 seed: {:?}", Hex(&seed));

    let count = rng.gen_range(1..=1024);

    println!("# count: {}", count);

    // this is the field modulus
    let _modulus = num_bigint::BigUint::from_str(
        "3618502788666131213697322783095070105623107215331596699973092056135872020481",
    )
    .unwrap();

    let _high_251 = num_bigint::BigUint::from_str(
        "3618502788666131106986593281521497120414687020801267626233049500247285301247",
    )
    .unwrap();

    let deletion = deletion_probability.map(|p| rand::distributions::Bernoulli::new(p).unwrap());
    let mut deletable_keys: HashSet<Vec<u8>> = HashSet::new();
    let mut tmp_deleted = Vec::new();

    let columns = usize::from(kind);

    for _ in 0..=count {
        if !deletable_keys.is_empty() {
            if let Some(true) = deletion.map(|b| rng.sample(b)) {
                let selected = deletable_keys.iter().next().expect("just checked");
                tmp_deleted.clear();
                tmp_deleted.extend(selected.iter().copied());
                assert!(deletable_keys.remove(&tmp_deleted));
                println!("0x{:?} 0x{:0>64}", Hex(&tmp_deleted), 0);
            }
        }

        // two for storage trees, three for global trees
        for i in 0..columns {
            if i > 0 {
                print!(" ");
            }

            let num = rng.gen_biguint_below(&_high_251);
            let raw = num.to_bytes_be();
            print!("0x{:?}", Hex(&raw));

            if i == 0 && deletion_probability.is_some() {
                deletable_keys.insert(raw);
            }
        }

        println!();
    }
}

#[derive(StructOpt, PartialEq)]
enum DocumentKind {
    StorageTree,
    GlobalTree,
}

impl From<DocumentKind> for usize {
    fn from(d: DocumentKind) -> usize {
        use DocumentKind::*;
        match d {
            StorageTree => 2,
            GlobalTree => 3,
        }
    }
}

impl std::str::FromStr for DocumentKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DocumentKind::*;
        Ok(match s {
            "storage" => StorageTree,
            "global" => GlobalTree,
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
