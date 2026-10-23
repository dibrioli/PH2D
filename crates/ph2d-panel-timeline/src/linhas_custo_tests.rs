//! ⭐ **Bloco C do F27 — o custo de [`super::layout`] com N canais** (plano
//! `docs/UI_New_and_Simple/F27_bloco_B_plano.md` §4): a família COMPONENTE multiplica as linhas do
//! dope-sheet. Sonda `#[ignore]`: imprime, não afirma (`docs/DevOps/MEDIR_VELOCIDADE.md`).
//!
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-panel-timeline --profile smoke --lib \
//!  linhas::custo -- --ignored --nocapture`

use super::*;
use ph2d_ecs::FieldKey;
use ph2d_timeline::{AnimTarget, Extrap, Interp, KeyId, KeyView, TrackView};
use std::time::Instant;

/// `n` faixas COMPONENTE do catálogo (pares e cores formam grupos), duas chaves cada, por objecto.
fn cena(n: usize, objetos: u64) -> TimelineViewSnapshot {
    let chaves: Vec<FieldKey> = ph2d_component_desc::all()
        .flat_map(ph2d_ecs::canal_componente::chaves_de)
        .take(n)
        .collect();
    assert_eq!(chaves.len(), n, "o catálogo não tem {n} canais");
    let mut tracks = Vec::new();
    for e in 0..objetos {
        for &k in &chaves {
            let target = tracks.len() as u64 + 1;
            tracks.push(TrackView {
                target: AnimTarget::new(target),
                prop: PropKind::Field(k),
                entity: 100 + e,
                missing: false,
                keys: (0..2)
                    .map(|j| KeyView {
                        id: KeyId::new(target * 10 + j),
                        t_seconds: j as f64,
                        value: 0.0,
                        interp: Interp::Linear,
                        selected: false,
                        roving: false,
                    })
                    .collect(),
                buffer_ghost: None,
                pre: Extrap::Hold,
                post: Extrap::Hold,
                expr: None,
            });
        }
    }
    TimelineViewSnapshot {
        fps: 60.0,
        tracks,
        ..TimelineViewSnapshot::default()
    }
}

#[test]
#[ignore = "sonda de relógio: imprime a tabela do plano §4"]
fn custo_do_layout_com_n_canais() {
    const RODADAS: usize = 9;
    const BLOCO: usize = 20;
    let todos = ph2d_component_desc::all()
        .flat_map(ph2d_ecs::canal_componente::chaves_de)
        .count();
    let vs: Vec<(String, TimelineViewSnapshot, Vec<GrupoChave>)> = [
        (8, 1, false),
        (64, 1, false),
        (todos, 1, false),
        (todos, 1, true),
        (todos, 16, true),
    ]
    .into_iter()
    .map(|(n, obj, abre)| {
        let s = cena(n, obj);
        // Aberto = cada grupo mostra os seus membros (o pior caso da pintura).
        let abertos: Vec<GrupoChave> = if abre {
            s.tracks.iter().filter_map(GrupoChave::de).collect()
        } else {
            Vec::new()
        };
        let nome = format!(
            "{} faixas ({n} × {obj} obj){}",
            s.tracks.len(),
            if abre { " · grupos abertos" } else { "" }
        );
        (nome, s, abertos)
    })
    .collect();
    let corre = |(_, s, a): &(String, TimelineViewSnapshot, Vec<GrupoChave>)| {
        std::hint::black_box(layout(s, Tab::Keys, &[], a, 132.0, 0.0, 0.0))
            .bandas
            .len()
    };
    for v in &vs {
        corre(v);
    }
    let mut amostras: Vec<Vec<f64>> = vec![Vec::new(); vs.len()];
    for rodada in 0..RODADAS {
        for o in 0..vs.len() {
            let i = (o + rodada) % vs.len();
            let t = Instant::now();
            for _ in 0..BLOCO {
                corre(&vs[i]);
            }
            amostras[i].push(t.elapsed().as_secs_f64() * 1e6 / BLOCO as f64);
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg no fim: {}", load.trim());
    println!("| variante | linhas | layout() |");
    println!("|---|---|---|");
    for (v, mut a) in vs.iter().zip(amostras) {
        a.sort_by(f64::total_cmp);
        println!(
            "| {} | {} | {:.1} µs (med {:.1}) |",
            v.0,
            corre(v),
            a[0],
            a[a.len() / 2]
        );
    }
}
