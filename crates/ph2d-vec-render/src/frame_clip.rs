//! **O recorte da MOLDURA** — módulo irmão (teto de LOC), e a única coisa que o [`crate::dispatch`]
//! precisa saber sobre contêineres.
//!
//! Quem recorta é qualquer forma FECHADA que carregue `ph2d_ecs::VecClipContent` — o componente
//! mora no ECS e aqui só chega o resultado, como `VecClipSpan` dentro do `VecViewState`.
//!
//! ⚠️ **Este módulo nunca soube que era uma moldura**, e é por isso que generalizá-lo em
//! 2026-08-21 custou zero linhas daqui: ele sempre recortou pela silhueta de um caminho ARBITRÁRIO
//! (`build_fill_bezpath` + a regra de preenchimento do próprio caminho). O que mudou foi a
//! elegibilidade, que é da shell. O nome do módulo e o campo `frame` do span guardam a época em
//! que só uma moldura podia recortar.
//!
//! # O que este módulo DEIXOU de fazer, e por quê
//!
//! Ele tinha duas metades: *desenhar o preenchimento da moldura como FUNDO* e *recortar o que vem
//! depois*. A primeira existia porque a pilha de z era o DFS **invertido**, o que punha todo pai na
//! FRENTE dos filhos — então o desenho da moldura era **antecipado** para a abertura do intervalo.
//!
//! ⚠️ Desde 2026-08-04 a pilha é o DFS **na ordem** (a lei de Godot: o filho desenha sobre o pai),
//! logo a moldura já é o primeiro membro da própria sub-árvore e o preenchimento dela **é** o fundo
//! do card sem ninguém fazer nada. A antecipação sumiu; o `clip: bool` do intervalo sumiu com ela
//! (quem não recorta não precisa de intervalo nenhum). Sobra o recorte, que é o que o nome diz.
//!
//! # Push e pop se emparelham SEMPRE
//!
//! ⚠️ A abertura corre **fora** do filtro de escondido, e o fechamento também. Se a abertura fosse
//! condicionada a *"a moldura desenha"*, uma moldura escondida deixaria a camada por abrir e o
//! `pop_layer` do fim do intervalo fecharia uma camada que não é dela — o Vello desmontaria o
//! recorte de outra pessoa, e o sintoma seria arte alheia sumindo. Uma camada vazia custa um push e
//! um pop; um desemparelhamento custa a cena.

use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecViewState, VecXforms};
use ph2d_vector::{Affine, VectorScene};

use crate::{LiveGeometry, build::build_fill_bezpath, fill_rule, path_to_screen};

/// As molduras abertas agora, da mais externa para a mais interna. É a pilha do Vello espelhada —
/// o que permite ao [`crate::dispatch`] responder *"a vez deste path é a de fechar?"* sem procurar
/// nada.
#[derive(Default)]
pub(crate) struct OpenClips {
    stack: Vec<VecPathId>,
}

impl OpenClips {
    /// Abre toda moldura cujo intervalo começa em `id` — ou seja, se `id` **é** uma moldura que
    /// recorta, empurra a camada dela.
    ///
    /// ⚠️ **Chamado DEPOIS do desenho de `id`**: o preenchimento da moldura é o fundo do card e
    /// não pode ser recortado pela própria silhueta (uma borda com anti-aliasing recortada por si
    /// mesma perde metade da tinta da borda).
    ///
    /// `resolve` entrega a geometria da moldura **como o dispatch a desenharia** — a derivada viva
    /// se houver, senão a fonte com a pose. Uma segunda resolução aqui recortaria num lugar e
    /// pintaria noutro.
    // Oito argumentos porque a abertura precisa das MESMAS entradas que o desenho de um caminho
    // pede — resolver a geometria da moldura por outra via seria recortar num lugar e pintar
    // noutro (é o ponto do `resolve`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn open_after(
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
            let mut bp = build_fill_bezpath(path);
            bp.apply_affine(xf);
            // A regra é a da PRÓPRIA moldura: um retângulo com furo (um card vazado) recorta
            // pelo furo, e sob `NonZero` o furo ainda tomaria tinta.
            target.push_clip_with_rule(&bp, fill_rule(path));
            self.stack.push(span.frame);
        }
    }

    /// Fecha toda moldura cujo intervalo acaba em `id`, de dentro para fora.
    ///
    /// ⚠️ **Chamado DEPOIS do desenho de `id`**: o `last` é o último path que o recorte alcança,
    /// e ele próprio é recortado.
    pub(crate) fn close_after(
        &mut self,
        id: VecPathId,
        view: &VecViewState,
        target: &mut VectorScene,
    ) {
        for span in view.clips_closing_at(id) {
            // Só fecha se for de facto o topo da pilha — um intervalo mal-formado (que não
            // aninha) não pode arrastar consigo a camada de outra pessoa.
            if self.stack.last() != Some(&span.frame) {
                continue;
            }
            self.stack.pop();
            target.pop_layer();
        }
    }

    /// Fecha o que sobrou. ⚠️ Não pode acontecer com uma lista bem-formada — existe porque a lista
    /// vem de fora, e uma cena com uma camada pendurada envenena **o resto do frame**, que é
    /// chrome que esta crate nem desenha.
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
