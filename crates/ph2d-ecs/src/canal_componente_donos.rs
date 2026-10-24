//! ⭐⭐ **Uma propriedade, um canal** (plano F27 F (d), `docs/UI_New_and_Simple/F27_F_plano.md`) —
//! as faixas do catálogo que uma família à mão JÁ é o canal: o catálogo cede-lhas. Uma faixa com
//! dono não é canal COMPONENTE: [`super::canal_componente::chaves_de`] não a gera, `resolver`/`info`
//! de uma chave dela dão `None`, e o funil do catálogo não a publica nem a escreve; o rótulo do
//! catálogo aponta o dono ([`super::canal_componente::chaves_do_rotulo`] /
//! [`super::canal_componente::props_do_rotulo`]). Sem dono, a faixa continua do catálogo.
//!
//! Cada par foi confirmado lendo OS DOIS leitores; os gates (`canal_componente_donos_tests`,
//! `canais_objeto_donos_tests` em `ph2d-app-components`, `canais_donos_tests` em `ph2d-app-physics`)
//! provam que escrever pelo dono muda o que o catálogo leria.

use crate::canal::FieldKey;
use crate::canal::familia::{FISICA, JUNTA, OBJETO};
use crate::canal_fisica as fs;
use crate::canal_junta as j;
use crate::canal_objeto as o;
use crate::scene::property::slot_in;
use crate::scene::registry::ComponentRegistry;
use crate::{Entity, World};

/// Quem é o canal de uma faixa do catálogo.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dono {
    /// O canal de uma família à mão.
    Canal(FieldKey),
    /// Um `PropKind` de sempre, pelo id travado do `AnimTarget` dele (`PropKind::target`, HR-8): o
    /// `ph2d-ecs` não conhece a `ph2d-timeline`.
    Prop(u64),
}

const fn ob(campo: u16, j: u16) -> Dono {
    Dono::Canal(FieldKey::new(OBJETO, campo, 0, j))
}
const fn fi(campo: u16) -> Dono {
    Dono::Canal(FieldKey::new(FISICA, campo, 0, 0))
}
const fn ju(campo: u16) -> Dono {
    Dono::Canal(FieldKey::new(JUNTA, campo, 0, 0))
}

/// `PropKind::Opacity` (o alfa do `Sprite::tint`) e os quatro números da junta.
pub const PROP_OPACIDADE: u64 = 5;
pub const PROP_MOTOR_ALVO: u64 = 9;
pub const PROP_MOTOR_TAXA: u64 = 10;
pub const PROP_COMPRIMENTO_REPOUSO: u64 = 11;
pub const PROP_COMPRIMENTO_MAX: u64 = 12;

const SPRITE: &str = "ph2d::render::Sprite";
const CANTOS: &str = "ph2d::ecs::SpriteCornerTint";
const CORPO: &str = "ph2d::physics::RigidBody";
const COLISOR: &str = "ph2d::physics::Collider";
const JUNTA_C: &str = "ph2d::physics::PhysicsJoint";
const RODA: &str = "ph2d::physics::PulleyWheel";

