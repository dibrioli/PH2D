//! Gates do [`super`] (`marcas_inspector.rs`) — o losango dos campos do Inspector.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

const E: u64 = 11;

fn em(t: f64) -> Instante {
    Instante {
        t_cut: t,
        solo: true,
    }
}

fn na_curva(_: u64, _: PropKind) -> Option<f32> {
    Some(2.0)
}

fn chave(d: &mut TimelineDoc, p: PropKind, t: f64) {
    d.insert_key(
        E,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(2.0),
        Interp::Linear,
    );
}

fn add(e: u64, p: PropKind) -> Option<TimelineIntent> {
    Some(TimelineIntent::AddKey {
        entity: e,
        prop: p,
        t: RationalTime::from_seconds(0.5),
        value: AnimValue::Float(1.0),
        interp: Interp::Linear,
    })
}

fn nada(_: u64) -> Vec<FieldKey> {
    Vec::new()
}

fn gravados(i: &[TimelineIntent]) -> Vec<PropKind> {
    i.iter()
        .filter_map(|x| match x {
            TimelineIntent::AddKey { prop, .. } => Some(*prop),
            _ => None,
        })
        .collect()
}

/// O X e o Y do Transform partilham o losango da posição; a rotação é a dela.
#[test]
fn o_x_e_o_y_mostram_a_posicao_inteira() {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    chave(&mut d, PropKind::TranslationY, 0.0);
    chave(&mut d, PropKind::TranslationY, 1.0);
    let m = marcas(&d, &[E], em(0.0), na_curva);
    assert_eq!(m[&insp::INSP_TRANSFORM_POS_X], KeyMark::Chave);
    assert_eq!(m[&insp::INSP_TRANSFORM_ROT], KeyMark::Livre);
    let m = marcas(&d, &[E], em(0.5), na_curva);
    assert_eq!(m[&insp::INSP_TRANSFORM_POS_Y], KeyMark::Animado);
    assert!(marcas(&d, &[], em(0.0), na_curva).is_empty());
}

/// Um objecto de eixos separados grava X e Y; um novo (o padrão do After Effects, a porta da K)
/// grava a `Position` — nunca os dois modos. (Mutação: gravar o grupo inteiro ⇒ os dois modos.)
#[test]
fn o_clique_grava_a_posicao_do_modo_do_objecto() {
    let d = TimelineDoc::new();
    let i = clique(
        &d,
        &[insp::INSP_TRANSFORM_POS_X],
        &[E],
        em(0.5),
        nada,
        na_curva,
        add,
    );
    assert_eq!(gravados(&i), vec![PropKind::Position]);
    let mut separados = TimelineDoc::new();
    separados.fps_display = 24.0;
    chave(&mut separados, PropKind::TranslationX, 0.0);
    let i = clique(
        &separados,
        &[insp::INSP_TRANSFORM_POS_Y],
        &[E],
        em(0.5),
        nada,
        na_curva,
        add,
    );
    assert_eq!(
        gravados(&i),
        vec![PropKind::TranslationX, PropKind::TranslationY]
    );
    let i = clique(
        &d,
        &[insp::INSP_TRANSFORM_SCALE_Y],
        &[E],
        em(0.5),
        nada,
        na_curva,
        add,
    );
    assert_eq!(gravados(&i), vec![PropKind::ScaleX, PropKind::ScaleY]);
    let i = clique(
        &d,
        &[insp::INSP_TRANSFORM_SKEW_X],
        &[E],
        em(0.5),
        nada,
        na_curva,
        add,
    );
    assert!(
        gravados(&i).is_empty(),
        "um grupo de família sem nada publicado não grava nada"
    );
}

/// Numa chave, o clique apaga — aqui a da rotação, num passo de undo.
#[test]
fn o_clique_numa_chave_apaga() {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    chave(&mut d, PropKind::Rotation, 0.0);
    let i = clique(
        &d,
        &[insp::INSP_TRANSFORM_ROT],
        &[E],
        em(0.0),
        nada,
        na_curva,
        add,
    );
    assert_eq!(i.first(), Some(&TimelineIntent::BeginEdit));
    assert!(i.contains(&TimelineIntent::DeleteSelection));
    assert!(gravados(&i).is_empty());
}

