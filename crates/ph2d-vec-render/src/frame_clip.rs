//! **O recorte da MOLDURA** — módulo irmão (teto de LOC), e a única coisa que o [`crate::dispatch`]
//! precisa saber sobre contêineres.
//!
//! A moldura é um retângulo vivo que ganhou `ph2d_ecs::VecFrame` (o componente mora no ECS; aqui só
//! chega o resultado, como `VecClipSpan` dentro do `VecViewState`). Ela faz DUAS coisas com a pilha
//! de z, e as duas acontecem no mesmo lugar:
//!
//! 1. **O preenchimento dela é o FUNDO.** O DFS lista o pai antes dos filhos e a pilha de z é o
//!    inverso disso ⇒ um pai desenha na FRENTE dos filhos. Invisível para um grupo (sem
//!    geometria), fatal para uma moldura: ela cobriria o próprio conteúdo. Então o desenho dela é
//!    antecipado para a ABERTURA do intervalo — que é literalmente o que "fundo do card" quer
//!    dizer, e o que o Figma faz.
//! 2. **A silhueta dela recorta** o que vem depois, até a vez dela chegar.
//!
//! # Push e pop se emparelham SEMPRE
//!
//! ⚠️ A abertura corre **antes** do filtro de escondido, e o fechamento **antes** de qualquer
//! outra decisão sobre a moldura. Se a abertura fosse condicionada a *"o primeiro descendente
//! desenha"*, um filho escondido deixaria a camada por abrir e o `pop_layer` da moldura fecharia
//! uma camada que não é dela — o Vello desmontaria o recorte de outra pessoa, e o sintoma seria
//! arte alheia sumindo. Uma camada vazia custa um push e um pop; um desemparelhamento custa a cena.

use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecViewState, VecXforms};
use ph2d_vector::{Affine, VectorScene};

use crate::{LiveGeometry, build::build_fill_bezpath, fill_rule, path_to_screen};

/// As molduras abertas agora, da mais externa para a mais interna. É a pilha do Vello espelhada —
/// o que permite ao [`crate::dispatch`] responder *"a vez deste path é a de fechar?"* sem procurar
/// nada.
#[derive(Default)]
pub(crate) struct OpenFrames {
    stack: Vec<VecPathId>,
}

impl OpenFrames {
    /// Abre toda moldura cujo intervalo começa em `id`: desenha o fundo dela (se ela não estiver
    /// escondida) e empurra a camada de clip.
    ///
    /// `resolve` entrega a geometria da moldura **como o dispatch a desenharia** — a derivada viva
    /// se houver, senão a fonte com a pose. Uma segunda resolução aqui recortaria num lugar e
    /// pintaria noutro.
    // Sete argumentos porque a abertura precisa das MESMAS entradas que o desenho de um
    // caminho — resolver a geometria da moldura por outra via seria recortar num lugar e pintar
    // noutro (é o ponto do `resolve`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn open_at(
        &mut self,
        id: VecPathId,
        scene: &VecScene,
        view: &VecViewState,
        xforms: &VecXforms,
        live: &LiveGeometry,
        camera: Affine,
        target: &mut VectorScene,
    ) {
        for span in view.clips_opening_at(id) {
            let Some((path, xf)) = resolve(scene, xforms, live, span.frame, camera) else {
                continue;
            };
            if !view.is_hidden(span.frame) {
                crate::draw_path(path, xf, target);
            }
            let mut bp = build_fill_bezpath(path);
            bp.apply_affine(xf);
            // A regra é a da PRÓPRIA moldura: um retângulo com furo (um card vazado) recorta pelo
            // furo, e sob `NonZero` o furo ainda tomaria tinta.
            target.push_clip_with_rule(&bp, fill_rule(path));
            self.stack.push(span.frame);
        }
    }

    /// Se `id` é a moldura aberta mais interna, fecha a camada dela e devolve `true` — e o
    /// chamador **não a desenha**, porque ela já foi desenhada na abertura.
    pub(crate) fn close_if_frame(&mut self, id: VecPathId, target: &mut VectorScene) -> bool {
        if self.stack.last() == Some(&id) {
            self.stack.pop();
            target.pop_layer();
            return true;
        }
        false
    }

    /// Fecha o que sobrou. ⚠️ Não pode acontecer com uma lista bem-formada (a moldura é o último
    /// membro da própria sub-árvore, logo toda camada aberta fecha dentro do laço) — existe porque
    /// a lista vem de fora, e uma cena com uma camada pendurada envenena **o resto do frame**, que
    /// é chrome que esta crate nem desenha.
    pub(crate) fn close_all(&mut self, target: &mut VectorScene) {
        for _ in self.stack.drain(..) {
            target.pop_layer();
        }
    }
}

#[cfg(test)]
#[path = "frame_clip_tests.rs"]
mod tests;

/// A geometria de `id` e o afim que a leva à tela, pela MESMA regra do [`crate::dispatch`]: a
/// derivada viva já está em MUNDO (sobe pela câmera), a fonte sobe pela pose.
fn resolve<'a>(
    scene: &'a VecScene,
    xforms: &VecXforms,
    live: &'a LiveGeometry,
    id: VecPathId,
    camera: Affine,
) -> Option<(&'a VecPath, Affine)> {
    if let Some(items) = live.get(&id) {
        // Uma moldura com produtor vivo desenha o primeiro item; recortar por ele é recortar pelo
        // que se vê. (Uma lista vazia é o "não desenhe nada" da booleana viva — sem silhueta, sem
        // recorte.)
        return items.first().map(|p| (p, camera));
    }
    let path = scene.paths().iter().find(|p| p.id == id)?;
    Some((path, path_to_screen(xforms, id, camera)))
}
