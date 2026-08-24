//! Os gates do [`param::FILL`] e do [`param::ROTATION`] — **a forma tem cor e aponta para um
//! lado** (doc 89 folha 14, as duas últimas células).
//!
//! ⚠️ **A fonte deste nó é um EXTERNAL que o shell publica**, então estes gates montam um
//! external à mão sob a chave que o nó calcula. É o único jeito de medir a costura sem o shell:
//! um gate que chamasse a lei directamente provaria a aritmética e deixaria por testar
//! *"o nó escreve na coluna certa do que lhe chegou"*, que é a metade que importa.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Graph, NodeId};

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&SourceShape as &dyn NodeOp)
    }
}

/// O que o shell publica para uma forma: uma linha com `P`, `size` e um `tint` BRANCO.
fn published() -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]]))
        .with("tint", Column::Vec4(vec![[0.25, 0.5, 0.75, 1.0]]))
}

/// Coze o nó com o external pousado sob a chave que ele próprio calcula.
fn cooked(setup: impl FnOnce(&mut Graph, NodeId)) -> Stream {
    let mut g = Graph::new();
    let sh = g.add_node("source.shape");
    setup(&mut g, sh);
    let mut cook = Cook::new();
    // ⚠️ A chave sai da MESMA função que o `eval` usa — se ela mudasse, o external ficaria
    // órfão e o gate mediria um stream vazio em vez de acusar.
    let key = shape_key(|n| {
        g.node_param_overrides(sh)
            .and_then(|o| o.get(n).copied())
            .or_else(|| MANIFEST.param_default(n))
            .unwrap_or(0.0)
    });
    cook.set_external(&key, published());
    cook.cook(&g, &Ops, sh, 0.0).expect("cozinha")[0]
        .as_stream()
        .clone()
}

fn tint(s: &Stream) -> Option<Vec<[f32; 4]>> {
    match s.get("tint") {
        Some(Column::Vec4(v)) => Some(v.clone()),
        _ => None,
    }
}

/// ⭐ **Desligado, o `tint` do shell atravessa INTOCADO** — a lei estrutural, não um valor
/// reescrito igual.
#[test]
fn without_its_own_fill_the_published_tint_passes_through() {
    let s = cooked(|_, _| {});
    let t = tint(&s).expect("o shell publicou um tint");
    assert_eq!(
        (t[0][0].to_bits(), t[0][1].to_bits(), t[0][2].to_bits()),
        (0.25_f32.to_bits(), 0.5_f32.to_bits(), 0.75_f32.to_bits()),
        "o tint publicado tinha de sair como entrou: {:?}",
        t[0]
    );
    // E não há coluna `rot` inventada.
    assert!(
        s.get("rot").is_none(),
        "um `rotation` de zero nao pode cunhar uma coluna"
    );
}

/// ⭐ **Ligado, a forma tem a cor que o artista escolheu.**
#[test]
fn its_own_fill_paints_the_shape() {
    let s = cooked(|g, n| {
        g.set_param(n, param::FILL, 1.0);
        g.set_param(n, param::FILL_R, 1.0);
        g.set_param(n, param::FILL_G, 0.0);
        g.set_param(n, param::FILL_B, 0.0);
        g.set_param(n, param::FILL_A, 0.5);
    });
    let t = tint(&s).expect("tint");
    assert_eq!(t.len(), 1, "uma linha, como o shell publicou");
    assert!(
        (t[0][0] - 1.0).abs() < 1e-6
            && t[0][1].abs() < 1e-6
            && t[0][2].abs() < 1e-6
            && (t[0][3] - 0.5).abs() < 1e-6,
        "a cor autorada tinha de vencer a publicada: {:?}",
        t[0]
    );
}

/// ⭐ **A rotação própria ATRIBUI** — este nó é uma FONTE, e não há nada a montante com que
/// compor (a lei do `motion.distribute_curve`, vista do mesmo lado).
#[test]
fn its_own_rotation_sets_the_column() {
    let s = cooked(|g, n| g.set_param(n, param::ROTATION, 30.0));
    match s.get("rot") {
        Some(Column::Scalar(v)) => {
            assert_eq!(v.len(), 1, "uma linha");
            assert!((v[0] - 30.0).abs() < 1e-6, "o angulo autorado: {v:?}");
        }
        _ => panic!("a coluna `rot` tinha de nascer"),
    }
}

/// ⚠️ **As duas são ORTOGONAIS** — ligar uma não pode mexer na outra.
#[test]
fn the_fill_and_the_rotation_do_not_touch_each_other() {
    let only_fill = cooked(|g, n| g.set_param(n, param::FILL, 1.0));
    assert!(
        only_fill.get("rot").is_none(),
        "pintar nao pode cunhar uma rotacao"
    );
    let only_rot = cooked(|g, n| g.set_param(n, param::ROTATION, 45.0));
    let t = tint(&only_rot).expect("tint");
    assert!(
        (t[0][0] - 0.25).abs() < 1e-6,
        "rodar nao pode repintar: {:?}",
        t[0]
    );
}

/// ⚠️ **Um external AUSENTE continua a dar um stream vazio, sem pânico** — o caminho do cook
/// adiantado (antes de o shell publicar), e as duas escritas novas têm de o respeitar.
#[test]
fn an_unpublished_shape_still_emits_nothing() {
    let mut g = Graph::new();
    let sh = g.add_node("source.shape");
    g.set_param(sh, param::FILL, 1.0);
    g.set_param(sh, param::ROTATION, 90.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, sh, 0.0).expect("cozinha");
    assert_eq!(
        out[0].as_stream().count(),
        0,
        "nada publicado, nada emitido"
    );
}

/// **Os seis params novos são alcançáveis pelo painel**, e o swatch está gateado ao modo.
#[test]
fn every_new_knob_is_reachable() {
    for p in [
        param::FILL,
        param::FILL_R,
        param::FILL_G,
        param::FILL_B,
        param::FILL_A,
        param::ROTATION,
    ] {
        assert!(
            MANIFEST.params.iter().any(|s| s.name == p),
            "`{p}` fora do manifesto"
        );
    }
    // Os três canais dobrados NÃO têm linha própria — quem tem é a âncora.
    for p in [param::FILL, param::FILL_R, param::ROTATION] {
        assert!(
            hints::PARAM_HINTS.iter().any(|h| h.param == p),
            "`{p}` sem hint de painel"
        );
    }
    assert!(
        hints::PARAM_GATES_ABOVE
            .iter()
            .any(|g| g.param == param::FILL_R && g.when == param::FILL),
        "o swatch tem de estar gateado ao modo que o le^"
    );
}
