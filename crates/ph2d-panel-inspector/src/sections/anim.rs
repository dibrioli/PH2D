//! **A §11 Animation** — a última das doze seções do Inspector (spec
//! [`08_animation_inline.md`](../../../../docs/Sprite_projeto/08_animation_inline.md) §8.7).
//!
//! # As duas metades, e os dois donos
//!
//! - **O TOCADOR** (`SpriteAnimator`): o que está a tocar, a que velocidade, com que direção.
//! - **A BIBLIOTECA** (`SpriteAnimations`): as animações desta sprite — nome, intervalo, ritmo.
//!
//! São dois componentes distintos no motor, e a seção di-lo com dois rótulos. É a lição que a §12
//! pagou ao pôr as âncoras deste objeto e a âncora do pai na mesma lista sem dizer de quem eram.
//!
//! # ⚠️ Clicar numa linha ESCOLHE a animação — e isso difere da §12
//!
//! Na §12, clicar numa linha muda só a ficha aberta e não vai ao barramento. Aqui vai: numa
//! biblioteca de animações, *a que se vê* e *a que toca* serem a mesma é o que o artista espera
//! (o `AnimationPlayer` do Godot faz assim), e separá-las pediria um seletor com as mesmas
//! entradas da lista logo abaixo dele — 65 ids e uma máquina de popover para duplicar uma lista.
//!
//! # ⛔ Sem tocador, a seção mostra UM botão
//!
//! Pintar os controlos de reprodução sobre uma sprite que não tem estado de reprodução seria
//! oferecer knobs que não vão a lado nenhum — o defeito que o `Simple` do 9-slice era.

use super::*;
use ph2d_editor_core::screens::hero::InspectorAnimInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

/// Espessura da barra de frames.
///
/// ⚠️ **Subiu de 6 para 10 px quando ela passou a arrastar-se**, e o número não é gosto: o
/// retângulo de acerto **é** a trilha (o despachante deriva o valor de `rect.x`/`rect.w`), então a
/// altura da trilha é o alvo do dedo. Seis pixels é uma linha de leitura, não um alvo — e uma
/// barra que se agarra tem de **parecer** que se agarra.
const BAR_H: f32 = 10.0; // LITERAL-PX-OK: altura de trilha de slider, é o alvo do ponteiro

/// Um segmentado de N botões com o índice `sel` aceso. Devolve o `y` seguinte.
///
/// ⛔⛔ **O nome estava POR CIMA e o grupo a toda a largura** (`row_y = y + font + Xs`) — a forma
/// que o dono reprovou duas vezes. ⇒ a porta [`ph2d_editor_core::property_row::paint_choice_row`],
/// com a coluna da secção (`anim_rows::seccao_da_animacao`, que já conta estes dois nomes).
#[allow(clippy::too_many_arguments)]
fn segmented_row<'a>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: impl Into<ph2d_editor_core::widget::key_mark::Rotulo<'a>>,
    ids_: &[NodeId],
    labels: &[&str],
    sel: usize,
) -> f32 {
    // ⚠️ **A selecção vem do SNAPSHOT**, não do store: ler dali qual está aceso faria o realce
    //    sobreviver à troca de sprite.
    let segments: Vec<(&str, bool, NodeId)> = ids_
        .iter()
        .zip(labels.iter())
        .enumerate()
        .map(|(i, (&id, &text))| (text, i == sel, id))
        .collect();
    let sec = super::anim_rows::seccao_da_animacao(text_system);
    ph2d_editor_core::property_row::paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        label,
        &segments,
        sec,
    )
}

/// **As duas SUBSTITUIÇÕES do tocador** — a direcção e o laço que esta reprodução impõe por cima
/// do que a animação declara.
///
/// ⚠️ **Sai do [`player_block`] por TETO de função** (200), estourado em 2026-09-15. ⛔ *A cura de
/// um teto é o corte por responsabilidade* — e as duas pertencem uma à outra: são a mesma pergunta
/// (*«o que o TOCADOR manda, contra o que a animação diz»*) feita a dois canais, e as duas nascem
/// em `Inherit`.
#[allow(clippy::too_many_arguments)]
fn override_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
) -> f32 {
    let mut cur_y = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        TextKey::new("component.field.sprite_animator.4"),
        &ids::INSP_ANIM_DIR_OVERRIDE,
        &[
            tr("panel.inspector.animation.inherit"),
            tr("panel.inspector.animation.fwd"),
            tr("panel.inspector.animation.rev"),
            tr("panel.inspector.animation.pp"),
            tr("panel.inspector.animation.pp_rev"),
        ],
        usize::from(info.direction_override_tag),
    );
    cur_y = segmented_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        TextKey::new("component.field.sprite_animator.5"),
        &ids::INSP_ANIM_LOOP_OVERRIDE,
        &[
            tr("panel.inspector.animation.inherit"),
            tr("panel.inspector.animation.on"),
            tr("panel.inspector.animation.off"),
        ],
        usize::from(info.loop_override_tag),
    );
    cur_y
}

