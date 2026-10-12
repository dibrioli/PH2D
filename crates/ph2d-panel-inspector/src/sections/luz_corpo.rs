//! ⭐ **Os cartões do CORPO na luz** — «Light Occluder» (`BloqueiaLuz`: a opacidade) e «Unlit»
//! (`NaoRecebeLuz`: não tem campos). Componente = ícone + cartão: sem eles o «Add Component»
//! punha o componente no objeto e o Inspector não mostrava nada — nem para o ver, nem para o
//! editar, nem para o retirar.

use super::*;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::tr;

/// O cabeçalho com a dobra. `Err(y)` = o cartão está dobrado; `y` é o de baixo do cabeçalho.
fn cabecalho(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    (section, titulo): (NodeId, &str),
) -> Result<(SectionFold, f32), f32> {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(store, section, titulo);
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let below = y + header_h;
    SectionFold::begin(store, section, x, w, below, scene, hit_index)
        .map(|f| (f, below))
        .ok_or(below)
}

/// **LIGHT OCCLUDER** — a opacidade e o botão de retirar.
pub(crate) fn paint_oclusor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
) -> f32 {
    let (fold, mut cur_y) = match cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        (x, w, y),
        (
            core_ids::INSP_LIVE_LUZ_OCLUSOR_SECTION,
            tr("component.bloqueia_luz.name"),
        ),
    ) {
        Ok(v) => v,
        Err(below) => return below,
    };
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.luz.opacity")],
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
        tr("panel.inspector.luz.opacity"),
        &[ids::INSP_LUZ_OPACIDADE],
        0.01, // LITERAL-PX-OK: passo de scrub da fração bloqueada (0..1)
        None,
        seccao,
    );
    let rotulo = tr("panel.inspector.luz.x_remove_occluder");
    let rm = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, cur_y, rotulo);
    hit_index.register(ids::INSP_LUZ_RETIRAR_OCLUSOR, rm);
    paint_button(
        &Button::new(ids::INSP_LUZ_RETIRAR_OCLUSOR, rotulo)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_LUZ_RETIRAR_OCLUSOR)),
        rm,
        scene,
        text_system,
        theme,
    );
    cur_y = ph2d_editor_core::property_row::abaixo_do_botao(rm);
    fold.finish(store, scene, hit_index, cur_y)
}

/// **UNLIT** — a frase do que ele faz e o botão de retirar.
pub(crate) fn paint_sem_luz(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
) -> f32 {
    let (fold, mut cur_y) = match cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        (x, w, y),
        (
            core_ids::INSP_LIVE_LUZ_SEM_LUZ_SECTION,
            tr("component.nao_recebe_luz.name"),
        ),
    ) {
        Ok(v) => v,
        Err(below) => return below,
    };
    cur_y = super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        cur_y,
        tr("panel.inspector.luz.unlit_explained"),
        ColorToken::Text2,
    );
    let rotulo = tr("panel.inspector.luz.x_remove_unlit");
    let rm = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, cur_y, rotulo);
    hit_index.register(ids::INSP_LUZ_RETIRAR_SEM_LUZ, rm);
    paint_button(
        &Button::new(ids::INSP_LUZ_RETIRAR_SEM_LUZ, rotulo)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_LUZ_RETIRAR_SEM_LUZ)),
        rm,
        scene,
        text_system,
        theme,
    );
    cur_y = ph2d_editor_core::property_row::abaixo_do_botao(rm);
    fold.finish(store, scene, hit_index, cur_y)
}
