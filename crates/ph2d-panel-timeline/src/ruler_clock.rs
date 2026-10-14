//! **Which clock a ruler measures** — the question, split out of the painting.
//!
//! It lived in `ruler.rs` until nesting gave it a third answer and the file went past the
//! panel LOC cap. The seam is the one that module's own docs already drew: *"answering here
//! rather than inside the paint loop is what gives that line an oracle at all."* Painting a
//! ruler and deciding what it is a ruler OF are different jobs.

use ph2d_timeline::TimelineViewSnapshot;

/// stand at the wrong second, and the Keys ruler carries none there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RulerClock {
    /// Where the playhead stands **on this ruler**, or `None` when the clock it
    /// measures has no answer here (an Arrange-side clip that plays zero/two times).
    pub now: Option<f64>,
    /// This ruler scrubs — registers the scrub hit and mirrors the playhead into it.
    /// Both tabs do (the Keys ruler scrubs the clip clock).
    pub scrub: bool,
    /// This ruler carries the TIMELINE's markers. Arrange always; Keys only without
    /// a stack (then the clip IS the timeline). The loop is NOT gated by this — it is
    /// each view's own, drawn against the very clock this ruler scrubs.
    pub markers: bool,
    /// This ruler draws (and offers the grips of) the loop band. Everywhere but the
    /// Containers LIST: the list is a library, not a view of time, and playback —
    /// which is all a loop describes — does not exist there (Enio, 2026-07-22).
    /// Gates the BAND and the BRACES together: an invisible band with live grips
    /// would be a control you can grab and cannot see.
    pub loop_band: bool,
    /// This ruler's drag moves the EDITOR's clock, the one a scene simulation (the Motion)
    /// reads — so it carries the recorded-history band (Motion Nodes doc 132 §2), on the axis
    /// [`HistoryAxis`] says. Arrange at the scene root; Keys without a stack (the ONE clock,
    /// `ph2d_timeline::ViewClock`); Keys under a stack only where its clip plays ONCE (the
    /// editor follows through that map — ADR-0115 R8 emenda 3). `None` where the drag cannot
    /// reach the editor's clock: the band would mark seconds the drag never replays.
    pub history: Option<HistoryAxis>,
}

/// How the editor's seconds — the band's — land on this ruler's axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HistoryAxis {
    /// The ruler measures the editor's clock itself.
    Editor,
    /// The ruler measures a clip that plays once: the editor's second `t` is clip second
    /// `map.local_time(t)` (clamped to where the clip plays).
    Clip(ph2d_timeline::ContainerMap),
}

impl HistoryAxis {
    /// The second on this ruler where the editor's second `t` lies.
    #[must_use]
    pub fn ruler_time(self, t: f64) -> f64 {
        match self {
            Self::Editor => t,
            Self::Clip(map) => map.local_time(t),
        }
    }
}

/// **Which clock this tab's ruler measures.** Pure, and tested as such.
///
/// The playhead is paint, not a widget — no hit index remembers it, so a seam test
/// can prove the ruler's *hits* and never its *line*. Answering here rather than
/// inside the paint loop is what gives that line an oracle at all.
pub(crate) fn clock_for(tab: crate::tab::Tab, snap: &TimelineViewSnapshot) -> RulerClock {
    // **The Containers LIST** (the tab at its root — same door as the painters,
    // `tab::rows`): a library of assets measured in seconds, so the ruler still
    // ticks and scrubs, but there is no playback here and therefore NO loop band
    // (Enio, 2026-07-22). Entering a container falls through to the lanes logic
    // below, where the band comes back.
    if crate::tab::rows(tab, snap) == crate::tab::Rows::Containers {
        // **No playhead on the LIST** (Enio, 2026-07-23): a library of assets has
        // no "now" — the line comes back inside a container and on the other tabs.
        // Scrub goes with it: a drag that moves an invisible clock is a control
        // that lies. The ticks stay (the bars are measured in seconds).
        return RulerClock {
            now: None,
            scrub: false,
            markers: true,
            loop_band: false,
            history: None,
        };
    }
    if tab.shows_lanes() {
        // **Inside a container the ruler measures the CONTAINER's OWN clock** (Enio,
        // 2026-07-22): the transport IS that clock now, so `time_seconds` is already the
        // interior-local time — the ruler marks it directly and the scrub writes it
        // directly (identity, `container_open` is the door). The old model ran the SCENE
        // playhead mapped into here, which is why playback was the Arrange's. Markers stay
        // OFF — they are stamped in scene seconds and belong on the Arrange ruler.
        if snap.container_open.is_some() {
            return RulerClock {
                now: Some(snap.time_seconds),
                scrub: true,
                markers: false,
                loop_band: true,
                history: None,
            };
        }
        // Arrange IS the timeline: the transport's playhead, and it owns the markers
        // because they are drawn against the very clock it scrubs.
        return RulerClock {
            now: Some(snap.time_seconds),
            scrub: true,
            markers: true,
            loop_band: true,
            history: Some(HistoryAxis::Editor),
        };
    }
    // Keys scrubs the active clip's own clock (published as `clip_time`). It carries
    // the markers **only when there is no stack** — then the clip IS the timeline,
    // one clock, and the Keys ruler is the timeline ruler it has always been. Under a
    // stack the clip clock ≠ the timeline clock, so timeline markers would sit at the
    // wrong second and belong on the Arrange ruler only. Its LOOP, however, is the
    // clip's own and is always drawn. Without a stack its clock IS the editor's; under one,
    // the editor follows only where the clip plays once (`snap.keys_scene`).
    let history = if snap.stacked() {
        snap.keys_scene.and_then(Result::ok).map(HistoryAxis::Clip)
    } else {
        Some(HistoryAxis::Editor)
    };
    RulerClock {
        now: snap.clip_time,
        scrub: true,
        markers: !snap.stacked(),
        loop_band: true,
        history,
    }
}

