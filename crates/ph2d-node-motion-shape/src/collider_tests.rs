//! Os gates do [`super::declare`] — **a forma declara o colisor, e só quando lho pedem** (doc 109).
//!
//! ⚠️ Como os do `fill`, estes montam o external À MÃO sob a chave que o nó calcula e COZEM o nó:
//! chamar o `declare` directamente provaria a aritmética e deixaria por testar *«o `eval` chama-o,
//! e com os params do nó»*, que é a metade que um fio esquecido parte.

use crate::{MANIFEST, NodeOp, NodeTypeId, SourceShape, param, shape_key};
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, INV_INERTIA_COLUMN,
    Stream,
};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Graph, NodeId};

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&SourceShape as &dyn NodeOp)
    }
}

/// O que o shell publica: a linha da forma **com a caixa envolvente** (meia `[1,5; 0,5]`, centro
/// `centro`).
fn published(centro: [f32; 2]) -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("size", Column::Vec2(vec![[2.0, 2.0]]))
        .with("geometry_id", Column::Scalar(vec![7.0]))
        .with(param::COLLIDER_FIT_CENTER_COL, Column::Vec2(vec![centro]))
        .with(param::COLLIDER_FIT_HALF_COL, Column::Vec2(vec![[1.5, 0.5]]))
}

fn cooked_from(centro: [f32; 2], setup: impl FnOnce(&mut Graph, NodeId)) -> Stream {
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
    cook.set_external(&key, published(centro));
    cook.cook(&g, &Ops, sh, 0.0).expect("cozinha")[0]
        .as_stream()
        .clone()
}

fn cooked(setup: impl FnOnce(&mut Graph, NodeId)) -> Stream {
    cooked_from([0.0, 0.25], setup)
}

