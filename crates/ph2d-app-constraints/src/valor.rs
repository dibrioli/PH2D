//! **Valor segue valor, lado da família** (`docs/BlenderConstraints/06_W5_resto.md` §3.1). Corre logo
//! depois do apply da timeline e ANTES do dreno das famílias: lê a fonte como a curva a deixou, corre a
//! mola, e escreve o destino — um campo vai para `CanaisEscritos` (o dreno da família dona aplica-o em
//! pré-visualização), a pose fica em [`Valores`] e a pilha aplica-a na posição do item.

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Kind, Prop, Valor};
use ph2d_constraints_ecs::{Constraints, Lei, Valores, ordem, pose};
use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld, StableId, World, world_transform};

use crate::relogio::Relogio;

/// O endereço da casa de um campo do [`Prop`].
#[must_use]
pub fn chave(c: [u16; 4]) -> FieldKey {
    FieldKey::new(c[0], c[1], c[2], c[3])
}

/// `prop` de `e` neste quadro: um campo como a curva o mandou (senão o publicado), a pose do mundo.
fn agora(w: &World, e: Entity, prop: Prop) -> Option<f64> {
    match prop {
        Prop::Pose(c) => world_transform(w, e).map(|t| c.of(pose(&t))),
        Prop::Campo(c) => {
            let (b, k) = (e.to_bits(), chave(c));
            w.get_resource::<CanaisEscritos>()
                .and_then(|x| x.valor(b, k))
                .or_else(|| w.get_resource::<CanaisLidos>()?.valor(b, k))
                .map(f64::from)
        }
    }
}

/// A fonte de `b` no instante `t` pela curva (`None` = não é animada: vale a história). A pose só
/// conta como animada com uma curva de pose: sem ela o `world_pose_at` devolve a pose VIVA.
fn pela_curva(w: &World, r: &Relogio<'_>, v: &Valor, b: u64, t: f64) -> Option<f64> {
    match v.fonte {
        Prop::Pose(_) => r
            .doc
            .bindings()
            .iter()
            .any(|x| x.entity == b && !x.missing && x.prop.as_sprite_transform().is_some())
            .then(|| ph2d_timeline::world_pose_at(w, r.doc, b, t))
            .flatten()
            .and_then(|p| v.da_pose(pose(&p))),
        Prop::Campo(c) => ph2d_timeline::field_at(w, r.doc, b, chave(c), t).map(f64::from),
    }
}

/// ⭐ **A porta do quadro.** Para cada Valor ligado, pela ordem da pilha: a saída (já na faixa do
/// destino, com a mola) em [`Valores`], e o campo de destino em `CanaisEscritos`. Sem relógio, ou sem
/// mola, segue no mesmo quadro.
pub fn valores(sim: &mut SimWorld, relogio: Option<Relogio<'_>>) {
    let w = sim.world_mut();
    let mut vs = w.remove_resource::<Valores>().unwrap_or_default();
    vs.saidas.clear();
    let itens: Vec<_> = ordem(w)
        .0
        .into_iter()
        .filter_map(|e| Some((e, w.get::<Constraints>(e)?)))
        .flat_map(|(e, c)| {
            c.items
                .iter()
                .filter(|it| it.enabled)
                .filter_map(move |it| match it.kind {
                    Kind::Valor(v) => Some((e, it.id, v, it.target, it.influence)),
                    _ => None,
                })
        })
        .collect();
    let vivos: BTreeSet<(u64, u32)> = itens.iter().map(|i| (i.0.to_bits(), i.1)).collect();
    vs.sims.retain(|k, _| vivos.contains(k));
    vs.historia.retain(|k, _| vivos.contains(k));
    let alvos: BTreeSet<StableId> = itens.iter().filter_map(|i| i.3).collect();
    let idx: BTreeMap<StableId, Entity> = if alvos.is_empty() {
        BTreeMap::new()
    } else {
        let mut q = w.query::<(Entity, &StableId)>();
        q.iter(w)
            .filter(|(_, s)| alvos.contains(s))
            .map(|(e, s)| (*s, e))
            .collect()
    };
    for (e, id, v, target, influence) in itens {
        let chave_item = (e.to_bits(), id);
        {
            let Some(&alvo) = target.and_then(|s| idx.get(&s)) else {
                continue;
            };
            let fonte = match v.fonte {
                Prop::Pose(_) => world_transform(w, alvo).and_then(|t| v.da_pose(pose(&t))),
                p => agora(w, alvo, p),
            };
            let (Some(fonte), Some(destino)) = (fonte, agora(w, e, v.destino)) else {
                continue;
            };
            let mut out = v.mapeia(fonte);
            if let (Some(mola), Some(r)) = (v.mola, relogio.as_ref())
                && r.doc.fps_display > 0.0
            {
                let fps = r.doc.fps_display;
                let q = r.t * fps;
                let b = alvo.to_bits();
                let animada = pela_curva(w, r, &v, b, r.t).is_some();
                let historia = vs.historia.entry(chave_item).or_default();
                if animada {
                    historia.clear();
                } else {
                    #[expect(clippy::cast_possible_truncation, reason = "um quadro cabe em i64")]
                    let f = q.floor() as i64;
                    let _ = historia.split_off(&(f + 1));
                    if historia.range(..=f).next_back().map(|(_, x)| *x) != Some(fonte) {
                        historia.insert(f, fonte);
                    }
                }
                let historia = &vs.historia[&chave_item];
                let mut entrada = |k: i64| {
                    #[expect(clippy::cast_precision_loss, reason = "um quadro")]
                    let t = k as f64 / fps;
                    let x = if animada {
                        pela_curva(w, r, &v, b, t).unwrap_or(fonte)
                    } else {
                        historia
                            .range(..=k)
                            .next_back()
                            .or_else(|| historia.iter().next())
                            .map_or(fonte, |(_, x)| *x)
                    };
                    [v.mapeia(x)]
                };
                let lei = Lei {
                    duracao: mola.duracao,
                    ressalto: mola.ressalto,
                    angular: [false],
                };
                let sim = vs.sims.entry(chave_item).or_default();
                out += sim.atraso_no_laco(q, r.laco, lei, fps, &mut entrada)[0];
            }
            vs.saidas.insert(chave_item, out);
            // Escrito já: um Valor mais abaixo na ordem que lê este campo vê-o neste quadro.
            if let Prop::Campo(k) = v.destino {
                #[expect(clippy::cast_possible_truncation, reason = "o canal da casa é f32")]
                let x = v.compoe(destino, out, influence) as f32;
                w.get_resource_or_insert_with(CanaisEscritos::default)
                    .escrever(e.to_bits(), chave(k), x);
            }
        }
    }
    w.insert_resource(vs);
}
