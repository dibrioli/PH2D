//! BottomHUD painter — pill StatusBar centered horizontally,
//! exposing the canonical runtime statistics that 2D game engines
//! track per frame. Values are placeholders until the pilot project
//! wires real metrics from the renderer / ECS / physics.
//!
//! Standard 2D engine HUD segments (left → right):
//! 1. Mode chip (EDIT / PLAY / PAUSE) with a status dot.
//! 2. Frame rate + frame time (`60 fps · 16.7 ms`).
//! 3. Draw calls (`42 draws`).
//! 4. Sprites / quads rendered this frame (`1.2K sprites`).
//! 5. Entities total / active (`256 / 1024 ent`).
//! 6. Rigidbodies + active collision pairs (`32 bodies · 18 coll`).
//! 7. Memory footprint (`128 MB`).
//! 8. Camera zoom (`100%`).
//! 9. Loaded scene name (`default-scene`, muted).

use super::HeroLayout;
use crate::widget::{SegmentTone, StatusBar, StatusSegment, paint_status_bar};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Per-frame render statistics surfaced in the bottom HUD.
/// Phase A (M14.4g): fps, frame_ms, draws, sprite_count, entity_count
/// are real (host writes via [`crate::HeroScreen::set_stats`]); the
/// memory / physics / network slots stay stub strings until those
/// subsystems land. Phase B will add GPU timestamp queries from
/// `wgpu::QuerySet` for per-pass breakdown.
#[derive(Copy, Clone, Debug, Default)]
pub struct BottomHudStats {
    /// Frames per second (derived from `frame_ms`).
    pub fps: f32,
    /// Frame time in milliseconds, EWMA-smoothed by the host.
    pub frame_ms: f32,
    /// GPU draw-calls per frame. For the M14.4d sprite pipeline this
    /// is 1 (one instanced draw per atlas). Tonemap, Vello chrome,
    /// and compositor are fullscreen blits and don't count as draws
    /// in the sprite-batching sense.
    pub draws: u32,
    /// Sprite instances submitted to the GPU this frame.
    pub sprite_count: u32,
    /// Total entity count in the live `SimWorld` (only meaningful in
    /// `PH2D_HERO_LIVE=1` mode; fixture demo reports the populator's
    /// constant).
    pub entity_count: u32,
    /// M14.7 polish (10.1): "raw" fps derived from the CPU-side
    /// frame work time (start of `render_frame` → end of
    /// `queue.submit`), with the vsync wait excluded. Useful as a
    /// hardware-capacity gauge — Unity / Godot expose the same
    /// reading. Status bar renders it alongside the synced fps so
    /// the user can see "60 fps synced · 2400 raw" and gauge how
    /// much headroom each new sprite costs.
    pub raw_fps: f32,
    /// Diagnostics: wall-clock NOT spent in the CPU encode window —
    /// `frame_ms − cpu_encode_ms`, i.e. the present/vsync acquire stall
    /// PLUS any between-frames input work (the gap that makes "Raw" rise
    /// while FPS falls). Watch this spike when a stroke stutters.
    pub present_stall_ms: f32,
    /// Diagnostics: painter CPU per frame (EWMA) — ALL canvas re-stamps this
    /// frame (the once-per-frame coalesced flush for fill methods, PLUS every
    /// incremental per-event stamp for Space/Dots/Airbrush) + the preview
    /// produce/upload dispatch. The per-layer-color cost lands here, NOT in
    /// `raw_fps` (it runs outside the encode window).
    pub paint_ms: f32,
    /// Diagnostics: raw winit `CursorMoved` events since the last frame
    /// (the input rate). Pairs with `paint_stamps` to expose coalescing.
    pub input_events: u32,
    /// Diagnostics: canvas pointer Moves actually delivered to the tool
    /// this frame (= whole-shape re-stamps). With per-frame coalescing this
    /// is ≤1 even when `input_events` is high; if it tracks `input_events`,
    /// the stroke is re-stamping per raw event (the FPS-drop path).
    pub paint_stamps: u32,
}

