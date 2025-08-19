use serde_cbor::ser::Serializer;

fn main() {
    let mut buf = Vec::new();
    let mut ser = Serializer::new(&mut buf);
    let _ = ser.self_describe();
    println!("ok");
}
