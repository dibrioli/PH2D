//! O corpo do painel. Percorre o **retrato** que o shell publicou — a mesma lista que o `event`
//! consulta —, então o que é pintado e o que despacha não podem discordar.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text_block, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_surface, paint_panel_title,
    panel_drag_handle_rect,
};
use ph2d_editor_core::widget::{NUMBER_INPUT_MIN_W_PX, paint_slider_with_chip_layout_adaptive};
use ph2d_editor_core::zones::Rect;
use ph2d_field::RadiusBound;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

use crate::state::{self, Model3dPanelState, RadiusRow};
use crate::{Model3dPanel, populate::MAX_MODES, populate::MAX_ROWS};

/// A goteira do rótulo. Os rótulos aqui são o **tipo do nó** ("Union", "Cylinder"), não uma frase.
const LABEL_COL_W: f32 = 72.0; // LITERAL-PX-OK: panel grid metric (label gutter width)

/// Quanto um nível de profundidade recua a linha — o mesmo gesto que a Hierarquia faz.
const ROW_INDENT_PX: f32 = 10.0; // LITERAL-PX-OK: panel grid metric (tree indent per level)

/// O piso da largura de uma linha recuada: abaixo disto o slider deixa de ser arrastável e o
/// controle passa a existir sem ser usável, que é pior do que não recuar.
const MIN_ROW_W_PX: f32 = 120.0; // LITERAL-PX-OK: panel grid metric (minimum usable row width)

/// Casas decimais do raio.
///
/// ⚠️ **Três, e não duas.** Um raio de CAD é pequeno em relação à peça — os das cenas de smoke vão
/// de 0,02 a 0,12 — e duas casas transformariam metade da faixa útil no mesmo número na tela.
const RADIUS_DECIMALS: usize = 3;

