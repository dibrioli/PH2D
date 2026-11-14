//! ⭐⭐ **A FILA DE FERRAMENTAS** — os chips do trilho, na horizontal, por cima da área de desenho.
//!
//! Enio, 2026-08-30: *«ainda temos os botões da lateral»* — o trilho vertical saiu, e é aqui que
//! eles voltam, no modelo do Godot: uma fila por cima do canvas, não uma coluna a comer largura.
//!
//! # ⭐ A fila é uma REGIÃO da área, e essa é a diferença que importa
//!
//! Ela sai de [`HeroLayout::tool_bar`], que é cortado da **área de desenho** — entre as colunas —,
//! não da janela. A régua começa por baixo dela, e nenhuma das duas pode tapar a outra porque não
//! partilham coordenada (D5). ⛔ O trilho antigo ancorava em `x = 0` e tapava **86,8 %** da régua
//! da esquerda; uma fila que atravessasse o ecrã faria o mesmo às colunas.
//!
//! # ⚠️ A LISTA é a mesma do trilho, e tem de ser
//!
//! [`super::left_rail::rail_entries`] é a fonte das duas disposições. Uma segunda lista aqui seria
//! a tabela paralela: um verbo novo apareceria numa e não na outra, conforme quem se lembrasse —
//! e o gate anti-botão-morto (`every_painted_rail_button_is_dispatched`) percorre aquela.
//!
//! # ⚠️ E a GEOMETRIA é a mesma porta
//!
//! [`crate::widget::entry_rects`] responde *«onde cai cada entrada?»* nos dois eixos, e é ela que
//! o pintor e o registo de hit perguntam. Enquanto ela não existia, a mesma aritmética estava
//! escrita **três** vezes — e um pintor horizontal com um hit vertical compilaria e passaria a
//! suíte inteira.

