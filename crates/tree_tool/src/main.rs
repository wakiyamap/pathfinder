//! generates trees from a seedable random number generator, suitable for ingestion to
//! py/src/generate_test_global_tree.py and examples/merkle_global_tree.rs OR the storage variant.

use fnv::FnvHashSet;
use num_bigint::RandBigInt;
use rand::{Rng, SeedableRng};
use std::fmt;
use std::io::Write;
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
        std::io::stdout().lock(),
    )
    .unwrap();
}

fn generate_doc<R: Rng, W: Write>(
    kind: DocumentKind,
    mut rng: R,
    seed: &[u8],
    deletion_probability: Option<f64>,
    mut writer: W,
) -> Result<(), std::io::Error> {
    // always include the seed, in case you find a non-working one
    writeln!(writer, "# chacha8 seed: {:?}", Hex(&seed))?;

    let count = rng.gen_range(1..=1024);

    writeln!(writer, "# count: {}", count)?;

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
    let mut deletable_keys: FnvHashSet<Vec<u8>> = FnvHashSet::default();
    let mut tmp_deleted = Vec::new();

    let columns = usize::from(kind);

    for _ in 0..=count {
        if !deletable_keys.is_empty() {
            if let Some(true) = deletion.map(|b| rng.sample(b)) {
                let selected = deletable_keys.iter().next().expect("just checked");
                tmp_deleted.clear();
                tmp_deleted.extend(selected.iter().copied());
                assert!(deletable_keys.remove(&tmp_deleted));
                writeln!(writer, "0x{:?} 0x{:0>64}", Hex(&tmp_deleted), 0)?;
            }
        }

        // two for storage trees, three for global trees
        for i in 0..columns {
            if i > 0 {
                write!(writer, " ")?;
            }

            let num = rng.gen_biguint_below(&_high_251);
            let raw = num.to_bytes_be();
            write!(writer, "0x{:?}", Hex(&raw))?;

            if i == 0 && deletion_probability.is_some() {
                deletable_keys.insert(raw);
            }
        }

        writeln!(writer)?;
    }

    Ok(())
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

#[test]
fn assert_doc_is_stable() {
    let seed_with_small_count =
        parse_seed("2576142f5bdc944cb6acfd95a3e02ce92475a58251c4a88aed77ac0a04fdef87").unwrap();

    let tests = [
        (line!(), DocumentKind::StorageTree, None, "# chacha8 seed: 2576142f5bdc944cb6acfd95a3e02ce92475a58251c4a88aed77ac0a04fdef87\n# count: 3\n0x030f78472d86ad4c0ddefa794216cc89b4c8c0a918832cf1a7ef312107af3c3e 0x00c38c9703034764819d751a8e0114af59135bbe778423c077f0730a79a4bd0f\n0x04b13e4d5659001ba7387d4f4952d9ad2c4083c487bb391f922665d6da09f2c0 0x0572f31b14ffef8a86e4551d0a0c5c3f7a25091c9d43b06478a7442d8cc09ecb\n0x01dbac00c8e4507aec7bc39cf5e2b183152fc15093fea0d589bf99eaa0224336 0x00ed25966d722b2b49b49a73ef3563b793d4ce928d4c237d34e1504954f34d39\n0x00d9bcd77f071dd368a19e9cf15752d02c189e63e5bc9f8ed4e504f63a714e8f 0x00b450a5716102a64fceb14bc5baf969b0067c339bad4cad1c2c17356852910c\n"),
        (line!(), DocumentKind::StorageTree, Some(0.5), "# chacha8 seed: 2576142f5bdc944cb6acfd95a3e02ce92475a58251c4a88aed77ac0a04fdef87\n# count: 3\n0x030f78472d86ad4c0ddefa794216cc89b4c8c0a918832cf1a7ef312107af3c3e 0x00c38c9703034764819d751a8e0114af59135bbe778423c077f0730a79a4bd0f\n0x03c53a218cc09ecb9627c9af5659001ba7387d4f4952d9ad2c4083c487bb391f 0x044dfccfa0224336ae5e636d14ffef8a86e4551d0a0c5c3f7a25091c9d43b064\n0x030f78472d86ad4c0ddefa794216cc89b4c8c0a918832cf1a7ef312107af3c3e 0x0000000000000000000000000000000000000000000000000000000000000000\n0x049ea6748d4c237d34e1504954f34d393b758002c8e4507aec7bc39cf5e2b183 0x0160c4f3e5bc9f8ed4e504f63a714e8f1da4b2d36d722b2b49b49a73ef3563b7\n0x049ea6748d4c237d34e1504954f34d393b758002c8e4507aec7bc39cf5e2b183 0x0000000000000000000000000000000000000000000000000000000000000000\n0x027e758ac5baf969b0067c339bad4cad1c2c17356852910c1b379afe7f071dd3 0x02946cfab1b691ee340dd25952f3dc0aa973e7f985dd64c4168a14a0716102a6\n"),
        (line!(), DocumentKind::GlobalTree, None, "# chacha8 seed: 2576142f5bdc944cb6acfd95a3e02ce92475a58251c4a88aed77ac0a04fdef87\n# count: 3\n0x030f78472d86ad4c0ddefa794216cc89b4c8c0a918832cf1a7ef312107af3c3e 0x00c38c9703034764819d751a8e0114af59135bbe778423c077f0730a79a4bd0f 0x04b13e4d5659001ba7387d4f4952d9ad2c4083c487bb391f922665d6da09f2c0\n0x0572f31b14ffef8a86e4551d0a0c5c3f7a25091c9d43b06478a7442d8cc09ecb 0x01dbac00c8e4507aec7bc39cf5e2b183152fc15093fea0d589bf99eaa0224336 0x00ed25966d722b2b49b49a73ef3563b793d4ce928d4c237d34e1504954f34d39\n0x00d9bcd77f071dd368a19e9cf15752d02c189e63e5bc9f8ed4e504f63a714e8f 0x00b450a5716102a64fceb14bc5baf969b0067c339bad4cad1c2c17356852910c 0x03a70f10ef7c5278528d9f41b1b691ee340dd25952f3dc0aa973e7f985dd64c4\n0x028736b148e8e31928016403fd2b5e314035a15eb6f4420872827e4d22d431d4 0x04523d7bcac328849fc38359a8e2749c7d57b7aa340fb9c248a13ccd639e3afb 0x044f5e9e5acb0e4f6e2f5fcd0b83e2b880e2be5b42f03a5a4d55f657e9fc09dd\n")
    ];

    let mut doc = Vec::new();

    for (line, kind, p, expected) in tests {
        doc.clear();

        let rng = rand_chacha::ChaCha8Rng::from_seed(seed_with_small_count);

        generate_doc(kind, rng, &seed_with_small_count, p, &mut doc).unwrap();

        let s = std::str::from_utf8(&doc).unwrap();

        assert_eq!(s, expected, "example on line {}", line);
    }
}
