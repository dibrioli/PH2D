//! **ESTE PASSO MUDOU A FIGURA SÓ DENTRO DE UM RETÂNGULO?** — filho de [`super`] (`#[path]`, então ele
//! enxerga o [`UndoEntry`] privado), split pelo cap de LOC e pela linha de corte natural: *o que a
//! história GUARDA* fica lá, *quanto dela a tela precisa repintar* fica aqui.
//!
//! # O número que justifica o módulo
//!
//! Um quadro depois de um Ctrl+Z custa **97,7 ms a 2048² e 381,3 a 4096²** contra **0,000 ms** de um
//! tick ocioso (doc 28 §5.62) — plane-bound, porque `restore_model` derruba o cache de composite e o
//! dirty-rect, mandando a tela inteira ser refeita. Um quadro NO MEIO de um traço, que passa pela pista
//! parcial, custa **0,6 ms e é plano na tela**. A maquinaria existe; o undo é que não a usava.
//!
//! # As DUAS metades da prova, e por que nenhuma basta
//!
//! * **os PLANOS** — [`crate::undo_planes::PlaneDeltas::confined_region`] diz em que retângulo os
//!   dezenove planos canvas-shaped diferem.
//! * **os METADADOS** — [`ModelSnapshot::confined_to`], aqui. Um passo pode não tocar pixel nenhum e
//!   ainda assim mudar a figura **em toda parte**: trocar a opacidade de uma camada, reordenar a pilha,
//!   esconder um grupo. É exatamente o que o comentário do `invalidate_composite` sempre disse, e é
//!   uma cerca de Chesterton legítima — a wave não a derruba, ela a **estreita**.
//!
//! # A regra que impede isto de apodrecer
//!
//! ⚠️ **Os campos são desestruturados um a um, sem `..`.** Um campo novo em [`ModelSnapshot`] **não
//! compila** até alguém o classificar, e essa é a única defesa possível: o modo de falha de esquecer um
//! é uma reivindicação de confinamento **boa demais**, cuja consequência é a tela mostrar pixels velhos
//! fora do retângulo — e nenhum gate de conteúdo pega isso, porque o conteúdo DENTRO do retângulo está
//! perfeito.

use super::{ModelSnapshot, UndoController, UndoEntry};
use crate::compositor::Region;

impl ModelSnapshot {
    /// **Estes dois endpoints diferem SÓ em pixels de planos canvas-shaped?**
    ///
    /// `true` habilita o chamador a confiar na região que os planos declaram. Toda dúvida responde
    /// `false`, e o preço de um `false` errado é um repaint — contra corrupção visual silenciosa do
    /// lado oposto.
    pub(crate) fn confined_to(&self, other: &Self) -> bool {
        // ⚠️ Sem `..`: acrescentar um campo tem de QUEBRAR A COMPILAÇÃO aqui.
        let Self {
            // — comparados, porque descrevem COMO a figura é montada —
            layers,
            canvas_size,
            selection,
            offset_norm,
            offset_base_px,
            active_op,
            mask_scratch_target,
            selection_active,
            selection_feather,
            selection_shapes,
            // — exigidos AUSENTES (ver abaixo) —
            shape,
            preview_patch,
            parked_shapes,
            deform,
            sculpt,
            // — os PIXELS: quem responde por eles é o `PlaneDeltas`, e compará-los aqui seria a
            //   segunda resposta à mesma pergunta (além de custar uma varredura por campo) —
            images: _,
            heights: _,
            covers: _,
            mats: _,
            canvas_rgba: _,
            mask_scratch: _,
            selection_mask: _,
            selection_crisp: _,
            // — NÃO são sobre a figura —
            //   `relief_elided` é uma TESTEMUNHA (quais planos este snapshot descreve sem segurar);
            //   `writes` é PROVENIÊNCIA (a janela declarada serve a este endpoint?). Nenhum dos dois
            //   muda um pixel, e exigir igualdade deles recusaria todo passo elidido — que é o comum.
            relief_elided: _,
            writes: _,
        } = self;

        // ⚠️ **Uma sessão VIVA desqualifica o passo, em vez de ser comparada.** O `restore_model`
        // termina em `restore_shape_overlay`, que **RE-CARIMBA** a figura aberta — pixels escritos onde
        // o editor a desenha, que não têm relação nenhuma com a janela do delta. O mesmo vale para as
        // sessões de Deform e Sculpt, que re-renderizam a partir dos planos congelados. Enquanto
        // qualquer uma estiver de pé, *"o passo está confinado"* é uma pergunta que este módulo não
        // sabe responder — e não saber responde `false`.
        let quiet = |m: &Self| {
            m.shape.is_none()
                && m.preview_patch.is_none()
                && m.parked_shapes.is_empty()
                && !m.deform.active
                && m.deform.relief_layer.is_none()
                && m.sculpt.layer.is_none()
        };
        let _ = (shape, preview_patch, parked_shapes, deform, sculpt);
        if !quiet(self) || !quiet(other) {
            return false;
        }

        *layers == other.layers
            && *canvas_size == other.canvas_size
            && *selection == other.selection
            && offset_norm.to_bits() == other.offset_norm.to_bits()
            && offset_base_px.to_bits() == other.offset_base_px.to_bits()
            && *active_op == other.active_op
            && *mask_scratch_target == other.mask_scratch_target
            && *selection_active == other.selection_active
            && selection_feather.to_bits() == other.selection_feather.to_bits()
            && *selection_shapes == other.selection_shapes
    }
}

impl UndoEntry {
    /// A região a que este passo está confinado, ou `None` para *não confinado*.
    fn confined_region(&self) -> Option<Region> {
        if !self.before.confined_to(&self.after) {
            return None;
        }
        self.planes.confined_region(self.before.canvas_size.0)
    }
}

#[cfg(test)]
impl UndoEntry {
    /// **Por que este passo NÃO é confinado** — o instrumento, não uma segunda implementação.
    ///
    /// Ele chama as MESMAS duas metades e só reporta qual delas recusou. Sem isto, `None` é um veredito
    /// mudo, e um veredito mudo manda a próxima pessoa adivinhar (doc 28 §5.49: *um instrumento
    /// silencioso é pior que um ausente — ele TRANQUILIZA*).
    pub(crate) fn confine_diagnosis(&self) -> String {
        let meta = self.before.confined_to(&self.after);
        let planes = self.planes.confined_region(self.before.canvas_size.0);
        format!(
            "metadados={meta} planos={planes:?} | {}",
            self.planes.confine_report()
        )
    }
}

impl UndoController {
    /// O diagnóstico do próximo passo — ver [`UndoEntry::confine_diagnosis`].
    #[cfg(test)]
    pub(crate) fn peek_confine_diagnosis(&self, redo: bool) -> Option<String> {
        let stack = if redo { &self.redo } else { &self.undo };
        Some(stack.last()?.confine_diagnosis())
    }

    /// **A que região o PRÓXIMO undo (ou redo) está confinado** — perguntado ANTES de aplicá-lo.
    ///
    /// ⚠️ **Uma pergunta, não um stash.** A alternativa — guardar a região num campo ao aplicar e o
    /// chamador a consumir depois — cria um segundo lugar onde *"o que o último passo mudou"* mora, e
    /// um segundo lugar é onde uma resposta obsoleta sobrevive a um caminho que retornou cedo.
    pub(crate) fn peek_confined_region(&self, redo: bool) -> Option<Region> {
        #[cfg(test)]
        if !self.confine {
            return None; // a ablação do A/B — ver `UndoController::confine`
        }
        let stack = if redo { &self.redo } else { &self.undo };
        stack.last()?.confined_region()
    }
}
