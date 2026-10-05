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

// ⭐ **O gesto de arrastar uma aba vive no irmão [`super::slot_tabs_drag`]** (corte por tecto de
// LOC, 2026-09-08) e é re-exportado AQUI: `slot_tabs` continua a ser o endereço único da feature.
// ⛔ Sem esta linha, um corte por tamanho obrigaria trinta chamadores a aprender uma segunda
// morada — e o tecto de um ficheiro não é um facto sobre a API dele.
pub use super::slot_tabs_drag::{
    drop_targets, paint_drag_overlay, resolve_tab_drop, tab_drop_caret,
};
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect_radii, rect_to_vello, resolve};
use crate::screens::slot::{Slot, SlotSet};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme};
use ph2d_vector::VectorScene;

/// A altura da fila de abas — **uma linha**, o mesmo token da barra de menus e de uma linha de
/// menu. ⚠️ Não é um número escolhido: uma aba é um rótulo clicável, que é o que uma linha é.
pub const TAB_BAR_H: f32 = ROW_H_PX;

// ⭐ **A CARA de uma aba vive no irmão [`super::slot_tabs_face`]** — o glifo, o nome e o recuo
// deles. Re-exportado AQUI pela mesma razão que o gesto de arrastar: um corte por tecto de LOC
// não pode obrigar os chamadores a aprender uma segunda morada.
pub use super::slot_tabs_face::{
    TabFace, face as tab_face, natural_w as tab_natural_w, tab_bg, tab_dividers, tab_icon_px,
    tab_pad_x, tab_radii, tab_row_bg,
};

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
    /// O `Panel::TITLE` — a CHAVE do que o artista lê na aba. ⚠️ Quem desenha a aba traduz, e é a
    /// mesma chave que o painel usa para se nomear a si próprio no cabeçalho.
    pub title: crate::panel::TextKey,
    /// O `Panel::ICON` — o que a aba diz depois de o nome já não caber.
    pub icon: crate::icons::IconId,
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
    let mut found: Vec<Occupant> = Vec::new();
    crate::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            let m = &p.manifest;
            // ⛔ Um painel que FLUTUA não ocupa encaixe nenhum: ele tem rect próprio e o artista
            // arrasta-o. Pô-lo numa aba tirar-lhe-ia a razão de ele declarar `CAN_FLOAT`.
            if m.can_float || slot_of(hero, m) != slot || !hero.is_panel_visible(m.id) {
                continue;
            }
            found.push(Occupant {
                id: m.id,
                node: m.panel_node_id,
                title: m.title,
                icon: m.icon,
            });
        }
    });
    // ⭐⭐⭐ **A ordem é a ARRUMADA por cima da do registo** (report do dono, 2026-09-08: *«não é
    // possível reordenar as abas arrastando com o mouse»*).
    //
    // ⚠️ `sort_by_key` é **estável**: quem o artista nunca arrumou fica onde o registo o pôs, e
    // vem depois de quem ele arrumou. É isso que faz um app acabado de instalar ter a ordem do
    // registo sem uma linha de semente.
    //
    // ⛔ E é aqui, e só aqui, que a ordem se decide — a ordem z continua a responder **apenas**
    // *quem está à frente* ([`chosen`]). As duas perguntas separaram-se em 2026-09-07 e não
    // voltam a juntar-se.
    let order = hero.store.panel_tab_order();
    found.sort_by_key(|o| {
        order
            .iter()
            .position(|n| *n == o.node)
            .unwrap_or(usize::MAX)
    });
    found
}

/// A posição deste painel na ordem z, ou `0` (o fundo) se ele não estiver nela.
///
/// ⚠️ Depois de [`reconcile_z`] todo painel visível está na ordem, então o `0` não acontece; se
/// acontecer, o painel **perde** a selecção em vez de a roubar — *o lado seguro de um desempate é
/// o que não muda o que se vê.*
fn z_of(hero: &HeroScreen, node: NodeId) -> usize {
    hero.store
        .panel_z_order()
        .iter()
        .position(|id| *id == node)
        .unwrap_or(0)
}

