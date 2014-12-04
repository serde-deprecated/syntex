extern crate syntex;

use std::os;
use std::io::File;

fn main() {
    let src = Path::new("src/hello_world.rs.syntex");
    let dst = Path::new(os::getenv("OUT_DIR").unwrap());

    let mut f = File::open(&src).unwrap();
    let s: String = String::from_utf8(f.read_to_end().unwrap()).unwrap();

    let mut f = File::create(&dst.join("hello_world.rs")).unwrap();
    f.write_str(syntex::expand_str("hello_world", s.as_slice()).as_slice()).unwrap();
}
