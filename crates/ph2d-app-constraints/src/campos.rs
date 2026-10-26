//! ⭐ **Os ajustes de cada tipo** — a porta única de «que campos tem este tipo, quanto valem e como se
//! escrevem», nas unidades do cartão (graus, metros, índice da opção). O cartão ([`crate::inspector`]), o
//! losango ([`crate::marcas`]) e a curva ([`crate::canais`]) passam todos por aqui.

use ph2d_constraints::{
    Ajuste, BendSide, Canais, Canal, ChildOf, DistanceMode, EixoDoChao, Kind, Mix, PathBy, Pose2,
    Space, Volume,
};
use ph2d_constraints_ecs::Item;
use ph2d_ecs::canal_constraint::{
    AJUSTE, AJUSTES, CANAIS, CLIP, CORRENTE, DE, DE_MAX, DE_MIN, DESVIO, DESVIO_X, DESVIO_Y,
    DISTANCIA, DOBRA, DOBRAS, DURACAO, EIXO, EIXOS, ESPACO, ESPACOS, ESTICA, EXTRAPOLA, FORCA,
    INSTANTE_ATE, INSTANTE_DE, LIGADA, LIMITE_MAX, LIMITE_MIN, MISTURA, MISTURAS, MODO, MODOS,
    MOLA, MOLAS, NAO_SIM, PARA, PARA_MAX, PARA_MIN, PONTO, POR, PORES, POSICAO, REPOUSO, RESSALTO,
    SEGUE, SUAVIDADE, USA_ROTACAO, VOLUME, VOLUMES,
};
use ph2d_editor_core::constraint_edits::{
    AjusteEscolha, AjusteNumero, MAX_ESCOLHAS, MAX_NUMEROS, Unidade,
};

/// Os números do tipo, pela ordem do cartão.
#[must_use]
pub fn numeros(k: &Kind) -> &'static [u16] {
    match k {
        Kind::Limit(l) if l.rot.is_some() => &[LIMITE_MIN, LIMITE_MAX],
        Kind::Distance(_) => &[DISTANCIA],
        Kind::LookAt(_) => &[DESVIO],
        // Com UM osso a IK aponta: o desvio vale e a suavidade não.
        Kind::Ik(ik) if ik.chain == 1 => &[CORRENTE, DESVIO],
        Kind::Ik(_) => &[CORRENTE, SUAVIDADE],
        Kind::StretchTo(_) => &[REPOUSO],
        Kind::FollowPath(_) => &[POSICAO, DESVIO],
        Kind::Pivot(_) => &[DESVIO_X, DESVIO_Y],
        Kind::Floor(_) => &[DESVIO],
        Kind::Transformation(_) => &[DE_MIN, DE_MAX, PARA_MIN, PARA_MAX],
        Kind::SplineIk(_) => &[CORRENTE],
        Kind::PontoDeForma(_) => &[PONTO],
        Kind::Action(_) => &[DE_MIN, DE_MAX, CLIP, INSTANTE_DE, INSTANTE_ATE],
        Kind::Spring(_) => &[DURACAO, RESSALTO],
        _ => &[],
    }
}

/// As escolhas do tipo, pela ordem do cartão.
#[must_use]
pub fn escolhas(k: &Kind) -> &'static [u16] {
    match k {
        Kind::Distance(_) => &[MODO],
        Kind::CopyTransform(_) => &[MISTURA, ESPACO],
        Kind::Limit(_) => &[ESPACO],
        Kind::Ik(_) => &[DOBRA, ESTICA],
        Kind::StretchTo(_) => &[VOLUME],
        Kind::FollowPath(_) => &[POR, SEGUE],
        Kind::Floor(_) => &[EIXO, USA_ROTACAO],
        Kind::Transformation(_) => &[DE, PARA, MISTURA, EXTRAPOLA, ESPACO],
        Kind::SplineIk(_) => &[AJUSTE],
        Kind::Action(_) => &[DE],
        Kind::Spring(_) => &[MOLA],
        _ => &[],
    }
}