/// ⭐⭐⭐ **Qual dos ocupantes está À FRENTE** — e a ordem z responde a ESTA pergunta, só a esta.
///
/// # ⛔⛔ O report que separou as duas perguntas
///
/// > *«quando se clica na aba ela troca de lugar com a outra aba. não permita isso»* — Enio,
/// > 2026-09-07, no smoke da wave 34.
///
/// A fila era **ordenada pela ordem z**, e a ordem z é *«quem foi tocado por último»*. ⇒ tocar numa
/// aba mandava-a para o fim da fila: o mesmo facto respondia a **duas** perguntas — *quem está à
/// frente* e *em que ordem elas se sentam* — e responder à primeira mexia na segunda.
///
/// ⇒ **a ORDEM é a do registo** (estável, e a mesma em toda sessão), e a **escolha** é o topo do z.
/// *Uma aba só muda de lugar quando o artista a ARRASTA* — que é o gesto que o
/// [`resolve_tab_drop`] já serve, e o único que deve mover uma.
#[must_use]
pub fn chosen(hero: &HeroScreen, slot: Slot) -> Option<NodeId> {
    occupants(hero, slot)
        .into_iter()
        .max_by_key(|o| z_of(hero, o.node))
        .map(|o| o.node)
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
        // ⚠️ **Todos MENOS o escolhido**, e não «todos menos o último»: desde 2026-09-07 a fila está
        //    na ordem do REGISTO, logo o último dela já não é quem está à frente. Ver [`chosen`].
        let front = chosen(hero, slot);
        for o in &occ {
            if Some(o.node) != front {
                hidden.push(o.node);
            }
        }
    }
    hidden
}

/// ⭐⭐⭐ **O PISO de uma aba: ela nunca é mais estreita do que é ALTA.**
///
/// ⛔ **Não é um número escolhido** — é a altura da própria fila ([`TAB_BAR_H`], que é o
/// `ROW_H_PX`). Um quadrado é a menor coisa que ainda se lê como um alvo, e desde 2026-09-09 ele
/// tem conteúdo: `22 − 14` ([`ph2d_tokens::INLINE_ICON_PX`]) deixa **4 px de cada lado**, que é o
/// respiro padrão desta casa. ⇒ *no piso a aba é exactamente o glifo dela* — ver
/// [`super::slot_tabs_face`].
fn tab_floor_w() -> f32 {
    TAB_BAR_H
}

/// ⭐⭐⭐ **Encolhe as larguras naturais até caberem, nunca abaixo do piso** — `None` quando nem no
/// piso elas cabem, e aí quem responde é a janela deslizante do [`tab_layout`].
///
/// # ⛔ O buraco que isto fecha
///
/// Uma aba que não é PINTADA não tem rect, logo não está no índice de acerto, logo **não se
/// clica**: o painel dela só voltava por um caminho que ninguém adivinha (fechá-lo e reabri-lo no
/// menu *Window*). E bastava a coluna ser estreita — três nomes desta casa medem ~231 px, e a
/// largura mínima de uma coluna deixa **212** úteis.
///
/// ⇒ enquanto `n × piso ≤ inner`, **toda aba é pintada**. Na coluna da direita de fábrica (296 px
/// úteis) isso dá **13** abas, que é exactamente a população máxima daquele encaixe — *o transbordo
/// deixa de ser alcançável pelo caminho normal do artista*, e a afordância `⋯` passa a servir só a
/// coluna espremida.
///
/// ⚠️ **A elisão do nome vem de graça** e não é conta desta função: o `paint_text_centered` corta
/// o texto ao orçamento do rect desde 2026-09-06. *Encolher o rect sem elidir escreveria o nome
/// por cima da aba vizinha.*
///
/// ⚠️ **O laço é iterativo, e não uma regra de três:** quem bate no piso deixa de encolher, e o
/// que ele não cedeu tem de ser redistribuído pelos outros. Cada passo ou faz caber ou prende mais
/// uma no piso, logo `n` passos bastam.
fn fitted_widths(natural: &[f32], inner: f32) -> Option<Vec<f32>> {
    let floor = tab_floor_w();
    if natural.is_empty() || floor * natural.len() as f32 > inner {
        return None;
    }
    let mut w = natural.to_vec();
    for _ in 0..=natural.len() {
        let total: f32 = w.iter().sum();
        if total <= inner {
            return Some(w);
        }
        let free: f32 = w.iter().filter(|x| **x > floor).sum();
        let fixed: f32 = w.iter().filter(|x| **x <= floor).sum();
        let room = inner - fixed;
        if free <= 0.0 || room <= 0.0 {
            break;
        }
        let k = room / free;
        for x in w.iter_mut() {
            if *x > floor {
                *x = (*x * k).max(floor);
            }
        }
    }
    // A rede: o piso para todas cabe por construção (foi verificado à entrada). Ela existe para o
    // caso de a aritmética em `f32` não convergir no orçamento de passos — nunca para decidir.
    Some(vec![floor; natural.len()])
}

