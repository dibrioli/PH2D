//! ⭐ **Bloco C do F27 — o custo por quadro do losango do Inspector com N canais** (plano
//! `docs/UI_New_and_Simple/F27_bloco_B_plano.md` §4). Sonda `#[ignore]`: imprime, não afirma
//! (`docs/DevOps/MEDIR_VELOCIDADE.md` — variantes no mesmo processo, blocos intercalados com o início
//! a rodar, o MÍNIMO das rodadas com a mediana ao lado).
//!
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-app-components --profile smoke --lib \
//!  marcas_inspector::custo -- --ignored --nocapture`

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_editor_core::widget::key_mark::{paint_decorator_campo, rotulos_pintados, set_key_marks};
use ph2d_i18n::TextKey;
use std::time::Instant;

const E: u64 = 11;

/// Um documento com `n` canais COMPONENTE do catálogo, duas chaves cada, em `objetos` objectos.
fn doc_com(n: usize, objetos: u64) -> TimelineDoc {
    let chaves: Vec<FieldKey> = ph2d_component_desc::all()
        .flat_map(ph2d_ecs::canal_componente::chaves_de)
        .take(n)
        .collect();
    assert_eq!(chaves.len(), n, "o catálogo não tem {n} canais");
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    for e in 0..objetos {
        for &k in &chaves {
            for t in [0.0, 1.0] {
                d.insert_key(
                    E + e,
                    PropKind::Field(k),
                    RationalTime::from_seconds(t),
                    AnimValue::Float(1.0),
                    Interp::Linear,
                );
            }
        }
    }
    d
}

/// Os rótulos do catálogo que um Inspector cheio pintaria (um id por rótulo).
fn rotulos(r: usize) -> Vec<(NodeId, TextKey)> {
    let mut vistos = std::collections::BTreeSet::new();
    (ph2d_component_desc::all().flat_map(|d| d.fields.iter().map(|f| f.label_key)))
        .filter(|k| vistos.insert(*k))
        .take(r)
        .enumerate()
        .map(|(i, k)| (NodeId(0xC0_0000 + i as u64), TextKey::new(k)))
        .collect()
}

struct Variante {
    nome: String,
    doc: TimelineDoc,
    rotulos: Vec<(NodeId, TextKey)>,
}

/// Um quadro: (registar os rótulos ao pintar, `rotulos_pintados`, `do_rotulo` de cada um, `marcas`),
/// em µs.
fn quadro(v: &Variante, cena: &mut ph2d_vector::VectorScene) -> [f64; 4] {
    let r = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 10.0, 10.0);
    let us = |t: Instant| t.elapsed().as_secs_f64() * 1e6;
    set_key_marks(BTreeMap::new());
    *cena = ph2d_vector::VectorScene::new();
    let t = Instant::now();
    for &(id, k) in &v.rotulos {
        let _ = paint_decorator_campo(cena, ph2d_tokens::Theme::Dark, r, &[id], k.into());
    }
    let registo = us(t);
    let t = Instant::now();
    let pintados = std::hint::black_box(rotulos_pintados());
    let lista = us(t);
    let t = Instant::now();
    let n: usize = pintados.iter().map(|(_, k)| do_rotulo(k).len()).sum();
    std::hint::black_box(n);
    let resolve = us(t);
    let inst = Instante {
        t_cut: 0.0,
        solo: true,
    };
    let t = Instant::now();
    std::hint::black_box(marcas(&v.doc, &[E], inst, |_, _| Some(1.0)));
    [registo, lista, resolve, us(t)]
}

fn min_e_mediana(mut xs: Vec<f64>) -> (f64, f64) {
    xs.sort_by(f64::total_cmp);
    (xs[0], xs[xs.len() / 2])
}

#[test]
#[ignore = "sonda de relógio: imprime a tabela do plano §4"]
fn custo_do_losango_do_inspector_com_n_canais() {
    const RODADAS: usize = 9;
    const BLOCO: usize = 20;
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    let todos_canais = ph2d_component_desc::all()
        .flat_map(ph2d_ecs::canal_componente::chaves_de)
        .count();
    let todos_rotulos = rotulos(usize::MAX).len();
    let mut vs: Vec<Variante> = [
        (0, 1, 0),
        (0, 1, 40),
        (0, 1, todos_rotulos),
        (64, 1, todos_rotulos),
        (todos_canais, 1, todos_rotulos),
        (todos_canais, 16, todos_rotulos),
    ]
    .into_iter()
    .map(|(n, obj, r)| Variante {
        nome: format!("{n} canais × {obj} obj · {r} rótulos"),
        doc: doc_com(n, obj),
        rotulos: rotulos(r),
    })
    .collect();
    let mut cena = ph2d_vector::VectorScene::new();
    for v in &vs {
        quadro(v, &mut cena);
    }
    let mut amostras: Vec<[Vec<f64>; 4]> = vs.iter().map(|_| Default::default()).collect();
    for rodada in 0..RODADAS {
        for o in 0..vs.len() {
            let i = (o + rodada) % vs.len();
            let mut soma = [0.0; 4];
            for _ in 0..BLOCO {
                let q = quadro(&vs[i], &mut cena);
                for (s, x) in soma.iter_mut().zip(q) {
                    *s += x;
                }
            }
            for (a, s) in amostras[i].iter_mut().zip(soma) {
                a.push(s / BLOCO as f64);
            }
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg no fim: {}", load.trim());
    println!("| variante | registo | rotulos_pintados | do_rotulo × R | marcas() |");
    println!("|---|---|---|---|---|");
    for (v, a) in vs.iter_mut().zip(amostras) {
        let c: Vec<String> = a
            .into_iter()
            .map(|xs| {
                let (m, med) = min_e_mediana(xs);
                format!("{m:.1} µs (med {med:.1})")
            })
            .collect();
        println!("| {} | {} |", v.nome, c.join(" | "));
    }
    set_key_marks(BTreeMap::new());
}
