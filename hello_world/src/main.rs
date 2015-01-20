mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/hello_world.rs"));
}

fn main() {
    let x = hello_world::hello_world();
    println!("x: {}", x);
}
