//! ⛔ `Encoding::append` (vello 0.11) faz `flags = other.flags`. Colar uma cena vazia logo depois de um texto
//! apagava o «re-emitir o transform» do glifo, e o preenchimento seguinte herdava o transform do último glifo do
//! HUD: o retângulo de selecção de Object saía da janela (`docs/CartoesUI/10`, F6).

use ph2d_vector::{Affine, Color, Rect, VectorScene};

/// Uma cena cujo último desenho é um texto: é o estado em que o `draw_glyphs` do vello a deixa.
fn depois_de_um_texto() -> VectorScene {
    let mut s = VectorScene::new();
    s.fill_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::WHITE);
    s.inner_mut()
        .encoding_mut()
        .force_next_transform_and_style();
    s
}

fn transforms(s: &VectorScene) -> usize {
    s.inner().encoding().transforms.len()
}

#[test]
fn colar_uma_cena_vazia_nao_faz_o_preenchimento_herdar_o_transform_do_texto() {
    let mut s = depois_de_um_texto();
    let antes = transforms(&s);
    s.append_transformed(&VectorScene::new(), Affine::IDENTITY);
    s.fill_rect(Rect::new(0.0, 0.0, 9.0, 9.0), Color::WHITE);
    assert_eq!(
        transforms(&s),
        antes + 1,
        "o preenchimento herdou o transform do glifo"
    );
}

/// O controlo: pelo `append` cru o defeito existe, e é ele que esta régua apanha.
#[test]
fn pelo_append_cru_do_vello_o_preenchimento_herda() {
    let mut s = depois_de_um_texto();
    let antes = transforms(&s);
    let vazia = VectorScene::new();
    s.inner_mut().append(vazia.inner(), None);
    s.fill_rect(Rect::new(0.0, 0.0, 9.0, 9.0), Color::WHITE);
    assert_eq!(
        transforms(&s),
        antes,
        "o vello mudou: a porta pode já não precisar da cura"
    );
}
