//! Transform — Inspector section painter (split from sections.rs,
//! architecture_panel_loc_cap). Logic verbatim; behavior unchanged.

use super::*;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::tr;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_transform_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    // O cabeçalho da secção + a dobra do corpo. Ver [`paint_header_and_begin_fold`].
    let (header_h, fold) =
        match paint_header_and_begin_fold(scene, text_system, theme, hit_index, store, x, w, y) {
            Ok(v) => v,
            Err(collapsed_y) => return collapsed_y,
        };
    // No inner separator — the orchestrator (paint.rs) draws ONE
    // separator AFTER this section's content. Pre-2026-05-24 this fn
    // also painted a separator between title and params, which broke
    // the "separators go BETWEEN sections" canon (DIRETRIZ §5.2).
    let mut cur_y = y + header_h;

    // ⭐⭐⭐ **A GEOMETRIA DESTA SECÇÃO DEIXOU DE EXISTIR** — report do dono, 2026-09-15, com o
    // desenho: *«A disposição ficou diferente. VC tinha colocado x e y na mesma linha.
    // Position X/Y Caixa Caixa»*.
    //
    // ⛔⛔ Viviam aqui **oito** números só desta secção — a coluna do rótulo, o vão entre colunas
    // (`Spacing::Md`, `8`, contra os `3` da porta), a coluna da letra de eixo, o vão dela, o vão do
    // nome-por-cima, o piso do chip, a largura utilizável e a largura de um chip. Eles eram a
    // SEGUNDA resposta do app a *«como se dispõe uma linha de N campos»*, e à mesma largura de
    // painel as duas respostas desenhavam coisas diferentes: no dock dele (`369,74`) a coluna do
    // controlo mede `160,9`, as Âncoras pedem `2 × 72 + 3 = 147` e cabem, esta pedia
    // `2 × (14 + 72) + 8 = 180` e **não cabia**.
    //
    // ⇒ *«a mesma formatação» não se obtém com duas portas que concordam — obtém-se com UMA.* O
    // `X`/`Y` viaja no NOME, como nas Âncoras, e as quatro linhas são quatro chamadas da
    // `rows::fields_row`.

    // A unidade de ângulo: rótulo e passo. Mecanismo em [`labels_for`] e [`step_for`].
    let angle = current_display_angle();
    let (rot_label, skew_label) = labels_for(angle);
    let angle_step = step_for(angle);
    let unit = current_display_unit();
    let (pos_label, pos_step, pos_unit) = match unit {
        ph2d_editor_core::project::DisplayUnit::Meters => (
            tr("panel.inspector.transform.position_m"),
            0.01_f64, // LITERAL-PX-OK: passo em metros
            ph2d_editor_core::widget::Unit::Meters,
        ),
        ph2d_editor_core::project::DisplayUnit::Pixels => (
            tr("panel.inspector.transform.position_px"),
            1.0_f64,
            ph2d_editor_core::widget::Unit::Px,
        ),
    };
    let ang_unit = angle_unit(angle);
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        pos_label,
        &[ids::INSP_TRANSFORM_POS_X, ids::INSP_TRANSFORM_POS_Y],
        pos_step,
        Some(pos_unit),
        None,
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        rot_label,
        &[ids::INSP_TRANSFORM_ROT],
        angle_step,
        Some(ang_unit),
        None,
    );
    // ⭐ **A ESCALA e o CISALHAMENTO saíram para uma porta própria** (tecto de fn do painel,
    //    2026-09-15): as duas são o par `X`/`Y` de um FACTOR e de um ÂNGULO, e nenhuma delas lê a
    //    régua da POSIÇÃO. *O corte é por responsabilidade, nunca uma entrada na lista de folgas.*
    cur_y = paint_scale_and_skew(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        skew_label,
        angle_step,
        ang_unit,
    );
    cur_y += SECTION_BOTTOM_PAD_PX;

    fold.finish(store, scene, hit_index, cur_y)
}

