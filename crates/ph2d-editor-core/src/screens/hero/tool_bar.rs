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
use super::ids;
use super::left_rail::{PAINTER_MASK_SUBS, PAINTER_SHAPES, rail_entries, tool_entry};
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::{
    LABEL_TO_CHIP_GAP_PX, LABEL_VISUAL_EXTENT_PX, RailAxis, RailButtonSize, ToolRail, entry_rects,
    paint_tool_rail_axis,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
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
    painter_active: bool,
    image_tools_on: bool,
    area_w: f32,
) -> (ToolRail, Vec<crate::widget::ToolRailEntry>) {
    let rail = bar_rail(store, painter_active, image_tools_on);
    let size = store.rail_button_size();
    let content_w = (area_w - Spacing::Xs.px() * 2.0).max(0.0);
    let gap = Spacing::Xs.px();
    let chip = size.chip_px();
    // Cabe tudo? Então não há `⋯` nenhum — e um chip de transbordo vazio seria um controlo morto.
    if crate::widget::horizontal_lines(&rail, content_w, size) <= 1 {
        return (rail, Vec::new());
    }
    let budget = (content_w - chip - gap).max(0.0);
    let mut fits: Vec<crate::widget::ToolRailEntry> = Vec::new();
    let mut over: Vec<crate::widget::ToolRailEntry> = Vec::new();
    let mut along = 0.0_f32;
    for entry in rail.entries {
        let advance = crate::widget::entry_advance(&entry, chip);
        let next = if fits.is_empty() {
            advance
        } else {
            along + gap + advance
        };
        if over.is_empty() && next <= budget {
            along = next;
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
    fits.push(
        crate::widget::ToolRailEntry::icon(
            ids::TOOL_BAR_OVERFLOW,
            "More",
            crate::icons::IconId::MoreHorizontal,
        )
        .with_sub(""),
    );
    (ToolRail::new(NodeId(203), "Editor tools", fits), over)
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
pub fn bar_rail(store: &WidgetStore, painter_active: bool, image_tools_on: bool) -> ToolRail {
    let mut entries = rail_entries(store, painter_active);
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
    if !store.area_menus().is_empty() {
        entries.push(crate::widget::ToolRailEntry::Divider);
        for (slot, menu) in store.area_menus().iter().enumerate() {
            entries.push(crate::widget::ToolRailEntry::compound(
                ids::area_menu_button(u32::try_from(slot).unwrap_or(ids::MAX_AREA_MENUS)),
                menu.label.clone(),
                menu.face.clone(),
                "",
            ));
        }
    }
    ToolRail::new(NodeId(203), "Editor tools", entries)
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
    let (rail, _over) = bar_split(store, painter_active, image_tools_on, bar.w);
    // (a publicação do transbordo faz-se no `publish_overflow`, que o hero chama com `&mut store`)
    let size = store.rail_button_size();
    // A faixa inteira leva o fundo do trilho — é o mesmo chrome, deitado.
    scene.fill_rect(
        crate::paint::rect_to_vello(bar),
        resolve(ColorToken::RailBg, theme),
    );
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
    paint_tool_rail_axis(
        &rail,
        content,
        scene,
        text_system,
        theme,
        store,
        &|id| Some(motion.get(id).unwrap_or(0.0)),
        motion.travels(),
        RailAxis::Horizontal,
    );
    let mut shapes_chip: Option<Rect> = None;
    let mut mask_group_chip: Option<Rect> = None;
    for slot in entry_rects(&rail, content, size, RailAxis::Horizontal) {
        let Some(id) = slot.id else {
            continue; // o divisor não se clica
        };
        hit_index.register(id, slot.rect);
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
            "Shape options",
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
            "Mask options",
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
    subs: &[(NodeId, &str, crate::icons::IconId, &str)],
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
        anchor.y + anchor.h + Spacing::Xs.px(),
        size.rail_width_px(),
        rail.preferred_height(size),
    );
    let bg = Rect::new(
        flyout.x,
        flyout.y - Spacing::Sm.px(),
        flyout.w,
        flyout.h + Spacing::Sm.px() * 2.0,
    );
    fill_rounded_rect(
        scene,
        bg,
        Radius::Md.px(),
        resolve(ColorToken::RailBg, theme),
    );
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
    layout: &HeroLayout,
    painter_active: bool,
    image_tools_on: bool,
) {
    let bar = layout.tool_bar;
    if bar.w <= 0.0 || bar.h <= 0.0 {
        store.set_tool_overflow(Vec::new());
        return;
    }
    let (_, over) = bar_split(store, painter_active, image_tools_on, bar.w);
    store.set_tool_overflow(over);
}
