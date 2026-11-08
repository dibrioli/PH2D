//! **A busca no popover do Preset** (dono 08/11: «precisamos de um campo de busca no dropdown»): um campo
//! de texto na primeira linha da lista aberta filtra os pincéis pelo nome (sem caixa nem acento); só os
//! títulos de família com algum achado ficam. O teclado: o foco vai ao campo quando a lista abre, ↑/↓
//! movem o realce (as setas armadas do campo, `arm_list_nav`, a porta das menções do Quadro), Enter
//! escolhe, Esc limpa e, vazio, fecha. O campo mora numa FAIXA fixa no topo da lista (dono 08/11: o de antes
//! «ficou feio» — encostava nas bordas e rolava com a lista): a faixa tem o fundo da lista e a folga dos cartões
//! (`LIST_PAD_X` dos lados, o campo centrado com a altura e a tinta dos campos do cartão, anel fino no foco), e as
//! linhas passam por BAIXO dela. A faixa cobre exactamente [`LINHAS_DO_CAMPO`] títulos vazios no topo das opções:
//! o filete do 1.º título de família é o fundo dela, e com a lista rolada a faixa pinta o seu.

use crate::paint_wetpaint_presets::{escolher_preset, preset_escolhido};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal, camada_das_listas};
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, PaintedPopover, TextInput, TextInputState,
    paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids;
use std::cell::{Cell, RefCell};

const CAMPO: NodeId = ids::PAINTER_WETPAINT_PRESET_SEARCH;
/// O valor das linhas do campo: nenhum preset o tem, nem o vão do arrasto nem o «sem preset».
const VALOR_DO_CAMPO: usize = usize::MAX - 2;
/// Quantas linhas da lista a faixa do campo cobre: o campo do cartão (`FIELD_H` 28) com folga não cabe numa (28).
pub(crate) const LINHAS_DO_CAMPO: usize = 2;
const FOLGA: f32 = ph2d_card_stack::metrics::LIST_PAD_X;
const ALTO_DO_CAMPO: f32 = ph2d_card_stack::metrics::FIELD_H;

/// A altura da faixa fixa do campo, do topo da lista ao fundo das suas [`LINHAS_DO_CAMPO`] linhas.
#[must_use]
fn altura_da_faixa(chip: Rect, panel: Rect) -> f32 {
    let dd = Dropdown::<usize>::new(ids::PAINTER_WETPAINT_PRESET, "", Vec::new());
    let ultima = dd.option_rect_in(chip, panel, LINHAS_DO_CAMPO - 1);
    ultima.y + ultima.h - panel.y
}

thread_local! {
    /// A lista estava aberta no quadro anterior (abrir = limpar a busca e focar o campo).
    static ABERTO: Cell<bool> = const { Cell::new(false) };
    /// A busca do quadro anterior: mudou ⇒ o realce volta ao primeiro achado e a lista ao topo.
    static ANTERIOR: RefCell<String> = const { RefCell::new(String::new()) };
    /// O valor realçado pelo teclado (`None` = o preset activo).
    static REALCE: Cell<Option<usize>> = const { Cell::new(None) };
    /// O id da linha realçada — o que o Enter escolhe.
    static REALCE_ID: Cell<Option<NodeId>> = const { Cell::new(None) };
}

/// Minúsculas e sem acento — a dobra dos dois lados da comparação.
pub(crate) fn dobrar(s: &str) -> String {
    s.chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            _ => c,
        })
        .collect()
}

/// O nome casa a busca: pedaço do nome, sem caixa nem acento; a busca vazia casa tudo.
pub(crate) fn casa(nome: &str, busca: &str) -> bool {
    let q = dobrar(busca.trim());
    q.is_empty() || dobrar(nome).contains(&q)
}

/// O realce depois de `passos` setas a partir de `de` (um valor), só entre as linhas escolhíveis, preso
/// às pontas. Sem ponto de partida, ↓ começa no primeiro e ↑ no último. `None` = nada a realçar.
pub(crate) fn passo(
    opcoes: &[DropdownOption<usize>],
    de: Option<usize>,
    passos: i32,
) -> Option<usize> {
    let vivas: Vec<usize> = opcoes
        .iter()
        .filter(|o| !o.header && !o.disabled)
        .map(|o| o.value)
        .collect();
    let n = vivas.len() as i64;
    if n == 0 {
        return None;
    }
    let p = i64::from(passos);
    let i = match de.and_then(|v| vivas.iter().position(|&x| x == v)) {
        Some(at) => at as i64 + p,
        None if p < 0 => n + p,
        None => (p - 1).max(0),
    };
    Some(vivas[i.clamp(0, n - 1) as usize])
}

/// A busca escrita no campo agora.
fn texto(ctx: &PaintCtx) -> String {
    match ctx.host.store().get(CAMPO) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => String::new(),
    }
}

fn foca(store: &mut ph2d_editor_core::interaction::WidgetStore, limpa: bool) {
    if let Some(InteractiveState::TextInput {
        state,
        text,
        caret,
        selection_anchor,
    }) = store.get_mut(CAMPO)
    {
        *state = TextInputState::Focused;
        if limpa {
            text.clear();
            *caret = 0;
            *selection_anchor = None;
        }
    }
    store.set_focus(Some(CAMPO));
}