pub(crate) fn paint(_state: &mut Model3dPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(Model3dPanel::ID) {
        // Limpeza simétrica do rect: sem isto o `panel_at` continuaria a devolver este painel
        // depois de ele fechar, e a roda do canvas ficaria a rolar um painel invisível.
        ctx.host.store_mut().clear_panel_rect(ids::MODEL3D_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    let snapshot = state::current();

    ctx.host
        .store_mut()
        .set_panel_rect(ids::MODEL3D_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        ctx.host
            .hit_index_mut()
            .register(ids::INSP_DRAG_HANDLE, drag_rect);
    }

    let title_size = paint_panel_title(
        rect,
        tr("panel.model3d.title"),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::MODEL3D_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let x = rect.x + PANEL_HEAD_PAD;
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);

    ctx.scene
        .push_clip(&rect_to_vello(Rect::new(rect.x, body_top, rect.w, body_h)));
    let mut y = body_top;
    // ⭐ **O seletor do verbo, no topo** — é a porta que se encontra sem saber que existe. As
    // teclas `G`/`R`/`S` são a outra, e são para quem já sabe.
    y = paint_modes(ctx, &snapshot, x, w, y);
    if snapshot.rows.is_empty() {
        y = paint_note(ctx, tr("panel.model3d.empty"), x, w, y);
    }
    // ⚠️ **Corta na família, e o rodapé DIZ que cortou** — ver `MAX_ROWS`. Uma linha além dela
    // ficaria sem controle registado: pintada e morta sob o rato, que é a falha de paridade de
    // fiação na sua forma mais cara.
    for (slot, row) in snapshot.rows.iter().enumerate().take(MAX_ROWS) {
        y = paint_row(ctx, row, slot as u32, x, w, y);
    }
    y = paint_footer(ctx, &snapshot, x, w, y);
    state::set_last_content_h(y - body_top + PANEL_HEAD_PAD);
    ctx.scene.pop_layer();
}

/// ⭐ **O seletor de verbo do gizmo**: mover · rodar · escalar. Devolve o **y seguinte**.
///
/// ⚠️ **Grupo ADAPTATIVO**, e não três botões de largura fixa: num painel estreito ou num idioma
/// mais comprido, três rótulos lado a lado deixam de caber, e a versão fixa quebraria o texto
/// DENTRO do botão — o artefato que a casa já registou e curou com este widget.
fn paint_modes(ctx: &mut PaintCtx, snap: &state::ModelSnapshot, x: f32, w: f32, y: f32) -> f32 {
    if snap.modes.is_empty() {
        return y;
    }
    let theme = ctx.host.theme();
    let labels: Vec<(&str, bool, ph2d_a11y::NodeId)> = snap
        .modes
        .iter()
        .take(MAX_MODES as usize)
        .enumerate()
        .map(|(i, m)| (tr(m.key), m.active, ids::model3d_mode_button(i as u32)))
        .collect();
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let used = paint_segmented_group_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        &labels,
        ctx.scene,
        ctx.text_system,
        theme,
        store,
        hit_index,
    );
    y + used + Spacing::Sm.px()
}

/// Uma linha: rótulo do tipo do nó + slider + campo numérico. Devolve o **y seguinte**.
///
/// ⚠️ **DEVOLVE O Y SEGUINTE, e o helper que ela chama devolve a ALTURA USADA.** As duas convenções
/// existem no mesmo repo e a confusão entre elas foi um smoke reprovado (Enio, 2026-08-19: *"o
/// painel apresenta apenas um slider"*): com `y = paint_row(...)`, a segunda linha ia parar em
/// `y = 28` **absoluto** — dentro do título, fora do recorte — e as três seguintes com ela. O painel
/// mostrava uma linha e o modelo parecia ter encolhido.
///
/// ⭐ Neste arquivo **toda** função de pintura devolve o y seguinte. Uma convenção por arquivo, dita
/// aqui: misturar as duas é como o erro entrou.
fn paint_row(ctx: &mut PaintCtx, row: &RadiusRow, slot: u32, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    // ⚠️ **O id vem da POSIÇÃO da linha, nunca da entidade.** O `populate` corre antes de a peça
    // existir e cunha a família às cegas (ver `MAX_ROWS`); os bits de uma entidade não cabem numa
    // família de 64. A entidade viaja no intent, que é onde ela importa.
    let slider = ids::model3d_radius_slider(slot);
    let chip = ids::model3d_radius_chip(slot);
    let top = row.bound.value().max(f32::MIN_POSITIVE);
    // O aninhamento da árvore, visível: um filho recua. Sem isto o painel lista quatro linhas
    // planas de uma peça que a Hierarquia mostra em dois níveis, e as duas vistas da mesma coisa
    // discordam à vista.
    let indent = f32::from(row.depth) * ROW_INDENT_PX;
    let (x, w) = (x + indent, (w - indent).max(MIN_ROW_W_PX));

    // ⚠️ **A faixa real é escrita por LINHA, todo quadro.** O teto de um raio é do nó — a caixa
    // aceita menos do que o cilindro —, e o par slider↔campo foi ligado em 0..1 no `populate`
    // justamente porque a escala não era conhecida lá.
    //
    // ⚠️ Sem `set_number_range` o campo deriva o passo do arrasto do TEXTO do buffer e escorrega
    // ~50 unidades por pixel (a nota que o painel de aquarela deixou: digitar continua a funcionar,
    // o que esconde o defeito).
    {
        let store = ctx.host.store_mut();
        store.link_slider_number_mapped(slider, chip, top, 0.0);
        // LITERAL-PX-OK: não é métrica de design — é a granularidade do arrasto, em
        // CENTÉSIMOS DO CURSO. Ela é relativa ao teto da linha de propósito: um passo
        // absoluto seria grosseiro num raio de 0,02 e fino demais num de 2,0.
        const STEPS_ACROSS_THE_RANGE: f64 = 100.0; // LITERAL-PX-OK: granularidade de arrasto, não métrica de design
        store.set_number_range(
            chip,
            0.0,
            f64::from(top),
            f64::from(top) / STEPS_ACROSS_THE_RANGE,
        );
    }

    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    // A verdade é o documento; a posição guardada é só o que o arrasto deixou para trás. Semear a
    // trilha a partir do valor todo quadro é o que mantém o controle honesto quando o raio muda de
    // outro lado — um desfazer, um arquivo aberto, uma segunda linha.
    let track = (row.radius / top).clamp(0.0, 1.0);
    let display = f64::from(row.radius);
    let text = format!("{display:.RADIUS_DECIMALS$}");

    let used = paint_slider_with_chip_layout_adaptive(
        Rect::new(x, y, w, ROW_H_PX),
        tr(row.kind_key),
        track,
        display,
        Some(&text),
        slider,
        chip,
        LABEL_COL_W,
        NUMBER_INPUT_MIN_W_PX,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y + used + Spacing::Xs.px()
}

/// Texto puro — um fato, não um controle. Sem hit-index: uma affordance que ele não pode honrar
/// seria pior do que nenhuma.
fn paint_note(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
    let font = TypeToken::Sm.px();
    let theme = ctx.host.theme();
    let used = paint_text_block(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    // O avanço é MEDIDO: uma nota quebra em duas linhas num painel estreito ou num idioma mais
    // comprido, e um avanço fixo escreveria a linha seguinte por cima dela.
    y + (ROW_H_PX - font).mul_add(0.5, used).max(ROW_H_PX) + Spacing::Xs.px()
}

/// O rodapé: quantos nós, e **quanto custou o último quadro**.
///
/// ⭐ O custo fica aqui, e não só no terminal, porque quem mexe num raio é quem paga o traçado
/// seguinte — e um painel que esconde isso deixa a lentidão parecer um defeito em vez de uma conta.
fn paint_footer(ctx: &mut PaintCtx, snap: &state::ModelSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let hidden = snap.rows.len().saturating_sub(MAX_ROWS);
    let mut line = format!(
        "{}: {} · {} {:.1} ms",
        tr("panel.model3d.nodes"),
        snap.node_count,
        tr("panel.model3d.trace_cost"),
        snap.last_trace_ms
    );
    if hidden > 0 {
        // ⛔ **Nunca em silêncio.** Se a família de ids não chegou para todos os nós, o artista tem
        // de saber quantos ficaram sem controle — senão a conclusão dele é que o modelo encolheu.
        line.push_str(&format!(" (+{hidden})"));
    }
    paint_note(ctx, &line, x, w, y + Spacing::Xs.px())
}

/// A parte de um limite que a UI precisa de saber: o topo, e se ele é parede.
///
/// ⚠️ Existe para o dia em que o painel **desenhar** a diferença (uma parede e uma sugestão não se
/// pintam igual). Hoje as duas viram a mesma faixa, e isto é o registo de que a distinção existe no
/// documento e ainda não chegou ao pixel.
#[allow(dead_code)]
fn bound_is_wall(b: RadiusBound) -> bool {
    matches!(b, RadiusBound::Hard(_))
}
