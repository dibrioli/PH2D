//! ⭐⭐ **O QUE A FERRAMENTA MOTION ABRE E FECHA** — a visibilidade dos painéis, a divisão do
//! centro e o relógio que arranca ao entrar.
//!
//! ⚠️ **Irmão de [`super`] por RESPONSABILIDADE** (HR-18): lá vive o laço por quadro (cozer,
//! aplicar intenções, publicar a vista), aqui *o que muda no ecrã quando a ferramenta entra e
//! sai*. As duas perguntas têm ritmos diferentes — uma corre sempre, a outra só na borda.

use super::{CenterSplit, HeroScreen, LAST_ACTIVE};

/// ⭐ **O painel lateral de params foi pedido de volta?** (`PH2D_MOTION_PANEL=1`).
///
/// ⚠️ **Lida UMA vez** — uma env var lida por quadro é uma decisão que pode mudar a meio de um
/// gesto, e o dock aparecia e desaparecia debaixo do dedo.
pub fn painel_lateral() -> bool {
    use std::sync::OnceLock;
    static PEDIDO: OnceLock<bool> = OnceLock::new();
    *PEDIDO.get_or_init(|| std::env::var("PH2D_MOTION_PANEL").is_ok_and(|v| v != "0"))
}

/// A metade de SUPERFÍCIE do `dispatch` — chamada por quadro, mas quase tudo aqui é
/// edge-triggered na activação.
pub(super) fn open_and_close(
    hero: &mut HeroScreen,
    playhead: &mut ph2d_core::Playhead,
    motion_active: bool,
    cursor: (f32, f32),
) {
    // ── 1. Panel visibility (mirror of the Vector dock takeover) ──────────
    hero.panel_visibility.insert(
        ph2d_editor::screens::hero::PANEL_MOTION_GRAPH,
        motion_active,
    );
    // ⭐⭐⭐ **O PAINEL LATERAL DE PARAMS SAIU** (doc 103, ordem do Enio de 2026-09-05: *«como no
    // Blender, os parâmetros dos nós são desenhados nos nós e vamos retirar o painel lateral»*).
    //
    // ⚠️ **A condição era um NÚMERO, e ele chegou a zero:** o censo do `panel_exit_probe` conta
    // os controlos que o cartão PINTA e não abre — eram **26** quando a conta começou. Retirá-lo
    // antes disso teria deixado cada um deles inalcançável, que é a forma exacta do knob morto
    // (`CLAUDE.md` §5.0): o controlo continua declarado, continua desenhado, e simplesmente não
    // abre.
    //
    // ⚠️ **E o painel não foi APAGADO, foi DESLIGADO** — `PH2D_MOTION_PANEL=1` traz-no de volta.
    // A crate continua a ser a casa das rows que a shell constrói (o gerador de tutoriais, os
    // params de um subgrafo, a leitura de volta do selector), e um smoke que precise de comparar
    // as duas superfícies lado a lado não pode depender de uma recompilação.
    hero.panel_visibility
        .insert("motion_params", motion_active && painel_lateral());

    // Graph keyboard focus follows the cursor, re-evaluated EVERY frame (not just
    // on move) so a cursor that stopped over the graph before the panel published
    // its rect still gets focus by the time a key is pressed. `panel_rect` is from
    // last frame's paint (stable); `None` off the graph → the scene owns keys.
    let over_graph = motion_active
        && hero
            .store
            .panel_rect(ph2d_editor::ids::MOTION_GRAPH_PANEL)
            .is_some_and(|r| r.contains(cursor.0, cursor.1));
    hero.store
        .set_graph_focused(over_graph.then_some(ph2d_editor::ids::MOTION_GRAPH_PANEL));

    // ── 2. Center split + Inspector takeover — edge-triggered on activation ──
    {
        use std::sync::atomic::Ordering;
        let was = LAST_ACTIVE.swap(motion_active, Ordering::Relaxed);
        if was != motion_active {
            // ⛔⛔ **A TOMADA DE CONTA DO INSPECTOR ACABOU AQUI, e a ausência é a mudança.**
            //
            // Ela existia por uma razão só: o `motion_params` ocupava a coluna da direita, e
            // dois painéis no mesmo sítio é um a tapar o outro. Com os params dentro do cartão
            // (doc 103) não há quem tome o lugar — e esconder o Inspector passaria a ser tirar
            // uma superfície sem pôr nenhuma. *O artista ficaria sem o Inspector **e** sem os
            // params, num sítio onde antes tinha um dos dois.*
            //
            // ⚠️ **E a ausência é LIDA:** o gate `a_layout_names_the_inspector_exactly_when_…`
            // varre as pontes à procura desta escrita e exige que o layout do `motion` passe a
            // NOMEAR o inspector — o que ele faz agora. As duas metades não podem divergir.
            //
            // ⚠️ Com `PH2D_MOTION_PANEL=1` o painel volta e os dois partilham a coluna
            // (empilhados). É uma porta de bissecção, não o produto.
            if motion_active {
                // Split into scene ⟂ graph. Keep any orientation the user already
                // chose (SplitH/SplitV chips); default to Cavalry-style horizontal.
                if !hero.view.center_split.is_split() {
                    hero.view.center_split = CenterSplit::Horizontal {
                        t: CenterSplit::T_DEFAULT,
                    };
                }
                // Auto-play on entry so time-driven behaviours animate live the
                // moment the tool opens (Cavalry/AE preview semantics). Space
                // toggles pause; nothing moves until a `Temporal` node is wired.
                // This plays the EDITOR's clock (W4.T7) — the timeline runs with
                // the graph, which is the point of there being only one.
                playhead.play();
                // …and the timeline COMES WITH IT (W4.T4). It was already running (the bridge
                // and the snapshot never cared whether it was visible) and already driving this
                // very clock — it just was not on screen unless the artist happened to have
                // pressed `L`. A tool that auto-plays and hides the transport is a tool that
                // asks you to scrub blind. The layout gives it a band of its own under the
                // graph (`HeroLayout::dock_timeline_into_motion`).
                //
                // Leaving the tool does NOT hide it again: it is the GLOBAL timeline, and taking
                // away a panel the artist can see is not ours to do. `L` still toggles it.
                hero.panel_visibility
                    .insert(ph2d_editor::screens::hero::PANEL_TIMELINE, true);
            } else {
                hero.view.center_split = CenterSplit::None;
            }
        }
    }
}