pub fn paint_bottom_hud(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &crate::interaction::WidgetStore,
    stats: BottomHudStats,
) {
    let bar = bar_que_cabe(stats, layout, store, text_system);
    let rect = rect_of(&bar, layout, store, text_system);
    paint_status_bar(&bar, rect, scene, text_system, theme);
    COLUNA_DO_MODULO.set(None);
}

thread_local! {
    /// A coluna ① do módulo neste quadro (em cartões): o HUD cede-lhe como cede a uma lista (M7).
    static COLUNA_DO_MODULO: std::cell::Cell<Option<Rect>> = const { std::cell::Cell::new(None) };
}

/// A coluna do módulo pintada neste quadro, antes do HUD; o HUD consome-a.
pub(super) fn publica_coluna_do_modulo(r: Option<Rect>) {
    COLUNA_DO_MODULO.set(r);
}

/// A ordem em que as leituras cedem quando a pílula não cabe no vão (índices de [`bar`]): as de
/// diagnóstico antes das que dizem o que há na cena. Tirar uma leitura inteira diz a verdade;
/// encurtar todas deixa a pílula cheia de `59 fps ·…`, que não se lê (foto `r2_wet_tuning_abre.png`).
const CEDEM_POR_ORDEM: [usize; 7] = [2, 3, 7, 8, 4, 10, 5];

/// A barra só com as leituras que cabem INTEIRAS no vão do HUD.
fn bar_que_cabe(
    stats: BottomHudStats,
    layout: &HeroLayout,
    store: &crate::interaction::WidgetStore,
    text_system: &mut TextSystem,
) -> StatusBar {
    let (esq, dir) = vao_do_hud(layout, store);
    cede_ao_vao(bar(stats), dir - esq, text_system)
}

fn cede_ao_vao(inteira: StatusBar, vao: f32, text_system: &mut TextSystem) -> StatusBar {
    let mut barra = inteira.clone();
    let mut sairam = 0;
    while sairam < CEDEM_POR_ORDEM.len() && barra.preferred_width(text_system) > vao {
        sairam += 1;
        let fora = &CEDEM_POR_ORDEM[..sairam];
        barra.segments = inteira
            .segments
            .iter()
            .enumerate()
            .filter(|(i, _)| !fora.contains(i))
            .map(|(_, s)| s.clone())
            .collect();
    }
    barra
}

/// Onde o HUD se pinta: centrado na JANELA, da largura que o texto pede (o canto do quadro desvia-se
/// dele quando a área do quadro é estreita).
#[must_use]
pub fn bottom_hud_rect(
    layout: &HeroLayout,
    store: &crate::interaction::WidgetStore,
    text_system: &mut TextSystem,
    stats: BottomHudStats,
) -> Rect {
    rect_of(&bar(stats), layout, store, text_system)
}

const MARGEM_DO_HUD: f32 = 20.0; // LITERAL-PX-OK: HUD viewport margin (chrome-specific)

/// O vão horizontal que o HUD tem: a janela menos os painéis que flutuam na faixa dele. Quem está na
/// metade esquerda aperta pela esquerda; quem está na direita, pela direita.
fn vao_do_hud(layout: &HeroLayout, store: &crate::interaction::WidgetStore) -> (f32, f32) {
    let coluna = COLUNA_DO_MODULO.get();
    let faixa = layout.bottom_hud;
    let meio = layout.viewport.x + layout.viewport.w * 0.5;
    let mut vao = (
        layout.viewport.x + MARGEM_DO_HUD,
        layout.viewport.x + layout.viewport.w - MARGEM_DO_HUD,
    );
    for r in store.floating_panel_rects().chain(coluna) {
        if r.y >= faixa.y + faixa.h || r.y + r.h <= faixa.y {
            continue;
        }
        if r.x + r.w * 0.5 < meio {
            vao.0 = vao.0.max(r.x + r.w + MARGEM_DO_HUD);
        } else {
            vao.1 = vao.1.min(r.x - MARGEM_DO_HUD);
        }
    }
    vao
}

