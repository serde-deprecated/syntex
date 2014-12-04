include!(concat!(env!("OUT_DIR"), "/hello_world.rs"))

#[test]
fn test() {
    let x = hello_world();
    println!("x: {}", x);
}
