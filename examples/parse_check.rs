fn main() {
    for p in std::fs::read_dir("data/characters").unwrap().flatten() {
        let path = p.path();
        if path.extension().map(|x| x == "toml").unwrap_or(false) {
            let content = std::fs::read_to_string(&path).unwrap();
            match toml::from_str::<sr_api::Character>(&content) {
                Ok(_) => {}
                Err(e) => { println!("{} => {}", path.display(), e); }
            }
        }
    }
}
