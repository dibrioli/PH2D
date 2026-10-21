//! ⭐⭐ **O losango de cada campo do Inspector** (plano 35 §5; plano 34 §9 metade B) — o irmão do
//! `ph2d_app_vec::marcas`: os `PropKind` de sempre (o Transform, o motor e os comprimentos das
//! juntas) e os campos das famílias OBJETO (`ph2d_ecs::canal_objeto`: skew, cores, emissão),
//! JUNTA (`ph2d_ecs::canal_junta`) e FISICA (`ph2d_ecs::canal_fisica`: a §11 Physics Body). O
//! estado e o clique passam pela porta única
//! [`ph2d_timeline::estado_do_grupo`] / [`ph2d_timeline::clique_no_grupo`].
//!
//! ⚠️ A swatch do Tint mostra o RGB (OBJETO) E o alfa, que é o `PropKind::Opacity` (`tint[3]`); o
//! slider Opacity é a opacidade FINAL (`Sprite::opacity`), um campo OBJETO à parte.
//!
//! ⚠️ A POSIÇÃO de um objecto numa trajectória é o canal `Position` (a K grava um `AddPathKey`) e a
//! de um objecto livre são `TranslationX/Y`: o clique grava a do modo em que o objecto está, nunca
//! as duas.

use std::collections::BTreeMap;

use ph2d_ecs::FieldKey;
use ph2d_ecs::canal::familia::{FISICA, JUNTA, OBJETO};
use ph2d_ecs::canal_fisica as fs;
use ph2d_ecs::canal_junta as j;
use ph2d_ecs::canal_objeto as o;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::widget::key_mark::KeyMark;
use ph2d_panel_inspector::ids as insp;
use ph2d_timeline::{Forca, Instante, PropKind, TimelineDoc, TimelineIntent};

const POSICAO: &[PropKind] = &[
    PropKind::TranslationX,
    PropKind::TranslationY,
    PropKind::Position,
];
const ESCALA: &[PropKind] = &[PropKind::ScaleX, PropKind::ScaleY];

const fn ob(c: u16) -> PropKind {
    PropKind::Field(FieldKey::new(OBJETO, c, 0, 0))
}

const fn ju(c: u16) -> PropKind {
    PropKind::Field(FieldKey::new(JUNTA, c, 0, 0))
}

const fn eixo(base: u16, a: u16) -> PropKind {
    ju(j::do_eixo(base, a))
}

const fn fi(c: u16) -> PropKind {
    PropKind::Field(FieldKey::new(FISICA, c, 0, 0))
}

