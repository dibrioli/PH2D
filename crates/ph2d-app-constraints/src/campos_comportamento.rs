//! Os ajustes dos *behaviours* (W5.4) — a parte de [`crate::campos`] que só o `Kind::Comportamento`
//! tem: que campos, quanto valem, como se escrevem e onde moram no item.

use ph2d_constraints::comportamento::{Comportamento, Onde, Perfil};
use ph2d_ecs::canal_constraint::{
    AMORTECE, AMPLITUDE, CURVAS, FASE, FREQUENCIA, LEVA_ROTACAO, NAO_SIM, ONDE, ONDES, PESO_CURVA,
    PESO_INVERTE, PESO_RAIO, PICO, PUXAO, RAIO, SEMENTE, VELOCIDADE,
};
use ph2d_ecs::canal_constraint::{DURACAO, MOLA_LIGADA, RESSALTO};
use ph2d_editor_core::constraint_edits::Unidade;

/// Os números, pela ordem do cartão (o raio do peso só com peso).
#[must_use]
pub fn numeros(c: &Comportamento) -> &'static [u16] {
    match (c.perfil, c.peso.is_some()) {
        (Perfil::Tremer, false) => &[AMPLITUDE, FREQUENCIA, SEMENTE],
        (Perfil::Tremer, true) => &[AMPLITUDE, FREQUENCIA, SEMENTE, PESO_RAIO],
        (Perfil::Flutuar, false) => &[AMPLITUDE, FREQUENCIA, FASE],
        (Perfil::Flutuar, true) => &[AMPLITUDE, FREQUENCIA, FASE, PESO_RAIO],
        (Perfil::Orbitar, peso) => match (c.segue.is_some(), peso) {
            (false, false) => &[VELOCIDADE, RAIO],
            (false, true) => &[VELOCIDADE, RAIO, PESO_RAIO],
            (true, false) => &[VELOCIDADE, RAIO, DURACAO, RESSALTO],
            (true, true) => &[VELOCIDADE, RAIO, DURACAO, RESSALTO, PESO_RAIO],
        },
        (Perfil::Atrair, false) => &[PUXAO, RAIO, PICO, AMORTECE],
        (Perfil::Atrair, true) => &[PUXAO, RAIO, PICO, AMORTECE, PESO_RAIO],
    }
}

/// As escolhas, pela ordem do cartão.
#[must_use]
pub fn escolhas(c: &Comportamento) -> &'static [u16] {
    match (c.perfil, c.peso.is_some()) {
        (Perfil::Tremer | Perfil::Flutuar, false) => &[ONDE],
        (Perfil::Tremer | Perfil::Flutuar, true) => &[ONDE, PESO_CURVA, PESO_INVERTE],
        (Perfil::Orbitar, false) => &[LEVA_ROTACAO, MOLA_LIGADA],
        (Perfil::Orbitar, true) => &[LEVA_ROTACAO, MOLA_LIGADA, PESO_CURVA, PESO_INVERTE],
        (Perfil::Atrair, false) => &[],
        (Perfil::Atrair, true) => &[PESO_CURVA, PESO_INVERTE],
    }
}

/// O rótulo de um campo daqui.
#[must_use]
pub fn rotulo(campo: u16) -> Option<&'static str> {
    Some(match campo {
        AMPLITUDE => "panel.inspector.constraint.amplitude",
        FREQUENCIA => "panel.inspector.constraint.frequency",
        SEMENTE => "panel.inspector.constraint.seed",
        FASE => "panel.inspector.constraint.phase",
        VELOCIDADE => "panel.inspector.constraint.speed",
        RAIO => "panel.inspector.constraint.radius",
        PICO => "panel.inspector.constraint.peak",
        PUXAO => "panel.inspector.constraint.strength",
        AMORTECE => "panel.inspector.constraint.damping",
        ONDE => "panel.inspector.constraint.where",
        LEVA_ROTACAO => "panel.inspector.constraint.turns_with",
        PESO_RAIO => "panel.inspector.constraint.weight_radius",
        PESO_CURVA => "panel.inspector.constraint.weight_curve",
        PESO_INVERTE => "panel.inspector.constraint.weight_invert",
        _ => return None,
    })
}

/// As palavras de uma escolha daqui.
#[must_use]
pub fn opcoes(campo: u16) -> Option<&'static [&'static str]> {
    match campo {
        ONDE => Some(ONDES),
        PESO_CURVA => Some(CURVAS),
        LEVA_ROTACAO | PESO_INVERTE => Some(NAO_SIM),
        _ => None,
    }
}

/// A unidade de um número.
#[must_use]
pub fn unidade(c: &Comportamento, campo: u16) -> Unidade {
    match campo {
        AMPLITUDE if c.onde == Onde::Rot => Unidade::Graus,
        AMPLITUDE | RAIO | PICO | PESO_RAIO => Unidade::Metros,
        VELOCIDADE => Unidade::Graus,
        DURACAO => Unidade::Segundos,
        RESSALTO => Unidade::Porcento,
        _ => Unidade::Nenhuma,
    }
}