/// ⭐ **As duas últimas linhas do Transform — a ESCALA e o CISALHAMENTO.**
///
/// Extraída do corpo de [`paint_transform_section`] pelo tecto de fn do painel (2026-09-15, quando a
/// unidade entrou em cada row). ⚠️ Elas andam juntas por uma razão e não por arrumação: **nenhuma
/// das duas lê a régua da POSIÇÃO** — a escala é um factor adimensional e o cisalhamento partilha a
/// régua do ÂNGULO com a rotação.
///
/// Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn paint_scale_and_skew(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    skew_label: &str,
    angle_step: f64,
    ang_unit: ph2d_editor_core::widget::Unit,
) -> f32 {
    const SCALE_STEP: f64 = 0.1; // LITERAL-PX-OK: passo de scrub de um FACTOR, não de pixels
    let cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.transform.scale"),
        &[ids::INSP_TRANSFORM_SCALE_X, ids::INSP_TRANSFORM_SCALE_Y],
        SCALE_STEP,
        None,
        None,
    );
    // Skew X/Y in degrees (ADR-0025-amendment-1). Authoring range is clamped to ±~89.4° at the
    // ECS-commit boundary; the slider itself is unbounded so over-typing snaps back on re-sync.
    super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        skew_label,
        &[ids::INSP_TRANSFORM_SKEW_X, ids::INSP_TRANSFORM_SKEW_Y],
        angle_step,
        Some(ang_unit),
        None,
    )
}

/// **O cabeçalho da secção Transform, e a dobra do corpo.**
///
/// # Por que é uma função própria
///
/// Corte por responsabilidade, cobrado pelo teto de LOC quando a unidade de ângulo
/// acrescentou linhas ao pintor: *desenhar o título, o ponto de cor, o botão de reset e
/// abrir (ou não) a dobra* é um assunto só, e não tem nada a ver com as quatro linhas de
/// campo que vêm depois. ⭐ É exactamente a forma que a mensagem do gate sugere — *"cada
/// ajudante recebe os mutáveis do quadro + `y`, e devolve `y`"*.
///
/// ⚠️ **O `Err` não é um erro** — é o caminho da secção RECOLHIDA, e carrega o `y` de
/// saída. Um `Option` perderia esse número, e o chamador teria de o recalcular a partir de
/// um `header_h` que só existe aqui dentro.
#[allow(clippy::too_many_arguments)]
fn paint_header_and_begin_fold(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> Result<(f32, SectionFold), f32> {
    // Canonical section header (ALL-CAPS + collapse chevron + color-dot
    // slot per UI canon — `docs/UI_Padrao/components/section_header.md`).
    // Reset is an ICON button (user feedback 2026-05-24).
    //
    // Order LEFT → RIGHT: title · reset icon · color dot. The color
    // dot lives at the FAR right (panel border), reset icon sits
    // just to its left. Pre-2026-05-24 the order was reversed; user
    // requested the swap.
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let reset_size = header_h; // square icon button matching header height
    let color_id = core_ids::INSP_LIVE_TRANSFORM_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default for unconfigured section accent
    // Header rect spans the FULL panel width so paint_section_header
    // anchors the color dot at the right edge (panel border).
    let header = section_header(
        store,
        core_ids::INSP_LIVE_TRANSFORM_SECTION,
        tr("panel.inspector.transform.transform"),
    )
    .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    // Reserve for the color dot at the right edge (≈ Md pad + 14 px
    // dot diameter) — the reset icon slots just to the LEFT of it.
    let color_slot_w = Spacing::Md.px() + 14.0; // LITERAL-PX-OK: color dot diameter (2 * radius 7)
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    let reset_rect = Rect::new(x + w - color_slot_w - reset_size, y, reset_size, reset_size);
    let reset_state = store.button_visual(ids::INSP_TRANSFORM_RESET);
    hit_index.register(ids::INSP_TRANSFORM_RESET, reset_rect);
    paint_icon_button(
        reset_rect,
        IconGlyph::Builtin(IconId::Reset),
        IconButtonStyle::Plain,
        reset_state,
        scene,
        theme,
    );
    // Collapsed → return after painting just the header. Body fields
    // (Position / Rotation / Scale) are skipped so the section
    // visually folds to a single row. Click on the header toggles
    // back via the dispatch (apply_click → toggle_collapsed).
    // ⚠️ **A DOBRA do corpo** — o escopo recorta a cena E o hit, e escala o `y` de saída, para
    //    que tudo o que está por baixo suba junto. Ver `SectionFold`.
    // ⚠️ **Pergunta o `t`, e NUNCA o `is_collapsed`:** ao clicar para fechar o flag semântico vira
    //    neste mesmo quadro enquanto o `t` ainda desce, então um corpo gateado no flag sumiria de
    //    repente por baixo de um chevron a rodar — as duas metades a discordar outra vez.
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_TRANSFORM_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return Err(y + header_h);
    };
    Ok((header_h, fold))
}