/// ⭐ **A tabela campo → grupo do Inspector.** O gate `architecture_every_editable_field_is_a_channel`
/// lê-a: um campo do Inspector pintado com coluna de losango tem de estar aqui ou na tabela
/// `⛔ não-animável` / de dívida do gate.
const CAMPOS: &[(NodeId, &[PropKind])] = &[
    (insp::INSP_TRANSFORM_POS_X, POSICAO),
    (insp::INSP_TRANSFORM_POS_Y, POSICAO),
    (insp::INSP_TRANSFORM_ROT, &[PropKind::Rotation]),
    (insp::INSP_TRANSFORM_SCALE_X, ESCALA),
    (insp::INSP_TRANSFORM_SCALE_Y, ESCALA),
    (insp::INSP_TRANSFORM_SKEW_X, &[ob(o::SKEW_X)]),
    (insp::INSP_TRANSFORM_SKEW_Y, &[ob(o::SKEW_X)]),
    (insp::INSP_SPRITE_OPACITY, &[ob(o::OPACIDADE)]),
    (insp::INSP_SPRITE_OPACITY_CHIP, &[ob(o::OPACIDADE)]),
    (
        insp::INSP_SPRITE_TINT_SWATCH,
        &[ob(o::TINTA), PropKind::Opacity],
    ),
    (insp::INSP_SPRITE_SELF_TINT_SWATCH, &[ob(o::AUTO_TINTA)]),
    (insp::INSP_SPRITE_CORNER_TL, &[ob(o::CANTOS)]),
    (insp::INSP_SPRITE_CORNER_TR, &[ob(o::CANTOS)]),
    (insp::INSP_SPRITE_CORNER_BL, &[ob(o::CANTOS)]),
    (insp::INSP_SPRITE_CORNER_BR, &[ob(o::CANTOS)]),
    (insp::INSP_SPRITE_TINT_FILL, &[ob(o::SILHUETA)]),
    (insp::INSP_SPRITE_EMISSIVE, &[ob(o::EMISSAO)]),
    (insp::INSP_SPRITE_EMISSIVE_CHIP, &[ob(o::EMISSAO)]),
    (insp::INSP_JOINT_MOTOR_TARGET, &[PropKind::JointMotorTarget]),
    (insp::INSP_JOINT_MOTOR_SPEED, &[PropKind::JointMotorSpeed]),
    (insp::INSP_JOINT_REST_LENGTH, &[PropKind::JointRestLength]),
    (insp::INSP_JOINT_MAX_LENGTH, &[PropKind::JointMaxLength]),
    (insp::INSP_JOINT_MOTOR[0], &[ju(j::MOTOR_LIGADO)]),
    (insp::INSP_JOINT_MOTOR[1], &[ju(j::MOTOR_LIGADO)]),
    (insp::INSP_JOINT_MOTOR_MODE[0], &[ju(j::MOTOR_MODO)]),
    (insp::INSP_JOINT_MOTOR_MODE[1], &[ju(j::MOTOR_MODO)]),
    (insp::INSP_JOINT_MOTOR_FORCE, &[ju(j::MOTOR_FORCA)]),
    (insp::INSP_JOINT_MOTOR_AXIS[0], &[ju(j::MOTOR_EIXO)]),
    (insp::INSP_JOINT_MOTOR_AXIS[1], &[ju(j::MOTOR_EIXO)]),
    (insp::INSP_JOINT_MOTOR_AXIS[2], &[ju(j::MOTOR_EIXO)]),
    (insp::INSP_JOINT_BREAK[0], &[ju(j::QUEBRA_LIGADA)]),
    (insp::INSP_JOINT_BREAK[1], &[ju(j::QUEBRA_LIGADA)]),
    (insp::INSP_JOINT_BREAK_FORCE, &[ju(j::QUEBRA_FORCA)]),
    (insp::INSP_JOINT_BREAK_TORQUE, &[ju(j::QUEBRA_TORQUE)]),
    (insp::INSP_JOINT_LIMITS[0], &[ju(j::LIMITES_LIGADOS)]),
    (insp::INSP_JOINT_LIMITS[1], &[ju(j::LIMITES_LIGADOS)]),
    (insp::INSP_JOINT_LIMIT_MIN, &[ju(j::LIMITE_MIN)]),
    (insp::INSP_JOINT_LIMIT_MAX, &[ju(j::LIMITE_MAX)]),
    (insp::INSP_JOINT_SOFT[0], &[ju(j::SOLDA_MOLE)]),
    (insp::INSP_JOINT_SOFT[1], &[ju(j::SOLDA_MOLE)]),
    (insp::INSP_JOINT_STIFFNESS, &[ju(j::RIGIDEZ)]),
    (insp::INSP_JOINT_DAMPING, &[ju(j::AMORTECIMENTO)]),
    // Os três eixos de um Custom: o modo (Free/Limited/Locked) e o par min/max de cada um.
    (insp::INSP_JOINT_AXIS_MODE[0][0], &[eixo(j::EIXO_MODO, 0)]),
    (insp::INSP_JOINT_AXIS_MODE[0][1], &[eixo(j::EIXO_MODO, 0)]),
    (insp::INSP_JOINT_AXIS_MODE[0][2], &[eixo(j::EIXO_MODO, 0)]),
    (insp::INSP_JOINT_AXIS_MODE[1][0], &[eixo(j::EIXO_MODO, 1)]),
    (insp::INSP_JOINT_AXIS_MODE[1][1], &[eixo(j::EIXO_MODO, 1)]),
    (insp::INSP_JOINT_AXIS_MODE[1][2], &[eixo(j::EIXO_MODO, 1)]),
    (insp::INSP_JOINT_AXIS_MODE[2][0], &[eixo(j::EIXO_MODO, 2)]),
    (insp::INSP_JOINT_AXIS_MODE[2][1], &[eixo(j::EIXO_MODO, 2)]),
    (insp::INSP_JOINT_AXIS_MODE[2][2], &[eixo(j::EIXO_MODO, 2)]),
    (insp::INSP_JOINT_AXIS_MIN[0], &[eixo(j::EIXO_MIN, 0)]),
    (insp::INSP_JOINT_AXIS_MIN[1], &[eixo(j::EIXO_MIN, 1)]),
    (insp::INSP_JOINT_AXIS_MIN[2], &[eixo(j::EIXO_MIN, 2)]),
    (insp::INSP_JOINT_AXIS_MAX[0], &[eixo(j::EIXO_MAX, 0)]),
    (insp::INSP_JOINT_AXIS_MAX[1], &[eixo(j::EIXO_MAX, 1)]),
    (insp::INSP_JOINT_AXIS_MAX[2], &[eixo(j::EIXO_MAX, 2)]),
    // A §11 Physics Body (família FISICA): o corpo, o colisor, a zona de um sensor. Um par de
    // eixos é UM grupo (o deslocamento, a caixa, a velocidade, a força), como a posição.
    (insp::INSP_PHYS_KIND[0], &[fi(fs::TIPO)]),
    (insp::INSP_PHYS_KIND[1], &[fi(fs::TIPO)]),
    (insp::INSP_PHYS_KIND[2], &[fi(fs::TIPO)]),
    (insp::INSP_PHYS_SHAPE[0], &[fi(fs::FORMA)]),
    (insp::INSP_PHYS_SHAPE[1], &[fi(fs::FORMA)]),
    (insp::INSP_PHYS_SHAPE[2], &[fi(fs::FORMA)]),
    (insp::INSP_PHYS_RADIUS, &[fi(fs::RAIO)]),
    (insp::INSP_PHYS_HALF_X, &[fi(fs::MEIO_X)]),
    (insp::INSP_PHYS_HALF_Y, &[fi(fs::MEIO_X)]),
    (insp::INSP_PHYS_CAP_HALF_H, &[fi(fs::CAPSULA)]),
    (insp::INSP_PHYS_OFFSET_X, &[fi(fs::DESLOC_X)]),
    (insp::INSP_PHYS_OFFSET_Y, &[fi(fs::DESLOC_X)]),
    (insp::INSP_PHYS_DENSITY, &[fi(fs::DENSIDADE)]),
    (insp::INSP_PHYS_MASSMODE[0], &[fi(fs::MASSA_MANUAL)]),
    (insp::INSP_PHYS_MASSMODE[1], &[fi(fs::MASSA_MANUAL)]),
    (insp::INSP_PHYS_MASS, &[fi(fs::MASSA)]),
    (insp::INSP_PHYS_RESTITUTION, &[fi(fs::RESTITUICAO)]),
    (insp::INSP_PHYS_FRICTION, &[fi(fs::ATRITO)]),
    (
        insp::INSP_PHYS_REST_COMBINE[0],
        &[fi(fs::COMBINA_RESTITUICAO)],
    ),
    (
        insp::INSP_PHYS_REST_COMBINE[1],
        &[fi(fs::COMBINA_RESTITUICAO)],
    ),
    (
        insp::INSP_PHYS_REST_COMBINE[2],
        &[fi(fs::COMBINA_RESTITUICAO)],
    ),
    (
        insp::INSP_PHYS_REST_COMBINE[3],
        &[fi(fs::COMBINA_RESTITUICAO)],
    ),
    (insp::INSP_PHYS_FRIC_COMBINE[0], &[fi(fs::COMBINA_ATRITO)]),
    (insp::INSP_PHYS_FRIC_COMBINE[1], &[fi(fs::COMBINA_ATRITO)]),
    (insp::INSP_PHYS_FRIC_COMBINE[2], &[fi(fs::COMBINA_ATRITO)]),
    (insp::INSP_PHYS_FRIC_COMBINE[3], &[fi(fs::COMBINA_ATRITO)]),
    (insp::INSP_PHYS_LAYER[0], &[fi(fs::CAMADA)]),
    (insp::INSP_PHYS_LAYER[1], &[fi(fs::CAMADA)]),
    (insp::INSP_PHYS_LAYER[2], &[fi(fs::CAMADA)]),
    (insp::INSP_PHYS_LAYER[3], &[fi(fs::CAMADA)]),
    (insp::INSP_PHYS_LAYER[4], &[fi(fs::CAMADA)]),
    (insp::INSP_PHYS_LAYER[5], &[fi(fs::CAMADA)]),
    (insp::INSP_PHYS_LAYER[6], &[fi(fs::CAMADA)]),
    (insp::INSP_PHYS_LAYER[7], &[fi(fs::CAMADA)]),
    (insp::INSP_PHYS_SENSOR[0], &[fi(fs::SENSOR)]),
    (insp::INSP_PHYS_SENSOR[1], &[fi(fs::SENSOR)]),
    (insp::INSP_PHYS_ONEWAY[0], &[fi(fs::UMA_VIA)]),
    (insp::INSP_PHYS_ONEWAY[1], &[fi(fs::UMA_VIA)]),
    (insp::INSP_PHYS_WALLMAT[0], &[fi(fs::SEM_PAREDE)]),
    (insp::INSP_PHYS_WALLMAT[1], &[fi(fs::SEM_PAREDE)]),
    (insp::INSP_PHYS_WALK_GRIP, &[fi(fs::ADERENCIA)]),
    (insp::INSP_PHYS_WALK_BELT, &[fi(fs::ESTEIRA)]),
    (insp::INSP_PHYS_FORCE_X, &[fi(fs::FORCA_X)]),
    (insp::INSP_PHYS_FORCE_Y, &[fi(fs::FORCA_X)]),
    (insp::INSP_PHYS_FORCE_AXES[0], &[fi(fs::FORCA_NO_MUNDO)]),
    (insp::INSP_PHYS_FORCE_AXES[1], &[fi(fs::FORCA_NO_MUNDO)]),
    (insp::INSP_PHYS_AREA_DRAG, &[fi(fs::ZONA_ARRASTO)]),
    (insp::INSP_PHYS_AREA_DENSITY, &[fi(fs::ZONA_DENSIDADE)]),
    (
        insp::INSP_PHYS_AREA_FORM_DRAG,
        &[fi(fs::ZONA_ARRASTO_FORMA)],
    ),
    (insp::INSP_PHYS_AREA_TORQUE, &[fi(fs::ZONA_TORQUE)]),
    (insp::INSP_PHYS_AREA_FALLOFF, &[fi(fs::ZONA_QUEDA)]),
    (insp::INSP_PHYS_GRAVITY_SCALE, &[fi(fs::ESCALA_GRAVIDADE)]),
    (insp::INSP_PHYS_LINVEL_X, &[fi(fs::VEL_X)]),
    (insp::INSP_PHYS_LINVEL_Y, &[fi(fs::VEL_X)]),
    (insp::INSP_PHYS_ANGVEL, &[fi(fs::VEL_ANGULAR)]),
    (insp::INSP_PHYS_DOMINANCE, &[fi(fs::DOMINANCIA)]),
    (insp::INSP_PHYS_CCD[0], &[fi(fs::CCD)]),
    (insp::INSP_PHYS_CCD[1], &[fi(fs::CCD)]),
    (insp::INSP_PHYS_LOCKROT[0], &[fi(fs::TRAVA_ROTACAO)]),
    (insp::INSP_PHYS_LOCKROT[1], &[fi(fs::TRAVA_ROTACAO)]),
    (insp::INSP_PHYS_LOCKX[0], &[fi(fs::TRAVA_X)]),
    (insp::INSP_PHYS_LOCKX[1], &[fi(fs::TRAVA_X)]),
    (insp::INSP_PHYS_LOCKY[0], &[fi(fs::TRAVA_Y)]),
    (insp::INSP_PHYS_LOCKY[1], &[fi(fs::TRAVA_Y)]),
    (insp::INSP_PHYS_LINEAR_DAMPING, &[fi(fs::AMORT_LINEAR)]),
    (insp::INSP_PHYS_ANGULAR_DAMPING, &[fi(fs::AMORT_ANGULAR)]),
    (insp::INSP_PHYS_DAMPMODE[0], &[fi(fs::AMORT_MODO)]),
    (insp::INSP_PHYS_DAMPMODE[1], &[fi(fs::AMORT_MODO)]),
];

