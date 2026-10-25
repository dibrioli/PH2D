//! ⭐ **Os ajustes de cada tipo** — a porta única de «que campos tem este tipo, quanto valem e como se
//! escrevem», nas unidades do cartão (graus, px, índice da opção). O cartão ([`crate::inspector`]), o
//! losango ([`crate::marcas`]) e a curva ([`crate::canais`]) passam todos por aqui.

use ph2d_constraints::{ChildOf, DistanceMode, Kind, Mix, Pose2, Space};
use ph2d_constraints_ecs::Item;
use ph2d_ecs::canal_constraint::{
    DESVIO, DISTANCIA, ESPACO, ESPACOS, FORCA, LIGADA, LIMITE_MAX, LIMITE_MIN, MISTURA, MISTURAS,
    MODO, MODOS,
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
        _ => "panel.inspector.constraint.mix",
    }
}

fn opcoes(campo: u16) -> &'static [&'static str] {
    match campo {
        MODO => MODOS,
        ESPACO => ESPACOS,
        _ => MISTURAS,
    }
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
                unidade: if campo == DISTANCIA {
                    Unidade::Px
                } else {
                    Unidade::Graus
                },
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
        _ => return None,
    })
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
        _ => return None,
    })
}