/// O tipo tem um desvio que o «Reset Offset» zera.
#[must_use]
pub fn zera_desvio(k: &Kind) -> bool {
    matches!(k, Kind::ChildOf(_))
}

fn rotulo(campo: u16) -> &'static str {
    match campo {
        LIMITE_MIN => "panel.inspector.constraint.min",
        LIMITE_MAX => "panel.inspector.constraint.max",
        DISTANCIA => "panel.inspector.constraint.distance",
        DESVIO => "panel.inspector.constraint.offset",
        MODO => "panel.inspector.constraint.mode",
        ESPACO => "panel.inspector.constraint.space",
        CORRENTE => "panel.inspector.constraint.chain",
        SUAVIDADE => "panel.inspector.constraint.softness",
        DOBRA => "panel.inspector.constraint.bend",
        ESTICA => "panel.inspector.constraint.stretch",
        REPOUSO => "panel.inspector.constraint.rest_length",
        VOLUME => "panel.inspector.constraint.volume",
        POSICAO => "panel.inspector.constraint.position",
        POR => "panel.inspector.constraint.by",
        SEGUE => "panel.inspector.constraint.follow_curve",
        DESVIO_X => "panel.inspector.constraint.offset_x",
        DESVIO_Y => "panel.inspector.constraint.offset_y",
        EIXO => "panel.inspector.constraint.axis",
        USA_ROTACAO => "panel.inspector.constraint.use_rotation",
        DE => "panel.inspector.constraint.from",
        PARA => "panel.inspector.constraint.to",
        DE_MIN => "panel.inspector.constraint.from_min",
        DE_MAX => "panel.inspector.constraint.from_max",
        PARA_MIN => "panel.inspector.constraint.to_min",
        PARA_MAX => "panel.inspector.constraint.to_max",
        EXTRAPOLA => "panel.inspector.constraint.extrapolate",
        AJUSTE => "panel.inspector.constraint.fit",
        PONTO => "panel.inspector.constraint.point",
        CLIP => "panel.inspector.constraint.clip",
        INSTANTE_DE => "panel.inspector.constraint.clip_from",
        INSTANTE_ATE => "panel.inspector.constraint.clip_to",
        DURACAO => "panel.inspector.constraint.duration",
        RESSALTO => "panel.inspector.constraint.bounce",
        MOLA => "panel.inspector.constraint.spring_channels",
        _ => "panel.inspector.constraint.mix",
    }
}

fn opcoes(campo: u16) -> &'static [&'static str] {
    match campo {
        MODO => MODOS,
        ESPACO => ESPACOS,
        DOBRA => DOBRAS,
        ESTICA | SEGUE | USA_ROTACAO | EXTRAPOLA => NAO_SIM,
        EIXO => EIXOS,
        DE | PARA => CANAIS,
        AJUSTE => AJUSTES,
        MOLA => MOLAS,
        VOLUME => VOLUMES,
        POR => PORES,
        _ => MISTURAS,
    }
}

/// A unidade do número `campo` do tipo `k` (a posição do Follow Path segue o «By»).
fn unidade(k: &Kind, campo: u16) -> Unidade {
    match (k, campo) {
        (Kind::Transformation(t), DE_MIN | DE_MAX) => unidade_do_canal(t.de),
        (Kind::Transformation(t), PARA_MIN | PARA_MAX) => unidade_do_canal(t.para),
        (Kind::Action(a), DE_MIN | DE_MAX) => unidade_do_canal(a.canal),
        (Kind::Floor(_), DESVIO) | (_, DESVIO_X | DESVIO_Y) => Unidade::Metros,
        (_, PONTO | CLIP) => Unidade::Contagem,
        (_, INSTANTE_DE | INSTANTE_ATE | DURACAO) => Unidade::Segundos,
        (_, RESSALTO) => Unidade::Porcento,
        (_, DISTANCIA | REPOUSO) => Unidade::Metros,
        (_, SUAVIDADE) => Unidade::Porcento,
        (_, CORRENTE) => Unidade::Contagem,
        (Kind::FollowPath(f), POSICAO) if f.by == PathBy::Fraction => Unidade::Porcento,
        (_, POSICAO) => Unidade::Metros,
        _ => Unidade::Graus,
    }
}