/// Centrado na janela, da largura que o texto pede, dentro do vão: o HUD cede a uma lista que flutua.
fn no_vao(pref_w: f32, layout: &HeroLayout, (esq, dir): (f32, f32)) -> Rect {
    let w = pref_w.min((dir - esq).max(0.0));
    let x = layout.viewport.x + (layout.viewport.w - w) * 0.5;
    Rect::new(
        x.min(dir - w).max(esq),
        layout.bottom_hud.y,
        w,
        layout.bottom_hud.h,
    )
}

fn rect_of(
    bar: &StatusBar,
    layout: &HeroLayout,
    store: &crate::interaction::WidgetStore,
    text_system: &mut TextSystem,
) -> Rect {
    no_vao(
        bar.preferred_width(text_system),
        layout,
        vao_do_hud(layout, store),
    )
}

fn bar(stats: BottomHudStats) -> StatusBar {
    // Format `entity_count` with a `K` suffix above 999 so the
    // segment width stays predictable when the scene scales up.
    let sprite_label = if stats.sprite_count >= 1_000 {
        ph2d_i18n::tr_with(
            "chrome.hud.sprites_k",
            &[("n", &format!("{:.1}", stats.sprite_count as f32 / 1_000.0))], // LITERAL-PX-OK: 1K divisor (display thousands)
        )
    } else {
        ph2d_i18n::tr_with("chrome.hud.sprites", &[("n", &stats.sprite_count)])
    };
    let draws_label = if stats.draws <= 1 {
        ph2d_i18n::tr_with("chrome.hud.one_draw", &[("n", &stats.sprite_count)])
    } else {
        ph2d_i18n::tr_with("chrome.hud.draws", &[("n", &stats.draws)])
    };
    StatusBar::new(
        NodeId(300),
        tr("chrome.hud.editor_statistics"),
        vec![
            StatusSegment::new(tr("chrome.hud.edit"))
                .dot(true)
                .tone(SegmentTone::Neutral),
            StatusSegment::new(ph2d_i18n::tr_with(
                "chrome.hud.fps",
                &[
                    ("fps", &(stats.fps as u32)),
                    ("ms", &format!("{:.1}", stats.frame_ms)),
                    ("raw", &(stats.raw_fps as u32)),
                ],
            )),
            // Diagnostics (M14.7+): present/input stall + painter CPU per frame.
            StatusSegment::new(ph2d_i18n::tr_with(
                "chrome.hud.stall",
                &[
                    ("stall", &format!("{:.1}", stats.present_stall_ms)),
                    ("paint", &format!("{:.1}", stats.paint_ms)),
                ],
            )),
            // Diagnostics: input rate vs delivered re-stamps (coalescing gauge).
            StatusSegment::new(ph2d_i18n::tr_with(
                "chrome.hud.events",
                &[("ev", &stats.input_events), ("stamps", &stats.paint_stamps)],
            )),
            StatusSegment::new(draws_label),
            StatusSegment::new(sprite_label).tone(SegmentTone::Accent),
            StatusSegment::new(ph2d_i18n::tr_with(
                "chrome.hud.entities",
                &[("n", &stats.entity_count)],
            )),
            // Physics / memory / network slots stay placeholder until
            // those subsystems land (M14.2 Luau + M10 Rapier are the
            // next major milestones; memory needs a `wgpu::Device`
            // global-memory query that's adapter-specific).
            StatusSegment::new(tr("chrome.hud.bodies")),
            StatusSegment::new(tr("chrome.hud.mb")),
            StatusSegment::new("100%"),
            StatusSegment::new(tr("chrome.hud.scene_name")).tone(SegmentTone::Muted),
        ],
    )
}