use super::HeroLayout;
use super::left_rail::{PAINTER_MASK_SUBS, PAINTER_SHAPES, rail_entries, tool_entry};
use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::{
    LABEL_TO_CHIP_GAP_PX, LABEL_VISUAL_EXTENT_PX, RailAxis, RailButtonSize, ToolRail, entry_rects,
    paint_tool_rail_axis,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// **A altura da fila** — o rótulo, a folga, o chip, e o respiro em cima e em baixo.
///
/// ⚠️ **Derivada, não escolhida**: cada parcela é a mesma constante que a coluna usa para a mesma
/// coisa. Um número aqui faria a fila e a coluna terem chips de tamanhos diferentes no dia em que
/// alguém mexesse no preset.
#[must_use]
pub fn tool_bar_h(size: RailButtonSize, lines: usize) -> f32 {
    let lines = lines.max(1) as f32;
    Spacing::Xxs.px() * 2.0
        + lines * (LABEL_VISUAL_EXTENT_PX + LABEL_TO_CHIP_GAP_PX + size.chip_px())
        + (lines - 1.0) * Spacing::Xs.px()
}

// ⛔⛔ **`tool_bar_lines` MORREU em 2026-08-31.** Ela respondia *«de quantas linhas a fila
// precisa?»* — uma pergunta que só existe enquanto a faixa PODE crescer, e ela já não pode. *Uma
// função que só sobrevive nos próprios testes é a última prova de que a capacidade que ela servia
// foi retirada.*
//
// ⚠️ E com ela caiu a **segunda passagem** do `frame_layout` (ver lá): a altura da faixa deixou de
// depender da largura da área.

/// ⭐⭐⭐ **O QUE CABE NUMA LINHA, e o que fica atrás do `⋯`** — a porta ÚNICA do transbordo.
///
/// Devolve `(a fila que se pinta, o que transbordou)`. A fila devolvida **já leva o chip `⋯` no
/// fim** quando há transbordo, e é por isso que o pintor, o registo de hit e o menu não podem
/// discordar: os três leem esta função.
///
/// # ⛔⛔ Por que a faixa não cresce
///
/// > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
///
/// Ela crescia: `54 → 108 px` no iPad 11 e no mini **no instante em que o pincel entrava em mãos**
/// (10 entradas em repouso, **18** com o Painter), e isso é `−3,3` pontos de área de desenho,
/// permanentes, justamente quando o ecrã faz falta
/// (`docs/UI_New_and_Simple/medicoes/06_o_orcamento_de_ecra_em_tablet.md`).
///
/// ⚠️ **A terceira saída ficou fora, e a recusa é antiga:** encolher o chip **mente sobre o preset
/// de tamanho** que o artista escolheu no menu.
///
/// # ⚠️ O `⋯` reserva o lugar dele ANTES de a conta correr
///
/// Senão a última entrada caberia por um triz, o `⋯` nasceria por cima dela, e o alvo do dedo
/// ficaria ambíguo. ⇒ a largura disponível já desconta o chip, e só depois se pergunta o que cabe.
#[must_use]
pub fn bar_split(
    store: &WidgetStore,
    text_system: &mut TextSystem,
    painter_active: bool,
    image_tools_on: bool,
    area_w: f32,
) -> (ToolRail, Vec<crate::widget::ToolRailEntry>) {
    let rail = bar_rail(store, text_system, painter_active, image_tools_on);
    if fila_fina(&rail) {
        return bar_split_fina(rail, area_w);
    }
    let size = store.rail_button_size();
    let content_w = (area_w - Spacing::Xs.px() * 2.0).max(0.0);
    // Cabe tudo? Então não há `⋯` nenhum — e um chip de transbordo vazio seria um controlo morto.
    if crate::widget::horizontal_lines(&rail, content_w, size) <= 1 {
        return (rail, Vec::new());
    }
    let more = crate::widget::ToolRailEntry::icon(
        ids::TOOL_BAR_OVERFLOW,
        tr("chrome.tool_bar.more"),
        crate::icons::IconId::MoreHorizontal,
    )
    .with_sub("");
    // ⭐ **O corte pergunta o MESMO que o pintor** — *a fila com esta entrada e o `⋯` fica numa
    // linha?* — pela mesma função ([`crate::widget::horizontal_lines`]), e não por uma soma própria
    // (`along + gap + advance` com `≤`, contra o `along += gap; along += advance` com `>` da porta):
    // duas contas da mesma pergunta arredondam diferente num encaixe exacto, e quem decide o que se
    // VÊ é o pintor — um `⋯` que coubesse aqui e quebrasse lá ficaria cortado pela faixa (D10, 07/10).
    let one_line = |fits: &[crate::widget::ToolRailEntry], next: &crate::widget::ToolRailEntry| {
        let mut probe: Vec<crate::widget::ToolRailEntry> = fits.to_vec();
        probe.push(next.clone());
        while matches!(probe.last(), Some(crate::widget::ToolRailEntry::Divider)) {
            probe.pop();
        }
        probe.push(more.clone());
        crate::widget::horizontal_lines(&ToolRail::new(NodeId(203), "", probe), content_w, size)
            <= 1
    };
    let mut fits: Vec<crate::widget::ToolRailEntry> = Vec::new();
    let mut over: Vec<crate::widget::ToolRailEntry> = Vec::new();
    for entry in rail.entries {
        if over.is_empty() && one_line(&fits, &entry) {
            fits.push(entry);
        } else {
            over.push(entry);
        }
    }
    // ⚠️ Um divisor no FIM da fila que fica não separa nada — ele iria sozinho contra a borda.
    while matches!(fits.last(), Some(crate::widget::ToolRailEntry::Divider)) {
        fits.pop();
    }
    // …e um divisor no PRINCÍPIO do transbordo também não.
    while matches!(over.first(), Some(crate::widget::ToolRailEntry::Divider)) {
        over.remove(0);
    }
    fits.push(more);
    (
        ToolRail::new(NodeId(203), tr("chrome.rail.editor_tools"), fits),
        over,
    )
}

/// ⭐ **A altura da faixa neste quadro**: `TOOL_ROW_H_PX` quando a fila é fina ([`fila_fina_da_loja`]),
/// senão a de sempre ([`tool_bar_h`]).
#[must_use]
pub fn tool_bar_h_da_loja(store: &WidgetStore) -> f32 {
    if fila_fina_da_loja(store) {
        ph2d_tokens::TOOL_ROW_H_PX
    } else {
        tool_bar_h(store.rail_button_size(), 1)
    }
}

/// A pergunta do [`fila_fina`] sem montar a fila (o layout não tem o `TextSystem`): em cartões, a
/// fila da cena é fina; a barra de um módulo só se todas as entradas dela forem glifos ou opções.
#[must_use]
pub fn fila_fina_da_loja(store: &WidgetStore) -> bool {
    crate::widget::lista_flutuante::cromo_em_cartoes()
        && store.tool_bars().is_none_or(|b| {
            b.row_lead
                .iter()
                .chain(b.row_tail.iter())
                .all(crate::widget::entrada_fina)
        })
}

/// ⭐ **A fila é FINA** (dono 12/11, maquete C `.c15tl`): em cartões, quando todas as entradas são
/// glifos ou opções. A faixa tem `TOOL_ROW_H_PX` em cartões ([`tool_bar_h`]); uma fila com campos
/// numéricos (a barra de um módulo) não cabe nela e usa a geometria de sempre, cortada pela faixa.
#[must_use]
pub fn fila_fina(rail: &ToolRail) -> bool {
    crate::widget::lista_flutuante::cromo_em_cartoes() && crate::widget::cabe_na_fila_fina(rail)
}

/// Em quantas linhas a fila `rail` se pinta numa área de `area_w` — pela régua que o [`bar_split`]
/// usa: a da fila fina quando ela é fina, a do trilho largo senão. Os portões do tablet perguntam AQUI
/// (com só a do trilho largo, a fila fina da onda 6 media 2 linhas onde se pinta numa).
#[must_use]
pub fn bar_lines(rail: &ToolRail, area_w: f32, size: crate::widget::RailButtonSize) -> usize {
    if fila_fina(rail) {
        return if crate::widget::fila_fina_cabe(rail, area_w) { 1 } else { 2 };
    }
    crate::widget::horizontal_lines(rail, (area_w - Spacing::Xs.px() * 2.0).max(0.0), size)
}

/// O transbordo da fila fina: a mesma pergunta (*a fila com esta entrada e o `⋯` cabe?*), pela porta
/// da fila fina ([`crate::widget::fila_fina_cabe`]).
fn bar_split_fina(rail: ToolRail, area_w: f32) -> (ToolRail, Vec<crate::widget::ToolRailEntry>) {
    if crate::widget::fila_fina_cabe(&rail, area_w) {
        return (rail, Vec::new());
    }
    let mut more = ToolRail::new(
        NodeId(203),
        "",
        vec![
            crate::widget::ToolRailEntry::icon(
                ids::TOOL_BAR_OVERFLOW,
                tr("chrome.tool_bar.more"),
                crate::icons::IconId::MoreHorizontal,
            )
            .with_sub(""),
        ],
    );
    let mut fits: Vec<crate::widget::ToolRailEntry> = Vec::new();
    let mut over: Vec<crate::widget::ToolRailEntry> = Vec::new();
    for entry in rail.entries {
        let mut probe = fits.clone();
        probe.push(entry.clone());
        probe.extend(more.entries.iter().cloned());
        let cabe = crate::widget::fila_fina_cabe(&ToolRail::new(NodeId(203), "", probe), area_w);
        if over.is_empty() && cabe {
            fits.push(entry);
        } else {
            over.push(entry);
        }
    }
    while matches!(fits.last(), Some(crate::widget::ToolRailEntry::Divider)) {
        fits.pop();
    }
    while matches!(over.first(), Some(crate::widget::ToolRailEntry::Divider)) {
        over.remove(0);
    }
    fits.append(&mut more.entries);
    (
        ToolRail::new(NodeId(203), tr("chrome.rail.editor_tools"), fits),
        over,
    )
}

/// ⭐ **Onde cai cada entrada da fila** — fina ou alta, a porta que o pintor, o hit e os popovers
/// perguntam.
#[must_use]
pub fn row_slots(
    rail: &ToolRail,
    bar: Rect,
    size: RailButtonSize,
) -> Vec<crate::widget::EntrySlot> {
    if fila_fina(rail) {
        crate::widget::fila_fina_slots(rail, bar)
    } else {
        entry_rects(rail, content_rect(bar), size, RailAxis::Horizontal)
    }
}

/// **O rectângulo em que os chips de facto correm** — a faixa menos o respiro.
///
/// ⚠️ **Uma função, e não uma conta inline no pintor.** Ela é a origem que a porta
/// ([`crate::widget::entry_rects`]) recebe, logo quem quiser saber onde um chip caiu — um gate, um
/// flyout, a próxima wave — tem de fazer a MESMA conta. Uma segunda cópia dela seria o espelho que
/// esta wave inteira existiu para apagar.
#[must_use]
pub fn content_rect(bar: Rect) -> Rect {
    Rect::new(
        bar.x + Spacing::Xs.px(),
        bar.y + Spacing::Xxs.px(),
        (bar.w - Spacing::Xs.px() * 2.0).max(0.0),
        (bar.h - Spacing::Xxs.px() * 2.0).max(0.0),
    )
}

/// ⭐ **O rail da fila** — a MESMA lista que a coluna pinta, mais as ferramentas de imagem quando
/// o modo delas está ligado.
///
/// ⛔⛔ **As ferramentas de imagem estão aqui porque ficaram INALCANÇÁVEIS.** Elas eram pintadas
/// num sítio só — a fila de pills dentro do `paint_top_bar` — e essa barra saiu de cena em
/// 2026-08-30; a auditoria do mesmo dia mediu que não havia atalho, nem linha de menu, nem
/// projecção na paleta. ⇒ o **Painter** incluído, e com ele toda a face de pintura desta fila.
///
/// ⚠️ **Elas NÃO entram no `rail_entries`**, e é de propósito: com a `F9` ligada o
/// `paint_top_bar` regista os mesmos ids, e dois rectângulos para um id no mesmo quadro é a
/// ambiguidade que o `HitIndex` resolve por ordem de pintura — não por significado.
///
/// ⚠️ E elas só aparecem com o modo ligado (*Window → Image Tools*), que é a mesma condição que a
/// shell exige para as ACTIVAR (`Some("image_tools") => hero.image_edit.mode_on`). Oferecer um
/// chip que o gate a jusante recusa seria a terceira espécie de knob morto.
#[must_use]
///
/// ⭐ Ela MEDE os pulldowns ([`crate::widget::size_row_pulldowns`]) — por isso pede o `TextSystem`:
/// a largura de um pulldown na fila é a da face mais larga que ele pode mostrar.
pub fn bar_rail(
    store: &WidgetStore,
    text_system: &mut TextSystem,
    painter_active: bool,
    image_tools_on: bool,
) -> ToolRail {
    // ⭐⭐ **O PRODUTOR do módulo** ([`crate::interaction::ToolBars`], hoje o Vetor — D10 ②). Com ele
    // publicado a fila é a BARRA DE CONTROLE do módulo (F16, dono 15/10: *«os botões que estão na
    // barra do topo devem sair»* — o gizmo, SPACE, VIEW, UNDO/REDO e o seletor de modo): só os
    // ajustes dele, na ordem dele, que é a PRIORIDADE — o `⋯` leva a cauda.
    let module = store.tool_bars();
    let mut entries = if module.is_some() {
        Vec::new()
    } else {
        rail_entries(store, painter_active)
    };
    if let Some(bars) = module {
        entries.extend(bars.row_lead.iter().cloned());
        if !bars.row_tail.is_empty() {
            entries.push(crate::widget::ToolRailEntry::Divider);
            entries.extend(bars.row_tail.iter().cloned());
        }
    }
    if image_tools_on {
        let tools = super::topbar::image_tool_rail_entries(store);
        if !tools.is_empty() {
            entries.push(crate::widget::ToolRailEntry::Divider);
            entries.extend(tools);
        }
    }
    // ⭐⭐⭐ **E os PULLDOWNS da área, no fim** — ver `ids::area_menu_button`.
    //
    // ⛔⛔ **PULLDOWNS, nunca os comandos crus — e o número é MEDIDO.** Com os nove comandos crus a
    // fila precisa de **2 linhas até no iPad 12,9"** e transborda `2` chips (mutação 6 de
    // 2026-08-31). *Poupar altura gastando largura não poupa nada.* O orçamento medido em
    // 2026-09-01 é de **3** chips de área (o 4.º põe o iPad 11 e o mini em duas linhas).
    //
    // ⚠️ **No FIM de propósito.** Se nem eles couberem, quem transborda são eles e não uma
    // ferramenta: o gesto do minuto a minuto é pegar numa ferramenta, e um pulldown já está a um
    // clique por natureza.
    //
    // ⚠️ A FACE de cada um é uma leitura (*qual é a vista agora*, *qual é o verbo do gizmo*), e é
    // por isso que ela vem do store em vez de ser o rótulo: um chip que diz sempre a mesma coisa
    // não distingue os estados que abre.
    // A coluna do PINTOR não é um módulo com barra de controle (F16): o seletor de modo continua na fila — é por
    // ele que se sai do modo Paint (`painter_column`).
    if module.is_none_or(crate::interaction::ToolBars::is_family_column)
        && !store.area_menus().is_empty()
    {
        entries.push(crate::widget::ToolRailEntry::Divider);
        for (slot, menu) in store.area_menus().iter().enumerate() {
            entries.push(
                crate::widget::ToolRailEntry::compound(
                    ids::area_menu_button(u32::try_from(slot).unwrap_or(ids::MAX_AREA_MENUS)),
                    menu.label.clone(),
                    menu.face.clone(),
                    "",
                )
                .showing(menu.faces.iter().cloned()),
            );
        }
    }
    let mut rail = ToolRail::new(NodeId(203), tr("chrome.rail.editor_tools"), entries);
    if fila_fina(&rail) {
        crate::widget::medir_fila_fina(&mut rail, text_system);
    } else {
        crate::widget::size_row_pulldowns(&mut rail, text_system);
    }
    rail
}

/// Desenha a fila e regista os alvos.
#[allow(clippy::too_many_arguments)] // o relógio é o 8º, como no irmão vertical
pub fn paint_tool_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    painter_active: bool,
    image_tools_on: bool,
    motion: &crate::motion::UiMotion,
) {
    let bar = layout.tool_bar;
    if bar.w <= 0.0 || bar.h <= 0.0 {
        return;
    }
    // ⭐ **A porta ÚNICA** — o que cabe numa linha, com o `⋯` já no fim quando algo transbordou.
    // O menu do transbordo lê a MESMA função (`context_menu_overlay::paint_tool_bar_overflow`).
    let (rail, _over) = bar_split(store, text_system, painter_active, image_tools_on, bar.w);
    // (a publicação do transbordo faz-se no `publish_overflow`, que o hero chama com `&mut store`)
    let size = store.rail_button_size();
    // A faixa inteira leva o fundo do trilho — é o mesmo chrome, deitado.
    let chao = crate::widget::cromo_barra::Barra::de(theme)
        .map_or_else(|| resolve(ColorToken::RailBg, theme), |b| b.chao());
    scene.fill_rect(crate::paint::rect_to_vello(bar), chao);
    let content = content_rect(bar);
    // ⚠️ **A fila é CORTADA pela própria faixa, no desenho E no hit.** Numa janela estreita (ou
    // com as duas colunas abertas) os chips passam do fim da área — e sem a blindagem eles
    // pintariam por cima da coluna da direita e continuariam **clicáveis lá**. A tinta e o dedo
    // recebem a mesma banda: cortar só um deixaria um alvo invisível, que é pior que um chip
    // cortado. (O `HitIndex::push_clip` já existia; era o painel de nós que o pedia.)
    let clip = ph2d_vector::Rect::new(
        bar.x as f64,
        bar.y as f64,
        (bar.x + bar.w) as f64,
        (bar.y + bar.h) as f64,
    );
    scene.push_clip(&clip);
    hit_index.push_clip(bar);
    // ⛔⛔ **O fundo ENGOLE o clique.** Medido pela auditoria de 2026-08-30: sem ele **70,6 %** da
    // faixa pintada deixava o ponteiro passar para a arte por baixo — incluindo a banda do
    // RÓTULO de cada chip, que fica por cima dele. É o mesmo `RAIL_BACKDROP` que o trilho
    // vertical regista desde 2026-07-16, pela mesma razão e depois do mesmo report do Enio.
    // ⚠️ Antes dos chips, de propósito: o `HitIndex` caminha de trás para a frente.
    hit_index.register(ids::RAIL_BACKDROP, bar);
    match crate::widget::cromo_barra::Barra::de(theme) {
        Some(barra) if fila_fina(&rail) => {
            crate::widget::paint_fila_fina(barra, &rail, bar, scene, text_system, theme, store);
        }
        _ => paint_tool_rail_axis(
            &rail,
            content,
            scene,
            text_system,
            theme,
            store,
            &|id| Some(motion.get(id).unwrap_or(0.0)),
            motion.travels(),
            RailAxis::Horizontal,
        ),
    }
    let mut shapes_chip: Option<Rect> = None;
    let mut mask_group_chip: Option<Rect> = None;
    for slot in row_slots(&rail, bar, size) {
        let Some(id) = slot.id else {
            continue; // o divisor não se clica
        };
        hit_index.register(id, slot.rect);
        // O losango do chip numérico ou de opção (o alvo que o pintor registou ao pintá-lo).
        if matches!(
            rail.entries.get(slot.index),
            Some(
                crate::widget::ToolRailEntry::Number { .. }
                    | crate::widget::ToolRailEntry::Compound { .. }
            )
        ) && let (_, Some(d)) = crate::widget::number_with_key_mark(id, slot.rect)
        {
            hit_index.register(crate::widget::key_mark::key_mark_id(id), d);
        }
        let steady = motion.reduced_motion();
        super::float_bar::flash::ring_if_home(scene, store, steady, theme, id, slot.rect);
        if id == ids::PAINTER_RAIL_SHAPES {
            shapes_chip = Some(slot.rect);
        } else if id == ids::PAINTER_RAIL_MASK_GROUP {
            mask_group_chip = Some(slot.rect);
        }
    }
    scene.pop_layer();
    hit_index.pop_clip();
    // Os dois flyouts de grupo (só em modo Painter). ⚠️ **Fora da blindagem, de propósito**: eles
    // caem POR BAIXO da faixa, e cortá-los pela faixa apagava-os inteiros. ⚠️ **Eles caem PARA BAIXO**, não para o lado:
    // numa fila horizontal o vizinho da direita é outro verbo, e um flyout lateral cobri-lo-ia.
    if painter_active
        && store.painter_shapes_flyout_open()
        && let Some(anchor) = shapes_chip
    {
        paint_flyout_below(
            anchor,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            &PAINTER_SHAPES,
            NodeId(204),
            ph2d_i18n::tr("chrome.flyout.shape_options"),
            motion,
        );
    }
    if painter_active
        && store.painter_mask_flyout_open()
        && let Some(anchor) = mask_group_chip
    {
        paint_flyout_below(
            anchor,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            &PAINTER_MASK_SUBS,
            NodeId(205),
            ph2d_i18n::tr("chrome.flyout.mask_options"),
            motion,
        );
    }
}

