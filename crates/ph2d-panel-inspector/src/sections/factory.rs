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
    InspectorFactory, InspectorFactoryInfo, InspectorSpawnWhere,
};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::{tr, tr_with};

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
        tr("panel.inspector.factory.where"),
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let row_y = y + font + Spacing::Xs.px();
    let gap = Spacing::Xs.px();
    // ⭐ Uma ESCOLHA, não três comandos — ver [`ph2d_editor_core::widget::composto`].
    ph2d_editor_core::widget::composto::grupo(crate::ids::INSP_FACTORY_WHERE.iter().copied());
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

/// ⭐⭐ **ONDE a cópia nasce** — as três formas de uma pergunta só. Devolve o `y` seguinte.
///
/// ⚠️ **Saiu do [`factory_body`] por TECTO DE FUNÇÃO** (`215/200` ao ganhar a linha da MIRA) **e é
/// o certo por responsabilidade:** o selector, a área, a tag e o sorteio respondem todos à mesma
/// pergunta, e as linhas que sobram no corpo respondem a outras (*o quê* · *ao ouvir o quê* ·
/// *quantos* · *para onde*).
///
/// ⚠️ **Só o que o MODO lê é pintado** — a lei do `SignalVerb::uses_arg`: um campo que o modo não
/// lê é um controlo morto; escondê-lo onde ele lê é uma feature inalcançável.
#[allow(clippy::too_many_arguments)]
fn onde_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    f: &InspectorFactory,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = y;

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
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.factory.area_m"),
            &[
                crate::ids::INSP_FACTORY_AREA_W,
                crate::ids::INSP_FACTORY_AREA_H,
            ],
            0.1, // LITERAL-PX-OK: passo em metros
            Some(ph2d_editor_core::widget::Unit::Meters),
            seccao,
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
            TextInput::new(crate::ids::INSP_FACTORY_TAG, "")
                .placeholder(ph2d_i18n::tr("panel.factory.tag")),
        );
        cur_y = ph2d_editor_core::property_row::paint_check_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            (
                crate::ids::INSP_FACTORY_PICK_RANDOM,
                tr("panel.inspector.factory.pick_at_random"),
                f.pick_random,
            ),
            seccao,
        );
    }

    cur_y
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
    // ⭐⭐ **A coluna do nome é da SECÇÃO** (`line/UIUX`, 2026-09-15): esta secção nasceu
    //    contra a porta antiga (`anchors::field_row`, o nome POR CIMA do campo) e passa à
    //    única que existe. ⚠️ Os nomes são os da secção INTEIRA, inclusive os das linhas que
    //    este quadro não pinta — *uma coluna que salta quando uma linha aparece é uma coluna
    //    por linha com outro nome.*
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        2,
        &[
            tr("panel.inspector.factory.area_m"),
            tr("panel.inspector.factory.burst"),
            tr("panel.inspector.factory.max_alive_0_no_limit"),
            tr("panel.inspector.factory.max_total_0_no_limit"),
            tr("panel.inspector.factory.seed"),
            // ⭐⭐ **As duas linhas de MARCAR entram aqui** — a lei está escrita três linhas
            //    acima (*«os nomes são os da secção INTEIRA»*) e elas faltavam, porque até
            //    2026-09-21 as duas eram montadas à mão com a coluna de omissão
            //    (`Seccao::apenas_campos(1)`). ⛔ Sem elas a coluna mede-se sem o nome mais
            //    comprido da secção, e ao passar pela porta ele saía CORTADO.
            tr("panel.inspector.factory.pick_at_random"),
            tr("panel.inspector.factory.aim_from_spawner"),
        ],
    );
    cur_y = factory_avisos(scene, text_system, theme, x, w, cur_y, f, clock_playing);
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
        TextInput::new(crate::ids::INSP_FACTORY_RECIPE, "")
            .placeholder(ph2d_i18n::tr("panel.factory.recipe")),
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
        TextInput::new(crate::ids::INSP_FACTORY_ON_SIGNAL, "")
            .placeholder(ph2d_i18n::tr("panel.factory.on_signal")),
    );
    cur_y = onde_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        f,
        seccao,
    );

    // ⭐ **A MIRA** (o gatilho, 2026-09-18) — a cópia sai apontada para onde a fábrica aponta.
    //
    // ⚠️ **Ela mora AQUI e não na secção do projéctil**, e a razão é de quem decide: quem sabe a
    // direcção é a FÁBRICA (a arma), não a bala — a bala já lê o ângulo do próprio corpo.
    cur_y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_FACTORY_AIM,
            tr("panel.inspector.factory.aim_from_spawner"),
            f.aim_from_spawner,
        ),
        seccao,
    );

    for (label, id, step) in [
        (
            tr("panel.inspector.factory.burst"),
            crate::ids::INSP_FACTORY_BURST,
            1.0,
        ), // LITERAL-PX-OK: contagem
        (
            tr("panel.inspector.factory.max_alive_0_no_limit"),
            crate::ids::INSP_FACTORY_ALIVE_MAX,
            1.0,
        ), // LITERAL-PX-OK: contagem
        (
            tr("panel.inspector.factory.max_total_0_no_limit"),
            crate::ids::INSP_FACTORY_TOTAL_MAX,
            1.0,
        ), // LITERAL-PX-OK: contagem
        (
            tr("panel.inspector.factory.seed"),
            crate::ids::INSP_FACTORY_SEED,
            1.0,
        ), // LITERAL-PX-OK: contagem
    ] {
        cur_y = super::rows::fields_row(
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
            None,
            seccao,
        );
    }
    // ⚠️ **Os dois pela TABELA**, como os irmãos — um literal aqui seria a palavra do app escrita
    // num sítio que a tradução não alcança (HR-15).
    for (id, chave) in [
        (
            crate::ids::INSP_FACTORY_ON_SPAWNED,
            "panel.factory.on_spawned",
        ),
        (
            crate::ids::INSP_FACTORY_ON_EXHAUSTED,
            "panel.factory.on_exhausted",
        ),
    ] {
        let ph = ph2d_i18n::tr(chave);
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
    super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        &tr_with("panel.inspector.factory.alive_now", &[("n", &f.alive)]),
        ColorToken::Text2,
    )
}

/// **Os avisos da FÁBRICA** — irmão por tecto de LOC (a função passou a `215/200` quando a
/// coluna da secção entrou). ⚠️ O corte é por RESPONSABILIDADE: aqui *porque é que nada nasce*;
/// lá *o que o artista afina*.
#[allow(clippy::too_many_arguments)]
fn factory_avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    f: &InspectorFactory,
    clock_playing: bool,
) -> f32 {
    let mut cur_y = y;
    if f.recipe.trim().is_empty() || !f.recipe_found {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.factory.no_recipe_u_nothing_to_make_copies_of"),
            ColorToken::Danger,
        );
    }
    if f.on_signal.trim().is_empty() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.factory.no_signal_u_this_factory_never_fires"),
            ColorToken::Warn,
        );
    } else if !clock_playing {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.factory.the_clock_is_stopped_u_copies_are_born_while_it_plays"),
            ColorToken::Text3,
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
        tr("panel.inspector.factory.factory"),
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
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.factory.editing_the_primary_selection_only"),
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
