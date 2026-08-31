//! ⭐⭐⭐ **AS ABAS** — a regra 1 do modelo de áreas: *um encaixe hospeda `0..n` painéis, e com
//! `n > 1` eles são **abas***. É assim que um encaixe absorve crescimento **sem crescer**
//! (`docs/UI_New_and_Simple/spec/01_modelo_de_areas.md` §2).
//!
//! # O defeito que isto cura, medido
//!
//! O `audio_editor` encaixava-se **a oeste** do Inspector (`insp.x − 240 − gap`) para poder estar
//! aberto ao lado do `audio_mixer`. Isso é uma **segunda coluna da direita**, e a spec recusa-a por
//! aritmética: duas colunas por lado são **89,6 %** da largura do alvo de 1366. O painel publicava
//! `168 480 px²` **sobre a área de desenho**.
//!
//! ⚠️ **E ele não era o único a partilhar a coluna — era o único a fazê-lo às escondidas.** Medido
//! em 2026-08-30 com tudo aberto: **treze** painéis publicam o rect `(1062, 28, 304, 996)`, o mesmo
//! ao pixel. Eles não colidiam por convenção (só um está visível de cada vez, conduzido pela
//! ferramenta activa), e nada no repo o afirmava. *As abas não introduzem a partilha: elas tornam
//! visível a que já existia, e dão-lhe um gesto.*
//!
//! # ⭐ A selecção NÃO é estado novo — é a ordem z restrita ao encaixe
//!
//! Guardar «qual aba está escolhida» ao lado da ordem z seria a segunda resposta à mesma pergunta,
//! e as duas divergiriam no primeiro clique que uma delas não visse. ⇒ **a aba escolhida é o
//! ocupante mais ao topo**, e clicar numa aba é [`WidgetStore::bump_panel_z`] — o mesmo verbo que
//! clicar dentro do painel já usava.
//!
//! ⚠️ **Isso obrigou a ordem z a ser reconciliada com a visibilidade** ([`reconcile_z`]): ela era
//! *append-only* e só crescia com cliques, então um painel **acabado de abrir** nascia no fundo e
//! ficava atrás de uma aba que ninguém tinha tocado. Hoje ela é exactamente *«os painéis visíveis,
//! o último a aparecer no topo»*.

use super::HeroScreen;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text_centered, rect_to_vello, resolve};
use crate::screens::slot::{Slot, SlotSet};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A altura da fila de abas — **uma linha**, o mesmo token da barra de menus e de uma linha de
/// menu. ⚠️ Não é um número escolhido: uma aba é um rótulo clicável, que é o que uma linha é.
pub const TAB_BAR_H: f32 = ROW_H_PX;

/// Largura mínima de uma aba antes de o rótulo deixar de caber.
///
/// ⚠️ **É o piso de LEGIBILIDADE, não de layout** — com sete ocupantes numa coluna de 304 px cada
/// aba ficaria com 43 px, que não mostra uma palavra. Acima deste piso as abas dividem a faixa por
/// igual; abaixo dele elas **transbordam** e as que não cabem ficam alcançáveis pelo menu *Window*,
/// que é onde um painel sempre se abriu.
const MIN_TAB_W: f32 = 64.0; // LITERAL-PX-OK: piso de legibilidade de um rótulo de aba (chrome)

/// Espessura do contorno do encaixe sob o dedo.
const DROP_OUTLINE_PX: f32 = 2.0; // LITERAL-PX-OK: contorno da zona de largada (chrome)

/// Largura da etiqueta fantasma que segue o dedo. ⚠️ Fixa, e não a largura da aba de origem: ela
/// atravessa encaixes de larguras diferentes, e uma etiqueta que muda de tamanho a meio do gesto
/// lê-se como o app a decidir alguma coisa que ele não decidiu.
const DRAG_LABEL_W: f32 = 120.0; // LITERAL-PX-OK: etiqueta fantasma do arrasto (chrome)

/// ⛔ **O salto que separa o id de uma ABA do id do PAINEL que ela escolhe.**
///
/// Os dois são controlos diferentes com rects diferentes, e o `HitIndex` mapeia `id → rect`:
/// registá-los com o mesmo id faria o segundo apagar o primeiro, em silêncio.
///
/// ⚠️ **XOR com uma constante é uma bijecção** — logo este derivado não pode criar uma colisão que
/// o espaço de ids dos painéis já não tivesse. *Uma segunda função de hash, sim, poderia.*
const TAB_ID_SALT: u64 = 0x7ab5_0000_5107_0001;

/// O id do controlo **aba** de um painel. Ver [`TAB_ID_SALT`].
#[must_use]
pub fn tab_node_id(panel_node: NodeId) -> NodeId {
    NodeId(panel_node.0 ^ TAB_ID_SALT)
}

