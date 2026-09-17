//! ⭐⭐⭐ **CADA UMA DAS QUATRO ALÇAS ESCREVE O CAMPO DELA** — e é isto que o censo de ida-e-volta
//! não consegue afirmar.
//!
//! O [`crate::prop_readback`] prova que ler é o inverso de escrever, canal a canal. ⛔⛔ **Mas ele
//! é CEGO a um erro de cópia SIMÉTRICO:** se a `BoneBendInY` escrevesse em `curve.inn[0]` **e** o
//! leitor dela lesse `curve.inn[0]`, a ida-e-volta fecha na mesma e o gate passa — sobre um produto
//! em que animar a alça vertical move a horizontal.
//!
//! *Dois números iguais não distinguem duas leis* — a lição que os dois sliders `Detail` da
//! escultura pagaram —, e a cura é a mesma: **quatro valores DIFERENTES**, e a tabela inteira
//! conferida campo a campo.
//!
//! # ⚠️ Porque ele mede o `Bone` e não o `AnimValue`
//!
//! A pergunta do `CLAUDE.md` §5.0 é *o valor CHEGA a um consumidor?* — e o consumidor desta wave é
//! o campo `Bone::curve`, que é o que a lei do *Bendy Bone* lê para arquear o osso. Perguntar ao
//! valor que a timeline resolveu seria perguntar à timeline sobre a timeline.

use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineDoc, TimelineState, apply_from_doc};

/// Uma chave achatada — o mesmo valor nas duas pontas, para o apply resolver um número só.
fn key(doc: &mut TimelineDoc, bits: u64, p: PropKind, v: f32) {
    doc.bind(bits, p);
    for t in [0.0_f64, 3.0] {
        doc.upsert_key(
            bits,
            p,
            ph2d_anim::RationalTime::from_seconds(t),
            ph2d_anim::AnimValue::Float(v),
            ph2d_anim::Interp::Linear,
        );
    }
}

/// ⭐⭐⭐ **As quatro alças, quatro valores, quatro campos** — e nenhum deles no campo do vizinho.
///
/// ⚠️ **Os valores são escolhidos para que NENHUM par coincida** e para que nenhum seja o neutro
/// (`Bend::STRAIGHT` é `[0, 0]`): um zero algures leria como *«este canal não escreveu»* e como
/// *«este canal escreveu zero»* ao mesmo tempo.
///
/// (Mutação: trocar o campo de qualquer braço do `BendField::set` ⇒ RED, a nomear o par.)
#[test]
fn cada_alca_escreve_o_campo_dela_e_nao_o_do_vizinho() {
    const IN_X: f32 = 1.25;
    const IN_Y: f32 = -2.5;
    const OUT_X: f32 = 3.75;
    const OUT_Y: f32 = -4.5;

    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Osso"),
            ph2d_skeleton_ecs::Bone::default(),
        ))
        .id()
        .to_bits();

    let mut st = TimelineState::new();
    key(&mut st.doc, e, PropKind::BoneBendInX, IN_X);
    key(&mut st.doc, e, PropKind::BoneBendInY, IN_Y);
    key(&mut st.doc, e, PropKind::BoneBendOutX, OUT_X);
    key(&mut st.doc, e, PropKind::BoneBendOutY, OUT_Y);

    apply_from_doc(sim.world_mut(), &mut st.doc, 1.0);

    let bone = sim
        .world()
        .get::<ph2d_skeleton_ecs::Bone>(Entity::from_bits(e))
        .expect("o osso continua la");
    let lido = [
        bone.curve.inn[0],
        bone.curve.inn[1],
        bone.curve.out[0],
        bone.curve.out[1],
    ];
    let esperado = [
        f64::from(IN_X),
        f64::from(IN_Y),
        f64::from(OUT_X),
        f64::from(OUT_Y),
    ];
    for (i, (got, want)) in lido.iter().zip(&esperado).enumerate() {
        let nome = ["inn[0]", "inn[1]", "out[0]", "out[1]"][i];
        assert!(
            (got - want).abs() < 1e-6,
            "curve.{nome} leu {got}, esperava {want} — os quatro valores sao distintos de \
             proposito, logo este numero DIZ qual canal escreveu aqui: {lido:?}"
        );
    }
}

/// ⭐⭐ **E o ponto neutro continua a ser o osso RECTO** — um osso que ninguém anima não é arqueado
/// por esta wave existir.
///
/// ⚠️ É a metade NEGATIVA, e sem ela o gate acima passaria num produto que escrevesse as quatro
/// alças **em todo osso do mundo** a cada quadro.
#[test]
fn um_osso_sem_track_fica_recto() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Osso"),
            ph2d_skeleton_ecs::Bone::default(),
        ))
        .id()
        .to_bits();

    let mut st = TimelineState::new();
    // Uma track de OUTRO canal no MESMO osso: o apply corre, e nada toca na curvatura.
    key(&mut st.doc, e, PropKind::Rotation, 0.5);
    apply_from_doc(sim.world_mut(), &mut st.doc, 1.0);

    let bone = sim
        .world()
        .get::<ph2d_skeleton_ecs::Bone>(Entity::from_bits(e))
        .expect("o osso continua la");
    assert_eq!(
        (bone.curve.inn, bone.curve.out),
        ([0.0, 0.0], [0.0, 0.0]),
        "um osso sem track de alça tem de sair RECTO — `Bend::STRAIGHT` é o nascimento, e todo rig \
         já autorado atravessa esta linha ao bit"
    );
}
