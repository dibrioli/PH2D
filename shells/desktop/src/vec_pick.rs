//! O **Picker de caminho-guia** — o gesto de duas mãos partilhado pelo Pattern on Path e pelo Text
//! on Path (Enio 2026-07-23: *"um botão Picker onde o usuário primeiro seleciona a shape, depois
//! aperta o botão e seleciona o path. Dessa forma é mais correto"*).
//!
//! A disambiguação por seleção (`link_candidate`) tem de ADIVINHAR qual dos selecionados é a fonte
//! e qual é o guia — e adivinha errado quando as premissas não valem (o *"escolhendo a si mesmo"* que
//! o bbox só resolve no caso comum). O Picker remove a adivinhação: a **fonte** é a que estava
//! selecionada quando o botão foi apertado, o **guia** é o que o clique seguinte acerta. Um é
//! capturado, o outro é apontado — nunca há dois candidatos para a mesma vaga.
//!
//! Este módulo é só o TIPO do pedido + o realce do hover; a captura (no `render_loop`), o clique (no
//! `input_dispatch`) e o vínculo em si (`pattern_live::link` / `vec_text_ride::link_explicit`) moram
//! onde já moram os seus irmãos, porque o Picker só troca a PROCEDÊNCIA do par — a inserção do
//! componente é a mesma que a auto-ligação já fazia.

use ph2d_vec_scene::{StrokeSpec, VecPath, VecPathId, VecScene, VecXforms};

/// Um pick ARMADO: quem espera um caminho-guia, com a fonte já capturada.
///
/// A fonte é lida no instante do arm porque a seleção pode mudar sob o clique seguinte (o clique
/// ESCOLHE o guia, e não deve virar a fonte). Guardar o id da fonte torna o vínculo sempre um par
/// explícito `(fonte, guia)` — o oposto exato da adivinhação de seleção.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PathPick {
    /// Um MOTIVO do Pattern on Path à espera do caminho que ele vai cavalgar.
    PatternMotif(VecPathId),
    /// Um TEXTO do Text on Path à espera do caminho que ele vai seguir.
    TextObject(VecPathId),
}

impl PathPick {
    /// A forma-FONTE (o motivo ou o texto). O clique nunca a toma como o próprio guia — prender uma
    /// forma a si mesma não quer dizer nada, e o `link` a recusaria de qualquer modo.
    pub(crate) fn source(self) -> VecPathId {
        match self {
            PathPick::PatternMotif(id) | PathPick::TextObject(id) => id,
        }
    }
}

/// O contorno de MUNDO de um caminho candidato a guia — o REALCE do que está sob o cursor no modo
/// pick (*"está para clicar aqui"*, o idioma do conta-gotas). Sem fill, só a silhueta num traço de
/// acento. `None` se a forma sumiu.
///
/// Assa a fonte LOCAL na pose dela (ADR-0111), como o `pick_preview` do Blend — a mesma escolha de
/// desenhar em MUNDO por ora; um guia por token em screen-space é o refinamento (quando o realce
/// virar UI de verdade).
#[must_use]
pub(crate) fn hover_outline(
    scene: &VecScene,
    xforms: &VecXforms,
    id: VecPathId,
) -> Option<VecPath> {
    let mut p = scene.paths().iter().find(|p| p.id == id)?.clone();
    ph2d_vec_scene::bake_xform(&mut p, &ph2d_vec_scene::xform_of(xforms, id));
    p.fill = None;
    p.stroke = Some(pick_stroke());
    Some(p)
}

/// O traço de realce do guia sob o cursor — um azul de acento em MUNDO, o MESMO do `pick_stroke` do
/// Blend (mesmo idioma, mesma dívida de token por overlay quando o realce virar UI de verdade).
fn pick_stroke() -> StrokeSpec {
    StrokeSpec::new(ph2d_vec_scene::Rgba8::new(80, 150, 240, 230), 0.04)
}
