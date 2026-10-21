//! **O vocabulário de canais do CORPO de física** ([`crate::canal::familia::FISICA`]) — o que a
//! §11 *Physics Body* do Inspector edita: o corpo (`RigidBody` e os opcionais dele), o colisor
//! (`Collider`, a superfície, os sinalizadores) e a zona de um sensor (plano 35 §A.5). Quem lê e
//! escreve é `ph2d-app-physics` (`canais_corpo.rs`), pelo MESMO funil que o painel usa
//! ([plano 35 §3.0](../../../docs/UI_New_and_Simple/vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md)).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_da_fisica_nao_renumera`).
//!
//! O valor é o do COMPONENTE: metros, kg, m/s, a rotação inicial em rad/s; um interruptor é 0/1;
//! uma escolha é a `tag` (tipo, forma, camada, regras de combinação). Um vector de duas componentes
//! (o deslocamento, a caixa, a velocidade, a força da zona) é UM grupo, como a posição do
//! Transform; o resto, uma linha do painel por grupo.
//!
//! ⛔ Fora (plano 35 §8): Add/Remove/peça (estrutura), Bake e o que ele grava (comando), juntar e
//! o rig (estrutura), os sinais e o filtro por tag (fiação por nome/identidade).

use crate::canal::CampoInfo;

/// `RigidBody::kind` (Dynamic 0 · Static 1 · Kinematic 2).
pub const TIPO: u16 = 1;
/// O marcador `Ccd` (0 discreto · 1 contínuo).
pub const CCD: u16 = 2;
/// Os marcadores `LockRotation`, `LockPositionX`, `LockPositionY`.
pub const TRAVA_ROTACAO: u16 = 3;
pub const TRAVA_X: u16 = 4;
pub const TRAVA_Y: u16 = 5;
/// `GravityScale` (ausente = 1).
pub const ESCALA_GRAVIDADE: u16 = 6;
/// `Dominance` (a prioridade `i8`; ausente = 0).
pub const DOMINANCIA: u16 = 7;
/// `InitialVelocity` — a linear (m/s, um grupo) e a angular (rad/s).
pub const VEL_X: u16 = 10;
pub const VEL_Y: u16 = 11;
pub const VEL_ANGULAR: u16 = 12;
/// `DampingOverride` — linear, angular e o modo (Combine 0 · Replace 1).
pub const AMORT_LINEAR: u16 = 13;
pub const AMORT_ANGULAR: u16 = 14;
pub const AMORT_MODO: u16 = 15;
/// `MassOverride`: a fonte (0 auto · 1 manual) e a massa em kg.
pub const MASSA_MANUAL: u16 = 16;
pub const MASSA: u16 = 17;
/// `Collider::shape` (Ball 0 · Box 1 · Capsule 2) e as medidas da forma (m).
pub const FORMA: u16 = 20;
pub const RAIO: u16 = 21;
pub const MEIO_X: u16 = 22;
pub const MEIO_Y: u16 = 23;
pub const CAPSULA: u16 = 24;
/// `Collider::offset` (m, um grupo).
pub const DESLOC_X: u16 = 25;
pub const DESLOC_Y: u16 = 26;
/// `Collider::density`, `restitution`, `friction`.
pub const DENSIDADE: u16 = 27;
pub const RESTITUICAO: u16 = 28;
pub const ATRITO: u16 = 29;
/// `MaterialCombine` — as regras de combinação (a `tag` do `CombineRule`).
pub const COMBINA_RESTITUICAO: u16 = 30;
pub const COMBINA_ATRITO: u16 = 31;
/// `Collider::layer`, `Collider::is_sensor`, e os marcadores `OneWayPlatform` e `NoWallCling`.
pub const CAMADA: u16 = 32;
pub const SENSOR: u16 = 33;
pub const UMA_VIA: u16 = 34;
pub const SEM_PAREDE: u16 = 35;
/// `WalkSurface` — a aderência e a esteira (m/s).
pub const ADERENCIA: u16 = 36;
pub const ESTEIRA: u16 = 37;
/// A zona de um sensor: `AreaEffector` (N, um grupo), o marcador `AreaForceWorldAxes`, e
/// `AreaDrag`, `AreaBuoyancy`, `AreaFormDrag`, `AreaTorque`, `AreaFalloff`.
pub const FORCA_X: u16 = 40;
pub const FORCA_Y: u16 = 41;
pub const FORCA_NO_MUNDO: u16 = 42;
pub const ZONA_ARRASTO: u16 = 43;
pub const ZONA_DENSIDADE: u16 = 44;
pub const ZONA_ARRASTO_FORMA: u16 = 45;
pub const ZONA_TORQUE: u16 = 46;
pub const ZONA_QUEDA: u16 = 47;