/// O flyout de um chip de grupo, **por baixo** dele — uma mini-coluna, com a geometria da mesma
/// porta.
#[allow(clippy::too_many_arguments)]
fn paint_flyout_below(
    anchor: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    subs: &[super::left_rail::RailTool],
    rail_id: NodeId,
    a11y: &str,
    motion: &crate::motion::UiMotion,
) {
    let entries = subs.iter().map(|t| tool_entry(store, *t)).collect();
    let rail = ToolRail::new(rail_id, a11y, entries);
    let size = store.rail_button_size();
    let flyout = Rect::new(
        // ⚠️ O `CHIP_X_OFFSET_PX` que a coluna reserva para o rótulo rodado desloca o chip para a
        // direita; recuá-lo aqui mantém os chips do flyout alinhados com o chip que os abriu.
        anchor.x - crate::widget::CHIP_X_OFFSET_PX,
        // F5: com a fila embaixo, o flyout sobe.
        if store.tool_row_bottom() {
            anchor.y - Spacing::Xs.px() - rail.preferred_height(size)
        } else {
            anchor.y + anchor.h + Spacing::Xs.px()
        },
        size.rail_width_px(),
        rail.preferred_height(size),
    );
    let bg = Rect::new(
        flyout.x,
        flyout.y - Spacing::Sm.px(),
        flyout.w,
        flyout.h + Spacing::Sm.px() * 2.0,
    );
    if let Some(b) = crate::widget::cromo_barra::Barra::de(theme) {
        b.painel(scene, bg);
    } else {
        fill_rounded_rect(
            scene,
            bg,
            crate::paint::frame_radius(theme, Radius::Md.px()),
            resolve(ColorToken::RailBg, theme),
        );
    }
    paint_tool_rail_axis(
        &rail,
        flyout,
        scene,
        text_system,
        theme,
        store,
        &|id| Some(motion.get(id).unwrap_or(0.0)),
        motion.travels(),
        RailAxis::Vertical,
    );
    for slot in entry_rects(&rail, flyout, size, RailAxis::Vertical) {
        if let Some(id) = slot.id {
            hit_index.register(id, slot.rect);
        }
    }
}

