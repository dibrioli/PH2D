//! Os gates do [`super::declare`] — **a forma declara o colisor, e só quando lho pedem** (doc 109).
//!
//! ⚠️ Como os do `fill`, estes montam o external À MÃO sob a chave que o nó calcula e COZEM o nó:
//! chamar o `declare` directamente provaria a aritmética e deixaria por testar *«o `eval` chama-o,
//! e com os params do nó»*, que é a metade que um fio esquecido parte.

use crate::{MANIFEST, NodeOp, NodeTypeId, SourceShape, param, shape_key};
use ph2d_nodegraph::attr::{COLLIDER_COLUMN, Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Graph, NodeId};

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&SourceShape as &dyn NodeOp)
    }
}

/// O que o shell publica: a linha da forma **com os dois raios** (`Around = 1,5`, `Inside = 0,7`).
fn published() -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![[2.0, 2.0]]))
        .with("geometry_id", Column::Scalar(vec![7.0]))
        .with(param::COLLIDER_AROUND_COL, Column::Scalar(vec![1.5]))
        .with(param::COLLIDER_INSIDE_COL, Column::Scalar(vec![0.7]))
}

fn cooked(setup: impl FnOnce(&mut Graph, NodeId)) -> Stream {
    let mut g = Graph::new();
    let sh = g.add_node("source.shape");
    setup(&mut g, sh);
    let key = shape_key(|n| {
        g.node_param_overrides(sh)
            .and_then(|o| o.get(n).copied())
            .or_else(|| MANIFEST.param_default(n))
            .unwrap_or(0.0)
    });
    let mut cook = Cook::new();
    cook.set_external(&key, published());
    cook.cook(&g, &Ops, sh, 0.0).expect("cozinha")[0]
        .as_stream()
        .clone()
}

fn collider(s: &Stream) -> Option<Vec<f32>> {
    match s.get(COLLIDER_COLUMN) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

/// ⭐ **Desligado, nenhuma declaração — e as colunas de raio do shell NÃO atravessam.**
///
/// ⚠️ As duas metades num gate, porque cada uma sozinha tem cura errada: *«não escrever o
/// `collider`»* passa com as colunas de raio a vazar para toda a cadeia, e *«retirá-las»* passa
/// com o `collider` escrito sempre.
#[test]
fn without_collide_the_shape_declares_nothing_and_the_radii_do_not_leak() {
    let s = cooked(|_, _| {});
    assert!(collider(&s).is_none(), "desligado nao declara colisor");
    for c in [param::COLLIDER_AROUND_COL, param::COLLIDER_INSIDE_COL] {
        assert!(s.get(c).is_none(), "a coluna de raio `{c}` vazou do nó");
    }
    // O resto do que o shell publicou atravessa.
    assert!(s.get("geometry_id").is_some() && s.get("size").is_some());
}

/// ⭐ **Ligado, o raio À VOLTA vezes a escala** — e continua sem as colunas de raio.
#[test]
fn collide_declares_the_radius_around_the_outline_times_the_scale() {
    let s = cooked(|g, n| {
        g.set_param(n, param::COLLIDE, 1.0);
        g.set_param(n, param::COLLIDER_SCALE, 2.0);
    });
    assert_eq!(collider(&s), Some(vec![3.0]), "Around 1,5 × escala 2");
    assert!(s.get(param::COLLIDER_AROUND_COL).is_none());
}

/// ⭐ **`Inside` escolhe o OUTRO raio** — a mutação que ignora o ajuste reprova aqui.
#[test]
fn the_inside_fit_declares_the_radius_inside_the_outline() {
    let s = cooked(|g, n| {
        g.set_param(n, param::COLLIDE, 1.0);
        g.set_param(n, param::COLLIDER_FIT, 1.0);
    });
    assert_eq!(collider(&s), Some(vec![0.7]), "Inside 0,7 × escala 1");
}

/// ⚠️ **Os params de colisão NÃO entram na chave de conteúdo** — senão cada clique na caixa
/// re-internaria a geometria, e duas formas iguais com colisores diferentes deixariam de partilhar
/// o `VecPath`.
#[test]
fn the_collision_params_stay_out_of_the_content_key() {
    let base = |n: &str| MANIFEST.param_default(n).unwrap_or(0.0);
    let aceso = |n: &str| match n {
        param::COLLIDE => 1.0,
        param::COLLIDER_FIT => 1.0,
        param::COLLIDER_SCALE => 1.7,
        _ => base(n),
    };
    assert_eq!(shape_key(base), shape_key(aceso));
}