/// A lista aberta neste quadro: regista o campo, foca-o ao abrir e devolve a busca.
pub(crate) fn abre(ctx: &mut PaintCtx) -> String {
    let store = ctx.host.store_mut();
    store.register_if_absent(
        CAMPO,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.mark_cancel_on_escape(CAMPO);
    store.arm_list_nav(Some(CAMPO));
    if !ABERTO.with(|c| c.replace(true)) {
        foca(store, true);
        REALCE.with(|c| c.set(None));
        ANTERIOR.with(|c| c.borrow_mut().clear());
    }
    texto(ctx)
}

/// A lista não se pintou neste quadro: larga o foco e as setas do campo.
pub(crate) fn fecha(ctx: &mut PaintCtx) {
    if !ABERTO.with(|c| c.replace(false)) {
        return;
    }
    let store = ctx.host.store_mut();
    store.arm_list_nav(None);
    if store.focus_id() == Some(CAMPO) {
        store.set_focus(None);
    }
    if let Some(InteractiveState::TextInput { state, .. }) = store.get_mut(CAMPO) {
        *state = TextInputState::Normal;
    }
    REALCE_ID.with(|c| c.set(None));
}

/// O valor a realçar neste quadro: as setas desde o último, a busca que mudou (volta ao primeiro achado
/// e ao topo da lista) ou o preset activo. Mantém a linha realçada à vista.
pub(crate) fn realce(
    ctx: &mut PaintCtx,
    opcoes: &[DropdownOption<usize>],
    chip: Rect,
    atual: usize,
    busca: &str,
) -> usize {
    let mudou = ANTERIOR.with(|c| {
        let mut c = c.borrow_mut();
        let m = *c != busca;
        if m {
            busca.clone_into(&mut c);
        }
        m
    });
    let store = ctx.host.store_mut();
    let passos = store.take_list_steps(CAMPO);
    let mut cur = if mudou {
        store.set_panel_scroll(ids::PAINTER_WETPAINT_PRESET, 0.0);
        None
    } else {
        REALCE.with(Cell::get)
    };
    let ativo = (atual != usize::MAX && busca.trim().is_empty()).then_some(atual);
    if passos != 0 {
        cur = passo(opcoes, cur.or(ativo), passos);
    } else if cur.is_none() && !busca.trim().is_empty() {
        cur = passo(opcoes, None, 1);
    }
    REALCE.with(|c| c.set(cur));
    let mostra = cur.or(ativo).unwrap_or(usize::MAX);
    let linha = opcoes
        .iter()
        .position(|o| o.value == mostra && !o.header && !o.disabled);
    REALCE_ID.with(|c| c.set(linha.map(|i| opcoes[i].id)));
    if passos != 0
        && let Some(i) = linha
    {
        a_vista(ctx, opcoes, chip, i + LINHAS_DO_CAMPO);
    }
    mostra
}

/// Rola a lista até a linha `i` (já contadas as do campo) caber por baixo da faixa fixa.
fn a_vista(ctx: &mut PaintCtx, opcoes: &[DropdownOption<usize>], chip: Rect, i: usize) {
    let dd = Dropdown::new(ids::PAINTER_WETPAINT_PRESET, "", com_campo(opcoes.to_vec())).open(true);
    let panel = dd.popover_rect_fitted(chip, Some(ctx.viewport), ctx.text_system);
    let r = dd.option_rect_in_scrolled(chip, panel, i, 0.0);
    let topo = r.y - panel.y;
    let store = ctx.host.store_mut();
    let mut s = store.panel_scroll(ids::PAINTER_WETPAINT_PRESET);
    let faixa = altura_da_faixa(chip, panel);
    if topo - faixa < s {
        // A linha de cima (o título da família) vem com ela.
        s = (topo - r.h - faixa).max(0.0);
    } else if topo + r.h > s + panel.h {
        s = topo + r.h - panel.h;
    }
    store.set_panel_scroll(ids::PAINTER_WETPAINT_PRESET, s);
}

/// As opções com as [`LINHAS_DO_CAMPO`] linhas da faixa no topo (títulos vazios: não se escolhem nem se arrastam).
pub(crate) fn com_campo(opcoes: Vec<DropdownOption<usize>>) -> Vec<DropdownOption<usize>> {
    (0..LINHAS_DO_CAMPO)
        .map(|k| {
            DropdownOption::new(
                ids::painter_wetpaint_preset_title_id(usize::MAX - k),
                VALOR_DO_CAMPO,
                String::new(),
            )
            .header(true)
        })
        .chain(opcoes)
        .collect()
}

/// A linha «No brushes found» (apagada) quando a busca não acha nada.
pub(crate) fn sem_achados(k: usize) -> DropdownOption<usize> {
    DropdownOption::new(
        ids::painter_wetpaint_preset_title_id(k),
        k,
        tr("panel.painter_layers.wetpaint.preset_no_match").to_owned(),
    )
    .disabled(true)
}

/// Pinta a faixa fixa do campo no topo da lista pintada (por cima das linhas que passam por baixo), com a
/// tinta da lista (`tinta` = a do cartão do chip; `None` = a das listas fora de cartão), e regista o alvo:
/// a faixa INTEIRA é do campo, para um clique nela nunca escolher a linha escondida por baixo.
pub(crate) fn pinta_campo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    tinta: Option<ph2d_editor_core::widget::card_ink::CardInk>,
    chip: Rect,
    p: PaintedPopover,
) {
    let faixa = Rect::new(
        p.panel.x,
        p.panel.y,
        p.panel.w,
        altura_da_faixa(chip, p.panel).min(p.panel.h),
    );
    let alto = ALTO_DO_CAMPO.min(faixa.h);
    let r = Rect::new(
        faixa.x + FOLGA,
        faixa.y + (faixa.h - alto) * 0.5,
        (faixa.w - 2.0 * FOLGA).max(0.0),
        alto,
    );
    let filete = p.scroll > 0.0;
    if r.h <= 0.0 {
        return;
    }
    let store = ctx.host.store();
    let (estado, texto, caret, ancora) = match store.get(CAMPO) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    let campo = TextInput::new(CAMPO, "")
        .placeholder(tr("panel.painter_layers.wetpaint.preset_search"))
        .visual((estado, store.hover_live(CAMPO)))
        .anel_fino();
    let focado = store.focus_id() == Some(CAMPO);
    let texto_sistema = &mut *ctx.text_system;
    let mut pinta = |cena: &mut ph2d_vector::VectorScene| {
        pinta_faixa(cena, faixa, filete, theme);
        paint_text_input_with_buffer(
            &campo,
            Some(texto.as_str()),
            focado.then_some(caret),
            ancora,
            r,
            cena,
            texto_sistema,
            theme,
        );
    };
    camada_das_listas::por_cima(ctx.scene, |cena| match tinta {
        Some(ink) => ph2d_editor_core::widget::card_ink::com(ink, || pinta(cena)),
        None => {
            let _veste = ph2d_editor_core::widget::Superficie::de(theme).veste();
            pinta(cena);
        }
    });
    let hit = ctx.host.hit_index_mut();
    camada_das_listas::alvo(hit, CAMPO, faixa);
}