fn escalar(s: &Stream, name: &str) -> Option<Vec<f32>> {
    match s.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

fn vec2(s: &Stream, name: &str) -> Option<Vec<[f32; 2]>> {
    match s.get(name) {
        Some(Column::Vec2(v)) => Some(v.clone()),
        _ => None,
    }
}

/// ⭐ **Desligado, nenhuma declaração — e a caixa envolvente do shell NÃO atravessa.**
///
/// ⚠️ As duas metades num gate, porque cada uma sozinha tem cura errada: *«não escrever o
/// colisor»* passa com as colunas da caixa a vazar para toda a cadeia, e *«retirá-las»* passa
/// com o colisor escrito sempre.
#[test]
fn without_collide_the_shape_declares_nothing_and_the_fit_does_not_leak() {
    let s = cooked(|_, _| {});
    for c in [COLLIDER_COLUMN, COLLIDER_BOX_COLUMN, COLLIDER_OFFSET_COLUMN] {
        assert!(s.get(c).is_none(), "desligado nao declara `{c}`");
    }
    for c in [param::COLLIDER_FIT_CENTER_COL, param::COLLIDER_FIT_HALF_COL] {
        assert!(s.get(c).is_none(), "a coluna da caixa `{c}` vazou do nó");
    }
    // O resto do que o shell publicou atravessa.
    assert!(s.get("geometry_id").is_some() && s.get("size").is_some());
}

/// ⭐⭐ **`Box` (o default) declara a CAIXA do contorno vezes a largura e a altura** — e não um raio.
#[test]
fn the_box_is_the_outline_bounds_times_width_and_height() {
    let s = cooked(|g, n| {
        g.set_param(n, param::COLLIDE, 1.0);
        g.set_param(n, param::COLLIDER_WIDTH, 2.0);
        g.set_param(n, param::COLLIDER_HEIGHT, 0.5);
    });
    assert_eq!(
        vec2(&s, COLLIDER_BOX_COLUMN),
        Some(vec![[3.0, 0.25]]),
        "meia [1,5; 0,5] × [2; 0,5]"
    );
    assert!(
        escalar(&s, COLLIDER_COLUMN).is_none(),
        "a caixa nao declara raio"
    );
    assert_eq!(vec2(&s, COLLIDER_OFFSET_COLUMN), Some(vec![[0.0, 0.25]]));
    assert!(s.get(param::COLLIDER_FIT_HALF_COL).is_none());
}

/// ⭐⭐ **`Circle` declara o círculo que toca os lados MAIORES da caixa**, vezes o raio — e nenhuma
/// caixa.
#[test]
fn the_circle_touches_the_longer_sides_of_the_bounds_times_the_radius() {
    let s = cooked(|g, n| {
        g.set_param(n, param::COLLIDE, 1.0);
        g.set_param(n, param::COLLIDER_SHAPE, 1.0);
        g.set_param(n, param::COLLIDER_RADIUS, 2.0);
    });
    assert_eq!(
        escalar(&s, COLLIDER_COLUMN),
        Some(vec![3.0]),
        "max(1,5; 0,5) × 2"
    );
    assert!(
        vec2(&s, COLLIDER_BOX_COLUMN).is_none(),
        "o circulo nao declara caixa"
    );
    assert_eq!(vec2(&s, COLLIDER_OFFSET_COLUMN), Some(vec![[0.0, 0.25]]));
}

/// ⭐ **A arte centrada NÃO escreve o centro** — e uma largura negativa lê como ZERO, nunca como uma
/// caixa virada do avesso.
///
/// ⚠️ O não-finito não se testa daqui: o `Graph::set_param` recusa-o à entrada, e a guarda do nó
/// existe para o valor CONDUZIDO por fio, que não passa por ele.
#[test]
fn a_centred_outline_writes_no_offset_and_a_negative_width_reads_as_zero() {
    let s = cooked_from([0.0, 0.0], |g, n| {
        g.set_param(n, param::COLLIDE, 1.0);
        g.set_param(n, param::COLLIDER_WIDTH, -3.0);
    });
    assert!(
        s.get(COLLIDER_OFFSET_COLUMN).is_none(),
        "centro na origem nao se escreve"
    );
    assert_eq!(vec2(&s, COLLIDER_BOX_COLUMN), Some(vec![[0.0, 0.5]]));
}

/// ⭐⭐ **`Lock Rotation` escreve a inércia a ZERO; destravada não escreve nada** (doc 109 §6).
///
/// ⚠️ As duas metades num gate: *«trava»* passa com a coluna escrita sempre, e *«destravada não
/// escreve»* passa com ela nunca escrita — e a ausência é o que diz ao solver *«deriva da forma»*.
#[test]
fn locking_the_rotation_writes_a_zero_inertia_column() {
    let travada = cooked(|g, n| {
        g.set_param(n, param::COLLIDE, 1.0);
        g.set_param(n, param::LOCK_ROTATION, 1.0);
    });
    assert_eq!(escalar(&travada, INV_INERTIA_COLUMN), Some(vec![0.0]));
    let solta = cooked(|g, n| g.set_param(n, param::COLLIDE, 1.0));
    assert!(
        solta.get(INV_INERTIA_COLUMN).is_none(),
        "destravada, a coluna nao existe"
    );
}

/// ⚠️ **Os params de colisão NÃO entram na chave de conteúdo** — senão cada clique na caixa
/// re-internaria a geometria, e duas formas iguais com colisores diferentes deixariam de partilhar
/// o `VecPath`.
#[test]
fn the_collision_params_stay_out_of_the_content_key() {
    let base = |n: &str| MANIFEST.param_default(n).unwrap_or(0.0);
    let aceso = |n: &str| match n {
        param::COLLIDE | param::COLLIDER_SHAPE | param::LOCK_ROTATION => 1.0,
        param::SHOW_COLLIDER => 0.0,
        param::COLLIDER_WIDTH | param::COLLIDER_HEIGHT | param::COLLIDER_RADIUS => 1.7,
        _ => base(n),
    };
    assert_eq!(shape_key(base), shape_key(aceso));
}
