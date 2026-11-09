//! Auto-key pass (W4.T1/T2) — the shell half of "the user moved something, key
//! it at the playhead".
//!
//! Runs ONCE per frame, AFTER every UI-driven Transform/opacity write for the
//! frame (the gizmo drag early, the Inspector + Hierarchy commits late) and after
//! the timeline apply. That placement is what makes it a genuine single choke
//! point despite the scattered write sites: it reads the *resulting* pose of each
//! selected sprite and asks [`ph2d_timeline::autokey_props`] which properties left
//! their curve. Because the apply pass has already written the document's value to
//! the world, an undo / paste / dope-sheet move leaves world == curve and keys
//! nothing — no feedback loop.
//!
//! Undo grouping mirrors the pre-existing gizmo bracket: a gizmo drag is one step
//! across the whole gesture; a discrete edit (Inspector field, Reset) is one step
//! for the frame it lands on.
//!
//! While the transport is **playing** the pass is inert: the pose is the animation, not a user edit.
//! The one exception is performing (W5): with `Record` armed, a live gizmo drag records along the
//! playhead (`performing && drag_now`). Disarmed, the pass pins instead of keying: a bound pose the
//! user displaced while paused joins [`AutokeyState::displaced`] so a manual K can record it, until
//! the playhead moves or it returns to its curve.

use std::collections::BTreeMap;

use ph2d_anim::{AnimValue, RationalTime};
use ph2d_core::Playhead;
use ph2d_editor_core::HeroScreen;
use ph2d_timeline::{PoseSample, PropKind, TimelineState, autokey_props};

#[path = "autokey_state.rs"]
mod autokey_state;
pub(crate) use autokey_state::AutokeyState;
#[path = "autokey_amostra.rs"]
mod autokey_amostra;
pub(crate) use autokey_amostra::populacao;
use autokey_amostra::sample_pose;
pub(crate) use ph2d_ecs::canal::campos_lidos;

use super::timeline_bridge::default_interp;
use ph2d_timeline::record_fit::{REC_SIMPLIFY_REL, REC_SMOOTH_PASSES, RecSpan, simplify_recorded};

/// Auto-key the current selection. `armed` is `panel_open && auto_key`, resolved
/// by the caller; the pass additionally goes inert while the transport is playing
/// (`playhead.is_playing()`). Rebuilds the baseline from the live selection every
/// frame (so it tracks selection changes), brackets the undo step, and manages
/// the displaced-pose pin (see the module docs).
// Os factos do quadro, cada um load-bearing — a MESMA nota do `apply_samples` abaixo e do
// `timeline_bridge::run`: agrupá-los numa struct esconde quais o chamador tem de fornecer por
// quadro. O 8.º é a SEGUNDA mão (a ferramenta Bone), que este passe não conhecia.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    timeline: &mut TimelineState,
    playhead: &Playhead,
    ak: &mut AutokeyState,
    toasts: &mut ph2d_editor_core::ToastQueue,
    hero: &HeroScreen,
    // ⭐ **O mundo chega como `SimWorld` e não como `&World`** porque a população deste passe deixou
    // de ser só a selecção: quem a MÃO segura sai da [`super::timeline_bridge::maos_do_quadro`], e
    // ela precisa de perguntar pela cadeia de ossos.
    sim: &ph2d_ecs::SimWorld,
    // A opacidade GRAVADA de um vetor que nenhuma curva conduz (plano 34 D4).
    vec: &ph2d_vec_scene::VecScene,
    preview: &ph2d_preview_drive::PreviewDrive,
    skeleton: &ph2d_app_skeleton::state::SkeletonState,
) {
    let panel_open = hero.is_panel_visible("timeline");
    let armed = panel_open && timeline.flags.auto_key;
    // Performing needs the panel open too (same gate as auto-key): record is a
    // timeline authoring mode, meaningless when its UI is hidden.
    let performing = panel_open && timeline.flags.performing;
    // Duas mãos arrastam: o gizmo e a ferramenta Bone; sem esta, cada quadro de um osso era um passo de undo.
    let drag_now = hero.gizmo.drag.is_some()
        || skeleton.bone_pose.is_some()
        || sim
            .world()
            .get_resource::<ph2d_ecs::MaosDasFamilias>()
            .is_some_and(ph2d_ecs::MaosDasFamilias::alguma);
    // Sample every selected sprite's live pose, in selection order.
    //
    // ⛔ Menos quem um MOTOR conduz (`populacao`): pré-visualização não é autoria.
    let samples: Vec<(u64, PoseSample)> = populacao(hero, skeleton, sim, preview)
        .into_iter()
        .map(|e| {
            let src = super::timeline_bridge::Mundo::com_vetor(sim.world(), vec);
            (e, sample_pose(src, e))
        })
        .collect();
    ak.campos = samples
        .iter()
        .map(|&(e, _)| (e, campos_lidos(sim, e)))
        .collect();
    ak.previa = ph2d_app_sprite_anim::previa::marcados(sim.world());
    propagar_topologia(timeline, ak);
    sonda(|| {
        let pop: Vec<u64> = samples.iter().map(|s| s.0).collect();
        format!("[autokey] painel={panel_open} armado={armed} populacao={pop:x?}")
    });
    apply_samples(
        timeline, playhead, &samples, drag_now, armed, performing, ak, toasts,
    );
}

