//! generates trees from a seedable random number generator, suitable for ingestion to
//! py/src/generate_test_global_tree.py and examples/merkle_global_tree.rs OR the storage variant.

use num_bigint::RandBigInt;
use rand::{Rng, SeedableRng};
use std::str::FromStr;

fn main() {
    let mut rng = rand::thread_rng();

    let mut seed = <<rand_chacha::ChaCha8Rng as SeedableRng>::Seed>::default();
    rng.fill(&mut seed);

    let mut rng = rand_chacha::ChaCha8Rng::from_seed(seed);
    println!("# chacha8 seed: {:?}", Hex(&seed));

    let count = rng.gen_range(1..=1024);

    println!("# count: {}", count);

    let modulus = num_bigint::BigUint::from_str(
        "3618502788666131213697322783095070105623107215331596699973092056135872020481",
    )
    .unwrap();
    // let mut bytes = [0u8; 32];

    for _ in 0..=count {
        let mut first = true;

        // two for storage trees, three for global trees
        for _ in 0..3 {
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

struct Hex<'a>(&'a [u8]);

use std::fmt;
impl fmt::Debug for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in self.0.len()..32 {
            write!(f, "00")?;
        }
        self.0.iter().try_for_each(|&b| write!(f, "{:02x}", b))
    }
}
