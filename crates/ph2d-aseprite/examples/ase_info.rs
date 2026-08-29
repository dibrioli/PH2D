//! **O que há dentro de um `.ase`** — o diagnóstico que responde *«porque é que este ficheiro não
//! importou como eu esperava?»* sem abrir o app.
//!
//! ```text
//! cargo run -p ph2d-aseprite --example ase_info -- <ficheiro.ase | pasta>
//! ```
//!
//! ⚠️ Ele corre o **mesmo** [`ph2d_aseprite::parse`] que o import do produto: o que ele imprime é o
//! que o app vai ver. Uma segunda leitura, mais permissiva, seria uma segunda resposta a *«o que
//! este ficheiro tem»* — e a que o artista lê seria a errada.

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_none() {
        eprintln!("usage: ase_info <file.ase | directory>");
        std::process::exit(2);
    }
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for a in args {
        let p = std::path::PathBuf::from(a);
        if p.is_dir() {
            if let Ok(rd) = std::fs::read_dir(&p) {
                for e in rd.flatten() {
                    let q = e.path();
                    if q.extension().and_then(|s| s.to_str()).is_some_and(|x| {
                        x.eq_ignore_ascii_case("ase") || x.eq_ignore_ascii_case("aseprite")
                    }) {
                        files.push(q);
                    }
                }
            }
        } else {
            files.push(p);
        }
    }
    files.sort();
    let (mut ok, mut bad) = (0_u32, 0_u32);
    for f in &files {
        let name = f.file_name().and_then(|s| s.to_str()).unwrap_or("?");
        let bytes = match std::fs::read(f) {
            Ok(b) => b,
            Err(e) => {
                println!("[X] {name}: read: {e}");
                bad += 1;
                continue;
            }
        };
        match ph2d_aseprite::parse(&bytes) {
            Ok(doc) => {
                ok += 1;
                let ms: Vec<u16> = doc.frames.iter().map(|x| x.duration_ms).collect();
                let uniform = ms.windows(2).all(|w| w[0] == w[1]);
                // Quantos pixels do primeiro quadro não são transparentes — a diferença entre «leu»
                // e «leu ALGUMA COISA». Um leitor partido devolve quadros do tamanho certo, vazios.
                let ink = doc.frames.first().map_or(0, |fr| {
                    fr.rgba
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .filter(|p| p[3] != 0)
                        .count()
                });
                println!(
                    "[v] {name}: {}x{} · {} frames · {} tags · {} px pintados no 1o quadro · duracao {}",
                    doc.width,
                    doc.height,
                    doc.frames.len(),
                    doc.tags.len(),
                    ink,
                    if uniform {
                        format!("{} ms", ms.first().copied().unwrap_or(0))
                    } else {
                        let (lo, hi) = (
                            ms.iter().copied().min().unwrap_or(0),
                            ms.iter().copied().max().unwrap_or(0),
                        );
                        format!("{lo}..{hi} ms (varia por quadro)")
                    }
                );
                for t in &doc.tags {
                    let dir = ["forward", "reverse", "ping-pong", "ping-pong rev"]
                        .get(usize::from(t.direction))
                        .copied()
                        .unwrap_or("?");
                    println!(
                        "      tag \"{}\" {}..{} {dir}{}{}",
                        t.name,
                        t.from,
                        t.to,
                        if t.repeat > 0 {
                            format!(" x{}", t.repeat)
                        } else {
                            String::new()
                        },
                        match t.uniform_duration_ms(&doc.frames) {
                            Some(v) => format!(" @ {v} ms"),
                            None => format!(
                                " @ ~{} ms (varia dentro da tag)",
                                t.dominant_duration_ms(&doc.frames)
                            ),
                        }
                    );
                }
                for n in &doc.notes {
                    println!("      ! {n}");
                }
            }
            Err(e) => {
                bad += 1;
                println!("[X] {name}: {e}");
            }
        }
    }
    println!(
        "\n{ok} lidos, {bad} recusados, de {} ficheiros",
        files.len()
    );
}
