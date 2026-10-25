//! **Cozer um intervalo** (plano §0.6, §2.2): a pose que a pilha INTEIRA dá ao dono em cada quadro de
//! `[de, ate]` vira chaves do `Transform`, e a força de todo item fica 0 no intervalo — dentro dele a
//! curva faz o que a pilha fazia, ao bit. Coze-se a pilha inteira: cozer um item só e deixar os
//! outros activos aplicá-los-ia duas vezes.
//!
//! Fora do intervalo nada muda: a força volta em `ate+1` ao valor que tinha, e o `Transform` ganha em
//! `de−1` e `ate+1` a pose autorada de antes (degraus), porque um tipo que parte da pose autorada
//! (Limit, Copy a meia força) daria outra coisa sobre a pose cozida.
//!
//! A avaliação corre no mundo vivo (curva → canais → pilha, como o quadro da shell) e devolve-o como
//! estava: o `Transform` de toda entidade, as pilhas e os canais por drenar.

use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Resource;
use ph2d_anim::KeyId;
use ph2d_constraints_ecs::Constraints;
use ph2d_ecs::canal_constraint::{FORCA, chave};
use ph2d_ecs::{CanaisEscritos, SimWorld, Transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::{Interp, PropKind, TimelineDoc, TimelineState};

use crate::chaves::Escrita;
use crate::compensa::{POSE, da_pose};

/// O mundo que a avaliação pisa, para o devolver.
struct Guardado {
    poses: Vec<(Entity, Transform)>,
    pilhas: Vec<(Entity, Constraints)>,
    canais: Option<CanaisEscritos>,
}

fn guardar(sim: &mut SimWorld) -> Guardado {
    let w = sim.world_mut();
    let poses = w
        .query::<(Entity, &Transform)>()
        .iter(w)
        .map(|(e, t)| (e, *t))
        .collect();
    let pilhas = w
        .query::<(Entity, &Constraints)>()
        .iter(w)
        .map(|(e, c)| (e, c.clone()))
        .collect();
    let canais = w.get_resource::<CanaisEscritos>().cloned();
    Guardado {
        poses,
        pilhas,
        canais,
    }
}

fn devolver(sim: &mut SimWorld, g: Guardado) {
    let w = sim.world_mut();
    for (e, t) in g.poses {
        if let Some(mut x) = w.get_mut::<Transform>(e) {
            *x = t;
        }
    }
    for (e, c) in g.pilhas {
        if let Some(mut x) = w.get_mut::<Constraints>(e) {
            *x = c;
        }
    }
    match g.canais {
        Some(c) => w.insert_resource(c),
        None => {
            w.remove_resource::<CanaisEscritos>();
        }
    }
}

/// Os donos cujo bake o cartão pediu (o botão «Bake»), à espera da timeline.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct Cozer(pub Vec<u64>);

/// Drena os pedidos.
pub fn pedidos(sim: &mut SimWorld) -> Vec<u64> {
    sim.world_mut()
        .remove_resource::<Cozer>()
        .map(|c| c.0)
        .unwrap_or_default()
}

fn segundos(doc: &TimelineDoc, q: i64) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "quadros de uma timeline")]
    let q = q as f64;
    q / doc.fps_display
}

/// As chaves de `p` em `e` com o tempo estritamente dentro de `(de, ate)` segundos.
fn por_dentro(doc: &TimelineDoc, e: u64, p: PropKind, de: f64, ate: f64) -> Vec<(KeyId, f64)> {
    let Some(track) = doc
        .binding_for(e, p)
        .and_then(|b| doc.active_clip().track(b.target))
    else {
        return Vec::new();
    };
    track
        .keys()
        .iter()
        .zip(track.ids())
        .map(|(k, &id)| (id, k.t.to_seconds()))
        .filter(|&(_, t)| t > de && t < ate)
        .collect()
}

/// A pose local do dono depois da pilha, no tempo `t` da cena: o quadro da shell (curva → canais →
/// pilha) sobre um ledger de rascunho.
fn pose_em(
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    doc: &mut TimelineDoc,
    dono: Entity,
    t: f64,
) -> Option<Transform> {
    ph2d_timeline::apply_from_doc(sim.world_mut(), doc, t);
    crate::canais::aplicar(sim, drive);
    ph2d_constraints_ecs::evaluate(sim, drive);
    drive.settle();
    sim.world().get::<Transform>(dono).copied()
}

/// A força de cada item e a pose autorada (sem a pilha) no tempo `t`.
fn sem_pilha(
    sim: &mut SimWorld,
    doc: &mut TimelineDoc,
    dono: Entity,
    itens: &[ph2d_constraints_ecs::Item],
    t: f64,
) -> (Vec<f32>, Option<Transform>) {
    ph2d_timeline::apply_from_doc(sim.world_mut(), doc, t);
    let forcas = itens
        .iter()
        .map(|it| {
            let k = PropKind::Field(chave(FORCA, it.id));
            #[expect(clippy::cast_possible_truncation, reason = "a força é 0..=1")]
            let solta = it.influence as f32;
            ph2d_timeline::curve_seen(doc, dono.to_bits(), k, t, false).unwrap_or(solta)
        })
        .collect();
    (forcas, sim.world().get::<Transform>(dono).copied())
}