/// ⑤ **O rodapé fala do módulo?** — a dica e as leituras, só no layout que o módulo É (o *Vetor*,
/// D7) ou com o Pixel Sprite em Draw (o zoom lê-se aqui, ADR-0196), e só quando ele publicou uma
/// dica; senão `None` e a pílula continua de motor.
#[must_use]
pub fn module_hint(store: &crate::interaction::WidgetStore) -> Option<(String, Vec<String>)> {
    let vector = store.active_layout() == crate::screens::task_layout::TaskLayout::Vector;
    store
        .tool_bars()
        .filter(|b| vector || b.pixel_rail)
        .filter(|b| !b.hint.is_empty())
        .map(|b| (b.hint.clone(), b.readout.clone()))
}

/// ⭐ **⑤ O RODAPÉ DE UM MÓDULO** (D10): *que teclas valem agora?* — a dica da ferramenta na mão
/// (o que o clique, o arrasto e os modificadores fazem) e as leituras do desenho (coordenadas,
/// encaixe, zoom), no lugar das estatísticas de motor. Mesma pílula, mesmo sítio: 0 px novos.
///
/// ⚠️ Só no layout que o módulo É (o *Vetor*, D7): nos outros a pílula continua de motor.
pub fn paint_tool_hint_hud(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &crate::interaction::WidgetStore,
    hint: &str,
    readout: &[String],
) {
    let mut segments = vec![
        StatusSegment::new(hint.to_owned())
            .dot(true)
            .tone(SegmentTone::Neutral),
    ];
    segments.extend(
        readout
            .iter()
            .map(|r| StatusSegment::new(r.clone()).tone(SegmentTone::Muted)),
    );
    let bar = StatusBar::new(NodeId(300), tr("chrome.hud.tool_hint"), segments);
    let rect = rect_of(&bar, layout, store, text_system);
    paint_status_bar(&bar, rect, scene, text_system, theme);
    COLUNA_DO_MODULO.set(None);
}

#[cfg(test)]
mod o_hud_cede_tests {
    use super::{BottomHudStats, bar, cede_ao_vao};
    use super::{no_vao, vao_do_hud};
    use crate::interaction::WidgetStore;
    use crate::screens::layout::HeroLayout;
    use crate::zones::Rect;
    use ph2d_a11y::NodeId;

    /// A largura que a pílula pede no app a 1200 px (a foto `b_physics_debug_dobrado_hud_por_cima.png`).
    const PILULA: f32 = 1036.0;

