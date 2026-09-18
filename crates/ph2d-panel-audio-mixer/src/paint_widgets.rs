//! Shared widget-row painters for the Audio Mixer panel — a labeled slider row
//! (master-fx params: EQ, reverb Size/Return, sends, ducking Depth) and a toggle
//! button (mute / solo / effect enables). Split out of `paint.rs` to keep the
//! paint orchestrator under the panel LOC cap.
//!
//! ⚠️ **O doc que esta linha substitui MENTIA**, e a mentira era o defeito: ele dizia que os dois
//! eram *«leaf helpers over the canonical gallery widgets (no bespoke chrome)»*. O slider é — ele
//! chama `paint_slider`; o **toggle não**: ele pinta `Bg3`/`active_bg` à mão. E nenhum dos dois
//! perguntava ao store, então os dois eram **inertes sob o rato** (o mesmo mecanismo que o painel
//! irmão, o Audio Editor, pagou em 2026-08-15: os ids registados, o store a saber, ninguém a
//! perguntar).
//!
//! ⚠️ **Os dois recebem o `WidgetStore` e respondem por si**, em vez de receberem o par visual. É
//! deliberadamente mais forte que a porta `visual(pair)` do catálogo: o pintor **já tem o id**, e
//! derivar as duas metades dele torna o par-descasado inexprimível.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::motion::{self, hover_of, pressed_of};
use ph2d_editor_core::paint::{paint_text_centered, resolve};
use ph2d_editor_core::widget::{ButtonState, Slider, SliderOrientation, paint_slider};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::{Color as VelloColor, VectorScene};

/// ⭐⭐⭐ **OS RÓTULOS DAS BARRAS DO MASTER — a POPULAÇÃO de que a coluna deriva.**
///
/// ⚠️ Ela é a fonte de [`coluna_dos_nomes`] e do gate `nenhum_nome_de_barra_corta_na_coluna`:
/// declarar a lista aqui é o que faz uma barra NOVA entrar na medição sem ninguém se lembrar
/// dela — *uma coluna medida sobre uma lista escrita noutro sítio mede o passado*.
pub const FX_ROW_KEYS: &[&str] = &[
    "panel.audio_mixer.master.low",
    "panel.audio_mixer.master.mid",
    "panel.audio_mixer.master.high",
    "panel.audio_mixer.master.size",
    "panel.audio_mixer.master.depth",
    "panel.audio_mixer.master.time",
    "panel.audio_mixer.master.fbk",
    "panel.audio_mixer.master.return",
    "panel.audio_mixer.bus.music",
    "panel.audio_mixer.bus.sfx",
    "panel.audio_mixer.bus.ui",
    "panel.audio_mixer.bus.voice",
];

/// O PISO da coluna dos nomes — o que ela mediu durante toda a vida deste painel.
///
/// ⛔ Ele fica para a coluna nunca ENCOLHER onde os nomes são curtos: sem piso, uma secção só de
/// `Low`/`Mid`/`High` puxaria as barras para a esquerda e o bloco deixaria de estar alinhado.
const FX_LABEL_MIN_W: f32 = 32.0; // LITERAL-PX-OK: a coluna que este painel sempre teve

/// ⭐⭐⭐ **A COLUNA DOS NOMES, MEDIDA — e não um literal.**
///
/// ⛔⛔ **Medido em 2026-09-18, e o defeito já shipava EM INGLÊS:** com a coluna cravada em
/// `32,0 px`, o `Depth` mede `32,3` e o `Return` `35,9` — *dois nomes cortados na língua em que o
/// app shipa*. No idioma de teste cortam **cinco de oito** (`Return` chega a `55,1`). A foto do
/// dono de 18/09 mostrava a fileira inteira em `[…]`.
///
/// ⚠️ **O TECTO é metade da linha, e ele NÃO morde hoje** (medido): o nome mais largo do idioma de
/// teste pede `55,1 px` e metade da linha mais estreita que o dock permite são `~98`. Ele existe
/// porque *acima de metade o artista lê mais do que arrasta*, e fica com a medição ao lado para
/// quem um dia o vir morder saber que é a lei e não um acidente.
pub fn coluna_dos_nomes(text_system: &mut TextSystem, content_w: f32) -> f32 {
    coluna_dos_nomes_em(ph2d_i18n::idioma(), text_system, content_w)
}