/// **Os rótulos das duas linhas de ângulo, na unidade activa.**
///
/// # ⛔ O report que a criou (Enio, 2026-08-30, com foto)
///
/// *"no inspector nao mudou as labels"*. A 1.ª entrega da unidade de ângulo trocava o
/// **valor** e deixava o rótulo fixo em `(°)`. ⚠️ **É pior do que não ter a feature:** a
/// caixa passava a mostrar `1.5708` debaixo de uma etiqueta a dizer GRAUS, que se lê como
/// *"o objecto rodou 1,5 grau"*. *Um rótulo que mente é pior que um rótulo ausente.*
///
/// ⚠️ Função pura de propósito: o painter resolve isto dentro do `paint`, onde um teste
/// não chega sem uma surface real. Tirá-la para aqui é o que a torna **gateável** — e o
/// gate existe porque a 1.ª entrega trocou o valor e deixou o rótulo a dizer `(°)`.
#[must_use]
fn labels_for(angle: ph2d_editor_core::project::DisplayAngle) -> (&'static str, &'static str) {
    // ⚠️⚠️ **Os dois rótulos deixaram de dizer a RÉGUA** (2026-09-15, ordem do dono: *«todos na
    //    caixa»*) — ela é hoje o SUFIXO do campo, e por isso os dois braços devolvem o mesmo texto.
    //    ⛔⛔ **E as duas chaves `_rad` FORAM APAGADAS**: eu escrevera aqui que ficavam *«para o dia
    //    em que se escreva outra palavra para radianos»*, e o
    //    `every_inspector_key_exists_on_both_sides` desmentiu-o na corrida seguinte — *uma chave que
    //    ninguém usa é dívida, não um sítio reservado*. A função fica porque o `angle` ainda decide
    //    o PASSO (`step_for`).
    let _ = angle;
    (
        tr("panel.inspector.transform.rotation"),
        tr("panel.inspector.transform.skew"),
    )
}

/// ⭐ **A unidade do ÂNGULO activo — a régua que o rótulo deixou de dizer.**
fn angle_unit(angle: ph2d_editor_core::project::DisplayAngle) -> ph2d_editor_core::widget::Unit {
    match angle {
        ph2d_editor_core::project::DisplayAngle::Degrees => ph2d_editor_core::widget::Unit::Degrees,
        ph2d_editor_core::project::DisplayAngle::Radians => ph2d_editor_core::widget::Unit::Radians,
    }
}

/// **O passo do stepper na unidade activa — quanto vale UM GRAU nela.**
///
/// ⭐ Derivado, não escolhido: em graus dá `1.0` (o valor que estava fixo aqui, logo o
/// caminho de omissão é byte-idêntico) e em radianos dá `0,01745`. ⛔ Deixá-lo em `1.0`
/// faria cada clique do stepper saltar **57°** em radianos.
#[must_use]
fn step_for(angle: ph2d_editor_core::project::DisplayAngle) -> f64 {
    f64::from(angle.from_radians(1.0_f32.to_radians()))
}

#[cfg(test)]
mod angle_unit_tests {
    use super::{angle_unit, labels_for, step_for};
    use ph2d_editor_core::project::DisplayAngle;