/// ⭐⭐ **EM QUE ENCAIXE ESTE PAINEL ESTÁ** — a porta única, e a única leitura de posição do
/// produto.
///
/// > *«Lugares pré-definidos. O artista escolhe **QUAL painel vai em cada lugar**.»* — D4
///
/// O `Panel::DEFAULT_SLOT` é a resposta de **omissão**; o que o artista moveu vive no
/// `WidgetStore` como excepção. ⚠️ Ler o `default_slot` directamente noutro sítio faria o painel
/// aparecer numa coluna e ser contado noutra — e é a contagem que decide se há abas.
#[must_use]
pub fn slot_of(hero: &HeroScreen, m: &crate::panel::PanelManifest) -> Slot {
    hero.store
        .panel_slot(m.panel_node_id)
        .unwrap_or(m.default_slot)
}

/// **De que painel é esta aba?** — a volta de [`tab_node_id`], resolvida pelo registry.
///
/// ⚠️ `with_registry_opt`: ver a nota em [`populate`].
#[must_use]
pub fn panel_for_tab(id: NodeId) -> Option<NodeId> {
    crate::panel::with_registry_opt(|reg| {
        reg.panels()
            .iter()
            .map(|p| p.manifest.panel_node_id)
            .find(|node| tab_node_id(*node) == id)
    })
    .flatten()
}

/// Um painel a ocupar um encaixe neste quadro.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Occupant {
    /// O `Panel::ID` (`"audio_mixer"`).
    pub id: &'static str,
    /// O `Panel::NODE_ID` — o rect do painel.
    pub node: NodeId,
    /// O `Panel::TITLE` — o que o artista lê na aba.
    pub title: &'static str,
}

/// ⭐ **A ordem z passa a ser «os painéis VISÍVEIS, o último a aparecer no topo».**
///
/// Corre no início do quadro. Duas metades, e ⚠️ **as duas são obrigatórias**: sem a poda, fechar e
/// reabrir um painel devolve-o à posição que ele tinha da última vez — ficando **atrás** de uma aba
/// que o artista nunca tocou. Sem o acrescento, um painel acabado de abrir nem sequer entra na
/// ordem e o `PANEL_Z_ORDER_FALLBACK` põe-no no fundo.
pub fn reconcile_z(hero: &mut HeroScreen) {
    let mut visible: Vec<(NodeId, &'static str)> = Vec::new();
    crate::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            if hero.is_panel_visible(p.manifest.id) {
                visible.push((p.manifest.panel_node_id, p.manifest.id));
            }
        }
    });
    hero.store
        .retain_panel_z(|id| visible.iter().any(|(n, _)| n == &id));
    for (node, _) in visible {
        if !hero.store.panel_z_order().contains(&node) {
            hero.store.bump_panel_z(node);
        }
    }
}

/// Os ocupantes de um encaixe, **do fundo para o topo** — o último é o escolhido.
#[must_use]
pub fn occupants(hero: &HeroScreen, slot: Slot) -> Vec<Occupant> {
    let mut found: Vec<(usize, Occupant)> = Vec::new();
    crate::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            let m = &p.manifest;
            // ⛔ Um painel que FLUTUA não ocupa encaixe nenhum: ele tem rect próprio e o artista
            // arrasta-o. Pô-lo numa aba tirar-lhe-ia a razão de ele declarar `CAN_FLOAT`.
            if m.can_float || slot_of(hero, m) != slot || !hero.is_panel_visible(m.id) {
                continue;
            }
            let z = hero
                .store
                .panel_z_order()
                .iter()
                .position(|id| *id == m.panel_node_id)
                // ⚠️ `0` = o fundo. Depois de `reconcile_z` todo painel visível está na
                // ordem, então isto não acontece; se acontecer, o painel perde a selecção em vez
                // de a roubar — *o lado seguro de um desempate é o que não muda o que se vê.*
                .unwrap_or(0);
            found.push((
                z,
                Occupant {
                    id: m.id,
                    node: m.panel_node_id,
                    title: m.title,
                },
            ));
        }
    });
    found.sort_by_key(|(z, _)| *z);
    found.into_iter().map(|(_, o)| o).collect()
}

/// Quantos ocupantes tem cada encaixe, na ordem de [`Slot::ALL`] — o que
/// [`crate::screens::layout::HeroLayout::reserve_slot_tabs`] consome.
#[must_use]
pub fn counts(hero: &HeroScreen) -> [usize; 6] {
    let mut c = [0usize; 6];
    for (i, slot) in Slot::ALL.into_iter().enumerate() {
        c[i] = occupants(hero, slot).len();
    }
    c
}

/// O conjunto de encaixes com pelo menos um ocupante.
#[must_use]
pub fn occupied(hero: &HeroScreen) -> SlotSet {
    let c = counts(hero);
    let mut set = SlotSet::NONE;
    for (i, slot) in Slot::ALL.into_iter().enumerate() {
        if c[i] > 0 {
            set = set.union(SlotSet::of(slot));
        }
    }
    set
}

