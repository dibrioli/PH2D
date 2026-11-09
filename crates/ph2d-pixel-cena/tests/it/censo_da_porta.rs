//! Censo: a porta `tag_do_aseprite` é a única que constrói uma `AnimationTag` no import de `.ase`
//! e nesta crate (P14 §7).

const SITIOS: [&str; 2] = [
    concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shells/desktop/src/ase_import.rs"
    ),
    concat!(env!("CARGO_MANIFEST_DIR"), "/src"),
];

fn ficheiros(p: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if p.is_dir() {
        for e in std::fs::read_dir(p).expect("pasta") {
            ficheiros(&e.expect("entrada").path(), out);
        }
    } else if p.extension().is_some_and(|x| x == "rs") {
        out.push(p.to_path_buf());
    }
}

#[test]
fn zero_animation_tag_construida_fora_da_porta() {
    let mut fs = Vec::new();
    for s in SITIOS {
        ficheiros(std::path::Path::new(s), &mut fs);
    }
    assert!(fs.len() >= 3, "o censo varre zero: {fs:?}");
    let agulha = ["AnimationTag", " {"].concat();
    let fora: Vec<_> = fs
        .iter()
        .filter(|f| std::fs::read_to_string(f).expect("lê").contains(&agulha))
        .collect();
    assert!(
        fora.is_empty(),
        "AnimationTag construída fora da porta: {fora:?}"
    );
}
