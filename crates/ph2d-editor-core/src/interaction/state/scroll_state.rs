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
use std::collections::BTreeMap;

/// O estado da rolagem que não é um escalar por painel.
#[derive(Debug, Default)]
pub struct ScrollState {
    /// `id da barra → (painel, trilha)`, publicado pela porta a cada quadro.
    bar_owner: BTreeMap<NodeId, (NodeId, Rect)>,
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
    pub fn wheel_panel(&mut self, panel: NodeId, delta_y: f32) {
        self.stop_fling(panel);
        let mut next = (self.panel_scroll_target(panel) - delta_y).max(0.0);
        if let Some(content_h) = self.panel_content_h(panel) {
            let visible_h = self.panel_visible_h(panel).unwrap_or_else(|| {
                self.panel_rect(panel)
                    .map_or(0.0, |r| (r.h - 60.0).max(0.0)) // LITERAL-PX-OK: palpite do 1.º quadro, anterior à publicação
            });
            next = next.min((content_h - visible_h).max(0.0));
        }
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