/// ⭐ **A ÚNICA porta da geometria de uma fila de abas** — o pintor, o registo de hit e o despacho
/// leem daqui.
///
/// ⚠️ A aritmética de um trilho já mordeu esta linha uma vez: ela vivia em **três** cópias
/// (pintor, hit do trilho, hit do flyout) e nada no repo as ligava — um pintor horizontal com um
/// hit vertical compilava e passava a suíte inteira.
#[must_use]
pub fn tab_rects(bar: Rect, widths: &[f32]) -> Vec<Rect> {
    if widths.is_empty() || bar.w <= 0.0 || bar.h <= 0.0 {
        return Vec::new();
    }
    let mut x = bar.x + Spacing::Xs.px();
    widths
        .iter()
        .map(|w| {
            let r = Rect::new(x, bar.y, *w, bar.h);
            x += *w;
            r
        })
        .collect()
}

/// ⭐⭐⭐ **A LARGURA de uma aba é a do NOME dela** — a lei do modelo, e a cura de um defeito medido.
///
/// ⛔⛔ **Repartir a fila em partes iguais faz dois painéis diferentes lerem-se IGUAIS.** Medido em
/// 2026-09-07 sobre a coluna no mínimo (220 px) com três abas: cada uma ficava com `70,67 px`, o
/// orçamento de texto com `54,67`, e *«Audio Editor»* e *«Audio Mixer»* elidiam **as duas** para
/// `Audio …`. A elisão estava a fazer o que promete — o defeito é ela receber um orçamento que
/// deita fora exactamente o que DISTINGUE, e nenhuma largura igual o evita.
///
/// ⇒ cada aba mede `prefix_width(título) + 2 · recuo`, como no `TabBar` do Godot e como as abas de
/// LAYOUT desta casa já faziam ([`super::layout_tabs::tab_rects`]) — *o desenho pedido já era lei
/// na outra metade do app.*
///
/// ⚠️ **As abas ENCOSTAM, sem vão** — a lei do grupo (wave 10): *o que separa duas peças é a QUINA,
/// não o espaço*. É a quina de cima e a cor que as separam, e é isso que as faz ler como uma fila.
fn tab_widths(occ: &[Occupant], text_system: &mut TextSystem) -> Vec<f32> {
    occ.iter()
        .map(|o| super::slot_tabs_face::natural_w(o.title.tr(), text_system))
        .collect()
}

/// ⭐⭐⭐ **QUEM aparece na fila, e ONDE** — a porta que emparelha ocupantes com rects.
///
/// ⛔⛔ **O emparelhamento ingénuo é `occ.iter().zip(tab_rects(bar, occ.len()))`, e ele descarta
/// SEMPRE a aba escolhida.** O `zip` trunca pelo mais curto, os rects são `fit` no transbordo, e o
/// escolhido é o **último** da ordem z (`panel_walk`: `occ.last()`). ⇒ com mais ocupantes do que
/// cabem, **nenhuma das abas pintadas acende**, e o painel que está a desenhar não tem aba nenhuma.
/// Foi metade do report de 2026-09-07 (*«abas espremidas»*, nenhuma marcada).
///
/// ⇒ **a janela é a do TOPO da ordem z**, isto é, os `fit` mais recentes — e ela contém o escolhido
/// por construção, porque ele é o último.
///
/// ⚠️ **Isto é uma PORTA e não uma correcção no pintor**: o `zip` estava copiado em cinco sítios
/// (o pintor e quatro testes), e *uma lei escrita em dois sítios ainda não é uma lei*. Quem
/// precisar de saber que aba está onde chama isto.
#[must_use]
pub fn tab_layout(
    occ: &[Occupant],
    front: Option<NodeId>,
    bar: Rect,
    text_system: &mut TextSystem,
) -> Vec<(Occupant, Rect)> {
    let Some(plan) = tab_plan(occ, front, bar, text_system) else {
        return Vec::new();
    };
    occ[plan.start..plan.start + plan.widths.len()]
        .iter()
        .copied()
        .zip(tab_rects(plan.bar, &plan.widths))
        .collect()
}