/// ⭐ **Os painéis que este quadro NÃO deve pintar** — os ocupantes que não estão à frente no seu
/// encaixe.
///
/// ⚠️ Devolve vazio quando cada encaixe tem no máximo um ocupante, e é por isso que o app de hoje é
/// byte-idêntico enquanto o artista não abrir dois painéis do mesmo lado.
#[must_use]
pub fn hidden_by_tabs(hero: &HeroScreen) -> Vec<NodeId> {
    let mut hidden = Vec::new();
    for slot in Slot::ALL {
        let occ = occupants(hero, slot);
        if occ.len() < 2 {
            continue;
        }
        for o in &occ[..occ.len() - 1] {
            hidden.push(o.node);
        }
    }
    hidden
}

/// ⭐ **A ÚNICA porta da geometria de uma fila de abas** — o pintor, o registo de hit e o despacho
/// leem daqui.
///
/// ⚠️ A aritmética de um trilho já mordeu esta linha uma vez: ela vivia em **três** cópias
/// (pintor, hit do trilho, hit do flyout) e nada no repo as ligava — um pintor horizontal com um
/// hit vertical compilava e passava a suíte inteira.
#[must_use]
pub fn tab_rects(bar: Rect, n: usize) -> Vec<Rect> {
    if n == 0 || bar.w <= 0.0 || bar.h <= 0.0 {
        return Vec::new();
    }
    let pad = Spacing::Xs.px();
    let inner = (bar.w - pad * 2.0).max(0.0);
    let each = (inner / n as f32).max(0.0);
    if each < MIN_TAB_W {
        // Transbordo: cabem `fit` abas, e as restantes ficam pelo menu *Window*.
        let fit = ((inner / MIN_TAB_W).floor() as usize).max(1).min(n);
        let each = inner / fit as f32;
        return (0..fit)
            .map(|i| Rect::new(bar.x + pad + each * i as f32, bar.y, each, bar.h))
            .collect();
    }
    (0..n)
        .map(|i| Rect::new(bar.x + pad + each * i as f32, bar.y, each, bar.h))
        .collect()
}

/// Regista os controlos de aba dos painéis registados. Chamado pelo `pre_populate` do hero.
///
/// ⛔⛔ **`with_registry_opt`, nunca `with_registry_ref` — e nas quatro funções deste ficheiro.**
/// O `pre_populate` corre dentro do `HeroScreen::new`, e nem toda a gente que constrói um hero
/// instalou o registry: na própria `ph2d-editor-core` o `test_support::ensure_panel_registry` é um
/// `{}`. A variante `_ref` faz `panic!` com uma mensagem sobre o *host*, e **12 testes de chrome
/// desta crate morreram assim** — nenhum deles tem a ver com painéis. *Uma leitura obrigatória de
/// um recurso opcional transforma um serviço em requisito, e quem paga é quem nunca o pediu.*
///
/// ⚠️ **Sem `InteractiveState` uma aba é pintada e nasce morta**: não é focável, o Down não arma o
/// `active` e o Up nunca emite `Click`. É o defeito que matou o pill `[SHEET]` e os quatro pills de
/// vetor, e o gate `hit_indexed_ids_are_registered` não o veria — estes ids são **derivados**.
pub fn populate(store: &mut WidgetStore) {
    crate::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            store.register(
                tab_node_id(p.manifest.panel_node_id),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    });
}

/// Pinta a fila de abas de um encaixe e regista os alvos.
#[allow(clippy::too_many_arguments)]
pub fn paint_slot_tabs(
    bar: Rect,
    occ: &[Occupant],
    selected: Option<NodeId>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let rects = tab_rects(bar, occ.len());
    if rects.is_empty() {
        return;
    }
    scene.fill_rect(rect_to_vello(bar), resolve(ColorToken::Bg1, theme));
    for (o, r) in occ.iter().zip(rects) {
        let is_on = selected == Some(o.node);
        let state = store
            .button_state(tab_node_id(o.node))
            .unwrap_or(ButtonState::Normal);
        let bg = if is_on {
            Some(ColorToken::BgElev)
        } else if matches!(
            state,
            ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed
        ) {
            Some(ColorToken::Bg2)
        } else {
            None
        };
        if let Some(bg) = bg {
            fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(bg, theme));
        }
        let fg = if is_on {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        };
        paint_text_centered(
            text_system,
            scene,
            o.title,
            r,
            TypeToken::Sm.px(),
            resolve(fg, theme),
        );
        hit_index.register(tab_node_id(o.node), r);
    }
}