/// O grupo que o campo `id` do Inspector mostra (vazio = sem canal).
#[must_use]
pub fn canal_do_campo(id: NodeId) -> &'static [PropKind] {
    CAMPOS
        .iter()
        .find(|(c, _)| *c == id)
        .map_or(&[], |&(_, g)| g)
}

fn marca(f: Forca) -> KeyMark {
    match f {
        Forca::Livre => KeyMark::Livre,
        Forca::Animado => KeyMark::Animado,
        Forca::Chave => KeyMark::Chave,
        Forca::Mudado => KeyMark::Mudado,
    }
}

/// ⭐ **O mapa do quadro** para os campos do Inspector (os objectos `entidades`, o mais forte).
#[must_use]
pub fn marcas(
    doc: &TimelineDoc,
    entidades: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
) -> BTreeMap<NodeId, KeyMark> {
    let mut out = BTreeMap::new();
    if entidades.is_empty() {
        return out;
    }
    // Um grupo é perguntado uma vez: os cantos, os interruptores e os eixos repetem-no.
    let mut por_grupo: BTreeMap<&[PropKind], KeyMark> = BTreeMap::new();
    for &(id, reps) in CAMPOS {
        let m = *por_grupo.entry(reps).or_insert_with(|| {
            marca(ph2d_timeline::estado_do_grupo(doc, entidades, reps, inst, &valor).0)
        });
        out.insert(id, m);
    }
    out
}