const ONDES_DO_TIPO: [Onde; 4] = [Onde::XY, Onde::X, Onde::Y, Onde::Rot];

#[expect(clippy::cast_precision_loss, reason = "um índice pequeno")]
fn indice<T: PartialEq>(todas: &[T], v: &T) -> f32 {
    todas.iter().position(|x| x == v).unwrap_or(0) as f32
}

fn sim(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

/// O valor de um campo (`None` = este *behaviour* não o tem).
#[must_use]
#[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
pub fn ler(c: &Comportamento, campo: u16) -> Option<f32> {
    Some(match campo {
        AMPLITUDE => c.amplitude as f32,
        FREQUENCIA => c.frequencia as f32,
        SEMENTE | FASE => c.semente as f32,
        VELOCIDADE => c.velocidade as f32,
        RAIO => c.raio as f32,
        PICO => c.pico as f32,
        PUXAO => c.forca as f32,
        AMORTECE => c.arrasto as f32,
        ONDE => indice(&ONDES_DO_TIPO, &c.onde),
        LEVA_ROTACAO => sim(c.leva_rotacao),
        PESO_RAIO => c.peso?.raio as f32,
        PESO_CURVA => f32::from(c.peso?.curva),
        PESO_INVERTE => sim(c.peso?.inverte),
        MOLA_LIGADA => sim(c.segue.is_some()),
        DURACAO => c.segue?.duracao as f32,
        RESSALTO => c.segue?.ressalto as f32,
        _ => return None,
    })
}

/// Escreve um campo; `false` se não é daqui (ou não há peso para o campo do peso). O valor já vem
/// finito; os não-negativos aparam em 0.
pub fn escrever(c: &mut Comportamento, campo: u16, v: f32) -> bool {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "já aparado a ≥ 0"
    )]
    let opcao = |n: usize| (v.round().max(0.0) as usize).min(n - 1);
    let x = f64::from(v);
    match campo {
        AMPLITUDE => c.amplitude = x,
        FREQUENCIA => c.frequencia = x.max(0.0),
        SEMENTE | FASE => c.semente = x,
        VELOCIDADE => c.velocidade = x,
        RAIO => c.raio = x.max(0.0),
        PICO => c.pico = x.max(0.0),
        PUXAO => c.forca = x,
        AMORTECE => c.arrasto = x.max(0.0),
        ONDE => c.onde = ONDES_DO_TIPO[opcao(ONDES_DO_TIPO.len())],
        LEVA_ROTACAO => c.leva_rotacao = opcao(2) == 1,
        // Ligar dá a mola de fábrica do seguir suave (sem ressalto).
        MOLA_LIGADA => {
            c.segue = (opcao(2) == 1).then_some(ph2d_constraints::comportamento::SEGUE_DE_FABRICA);
        }
        DURACAO => {
            let Some(m) = &mut c.segue else { return false };
            m.duracao = x.max(0.0);
        }
        RESSALTO => {
            let Some(m) = &mut c.segue else { return false };
            m.ressalto = x.clamp(-1.0, 1.0);
        }
        PESO_RAIO | PESO_CURVA | PESO_INVERTE => {
            let Some(p) = &mut c.peso else { return false };
            match campo {
                PESO_RAIO => p.raio = x.max(0.0),
                #[expect(clippy::cast_possible_truncation, reason = "um índice < 4")]
                PESO_CURVA => p.curva = opcao(CURVAS.len()) as u8,
                _ => p.inverte = opcao(2) == 1,
            }
        }
        _ => return false,
    }
    true
}

/// A chave serde do campo dentro do item.
#[must_use]
pub fn chave_no_item(campo: u16) -> Option<&'static str> {
    Some(match campo {
        AMPLITUDE => "kind/Comportamento/amplitude",
        FREQUENCIA => "kind/Comportamento/frequencia",
        SEMENTE | FASE => "kind/Comportamento/semente",
        VELOCIDADE => "kind/Comportamento/velocidade",
        RAIO => "kind/Comportamento/raio",
        PICO => "kind/Comportamento/pico",
        PUXAO => "kind/Comportamento/forca",
        AMORTECE => "kind/Comportamento/arrasto",
        ONDE => "kind/Comportamento/onde",
        LEVA_ROTACAO => "kind/Comportamento/leva_rotacao",
        PESO_RAIO => "kind/Comportamento/peso/raio",
        PESO_CURVA => "kind/Comportamento/peso/curva",
        PESO_INVERTE => "kind/Comportamento/peso/inverte",
        MOLA_LIGADA => "kind/Comportamento/segue",
        DURACAO => "kind/Comportamento/segue/duracao",
        RESSALTO => "kind/Comportamento/segue/ressalto",
        _ => return None,
    })
}