/// Todos os campos, na ordem do vocabulário.
pub const CAMPOS: [u16; 41] = [
    TIPO,
    CCD,
    TRAVA_ROTACAO,
    TRAVA_X,
    TRAVA_Y,
    ESCALA_GRAVIDADE,
    DOMINANCIA,
    VEL_X,
    VEL_Y,
    VEL_ANGULAR,
    AMORT_LINEAR,
    AMORT_ANGULAR,
    AMORT_MODO,
    MASSA_MANUAL,
    MASSA,
    FORMA,
    RAIO,
    MEIO_X,
    MEIO_Y,
    CAPSULA,
    DESLOC_X,
    DESLOC_Y,
    DENSIDADE,
    RESTITUICAO,
    ATRITO,
    COMBINA_RESTITUICAO,
    COMBINA_ATRITO,
    CAMADA,
    SENSOR,
    UMA_VIA,
    SEM_PAREDE,
    ADERENCIA,
    ESTEIRA,
    FORCA_X,
    FORCA_Y,
    FORCA_NO_MUNDO,
    ZONA_ARRASTO,
    ZONA_DENSIDADE,
    ZONA_ARRASTO_FORMA,
    ZONA_TORQUE,
    ZONA_QUEDA,
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

/// O que o campo `campo` do corpo é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    Some(match campo {
        TIPO => d("canal.fisica.tipo", campo),
        CCD => d("canal.fisica.ccd", campo),
        TRAVA_ROTACAO => d("canal.fisica.trava_rotacao", campo),
        TRAVA_X => d("canal.fisica.trava_x", campo),
        TRAVA_Y => d("canal.fisica.trava_y", campo),
        ESCALA_GRAVIDADE => c("canal.fisica.escala_gravidade", campo),
        DOMINANCIA => d("canal.fisica.dominancia", campo),
        VEL_X | VEL_Y => c("canal.fisica.velocidade", VEL_X),
        VEL_ANGULAR => c("canal.fisica.vel_angular", campo),
        AMORT_LINEAR => c("canal.fisica.amort_linear", campo),
        AMORT_ANGULAR => c("canal.fisica.amort_angular", campo),
        AMORT_MODO => d("canal.fisica.amort_modo", campo),
        MASSA_MANUAL => d("canal.fisica.massa_manual", campo),
        MASSA => c("canal.fisica.massa", campo),
        FORMA => d("canal.fisica.forma", campo),
        RAIO => c("canal.fisica.raio", campo),
        MEIO_X | MEIO_Y => c("canal.fisica.caixa", MEIO_X),
        CAPSULA => c("canal.fisica.capsula", campo),
        DESLOC_X | DESLOC_Y => c("canal.fisica.deslocamento", DESLOC_X),
        DENSIDADE => c("canal.fisica.densidade", campo),
        RESTITUICAO => c("canal.fisica.restituicao", campo),
        ATRITO => c("canal.fisica.atrito", campo),
        COMBINA_RESTITUICAO => d("canal.fisica.combina_restituicao", campo),
        COMBINA_ATRITO => d("canal.fisica.combina_atrito", campo),
        CAMADA => d("canal.fisica.camada", campo),
        SENSOR => d("canal.fisica.sensor", campo),
        UMA_VIA => d("canal.fisica.uma_via", campo),
        SEM_PAREDE => d("canal.fisica.sem_parede", campo),
        ADERENCIA => c("canal.fisica.aderencia", campo),
        ESTEIRA => c("canal.fisica.esteira", campo),
        FORCA_X | FORCA_Y => c("canal.fisica.forca", FORCA_X),
        FORCA_NO_MUNDO => d("canal.fisica.forca_no_mundo", campo),
        ZONA_ARRASTO => c("canal.fisica.zona_arrasto", campo),
        ZONA_DENSIDADE => c("canal.fisica.zona_densidade", campo),
        ZONA_ARRASTO_FORMA => c("canal.fisica.zona_arrasto_forma", campo),
        ZONA_TORQUE => c("canal.fisica.zona_torque", campo),
        ZONA_QUEDA => c("canal.fisica.zona_queda", campo),
        _ => return None,
    })
}

/// O nome do campo dentro do grupo: os pares nomeiam o eixo; o resto, o grupo.
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(match campo {
        VEL_X | MEIO_X | DESLOC_X | FORCA_X => "canal.fisica.x",
        VEL_Y | MEIO_Y | DESLOC_Y | FORCA_Y => "canal.fisica.y",
        _ => info(campo)?.rotulo,
    })
}

#[cfg(test)]
#[path = "canal_fisica_tests.rs"]
mod tests;