const fn obj(c: u16, jj: u16) -> FieldKey {
    FieldKey::new(OBJETO, c, 0, jj)
}

const fn jun(c: u16) -> FieldKey {
    FieldKey::new(JUNTA, c, 0, 0)
}

/// ⭐ Cada id da dívida do gate mostra o SEU grupo: skew, cores, silhueta, emissão, opacidade final,
/// e da junta o motor (interruptor, modo, força, eixo), a quebra, os limites, a mola e os eixos.
#[test]
fn cada_campo_novo_mostra_o_seu_grupo() {
    let grupo = |id| match canal_do_campo(id) {
        [PropKind::Field(k), ..] => k.grupo(),
        _ => None,
    };
    let casos: &[(NodeId, FieldKey)] = &[
        (insp::INSP_TRANSFORM_SKEW_X, obj(o::SKEW_Y, 0)),
        (insp::INSP_TRANSFORM_SKEW_Y, obj(o::SKEW_X, 0)),
        (insp::INSP_SPRITE_EMISSIVE_CHIP, obj(o::EMISSAO, 0)),
        (insp::INSP_SPRITE_OPACITY, obj(o::OPACIDADE, 0)),
        (insp::INSP_SPRITE_TINT_SWATCH, obj(o::TINTA + 2, 0)),
        (
            insp::INSP_SPRITE_SELF_TINT_SWATCH,
            obj(o::AUTO_TINTA + 3, 0),
        ),
        (insp::INSP_SPRITE_CORNER_BR, obj(o::CANTOS + 1, 3)),
        (insp::INSP_SPRITE_TINT_FILL, obj(o::SILHUETA, 0)),
        (insp::INSP_JOINT_MOTOR[1], jun(j::MOTOR_LIGADO)),
        (insp::INSP_JOINT_MOTOR_MODE[0], jun(j::MOTOR_MODO)),
        (insp::INSP_JOINT_MOTOR_FORCE, jun(j::MOTOR_FORCA)),
        (insp::INSP_JOINT_MOTOR_AXIS[2], jun(j::MOTOR_EIXO)),
        (insp::INSP_JOINT_BREAK[0], jun(j::QUEBRA_LIGADA)),
        (insp::INSP_JOINT_BREAK_FORCE, jun(j::QUEBRA_FORCA)),
        (insp::INSP_JOINT_BREAK_TORQUE, jun(j::QUEBRA_TORQUE)),
        (insp::INSP_JOINT_LIMITS[1], jun(j::LIMITES_LIGADOS)),
        (insp::INSP_JOINT_LIMIT_MIN, jun(j::LIMITE_MIN)),
        (insp::INSP_JOINT_LIMIT_MAX, jun(j::LIMITE_MAX)),
        (insp::INSP_JOINT_SOFT[1], jun(j::SOLDA_MOLE)),
        (insp::INSP_JOINT_STIFFNESS, jun(j::RIGIDEZ)),
        (insp::INSP_JOINT_DAMPING, jun(j::AMORTECIMENTO)),
        (
            insp::INSP_JOINT_AXIS_MODE[1][2],
            jun(j::do_eixo(j::EIXO_MODO, 1)),
        ),
        (
            insp::INSP_JOINT_AXIS_MIN[2],
            jun(j::do_eixo(j::EIXO_MIN, 2)),
        ),
        (
            insp::INSP_JOINT_AXIS_MAX[0],
            jun(j::do_eixo(j::EIXO_MAX, 0)),
        ),
    ];
    for &(id, k) in casos {
        assert_eq!(grupo(id), k.grupo(), "{id:?} → {k:?}");
        assert!(grupo(id).is_some());
    }
    assert_eq!(
        canal_do_campo(insp::INSP_SPRITE_TINT_SWATCH)[1],
        PropKind::Opacity,
        "o alfa da swatch é o `tint[3]`"
    );
    assert!(!canal_do_campo(insp::INSP_SPRITE_OPACITY).contains(&PropKind::Opacity));
}