/// ⭐ **A tabela** — `(canonical_name, field_id, faixa, dono)`.
pub const DONOS: &[(&str, u16, u16, Dono)] = &[
    // OBJETO — a aparência da sprite.
    (SPRITE, 1, 0, ob(o::TINTA, 0)),
    (SPRITE, 1, 1, ob(o::TINTA + 1, 0)),
    (SPRITE, 1, 2, ob(o::TINTA + 2, 0)),
    (SPRITE, 1, 3, Dono::Prop(PROP_OPACIDADE)),
    (SPRITE, 6, 0, ob(o::AUTO_TINTA, 0)),
    (SPRITE, 6, 1, ob(o::AUTO_TINTA + 1, 0)),
    (SPRITE, 6, 2, ob(o::AUTO_TINTA + 2, 0)),
    (SPRITE, 6, 3, ob(o::AUTO_TINTA + 3, 0)),
    (SPRITE, 7, 0, ob(o::OPACIDADE, 0)),
    (SPRITE, 8, 0, ob(o::SILHUETA, 0)),
    ("ph2d::ecs::SpriteEmissive", 1, 0, ob(o::EMISSAO, 0)),
    // Os quatro cantos: o campo `n` é o canto `j = n - 1`.
    (CANTOS, 1, 0, ob(o::CANTOS, 0)),
    (CANTOS, 1, 1, ob(o::CANTOS + 1, 0)),
    (CANTOS, 1, 2, ob(o::CANTOS + 2, 0)),
    (CANTOS, 1, 3, ob(o::CANTOS + 3, 0)),
    (CANTOS, 2, 0, ob(o::CANTOS, 1)),
    (CANTOS, 2, 1, ob(o::CANTOS + 1, 1)),
    (CANTOS, 2, 2, ob(o::CANTOS + 2, 1)),
    (CANTOS, 2, 3, ob(o::CANTOS + 3, 1)),
    (CANTOS, 3, 0, ob(o::CANTOS, 2)),
    (CANTOS, 3, 1, ob(o::CANTOS + 1, 2)),
    (CANTOS, 3, 2, ob(o::CANTOS + 2, 2)),
    (CANTOS, 3, 3, ob(o::CANTOS + 3, 2)),
    (CANTOS, 4, 0, ob(o::CANTOS, 3)),
    (CANTOS, 4, 1, ob(o::CANTOS + 1, 3)),
    (CANTOS, 4, 2, ob(o::CANTOS + 2, 3)),
    (CANTOS, 4, 3, ob(o::CANTOS + 3, 3)),
    // FISICA — a §11 Physics Body.
    (CORPO, 0, 0, fi(fs::TIPO)),
    (COLISOR, 1, 0, fi(fs::DENSIDADE)),
    (COLISOR, 2, 0, fi(fs::RESTITUICAO)),
    (COLISOR, 3, 0, fi(fs::ATRITO)),
    (COLISOR, 4, 0, fi(fs::CAMADA)),
    (COLISOR, 5, 0, fi(fs::SENSOR)),
    (COLISOR, 6, 0, fi(fs::DESLOC_X)),
    (COLISOR, 6, 1, fi(fs::DESLOC_Y)),
    ("ph2d::physics::DampingOverride", 0, 0, fi(fs::AMORT_LINEAR)),
    (
        "ph2d::physics::DampingOverride",
        1,
        0,
        fi(fs::AMORT_ANGULAR),
    ),
    ("ph2d::physics::DampingOverride", 2, 0, fi(fs::AMORT_MODO)),
    ("ph2d::physics::Dominance", 0, 0, fi(fs::DOMINANCIA)),
    (
        "ph2d::physics::GravityScale",
        0,
        0,
        fi(fs::ESCALA_GRAVIDADE),
    ),
    ("ph2d::physics::InitialVelocity", 0, 0, fi(fs::VEL_X)),
    ("ph2d::physics::InitialVelocity", 0, 1, fi(fs::VEL_Y)),
    ("ph2d::physics::InitialVelocity", 1, 0, fi(fs::VEL_ANGULAR)),
    ("ph2d::physics::MassOverride", 0, 0, fi(fs::MASSA)),
    (
        "ph2d::physics::MaterialCombine",
        0,
        0,
        fi(fs::COMBINA_RESTITUICAO),
    ),
    (
        "ph2d::physics::MaterialCombine",
        1,
        0,
        fi(fs::COMBINA_ATRITO),
    ),
    ("ph2d::physics::WalkSurface", 0, 0, fi(fs::ADERENCIA)),
    ("ph2d::physics::WalkSurface", 1, 0, fi(fs::ESTEIRA)),
    ("ph2d::physics::AreaEffector", 0, 0, fi(fs::FORCA_X)),
    ("ph2d::physics::AreaEffector", 0, 1, fi(fs::FORCA_Y)),
    ("ph2d::physics::AreaDrag", 0, 0, fi(fs::ZONA_ARRASTO)),
    ("ph2d::physics::AreaBuoyancy", 0, 0, fi(fs::ZONA_DENSIDADE)),
    (
        "ph2d::physics::AreaFormDrag",
        0,
        0,
        fi(fs::ZONA_ARRASTO_FORMA),
    ),
    ("ph2d::physics::AreaTorque", 0, 0, fi(fs::ZONA_TORQUE)),
    ("ph2d::physics::AreaFalloff", 0, 0, fi(fs::ZONA_QUEDA)),
    // JUNTA — a §12 e a §13 (a roldana); os quatro números de sempre são `PropKind`.
    (JUNTA_C, 3, 0, ju(j::TIPO)),
    (JUNTA_C, 4, 0, ju(j::LIMITES_LIGADOS)),
    (JUNTA_C, 5, 0, ju(j::LIMITE_MIN)),
    (JUNTA_C, 6, 0, ju(j::LIMITE_MAX)),
    (JUNTA_C, 7, 0, ju(j::MOTOR_LIGADO)),
    (JUNTA_C, 8, 0, Dono::Prop(PROP_MOTOR_TAXA)),
    (JUNTA_C, 9, 0, ju(j::MOTOR_FORCA)),
    (JUNTA_C, 10, 0, Dono::Prop(PROP_COMPRIMENTO_REPOUSO)),
    (JUNTA_C, 11, 0, ju(j::RIGIDEZ)),
    (JUNTA_C, 12, 0, ju(j::AMORTECIMENTO)),
    (JUNTA_C, 13, 0, Dono::Prop(PROP_COMPRIMENTO_MAX)),
    (JUNTA_C, 17, 0, ju(j::MOTOR_MODO)),
    (JUNTA_C, 18, 0, Dono::Prop(PROP_MOTOR_ALVO)),
    (JUNTA_C, 19, 0, ju(j::QUEBRA_LIGADA)),
    (JUNTA_C, 20, 0, ju(j::QUEBRA_FORCA)),
    (JUNTA_C, 21, 0, ju(j::QUEBRA_TORQUE)),
    (JUNTA_C, 22, 0, ju(j::ATIVA)),
    (JUNTA_C, 23, 0, ju(j::COLIDE_PAR)),
    (JUNTA_C, 24, 0, ju(j::SOLDA_MOLE)),
    (RODA, 2, 0, ju(j::RODA_ORDEM)),
    (RODA, 3, 0, ju(j::RODA_RAIO)),
    (RODA, 4, 0, ju(j::RODA_LADO)),
    (RODA, 5, 0, ju(j::RODA_MOTOR)),
    (RODA, 6, 0, ju(j::RODA_QUEBRA_LIGADA)),
    (RODA, 10, 0, ju(j::RODA_QUEBRA_FORCA)),
    (RODA, 11, 0, ju(j::RODA_RAIO_SAIDA)),
];

/// O dono da faixa `faixa` do campo `field_id` de `canonical` — `None` = é canal do catálogo.
#[must_use]
pub fn dono(canonical: &str, field_id: u16, faixa: u16) -> Option<Dono> {
    DONOS
        .iter()
        .find(|&&(c, f, x, _)| c == canonical && f == field_id && x == faixa)
        .map(|&(.., d)| d)
}

/// ⭐ **O que o CATÁLOGO leria** da faixa no componente da entidade, com dono ou sem ele — o lado
/// do catálogo nos gates dos donos (o funil do catálogo lê só as faixas sem dono).
#[must_use]
pub fn ler_pelo_catalogo(
    reg: &ComponentRegistry,
    w: &World,
    e: Entity,
    canonical: &str,
    field_id: u16,
    faixa: u16,
) -> Option<f32> {
    let d = ph2d_component_desc::desc_for(canonical)?;
    let f = d.fields.iter().find(|f| f.field_id == field_id)?;
    let mut v = (reg.get_by_name(canonical)?.value_of)(w, e).ok()??;
    super::canal_componente::ler_valor(f, faixa, slot_in(&mut v, f.serde_key())?)
}

#[cfg(test)]
#[path = "canal_componente_donos_tests.rs"]
mod tests;
