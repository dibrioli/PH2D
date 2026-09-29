//! ⭐⭐⭐ **O estado da rolagem ÚNICA** (spec `04_a_rolagem_unica`) — o dono de cada barra, as
//! amostras do dedo e a inércia em voo.
//!
//! Mora num ficheiro próprio porque o `state/mod.rs` está no tecto de LOC; o `WidgetStore` leva
//! UM campo ([`ScrollState`]) e as portas vivem aqui.
//!
//! ## O dono de uma barra é PUBLICADO, não escrito numa tabela
//!
//! Até 2026-09-29 a pergunta *«esta barra rola que painel?»* tinha uma resposta escrita à mão (o
//! `scrollbar_panel_for_id` do despachante), e um painel novo que não soubesse dela emprestava o
//! id de outro: o painel de ossos pintava com o id do Vector, e arrastar a barra dele **rolava o
//! Vector**. A porta [`crate::widget::scroll_area`] publica aqui, a cada quadro, *esta barra é
//! deste painel e a trilha dela é este rectângulo* — e esta é a ÚNICA resposta: a tabela à mão
//! morreu no mesmo dia, quando o último painel passou pela porta.
//! ⇒ um painel que passa pela porta não tem onde escrever o id do vizinho.
//!
//! ⚠️ **A TRILHA viaja junto com o dono**, e é ela que cura o arrasto rápido demais: o despachante
//! lia a altura do rect ACERTADO como se fosse a da trilha, e 14 painéis registavam só o polegar.

use super::WidgetStore;
use crate::interaction::fling;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use std::collections::{BTreeMap, BTreeSet};

/// O estado da rolagem que não é um escalar por painel.
#[derive(Debug, Default)]
pub struct ScrollState {
    /// `id da barra → (painel, trilha)`, publicado pela porta a cada quadro.
    bar_owner: BTreeMap<NodeId, (NodeId, Rect)>,
    /// Quem publicou as alturas NESTE quadro — ver [`WidgetStore::end_scroll_frame`].
    heights_this_frame: BTreeSet<NodeId>,
    /// As barras publicadas NESTE quadro.
    bars_this_frame: BTreeSet<NodeId>,
    /// `painel → velocidade de rolagem em px/s` enquanto a lista voa depois de largada.
    fling: BTreeMap<NodeId, f32>,
    /// `(tempo_ns, y)` dos `Move` do arrasto no corpo que está em curso.
    samples: Vec<(u128, f32)>,
}

/// Quantas amostras se guardam, no máximo. ⚠️ Não é um tecto de produto: o [`fling::HORIZON_NS`]
/// já descarta as velhas; isto só impede um arrasto de uma hora de crescer o vector sem fim. A
/// `1000 Hz` (o rato mais rápido que esta máquina entrega) a janela de 100 ms tem `100` amostras.
const SAMPLES_CAP: usize = 256;

impl WidgetStore {
    /// ⭐ **A porta publica o dono e a trilha da barra que acabou de pintar.**
    pub fn publish_scroll_bar(&mut self, bar: NodeId, panel: NodeId, track: Rect) {
        self.scroll.bar_owner.insert(bar, (panel, track));
        self.scroll.bars_this_frame.insert(bar);
    }

    /// O painel `id` publicou uma das alturas neste quadro (os dois `set_panel_*_h` chamam isto).
    pub(super) fn mark_heights_published(&mut self, id: NodeId) {
        self.scroll.heights_this_frame.insert(id);
    }

    /// ⭐ **O quadro de pintura começa** — ver [`Self::end_scroll_frame`].
    pub fn begin_scroll_frame(&mut self) {
        self.scroll.heights_this_frame.clear();
        self.scroll.bars_this_frame.clear();
    }

    /// ⭐⭐ **O quadro de pintura acaba: o que NÃO foi publicado nele é esquecido** (D9 da spec
    /// `04_a_rolagem_unica`).
    ///
    /// ⛔ As alturas (`content_h`/`visible_h`) e o dono de cada barra só eram escritos — nunca
    /// apagados. Um painel que deixava de passar pela porta (o estado vazio da escultura, que pinta
    /// o rect e nenhuma lista; um menu suspenso que fechou) ficava com as alturas do ÚLTIMO quadro em
    /// que rolava: a roda continuava a mexer num alvo que ninguém desenhava, o arrasto no corpo armava
    /// e a inércia voava sobre o vazio, e quando a lista voltava ela aparecia noutro sítio.
    ///
    /// ⇒ as três tabelas passam a descrever **o último quadro pintado**, sem mais: quem publicou
    /// fica, quem não publicou sai, e um voo sobre um painel que saiu **pára** (senão o tique, sem
    /// alturas, lia `max = 0` e puxava o alvo a zero — perdendo a posição que o artista deixou).
    /// ⚠️ O ALVO de rolagem (`panel_scroll`) **não** é esquecido: fechar e reabrir um painel volta ao
    /// sítio onde estava, e a porta prende-o ao conteúdo novo ao publicar.
    ///
    /// ⚠️ Só a pintura do ecrã chama o par; quem escreve alturas fora dela (um teste, uma cena)
    /// não é varrido.
    pub fn end_scroll_frame(&mut self) {
        let heights = &self.scroll.heights_this_frame;
        self.panel_content_h.retain(|id, _| heights.contains(id));
        self.panel_visible_h.retain(|id, _| heights.contains(id));
        self.scroll.fling.retain(|id, _| heights.contains(id));
        let bars = &self.scroll.bars_this_frame;
        self.scroll.bar_owner.retain(|id, _| bars.contains(id));
    }

