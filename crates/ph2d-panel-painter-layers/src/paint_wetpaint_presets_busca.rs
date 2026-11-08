//! **A busca no popover do Preset** (dono 08/11: «precisamos de um campo de busca no dropdown»): um campo
//! de texto na primeira linha da lista aberta filtra os pincéis pelo nome (sem caixa nem acento); só os
//! títulos de família com algum achado ficam. O teclado: o foco vai ao campo quando a lista abre, ↑/↓
//! movem o realce (as setas armadas do campo, `arm_list_nav`, a porta das menções do Quadro), Enter
//! escolhe, Esc limpa e, vazio, fecha. A linha do campo é um título vazio no topo das opções, e o
//! campo pinta-se por cima dela, recortado ao popover: rola com a lista.

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
use ph2d_tokens::Spacing;
use ph2d_tool_painter::ids;
use std::cell::{Cell, RefCell};

const CAMPO: NodeId = ids::PAINTER_WETPAINT_PRESET_SEARCH;
/// O valor da linha do campo: nenhum preset o tem, nem o vão do arrasto nem o «sem preset».
const VALOR_DO_CAMPO: usize = usize::MAX - 2;

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
        a_vista(ctx, opcoes, chip, i + 1);
    }
    mostra
}

/// Rola a lista (com a linha do campo no topo) até a linha `i` caber.
fn a_vista(ctx: &mut PaintCtx, opcoes: &[DropdownOption<usize>], chip: Rect, i: usize) {
    let dd = Dropdown::new(ids::PAINTER_WETPAINT_PRESET, "", com_campo(opcoes.to_vec())).open(true);
    let panel = dd.popover_rect_fitted(chip, Some(ctx.viewport), ctx.text_system);
    let r = dd.option_rect_in_scrolled(chip, panel, i, 0.0);
    let topo = r.y - panel.y;
    let store = ctx.host.store_mut();
    let mut s = store.panel_scroll(ids::PAINTER_WETPAINT_PRESET);
    if topo - r.h < s {
        // A linha de cima (título ou campo) vem com ela.
        s = (topo - r.h).max(0.0);
    } else if topo + r.h > s + panel.h {
        s = topo + r.h - panel.h;
    }
    store.set_panel_scroll(ids::PAINTER_WETPAINT_PRESET, s);
}

/// As opções com a linha do campo no topo (um título vazio: não se escolhe nem se arrasta).
pub(crate) fn com_campo(mut opcoes: Vec<DropdownOption<usize>>) -> Vec<DropdownOption<usize>> {
    opcoes.insert(
        0,
        DropdownOption::new(
            ids::painter_wetpaint_preset_title_id(usize::MAX),
            VALOR_DO_CAMPO,
            String::new(),
        )
        .header(true),
    );
    opcoes
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

/// Pinta o campo sobre a linha 0 da lista pintada, recortado ao popover, e regista o alvo dele.
pub(crate) fn pinta_campo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    opcoes: &[DropdownOption<usize>],
    chip: Rect,
    p: PaintedPopover,
) {
    let dd = Dropdown::new(ids::PAINTER_WETPAINT_PRESET, "", opcoes.to_vec()).open(true);
    let linha = dd.option_rect_in_scrolled(chip, p.panel, 0, p.scroll);
    let m = Spacing::Xs.px();
    let r = Rect::new(linha.x + m, linha.y, (linha.w - 2.0 * m).max(0.0), linha.h);
    let topo = r.y.max(p.panel.y);
    let base = (r.y + r.h).min(p.panel.y + p.panel.h);
    if base <= topo {
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
        .visual((estado, store.hover_live(CAMPO)));
    let focado = store.focus_id() == Some(CAMPO);
    let texto_sistema = &mut *ctx.text_system;
    camada_das_listas::por_cima(ctx.scene, |cena| {
        let clip = ph2d_vector::Rect::new(
            f64::from(p.panel.x),
            f64::from(p.panel.y),
            f64::from(p.panel.x + p.panel.w),
            f64::from(p.panel.y + p.panel.h),
        );
        cena.push_clip(&clip);
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
        cena.pop_layer();
    });
    let hit = ctx.host.hit_index_mut();
    camada_das_listas::alvo(hit, CAMPO, Rect::new(r.x, topo, r.w, base - topo));
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