/// ⭐ Um ponto acrescentado/apagado numa forma animada vale em TODAS as chaves (plano 35, decisão
/// do dono) — antes do diff, e num passo de undo da timeline próprio (ou no do gesto aberto).
pub(crate) fn propagar_topologia(timeline: &mut TimelineState, ak: &AutokeyState) {
    for (&e, agora) in &ak.campos {
        let Some(antes) = ak.base_campos.get(&e) else {
            continue;
        };
        let pontos = ph2d_physics_outline::canais_pontos::pede_topologia(antes, agora);
        if !pontos && !ph2d_app_vec::canais_topologia::mudou(antes, agora) {
            continue;
        }
        // Um gesto já aberto (um arrasto) leva a propagação no passo dele.
        let proprio = !timeline.history.is_open();
        if proprio {
            timeline.history.begin(&timeline.doc);
        }
        ph2d_physics_outline::canais_pontos::propagar(&mut timeline.doc, e, antes, agora);
        ph2d_app_vec::canais_topologia::propagar(&mut timeline.doc, e, antes, agora);
        if proprio {
            let doc = timeline.doc.clone();
            timeline.history.commit_da_cena(&doc);
        }
    }
}

/// O laço de [`apply_samples`]: cada pose amostrada contra a sua curva (ou o quadro anterior),
/// com o pino da pose deslocada; devolve a primeira recusa do quadro.
#[allow(clippy::too_many_arguments)]
fn diff_each_sample(
    timeline: &TimelineState,
    samples: &[(u64, PoseSample)],
    ak: &mut AutokeyState,
    to_key: &mut Vec<(u64, PropKind, f32, RationalTime)>,
    to_path: &mut Vec<(u64, [f32; 2], RationalTime)>,
    next_baseline: &mut BTreeMap<u64, PoseSample>,
    t_cut: f64,
    t: RationalTime,
    solo: bool,
    playing: bool,
    capturing: bool,
    default_path: bool,
) -> Option<ph2d_timeline::KeyRefusal> {
    // The first refusal this frame, if any (they share a cause far more often than
    // not — one stack, one playhead).
    let mut refused_now: Option<ph2d_timeline::KeyRefusal> = None;
    for &(entity, pose) in samples {
        // The entity's own clock: under a Time Remap the key lands at the source time the apply sampled.
        let t_src = ph2d_timeline::remapped_time(&timeline.doc, entity, t_cut);
        // Where a key lands — the active clip's OWN time; a clip playing twice or not at
        // all has no answer (ADR-0115 R9), and a key with no home is not written.
        //
        // Performing records in real time (the simplify places clean keys afterwards); paused authoring
        // snaps to whole frames. `key_home` carries the REASON of a refusal.
        let home = if solo {
            Ok(t_src)
        } else {
            ph2d_timeline::key_home(&timeline.doc, entity, t_cut)
        };
        let t_e = home.ok().map(|ts| {
            if playing || ts != t_cut {
                RationalTime::from_seconds(ts)
            } else {
                t
            }
        });
        // The diff's clock is the RAW `t_src`, the exact `f64` the apply sampled: compare where you read.
        let t_diff = t_src;
        let base = ak
            .baseline
            .get(&entity)
            .copied()
            .unwrap_or([None; PropKind::AUTOKEYED.len()]);
        // Os campos de família (plano 35): a mesma diferença, ao lado da pose; o ▶ é só prévia: não grava.
        let todos = ak.campos.get(&entity).cloned().unwrap_or_default();
        let em_previa = ak.previa.contains(&entity);
        let campos: Vec<_> = todos
            .iter()
            .copied()
            .filter(|(k, _)| !(em_previa && ph2d_app_sprite_anim::previa::e_campo_da_previa(*k)))
            .collect();
        let base_c = ak.base_campos.get(&entity).cloned().unwrap_or_default();
        let doc = &timeline.doc;
        let campos_fora = |criar| {
            ph2d_timeline::autokey_campos(doc, entity, t_diff, &campos, &base_c, criar, solo)
        };
        if capturing {
            // Performing records only what the DRAG pushed off the curve.
            let plan = if solo {
                ph2d_timeline::autokey_props_solo(
                    &timeline.doc,
                    entity,
                    t_diff,
                    &pose,
                    &base,
                    true,
                    default_path,
                )
            } else {
                autokey_props(
                    &timeline.doc,
                    entity,
                    t_diff,
                    &pose,
                    &base,
                    true,
                    default_path,
                )
            };
            let mut plan = plan;
            let pc = campos_fora(true);
            plan.keys.extend(pc.keys);
            plan.refused.extend(pc.refused);
            // A refusal is a result: no key is written and the animator is owed a reason (ADR-0115 R9).
            if !plan.refused.is_empty() {
                refused_now = Some(home.err().unwrap_or(ph2d_timeline::KeyRefusal::Overridden));
            }
            // A âncora de trajetória recusada só fala se nada mais falou (o motivo estrutural informa mais).
            refused_now = refused_now.or(plan.path_refused).or(plan.scene_refused);
            if let Some(t_e) = t_e {
                // The motion-path anchor (Path mode), at the same home time as the scalar keys.
                if let Some(at) = plan.path_key {
                    to_path.push((entity, at, t_e));
                }
                for (prop, v) in plan.keys {
                    to_key.push((entity, prop, v, t_e));
                    // Track the recorded span so the drag's end can simplify
                    // exactly what it recorded (playing = a performing session,
                    // not a paused one-off edit).
                    if playing {
                        let (ts, vf) = (t_e.to_seconds(), f64::from(v));
                        ak.record
                            .entry((entity, prop))
                            .and_modify(|s| s.extend(ts, vf))
                            .or_insert_with(|| RecSpan::seed(ts, vf));
                    }
                }
            }
        }
        // Displaced-pose pin (paused only): disarmed and off its curve = posed for a manual K.
        if !playing {
            // `allow_create = false`: only BOUND props are overwritten by the apply.
            let off_curve = !(if solo {
                ph2d_timeline::autokey_props_solo(
                    &timeline.doc,
                    entity,
                    t_diff,
                    &pose,
                    &base,
                    false,
                    default_path,
                )
            } else {
                autokey_props(
                    &timeline.doc,
                    entity,
                    t_diff,
                    &pose,
                    &base,
                    false,
                    default_path,
                )
            })
            .is_empty()
                || !campos_fora(false).is_empty();
            // Sobre uma curva a prévia segura-se como uma pose posada à mão, armado ou não.
            let previa_fora = em_previa
                && !ph2d_timeline::autokey_campos(
                    doc, entity, t_diff, &todos, &base_c, false, solo,
                )
                .is_empty();
            if (!capturing && off_curve) || previa_fora {
                ak.displaced.insert(entity);
            } else {
                ak.displaced.remove(&entity);
            }
        }
        sonda(|| {
            format!(
                "[autokey] e={entity:#x} capturando={capturing} tocando={playing} solo={solo} t={t_diff:.3} casa={:?} pose={pose:?} fila={}",
                home.as_ref().ok(),
                to_key.len()
            )
        });
        next_baseline.insert(entity, pose);
    }
    ak.base_campos = ak
        .campos
        .iter()
        .map(|(&e, v)| (e, v.iter().copied().collect()))
        .collect();
    refused_now
}