    /// ⭐ **O painel que uma barra rola** — o que a porta publicou ao pintá-la, e mais nada.
    ///
    /// ⚠️ A tabela à mão do despachante (`scrollbar_panel_for_id`) morreu em 2026-09-29: toda barra
    /// passa pela porta, e uma barra nunca pintada não pode ser carregada. `None` antes do 1.º
    /// quadro é a resposta certa — ainda não há barra nenhuma.
    #[must_use]
    pub fn scroll_bar_panel(&self, bar: NodeId) -> Option<NodeId> {
        self.scroll.bar_owner.get(&bar).map(|(p, _)| *p)
    }

    /// A TRILHA publicada de uma barra, se ela passa pela porta.
    #[must_use]
    pub fn scroll_bar_track(&self, bar: NodeId) -> Option<Rect> {
        self.scroll.bar_owner.get(&bar).map(|(_, t)| *t)
    }

    /// As barras publicadas pela porta — o acendimento no hover percorre-as.
    pub fn published_scroll_bars(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.scroll.bar_owner.keys().copied()
    }

    /// O arrasto no corpo começou: esquece as amostras do anterior.
    pub(crate) fn clear_body_scroll_samples(&mut self) {
        self.scroll.samples.clear();
    }

    /// Uma amostra do dedo durante o arrasto no corpo.
    pub(crate) fn push_body_scroll_sample(&mut self, t_ns: u128, y: f32) {
        if self.scroll.samples.len() >= SAMPLES_CAP {
            self.scroll.samples.remove(0);
        }
        self.scroll.samples.push((t_ns, y));
    }

    /// ⭐ **O dedo saiu do corpo: a lista LANÇA-SE**, se ele ia depressa o bastante.
    ///
    /// Chamado pelo `Up` **antes** de o arrasto acabar (precisa do painel dele).
    pub(crate) fn release_body_scroll(&mut self, t_up_ns: u128) {
        let Some(anchor) = self.body_scroll_drag() else {
            return;
        };
        let v = fling::release_velocity(&self.scroll.samples, t_up_ns);
        self.scroll.samples.clear();
        if let Some(scroll_v) = fling::launch(v) {
            self.scroll.fling.insert(anchor.panel, scroll_v);
        }
    }

    /// ⭐ **A roda sobre `panel`** — a lei ÚNICA dela, para todo corpo rolável (os painéis pelo
    /// `dispatch_wheel`, a paleta de comandos pela shell, que toma a roda da tela inteira).
    ///
    /// * O **alvo**, nunca o vivo: girar depressa sobre uma posição em voo andaria menos do que o
    ///   dedo pediu.
    /// * A roda **toma o comando de volta** de uma lista em voo.
    /// * `delta_y > 0` (winit) é «para a frente»: o conteúdo sobe, logo o deslocamento DESCE.
    /// * Presa ao fim publicado (`content_h − visible_h`); sem o fim, a pintura seguinte prendia-a de
    ///   volta com um salto de um quadro («saltos indesejados se rodamos a roda no fim»). Antes da
    ///   1.ª publicação da altura visível vale o palpite `rect.h − 60`.
    ///
    /// ⛔ **Sem altura publicada, a roda NÃO mexe** (D9): antes, `content_h` em falta queria dizer
    /// *«sem tecto»* e a roda sobre um painel que não desenha lista nenhuma (o estado vazio da
    /// escultura) somava sem fim num alvo que ninguém lia — e a lista reaparecia lá em baixo. Todo
    /// pintor que lê a rolagem publica as alturas pela porta, logo esta ausência só acontece onde
    /// não há nada para rolar.
    pub fn wheel_panel(&mut self, panel: NodeId, delta_y: f32) {
        self.stop_fling(panel);
        let Some(content_h) = self.panel_content_h(panel) else {
            return;
        };
        let visible_h = self.panel_visible_h(panel).unwrap_or_else(|| {
            self.panel_rect(panel)
                .map_or(0.0, |r| (r.h - 60.0).max(0.0)) // LITERAL-PX-OK: palpite do 1.º quadro, anterior à publicação
        });
        let next = (self.panel_scroll_target(panel) - delta_y)
            .max(0.0)
            .min((content_h - visible_h).max(0.0));
        self.set_panel_scroll(panel, next);
    }

    /// Pegar no conteúdo em voo SEGURA-O — a lei de todo sistema tátil.
    pub fn stop_fling(&mut self, panel: NodeId) {
        self.scroll.fling.remove(&panel);
    }

    /// Os painéis em voo e a velocidade de cada um.
    pub fn flings(&self) -> impl Iterator<Item = (NodeId, f32)> + '_ {
        self.scroll.fling.iter().map(|(p, v)| (*p, *v))
    }

    /// O tique escreve a velocidade que sobrou, ou acaba o voo com `None`.
    pub(crate) fn set_fling(&mut self, panel: NodeId, v: Option<f32>) {
        match v {
            Some(v) => {
                self.scroll.fling.insert(panel, v);
            }
            None => {
                self.scroll.fling.remove(&panel);
            }
        }
    }

    /// ⭐ **O conteúdo deste painel está a ser COMANDADO 1:1** — pelo dedo no corpo, pelo polegar
    /// da barra, ou pela inércia que o dedo deixou.
    ///
    /// Aí a mola da suavidade fica de fora: ela existe para a roda, que nomeia um DESTINO; um dedo
    /// segura o conteúdo, e um conteúdo que segue o dedo por uma mola escorrega debaixo dele.
    #[must_use]
    pub fn scroll_is_direct(&self, panel: NodeId) -> bool {
        self.body_scroll_drag().is_some_and(|a| a.panel == panel)
            || self.scrollbar_drag().is_some_and(|a| a.panel == panel)
            || self.scroll.fling.contains_key(&panel)
    }
}

#[cfg(test)]
#[path = "scroll_state_tests.rs"]
mod tests;
