//! **A seção SNAP** — o ímã das formas, do caminho, dos cruzamentos e das guias. (A régua
//! mora no menu *View* desde a D10 — um comando, uma porta.)
//!
//! ⚠️ Volta ao painel a 15/10 (dono, F16: *«Snap volta pro painel lateral como era antes»*): a fila ②
//! virou a barra de controle do Illustrator e o pulldown do encaixe saiu dela. O corpo é o de antes
//! da D10 (`paint_arrange.rs` em `6d19371ae4^`), sem mudança de lei.

use crate::paint_sections::BodyCtx;
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;

impl BodyCtx<'_> {
    /// Snap em FORMAS — ajuste da ferramenta (não da seleção), sempre visível.
    /// Segurar Alt ignora o snap momentaneamente, como no Figma.
    ///
    /// A grade **não mora aqui**: o editor tem um subsistema universal de grade
    /// (nove tipos, magnetismo, subdivisões) com painel próprio, e o Vector só
    /// pergunta a ele. Ligar/desligar a grade é lá.
    /// ⚠️ As três linhas são **independentes**, e não modos de um mesmo rádio: "Shapes" ALINHA
    /// um eixo de cada vez (a lei do Figma), enquanto "Path" e "Cross" pousam o ponto num
    /// lugar. Colapsá-las num seletor obrigaria o artista a abrir mão do alinhamento para
    /// ganhar a linha, que são perguntas diferentes.
    pub(crate) fn snap_section(&mut self, y: f32) -> f32 {
        let on = state::current_snap();
        let path = state::current_snap_path();
        let cross = state::current_snap_crossings();
        let guides = state::current_snap_guides();
        let (y, collapsed) = self.section_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_SNAP,
            tr("panel.vector.section.snap"),
            y,
        );
        if collapsed {
            return y;
        }
        let y = self.segmented(
            tr("panel.vector.arrange.shapes"),
            &off_on(
                crate::ids::VECTOR_SNAP_OFF,
                ph2d_editor_core::ids::VECTOR_SNAP_ON,
                on,
            ),
            y,
        );
        let y = self.segmented(
            tr("panel.vector.arrange.path"),
            &off_on(
                crate::ids::VECTOR_SNAP_PATH_OFF,
                crate::ids::VECTOR_SNAP_PATH_ON,
                path,
            ),
            y,
        );
        let y = self.segmented(
            tr("panel.vector.arrange.cross"),
            &off_on(
                crate::ids::VECTOR_SNAP_CROSS_OFF,
                crate::ids::VECTOR_SNAP_CROSS_ON,
                cross,
            ),
            y,
        );
        // As GUIAS (W6.2). Alinhamento como o "Shapes", com um interruptor próprio: desligar
        // o ímã das formas não deve desligar o das linhas que o artista pôs à mão.
        self.segmented(
            tr("panel.vector.arrange.guides"),
            &off_on(
                crate::ids::VECTOR_SNAP_GUIDES_OFF,
                crate::ids::VECTOR_SNAP_GUIDES_ON,
                guides,
            ),
            y,
        )
    }
}

/// Um interruptor Off/On do ímã: as duas peças do segmentado (`segmented` → `paint_choice_row`),
/// cada uma com o SEU `NodeId` — a identidade de AccessKit que o hit, o store e a árvore partilham —,
/// o nome e o estado `selected`.
fn off_on(off: NodeId, on: NodeId, value: bool) -> [(NodeId, &'static str, bool); 2] {
    [
        (off, tr("panel.vector.arrange.off"), !value),
        (on, tr("panel.vector.arrange.on"), value),
    ]
}
