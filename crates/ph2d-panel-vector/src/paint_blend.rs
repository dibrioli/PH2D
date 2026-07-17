//! As seções **BLEND** e **MORPH** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC).
//!
//! As duas moram no mesmo arquivo porque partilham o motor (a correspondência de
//! `ph2d-vec-blend`), mas são **seções separadas** na tela, e isso é deliberado: o que elas
//! produzem são objetos diferentes — o Blend dá N passos virtuais em volta de um spine; o Morph dá
//! UMA forma real, cujo `t` a timeline keya. O Morph não é um modo do Blend.
//!
//! **Steps** + o botão **Blend**, sobre o **Blend Object VIVO** (ADR-0128 — o Blend do Illustrator:
//! objeto único, não-destrutivo, fontes sempre editáveis). O botão cria um blend vivo sobre as
//! formas fechadas selecionadas (2..=5, em z); o slider Steps ajusta o blend selecionado **ao
//! vivo**. Não há mais **Stack Each Above** — no modelo vivo o z é automático (fonte0 embaixo →
//! passos → fonteN em cima). A correspondência é 100% automática (o problema que ninguém do mercado
//! resolveu — GSAP/Corel exigem controle manual; o nosso não pede nada ao artista).

use super::*;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::DrawMode;

impl BodyCtx<'_> {
    /// Seção **BLEND** — os passos intermediários entre as formas selecionadas (Blend Object vivo).
    ///
    /// **A correspondência é automática** — que ponto de A vira que ponto de B é o problema que
    /// ninguém do mercado resolveu (o GSAP tem um `shapeIndex` manual E uma ferramenta de debug que
    /// admite que o automático erra; o Corel pede para o usuário clicar um nó em cada forma), e o
    /// nosso motor o resolve sem pedir nada ao artista. O ajuste, no modelo vivo, é **editar as
    /// formas-fonte** (mover/girar/escalar uma adapta os intermediários), não um botão de escape.
    ///
    /// O **Pick Shapes** vive AQUI (não na fileira de modos): é uma etapa DO blend — escolher as
    /// formas na ORDEM de clique —, e sua casa natural é ao lado do botão que as liga. O botão acende
    /// quando o modo Pick está ativo (`snap.mode`).
    pub(crate) fn blend_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_BLEND,
            tr("panel.vector.section.blend"),
            y,
        );
        if collapsed {
            return y;
        }
        // **Pick Shapes** — entra no modo de escolher as formas na ORDEM de clique (acende quando
        // ativo). É um botão segmentado (não `action_button`) para poder realçar o estado do modo.
        let picking = snap.mode == DrawMode::PickBlend;
        y = self.button_grid(y, 1, 1, |_| {
            (ids::VECTOR_MODE_PICKBLEND, "Pick Shapes", picking)
        });
        let track = self
            .store
            .slider(ids::VECTOR_BLEND_STEPS)
            .map_or_else(|| blend_steps_to_track(BLEND_STEPS_DEFAULT), |(_, v)| v);
        let steps = blend_steps_from_track(f64::from(track));
        y = self.slider_row(
            "Steps",
            ids::VECTOR_BLEND_STEPS,
            ids::VECTOR_BLEND_STEPS_NUM,
            track,
            f64::from(steps),
            &format!("{steps}"),
            y,
        );
        // Os botões de **COMANDO** da seção, na ordem em que aparecem. A tabela é tipada em
        // `ph2d_a11y::NodeId` porque é o que eles de fato são: cada id vira um nó de AccessKit lá
        // dentro (`action_button` → `paint_button`, que é quem emite). A seção **delega** o a11y às
        // primitivas, e o HR-12 pede que a delegação esteja DITA — o gate `every_widget_file_wires_
        // _a11y` lia este arquivo e não achava a palavra (vermelho latente desde a Fase C1).
        //
        // - **Reset Spine** desfaz a edição do spine (modo Node), voltando à reta pelos centros das
        //   fontes. Sem ele, a única saída da edição seria o undo global (ADR-0128 C2b).
        // - **Expand** e **Release** são as duas saídas do objeto vivo (ADR-0128 Fase D), e ficam
        //   lado a lado porque a escolha é entre elas: *fico com os passos* (o Expand materializa e
        //   o objeto morre) ou *desisto do blend* (os passos somem, as fontes ficam). Nenhuma é um
        //   MODO — cada uma acontece e acabou —, então são `action_button`, não um par segmentado.
        let commands: [(ph2d_a11y::NodeId, &str); 4] = [
            (ids::VECTOR_BLEND_RUN, "Blend"),
            (ids::VECTOR_BLEND_RESET_SPINE, "Reset Spine"),
            (ids::VECTOR_BLEND_EXPAND, "Expand"),
            (ids::VECTOR_BLEND_RELEASE, "Release"),
        ];
        for (id, label) in commands {
            y = self.action_button(id, label, y);
        }
        y
    }

    /// Seção **MORPH** — a forma única entre duas, com o `t` animável.
    ///
    /// **Seção própria, e não uma linha dentro do Blend** (Enio, 2026-07-16): eles partilham o
    /// motor de correspondência, mas o que produzem são objetos DIFERENTES — o Blend dá N passos
    /// virtuais em volta de um spine, o Morph dá UMA forma real cujo `t` a timeline keya. Encaixar
    /// um dentro do outro sugeria que o Morph é um modo do Blend; ele não é.
    ///
    /// O slider é a autoria ao vivo: o artista estaciona a forma onde ela fica bem e aperta **K**
    /// (o `sample_prop_value` captura o `t` como captura uma pose). Sem ele, keyar exigiria digitar
    /// números às cegas numa track vazia.
    pub(crate) fn morph_section(&mut self, y: f32) -> f32 {
        let (y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_MORPH,
            tr("panel.vector.section.morph"),
            y,
        );
        if collapsed {
            return y;
        }
        let mut y = self.action_button(ids::VECTOR_MORPH_RUN, "Morph", y);
        let t = self
            .store
            .slider(ids::VECTOR_MORPH_T)
            .map_or(MORPH_T_DEFAULT, |(_, v)| v);
        y = self.slider_row(
            "Morph t",
            ids::VECTOR_MORPH_T,
            ids::VECTOR_MORPH_T_NUM,
            t,
            f64::from(t),
            &format!("{t:.2}"),
            y,
        );
        y
    }
}
