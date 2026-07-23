//! A seção **TEXT ON PATH** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC).
//!
//! O texto corre ao longo de uma curva com os glyphs **rígidos** (plano 22 §1, caso A): cada
//! letra translada e gira, e não deforma. É outra feature que o Envelope, que amassa a arte com
//! um mapa não-afim — e é por isso que esta é pequena e aquela custou um ADR.
//!
//! **Esta seção é a única porta do produto para o vínculo.** Sem ela o motor existiria, gateado
//! e smokado, e a feature não existiria para o artista — a mesma frase que a seção do envelope
//! teve de escrever sobre si.
//!
//! # As duas caras da seção
//!
//! Um texto **solto** mostra só a porta de entrada, e mesmo essa só quando a seleção a permite
//! (um texto + um caminho). Um texto **preso** mostra o que se pode dizer sobre o vínculo:
//! onde começa, de que lado, e como sair. Nunca as duas ao mesmo tempo — *prender* não quer
//! dizer nada num texto já preso, e *offset* não quer dizer nada num texto solto.
//!
//! # Por que Detach
//!
//! Prender sem soltar é **porta de mão única**. É o mesmo argumento do `Release` do Envelope e
//! do Blend, e a saída aqui é ainda mais barata: o vínculo é um componente, então soltar é
//! removê-lo — o caminho fica, o texto volta a ser texto reto, nada é destruído.

use super::*;

impl BodyCtx<'_> {
    /// Seção **TEXT ON PATH**.
    ///
    /// Só existe com um texto em foco (`state::text_visible`, a mesma condição das seções Font e
    /// Paragraph): num retângulo selecionado ela não teria sujeito.
    pub(crate) fn textpath_section(&mut self, y: f32) -> f32 {
        if !state::text_visible() {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_TEXTPATH,
            tr("panel.vector.section.textpath"),
            y,
        );
        if collapsed {
            return y;
        }
        if !state::linked() {
            // Duas portas para prender. A auto-ligação (`Text on Path`) depende de uma seleção MUITO
            // específica — um texto E um caminho —, então é oferecida só quando essa seleção existe:
            // um clique com a seleção errada não teria como explicar-se. O **Picker** (`Pick Path`)
            // não tem essa dependência — a fonte é o texto em foco, o guia vem do clique seguinte no
            // canvas —, então é oferecido SEMPRE que há um texto solto. As duas coexistem quando
            // ambas valem; o Picker é a porta explícita e mais correta (Enio 2026-07-23).
            let mut y = y;
            if state::can_link() {
                y = self.action_button(ids::VECTOR_TEXTPATH_LINK, "Text on Path", y);
            }
            return self.action_button(ids::VECTOR_TEXTPATH_PICK, "Pick Path", y);
        }
        let track = self
            .store
            .slider(ids::VECTOR_TEXTPATH_OFFSET)
            .map_or_else(|| state::offset() as f32, |(_, v)| v);
        let val = self
            .store
            .number_value(ids::VECTOR_TEXTPATH_OFFSET_NUM)
            .unwrap_or_else(state::offset);
        y = self.slider_row(
            "Offset",
            ids::VECTOR_TEXTPATH_OFFSET,
            ids::VECTOR_TEXTPATH_OFFSET_NUM,
            track,
            val,
            &format!("{val:.2}"),
            y,
        );
        // O lado é um par exclusivo, não um checkbox: "deste lado / do outro" é uma escolha
        // entre dois, e um segmentado a mostra sem o artista ter de descobrir o que o estado
        // desmarcado significa.
        //
        // ⚠️ E o **Detach NÃO entra aqui**. A primeira versão o pôs como terceira opção do
        // segmentado, o que é errado de um jeito que só se vê ao clicar: os outros dois são
        // ESTADOS (o texto fica deste lado, ou do outro) e ele é uma AÇÃO que não tem estado
        // ligado. Um controlo que mistura as duas coisas ensina a coisa errada sobre si.
        let flip = state::flip();
        // Tabela TIPADA (HR-12), como a do Blend e a do Envelope: o `segmented` e o
        // `action_button` deste painel delegam aos primitivos `paint_segmented_button` /
        // `paint_button`, que são quem wira o AccessKit — nomear o `NodeId` aqui é o que torna
        // essa delegação visível a quem lê o arquivo (e ao scan que a cobra).
        let sides: [(ph2d_a11y::NodeId, &str, bool); 2] = [
            (ids::VECTOR_TEXTPATH_FLIP_OFF, "This side", !flip),
            (ids::VECTOR_TEXTPATH_FLIP, "Other side", flip),
        ];
        y = self.segmented("Side", &sides, y);
        self.action_button(ids::VECTOR_TEXTPATH_DETACH, "Detach from Path", y)
    }
}