/// ⭐ **Clicar numa aba levanta o painel dela** — e é tudo o que uma aba faz.
///
/// Corre no topo do `HeroScreen::apply_event`, pela mesma razão que o fecho da barra de menus: o
/// registo de painéis é caminhado **antes** do `chrome::dispatch_all`, então um `Click` sobre um id
/// derivado de painel nunca chegaria a um handler de chrome.
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if let Some(node) = panel_for_tab(id) {
        hero.store.bump_panel_z(node);
        return true;
    }
    false
}

/// ⭐⭐⭐ **AS ZONAS DE LARGADA de um arrasto em curso** — e é aqui que a **D1** deixa de ser uma
/// verificação e passa a ser um `Constraint`.
///
/// > *«O erro não é detectado, é **inexprimível**.»* — `00_DECISOES_DO_ENIO.md`, D4
///
/// Um encaixe que o painel **não** permite simplesmente **não é oferecido**: não se pinta, não se
/// testa, não existe para este gesto. ⛔ A alternativa — aceitar a largada e depois recusá-la — é a
/// forma que o Enio nomeou como errada: *o artista faz o gesto, vê a resposta, e não sabe porquê.*
///
/// ⚠️ **O encaixe de ONDE ele veio é oferecido também**, e de propósito: largar de volta é como se
/// desiste de um arrasto sem precisar de saber que a tecla `Esc` existe.
///
/// Devolve `(encaixe, rect)` para cada destino legal, na ordem de [`Slot::ALL`].
#[must_use]
pub fn drop_targets(hero: &HeroScreen, panel: NodeId) -> Vec<(Slot, Rect)> {
    let Some(layout) = hero.last_layout else {
        return Vec::new();
    };
    let allowed = crate::panel::with_registry_opt(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.panel_node_id == panel)
            .map(|p| p.manifest.allowed_slots)
    })
    .flatten();
    let Some(allowed) = allowed else {
        return Vec::new();
    };
    let rects = layout.slot_rects(occupied(hero));
    allowed
        .iter()
        .filter_map(|slot| {
            let r = rects.get(slot);
            (r.w > 0.0 && r.h > 0.0).then_some((slot, r))
        })
        .collect()
}

/// ⭐⭐ **RESOLVE a largada** — corre no início do quadro, e consome o pedido uma vez só.
///
/// ⚠️ **Ele julga contra o layout do quadro ANTERIOR, e isso é o correcto**, não um compromisso: a
/// largada tem de ser medida contra a geometria que o artista estava a ver quando largou. Julgá-la
/// contra um layout já reconstruído com o painel movido seria perguntar ao futuro.
pub fn resolve_tab_drop(hero: &mut HeroScreen) {
    let Some((panel, (x, y))) = hero.store.take_tab_drop() else {
        return;
    };
    for (slot, r) in drop_targets(hero, panel) {
        if r.contains(x, y) {
            hero.store.set_panel_slot(panel, slot);
            // ⭐ E o painel largado fica à FRENTE no encaixe novo — senão ele desaparece atrás de
            // quem já lá estava, e o artista conclui que o gesto falhou.
            hero.store.bump_panel_z(panel);
            return;
        }
    }
    // ⚠️ Largar fora de todo destino legal **não faz nada**, e não é um erro: é a forma de
    // desistir. ⛔ Nenhuma mensagem — um aviso por cada gesto abandonado seria ruído.
}

/// Pinta as zonas de largada e a etiqueta que segue o dedo. No-op sem arrasto em curso.
#[allow(clippy::too_many_arguments)]
pub fn paint_drag_overlay(
    hero: &HeroScreen,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let Some((panel, cursor)) = hero.store.tab_being_dragged() else {
        return;
    };
    let title = crate::panel::with_registry_opt(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.panel_node_id == panel)
            .map(|p| p.manifest.title)
    })
    .flatten()
    .unwrap_or("");

    for (_, r) in drop_targets(hero, panel) {
        let under = r.contains(cursor.0, cursor.1);
        let token = if under {
            ColorToken::AccentSoft
        } else {
            ColorToken::BgElev
        };
        fill_rounded_rect(scene, r, Radius::Sm.px(), resolve(token, theme));
        if under {
            crate::paint::stroke_rounded_rect(
                scene,
                r,
                Radius::Sm.px(),
                DROP_OUTLINE_PX,
                resolve(ColorToken::Accent, theme),
            );
        }
    }

    // A etiqueta segue o dedo — é o que diz *o que* está a ser movido.
    let w = DRAG_LABEL_W;
    let ghost = Rect::new(cursor.0 - w * 0.5, cursor.1 - TAB_BAR_H * 0.5, w, TAB_BAR_H);
    fill_rounded_rect(
        scene,
        ghost,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        title,
        ghost,
        TypeToken::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
}

#[cfg(test)]
#[path = "slot_tabs_tests.rs"]
mod tests;
