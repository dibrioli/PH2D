//! **O vocabulário de canais da JUNTA** ([`crate::canal::familia::JUNTA`]) — os campos do
//! `PhysicsJoint` que a §12 do Inspector edita e não são um dos `PropKind` de sempre (o alvo e a
//! velocidade do motor, o comprimento de repouso e o máximo FICAM `PropKind`). Quem lê e escreve é
//! `ph2d-app-physics` (`canais_junta.rs`), pelo MESMO funil que a edição do painel usa
//! ([plano 35 §3.0](../../../docs/UI_New_and_Simple/vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md)).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_da_junta_nao_renumera`).
//!
//! O valor é o do DOCUMENTO (o componente): limites em radianos numa dobradiça e em metros num
//! curso — a mesma unidade do `PropKind::JointMotorTarget`; um bool é 0/1 (um marcador também: a
//! presença); um enum é a `tag`. A §13 (a roldana) mora aqui: uma Pulley é um tipo de junta.
//! Cada linha do painel é um grupo (um losango por linha).

use crate::canal::CampoInfo;

/// `motor_enabled`.
pub const MOTOR_LIGADO: u16 = 1;
/// `motor_mode` (Velocity 0 · Position 1).
pub const MOTOR_MODO: u16 = 2;
/// `motor_max_force`.
pub const MOTOR_FORCA: u16 = 3;
/// `custom.motor_axis` (X 0 · Y 1 · Rotation 2) — o eixo do motor de um Custom.
pub const MOTOR_EIXO: u16 = 4;
/// `break_enabled`, `break_force` (N), `break_torque` (N·m).
pub const QUEBRA_LIGADA: u16 = 10;
pub const QUEBRA_FORCA: u16 = 11;
pub const QUEBRA_TORQUE: u16 = 12;
/// `limits_enabled`, `limit_min`, `limit_max` (a unidade do tipo).
pub const LIMITES_LIGADOS: u16 = 20;
pub const LIMITE_MIN: u16 = 21;
pub const LIMITE_MAX: u16 = 22;
/// `soft` (a solda que cede), `stiffness`, `damping`.
pub const SOLDA_MOLE: u16 = 30;
pub const RIGIDEZ: u16 = 31;
pub const AMORTECIMENTO: u16 = 32;
/// Os eixos de um Custom (X · Y · Rotation): o do eixo `a` é `EIXO_* + EIXO_PASSO * a` — o modo
/// (Free 0 · Limited 1 · Locked 2) e o par min/max (metros nos lineares, radianos na rotação).
/// Um campo por eixo (e não um `i`) para que o dope-sheet diga de que eixo é a linha.
pub const EIXO_MODO: u16 = 40;
pub const EIXO_MIN: u16 = 41;
pub const EIXO_MAX: u16 = 42;
pub const EIXO_PASSO: u16 = 3;

/// Quantos eixos um Custom tem.
pub const N_EIXOS: u16 = 3;

/// `kind` (a tag de `JointKind::ALL`), `active`, `collide_connected`.
pub const TIPO: u16 = 50;
pub const ATIVA: u16 = 51;
pub const COLIDE_PAR: u16 = 52;
/// O lado B preso ao MUNDO — a presença do marcador `JointWorldAnchor` (0/1).
pub const PONTA_NO_MUNDO: u16 = 53;

/// **A §13 — a roldana selecionada** (`PulleyWheel`, numa entidade que não é a corda): o raio, o
/// segundo raio, a talha de Weston (o marcador `WestonAxle`, 0/1), a posição na rota (`order`,
/// 0-based), o lado da corda (`WrapSide`), o motor (rad/s) e a quebra do eixo.
pub const RODA_RAIO: u16 = 60;
pub const RODA_RAIO_SAIDA: u16 = 61;
pub const RODA_WESTON: u16 = 62;
pub const RODA_ORDEM: u16 = 63;
pub const RODA_LADO: u16 = 64;
pub const RODA_MOTOR: u16 = 65;
pub const RODA_QUEBRA_LIGADA: u16 = 66;
pub const RODA_QUEBRA_FORCA: u16 = 67;

