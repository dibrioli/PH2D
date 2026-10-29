//! O núcleo não lê relógio, aleatoriedade, SO nem itera por hash (ADR-0196).

const FORBIDDEN: &[&str] = &[
    "std::time",
    "Instant",
    "SystemTime",
    "std::fs",
    "std::env",
    "std::thread",
    "rand::",
    "HashMap",
    "HashSet",
];

#[test]
fn the_pixel_core_never_touches_the_clock_the_os_or_a_hash_order() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = 0;
    for entry in std::fs::read_dir(&src).expect("src") {
        let path = entry.expect("entry").path();
        let text = std::fs::read_to_string(&path).expect("read");
        read += 1;
        for word in FORBIDDEN {
            assert!(!text.contains(word), "{} contém `{word}`", path.display());
        }
    }
    assert!(
        read >= 8,
        "o censo leu {read} ficheiros — o src mudou de forma?"
    );
}

#[test]
fn the_pixel_core_declares_no_dependency() {
    let toml = include_str!("../../Cargo.toml");
    let deps = toml
        .split("[dependencies]")
        .nth(1)
        .expect("[dependencies]")
        .split('[')
        .next()
        .expect("secção");
    let declared: Vec<&str> = deps
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    assert!(
        declared.is_empty(),
        "o núcleo ganhou dependências: {declared:?}"
    );
}