/// ⭐ **As escritas do bake** de `dono` sobre `[t0, t1]` segundos da cena (os quadros inteiros lá
/// dentro, mais as chaves da pose que já lá estão). Vazio se o dono não tem pilha ou o intervalo não
/// tem quadros. O mundo volta como estava.
#[must_use]
pub fn bake(
    sim: &mut SimWorld,
    tl: &TimelineState,
    preview: &PreviewDrive,
    dono: u64,
    t0: f64,
    t1: f64,
) -> Vec<Escrita> {
    let fps = tl.doc.fps_display;
    let Some(e) = Entity::try_from_bits(dono) else {
        return Vec::new();
    };
    let Some(itens) = sim.world().get::<Constraints>(e).map(|c| c.items.clone()) else {
        return Vec::new();
    };
    if !(fps > 0.0) || !(t1 >= t0) {
        return Vec::new();
    }
    #[expect(clippy::cast_possible_truncation, reason = "quadros de uma timeline")]
    // A folga: um loop num quadro exacto (30/24 s) volta de `t·fps` a 30,000…04 e o `ceil` saltaria.
    let (de, ate) = (
        (t0 * fps - 1e-9).ceil() as i64,
        (t1 * fps + 1e-9).floor() as i64,
    );
    if ate < de {
        return Vec::new();
    }
    let mut doc = tl.doc.clone();
    let (t0, t1) = (segundos(&doc, de), segundos(&doc, ate));
    let guardado = guardar(sim);
    // Parte-se da pose AUTORADA: a viva de quem a pilha conduz já leva a pilha.
    let _ = preview.substitute_authored(sim);
    let mut bordas = Vec::new();
    for q in [de - 1, ate + 1].into_iter().filter(|&q| q >= 0) {
        let t = segundos(&doc, q);
        bordas.push((q < de, t, sem_pilha(sim, &mut doc, e, &itens, t)));
    }
    let mut tempos: Vec<f64> = (de..=ate).map(|q| segundos(&doc, q)).collect();
    for p in POSE {
        tempos.extend(
            por_dentro(&doc, dono, p, t0, t1)
                .into_iter()
                .map(|(_, t)| t),
        );
    }
    tempos.sort_by(f64::total_cmp);
    tempos.dedup();
    let mut out = Vec::new();
    // UM ledger para o laço inteiro: um novo por quadro partiria da pose que o anterior escreveu.
    let mut drive = PreviewDrive::default();
    for &t in &tempos {
        if let Some(pose) = pose_em(sim, &mut drive, &mut doc, e, t) {
            for p in POSE {
                let v = da_pose(&pose, p);
                out.push(Escrita::Chave {
                    e: dono,
                    p,
                    t,
                    v,
                    interp: Interp::Linear,
                });
            }
        }
    }
    devolver(sim, guardado);
    // A borda: a força de antes volta, e a pose autorada (degraus) — fora do intervalo nada muda.
    for (antes, t, (forcas, autorada)) in bordas {
        for (it, v) in itens.iter().zip(forcas) {
            let p = PropKind::Field(chave(FORCA, it.id));
            let interp = if antes { Interp::Hold } else { Interp::Linear };
            out.push(Escrita::Chave {
                e: dono,
                p,
                t,
                v,
                interp,
            });
        }
        for p in POSE.iter().filter(|_| autorada.is_some()) {
            let v = autorada.map_or(0.0, |a| da_pose(&a, *p));
            out.push(Escrita::Chave {
                e: dono,
                p: *p,
                t,
                v,
                interp: Interp::Hold,
            });
        }
    }
    // O último quadro cozido segura até à borda (sem rampa para a pose autorada).
    for p in POSE {
        if let Some(Escrita::Chave { interp, .. }) = out
            .iter_mut()
            .rev()
            .find(|w| matches!(w, Escrita::Chave { p: q, t, .. } if *q == p && *t == t1))
        {
            *interp = Interp::Hold;
        }
    }
    for it in &itens {
        let p = PropKind::Field(chave(FORCA, it.id));
        for t in [t0, t1] {
            out.push(Escrita::Chave {
                e: dono,
                p,
                t,
                v: 0.0,
                interp: Interp::Hold,
            });
        }
        if let Some(b) = doc.binding_for(dono, p) {
            for (key, _) in por_dentro(&doc, dono, p, t0, t1) {
                out.push(Escrita::Valor {
                    target: b.target,
                    key,
                    v: 0.0,
                });
            }
        }
    }
    out
}