/// The pure core of the pass: given each selected sprite's sampled pose, key what
/// left its curve and bracket the undo step. Separated from [`run`] (which owns
/// the `HeroScreen`/`World` sampling) so the frame logic — the diff, the
/// auto-create, the bracket — is testable headless.
#[allow(clippy::too_many_arguments)] // the frame's inputs; bundling them hides them
pub(crate) fn apply_samples(
    timeline: &mut TimelineState,
    playhead: &Playhead,
    samples: &[(u64, PoseSample)],
    drag_now: bool,
    armed: bool,
    performing: bool,
    ak: &mut AutokeyState,
    toasts: &mut ph2d_editor_core::ToastQueue,
) {
    if std::mem::take(&mut ak.restaurado) {
        return;
    }
    // Which scene the diff believes in: in the KEYS view, the active clip at the clip playhead.
    let solo = timeline.keys_mode;
    // Inside a container: the container's blend, at its clock (2026-07-22).
    let root = timeline.container_open;
    // The stack's scratch must describe THIS instant; solo never consults it.
    if !solo {
        timeline.doc.prime_rooted(root, playhead.time());
    }

    // Whether this frame CAPTURES the pose: paused, `armed`; playing, only a live drag while performing.
    // The baseline advances regardless, so pausing mid-play is not a jump.
    let playing = playhead.is_playing();
    // ⛔ E o RELÓGIO A ANDAR SEM TOCAR é o mesmo caso (report do dono, 17/09): ver
    // [`AutokeyState::clock_t`].
    let clock_moved = ak.clock_t.is_some_and(|t0| t0 != playhead.time());
    // ⛔ **E a MÃO NA RÉGUA é o gesto em si** — ver [`AutokeyState::scrub_now`]. Com o Snap ligado
    // o `clock_moved` é falso no meio de um arrasto, e sem este guarda o passe voltava a capturar.
    let capturing = if playing {
        performing && drag_now
    } else {
        armed && !clock_moved && !ak.scrub_now
    };
    ak.clock_t = Some(playhead.time());
    let fps = timeline.doc.fps_display;
    // **The author's clock is the apply's clock, CUT included** (seed == sample): past the
    // cut the pose is FROZEN at `curve(cut)`, and a diff reading `curve(raw)` sees a phantom
    // delta, a key per scrubbed frame (2026-07-23). This cut is the SOLE boundary: the playhead
    // runs free past the authored end. Which cut mirrors which apply:
    //  - Keys/solo, and Arrange with an EMPTY stack: the active clip's own cut.
    //  - A real stack / container: the view's frame-0 cut. The diff there reads
    //    the primed scratch (already cut per strip), and `key_home`'s stacked
    //    branch debug-asserts the clock it is handed matches the scratch's own —
    //    so hand it the same one.
    // All cuts are `t.min(len)`, so within the authored range (and with nothing
    // authored) `t_cut == playhead.time()` and this is byte-identical to before.
    let t_cut = timeline.doc.author_cut(solo, root, playhead.time());
    let t = ph2d_timeline::snap_time(
        RationalTime::from_seconds(t_cut),
        fps,
        timeline.flags.frame_snap,
    );

    // The fresh-object default: Position is a motion path (ADR-0141).
    let default_path = true;

    // The diff reads the document BEFORE any upsert: one consistent state for the whole selection.
    let mut to_key: Vec<(u64, PropKind, f32, RationalTime)> = Vec::new();
    // Motion-path anchors to author (Path mode): `(entity, [x, y], key time)`.
    // Separate from `to_key` because an anchor is 2D geometry, keyed through
    // `key_the_path`, not a scalar upsert.
    let mut to_path: Vec<(u64, [f32; 2], RationalTime)> = Vec::new();
    let mut next_baseline: BTreeMap<u64, PoseSample> = BTreeMap::new();
    let refused_now = diff_each_sample(
        timeline,
        samples,
        ak,
        &mut to_key,
        &mut to_path,
        &mut next_baseline,
        t_cut,
        t,
        solo,
        playing,
        capturing,
        default_path,
    );
    ak.baseline = next_baseline;

    // Say it once: on the rising edge, and again if the REASON changes.
    if refused_now != ak.refusal {
        if let Some(r) = refused_now {
            toasts.push(ph2d_editor_core::Toast::warning(ph2d_i18n::tr(
                r.message_key(),
            )));
        }
        ak.refusal = refused_now;
    }

    if capturing {
        // A gizmo drag opens one step on its first frame; a discrete edit brackets just this frame.
        if drag_now && !ak.drag_active && !timeline.history.is_open() {
            timeline.history.begin(&timeline.doc);
        }
        // A path anchor is a discrete edit too.
        let discrete =
            !drag_now && (!to_key.is_empty() || !to_path.is_empty()) && !timeline.history.is_open();
        if discrete {
            timeline.history.begin(&timeline.doc);
        }
        let padrao = default_interp();
        let pivo = ph2d_sprite_edit::pivot_chave::com_pivo(to_key.iter().map(|k| (k.0, k.1)));
        for (entity, prop, v, t_e) in &to_key {
            let interp = ph2d_sprite_edit::pivot_chave::ritmo_da_chave(
                &timeline.doc,
                *entity,
                *prop,
                &pivo,
                padrao,
            );
            timeline
                .doc
                .upsert_key(*entity, *prop, *t_e, AnimValue::Float(*v), interp);
        }
        // The motion-path anchors (Path mode): `key_the_path` binds Position on
        // first touch, adds/moves the anchor, and rewrites the distances the keys
        // hold — the same door the manual `K` uses.
        for (entity, at, t_e) in &to_path {
            timeline.doc.key_the_path(*entity, *t_e, *at);
            if !pivo.is_empty() {
                timeline.doc.path_key_corner(*entity, *t_e);
            }
        }
        if discrete {
            let doc = timeline.doc.clone();
            timeline.history.commit_da_cena(&doc);
        }
    }

    // Close the gizmo drag's step when the drag ends: one undo step if it changed
    // the document.
    if ak.drag_active && !drag_now {
        // A performing session just ended: simplify within this same undo step.
        simplify_recorded(timeline, &ak.record, REC_SMOOTH_PASSES, REC_SIMPLIFY_REL);
        ak.record.clear();
        let doc = timeline.doc.clone();
        timeline.history.commit_da_cena(&doc);
    }
    ak.drag_active = drag_now;
}

