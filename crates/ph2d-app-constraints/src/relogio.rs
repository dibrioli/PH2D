//! Os tipos que leem o TEMPO, do lado da família (W3, `docs/BlenderConstraints/03_W3_poderosas.md`
//! §2): o clip de cada Action amostrado sem escrever o mundo, e a mola de cada Spring re-simulada
//! sobre a pose ANIMADA (a «cena no quadro» do `SceneAtTick` da física é a timeline).

use std::sync::Arc;

use bevy_ecs::entity::Entity;
use ph2d_anim::{AnimValue, AttributeEvaluator, Track};
use ph2d_constraints::Kind;
use ph2d_constraints_ecs::{Accoes, Amostra, Constraints, Laco, Molas};
use ph2d_core::Playhead;
use ph2d_ecs::SimWorld;
use ph2d_timeline::TimelineDoc;

/// O relógio do quadro: o documento, o instante da vista (segundos) e o laço do transporte (em
/// quadros, com o sentido de agora no vaivém).
#[derive(Clone, Copy)]
pub struct Relogio<'a> {
    pub doc: &'a TimelineDoc,
    pub t: f64,
    pub laco: Option<Laco>,
}

impl<'a> Relogio<'a> {
    /// O relógio da VISTA (a cena, o clip da Keys ou o contêiner — `ViewClock`): `relogios` são os
    /// três transportes, por essa ordem.
    #[must_use]
    pub fn da_vista(tl: &'a ph2d_timeline::TimelineState, relogios: [&Playhead; 3]) -> Self {
        let [e, c, k] = relogios;
        let ph = tl.view_clock().pick(e, c, k);
        let fps = tl.doc.fps_display;
        Self {
            doc: &tl.doc,
            t: ph.time(),
            laco: ph.loop_range().map(|(de, ate)| Laco {
                de: de * fps,
                ate: ate * fps,
                vaivem: ph.is_ping_pong(),
                para_a_frente: ph.is_advancing_forward(),
            }),
        }
    }
}

/// Os itens `(dono, item, tipo)` da cena.
fn itens(sim: &SimWorld) -> Vec<(Entity, u32, Kind)> {
    let w = sim.world();
    w.iter_entities()
        .filter_map(|e| Some((e.id(), e.get::<Constraints>()?)))
        .flat_map(|(e, c)| c.items.iter().map(move |i| (e, i.id, i.kind)))
        .collect()
}

/// O clip `posicao` da animação do dono amostrado nos canais da pose — `None` sem esse clip.
fn amostra(doc: &TimelineDoc, dono: u64, posicao: u32) -> Option<Amostra> {
    let clip = doc.clips_of(dono).nth(usize::try_from(posicao).ok()?)?;
    let named = doc.clips().get(clip)?;
    let faixas: Vec<(usize, Track)> = doc
        .bindings()
        .iter()
        .filter(|b| b.entity == dono && !b.missing)
        .filter_map(|b| {
            let canal = b.prop.as_sprite_transform()? as usize;
            Some((canal, named.clip.track(b.target)?.clone()))
        })
        .collect();
    Some(Arc::new(move |t| {
        let mut out = [None; 5];
        for (canal, faixa) in &faixas {
            if let AnimValue::Float(v) = faixa.sample(t) {
                out[*canal] = Some(f64::from(v));
            }
        }
        out
    }))
}

/// O relógio de PAREDE (s) que a mola ao vivo lê; sem ele, o monotónico do processo. Os gates põem-no.
#[derive(bevy_ecs::prelude::Resource, Clone, Copy, Debug)]
pub struct Parede(pub f64);

fn agora(w: &bevy_ecs::world::World) -> f64 {
    static ZERO: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    w.get_resource::<Parede>()
        .map_or_else(|| ZERO.get_or_init(std::time::Instant::now).elapsed().as_secs_f64(), |p| p.0)
}

/// A pose MOSTRADA do dono (antes da pilha): a do gizmo durante um arrasto, não a da animação.
fn mostrada(w: &bevy_ecs::world::World, e: Entity) -> Option<[f64; 3]> {
    let t = ph2d_ecs::world_transform(w, e)?;
    Some([t.translation.x, t.translation.y, t.rotation].map(f64::from))
}

fn soma(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|c| a[c] + b[c])
}

