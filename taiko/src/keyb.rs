use std::process::Command;
pub fn don_fn() {
    println!("Don");
    type_key("j");
}
pub fn ka_fn() {
    println!("Ka");
    type_key("k");
}

pub fn type_key(key: &str) {
    let _cmd = Command::new("ydotool")
        .args(["type", key]).output().expect("dang");
}