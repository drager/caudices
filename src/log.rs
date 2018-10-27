pub fn log(s: &str) {
    if cfg!(target_arch = "wasm32") {
        console!(log, s);
    }
    println!("{}", s);
}
