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
//! deste painel e a trilha dela é este rectângulo* — e o despachante lê isto **antes** da tabela.
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

    /// ⭐ **O painel que uma barra rola** — o publicado pela porta, e só depois a tabela antiga
    /// do despachante (os painéis que ainda não passam pela porta).
    #[must_use]
    pub fn scroll_bar_panel(&self, bar: NodeId) -> Option<NodeId> {
        self.scroll
            .bar_owner
            .get(&bar)
            .map(|(p, _)| *p)
            .or_else(|| crate::interaction::dispatch::scroll::scrollbar_panel_for_id(bar))
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