#[cfg(test)]
mod tests {

    fn snap(stacked: bool, playhead: f64, clip_time: Option<f64>) -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            time_seconds: playhead,
            clip_time,
            lanes: if stacked {
                vec![ph2d_timeline::LaneView {
                    name: "L".into(),
                    muted: false,
                    weight: 1.0,
                    mode: ph2d_timeline::LaneMode::Override,
                    strips: Vec::new(),
                }]
            } else {
                Vec::new()
            },
            ..TimelineViewSnapshot::default()
        }
    }
    use super::*;
    use crate::tab::Tab;
    /// ⭐ **Without a stack both tabs rule the ONE clock there is** (ADR-0115 R8 emenda 3) —
    /// proven through the door the shell uses (`TimelineState::view_clock`), not asserted on a
    /// hand-built snapshot: with the editor at 1.25 s and a stale clip clock at 9 s, BOTH rulers
    /// read 1.25 and both carry the history band on the editor's own axis.
    #[test]
    fn without_a_stack_both_tabs_rule_the_one_clock_there_is() {
        let (mut editor, mut clip, mut cont) = (
            ph2d_core::Playhead::new(1.0 / 60.0),
            ph2d_core::Playhead::new(1.0 / 60.0),
            ph2d_core::Playhead::new(1.0 / 60.0),
        );
        editor.seek(1.25);
        clip.seek(9.0);
        cont.seek(9.0);
        let mut st = ph2d_timeline::TimelineState::new();
        for tab in [Tab::Keys, Tab::Arrange] {
            st.keys_mode = tab == Tab::Keys;
            let clock = st.view_clock().pick(&editor, &clip, &cont);
            let mut s = TimelineViewSnapshot::default();
            s.rebuild(&mut st, clock, tab == Tab::Keys);
            assert_eq!(
                clock_for(tab, &s),
                RulerClock {
                    now: Some(1.25),
                    scrub: true,
                    markers: true,
                    loop_band: true,
                    history: Some(HistoryAxis::Editor),
                },
                "{tab:?} must rule the editor's clock"
            );
        }
    }
    /// ⭐ **The recorded-history band rides the ruler whose drag moves the editor's clock**
    /// (Motion Nodes doc 132 §2 + ADR-0115 R8 emenda 3): Arrange at the scene root, Keys
    /// without a stack, Keys under a stack ONLY where the clip plays once (mapped onto the
    /// clip's axis). Not where the clip plays zero/two times, not inside a container.
    #[test]
    fn the_history_band_rides_only_a_ruler_that_moves_the_editor_clock() {
        for stacked in [false, true] {
            let s = snap(stacked, 4.0, Some(2.0));
            assert_eq!(
                clock_for(Tab::Arrange, &s).history,
                Some(HistoryAxis::Editor)
            );
        }
        assert_eq!(
            clock_for(Tab::Keys, &snap(false, 4.0, Some(4.0))).history,
            Some(HistoryAxis::Editor),
            "Keys without a stack: the one clock"
        );
        let map = ph2d_timeline::ContainerMap {
            t0: 2.0,
            t1: 5.0,
            u0: 0.0,
            u1: 3.0,
        };
        let mut once = snap(true, 4.0, Some(2.0));
        once.keys_scene = Some(Ok(map));
        assert_eq!(
            clock_for(Tab::Keys, &once).history,
            Some(HistoryAxis::Clip(map))
        );
        assert!((HistoryAxis::Clip(map).ruler_time(3.5) - 1.5).abs() < 1e-12);
        for r in [
            ph2d_timeline::KeyRefusal::PlaysTwice,
            ph2d_timeline::KeyRefusal::NotPlaying,
        ] {
            let mut held = snap(true, 4.0, Some(2.0));
            held.keys_scene = Some(Err(r));
            assert_eq!(clock_for(Tab::Keys, &held).history, None, "{r:?}");
        }
        let mut dentro = snap(false, 4.0, Some(2.0));
        dentro.container_open = Some(0);
        assert_eq!(
            clock_for(Tab::Arrange, &dentro).history,
            None,
            "a container's own clock"
        );
    }
    #[test]
    fn each_tab_reads_its_own_clock() {
        let s = snap(true, 4.0, Some(2.0));
        assert_eq!(clock_for(Tab::Keys, &s).now, Some(2.0), "the CLIP's clock");
        assert_eq!(
            clock_for(Tab::Arrange, &s).now,
            Some(4.0),
            "the TIMELINE's clock"
        );
    }
    #[test]
    fn both_tabs_scrub_their_own_clock() {
        for stacked in [false, true] {
            assert!(
                clock_for(Tab::Keys, &snap(stacked, 4.0, Some(2.0))).scrub,
                "the Keys ruler scrubs the clip clock, stacked={stacked}"
            );
            assert!(clock_for(Tab::Arrange, &snap(stacked, 4.0, Some(2.0))).scrub);
        }
    }
    /// **Dentro de um container a régua arrasta o RELÓGIO DELE — sempre, por identidade**
    /// (Enio, 2026-07-22). O transporte ali é o relógio próprio do container, um clock
    /// autônomo, então o scrub é o `time_seconds` cru — não há mais mapa cena→interior para
    /// faltar, e a antiga recusa "sem inverso" (que dependia desse mapa) deixou de existir
    /// por desenho. `container_open` é a porta; `host_map` não gateia mais nada aqui.
    #[test]
    fn a_container_ruler_scrubs_its_own_clock_by_identity() {
        let mut s = snap(true, 4.0, Some(2.0));
        s.crumbs = vec![(0, "Walk".into())];
        s.container_open = Some(0);
        s.time_seconds = 1.25;
        s.host_map = None; // não importa mais — o relógio do container é autônomo
        let c = clock_for(Tab::Containers, &s);
        assert!(c.scrub, "o relógio próprio do container sempre arrasta");
        assert_eq!(c.now, Some(1.25), "e o now é o tempo LOCAL do container");
        assert!(!c.markers, "os markers são da cena — ficam de fora aqui");
        // Controle: SEM `container_open` (na cena/Arrange) a régua é a da timeline, e o
        // `now` é o mesmo `time_seconds` — o campo é o que distingue as duas leituras.
        s.container_open = None;
        assert!(
            clock_for(Tab::Arrange, &s).markers,
            "a cena carrega os markers"
        );
    }

    #[test]
    fn the_markers_follow_the_timeline_clock() {
        // No stack: the Keys ruler is the timeline ruler, so it keeps the markers.
        assert!(clock_for(Tab::Keys, &snap(false, 4.0, Some(4.0))).markers);
        // Under a stack: only Arrange carries them.
        assert!(!clock_for(Tab::Keys, &snap(true, 4.0, Some(2.0))).markers);
        for stacked in [false, true] {
            assert!(clock_for(Tab::Arrange, &snap(stacked, 4.0, Some(2.0))).markers);
        }
    }

    /// **A LISTA de Containers não tem playhead** (Enio, 2026-07-23: *"na aba
    /// Containers o Playhead deve ser ocultado (não dentro de um container)"*) —
    /// uma biblioteca de assets não tem "agora", então nem linha nem scrub; os
    /// dois voltam DENTRO de um container e nas outras abas.
    #[test]
    fn the_containers_list_has_no_playhead_and_no_scrub() {
        let s = snap(true, 4.0, Some(2.0));
        let list = clock_for(Tab::Containers, &s);
        assert_eq!(list.now, None, "a lista não desenha playhead");
        assert!(
            !list.scrub,
            "um arrasto que move um relógio invisível mente"
        );
        // Dentro de um container os dois voltam.
        let mut inside = snap(true, 4.0, Some(2.0));
        inside.crumbs = vec![(0, "Walk".into())];
        inside.container_open = Some(0);
        inside.time_seconds = 1.25;
        let c = clock_for(Tab::Containers, &inside);
        assert_eq!(c.now, Some(1.25));
        assert!(c.scrub);
    }

    /// **Só a lista de Containers perde a banda de loop** (Enio, 2026-07-22: *"a marca de
    /// loop não deve aparecer"* — a lista é biblioteca, não vista de tempo). Toda outra
    /// vista a mantém, incluindo DENTRO de um container — onde os comandos "voltam a
    /// funcionar", nas palavras do pedido.
    #[test]
    fn only_the_containers_list_loses_the_loop_band() {
        let s = snap(true, 4.0, Some(2.0));
        assert!(
            !clock_for(Tab::Containers, &s).loop_band,
            "a LISTA (crumbs vazios) não desenha a marca de loop"
        );
        assert!(clock_for(Tab::Keys, &s).loop_band);
        assert!(clock_for(Tab::Arrange, &s).loop_band);
        // Dentro de um container (marcado por `container_open`) a banda volta.
        let mut inside = snap(true, 4.0, Some(2.0));
        inside.crumbs = vec![(0, "Walk".into())];
        inside.container_open = Some(0);
        assert!(
            clock_for(Tab::Containers, &inside).loop_band,
            "entrar no container devolve o playback — e a marca junto"
        );
    }
}
