//! **Os campos que moram em componentes da ENTIDADE** (e não no `VecPath`) — a porta única que o
//! [`crate::canais`] encaminha: os efeitos vivos ([`crate::canais_efeitos`]), os filtros
//! ([`crate::canais_filtros`]), os componentes vivos ([`crate::canais_vivos`]) e a Interface
//! ([`crate::canais_interface`]). Uma entidade sem caminho (o contentor de um Envelope) tem estes
//! campos na mesma.

use ph2d_ecs::canal_vetor as c;
use ph2d_ecs::{Entity, FieldKey, SimWorld, TextosDosCanais, VecContour, VecFilter, VecOffset};

use crate::canais_efeitos::{Efeitos, e_de_efeito};
use crate::canais_filtros::e_de_filtro;
use crate::canais_interface::e_da_interface;
use crate::canais_vivos::e_vivo;

/// O campo `kk` mora num componente da entidade?
#[must_use]
pub fn e_da_entidade(kk: FieldKey) -> bool {
    e_de_efeito(kk) || e_de_filtro(kk) || e_vivo(kk) || e_da_interface(kk)
}

/// O que a entidade é, para os interruptores que só se oferecem a algumas: ter um caminho (o
/// perfil de largura) e ele ser fechado (o recorte).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Alvo {
    pub caminho: bool,
    pub fechado: bool,
}

fn efeitos(sim: &SimWorld, ent: Entity) -> Efeitos {
    Efeitos {
        contorno: sim.world().get::<VecContour>(ent).copied(),
        offset: sim.world().get::<VecOffset>(ent).copied(),
    }
}

/// Os campos que a entidade tem agora nos seus componentes.
#[must_use]
pub fn campos(sim: &SimWorld, ent: Entity, alvo: Alvo) -> Vec<FieldKey> {
    let mut out = efeitos(sim, ent).campos();
    if let Some(f) = sim.world().get::<VecFilter>(ent) {
        out.extend(crate::canais_filtros::campos(f));
    }
    crate::canais_vivos::campos(sim, ent, alvo.caminho, &mut out);
    crate::canais_interface::campos(sim, ent, alvo.fechado, &mut out);
    out
}

/// Quanto vale `kk` na entidade — `None` se o componente dele não está lá.
pub fn ler(sim: &SimWorld, ent: Entity, kk: FieldKey, textos: &mut TextosDosCanais) -> Option<f32> {
    if e_de_efeito(kk) {
        efeitos(sim, ent).ler(kk)
    } else if e_de_filtro(kk) {
        crate::canais_filtros::ler(sim.world().get::<VecFilter>(ent)?, kk)
    } else if e_vivo(kk) {
        crate::canais_vivos::ler(sim, ent, kk)
    } else {
        crate::canais_interface::ler(sim, ent, kk, textos)
    }
}

/// ⭐ Aplica `v` a `kk` na entidade — e só se o campo, VISTO PELO CANAL, difere (o idioma do
/// `canais_campos::escrever`: reescrever o que se leu não suja o documento nem a detecção de
/// mudança do ECS).
pub fn escrever(
    sim: &mut SimWorld,
    ent: Entity,
    kk: FieldKey,
    v: f32,
    textos: &mut TextosDosCanais,
) {
    if !e_da_entidade(kk) || ler(sim, ent, kk, textos) == Some(v) {
        return;
    }
    if e_de_efeito(kk) {
        let antes = efeitos(sim, ent);
        let mut fx = antes;
        fx.escrever(kk, v);
        let mut em = sim.world_mut().entity_mut(ent);
        if fx.contorno != antes.contorno
            && let Some(ct) = fx.contorno
        {
            em.insert(ct);
        }
        if fx.offset != antes.offset
            && let Some(o) = fx.offset
        {
            em.insert(o);
        }
    } else if e_de_filtro(kk) {
        if let Some(mut f) = sim.world_mut().get_mut::<VecFilter>(ent) {
            crate::canais_filtros::escrever(&mut f, kk, v);
        }
    } else if e_vivo(kk) {
        crate::canais_vivos::escrever(sim, ent, kk, v);
    } else {
        crate::canais_interface::escrever(sim, ent, kk, v, textos);
    }
}

/// A ordem das escritas de um quadro na entidade: os interruptores de presença, depois o tipo,
/// depois as contagens, e só então os números que eles abrigam (o irmão do
/// `canais_campos::prioridade`).
#[must_use]
pub fn prioridade(kk: FieldKey) -> u8 {
    match kk.campo {
        c::PERFIL_EXISTE
        | c::ANCORAS_EXISTE
        | c::LAYOUT_DIR
        | c::LAYOUT_TEM_BASE
        | c::LAYOUT_TEM_MIN
        | c::LAYOUT_TEM_MAX => 0,
        c::FILTRO_TIPO | c::ENVELOPE_TIPO => 1,
        c::FILTRO_N_PARAGENS | c::PERFIL_N | c::ENVELOPE_N_PINOS => 2,
        _ => 3,
    }
}