fn unidade_do_canal(c: Canal) -> Unidade {
    match c {
        Canal::X | Canal::Y => Unidade::Metros,
        Canal::Rot => Unidade::Graus,
        Canal::Sx | Canal::Sy => Unidade::Porcento,
    }
}

/// Um valor de um canal na unidade do cartão (a rotação em graus).
fn no_cartao(c: Canal, v: f64) -> f32 {
    if c == Canal::Rot { graus(v) } else { f32_de(v) }
}

/// O inverso de [`no_cartao`].
fn do_cartao(c: Canal, v: f32) -> f64 {
    if c == Canal::Rot {
        f64::from(v).to_radians()
    } else {
        f64::from(v)
    }
}

#[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
fn f32_de(v: f64) -> f32 {
    v as f32
}

/// O que a linha do cartão mostra debaixo da força.
#[must_use]
pub fn ajustes(k: &Kind) -> (Vec<AjusteNumero>, Vec<AjusteEscolha>) {
    debug_assert!(numeros(k).len() <= MAX_NUMEROS && escolhas(k).len() <= MAX_ESCOLHAS);
    let n = numeros(k)
        .iter()
        .filter_map(|&campo| {
            Some(AjusteNumero {
                campo,
                rotulo: rotulo(campo),
                valor: ler(k, campo)?,
                unidade: unidade(k, campo),
            })
        })
        .collect();
    let e = escolhas(k)
        .iter()
        .filter_map(|&campo| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "um índice pequeno"
            )]
            Some(AjusteEscolha {
                campo,
                rotulo: rotulo(campo),
                opcoes: opcoes(campo),
                valor: ler(k, campo)? as u8,
            })
        })
        .collect();
    (n, e)
}

#[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
fn graus(r: f64) -> f32 {
    r.to_degrees() as f32
}

fn espaco(s: Space) -> f32 {
    match s {
        Space::World => 0.0,
        Space::Local => 1.0,
    }
}

