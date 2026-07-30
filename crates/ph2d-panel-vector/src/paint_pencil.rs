//! **A seção PENCIL** do painel — irmã de [`super::paint_modes`] pelo teto de 600 LOC dos
//! painéis, e o corte é por responsabilidade: aqui mora a seção de UMA ferramenta (a mão livre),
//! lá a fileira de MODOS e as seções que valem para todas.

use ph2d_i18n::tr;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::{
    DrawMode, PENCIL_FIDELITY_DEFAULT_PX, PENCIL_STABILIZER_DEFAULT,
    PENCIL_STABILIZER_SLIDER_SCALE, fidelity_px_to_slider,
};

use crate::ids;
use crate::paint_sections::BodyCtx;

impl BodyCtx<'_> {
    /// Seção dos **PARÂMETROS** da forma em foco — o cabeçalho É o nome dela
    /// (`paint_section_header` já pinta em MAIÚSCULAS), o que responde "de quem são estes
    /// campos?" sem gastar uma linha a mais.
    ///
    /// **De quem são os campos** é decidido em [`crate::shape_focus`] (as três respostas:
    /// forma viva · catálogo · ninguém). `None` ⇒ a seção INTEIRA some — foi selecionado
    /// algo que não é forma viva (um conector, uma curva comum), e um campo editável que
    /// não edita nada é pior que campo nenhum.
    ///
    /// Formas sem campo (Rect / Decision / Terminal / Delay / Junction…) mostram uma
    /// linha "No parameters": a seção nunca parece quebrada.
    /// **A seção PENCIL** — os dois controles da mão livre, e só no modo Lápis.
    ///
    /// Condicional pela convenção deste painel (uma seção por *coisa ativa*, como a de parâmetros
    /// da forma logo abaixo): o artista pega o Lápis e os ajustes dele aparecem imediatamente sob a
    /// fileira TOOL. Os dois knobs editam como a MÃO é capturada — noutro modo não haveria mão a
    /// capturar, e seriam dois sliders que não fazem nada.
    ///
    /// ⚠️ **Dois controles porque são duas PERGUNTAS**, e uma delas não pode responder pela outra:
    /// *Fidelity* é a tolerância do decimador (na SAÍDA — quanto detalhe fica na curva) e
    /// *Stabilizer* filtra o tremor (na ENTRADA — que mão o lápis escuta). O decimador preserva
    /// extremos locais de propósito, e um tremor é um extremo local: nenhuma Fidelity o remove.
    pub(crate) fn pencil_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        if snap.mode != DrawMode::Pencil {
            return y;
        }
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_PENCIL,
            tr("panel.vector.section.pencil"),
            y,
        );
        if collapsed {
            return y;
        }
        // O store é a fonte da verdade do knob (o arrasto manda); o default do tool só o semeia.
        let fid_track = self
            .store
            .slider(ids::VECTOR_PENCIL_FIDELITY)
            .map(|(_, v)| v)
            .unwrap_or_else(|| fidelity_px_to_slider(PENCIL_FIDELITY_DEFAULT_PX));
        let fid_px = self
            .store
            .number_value(ids::VECTOR_PENCIL_FIDELITY_NUM)
            .unwrap_or(PENCIL_FIDELITY_DEFAULT_PX);
        y = self.slider_row(
            "Fidelity",
            ids::VECTOR_PENCIL_FIDELITY,
            ids::VECTOR_PENCIL_FIDELITY_NUM,
            fid_track,
            fid_px,
            &format!("{fid_px:.1} px"),
            y,
        );
        let stab_track = self
            .store
            .slider(ids::VECTOR_PENCIL_STABILIZER)
            .map(|(_, v)| v)
            .unwrap_or(PENCIL_STABILIZER_DEFAULT);
        // O chip já fala por cento (o mapeamento vive no `link_slider_number_mapped`).
        let stab_pct = self
            .store
            .number_value(ids::VECTOR_PENCIL_STABILIZER_NUM)
            .unwrap_or(f64::from(
                PENCIL_STABILIZER_DEFAULT * PENCIL_STABILIZER_SLIDER_SCALE,
            ));
        y = self.slider_row(
            "Stabilizer",
            ids::VECTOR_PENCIL_STABILIZER,
            ids::VECTOR_PENCIL_STABILIZER_NUM,
            stab_track,
            stab_pct,
            &format!("{stab_pct:.0}%"),
            y,
        );
        // **A FONTE da largura** (W1d) — a 3ª pergunta do gesto, e a que faz do lápis um pincel:
        // *que mão eu escuto* (Stabilizer) · *que detalhe eu guardo* (Fidelity) · **de onde vem a
        // ESPESSURA**. `Pressure` é oferecida e hoje não chega (a shell não recebe pressão de
        // dispositivo nenhum); o rótulo o diz, em vez de deixar o artista descobrir.
        let src = snap.pencil_width_source;
        use ph2d_vec_edit::pencil_width::WidthSource as Ws;
        // Os ids são tipados como `ph2d_a11y::NodeId` porque é o que eles de fato são: cada um
        // vira um nó de AccessKit lá dentro (`slider_row` → `paint_slider_with_chip…`,
        // `segmented3` → `paint_segmented_button`, que é quem emite). A seção **delega** o a11y
        // aos primitivos canónicos — o gate HR-12 lê esta delegação aqui, como no irmão
        // `paint_expand`.
        let width_opts: [(ph2d_a11y::NodeId, &str, bool); 3] = [
            (
                ids::VECTOR_PENCIL_W_UNIFORM,
                tr("panel.vector.pencil.width.uniform"),
                src == Ws::Uniform,
            ),
            (
                ids::VECTOR_PENCIL_W_SPEED,
                tr("panel.vector.pencil.width.speed"),
                src == Ws::Speed,
            ),
            (
                ids::VECTOR_PENCIL_W_PRESSURE,
                tr("panel.vector.pencil.width.pressure"),
                src == Ws::Pressure,
            ),
        ];
        self.segmented3("Width", width_opts, y)
    }
}
