//! Os gates da `=138` e da `PH2D_MOTION_OBJ_SMOKE=18` (doc 132 §1 e §6): cada uma monta, nasce com
//! o filtro em CADA forma, e a SIMULAÇÃO entrega as peças a cair.

use super::*;
use crate::motion_state::MotionState;

/// O nome do objecto que o P6 carimba (o do `motion_object_smoke`).
const OBJETO: &str = "Object";

/// Coze `cena` pela CPU até ao tique `ate`; devolve `(posições das formas, posições das sprites)`.
fn coze(
    cena: impl Fn(&mut MotionDoc, &NodeRegistry) -> Option<Cena>,
    ate: u64,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let c = cena(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = vec![c.saida];
    m.pump.define_a_lei(true);
    m.pump.mark_dirty();
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    let sinks = m.sinks.clone();
    for t in 0..=ate {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        crate::motion_lsystem_testkit::publish_object(&mut m, OBJETO, 0);
        m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    let formas = m
        .pump
        .vector_instances
        .iter()
        .map(|v| v.world_pos)
        .collect();
    let sprites = m.pump.instances.iter().map(|v| v.world_pos).collect();
    (formas, sprites)
}

/// ⭐ As duas cenas MONTAM, com o filtro do tipo certo em `Each Shape` (o `Apply To` de omissão);
/// e o roteador monta a `=138` pelo número.
#[test]
fn as_cenas_do_p1_e_do_p6_montam_com_o_filtro_em_cada_forma() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    for (nome, tipo) in [
        ("=138", ph2d_ecs::FxOp::GLOW),
        ("OBJ_SMOKE=18", ph2d_ecs::FxOp::DROP_SHADOW),
    ] {
        let mut doc = MotionDoc::new();
        let c = if nome == "=138" {
            monta_discos(&mut doc, &reg)
        } else {
            monta_sprites(&mut doc, &reg, OBJETO)
        }
        .unwrap_or_else(|| panic!("a {nome} monta"));
        let g = &doc.graph;
        let le = |n: NodeId, k: &str| g.node_param_overrides(n).and_then(|m| m.get(k).copied());
        // Um param sem override é o default do manifesto: `0` = `Each Shape`.
        let p = |k: &str| le(c.filtro, k).unwrap_or(0.0);
        assert_eq!(
            p(ph2d_node_fx_filter::param::KIND),
            f32::from(tipo),
            "{nome}: o efeito"
        );
        assert_eq!(
            p(ph2d_node_fx_filter::param::SCOPE),
            0.0,
            "{nome}: nasce em `Each Shape`"
        );
        assert_eq!(le(c.campo, "radius"), Some(CAMPO), "{nome}: o campo");
    }
    let mut doc = MotionDoc::new();
    assert!(
        !crate::motion_state::demo_router::build_level(Some("138"), &mut doc, &reg).is_empty(),
        "o roteador monta a `=138`"
    );
}

/// ⭐ A `=138` entrega milhares de DISCOS a cair (a simulação corre).
#[test]
fn a_cena_138_chove_discos() {
    let (p, _) = coze(monta_discos, 180);
    assert!(p.len() > 1_000, "a chuva tem discos: {}", p.len());
    let caidos = p.iter().filter(|q| q[1] < 3.0).count();
    assert!(caidos > 200, "{caidos} de {} discos caíram", p.len());
}

/// ⭐ O P6 entrega cópias da SPRITE a cair (o controlo é a `=138`: formas, zero sprites).
#[test]
fn a_cena_do_p6_chove_sprites() {
    let (formas, sprites) = coze(|d, r| monta_sprites(d, r, OBJETO), 180);
    assert!(
        sprites.len() > 500,
        "a chuva tem sprites: {}",
        sprites.len()
    );
    assert!(formas.is_empty(), "o P6 não tem formas: {}", formas.len());
    let caidas = sprites.iter().filter(|q| q[1] < 3.0).count();
    assert!(caidas > 100, "{caidas} de {} sprites caíram", sprites.len());
    let (_, nenhuma) = coze(monta_discos, 30);
    assert!(nenhuma.is_empty(), "controlo: a `=138` não desenha sprites");
}