    /// ⛔⛔ **O ROTULO segue a unidade** — o report do Enio de 2026-08-30, com foto:
    /// *"no inspector nao mudou as labels"*.
    ///
    /// A 1.a entrega da unidade de angulo trocava o VALOR e deixava o rotulo fixo em
    /// `(°)`. ⚠️ **E' pior do que nao ter a feature:** a caixa passava a mostrar `1.5708`
    /// debaixo de uma etiqueta que dizia GRAUS, que se le^ como *"o objecto rodou 1,5
    /// grau"*. *Um rotulo que mente e' pior que um rotulo ausente.*
    ///
    /// ⚠️ A regua vive aqui como uma funcao pura para poder ser gateada -- o painter
    /// resolve o par no `paint`, onde um teste nao chega sem uma surface real.
    /// ⚠️⚠️ **A LEI MUDOU DE SÍTIO em 2026-09-15, e não de conteúdo.** O dono ordenou *«todos na
    /// caixa»* (foto de `Speed (°/s)`), logo a régua activa deixou de viver no RÓTULO e passa a ser
    /// o **sufixo do campo**. ⛔ A asserção antiga comparava os dois rótulos e reprovaria sobre o
    /// desenho CERTO — *um gate escrito sobre ONDE a lei aparecia é um gate que reprova quando ela
    /// se muda*. O que ele defendia — **a régua chega ao artista, e as duas discordam** — é o que
    /// fica afirmado, agora sobre a porta que a entrega.
    #[test]
    fn the_row_labels_follow_the_active_angle_unit() {
        assert_eq!(
            angle_unit(DisplayAngle::Degrees),
            ph2d_editor_core::widget::Unit::Degrees
        );
        assert_eq!(
            angle_unit(DisplayAngle::Radians),
            ph2d_editor_core::widget::Unit::Radians
        );
        // ⛔ O controlo: as duas tem de DISCORDAR, senao a funcao e' decorativa.
        assert_ne!(
            angle_unit(DisplayAngle::Degrees),
            angle_unit(DisplayAngle::Radians)
        );
        // ⛔⛔ E a METADE que o report de 2026-08-30 pagou: o rotulo ja' NAO diz a regua, logo ele
        //    tem de ser o MESMO nos dois — senao ha' duas respostas a' mesma pergunta.
        assert_eq!(
            labels_for(DisplayAngle::Degrees),
            labels_for(DisplayAngle::Radians),
            "o rotulo voltou a dizer a regua — ela vive no campo desde 2026-09-15"
        );
    }

    /// ⭐⭐ **O PASSO segue a unidade, e era o SEGUNDO defeito do mesmo report.**
    ///
    /// O passo estava fixo em `1.0`. Em graus e' um grau; em radianos **um clique do
    /// stepper saltaria 57°** -- o controlo ficaria inutilizavel exactamente no modo que
    /// a feature acabou de abrir. *Trocar a regua sem trocar o passo entrega um controlo
    /// pior do que o de antes.*
    ///
    /// ⚠️ **O passo e' DERIVADO: e' quanto vale UM GRAU na unidade activa.** Nao e' um
    /// numero escolhido -- e' a mesma lei que a irma do comprimento ja' obedece sem a
    /// dizer (`0,01 m` e `1 px` sao o mesmo deslocamento a` escala de fabrica).
    #[test]
    fn the_stepper_moves_the_same_amount_in_either_unit() {
        let deg = step_for(DisplayAngle::Degrees);
        let rad = step_for(DisplayAngle::Radians);
        // ⭐ Em graus tem de dar exactamente 1.0 -- e' o valor que estava fixo aqui, logo
        // o caminho de omissao fica byte-identico.
        assert!(
            (deg - 1.0).abs() < 1e-9,
            "o passo em graus tem de continuar 1.0, e deu {deg}"
        );
        // E o de radianos tem de ser o MESMO angulo fisico.
        assert!(
            (rad - f64::from(1.0_f32.to_radians())).abs() < 1e-9,
            "um passo em radianos tem de valer um grau, e deu {rad}"
        );
        assert!(
            rad < deg,
            "o numero em radianos e' menor -- a unidade e' maior"
        );
    }
}
