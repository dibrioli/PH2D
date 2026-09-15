//! ⭐⭐⭐ **As secções FACTORY e LIFECYCLE** — o que faz nascer e o que faz morrer (TOP-20 #11 e #12).
//!
//! # ⚠️ Elas nascem COM a wave, e isso é a lição que o `Timers` custou
//!
//! O `Timers` shipou anexável e sem linha de edição, e o report do dono foi *«timer sumiu do modal
//! de componente»*. *Um componente anexável sem painel é indistinguível de um que não foi anexado.*
//!
//! # ⚠️ DUAS secções, porque o SUJEITO é outro
//!
//! A `Factory` vive em quem fabrica; a `Lifetime` e o `DestroyOutside` vivem na **RECEITA**. Uma
//! secção só chamada *Factory* a mostrar apenas uma vida seria um título a mentir — ao contrário da
//! câmera, onde os três corpos são do MESMO objecto.
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `no recipe` | o `Recipe` não aponta a um mestre vivo | escrever o nome da receita |
//! | `never fires` | o `On Signal` está vazio ⇒ nada a acorda | escrever o nome do sinal |
//! | `the clock is stopped` | a corrida é o relógio a andar | carregar no play |
//! | `nothing is born from this object` | uma vida num objecto que nenhuma fábrica fez nascer | pôr o componente na RECEITA |
//! | `no game camera` | sem ecrã de jogo, o fora-do-ecrã não mede nada | anexar uma `GameCamera` à cena |
//!
//! ⚠️ **O quarto é a metade honesta do `Timer`** (*«This timer never starts»*), e sem ele um
//! `Lifetime` num objecto desenhado parece um controlo partido em vez de um controlo **inerte por
//! lei**.

use super::*;
use ph2d_editor_core::screens::hero::{
    InspectorFactory, InspectorFactoryInfo, InspectorLifecycle, InspectorSpawnWhere,
};
use ph2d_editor_core::widget::SectionFold;

const CHECK_H: f32 = 18.0; // LITERAL-PX-OK: altura visual do Checkbox, igual à das irmãs

/// Uma linha de aviso. Devolve o `y` seguinte. (Gémea da da câmera — ver o irmão.)
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

/// O segmentado do ONDE. ⚠️ A selecção vem do SNAPSHOT, nunca do store.
#[allow(clippy::too_many_arguments)]
fn where_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    sel: InspectorSpawnWhere,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        "Where",
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let row_y = y + font + Spacing::Xs.px();
    let gap = Spacing::Xs.px();
    let n = ph2d_editor_core::ids::INSP_FACTORY_WHERE_LEN as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, &id) in crate::ids::INSP_FACTORY_WHERE.iter().enumerate() {
        let rect = Rect::new(x + (cw + gap) * i as f32, row_y, cw, ph2d_tokens::ROW_H_PX);
        hit_index.register(id, rect);
        let kind = if InspectorSpawnWhere::ALL[i] == sel {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, InspectorSpawnWhere::ALL[i].label())
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

/// O corpo da FÁBRICA.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn factory_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    f: &InspectorFactory,
    clock_playing: bool,
) -> f32 {
    // ⚠️ **Os avisos vêm ANTES dos números** — quem não vê nada nascer não quer afinar uma rajada.
    let mut cur_y = y;
    if f.recipe.trim().is_empty() || !f.recipe_found {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "No recipe \u{2014} nothing to make copies of.",
            ColorToken::Danger,
        );
    }
    if f.on_signal.trim().is_empty() {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "No signal \u{2014} this factory never fires.",
            ColorToken::Warn,
        );
    } else if !clock_playing {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "The clock is stopped \u{2014} copies are born while it plays.",
            ColorToken::Text3,
        );
    }

    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_FACTORY_RECIPE,
        TextInput::new(crate::ids::INSP_FACTORY_RECIPE, "").placeholder("recipe name\u{2026}"),
    );
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_FACTORY_ON_SIGNAL,
        TextInput::new(crate::ids::INSP_FACTORY_ON_SIGNAL, "").placeholder("on signal\u{2026}"),
    );
    cur_y = where_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        f.spawn_where,
    );
    // ⚠️ **Só o que o MODO lê é pintado** — a lei do `SignalVerb::uses_arg`: um campo que o modo não
    // lê é um controlo morto; escondê-lo onde ele lê é uma feature inalcançável.
    if f.spawn_where.uses_area() {
        cur_y = super::anchors::field_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            "Area (m)",
            &[
                crate::ids::INSP_FACTORY_AREA_W,
                crate::ids::INSP_FACTORY_AREA_H,
            ],
            0.1, // LITERAL-PX-OK: passo em metros
        );
    }
    if f.spawn_where.uses_tag() {
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            crate::ids::INSP_FACTORY_TAG,
            TextInput::new(crate::ids::INSP_FACTORY_TAG, "").placeholder("spawn point tag\u{2026}"),
        );
        let rect = Rect::new(x, cur_y, w, CHECK_H);
        hit_index.register(crate::ids::INSP_FACTORY_PICK_RANDOM, rect);
        paint_checkbox(
            &Checkbox::new(crate::ids::INSP_FACTORY_PICK_RANDOM, "Pick at random")
                .visual(store.checkbox_visual(crate::ids::INSP_FACTORY_PICK_RANDOM))
                .value(if f.pick_random {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                }),
            rect,
            scene,
            text_system,
            theme,
        );
        cur_y += CHECK_H + ph2d_tokens::control_gap_px();
    }

    for (label, id, step) in [
        ("Burst", crate::ids::INSP_FACTORY_BURST, 1.0), // LITERAL-PX-OK: contagem
        (
            "Max Alive (0 = no limit)",
            crate::ids::INSP_FACTORY_ALIVE_MAX,
            1.0,
        ), // LITERAL-PX-OK: contagem
        (
            "Max Total (0 = no limit)",
            crate::ids::INSP_FACTORY_TOTAL_MAX,
            1.0,
        ), // LITERAL-PX-OK: contagem
        ("Seed", crate::ids::INSP_FACTORY_SEED, 1.0),   // LITERAL-PX-OK: contagem
    ] {
        cur_y = super::anchors::field_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &[id],
            step,
        );
    }
    for (id, ph) in [
        (crate::ids::INSP_FACTORY_ON_SPAWNED, "on spawned\u{2026}"),
        (
            crate::ids::INSP_FACTORY_ON_EXHAUSTED,
            "on exhausted\u{2026}",
        ),
    ] {
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            id,
            TextInput::new(id, "").placeholder(ph),
        );
    }
    // ⭐ **O número que muda sozinho** — é ele que responde *«a fábrica está a trabalhar?»* sem o
    // artista contar objectos no ecrã.
    warn(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        &format!("{} alive now", f.alive),
        ColorToken::Text2,
    )
}