/// O fundo da faixa (o da lista, cantos de cima redondos como os dela) e, com a lista rolada, o filete que a separa
/// das linhas (no topo, é o do 1.º título que lá está).
fn pinta_faixa(
    cena: &mut ph2d_vector::VectorScene,
    faixa: Rect,
    filete: bool,
    theme: ph2d_tokens::Theme,
) {
    use ph2d_editor_core::paint::fill_rounded_rect;
    use ph2d_editor_core::widget::resolve_opaque as opaque;
    let fundo = opaque(ph2d_tokens::ColorToken::BgElev, theme);
    let raio = ph2d_card_stack::metrics::FIELD_RADIUS.min(faixa.h * 0.5);
    fill_rounded_rect(
        cena,
        Rect::new(faixa.x, faixa.y, faixa.w, raio * 2.0),
        raio,
        fundo,
    );
    cena.fill_rect(
        ph2d_editor_core::paint::rect_to_vello(Rect::new(
            faixa.x,
            faixa.y + raio,
            faixa.w,
            (faixa.h - raio).max(0.0),
        )),
        fundo,
    );
    if !filete {
        return;
    }
    let fio = ph2d_tokens::StrokeToken::Thin.px();
    cena.fill_rect(
        ph2d_editor_core::paint::rect_to_vello(Rect::new(
            faixa.x + FOLGA,
            faixa.y + faixa.h - fio,
            (faixa.w - 2.0 * FOLGA).max(0.0),
            fio,
        )),
        opaque(ph2d_tokens::ColorToken::Border, theme),
    );
}

/// Os eventos do campo. `None` = não é daqui.
pub(crate) fn try_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> Option<bool> {
    match ev {
        WidgetEvent::Submit(id) if id == CAMPO => {
            match REALCE_ID.with(Cell::get).and_then(preset_escolhido) {
                Some(rel) => escolher_preset(host, rel),
                None => foca(host.store_mut(), false),
            }
            Some(true)
        }
        WidgetEvent::Cancel(id) if id == CAMPO => {
            let vazio = matches!(
                host.store().get(CAMPO),
                Some(InteractiveState::TextInput { text, .. }) if text.is_empty()
            );
            if vazio {
                if let Some(InteractiveState::Dropdown { open, .. }) =
                    host.store_mut().get_mut(ids::PAINTER_WETPAINT_PRESET)
                {
                    *open = false;
                }
            } else {
                foca(host.store_mut(), true);
            }
            Some(true)
        }
        WidgetEvent::Click(id) | WidgetEvent::TextChanged(id) | WidgetEvent::Blur(id)
            if id == CAMPO =>
        {
            Some(true)
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "paint_wetpaint_presets_busca_tests.rs"]
mod tests;
