//! O botão «Rascunho ↔ Final» (a onda) e a opção Rascunho/Final da barra da forma: UMA porta para a
//! luz e o clique (doc 17 §9).

use ph2d_board_model::{BoardDoc, BoardOp, Element, Letter};

use crate::{Editor, History};

impl Editor {
    /// ⭐ **O botão «Rascunho ↔ Final»** (W4): o que ele alcança ([`Self::sketch_targets`]) — a
    /// selecção, ou o quadro INTEIRO, e então devolve o modo novo do quadro (`Some`), que é também o
    /// das formas que nascerem, no MESMO passo ([`History::apply_mode`]). Dá o contrário da luz
    /// ([`Self::sketch_lit`]). As notas entram só na letra; os traços livres da caneta não entram (já
    /// são mão), a tinta limpa sim. UM passo. A letra
    /// fixa de um `.excalidraw` (`Letter::Hand`/`Sans`) volta ao modo (`Auto`) em todo o alcançado. Um
    /// texto aberto fecha-se primeiro, no seu próprio passo.
    pub fn toggle_sketch(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        board: bool,
    ) -> Option<bool> {
        let on = !self.sketch_lit(doc, board);
        self.set_sketch(doc, history, board, on)
    }

    /// A opção «Rascunho»/«Final» da barra da forma (a «Sloppiness» do Excalidraw, decisão do dono
    /// 22/10): põe `on` em tudo o que o botão alcança — o mesmo alcance, o mesmo passo e o mesmo
    /// modo do quadro que [`Self::toggle_sketch`], que é esta com o contrário da luz.
    pub fn set_sketch(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        board: bool,
        on: bool,
    ) -> Option<bool> {
        // O texto em edição fecha-se antes (é o passo dele): a letra nova não se mede no que está aberto.
        self.commit_text(doc, history);
        let (whole, targets) = self.sketch_targets(doc);
        // A letra fixa que um `.excalidraw` trouxe volta ao modo com o botão (decisão do dono, W10).
        let ops = targets
            .into_iter()
            .filter(|el| el.style().sketch != on || el.style().look.letter != Letter::Auto)
            .map(|el| {
                let mut el = el.clone();
                let st = el.style_mut();
                st.sketch = on;
                st.look.letter = Letter::Auto;
                BoardOp::Put(el)
            })
            .collect();
        if !whole {
            history.apply(doc, ops);
            return None;
        }
        history.apply_mode(doc, ops, [board, on]);
        self.style.sketch = on;
        Some(on)
    }

    /// ⭐ O que o botão «Rascunho ↔ Final» alcança — a porta ÚNICA da luz dele e do clique: os
    /// alcançáveis editáveis da selecção; sem selecção, ou com uma que ele não alcança (só traços
    /// livres, frames, vídeos, travados), o quadro INTEIRO (`true`) — nunca um botão morto (auditoria
    /// 21/10: o clique não fazia nada e a luz contava os travados).
    #[must_use]
    pub fn sketch_targets<'a>(&self, doc: &'a BoardDoc) -> (bool, Vec<&'a Element>) {
        let picked: Vec<&Element> = self
            .editable(doc)
            .into_iter()
            .filter(|el| sketchable(el))
            .collect();
        if picked.is_empty() {
            let all = doc.live_in_z_order();
            (true, all.into_iter().filter(|el| sketchable(el)).collect())
        } else {
            (false, picked)
        }
    }

    /// A luz do botão: o modo do quadro (`board`) quando ele troca o quadro inteiro; senão, tudo o
    /// que alcança já à mão. Carregar dá sempre o contrário.
    #[must_use]
    pub fn sketch_lit(&self, doc: &BoardDoc, board: bool) -> bool {
        self.sketch_look(doc, board) == Some(true)
    }

    /// O aspecto do que o botão alcança: `Some(true)` à mão, `Some(false)` final, `None` misturado
    /// (nenhuma das duas opções da barra acende, como no Excalidraw). A luz da onda é `Some(true)`.
    #[must_use]
    pub fn sketch_look(&self, doc: &BoardDoc, board: bool) -> Option<bool> {
        match self.sketch_targets(doc) {
            (true, _) => Some(board),
            (false, t) if t.iter().all(|el| el.style().sketch) => Some(true),
            (false, t) if t.iter().all(|el| !el.style().sketch) => Some(false),
            (false, _) => None,
        }
    }
}

/// O botão «Rascunho ↔ Final» mexe neste elemento? As formas e as setas — as notas também, só na
/// letra (decisão do dono, 10/10: o papel fica liso, o desenho não as passa ao traço tremido).
#[must_use]
pub fn sketchable(el: &Element) -> bool {
    // W15: a imagem pela moldura (o contorno à mão, como o de uma forma); W23 B13: a tinta LIMPA de
    // uma forma da caneta + segurar (o traço livre já é mão).
    el.connector().is_some()
        || el.shape().is_some()
        || el.image().is_some()
        || el.ink().is_some_and(|i| i.clean)
}
