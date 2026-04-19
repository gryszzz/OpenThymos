fn main() {
    if let Err(err) = thymos_tools::worker_entrypoint() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