/// ⭐⭐ **A LEI, com o idioma DADO** — e a [`coluna_dos_nomes`] é o acessório que lhe passa o do
/// ambiente, à maneira do par [`ph2d_i18n::tr`]/[`ph2d_i18n::tr_em`].
///
/// ⛔⛔ **Ela existe porque o gate dela nasceu VÁCUO sem ela** (2026-09-18): o `tr` lê o idioma de
/// um `OnceLock` sobre o ambiente do PROCESSO, que numa suíte está sempre em inglês — logo uma
/// régua que medisse o texto deformado contra uma coluna calculada em inglês acusava o produto
/// CERTO. *A coluna de uma língua mede os nomes DESSA língua*, e é isso que o app faz em
/// execução; sem esta porta, o gate não consegue dizê-lo.
#[must_use]
pub fn coluna_dos_nomes_em(
    idioma: ph2d_i18n::Idioma,
    text_system: &mut TextSystem,
    content_w: f32,
) -> f32 {
    let fonte = TypeToken::Xs.px();
    let mais_largo = FX_ROW_KEYS
        .iter()
        .map(|k| text_system.prefix_width(ph2d_i18n::tr_em(idioma, k), fonte))
        .fold(0.0_f32, f32::max);
    let tecto = (content_w * 0.5).max(FX_LABEL_MIN_W);
    mais_largo.clamp(FX_LABEL_MIN_W, tecto) // CLAMP-OK: o tecto é forçado acima do piso na linha de cima
}

/// Paint a small left label + a full-width horizontal Slider on one row (the
/// master-fx parameter rows: EQ, reverb Size/Return, sends, ducking Depth).
/// Returns the next y.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_labeled_slider(
    y: f32,
    col_w: f32,
    label: &str,
    id: NodeId,
    value: f32,
    content_x: f32,
    content_w: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
) -> f32 {
    let label_rect = Rect::new(content_x, y, col_w, Spacing::Md.px());
    paint_text_centered(
        text_system,
        scene,
        label,
        label_rect,
        TypeToken::Xs.px(),
        resolve(ColorToken::Text2, theme),
    );
    let slider_x = content_x + col_w + Spacing::Sm.px();
    let slider_w = (content_w - col_w - Spacing::Sm.px()).max(1.0);
    // ⚠️ A altura da PISTA tem nome desde a wave 19: escrita em linha, ela reaparecia na cauda
    //    (`y + Spacing::Md.px() + …`) e lia-se como um segundo vão — o censo da cauda acusou-a.
    let track_h = Spacing::Md.px();
    let slider_rect = Rect::new(slider_x, y, slider_w, track_h);
    let mut slider = Slider::new(id, label)
        .orientation(SliderOrientation::Horizontal)
        .visual(store.slider_visual(id));
    slider.set_value(value.clamp(0.0, 1.0));
    paint_slider(&slider, slider_rect, scene, theme);
    hit_index.register(id, slider_rect);
    y + track_h + ph2d_tokens::control_gap_px()
}

/// Paint one toggle button (mute / solo / effect enable): `active_bg` tint +
/// `AccentFg` text when engaged, else `Bg3` + `Text1`. Registers `id` as the hit
/// rect.
///
/// ⚠️ **O tom QUENTE é derivado do de repouso, não escolhido** — e aqui isso não é conveniência,
/// é a única resposta possível: o `active_bg` é um PARÂMETRO (`Danger` no Mute, `Warn` no Solo,
/// `Accent` nas master-fx), então uma tabela de pares repouso→quente teria de crescer com cada
/// chamador novo. `ColorToken::hover_of` responde pela FAMÍLIA.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_toggle(
    rect: Rect,
    label: &str,
    active: bool,
    active_bg: ColorToken,
    id: NodeId,
    cell: ph2d_editor_core::widget::GroupCell,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
) {
    let (rest, fg) = if active {
        (active_bg, ColorToken::AccentFg)
    } else {
        (ColorToken::Bg3, ColorToken::Text1)
    };
    let (state, t) = store.button_visual(id);
    let bg = if state == ButtonState::Pressed {
        resolve(pressed_of(rest), theme)
    } else {
        let soft = matches!(state, ButtonState::Normal | ButtonState::Hovered);
        let hot = hover_of(rest);
        motion::hover_axis(soft, t, Some(rest.resolve(theme)), Some(hot.resolve(theme)))
            .map_or_else(
                || {
                    resolve(
                        if state == ButtonState::Hovered {
                            hot
                        } else {
                            rest
                        },
                        theme,
                    )
                },
                |c| VelloColor::from_rgba8(c.r, c.g, c.b, c.a), // LITERAL-COLOR-OK: token-bridge
            )
    };
    // ⭐ As quatro quinas saem da POSIÇÃO no grupo (wave 20): `M | S` de uma faixa encostam, e um
    //    interruptor sozinho continua a arredondar os quatro.
    ph2d_editor_core::paint::fill_rounded_rect_radii(
        scene,
        rect,
        cell.radii(ph2d_editor_core::paint::frame_radius(
            theme,
            ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
        )),
        bg,
    );
    paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        TypeToken::Sm.px(),
        resolve(fg, theme),
    );
    hit_index.register(id, rect);
}
