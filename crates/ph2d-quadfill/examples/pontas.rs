//! ⭐⭐⭐ **A TABELA POR PONTA de uma ou mais malhas** — o instrumento do report de 04/09
//! (*«muitas pontas boas no mesmo mesh, e apenas uma ruim»*).
//!
//! ```bash
//! cargo run -p ph2d-quadfill --release --example pontas -- \
//!     --entrada <escultura.obj> [--unit <h>] <saida.obj> [f1.obj …]
//! ```
//!
//! Uma linha por espinho da **entrada** ([`ph2d_quadfill::tip_rows`]), para cada malha
//! comparada. ⛔ **A `unit` é a MESMA em todas** (a primeira manda, ou `--unit`): a lei do
//! ápice decide *o que é um espinho* à escala da unidade, então duas unidades dariam dois
//! censos e as tabelas não se leriam lado a lado.

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut entrada: Option<String> = None;
    let mut unit: Option<f32> = None;
    // ⭐⭐⭐ **`--recentrar` passa a entrada pela PORTA do importador**
    // ([`ph2d_mesh::Mesh::recenter`]) — ⛔ sem isto a tabela mede a escultura no sítio em que
    // ela foi exportada e as malhas comparadas vivem noutro espaço, e a régua lê amputação
    // onde não há. *Esta linha pagou o mesmo erro quatro vezes em dois dias* (plano §104).
    let recentrar = args.iter().any(|a| a == "--recentrar");
    args.retain(|a| a != "--recentrar");
    // ⭐ **`--rematar` aplica [`ph2d_quadfill::snap_tips`] a cada malha antes de a medir** — o
    // instrumento que responde *«o remate pega nesta peça?»* sem pagar uma corrida do botão.
    let rematar = args.iter().any(|a| a == "--rematar");
    args.retain(|a| a != "--rematar");
    // ⭐ **`--bandas` varre a grade do bico para fora** (`0–3`, `3–6`, `6–12`, `12–24 h`): a
    // foto do dono mostra o espinho INTEIRO, e uma leitura só no bico não diz onde a grade
    // muda de carácter.
    let bandas = args.iter().any(|a| a == "--bandas");
    args.retain(|a| a != "--bandas");
    while let Some(i) = args.iter().position(|a| a == "--entrada" || a == "--unit") {
        let chave = args[i].clone();
        let valor = args.get(i + 1).cloned().unwrap_or_default();
        args.drain(i..=(i + 1).min(args.len() - 1));
        if chave == "--entrada" {
            entrada = Some(valor);
        } else {
            unit = valor.parse().ok();
        }
    }
    let Some(mut entrada) = entrada.and_then(|p| ler(&p)) else {
        eprintln!(
            "uso: cargo run -p ph2d-quadfill --release --example pontas -- \
             --entrada <escultura.obj> [--unit <h>] <malha.obj> [outra.obj …]"
        );
        return;
    };
    if args.is_empty() {
        eprintln!("sem malha para comparar");
        return;
    }
    if recentrar {
        let c = entrada.recenter();
        println!("entrada RECENTRADA pela porta do importador (centro da caixa era {c:?})");
    }
    let mut malhas: Vec<(String, ph2d_mesh::Mesh)> = args
        .iter()
        .filter_map(|p| ler(p).map(|m| (nome(p), m)))
        .collect();
    let Some((_, primeira)) = malhas.first() else {
        return;
    };
    let h = unit.unwrap_or_else(|| ph2d_quadfill::median_edge(primeira));
    if rematar {
        for (nome, m) in &mut malhas {
            let n = ph2d_quadfill::snap_tips(m, &entrada, h);
            println!("{nome}: REMATE — {n} bico(s) encostaram no apice");
        }
    }
    println!(
        "unidade h = {h:.5} ({}) | entrada {} verts",
        if unit.is_some() {
            "dada por --unit"
        } else {
            "aresta mediana da 1.a malha"
        },
        entrada.vert_count(),
    );
    for (nome, m) in &malhas {
        let linhas = ph2d_quadfill::tip_rows(&entrada, m, h);
        println!(
            "\n{nome}: {} faces | {} espinho(s) | barra: gap {:.2} · grade {:.2}",
            m.face_count(),
            linhas.len(),
            ph2d_quadfill::TIP_GAP_MAX,
            ph2d_quadfill::TIP_DENSITY_MAX,
        );
        println!(
            "   apice   raio   cone    gap  grade  aspecto(n)   CORPO grade/asp(n)  polo   dev p50   p90"
        );
        for r in &linhas {
            let dev = r.dev.unwrap_or([f32::NAN; 3]);
            println!(
                "   {:>6} {:>6.3} {:>6} {:>6.2}{} {:>6.2}{} {:>11} {:>20} {:>8} {:>7.2} {:>5.2}{}",
                r.apex,
                r.radius,
                r.cone
                    .map_or_else(|| "  s/a".to_string(), |c| format!("{c:6.2}")),
                r.gap,
                if r.gap > ph2d_quadfill::TIP_GAP_MAX {
                    "*"
                } else {
                    " "
                },
                r.grade.unwrap_or(f32::NAN),
                if r.grade.unwrap_or(0.0) > ph2d_quadfill::TIP_DENSITY_MAX {
                    "*"
                } else {
                    " "
                },
                format!("{:>5.2}({:<3})", r.faces.0, r.faces.1),
                format!("{:>6.2} /{:>5.2}({:<4})", r.shaft.0, r.shaft.1, r.shaft.2),
                format!("{:>3},{:<3}", r.pole.0, r.pole.1),
                dev[0],
                dev[1],
                if r.blind { "  ⛔ CEGA (piso)" } else { "" },
            );
        }
        if bandas {
            let pos = entrada.positions();
            for r in &linhas {
                let p = pos[r.apex];
                let mut linha = format!("   ponta {:>6}:", r.apex);
                for (de, ate) in [(0.0, 3.0), (3.0, 6.0), (6.0, 12.0), (12.0, 24.0)] {
                    let (grade, asp, n) = ph2d_quadfill::tip_band(m, p, h, de, ate);
                    linha.push_str(&format!("  [{de:.0}-{ate:.0}h] {grade:.2}/{asp:.2}({n})"));
                }
                // ⭐⭐⭐ **QUANTOS QUADS DÃO A VOLTA** — o número que o olho lê como *«a
                // densidade deste espinho»*: um espinho fino com quads pequenos ainda pode ter
                // só cinco faces em volta, e é isso que se vê na foto. Conta-se o anel de
                // vértices a `d ± ½` célula de caminho do bico.
                let onbr = ph2d_quadfill::adjacency(m);
                let opos = m.positions();
                if let Some(seed) = (0..opos.len()).min_by(|&i, &j| {
                    let (a, b) = (opos[i], opos[j]);
                    let da = (a[0] - p[0]).powi(2) + (a[1] - p[1]).powi(2) + (a[2] - p[2]).powi(2);
                    let db = (b[0] - p[0]).powi(2) + (b[1] - p[1]).powi(2) + (b[2] - p[2]).powi(2);
                    da.total_cmp(&db)
                }) {
                    let bola = ph2d_quadfill::path_ball(opos, &onbr, seed, 13.0 * h);
                    linha.push_str("  voltas:");
                    for d in [3.0f32, 6.0, 12.0] {
                        let n = bola
                            .values()
                            .filter(|x| ((*x / h) - d).abs() <= 0.5)
                            .count();
                        linha.push_str(&format!(" {d:.0}h={n}"));
                    }
                }
                println!("{linha}");
            }
        }
        let cortadas = linhas
            .iter()
            .filter(|r| r.gap > ph2d_quadfill::TIP_GAP_MAX)
            .count();
        let grossas = linhas
            .iter()
            .filter(|r| r.grade.unwrap_or(0.0) > ph2d_quadfill::TIP_DENSITY_MAX)
            .count();
        println!(
            "   RESUMO: {cortadas} amputada(s) · {grossas} com a grade grossa · de {}",
            linhas.len()
        );
    }
}

fn nome(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn ler(path: &str) -> Option<ph2d_mesh::Mesh> {
    let texto = std::fs::read_to_string(path)
        .map_err(|e| eprintln!("{path}: {e}"))
        .ok()?;
    ph2d_mesh::import_obj(&texto)
        .map_err(|e| eprintln!("{path}: nao e' um OBJ deste leitor: {e:?}"))
        .ok()?
        .into_iter()
        .next()
        .map(|p| p.mesh)
}
