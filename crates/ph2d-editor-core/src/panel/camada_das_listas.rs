//! ⭐ **Uma lista aberta fica por cima de tudo** (R4, foto `docs/CartoesUI/maquete/j_timeline_track_aberto.png`:
//! o HUD tapava «Opacity» na lista de «+ Track»). Cada painel pinta as listas no fim do PRÓPRIO `paint`, e
//! depois dele vêm os outros painéis, os cartões soltos, a barra de ferramentas e o HUD.
//!
//! A porta: o quadro do hero [`abre`] a camada, os pintores de lista pintam por [`por_cima`] e registam
//! alvos por [`alvo`], e o hero [`fecha`] a camada depois de todo o chrome e antes das dicas e dos menus.
//! Sem camada aberta (um painel pintado sozinho num teste, um modal já por cima) pinta-se direto na cena.

use crate::interaction::HitIndex;
pub use crate::widget::camada_das_listas::{abre, por_cima};
use crate::widget::camada_das_listas::{guarda_alvo, tira};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_vector::{Affine, VectorScene};

/// Regista o alvo de uma linha da lista: agora, para quem lê o índice no mesmo passe, e outra vez ao
/// fechar, por cima do que se registou depois.
pub fn alvo(hit_index: &mut HitIndex, id: NodeId, rect: Rect) {
    hit_index.register(id, rect);
    guarda_alvo(id, rect);
}

/// Cola a camada por cima de `scene`, e os alvos por cima do índice.
pub fn fecha(scene: &mut VectorScene, hit_index: &mut HitIndex) {
    let Some(mut camada) = tira() else {
        return;
    };
    scene.append_transformed(&camada.cena, Affine::IDENTITY);
    scene.extend_record(camada.cena.take_record());
    for (id, rect) in camada.alvos {
        hit_index.register(id, rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LISTA: NodeId = NodeId(1);
    const POR_CIMA: NodeId = NodeId(2);

    fn r() -> Rect {
        Rect::new(0.0, 0.0, 10.0, 10.0)
    }

    /// O que se pinta e regista depois da lista fica por baixo dela.
    #[test]
    fn a_lista_ganha_de_quem_pinta_depois_dela() {
        let mut scene = VectorScene::new();
        scene.record();
        let mut hits = HitIndex::new();
        abre(&scene);
        por_cima(&mut scene, |s| {
            s.fill_rect(
                ph2d_vector::Rect::new(0.0, 0.0, 1.0, 1.0),
                ph2d_vector::Color::WHITE,
            );
        });
        alvo(&mut hits, LISTA, r());
        scene.fill_rect(
            ph2d_vector::Rect::new(0.0, 0.0, 2.0, 2.0),
            ph2d_vector::Color::BLACK,
        );
        hits.register(POR_CIMA, r());
        assert_eq!(
            scene.take_record().len(),
            1,
            "a lista saiu da cena do painel"
        );
        fecha(&mut scene, &mut hits);
        assert_eq!(scene.take_record().len(), 1, "a lista não voltou ao fechar");
        assert_eq!(hits.hit(5.0, 5.0), Some(LISTA));
    }

    /// R5: o seletor de datas (verso do cartão, Planner) passa pela porta.
    /// *Mutação: `paint_date_picker` a pintar direto em `scene`.*
    #[test]
    fn o_seletor_de_datas_vai_para_a_camada() {
        let mut scene = VectorScene::new();
        scene.record();
        let mut hits = HitIndex::new();
        let mut ts = ph2d_text::TextSystem::without_system_fonts();
        let p = crate::widget::showcase::card_back::demo_date_picker(LISTA);
        abre(&scene);
        crate::widget::paint_date_picker(
            &p,
            Rect::new(0.0, 0.0, 300.0, 400.0),
            &mut scene,
            &mut ts,
            ph2d_tokens::Theme::default(),
        );
        assert!(
            scene.take_record().is_empty(),
            "o seletor ficou na cena do painel"
        );
        fecha(&mut scene, &mut hits);
        assert!(!scene.take_record().is_empty(), "o seletor perdeu-se");
    }

    /// Sem camada aberta, pinta-se direto: o painel sozinho e os modais continuam a ver a lista.
    #[test]
    fn sem_camada_a_lista_fica_na_cena_dada() {
        let mut scene = VectorScene::new();
        scene.record();
        por_cima(&mut scene, |s| {
            s.fill_rect(
                ph2d_vector::Rect::new(0.0, 0.0, 1.0, 1.0),
                ph2d_vector::Color::WHITE,
            );
        });
        assert_eq!(scene.take_record().len(), 1);
    }
}