/// ⭐⭐ **O PLANO de uma fila** — quem é pintado, com que largura, em que faixa, e quantos ficam de
/// fora de cada lado.
///
/// ⚠️ **Existe porque a saída de transbordo precisa da MESMA janela que o pintor usa.** Perguntar
/// *«há abas escondidas?»* por uma segunda conta seria duas respostas à mesma pergunta, e elas
/// divergiriam no primeiro caso que uma delas não visse — o defeito que este ficheiro já pagou com
/// a aritmética do trilho, escrita em três cópias.
pub struct TabPlan {
    /// O índice, em `occ`, do primeiro ocupante PINTADO.
    pub start: usize,
    /// As larguras dos pintados, na ordem em que aparecem.
    pub widths: Vec<f32>,
    /// A faixa que sobra para as abas — já **sem** o espaço das setas, quando elas existem.
    pub bar: Rect,
    /// Quantos ocupantes ficam escondidos antes e depois da janela.
    pub hidden_before: usize,
    pub hidden_after: usize,
}

/// Ver [`TabPlan`]. `None` quando não há fila nenhuma para dispor.
#[must_use]
pub fn tab_plan(
    occ: &[Occupant],
    front: Option<NodeId>,
    bar: Rect,
    text_system: &mut TextSystem,
) -> Option<TabPlan> {
    if occ.is_empty() || bar.w <= 0.0 || bar.h <= 0.0 {
        return None;
    }
    let widths = tab_widths(occ, text_system);
    let inset = Spacing::Xs.px() * 2.0;

    // ⭐⭐⭐ **PRIMEIRO tenta-se dar aba a TODOS, na faixa INTEIRA** — ver [`fitted_widths`]. É esta
    // tentativa que decide se há setas: reservar-lhes espaço antes de saber se são precisas faria
    // uma fila que cabia deixar de caber por causa de uma saída que ela não usa.
    if let Some(w) = fitted_widths(&widths, (bar.w - inset).max(0.0)) {
        return Some(TabPlan {
            start: 0,
            widths: w,
            bar,
            hidden_before: 0,
            hidden_after: 0,
        });
    }

    // Transborda ⇒ as duas setas ocupam o fim da faixa, e as abas dispõem-se no que sobra.
    let bar = Rect::new(
        bar.x,
        bar.y,
        (bar.w - super::slot_tabs_overflow::arrow_w() * 2.0).max(0.0),
        bar.h,
    );
    let inner = (bar.w - inset).max(0.0);
    if inner <= 0.0 {
        return None;
    }

    // Quantas cabem a partir de `start`. ⚠️ A primeira entra SEMPRE, aparada — um nome mais largo
    // que a coluna inteira ainda tem de ter aba, senão o painel que desenha fica sem nenhuma.
    let fits_from = |start: usize| -> usize {
        let mut used = widths[start].min(inner);
        let mut n = 1;
        while start + n < occ.len() {
            let w = widths[start + n];
            if used + w > inner {
                break;
            }
            used += w;
            n += 1;
        }
        n
    };

    // ⭐⭐ **A janela começa no PRINCÍPIO e só desliza quando o escolhido não cabe nela.**
    //
    // ⛔ Ela crescia para trás a partir do fim, o que só funcionava enquanto a fila estivesse
    //    ordenada por z — e era essa ordenação que fazia uma aba trocar de lugar ao ser tocada
    //    (report do dono, 2026-09-07). Com a ordem estável, a fila comporta-se como uma fila:
    //    as abas ficam onde estão, e o que se move é a JANELA.
    let want = front
        .and_then(|c| occ.iter().position(|o| o.node == c))
        .unwrap_or(0);
    let mut start = 0usize;
    while start < want && start + fits_from(start) <= want {
        start += 1;
    }
    let n = fits_from(start);

    let mut shown: Vec<f32> = widths[start..start + n].to_vec();
    if let Some(first) = shown.first_mut() {
        *first = first.min(inner);
    }
    Some(TabPlan {
        start,
        widths: shown,
        bar,
        hidden_before: start,
        hidden_after: occ.len() - (start + n),
    })
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
    // ⭐ As duas setas de cada encaixe — ver [`super::slot_tabs_overflow`].
    super::slot_tabs_overflow::populate(store);
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
    slot: Slot,
    occ: &[Occupant],
    selected: Option<NodeId>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let Some(plan) = tab_plan(occ, selected, bar, text_system) else {
        return;
    };
    let painted: Vec<(Occupant, Rect)> = occ[plan.start..plan.start + plan.widths.len()]
        .iter()
        .copied()
        .zip(tab_rects(plan.bar, &plan.widths))
        .collect();
    if painted.is_empty() {
        return;
    }
    let painted_for_dividers = painted.clone();
    // ⚠️ **O chão é o da faixa INTEIRA**, e não o da parte que sobrou para as abas: as setas vivem
    //    dentro da fila, não ao lado dela.
    scene.fill_rect(rect_to_vello(bar), resolve(tab_row_bg(), theme));
    super::slot_tabs_overflow::paint(
        bar,
        slot,
        plan.hidden_before,
        plan.hidden_after,
        scene,
        theme,
        hit_index,
    );
    let radii = tab_radii(theme);
    for (o, r) in painted {
        let is_on = selected == Some(o.node);
        let state = store
            .button_state(tab_node_id(o.node))
            .unwrap_or(ButtonState::Normal);
        if let Some(bg) = tab_bg(is_on, state) {
            fill_rounded_rect_radii(scene, r, radii, resolve(bg, theme));
        }
        let fg = if is_on {
            ColorToken::Text1
        } else {
            ColorToken::Text2
        };
        super::slot_tabs_face::paint(scene, text_system, r, o.icon, o.title.tr(), fg, theme);
        hit_index.register(tab_node_id(o.node), r);
    }
    // ⚠️ **Depois das abas**, para a divisória não ficar por baixo do corpo da escolhida.
    for d in tab_dividers(&painted_for_dividers, selected) {
        scene.fill_rect(rect_to_vello(d), resolve(ColorToken::Border, theme));
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
    // ⭐ As setas de transbordo — elas escolhem o ocupante seguinte, e a janela segue-o.
    super::slot_tabs_overflow::apply_event(hero, id)
}

/// ⭐⭐ **REPÕE A ARRUMAÇÃO DE FÁBRICA** — a porta do *Reset Panel Layout*.
///
/// > *«Precisamos da opção de resetar.»* — Enio, 2026-08-30
///
/// As **três** coisas que uma arrumação é, e as três têm de voltar juntas:
///
/// | o que volta | de onde |
/// |---|---|
/// | onde cada painel está | as excepções de encaixe apagadas ⇒ vale o `DEFAULT_SLOT` |
/// | quais estão abertos | vale o `DEFAULT_VISIBLE` |
/// | a largura das colunas | as escolhas apagadas ⇒ vale o `ChromeBands::DEFAULT` |
///
/// ⛔ **Repor duas de três não é repor**: o artista clica, vê o ecrã mudar, e conclui que funcionou
/// — e o terço que ficou volta a mordê-lo mais tarde, sem ligação com o gesto que o deixou.
///
/// ⚠️ **Ele não toca no ficheiro.** O que está gravado é uma **projecção** do que o app tem agora, e
/// o detector do quadro grava a projecção vazia sozinho. *Apagar o ficheiro aqui seria um segundo
/// caminho para o mesmo facto.*
pub fn reset(hero: &mut HeroScreen) {
    hero.store.reset_panel_layout();
    crate::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            hero.panel_visibility
                .insert(p.manifest.id, p.manifest.default_visible);
        }
    });
}

#[cfg(test)]
#[path = "slot_tabs_tests.rs"]
mod tests;