#[cfg(test)]
#[path = "autokey_bone_tests.rs"]
mod bone_tests;
#[cfg(test)]
#[path = "autokey_cut_clock_tests.rs"]
mod cut_clock_tests;
#[cfg(test)]
#[path = "autokey_desfazer_tests.rs"]
mod desfazer_tests;
/// `PH2D_AUTOKEY_LOG=1` — o que o Auto Key viu, uma linha quando muda (plano 34: o report do dono
/// de 19/10 de que ele não gravava formas vetoriais).
fn sonda(linha: impl FnOnce() -> String) {
    static LIGADA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    static ULTIMA: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
    if !*LIGADA.get_or_init(|| std::env::var_os("PH2D_AUTOKEY_LOG").is_some()) {
        return;
    }
    let l = linha();
    if let Ok(mut u) = ULTIMA.lock()
        && *u != l
    {
        eprintln!("{l}");
        *u = l;
    }
}

#[cfg(test)]
#[path = "autokey_campos_vetor_tests.rs"]
mod campos_vetor_tests;
#[cfg(test)]
#[path = "autokey_opacidade_vetor_tests.rs"]
mod opacidade_vetor_tests;
#[cfg(test)]
#[path = "autokey_performing_tests.rs"]
mod performing_tests;
#[cfg(test)]
#[path = "autokey_pontos_tests.rs"]
mod pontos_tests;
#[cfg(test)]
#[path = "autokey_preso_tests.rs"]
mod preso_tests;
#[cfg(test)]
#[path = "autokey_previa_tests.rs"]
mod previa_tests;
#[cfg(test)]
#[path = "autokey_refusal_tests.rs"]
mod refusal_tests;
/// Os dois relatos do dono de 2026-09-17 — ver o cabeçalho de lá.
#[cfg(test)]
#[path = "autokey_scrub_tests.rs"]
mod scrub_tests;
#[cfg(test)]
#[path = "autokey_test_helpers.rs"]
mod test_helpers;
#[cfg(test)]
#[path = "autokey_pass_tests.rs"]
mod tests;
