//! **A chave de compensação do Auto Key** — pegar e largar sem estragar o passado.
//!
//! Ligar um item no quadro f (força 0→1) com o Auto Key grava UMA chave: a curva fica constante e o
//! item passa a valer desde o quadro 0. Largar (1→0) interpola desde a chave anterior. O cartão
//! regista o gesto ([`Pegas`], em `inspector::liga`) e [`compensar`] devolve as chaves: o campo em
//! f−1 com o valor de ANTES (degrau) e em f com o novo; ao largar um Child Of, o `Transform` em f−1
//! com a pose autorada de antes e em f com a pose que se vê (a que o [`Adotar`] torna autorada).
//!
//! [`Adotar`]: ph2d_constraints_ecs::Adotar

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use ph2d_ecs::canal_constraint::chave;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_timeline::{Instante, Interp, PropKind, TimelineDoc};

use crate::chaves::Escrita;

/// Um item que passou de inerte a activo ou o contrário, à espera das chaves do quadro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pega {
    pub dono: u64,
    pub item: u32,
    /// O canal que mudou (`FORCA` ou `LIGADA`).
    pub campo: u16,
    pub antes: f32,
    pub depois: f32,
    /// Largou um Child Of: a pose também se grava.
    pub larga_pose: bool,
}

/// Os gestos desde o último [`compensar`].
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct Pegas(pub Vec<Pega>);

/// Os cinco canais da pose, na ordem do `Transform`.
pub(crate) const POSE: [PropKind; 5] = [
    PropKind::TranslationX,
    PropKind::TranslationY,
    PropKind::Rotation,
    PropKind::ScaleX,
    PropKind::ScaleY,
];

/// O valor de `p` na pose `t`.
pub(crate) fn da_pose(t: &Transform, p: PropKind) -> f32 {
    match p {
        PropKind::TranslationX => t.translation.x,
        PropKind::TranslationY => t.translation.y,
        PropKind::Rotation => t.rotation,
        PropKind::ScaleX => t.scale.x,
        _ => t.scale.y,
    }
}

/// O que a cena mostrava em `t`: a curva, se há; senão `sem_curva`.
fn visto(doc: &TimelineDoc, e: u64, p: PropKind, t: f64, solo: bool, sem_curva: f32) -> f32 {
    let fonte = ph2d_timeline::remapped_time(doc, e, t);
    ph2d_timeline::curve_seen(doc, e, p, fonte, solo).unwrap_or(sem_curva)
}

/// ⭐ **As chaves dos gestos do quadro.** Drena sempre [`Pegas`]; só as devolve `armado` (o Auto
/// Key). `interp` = a da chave nova da casa (a de f); a de f−1 é um degrau ([`Interp::Hold`]). Ler
/// ANTES de o Auto Key gravar f (senão f−1 lê a chave nova — medido no app) e antes de a pilha voltar
/// a correr: o `preview` ainda sabe a pose autorada de quem a pilha conduzia, e o `Transform` vivo é
/// a pose que se vê.
#[must_use]
pub fn compensar(
    doc: &TimelineDoc,
    sim: &mut SimWorld,
    preview: &PreviewDrive,
    inst: Instante,
    armado: bool,
    interp: Interp,
) -> Vec<Escrita> {
    let pegas = sim
        .world_mut()
        .remove_resource::<Pegas>()
        .unwrap_or_default();
    if !armado || doc.fps_display.is_nan() || doc.fps_display <= 0.0 {
        return Vec::new();
    }
    let (f, solo) = (inst.t_cut, inst.solo);
    let antes = f - 1.0 / doc.fps_display;
    let mut out = Vec::new();
    let mut par = |e, p, v_antes, v| {
        out.push(Escrita::Chave {
            e,
            p,
            t: antes,
            v: v_antes,
            interp: Interp::Hold,
        });
        out.push(Escrita::Chave {
            e,
            p,
            t: f,
            v,
            interp,
        });
    };
    for g in pegas.0 {
        let p = PropKind::Field(chave(g.campo, g.item));
        par(
            g.dono,
            p,
            visto(doc, g.dono, p, antes, solo, g.antes),
            g.depois,
        );
        if !g.larga_pose {
            continue;
        }
        let Some(vista) =
            Entity::try_from_bits(g.dono).and_then(|e| sim.world().get::<Transform>(e).copied())
        else {
            continue;
        };
        let autorada = match preview.authored(g.dono, Driver::ConstraintPose) {
            Some(Driven::ConstraintPose(t)) => t,
            _ => vista,
        };
        for p in POSE {
            let v = visto(doc, g.dono, p, antes, solo, da_pose(&autorada, p));
            par(g.dono, p, v, da_pose(&vista, p));
        }
    }
    out
}