/// O valor de um campo do TIPO (`None` = o tipo não o tem).
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
pub fn ler(k: &Kind, campo: u16) -> Option<f32> {
    Some(match (k, campo) {
        (Kind::Limit(l), LIMITE_MIN) => graus(l.rot?[0]),
        (Kind::Limit(l), LIMITE_MAX) => graus(l.rot?[1]),
        (Kind::Limit(l), ESPACO) => espaco(l.space),
        (Kind::Distance(d), DISTANCIA) => d.distance as f32,
        (Kind::Distance(d), MODO) => match d.mode {
            DistanceMode::Inside => 0.0,
            DistanceMode::Outside => 1.0,
            DistanceMode::Surface => 2.0,
        },
        (Kind::LookAt(l), DESVIO) => graus(l.offset),
        (Kind::CopyTransform(c), ESPACO) => espaco(c.space),
        (Kind::CopyTransform(c), MISTURA) => match c.mix {
            Mix::Replace => 0.0,
            Mix::Add => 1.0,
        },
        (Kind::Ik(ik), CORRENTE) => ik.chain as f32,
        (Kind::Ik(ik), SUAVIDADE) => ik.softness as f32,
        (Kind::Ik(ik), DOBRA) => indice(&BendSide::ALL, &ik.bend),
        (Kind::Ik(ik), ESTICA) => sim(ik.stretch),
        (Kind::Ik(ik), DESVIO) => graus(ik.offset),
        (Kind::StretchTo(st), REPOUSO) => st.rest as f32,
        (Kind::StretchTo(st), VOLUME) => indice(&VOLUMES_DO_TIPO, &st.volume),
        (Kind::FollowPath(f), POSICAO) => f.value as f32,
        (Kind::FollowPath(f), POR) => indice(&PORES_DO_TIPO, &f.by),
        (Kind::FollowPath(f), SEGUE) => sim(f.follow),
        (Kind::FollowPath(f), DESVIO) => graus(f.offset),
        (Kind::Pivot(p), DESVIO_X) => p.offset[0] as f32,
        (Kind::Pivot(p), DESVIO_Y) => p.offset[1] as f32,
        (Kind::Floor(f), DESVIO) => f.desvio as f32,
        (Kind::Floor(f), EIXO) => indice(&EIXOS_DO_TIPO, &f.eixo),
        (Kind::Floor(f), USA_ROTACAO) => sim(f.usa_rotacao),
        (Kind::Transformation(t), DE) => indice(&CANAIS_DO_TIPO, &t.de),
        (Kind::Transformation(t), PARA) => indice(&CANAIS_DO_TIPO, &t.para),
        (Kind::Transformation(t), DE_MIN) => no_cartao(t.de, t.de_faixa[0]),
        (Kind::Transformation(t), DE_MAX) => no_cartao(t.de, t.de_faixa[1]),
        (Kind::Transformation(t), PARA_MIN) => no_cartao(t.para, t.para_faixa[0]),
        (Kind::Transformation(t), PARA_MAX) => no_cartao(t.para, t.para_faixa[1]),
        (Kind::Transformation(t), MISTURA) => indice(&[Mix::Replace, Mix::Add], &t.mix),
        (Kind::Transformation(t), EXTRAPOLA) => sim(t.extrapola),
        (Kind::Transformation(t), ESPACO) => espaco(t.space),
        (Kind::SplineIk(sp), CORRENTE) => sp.chain as f32,
        (Kind::SplineIk(sp), AJUSTE) => indice(&AJUSTES_DO_TIPO, &sp.ajuste),
        (Kind::PontoDeForma(p), PONTO) => p.vert.0 as f32,
        (Kind::Action(a), DE) => indice(&CANAIS_DO_TIPO, &a.canal),
        (Kind::Action(a), DE_MIN) => no_cartao(a.canal, a.faixa[0]),
        (Kind::Action(a), DE_MAX) => no_cartao(a.canal, a.faixa[1]),
        (Kind::Action(a), CLIP) => a.clip as f32,
        (Kind::Action(a), INSTANTE_DE) => a.de_t as f32,
        (Kind::Action(a), INSTANTE_ATE) => a.ate_t as f32,
        (Kind::Spring(m), DURACAO) => m.duracao as f32,
        (Kind::Spring(m), RESSALTO) => m.ressalto as f32,
        (Kind::Spring(m), MOLA) => indice(&MOLAS_DO_TIPO, &m.canais),
        _ => return None,
    })
}

/// As opções do tipo pela ordem das palavras ([`VOLUMES`], [`PORES`]).
const VOLUMES_DO_TIPO: [Volume; 3] = [Volume::Xz, Volume::Area, Volume::Nenhum];
const PORES_DO_TIPO: [PathBy; 2] = [PathBy::Fraction, PathBy::Distance];
const EIXOS_DO_TIPO: [EixoDoChao; 4] = [
    EixoDoChao::MaisY,
    EixoDoChao::MenosY,
    EixoDoChao::MaisX,
    EixoDoChao::MenosX,
];
const CANAIS_DO_TIPO: [Canal; 5] = [Canal::X, Canal::Y, Canal::Rot, Canal::Sx, Canal::Sy];
const AJUSTES_DO_TIPO: [Ajuste; 2] = [Ajuste::Nenhum, Ajuste::Esticar];
const MOLAS_DO_TIPO: [Canais; 3] = [
    Canais::LOCATION,
    Canais::ROTATION,
    Canais {
        x: true,
        y: true,
        rot: true,
        sx: false,
        sy: false,
    },
];

