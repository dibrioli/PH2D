//! A seção **ENVELOPE** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC).
//!
//! A deformação das formas selecionadas por uma **gaiola de 4 cantos** (ADR-0129). O envelope é um
//! **CONTAINER** (Fatia 3): as formas envolvidas viram FILHAS de uma entidade que carrega a gaiola —
//! uma gaiola só deforma 1..N formas (o *warp group* do Affinity).
//!
//! **Esta seção é a única porta do produto para o envelope.** Antes dela, `envelope_live::create` só
//! era chamado pela env `PH2D_BUILD_SMOKE` — a feature existia no motor, gateada e smokada, e **não
//! existia para o artista**.
//!
//! # Por que Expand E Release
//!
//! Criar sem desfazer é **porta de mão única**: envolveu, nunca mais tira. Os dois são o MESMO
//! `dissolve` com uma pergunta diferente — *qual geometria fica?* **Expand** materializa a deformada
//! (a deformação vira o desenho); **Release** ressuscita a fonte autorada (a deformação é desfeita).
//! Espelham o par Expand/Release do Blend, e é de propósito: são a mesma pergunta em dois objetos.
//!
//! Os dois só são OFERECIDOS quando a seleção é de fato um envelope (`state::has_envelope`, que o
//! shell publica a cada frame a partir de `envelope_live::sole_container` — a mesma porta que decide
//! a seleção e executa o dissolve). Um botão que não faz nada é pior que um botão que falta.

use super::*;

impl BodyCtx<'_> {
    /// Seção **ENVELOPE** — envolver a seleção numa gaiola, e as duas saídas dela.
    ///
    /// O botão **Envelope** é pintado sempre (como o do Blend): ele age sobre a SELEÇÃO, e recusar
    /// no shell com uma mensagem é mais honesto que esconder a porta de entrada da feature. Já
    /// **Expand**/**Release** operam sobre um envelope EXISTENTE — sem um, não são oferecidos.
    pub(crate) fn envelope_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_ENVELOPE,
            tr("panel.vector.section.envelope"),
            y,
        );
        if collapsed {
            return y;
        }
        y = self.action_button(ids::VECTOR_ENVELOPE_RUN, "Envelope", y);
        if state::has_envelope() {
            // QUAL mapa a gaiola aplica (ADR-0129 §4). Não é um knob de intensidade: os dois
            // divergem no miolo (projetivo mantém as retas retas; o Coons de lados retos é
            // bilinear), e o Mesh é o único que oferece as 8 alças de lado. Vem ANTES do
            // Expand/Release porque escolhe como o envelope VIVO se comporta; os outros dois o
            // terminam.
            let mesh = state::envelope_mesh();
            y = self.segmented2(
                "Cage",
                [
                    (ids::VECTOR_ENVELOPE_PERSPECTIVE, "Perspective", !mesh),
                    (ids::VECTOR_ENVELOPE_MESH, "Mesh", mesh),
                ],
                y,
            );
            // Tabela tipada como a do Blend (HR-12): o `action_button` delega ao `paint_button`
            // canônico, que é quem costura o AccessKit — e nomear o `NodeId` aqui é o idioma que o
            // gate `every_widget_file_wires_a11y` reconhece nos irmãos desta pasta.
            let commands: [(ph2d_a11y::NodeId, &str); 2] = [
                (ids::VECTOR_ENVELOPE_EXPAND, "Expand"),
                (ids::VECTOR_ENVELOPE_RELEASE, "Release"),
            ];
            for (id, label) in commands {
                y = self.action_button(id, label, y);
            }
        }
        y
    }
}
