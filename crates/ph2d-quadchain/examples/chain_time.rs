//! ⭐⭐⭐ **ONDE O TEMPO DO BOTÃO VAI** — a cadeia canónica, fase a fase.
//!
//! ```text
//! cargo run --release -p ph2d-quadchain --example chain_time -- <peca.obj> [alvo]
//! ```
//!
//! ⚠️ **Ela corre a MESMA função que o produto** ([`ph2d_quadchain::quads_from_mesh`]) — uma
//! sonda que reescrevesse a ordem mediria outro programa. ⛔ O botão da escultura corre a
//! cadeia **duas** vezes (campo alinhado e campo liso), então o relógio dele é ~`2×` isto.

fn load(name: &str) -> ph2d_mesh::Mesh {
    if let Some(rest) = name.strip_prefix("esfera:") {
        let n: usize = rest.parse().unwrap_or(48);
        return ph2d_mesh::shapes::uv_sphere(n, n * 3 / 2, 1.0);
    }
    let text = std::fs::read_to_string(name).unwrap_or_else(|e| panic!("{name}: {e}"));
    ph2d_mesh::import_obj(&text)
        .unwrap_or_else(|e| panic!("{name} nao e' um OBJ deste leitor: {e:?}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{name} nao tem peca dentro"))
        .mesh
}

fn main() {
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| String::from("esfera:48"));
    let scale: f32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut piece = load(&name);
    piece.triangulate();
    let target = ph2d_remesh_iso::target_edge(&piece, ph2d_remesh_iso::ALPHA) * scale;
    // ⭐⭐⭐ **DUAS CADEIAS INDEPENDENTES, em série e em paralelo** (`duas` como 3.º
    // argumento) — é o que o botão faz (campo alinhado e campo liso) e a pergunta é se o
    // paralelo paga. ⚠️ *Não é uma cópia da ordem*: são duas invocações da **porta**.
    if std::env::args().nth(3).as_deref() == Some("duas") {
        let serie = std::time::Instant::now();
        let a = ph2d_quadchain::quads_from_mesh(&piece, target);
        let b = ph2d_quadchain::quads_from_mesh(&piece, target);
        let t_serie = serie.elapsed().as_secs_f32() * 1000.0;
        let par = std::time::Instant::now();
        let (c, d) = rayon::join(
            || ph2d_quadchain::quads_from_mesh(&piece, target),
            || ph2d_quadchain::quads_from_mesh(&piece, target),
        );
        let t_par = par.elapsed().as_secs_f32() * 1000.0;
        let q = |r: &Result<(ph2d_mesh::Mesh, ph2d_quadchain::ChainReport), _>| {
            r.as_ref().map_or(0, |(_, x)| x.quads)
        };
        println!(
            "{name}: DUAS cadeias -> serie {t_serie:.0} ms · paralelo {t_par:.0} ms ({:.2}x) \
             | quads {} {} {} {}",
            t_serie / t_par.max(1.0e-6),
            q(&a),
            q(&b),
            q(&c),
            q(&d)
        );
        return;
    }
    let clock = std::time::Instant::now();
    match ph2d_quadchain::quads_from_mesh(&piece, target) {
        Err(e) => println!("{name}: a cadeia recusou: {e:?}"),
        Ok((out, r)) => {
            let t = r.ms;
            let all = t.total().max(1.0e-6);
            let pct = |x: f32| 100.0 * x / all;
            println!(
                "{name} (alvo x{scale}) -> {} quads em {:.0} ms (relogio de parede {:.0} ms)",
                r.quads,
                all,
                clock.elapsed().as_secs_f32() * 1000.0
            );
            for (nome, v) in [
                ("F1 remalha ", t.remesh),
                ("F2 campo   ", t.field),
                ("F3 tracado ", t.trace),
                ("G1/G2 corte", t.cut),
                ("G3/G5 mapa ", t.map),
                ("extraccao  ", t.extract),
                ("ACABAMENTO ", t.finish),
            ] {
                println!("   {nome} {v:8.0} ms  {:5.1}%", pct(v));
            }
            println!(
                "   * acabamento: {} rondas, 1a aceite {}, ficou {} (cega {})",
                r.finish.rounds, r.finish.first, r.finish.kept, r.finish.blind
            );
            // ⭐ χ e o bordo — a topologia é o veto DURO, e uma tabela de forma sem eles
            // não decide nada.
            let mut edges = std::collections::BTreeMap::new();
            for f in out.faces() {
                let v = f.verts();
                for k in 0..v.len() {
                    let (a, b) = (v[k], v[(k + 1) % v.len()]);
                    *edges.entry((a.min(b), a.max(b))).or_insert(0usize) += 1;
                }
            }
            let bordo = edges.values().filter(|&&c| c == 1).count();
            let nm = edges.values().filter(|&&c| c > 2).count();
            let chi = out.vert_count() as i64 - edges.len() as i64 + out.face_count() as i64;
            println!(
                "   TOPOLOGIA: X = {chi} | {bordo} bordo | {nm} nao-manifold | {} quads {} nao-quads",
                r.quads, r.non_quads
            );
            println!(
                "   forma: aspecto p50 {:.2} | enviesamento p50 {:.1} p99 {:.1} | {} faces",
                r.shape.aspect_p50,
                r.shape.skew_p50,
                r.shape.skew_p99,
                out.face_count()
            );
        }
    }
}
