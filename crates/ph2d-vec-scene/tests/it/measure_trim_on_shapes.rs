//! **SONDA — o `trim_path` que já existe alcança uma FORMA?**
//!
//! A folha 14 marca `P1` no *"sem TRIM / dash"* do `source.shape` e escreve a cura ao lado:
//! *"`ph2d_vec_scene::trim_path(path, start, end)` **existe** (`marker.rs:395`)"*, com o
//! diagnóstico *"nenhum nó do grafo alcança `trim_path`"*. Isso lê como *ligue a função e está
//! feito*.
//!
//! ⚠️ **Mas a primeira linha daquela função é `if path.closed { return clone }`**, e toda forma
//! do catálogo é um contorno FECHADO. Se for assim, ligar o nó à função entregaria um par de
//! sliders que não move um pixel — o *"botão que não faz nada"* que esta casa considera pior que
//! o botão que falta.
//!
//! Esta sonda varre o catálogo, corta `25%` de cada ponta, e imprime quantas formas **mudaram**.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-vec-scene --test measure_trim_on_shapes -- --ignored --nocapture`.

use ph2d_vec_scene::{ALL_SHAPES, ShapeKind, cook, trim_path};

/// A caixa em que cada forma é cozida.
const BOX_A: [f64; 2] = [-1.0, -1.0];
const BOX_B: [f64; 2] = [1.0, 1.0];
/// A fracção cortada de cada ponta.
const CUT: f64 = 0.25;

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn how_many_catalogue_shapes_the_trim_can_actually_cut() {
    let kinds: &[ShapeKind] = ALL_SHAPES;
    let (mut closed, mut cut, mut refused) = (0usize, 0usize, 0usize);
    let mut movers: Vec<String> = Vec::new();
    eprintln!(
        "\n[trim] cortando {:.0}% de cada ponta de cada forma do catalogo\n",
        CUT * 100.0
    );
    for &k in kinds {
        let path = cook(k, BOX_A, BOX_B, k.defaults().as_slice());
        if path.closed {
            closed += 1;
        }
        match trim_path(&path, CUT, CUT) {
            None => refused += 1,
            Some(t) => {
                let same = t.closed == path.closed
                    && t.verts.len() == path.verts.len()
                    && t.verts
                        .iter()
                        .zip(&path.verts)
                        .all(|(a, b)| a.anchor == b.anchor);
                if !same {
                    cut += 1;
                    movers.push(format!("{k:?}"));
                }
            }
        }
    }
    eprintln!(
        "  {} formas no catalogo · {closed} FECHADAS · {cut} de facto cortadas · {refused} recusadas",
        kinds.len()
    );
    eprintln!("  as que mudaram: {movers:?}");
    eprintln!(
        "\n  LEITURA: se `cortadas` for 0, ligar o `trim_path` ao no' daria dois sliders
  INERTES — a cura da celula nao e' a fiacao, e' abrir o contorno primeiro."
    );
}