/// Um título de grupo do cartão («Playback», o nome da animação escolhida): `SemiBold`, `Text1`, sem
/// chevron (o grupo não dobra). Devolve o `y` seguinte.
pub(super) fn subtitle(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    text: &str,
) -> f32 {
    let font = TypeToken::Sm.px();
    ph2d_editor_core::paint::paint_text_title(
        text_system,
        scene,
        text,
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text1, theme),
    );
    y + font + ph2d_tokens::control_gap_px()
}

/// Um botão-ícone quadrado de uma fileira; o rótulo fica como nome acessível.
#[allow(clippy::too_many_arguments)]
pub(super) fn icon_button(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    rect: Rect,
    id: NodeId,
    icon: IconId,
    label: &str,
) {
    hit_index.register(id, rect);
    paint_button(
        &Button::new(id, label)
            .kind(ButtonKind::IconOnly { icon })
            .visual(store.button_visual(id)),
        rect,
        scene,
        text_system,
        theme,
    );
}

/// **«Playback»** — sempre à vista por baixo da lista (plano 07, desenho aprovado 25/10): numa fileira
/// ⏮ · ▶/⏸ · a barra · «Frame 3 / 5»; por baixo a velocidade e, SÓ quando o quadro no ecrã tem tempo
/// próprio, «This frame». O ▶/⏸ é o facto «a tocar» (a caixa «Playing» saiu). Devolve o `y` seguinte.
///
/// ⚠️ A barra é um `Slider` cujo rectângulo de acerto É a trilha (o despacho deriva o valor de
/// `rect.x`/`rect.w`), e a posição vem do SNAPSHOT (`scrub_position`), nunca do store.
#[allow(clippy::too_many_arguments)]
fn player_block(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
) -> f32 {
    let font = TypeToken::Sm.px();
    let gap = ph2d_tokens::control_gap_px();
    let row_h = ph2d_tokens::ROW_H_PX;
    let sec = super::anim_rows::seccao_da_animacao(text_system);
    let mut cur_y = subtitle(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        tr("panel.inspector.anim_card.playback"),
    );
    icon_button(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        Rect::new(x, cur_y, row_h, row_h),
        ids::INSP_ANIM_REWIND,
        IconId::SkipBack,
        tr("panel.inspector.animation.rewind"),
    );
    let (icon, label) = if info.playing {
        (IconId::Pause, tr("panel.inspector.anim_card.pause"))
    } else {
        (IconId::Play, tr("panel.inspector.anim_card.play"))
    };
    icon_button(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        Rect::new(x + row_h + gap, cur_y, row_h, row_h),
        ids::INSP_ANIM_PLAYING,
        icon,
        label,
    );
    if let Some((step, span)) = info.progress() {
        let text = tr_with(
            "panel.inspector.animation.frame_of",
            &[("n", &(step + 1)), ("span", &span)],
        );
        // O rótulo mede-se (nunca se elide); a barra fica com o que sobra entre os botões e ele.
        let text_w =
            ph2d_editor_core::paint::label_column_width(text_system, font, [text.as_str()]);
        let text_x = x + w - text_w;
        paint_text(
            text_system,
            scene,
            &text,
            text_x,
            cur_y + (row_h - font) * 0.5,
            font,
            text_w + 1.0, // LITERAL-PX-OK: folga de arredondamento da medida
            resolve(ColorToken::Text2, theme),
        );
        let bar_x = x + 2.0 * (row_h + gap);
        let track = Rect::new(
            bar_x,
            cur_y + (row_h - BAR_H) * 0.5,
            (text_x - gap - bar_x).max(0.0),
            BAR_H,
        );
        hit_index.register(ids::INSP_ANIM_FRAME_SCRUB, track);
        ph2d_editor_core::widget::paint_slider_track(
            track,
            info.scrub_position().unwrap_or(0.0),
            ph2d_editor_core::widget::SliderOrientation::Horizontal,
            store.slider_visual(ids::INSP_ANIM_FRAME_SCRUB),
            scene,
            theme,
        );
    }
    cur_y += row_h + gap;
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        TextKey::new("component.field.sprite_animator.3"),
        &[ids::INSP_ANIM_SPEED],
        0.1, // LITERAL-PX-OK: passo de scrub em MÚLTIPLOS de velocidade, não em pixels
        None,
        sec,
    );
    // A duração DESTE quadro (spec §8.12) — só quando ele a tem (o editor é onde se dá tempo a um
    // quadro); colada à barra, que é o selector do quadro.
    if info
        .this_frame_ms()
        .is_some_and(|ms| ms > 0 && Some(ms) != info.current_frame_ms())
    {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.animation.this_frame_ms_0_use"),
            &[ids::INSP_ANIM_FRAME_MS_THIS],
            10.0, // LITERAL-PX-OK: passo de scrub em MILISSEGUNDOS, não em pixels
            Some(ph2d_editor_core::widget::Unit::Milliseconds),
            sec,
        );
    }
    if info.current_dangling() {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.animation.this_sprite_has_no_animation"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Danger, theme),
        );
        cur_y += font + gap;
    }
    cur_y
}