/// O campo `base` (`EIXO_MODO`/`EIXO_MIN`/`EIXO_MAX`) do eixo `a`.
#[must_use]
pub const fn do_eixo(base: u16, a: u16) -> u16 {
    base + EIXO_PASSO * a
}

/// O eixo e a base de um campo de eixo — `None` fora do bloco.
#[must_use]
pub const fn eixo_de(campo: u16) -> Option<(u16, u16)> {
    if campo < EIXO_MODO || campo >= EIXO_MODO + EIXO_PASSO * N_EIXOS {
        return None;
    }
    let off = campo - EIXO_MODO;
    Some((off / EIXO_PASSO, EIXO_MODO + off % EIXO_PASSO))
}

const EIXO_ROTULOS: [[&str; 3]; 3] = [
    [
        "canal.junta.eixo_x_modo",
        "canal.junta.eixo_x_min",
        "canal.junta.eixo_x_max",
    ],
    [
        "canal.junta.eixo_y_modo",
        "canal.junta.eixo_y_min",
        "canal.junta.eixo_y_max",
    ],
    [
        "canal.junta.eixo_rot_modo",
        "canal.junta.eixo_rot_min",
        "canal.junta.eixo_rot_max",
    ],
];

const fn c(rotulo: &'static str, grupo: u16) -> CampoInfo {
    CampoInfo {
        rotulo,
        discreto: false,
        grupo,
        por_i: false,
        texto: false,
    }
}

const fn d(rotulo: &'static str, grupo: u16) -> CampoInfo {
    CampoInfo {
        discreto: true,
        ..c(rotulo, grupo)
    }
}

/// O que o campo `campo` da junta é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    if let Some((a, base)) = eixo_de(campo) {
        let rotulo = EIXO_ROTULOS[a as usize][(base - EIXO_MODO) as usize];
        return Some(if base == EIXO_MODO {
            d(rotulo, campo)
        } else {
            c(rotulo, campo)
        });
    }
    Some(match campo {
        MOTOR_LIGADO => d("canal.junta.motor_ligado", campo),
        MOTOR_MODO => d("canal.junta.motor_modo", campo),
        MOTOR_FORCA => c("canal.junta.motor_forca", campo),
        MOTOR_EIXO => d("canal.junta.motor_eixo", campo),
        QUEBRA_LIGADA => d("canal.junta.quebra_ligada", campo),
        QUEBRA_FORCA => c("canal.junta.quebra_forca", campo),
        QUEBRA_TORQUE => c("canal.junta.quebra_torque", campo),
        LIMITES_LIGADOS => d("canal.junta.limites_ligados", campo),
        LIMITE_MIN => c("canal.junta.limite_min", campo),
        LIMITE_MAX => c("canal.junta.limite_max", campo),
        SOLDA_MOLE => d("canal.junta.solda_mole", campo),
        RIGIDEZ => c("canal.junta.rigidez", campo),
        AMORTECIMENTO => c("canal.junta.amortecimento", campo),
        TIPO => d("canal.junta.tipo", campo),
        ATIVA => d("canal.junta.ativa", campo),
        COLIDE_PAR => d("canal.junta.colide_par", campo),
        PONTA_NO_MUNDO => d("canal.junta.ponta_no_mundo", campo),
        RODA_RAIO => c("canal.junta.roda_raio", campo),
        RODA_RAIO_SAIDA => c("canal.junta.roda_raio_saida", campo),
        RODA_WESTON => d("canal.junta.roda_weston", campo),
        RODA_ORDEM => d("canal.junta.roda_ordem", campo),
        RODA_LADO => d("canal.junta.roda_lado", campo),
        RODA_MOTOR => c("canal.junta.roda_motor", campo),
        RODA_QUEBRA_LIGADA => d("canal.junta.roda_quebra_ligada", campo),
        RODA_QUEBRA_FORCA => c("canal.junta.roda_quebra_forca", campo),
        _ => return None,
    })
}

/// O nome do campo dentro do grupo: cada grupo da junta é um campo só, o nome é o do grupo.
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(info(campo)?.rotulo)
}

#[cfg(test)]
#[path = "canal_junta_tests.rs"]
mod tests;