/// ⭐ A mola com o tempo parado: nasce quando a pose mostrada se mexe (no sítio onde o ecrã a tinha),
/// persegue `mostrada + atraso puro` e larga-se ao assentar. Devolve o atraso a somar à pose.
fn ao_vivo(
    molas: &mut Molas,
    k: (u64, u32),
    mostrada: Option<[f64; 3]>,
    (puro, dt): ([f64; 3], f64),
    lei: ph2d_constraints_ecs::Lei<3>,
) -> [f64; 3] {
    let Some(m) = mostrada else {
        molas.vivas.remove(&k);
        return puro;
    };
    if let Some(&(antes, ecra)) = molas.vistos.get(&k)
        && antes != m
    {
        molas
            .vivas
            .entry(k)
            .or_insert_with(|| ph2d_constraints_ecs::relogio::AoVivo::nasce(ecra));
    }
    let alvo = soma(m, puro);
    let viva = molas.vivas.get_mut(&k).map(|v| (v.anda(lei, alvo, dt), v.posicao()));
    let atraso = match viva {
        Some((false, p)) => [p[0] - m[0], p[1] - m[1], ph2d_constraints::wrap_pi(p[2] - m[2])],
        Some((true, _)) => {
            molas.vivas.remove(&k);
            puro
        }
        None => puro,
    };
    molas.vistos.insert(k, (m, soma(m, atraso)));
    atraso
}

/// Instala o clip de cada Action ([`Accoes`]) e o atraso de cada Spring ([`Molas`]) deste quadro.
pub(crate) fn preparar(sim: &mut SimWorld, relogio: Relogio<'_>) {
    let itens = itens(sim);
    let mut accoes = Accoes::default();
    for &(e, _, k) in &itens {
        if let Kind::Action(a) = k
            && let Some(f) = amostra(relogio.doc, e.to_bits(), a.clip)
        {
            accoes.0.insert((e.to_bits(), a.clip), f);
        }
    }
    let w = sim.world_mut();
    w.insert_resource(accoes);
    let mut molas = w.remove_resource::<Molas>().unwrap_or_default();
    molas.atrasos.clear();
    let fps = relogio.doc.fps_display;
    let vivas: std::collections::BTreeSet<(u64, u32)> = itens
        .iter()
        .filter(|(_, _, k)| matches!(k, Kind::Spring(_)))
        .map(|&(e, id, _)| (e.to_bits(), id))
        .collect();
    molas.sims.retain(|k, _| vivas.contains(k));
    molas.vivas.retain(|k, _| vivas.contains(k));
    molas.vistos.retain(|k, _| vivas.contains(k));
    // ⭐ O tempo PARADO (o instante da vista igual ao do quadro anterior) liga a mola ao vivo; tocar ou
    // arrastar o tempo larga-a e o caminho puro `(doc, t)` manda sozinho.
    let parede = agora(w);
    let antes = molas.parede.replace((relogio.t.to_bits(), parede));
    let dt = match antes {
        Some((t, p)) if t == relogio.t.to_bits() => Some(parede - p),
        _ => {
            molas.vivas.clear();
            None
        }
    };
    if fps > 0.0 {
        for &(e, id, k) in &itens {
            let Kind::Spring(mola) = k else { continue };
            let dono = e.to_bits();
            let mut entrada = |q: i64| {
                #[expect(clippy::cast_precision_loss, reason = "um quadro")]
                let t = q as f64 / fps;
                ph2d_timeline::world_pose_at(w, relogio.doc, dono, t).map_or([0.0; 3], |p| {
                    [
                        f64::from(p.translation.x),
                        f64::from(p.translation.y),
                        f64::from(p.rotation),
                    ]
                })
            };
            let sim = molas.sims.entry((dono, id)).or_default();
            let q = relogio.t * fps;
            let mut atraso = sim.atraso_no_laco(q, relogio.laco, &mola, fps, &mut entrada);
            if let Some(dt) = dt {
                atraso = ao_vivo(&mut molas, (dono, id), mostrada(w, e), (atraso, dt), (&mola).into());
            } else if let Some(m) = mostrada(w, e) {
                molas.vistos.insert((dono, id), (m, soma(m, atraso)));
            }
            molas.atrasos.insert((dono, id), atraso);
        }
    }
    w.insert_resource(molas);
    crate::comportamento::preparar(sim, relogio);
}