/// O `⋯` em cartões: uma lista com o chip e o nome inteiro de cada entrada, nunca uma legenda rodada que
/// não cabe no lado do chip. Os ids são os da fila, e a linha inteira é o alvo.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_transbordo_em_lista(
    req: crate::interaction::ContextMenuRequest,
    over: Vec<crate::widget::ToolRailEntry>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    motion: &crate::motion::UiMotion,
    viewport: Rect,
) {
    let superficie = crate::widget::lista_flutuante::Superficie::de(theme);
    let size = store.rail_button_size();
    let pad = Spacing::Sm.px();
    let font = ph2d_tokens::TypeToken::Sm.px();
    let nomes: Vec<String> = over
        .iter()
        .map(|e| e.label().unwrap_or_default().to_owned())
        .collect();
    let nome_w = nomes
        .iter()
        .map(|n| text_system.prefix_width(n, font))
        .fold(0.0_f32, f32::max)
        .ceil();
    let rail = ToolRail::new(
        NodeId(204),
        tr("chrome.menu.more_tools"),
        over.into_iter()
            .map(crate::widget::ToolRailEntry::sem_legenda)
            .collect(),
    );
    let chip_w = crate::widget::column_width_px(&rail, size)
        - crate::widget::CHIP_X_OFFSET_PX
        - Spacing::Xs.px();
    let inner_w = chip_w + pad + nome_w;
    let inner_h = rail.preferred_height(size);
    let rect = crate::widget::panel_chrome::clamp_menu_to_viewport(
        req.x,
        super::context_menu_overlay::anchor_y(store, req, inner_h + pad * 2.0),
        inner_w + pad * 2.0,
        inner_h + pad * 2.0,
        viewport,
    );
    superficie.moldura(scene, rect);
    // O fundo ENGOLE o clique e vai ANTES das linhas: o `HitIndex` caminha de trás para a frente.
    hit_index.register(ids::RAIL_BACKDROP, rect);
    // A coluna reserva à esquerda o lugar da legenda rodada, que aqui não existe.
    let coluna = Rect::new(
        rect.x + pad - crate::widget::CHIP_X_OFFSET_PX,
        rect.y + pad,
        inner_w + crate::widget::CHIP_X_OFFSET_PX,
        inner_h,
    );
    let slots = entry_rects(&rail, coluna, size, RailAxis::Vertical);
    for slot in &slots {
        if slot.id.is_some() && slot.id == store.hot_id() {
            let linha = Rect::new(rect.x, slot.rect.y, rect.w, slot.rect.h);
            superficie.realce(scene, linha);
        }
    }
    paint_tool_rail_axis(
        &rail,
        coluna,
        scene,
        text_system,
        theme,
        store,
        &|id| Some(motion.get(id).unwrap_or(0.0)),
        motion.travels(),
        RailAxis::Vertical,
    );
    let x_do_nome = rect.x + pad + chip_w + pad;
    let cor = superficie.cor(ColorToken::Text1);
    for slot in slots {
        let Some(id) = slot.id else { continue };
        crate::paint::paint_text_elided(
            text_system,
            scene,
            &nomes[slot.index],
            x_do_nome,
            slot.rect.y + superficie.folga(slot.rect.h, font),
            font,
            nome_w,
            cor,
        );
        hit_index.register(id, Rect::new(rect.x, slot.rect.y, rect.w, slot.rect.h));
    }
}

