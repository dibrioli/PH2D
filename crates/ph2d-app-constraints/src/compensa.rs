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
use ph2d_anim::RationalTime;
use ph2d_ecs::canal_constraint::chave;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_timeline::{
    AnimValue, Instante, Interp, PropKind, TimelineDoc, TimelineIntent, TimelineState,
};

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

const POSE: [PropKind; 5] = [
    PropKind::TranslationX,
    PropKind::TranslationY,
    PropKind::Rotation,
    PropKind::ScaleX,
    PropKind::ScaleY,
];

fn da_pose(t: &Transform, p: PropKind) -> f32 {
    match p {
        PropKind::TranslationX => t.translation.x,
        PropKind::TranslationY => t.translation.y,
        PropKind::Rotation => t.rotation,
        PropKind::ScaleX => t.scale.x,
        _ => t.scale.y,
    }
}

/// A chave de `p` no instante `t` (relógio da vista): o tempo e o valor pelas portas do Auto Key
/// (`remapped_time`/`key_home`, `key_value_in_active_clip`). `None` = o clip recusa.
fn chave_em(
    doc: &TimelineDoc,
    e: u64,
    p: PropKind,
    t: f64,
    solo: bool,
    valor: f32,
    interp: Interp,
) -> Option<TimelineIntent> {
    let fonte = ph2d_timeline::remapped_time(doc, e, t);
    let casa = if solo {
        fonte
    } else {
        ph2d_timeline::key_home(doc, e, t).ok()?
    };
    let value = if solo {
        valor
    } else {
        ph2d_timeline::key_value_in_active_clip(doc, e, p, valor, fonte).ok()?
    };
    Some(TimelineIntent::AddKey {
        entity: e,
        prop: p,
        t: ph2d_timeline::snap_time(RationalTime::from_seconds(casa), doc.fps_display, true),
        value: AnimValue::Float(value),
        interp,
    })
}

/// O que a cena mostrava em `t`: a curva, se há; senão `sem_curva`.
fn visto(doc: &TimelineDoc, e: u64, p: PropKind, t: f64, solo: bool, sem_curva: f32) -> f32 {
    let fonte = ph2d_timeline::remapped_time(doc, e, t);
    ph2d_timeline::curve_seen(doc, e, p, fonte, solo).unwrap_or(sem_curva)
}

/// ⭐ **As chaves dos gestos do quadro.** Drena sempre [`Pegas`]; só devolve chaves com o Auto Key
/// armado (`painel` da timeline aberto e a flag, como o `autokey_pass`). `interp` = a da chave nova
/// da casa (a de f); a de f−1 é um degrau ([`Interp::Hold`]). Chamar no mesmo quadro do commit do
/// cartão, antes de a pilha voltar a correr: o `preview` ainda sabe a pose autorada de quem a pilha
/// conduzia, e o `Transform` vivo é a pose que se vê.
#[must_use]
pub fn compensar(
    tl: &TimelineState,
    painel: bool,
    sim: &mut SimWorld,
    preview: &PreviewDrive,
    inst: Instante,
    interp: Interp,
) -> Vec<TimelineIntent> {
    let pegas = sim
        .world_mut()
        .remove_resource::<Pegas>()
        .unwrap_or_default();
    let doc = &tl.doc;
    if !(painel && tl.flags.auto_key) || !(doc.fps_display > 0.0) {
        return Vec::new();
    }
    let (f, solo) = (inst.t_cut, inst.solo);
    let antes = f - 1.0 / doc.fps_display;
    let mut out = Vec::new();
    for g in pegas.0 {
        let p = PropKind::Field(chave(g.campo, g.item));
        let v = visto(doc, g.dono, p, antes, solo, g.antes);
        out.extend(chave_em(doc, g.dono, p, antes, solo, v, Interp::Hold));
        out.extend(chave_em(doc, g.dono, p, f, solo, g.depois, interp));
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
            out.extend(chave_em(doc, g.dono, p, antes, solo, v, Interp::Hold));
            out.extend(chave_em(
                doc,
                g.dono,
                p,
                f,
                solo,
                da_pose(&vista, p),
                interp,
            ));
        }
    }
    out
}