/// ⭐ **O que o clique num losango do Inspector escreve.** Numa chave, apaga o grupo neste quadro;
/// senão grava-o pela porta da K, cada objecto no modo que a K escolheria
/// (`TimelineDoc::position_key_mode`: `Position` na trajectória, `TranslationX/Y` nos eixos
/// separados). Um grupo de família grava os campos que ela publicou (`lidos`). Fora da tabela,
/// nada.
#[must_use]
pub fn clique(
    doc: &TimelineDoc,
    campos: &[NodeId],
    entidades: &[u64],
    inst: Instante,
    lidos: impl Fn(u64) -> Vec<FieldKey>,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
    autorar: impl Fn(u64, PropKind) -> Option<TimelineIntent>,
) -> Vec<TimelineIntent> {
    let Some(reps) = campos
        .iter()
        .map(|&c| canal_do_campo(c))
        .find(|r| !r.is_empty())
    else {
        return Vec::new();
    };
    if reps.iter().any(|r| matches!(r, PropKind::Field(_))) {
        return ph2d_timeline::clique_no_grupo(doc, reps, entidades, inst, lidos, valor, autorar);
    }
    let apagar = ph2d_timeline::clique_no_grupo(
        doc,
        reps,
        entidades,
        inst,
        |_| Vec::new(),
        &valor,
        |_, _| None,
    );
    if apagar.contains(&TimelineIntent::DeleteSelection) {
        return apagar;
    }
    let mut out = vec![TimelineIntent::BeginEdit];
    for &e in entidades {
        // A porta da K: a animação que existe decide; um objecto novo vai para a trajectória.
        let na_trajetoria = doc.position_key_mode(e, true) == ph2d_timeline::PositionKeyMode::Path;
        out.extend(
            reps.iter()
                .filter(|&&p| (p == PropKind::Position) == na_trajetoria || !POSICAO.contains(&p))
                .filter_map(|&p| autorar(e, p)),
        );
    }
    out.push(TimelineIntent::EndEdit);
    out
}

#[cfg(test)]
#[path = "marcas_inspector_tests.rs"]
mod tests;
