//! ⭐⭐⭐ **O que o Inspector mostra do EMISSOR DE PARTÍCULAS** (TOP-20 #18, W3).
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `The clock is stopped` | a corrida é o relógio a andar — parado, nada nasce |
//! | `Nothing alive right now` | está a correr e não há partícula nenhuma (a emissão acabou, ou está desligada) |
//! | `It starts stopped` | o `Emitting` está desmarcado: só um sinal o acende |
//!
//! ⚠️ **Sem eles, um emissor que está exactamente como o artista pediu lê-se como partido, com
//! todos os números certos no ecrã** — a lição do projéctil e do mover de vista de cima.
//!
//! # ⭐ E as linhas SOMEM conforme o modo
//!
//! A largura/altura da forma só existem fora do `Point`, a explosividade e o `One Shot` andam
//! juntos, e os quatro sinais são um grupo à parte — é a lei do `SignalVerb::uses_arg`, e a
//! alternativa (mostrar sempre) entrega um controlo morto.

use super::*;
use ph2d_editor_core::particles_edits::{
    InspectorParticlesInfo, PARTICLES_NUMBERS, PARTICLES_TEXTS, ParticlesNumber as N,
};
use ph2d_editor_core::widget::SectionFold;

const CHECK_H: f32 = 18.0; // LITERAL-PX-OK: altura visual do Checkbox, igual à das irmãs

/// O rótulo, o passo e a que módulo pertence cada número — **pela ordem do modelo**.
///
/// ⚠️ A tabela é indexada pela [`PARTICLES_NUMBERS`], e há gate a atar os comprimentos: uma linha
/// nova entra num sítio só.
const ROTULOS: [(&str, f64); 19] = [
    ("Amount", 1.0),                   // LITERAL-PX-OK: partículas (inteiro)
    ("Lifetime (s)", 0.1),             // LITERAL-PX-OK: segundos
    ("Lifetime Randomness", 0.05),     // LITERAL-PX-OK: fracção 0..1
    ("Explosiveness", 0.05),           // LITERAL-PX-OK: fracção 0..1
    ("Preprocess (s)", 0.1),           // LITERAL-PX-OK: segundos
    ("Speed Scale", 0.1),              // LITERAL-PX-OK: multiplicador do relógio
    ("Seed", 1.0),                     // LITERAL-PX-OK: semente (inteiro)
    ("Shape Width / Radius (m)", 0.1), // LITERAL-PX-OK: metros
    ("Shape Height (m)", 0.1),         // LITERAL-PX-OK: metros
    ("Speed (m/s)", 0.5),              // LITERAL-PX-OK: metros por segundo
    ("Speed Randomness", 0.05),        // LITERAL-PX-OK: fracção 0..1
    ("Direction (deg)", 5.0),          // LITERAL-PX-OK: graus
    ("Spread (deg)", 5.0),             // LITERAL-PX-OK: graus
    ("Gravity X (m/s\u{b2})", 0.5),    // LITERAL-PX-OK: metros por segundo²
    ("Gravity Y (m/s\u{b2})", 0.5),    // LITERAL-PX-OK: metros por segundo²
    ("Damping", 0.1),                  // LITERAL-PX-OK: fracção por segundo
    ("Size (m)", 0.05),                // LITERAL-PX-OK: metros
    ("Size Randomness", 0.05),         // LITERAL-PX-OK: fracção 0..1
    ("Size at Death", 0.05),           // LITERAL-PX-OK: fracção do tamanho ao nascer
];

/// Uma linha de aviso. (Gémea da do projéctil — ver a irmã.)
#[allow(clippy::too_many_arguments)]
fn warn(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
    token: ColorToken,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(token, theme),
    );
    y + font + ph2d_tokens::control_gap_px()
}

/// Um título de módulo — a arrumação que separa *quantas* de *como saem* e de *como são*.
fn titulo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    // ⚠️ **A cauda de um bloco tem UMA porta** — o que separa o título do que ele intitula é o
    // mesmo degrau de sempre, e escrevê-lo aqui seria a quinta resposta a essa pergunta.
    y + font + ph2d_tokens::control_gap_px()
}

/// Um segmentado. ⚠️ **A selecção vem do SNAPSHOT, nunca do store** (a lei do irmão de vista de
/// cima): ler o store faria o primeiro clique depois de trocar de objecto mandar o valor anterior.
#[allow(clippy::too_many_arguments)]
fn seg_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    titulo_txt: &str,
    ids: &[ph2d_a11y::NodeId],
    rotulos: &[&'static str],
    escolhido: usize,
) -> f32 {
    let row_y = titulo(scene, text_system, theme, x, w, y, titulo_txt);
    let gap = Spacing::Xs.px();
    let n = ids.len() as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, &id) in ids.iter().enumerate() {
        let rect = Rect::new(x + (cw + gap) * i as f32, row_y, cw, ph2d_tokens::ROW_H_PX);
        hit_index.register(id, rect);
        let kind = if i == escolhido {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, rotulos.get(i).copied().unwrap_or(""))
                .kind(kind)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    row_y + ph2d_tokens::row_pitch_px()
}