/// ⭐ **Publica o que não coube** para o corpo do menu de transbordo o desenhar.
///
/// ⚠️ **Chamado em TODO quadro, vazio incluído** — um mapa que o tique apaga não envelhece; um que
/// só se escreve quando há transbordo deixaria chips fantasma atrás do `⋯` depois de a janela
/// crescer.
///
/// ⚠️ Separado do [`paint_tool_bar`] por empréstimo: o pintor tem `&WidgetStore` e isto precisa de
/// `&mut`. A CONTA é a mesma ([`bar_split`]), e é essa a garantia que interessa.
pub fn publish_overflow(
    store: &mut WidgetStore,
    text_system: &mut TextSystem,
    layout: &HeroLayout,
    painter_active: bool,
    image_tools_on: bool,
) {
    let bar = layout.tool_bar;
    if bar.w <= 0.0 || bar.h <= 0.0 {
        store.set_tool_overflow(Vec::new());
        store.set_tool_row_slots(Vec::new());
        return;
    }
    let (rail, over) = bar_split(store, text_system, painter_active, image_tools_on, bar.w);
    store.set_tool_overflow(over);
    // A fila fina não tem legenda: o nome de cada botão vai na dica (dono 12/11). Quem já tem dica
    // própria (os do gizmo, undo/redo) fica com ela.
    if fila_fina(&rail) {
        for entry in &rail.entries {
            if let (Some(id), Some(nome)) = (entry.node_id(), entry.label())
                && store.tooltip_for(id).is_none()
                && !nome.is_empty()
            {
                store.set_tooltip(id, nome.to_owned());
            }
        }
    }
    // F16: a mesma porta da geometria que o pintor e o hit perguntam — a âncora dos popovers.
    let slots = row_slots(&rail, bar, store.rail_button_size())
        .into_iter()
        .filter_map(|s| s.id.map(|id| (id, s.rect)))
        .collect();
    store.set_tool_row_slots(slots);
}

#[cfg(test)]
#[path = "tool_bar_tests.rs"]
mod tests;