/// **«Advanced»** — nasce FECHADO: os campos finos da animação escolhida e as substituições do
/// tocador. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn advanced_group(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    selected: usize,
) -> f32 {
    let mut cur_y = y;
    let adv = super::rows::grupo_dobravel(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_ANIM_ADVANCED,
        tr("panel.inspector.anim_card.advanced"),
    );
    if adv.aberto() {
        cur_y = super::anim_rows::paint_advanced(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            adv.x,
            adv.w,
            adv.y,
            info,
            selected,
        );
        cur_y = override_rows(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            adv.x,
            adv.w,
            cur_y,
            info,
        );
    }
    adv.fecha(store, scene, hit_index, cur_y)
}

/// Pinta a §11 e devolve o `y` a seguir a ela.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_anim_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorAnimInfo,
    selected: usize,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let title = if info.rows.is_empty() {
        String::from(tr("component.field.sprite_animator.0"))
    } else {
        tr_with(
            "panel.inspector.animation.title_count",
            &[("n", &info.rows.len())],
        )
    };
    let header = section_header(store, core_ids::INSP_LIVE_ANIM_SECTION, &title);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_ANIM_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    let font = TypeToken::Sm.px();

    // ⚠️ **A SELEÇÃO MÚLTIPLA tem de se dizer**, e esta é a única seção do Inspector cujo silêncio
    // custa trabalho perdido: as edições da §11 **não** se espalham (o índice que elas carregam só
    // significa alguma coisa na biblioteca da primária — espalhá-lo escreveria na animação errada
    // de todas as outras). Sem esta linha, marcar cinco goblins e renomear uma animação muda
    // **um**, em silêncio.
    //
    // ⚠️ Ela vem ANTES de tudo, inclusive da face vazia: a pergunta *«em quem é que isto pega?»* é
    // anterior a qualquer controlo.
    // ⛔ Sem tocador, UM botão — e mais nada. Ver o doc do módulo.
    if !info.player_present {
        paint_text(
            text_system,
            scene,
            tr("panel.inspector.animation.this_sprite_does_not_play"),
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
        let add = ph2d_editor_core::property_row::caixa_do_botao(
            text_system,
            x,
            w,
            cur_y,
            tr("panel.inspector.animation.plus_add_animator"),
        );
        hit_index.register(ids::INSP_ANIM_ADD_PLAYER, add);
        paint_button(
            &Button::new(
                ids::INSP_ANIM_ADD_PLAYER,
                tr("panel.inspector.animation.plus_add_animator"),
            )
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_ANIM_ADD_PLAYER)),
            add,
            scene,
            text_system,
            theme,
        );
        return fold.finish(
            store,
            scene,
            hit_index,
            ph2d_editor_core::property_row::abaixo_do_botao(add) + SECTION_BOTTOM_PAD_PX,
        );
    }

    // ⭐ O CARTÃO (plano GameEngineFuncional 07, desenho aprovado 25/10), de cima para baixo: ver ·
    // a lista · «Playback» · os verbos da biblioteca · «Edit Animations…» · a animação escolhida ·
    // «Who Picks» · a grelha · «Advanced» (nasce FECHADO). Nada se perdeu do que a §11 tinha.
    cur_y = super::anim_card::paint_preview(scene, x, w, cur_y, info);
    cur_y = super::anim_card::paint_list(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        selected,
    );
    cur_y = player_block(
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
    cur_y = super::anim_rows::paint_add_remove(
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
    cur_y = super::anim_card::button_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        ids::INSP_ANIM_OPEN_EDITOR,
        tr("panel.inspector.anim_card.open_editor"),
    );
    cur_y = super::anim_rows::paint_selected_editor(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        selected,
    );
    let sec = super::anim_rows::seccao_da_animacao(text_system);
    cur_y = super::anim_card::paint_chooser(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        sec,
    );
    cur_y = super::paint_sheet_grid_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info.card.hframes,
        info.card.vframes,
        false,
        sec,
    );
    cur_y = advanced_group(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
        selected,
    );
    fold.finish(store, scene, hit_index, cur_y + SECTION_BOTTOM_PAD_PX)
}