/// Uma caixa.
#[allow(clippy::too_many_arguments)]
fn check_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
) -> f32 {
    let rect = Rect::new(x, y, w, CHECK_H);
    hit_index.register(id, rect);
    paint_checkbox(
        &Checkbox::new(id, label)
            .visual(store.checkbox_visual(id))
            .value(if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            }),
        rect,
        scene,
        text_system,
        theme,
    );
    y + CHECK_H + ph2d_tokens::control_gap_px()
}

/// Uma linha de número, pela ordem do modelo.
#[allow(clippy::too_many_arguments)]
fn num_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    which: N,
) -> f32 {
    let Some(i) = PARTICLES_NUMBERS.iter().position(|&n| n == which) else {
        return y;
    };
    let (label, step) = ROTULOS[i];
    // ⭐⭐ **A coluna do nome é da SECÇÃO** (`line/UIUX`, 2026-09-15): aqui ela mede-se da
    //    própria tabela, que É a lista de nomes desta secção — as 19 linhas partilham-na, e
    //    medir só a desta faria a coluna saltar de linha para linha.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &ROTULOS.map(|(nome, _)| nome),
    );
    super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        &[crate::ids::INSP_PART_NUM[i]],
        step,
        None,
        seccao,
    )
}

/// **Os AVISOS** — a metade que responde a *«pus o componente e não vejo nada»*.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorParticlesInfo,
) -> f32 {
    let mut cur_y = y;
    if !i.clock_playing {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "The clock is stopped \u{2014} particles are born while the clock plays.",
            ColorToken::Text3,
        );
    } else if i.alive == 0 {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "Nothing alive right now \u{2014} the emission ended, or it is switched off.",
            ColorToken::Text3,
        );
    }
    if !i.emitting {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "It starts stopped \u{2014} a signal switches it on.",
            ColorToken::Warn,
        );
    }
    cur_y
}

/// O corpo da secção — os CONTROLOS, por módulo.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorParticlesInfo,
) -> f32 {
    let mut cur_y = avisos(scene, text_system, theme, x, w, y, i);
    // ── Emissão ──────────────────────────────────────────────────────────────
    cur_y = titulo(scene, text_system, theme, x, w, cur_y, "Emission");
    cur_y = check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_PART_EMITTING,
        "Emitting",
        i.emitting,
    );
    cur_y = check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_PART_ONE_SHOT,
        "One Shot",
        i.one_shot,
    );
    for n in [
        N::Amount,
        N::Life,
        N::LifeRandom,
        N::Explosiveness,
        N::Prewarm,
        N::TimeScale,
        N::Seed,
    ] {
        cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
    }
    // ── De onde nascem ───────────────────────────────────────────────────────
    cur_y = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Emission Shape",
        &crate::ids::INSP_PART_SHAPE,
        &["Point", "Disc", "Ring", "Rect"],
        usize::from(i.shape),
    );
    // ⭐ **As duas medidas só existem fora do ponto** — num ponto elas são inertes, e um controlo
    // vivo que não muda nada é a doença que os gates de param existem para curar.
    if i.shape != 0 {
        for n in [N::ShapeW, N::ShapeH] {
            cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
        }
    }
    // ── Para onde vão ────────────────────────────────────────────────────────
    cur_y = titulo(scene, text_system, theme, x, w, cur_y, "Velocity");
    for n in [N::Speed, N::SpeedRandom, N::Angle, N::Spread] {
        cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
    }
    cur_y = titulo(scene, text_system, theme, x, w, cur_y, "Forces");
    for n in [N::GravityX, N::GravityY, N::Damping] {
        cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
    }
    // ── Como são ─────────────────────────────────────────────────────────────
    cur_y = titulo(scene, text_system, theme, x, w, cur_y, "Look");
    for n in [N::Size, N::SizeRandom, N::SizeEnd] {
        cur_y = num_row(scene, text_system, theme, hit_index, store, x, w, cur_y, n);
    }
    for (id, label, rgba) in [
        (crate::ids::INSP_PART_COLOR, "Color", i.color),
        (
            crate::ids::INSP_PART_COLOR_END,
            "Color at Death",
            i.color_end,
        ),
    ] {
        let cell = Rect::new(x, cur_y, w, ph2d_tokens::ROW_H_PX);
        super::color_tint::paint_tint_swatch_cell(
            cell,
            label,
            id,
            crate::state_tint::tint_f32_to_u8(rgba),
            false,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        cur_y += ph2d_tokens::row_pitch_px();
    }
    // ── Onde vivem ───────────────────────────────────────────────────────────
    cur_y = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        "Simulation Space",
        &crate::ids::INSP_PART_SPACE,
        &["World", "Local"],
        usize::from(i.space),
    );
    // ── Os sinais ────────────────────────────────────────────────────────────
    cur_y = titulo(scene, text_system, theme, x, w, cur_y, "Signals");
    for (k, _t) in PARTICLES_TEXTS.into_iter().enumerate() {
        let dica = [
            "switch on\u{2026}",
            "switch off\u{2026}",
            "restart\u{2026}",
            "shout when done\u{2026}",
        ][k];
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            crate::ids::INSP_PART_TEXT[k],
            TextInput::new(crate::ids::INSP_PART_TEXT[k], "").placeholder(dica),
        );
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_particles_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorParticlesInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PARTICLES_SECTION,
        "Particles",
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PARTICLES_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    if info.selected_count > 1 {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "Editing the primary selection only.",
            ColorToken::Text3,
        );
    }
    cur_y = corpo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
