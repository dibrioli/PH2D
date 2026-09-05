//! ⭐⭐⭐ **A APARÊNCIA CONDUZIDA CHEGA À PROJECÇÃO DO QUADRO** — a segunda metade da ponte que o
//! [`ph2d_ecs::VecDrivenStyle`] abre.
//!
//! A linha do tempo escreve o componente (ela só sabe falar com o mundo ECS); este módulo lê-o e
//! funde-o na **mesma** entrada de estilo que os tokens de design e as rows autoradas já produzem.
//! É o irmão exacto do [`crate::vec_widget_drive`], e de propósito: o corte entre os dois é *quem
//! produz o número* (um motor de animação · um controlo que o artista está a segurar), nunca o que
//! se faz com ele.
//!
//! # ⚠️⚠️ FUNDIR, nunca acrescentar ao lado
//!
//! O consumidor lê **uma** entrada por forma (`bound_style(id)` devolve a primeira), então uma
//! segunda entrada para a mesma forma seria descartada **em silêncio** — e qual das duas some
//! dependeria da ordem de iteração de um mapa. É a lição que o `vec_widget_drive` já pagou, e este
//! módulo é o **terceiro** produtor da mesma lista.
//!
//! # A ordem no quadro, e por que ela é esta
//!
//! `tokens → ESTE → rows autoradas`. A linha do tempo corre **antes** do controlo que o artista
//! está a segurar, e é o precedente que o passe de estados já escreve: *se as duas coisas escrevem
//! o mesmo objecto, quem manda é o gesto que o artista acabou de fazer; o motor é o estado de
//! fundo*. Arrastar um slider de opacidade durante uma reprodução mostra o slider — se mostrasse a
//! curva, o controlo pareceria partido.

use ph2d_ecs::{Entity, SimWorld, VecDrivenStyle, VecPathRef};
use ph2d_vec_scene::{BoundStyle, VecPathId, VecViewState};

use crate::vec_entities::VecEntityMap;

/// **A opacidade que um motor conduz nesta forma, neste quadro.** Vazio quando nada é conduzido —
/// que é todo documento em que ninguém carregou no play, e é o que mantém o desenho deles
/// byte-idêntico ao mundo sem linha do tempo.
///
/// ⚠️ **Percorre o MAPA e não uma query**, pela mesma razão que os irmãos: o que interessa é o
/// `VecPathId`, e é o mapa que o conhece. Uma entidade morta é saltada em vez de acusada — o
/// ciclo de vida é do `vec_entities::sync`, não desta leitura.
pub(crate) fn resolve(sim: &SimWorld, map: &VecEntityMap) -> Vec<(VecPathId, f32)> {
    let w = sim.world();
    let mut out = Vec::new();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        if w.get_entity(e).is_err() {
            continue;
        }
        // ⚠️ O `VecPathRef` entra no teste **além** do mapa: o componente é escrito por quem
        // reconhece um caminho vetorial, e ler os dois mantém a mesma pergunta com uma resposta só.
        if w.get::<VecPathRef>(e).is_none() {
            continue;
        }
        if let Some(d) = w.get::<VecDrivenStyle>(e)
            && let Some(a) = d.alpha
        {
            out.push((id, a));
        }
    }
    out
}

/// **Funde os valores conduzidos na projecção do quadro.**
///
/// ⚠️ **A conversão para `u8` é `round`, não `as`.** Um `as u8` trunca, e `0.999 * 255 = 254,7`
/// sairia `254`: o topo de um fade nunca fecharia em opaco, e a forma ficaria a um degrau de
/// distância da arte que o artista desenhou — invisível numa cor chapada e visível numa borda.
pub(crate) fn apply(driven: &[(VecPathId, f32)], view: &mut VecViewState) {
    for &(id, a) in driven {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let alpha = (a.clamp(0.0, 1.0) * f32::from(u8::MAX)).round() as u8;
        if let Some(b) = view.bound.iter_mut().find(|b| b.path == id) {
            b.alpha = Some(alpha);
        } else {
            view.bound.push(BoundStyle {
                path: id,
                alpha: Some(alpha),
                ..BoundStyle::default()
            });
        }
    }
}

/// ⭐⭐⭐ **O COMPONENTE VOLTA A SER O AUTORADO depois de o quadro o ler** (v19, 2026-09-05).
///
/// # O defeito que ela fecha
///
/// A ponte da linha do tempo **escreve** [`VecDrivenStyle`] e nunca o apaga: apagar uma track
/// deixava a forma congelada no último valor da curva, **para sempre e sem gesto de volta** — o
/// documento dizia `1,0` e a tela mostrava `0,4`. Enquanto não havia opacidade autorada isso era
/// invisível (não havia outro valor a discordar); com a v19 há, e o silêncio virou contradição.
///
/// ⇒ Depois de a projecção deste quadro ser lida, o componente volta ao valor do DOCUMENTO. Um
/// quadro em que a curva escreve, ela ganha; um quadro em que ninguém escreve, o autorado desenha.
///
/// # ⚠️ Porquê DEPOIS, e não antes
///
/// A lista de estilo (`vec_view.bound`) já foi construída a partir do componente, e é ELA que
/// desenha — repor antes apagaria a curva **deste** quadro. Repor depois deixa o componente
/// correcto para o quadro seguinte, que é onde a ausência de curva se manifesta.
///
/// # ⚠️ E porquê só quem JÁ TEM o componente
///
/// A população é «as formas que a linha do tempo alguma vez conduziu», que é pequena. Semeá-lo em
/// TODA forma da cena poria uma entrada de estilo por forma na `vec_view.bound`, e o
/// `bound_style(id)` é uma varredura linear — `O(N²)` por quadro numa cena de dez mil formas.
/// *Uma cura que custa um quadrado não é uma cura.*
///
/// ⏳ **O que fica em aberto, com o mecanismo nomeado:** uma forma **nunca conduzida** não tem o
/// componente, então as três leitoras de *«quanto vale esta propriedade?»* continuam a responder
/// `1,0` para ela — a tecla **K** sobre uma forma autorada a 40 % grava a 1.ª chave em `1,0`. As
/// duas leitoras da crate (`read_prop_kind`, que semeia o `rest`) **não alcançam o documento
/// vectorial**, e fechá-lo é passar a cena por três assinaturas.
pub(crate) fn settle_to_authored(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    scene: &ph2d_vec_scene::VecScene,
) {
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        let Some(autorada) = scene
            .paths()
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.opacity.get())
        else {
            continue;
        };
        // ⚠️ Só quem tem o componente: `get_mut` não o cria, e é isso que mantém a população
        // pequena (ver o doc acima).
        if let Some(mut d) = sim.world_mut().get_mut::<VecDrivenStyle>(e) {
            let alvo = Some((autorada.clamp(0.0, 1.0) * f32::from(u8::MAX)).round() as u8);
            let alvo = alvo.map(|a| f32::from(a) / f32::from(u8::MAX));
            if d.alpha != alvo {
                d.alpha = alvo;
            }
        }
    }
}

#[cfg(test)]
#[path = "vec_driven_style_tests.rs"]
mod tests;
