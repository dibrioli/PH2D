//! A seção **BLEND** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC do painel).
//!
//! **Steps** + o botão **Blend**, sobre o **Blend Object VIVO** (ADR-0122 — o Blend do Illustrator:
//! objeto único, não-destrutivo, fontes sempre editáveis). O botão cria um blend vivo sobre as
//! formas fechadas selecionadas (2..=5, em z); o slider Steps ajusta o blend selecionado **ao
//! vivo**. Não há mais **Stack Each Above** — no modelo vivo o z é automático (fonte0 embaixo →
//! passos → fonteN em cima). A correspondência é 100% automática (o problema que ninguém do mercado
//! resolveu — GSAP/Corel exigem controle manual; o nosso não pede nada ao artista).

use super::*;

impl BodyCtx<'_> {
    /// Seção **BLEND** — os passos intermediários entre as formas selecionadas (Blend Object vivo).
    ///
    /// **A correspondência é automática** — que ponto de A vira que ponto de B é o problema que
    /// ninguém do mercado resolveu (o GSAP tem um `shapeIndex` manual E uma ferramenta de debug que
    /// admite que o automático erra; o Corel pede para o usuário clicar um nó em cada forma), e o
    /// nosso motor o resolve sem pedir nada ao artista. O ajuste, no modelo vivo, é **editar as
    /// formas-fonte** (mover/girar/escalar uma adapta os intermediários), não um botão de escape.
    pub(crate) fn blend_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_BLEND,
            tr("panel.vector.section.blend"),
            y,
        );
        if collapsed {
            return y;
        }
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
        self.action_button(ids::VECTOR_BLEND_RUN, "Blend", y)
    }
}