    /// *Mutação: o `vao.1.min(..)` fora ⇒ o HUD pinta-se por cima da lista.*
    #[test]
    fn the_hud_gives_way_to_the_module_column_on_its_band() {
        let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 1000.0, 800.0));
        let store = WidgetStore::default();
        let (esq_antes, _) = vao_do_hud(&layout, &store);
        let faixa = layout.bottom_hud;
        super::publica_coluna_do_modulo(Some(Rect::new(229.0, 100.0, 32.0, faixa.y + 40.0)));
        let (esq, _) = vao_do_hud(&layout, &store);
        assert!(esq > 261.0, "o vão começa depois da coluna: {esq}");
        super::publica_coluna_do_modulo(Some(Rect::new(229.0, 100.0, 32.0, faixa.y - 110.0)));
        assert_eq!(
            vao_do_hud(&layout, &store).0,
            esq_antes,
            "a coluna acima da faixa não pesa"
        );
        super::publica_coluna_do_modulo(None);
    }

    #[test]
    fn the_hud_gives_way_to_a_list_floating_on_its_band() {
        let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 1200.0, 1672.0));
        let faixa = layout.bottom_hud;
        let mut store = WidgetStore::default();
        let centrado = no_vao(PILULA, &layout, vao_do_hud(&layout, &store));
        assert!(
            (centrado.x + centrado.w * 0.5 - 600.0).abs() < 0.5,
            "{centrado:?}"
        );
        assert!((centrado.w - PILULA).abs() < 0.5, "cabe: não encolhe");

        let lista = Rect::new(938.0, 100.0, 252.0, faixa.y + faixa.h);
        store.set_floating_panel_rect(NodeId(77), lista);
        let r = no_vao(PILULA, &layout, vao_do_hud(&layout, &store));
        assert!(r.x + r.w <= lista.x, "o HUD {r:?} entra na lista {lista:?}");
        assert!(r.x >= 0.0 && r.w > 0.0, "{r:?}");

        // Uma lista à esquerda aperta pelo outro lado.
        let hier = Rect::new(4.0, 100.0, 262.0, faixa.y + faixa.h);
        store.set_floating_panel_rect(NodeId(78), hier);
        let r = no_vao(PILULA, &layout, vao_do_hud(&layout, &store));
        assert!(r.x >= hier.x + hier.w, "{r:?}");
        assert!(r.x + r.w <= lista.x, "{r:?}");

        // Lista que acaba acima da faixa não aperta nada.
        let mut so_alta = WidgetStore::default();
        so_alta
            .set_floating_panel_rect(NodeId(77), Rect::new(938.0, 100.0, 252.0, faixa.y - 100.0));
        let r = no_vao(PILULA, &layout, vao_do_hud(&layout, &so_alta));
        assert!((r.w - PILULA).abs() < 0.5, "{r:?}");
    }

    /// *Mutação: `cede_ao_vao` devolve a barra inteira ⇒ no vão apertado as leituras viram `…`.*
    #[test]
    fn the_hud_drops_whole_readings_rather_than_cutting_them() {
        let mut ts = ph2d_text::TextSystem::new();
        let inteira = bar(BottomHudStats::default());
        let pede = inteira.preferred_width(&mut ts);
        assert!(pede > 0.0, "a medida precisa de fonte");

        let cabe = cede_ao_vao(inteira.clone(), pede + 1.0, &mut ts);
        assert_eq!(
            cabe.segments.len(),
            inteira.segments.len(),
            "cabe: não tira nada"
        );

        let vao = pede * 0.6;
        let cedeu = cede_ao_vao(inteira.clone(), vao, &mut ts);
        assert!(cedeu.segments.len() < inteira.segments.len());
        assert!(
            cedeu.preferred_width(&mut ts) <= vao,
            "as que ficam cabem INTEIRAS em {vao}"
        );
        let textos: Vec<_> = cedeu.segments.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(
            textos[0], inteira.segments[0].text,
            "o modo fica: {textos:?}"
        );
        assert!(
            textos.contains(&inteira.segments[1].text.as_str()),
            "o fps fica: {textos:?}"
        );
        assert!(
            !textos.contains(&inteira.segments[2].text.as_str()),
            "o diagnóstico sai primeiro: {textos:?}"
        );
    }
}

#[cfg(test)]
mod module_hint_tests {
    use super::module_hint;
    use crate::interaction::{ToolBars, WidgetStore};
    use crate::screens::task_layout::TaskLayout;

    /// ⑤ Só no layout Vetor, e só com dica: noutro layout, ou sem barras, a pílula é de motor.
    #[test]
    fn the_footer_speaks_for_the_module_only_in_its_layout() {
        let mut store = WidgetStore::default();
        let bars = ToolBars {
            hint: "Click: corner".to_owned(),
            readout: vec!["x 1 px  y 2 px".to_owned()],
            ..ToolBars::default()
        };
        store.set_tool_bars(Some(bars));
        store.set_active_layout(TaskLayout::Drawing2d);
        assert_eq!(
            module_hint(&store),
            None,
            "fora do layout Vetor o rodapé é de motor"
        );
        store.set_active_layout(TaskLayout::Vector);
        assert_eq!(
            module_hint(&store),
            Some((
                "Click: corner".to_owned(),
                vec!["x 1 px  y 2 px".to_owned()]
            ))
        );
        store.set_tool_bars(None);
        assert_eq!(
            module_hint(&store),
            None,
            "sem barras publicadas não há dica"
        );
    }
}