#[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
fn indice<T: PartialEq>(todas: &[T], v: &T) -> f32 {
    todas.iter().position(|x| x == v).unwrap_or(0) as f32
}

fn sim(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

/// Escreve um campo do TIPO; `true` se mudou. O Min nunca passa do Max (e vice-versa: uma faixa
/// invertida trava no centro, `clamp_to_limit`), a distância nunca é negativa e o índice vai à opção
/// válida mais próxima.
pub fn escrever(k: &mut Kind, campo: u16, v: f32) -> bool {
    if !v.is_finite() {
        return false;
    }
    let antes = *k;
    let rad = f64::from(v).to_radians();
    let opcao = |n: usize| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "já aparado a ≥ 0"
        )]
        let i = v.round().max(0.0) as usize;
        i.min(n - 1)
    };
    let space = |i| if i == 0 { Space::World } else { Space::Local };
    match (&mut *k, campo) {
        (Kind::Limit(l), LIMITE_MIN) => {
            let Some(r) = &mut l.rot else { return false };
            r[0] = rad.min(r[1]);
        }
        (Kind::Limit(l), LIMITE_MAX) => {
            let Some(r) = &mut l.rot else { return false };
            r[1] = rad.max(r[0]);
        }
        (Kind::Limit(l), ESPACO) => l.space = space(opcao(2)),
        (Kind::Distance(d), DISTANCIA) => d.distance = f64::from(v.max(0.0)),
        (Kind::Distance(d), MODO) => {
            d.mode = [
                DistanceMode::Inside,
                DistanceMode::Outside,
                DistanceMode::Surface,
            ][opcao(3)];
        }
        (Kind::LookAt(l), DESVIO) => l.offset = rad,
        (Kind::CopyTransform(c), ESPACO) => c.space = space(opcao(2)),
        (Kind::CopyTransform(c), MISTURA) => c.mix = [Mix::Replace, Mix::Add][opcao(2)],
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "≥ 0, arredondado"
        )]
        (Kind::Ik(ik), CORRENTE) => ik.chain = v.round().max(0.0) as u32,
        (Kind::Ik(ik), SUAVIDADE) => ik.softness = f64::from(v.clamp(0.0, 1.0)),
        (Kind::Ik(ik), DOBRA) => ik.bend = BendSide::ALL[opcao(BendSide::ALL.len())],
        (Kind::Ik(ik), ESTICA) => ik.stretch = opcao(2) == 1,
        (Kind::Ik(ik), DESVIO) => ik.offset = rad,
        (Kind::StretchTo(st), REPOUSO) => st.rest = f64::from(v.max(0.0)),
        (Kind::StretchTo(st), VOLUME) => st.volume = VOLUMES_DO_TIPO[opcao(3)],
        (Kind::FollowPath(f), POSICAO) => f.value = f64::from(v),
        (Kind::FollowPath(f), POR) => f.by = PORES_DO_TIPO[opcao(2)],
        (Kind::FollowPath(f), SEGUE) => f.follow = opcao(2) == 1,
        (Kind::FollowPath(f), DESVIO) => f.offset = rad,
        (Kind::Pivot(p), DESVIO_X) => p.offset[0] = f64::from(v),
        (Kind::Pivot(p), DESVIO_Y) => p.offset[1] = f64::from(v),
        (Kind::Floor(f), DESVIO) => f.desvio = f64::from(v),
        (Kind::Floor(f), EIXO) => f.eixo = EIXOS_DO_TIPO[opcao(4)],
        (Kind::Floor(f), USA_ROTACAO) => f.usa_rotacao = opcao(2) == 1,
        (Kind::Transformation(t), DE) => t.de = CANAIS_DO_TIPO[opcao(5)],
        (Kind::Transformation(t), PARA) => t.para = CANAIS_DO_TIPO[opcao(5)],
        (Kind::Transformation(t), DE_MIN) => t.de_faixa[0] = do_cartao(t.de, v),
        (Kind::Transformation(t), DE_MAX) => t.de_faixa[1] = do_cartao(t.de, v),
        (Kind::Transformation(t), PARA_MIN) => t.para_faixa[0] = do_cartao(t.para, v),
        (Kind::Transformation(t), PARA_MAX) => t.para_faixa[1] = do_cartao(t.para, v),
        (Kind::Transformation(t), MISTURA) => t.mix = [Mix::Replace, Mix::Add][opcao(2)],
        (Kind::Transformation(t), EXTRAPOLA) => t.extrapola = opcao(2) == 1,
        (Kind::Transformation(t), ESPACO) => t.space = space(opcao(2)),
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "≥ 0, arredondado"
        )]
        (Kind::SplineIk(sp), CORRENTE) => sp.chain = v.round().max(0.0) as u32,
        (Kind::SplineIk(sp), AJUSTE) => sp.ajuste = AJUSTES_DO_TIPO[opcao(2)],
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "≥ 0, arredondado"
        )]
        (Kind::PontoDeForma(p), PONTO) => {
            p.vert = ph2d_vec_scene::VertId(v.round().max(0.0) as u32)
        }
        (Kind::Action(a), DE) => a.canal = CANAIS_DO_TIPO[opcao(5)],
        (Kind::Action(a), DE_MIN) => a.faixa[0] = do_cartao(a.canal, v),
        (Kind::Action(a), DE_MAX) => a.faixa[1] = do_cartao(a.canal, v),
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "≥ 0, arredondado"
        )]
        (Kind::Action(a), CLIP) => a.clip = v.round().max(0.0) as u32,
        (Kind::Action(a), INSTANTE_DE) => a.de_t = f64::from(v),
        (Kind::Action(a), INSTANTE_ATE) => a.ate_t = f64::from(v),
        (Kind::Spring(m), DURACAO) => m.duracao = f64::from(v.max(0.0)),
        (Kind::Spring(m), RESSALTO) => m.ressalto = f64::from(v.clamp(-1.0, 1.0)),
        (Kind::Spring(m), MOLA) => m.canais = MOLAS_DO_TIPO[opcao(3)],
        _ => return false,
    }
    *k != antes
}

