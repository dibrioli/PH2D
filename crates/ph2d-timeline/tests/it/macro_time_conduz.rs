//! ⭐ **A TIMELINE CONDUZ O RELÓGIO DE UMA MACRO** (ADR-0186, item 5 da W4) — `PropKind::MacroTime`
//! escreve `ph2d_ecs::MacroTempo` numa entidade com `Macro`, e só nela.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

fn tempo_de(sim: &SimWorld, e: u64) -> Option<f64> {
    sim.world()
        .get::<ph2d_ecs::MacroTempo>(Entity::from_bits(e))
        .map(|m| m.segundos)
}

fn com_chaves_lineares(e: u64, chaves: &[(f64, f32)]) -> TimelineState {
    let mut st = TimelineState::new();
    st.doc.bind(e, PropKind::MacroTime);
    for &(t, v) in chaves {
        st.doc.upsert_key(
            e,
            PropKind::MacroTime,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    st
}

/// O id `18` é valor de FIO: uma faixa gravada tem de voltar a ser este canal ao carregar (o
/// `from_target` tem `_ => None`, onde uma variante nova some em silêncio — ver o `IkBendSide`).
#[test]
fn o_canal_resolve_se_do_id_que_o_documento_grava() {
    assert_eq!(
        PropKind::from_target(ph2d_anim::AnimTarget::new(18)),
        Some(PropKind::MacroTime),
        "o id 18 nao volta a ser o relogio da macro: a faixa perde-se ao carregar"
    );
    assert_eq!(PropKind::MacroTime.target().get(), 18);
}

/// Chaves lineares `(0, 0) → (2, 2)` — a forma que o Gravar ▸ Parar cria — põem a macro no próprio
/// instante do clipe; uma rampa mais lenta re-cronometra-a.
#[test]
fn chaves_lineares_poem_a_macro_no_instante_do_clipe() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Gravado"),
            ph2d_ecs::Macro::default(),
        ))
        .id()
        .to_bits();

    let mut st = com_chaves_lineares(e, &[(0.0, 0.0), (2.0, 2.0)]);
    for t in [0.0, 0.5, 1.25, 2.0] {
        apply_from_doc(sim.world_mut(), &mut st.doc, t);
        let got = tempo_de(&sim, e).expect("a faixa nao escreveu o MacroTempo");
        assert!((got - t).abs() < 1e-5, "t={t}: a macro esta em {got}");
    }

    // Metade da velocidade: o instante do clipe 1,0 é o segundo 0,5 da macro.
    let mut lenta = com_chaves_lineares(e, &[(0.0, 0.0), (2.0, 1.0)]);
    apply_from_doc(sim.world_mut(), &mut lenta.doc, 1.0);
    let got = tempo_de(&sim, e).unwrap();
    assert!((got - 0.5).abs() < 1e-5, "a rampa lenta deu {got}, nao 0,5");
}

/// ⛔ Sem `Macro` o canal é inerte: não inventa um tempo que nenhum tocador lê.
#[test]
fn sem_macro_o_canal_e_inerte() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Sem gravacao")))
        .id()
        .to_bits();
    let mut st = com_chaves_lineares(e, &[(0.0, 0.0), (2.0, 2.0)]);
    apply_from_doc(sim.world_mut(), &mut st.doc, 1.0);
    assert_eq!(
        tempo_de(&sim, e),
        None,
        "o canal escreveu um MacroTempo num objecto sem Macro"
    );
}