/// ⭐ O clique num grupo de família grava o grupo INTEIRO que ela publicou (os 16 campos dos
/// cantos, não só o do canto clicado) — e nada de outro grupo.
#[test]
fn o_clique_num_grupo_de_familia_grava_o_que_ela_publicou() {
    let d = TimelineDoc::new();
    let publicados = |_| {
        let mut v: Vec<FieldKey> = (0..o::N_CANTOS)
            .flat_map(|jj| (0..4).map(move |c| obj(o::CANTOS + c, jj)))
            .collect();
        v.extend([obj(o::SKEW_X, 0), jun(j::RIGIDEZ)]);
        v
    };
    let i = clique(
        &d,
        &[insp::INSP_SPRITE_CORNER_TR],
        &[E],
        em(0.5),
        publicados,
        na_curva,
        add,
    );
    let g = gravados(&i);
    assert_eq!(g.len(), 16, "o degradê inteiro: {g:?}");
    assert!(
        g.iter()
            .all(|p| matches!(p, PropKind::Field(k) if k.campo >= o::CANTOS))
    );
    let i = clique(
        &d,
        &[insp::INSP_JOINT_STIFFNESS],
        &[E],
        em(0.5),
        publicados,
        na_curva,
        add,
    );
    assert_eq!(gravados(&i), vec![PropKind::Field(jun(j::RIGIDEZ))]);
}

/// A swatch do Tint grava o RGB publicado E o alfa (`Opacity`) — a cor inteira.
#[test]
fn a_swatch_do_tint_grava_a_cor_e_o_alfa() {
    let d = TimelineDoc::new();
    let i = clique(
        &d,
        &[insp::INSP_SPRITE_TINT_SWATCH],
        &[E],
        em(0.5),
        |_| (0..3).map(|c| obj(o::TINTA + c, 0)).collect(),
        na_curva,
        add,
    );
    let g = gravados(&i);
    assert_eq!(g.len(), 4, "{g:?}");
    assert!(g.contains(&PropKind::Opacity));
}

/// ⭐ **A tabela id → grupo da §11 Physics Body**: todo campo do vocabulário FISICA tem um controlo
/// no Inspector; as opções de uma escolha partilham o losango; o par de eixos (a caixa, o
/// deslocamento, a velocidade, a força) é UM grupo e o clique grava os dois.
#[test]
fn a_fisica_do_corpo_tem_um_losango_por_linha() {
    use ph2d_ecs::canal_fisica as fs;
    let fis = |c| PropKind::Field(FieldKey::new(FISICA, c, 0, 0));
    for c in fs::CAMPOS {
        assert!(
            CAMPOS.iter().any(|(_, g)| g.contains(&fis(c))
                || g.iter().any(|p| matches!(p, PropKind::Field(k) if k.mesmo_grupo(FieldKey::new(FISICA, c, 0, 0))))),
            "o campo {c} da física não tem controlo na §11"
        );
    }
    for id in insp::INSP_PHYS_LAYER {
        assert_eq!(canal_do_campo(id), &[fis(fs::CAMADA)]);
    }
    assert_eq!(
        canal_do_campo(insp::INSP_PHYS_HALF_Y),
        canal_do_campo(insp::INSP_PHYS_HALF_X)
    );
    assert!(
        canal_do_campo(insp::INSP_PHYS_BAKE_CH[0]).is_empty(),
        "o Bake é um comando"
    );
    let d = TimelineDoc::new();
    let i = clique(
        &d,
        &[insp::INSP_PHYS_LINVEL_Y],
        &[E],
        em(0.5),
        |_| {
            vec![
                FieldKey::new(FISICA, fs::VEL_X, 0, 0),
                FieldKey::new(FISICA, fs::VEL_Y, 0, 0),
            ]
        },
        na_curva,
        add,
    );
    assert_eq!(gravados(&i), vec![fis(fs::VEL_X), fis(fs::VEL_Y)]);
}
