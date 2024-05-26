use std::env;

use air_infra::airs::examples::bit_unpacking::create_bit_unpacking_json;
use air_infra::airs::examples::fibonacci::create_fibonacci_json;
use air_infra::airs::uint32_utils::create_add32_json;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(
        args.iter()
            .all(|x| x.starts_with("target") || x == "fib" || x == "add32" || x == "bit_unpack"),
        "Usage: cargo run [fib] [add32] [bit_unpack]. Got args: {:?}",
        args
    );

    let create_fib_json = args.iter().any(|x| x == "fib");
    if create_fib_json {
        println!("Creating a json file for fibonacci");
        create_fibonacci_json();
    }

    let create_uint32_utils_json = args.iter().any(|x| x == "add32");
    if create_uint32_utils_json {
        println!("Creating a json file for add32");
        create_add32_json();
    }

    let create_bit_unpack_json = args.iter().any(|x| x == "bit_unpack");
    if create_bit_unpack_json {
        println!("Creating a json file for bit unpacking");
        create_bit_unpacking_json();
    }
}