/// Zera o desvio do Child Of: o dono cola-se ao alvo. `true` se mudou.
pub fn zerar_desvio(k: &mut Kind) -> bool {
    let Kind::ChildOf(c) = k else {
        return false;
    };
    let antes = *c;
    *c = ChildOf {
        offset: Pose2::IDENTITY,
        ..*c
    };
    *c != antes
}

/// O valor de qualquer campo do ITEM (a força e o liga/desliga incluídos).
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "a força é 0..=1")]
pub fn ler_item(it: &Item, campo: u16) -> Option<f32> {
    match campo {
        FORCA => Some(it.influence as f32),
        LIGADA => Some(if it.enabled { 1.0 } else { 0.0 }),
        _ => ler(&it.kind, campo),
    }
}

/// Escreve qualquer campo do ITEM; `true` se mudou.
pub fn escrever_item(it: &mut Item, campo: u16, v: f32) -> bool {
    let antes = *it;
    match campo {
        FORCA if v.is_finite() => it.influence = f64::from(v.clamp(0.0, 1.0)),
        FORCA => return false,
        LIGADA => it.enabled = v >= 0.5,
        _ => return escrever(&mut it.kind, campo, v),
    }
    *it != antes
}

/// A chave serde do campo DENTRO do item (`"kind/Limit/rot[0]"`) — o endereço do ledger do
/// `PreviewDrive`. `None` = o tipo não tem o campo.
#[must_use]
pub fn chave_no_item(k: &Kind, campo: u16) -> Option<&'static str> {
    Some(match (k, campo) {
        (_, FORCA) => "influence",
        (_, LIGADA) => "enabled",
        (Kind::Limit(_), LIMITE_MIN) => "kind/Limit/rot[0]",
        (Kind::Limit(_), LIMITE_MAX) => "kind/Limit/rot[1]",
        (Kind::Limit(_), ESPACO) => "kind/Limit/space",
        (Kind::Distance(_), DISTANCIA) => "kind/Distance/distance",
        (Kind::Distance(_), MODO) => "kind/Distance/mode",
        (Kind::LookAt(_), DESVIO) => "kind/LookAt/offset",
        (Kind::CopyTransform(_), ESPACO) => "kind/CopyTransform/space",
        (Kind::CopyTransform(_), MISTURA) => "kind/CopyTransform/mix",
        (Kind::Ik(_), CORRENTE) => "kind/Ik/chain",
        (Kind::Ik(_), SUAVIDADE) => "kind/Ik/softness",
        (Kind::Ik(_), DOBRA) => "kind/Ik/bend",
        (Kind::Ik(_), ESTICA) => "kind/Ik/stretch",
        (Kind::Ik(_), DESVIO) => "kind/Ik/offset",
        (Kind::StretchTo(_), REPOUSO) => "kind/StretchTo/rest",
        (Kind::StretchTo(_), VOLUME) => "kind/StretchTo/volume",
        (Kind::FollowPath(_), POSICAO) => "kind/FollowPath/value",
        (Kind::FollowPath(_), POR) => "kind/FollowPath/by",
        (Kind::FollowPath(_), SEGUE) => "kind/FollowPath/follow",
        (Kind::FollowPath(_), DESVIO) => "kind/FollowPath/offset",
        (Kind::Pivot(_), DESVIO_X) => "kind/Pivot/offset[0]",
        (Kind::Pivot(_), DESVIO_Y) => "kind/Pivot/offset[1]",
        (Kind::Floor(_), DESVIO) => "kind/Floor/desvio",
        (Kind::Floor(_), EIXO) => "kind/Floor/eixo",
        (Kind::Floor(_), USA_ROTACAO) => "kind/Floor/usa_rotacao",
        (Kind::Transformation(_), DE) => "kind/Transformation/de",
        (Kind::Transformation(_), PARA) => "kind/Transformation/para",
        (Kind::Transformation(_), DE_MIN) => "kind/Transformation/de_faixa[0]",
        (Kind::Transformation(_), DE_MAX) => "kind/Transformation/de_faixa[1]",
        (Kind::Transformation(_), PARA_MIN) => "kind/Transformation/para_faixa[0]",
        (Kind::Transformation(_), PARA_MAX) => "kind/Transformation/para_faixa[1]",
        (Kind::Transformation(_), MISTURA) => "kind/Transformation/mix",
        (Kind::Transformation(_), EXTRAPOLA) => "kind/Transformation/extrapola",
        (Kind::Transformation(_), ESPACO) => "kind/Transformation/space",
        (Kind::SplineIk(_), CORRENTE) => "kind/SplineIk/chain",
        (Kind::SplineIk(_), AJUSTE) => "kind/SplineIk/ajuste",
        (Kind::PontoDeForma(_), PONTO) => "kind/PontoDeForma/vert",
        (Kind::Action(_), DE) => "kind/Action/canal",
        (Kind::Action(_), DE_MIN) => "kind/Action/faixa[0]",
        (Kind::Action(_), DE_MAX) => "kind/Action/faixa[1]",
        (Kind::Action(_), CLIP) => "kind/Action/clip",
        (Kind::Action(_), INSTANTE_DE) => "kind/Action/de_t",
        (Kind::Action(_), INSTANTE_ATE) => "kind/Action/ate_t",
        (Kind::Spring(_), DURACAO) => "kind/Spring/duracao",
        (Kind::Spring(_), RESSALTO) => "kind/Spring/ressalto",
        (Kind::Spring(_), MOLA) => "kind/Spring/canais",
        _ => return None,
    })
}