/// O corpo do CICLO DE VIDA.
#[allow(clippy::too_many_arguments)]
fn lifecycle_body(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    l: &InspectorLifecycle,
    info: &InspectorFactoryInfo,
) -> f32 {
    let mut cur_y = y;
    // ⭐⭐ **A metade honesta** — a lei é *a morte só alcança quem nasceu numa corrida*.
    if !info.is_spawned {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "Nothing is born from this object \u{2014} put this on the recipe a Factory makes.",
            ColorToken::Text3,
        );
    }
    if l.lifetime_s.is_some() {
        cur_y = super::anchors::field_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            "Lifetime (s, 0 = forever)",
            &[crate::ids::INSP_LIFE_SECONDS],
            0.1, // LITERAL-PX-OK: passo em segundos
        );
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            crate::ids::INSP_LIFE_ON_DEATH,
            TextInput::new(crate::ids::INSP_LIFE_ON_DEATH, "").placeholder("on death\u{2026}"),
        );
    }
    if l.outside_margin.is_some() {
        if !info.has_game_camera {
            cur_y = warn(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                "No game camera \u{2014} off-screen has no screen to measure.",
                ColorToken::Warn,
            );
        }
        cur_y = super::anchors::field_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            "Off-screen margin (m)",
            &[crate::ids::INSP_LIFE_OUTSIDE_MARGIN],
            0.1, // LITERAL-PX-OK: passo em metros
        );
    }
    cur_y
}

/// Pinta a secção FACTORY. Devolve o `y` seguinte. ⚠️ Gémea da da câmera: o cabeçalho é desta
/// função e o invólucro (`begin_section`/`finish_section`) é de quem chama.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_factory_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorFactoryInfo,
) -> f32 {
    let Some(f) = info.factory.as_ref() else {
        return y;
    };
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_FACTORY_SECTION,
        "Factory",
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
        ph2d_editor_core::ids::INSP_LIVE_FACTORY_SECTION,
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
    cur_y = factory_body(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        f,
        info.clock_playing,
    );
    fold.finish(store, scene, hit_index, cur_y)
}

/// Pinta a secção LIFECYCLE. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_lifecycle_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorFactoryInfo,
) -> f32 {
    let Some(l) = info.lifecycle.as_ref() else {
        return y;
    };
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_LIFECYCLE_SECTION,
        "Lifecycle",
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
        ph2d_editor_core::ids::INSP_LIVE_LIFECYCLE_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let cur_y = lifecycle_body(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y + header_h,
        l,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
