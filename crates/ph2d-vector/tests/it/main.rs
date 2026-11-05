//! ⛔ `Encoding::append` (vello 0.11) faz `flags = other.flags`. Colar uma cena vazia logo depois de um texto
//! apagava o «re-emitir o transform» do glifo, e o preenchimento seguinte herdava o transform do último glifo do
//! HUD: o retângulo de selecção de Object saía da janela (`docs/CartoesUI/10`, F6).

mod censo_das_colagens;

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

/// As irmãs têm a mesma lei: a sem transform e a de `Scene` crua (`ph2d-render`).
#[test]
fn as_irmas_da_porta_tambem_devolvem_o_pedido_do_texto() {
    let vazia = VectorScene::new();
    let mut a = depois_de_um_texto();
    let mut b = depois_de_um_texto();
    let antes = transforms(&a);
    a.append_scene(&vazia);
    ph2d_vector::scene::colar_cena(b.inner_mut(), vazia.inner(), None);
    for s in [&mut a, &mut b] {
        s.fill_rect(Rect::new(0.0, 0.0, 9.0, 9.0), Color::WHITE);
        assert_eq!(
            transforms(s),
            antes + 1,
            "o preenchimento herdou o transform do glifo"
        );
    }
}

/// A colagem sã: uma cena com desenho dá o mesmo desenho pela porta e pelo `append` cru.
#[test]
fn a_porta_nao_muda_o_que_uma_colagem_sa_desenha() {
    let parte = depois_de_um_texto();
    let mut cru = VectorScene::new();
    let mut porta = VectorScene::new();
    cru.inner_mut().append(parte.inner(), None);
    porta.append_scene(&parte);
    let (c, p) = (cru.inner().encoding(), porta.inner().encoding());
    assert!(c.path_tags == p.path_tags, "path_tags");
    assert!(c.path_data == p.path_data, "path_data");
    assert!(c.draw_tags == p.draw_tags, "draw_tags");
    assert!(c.draw_data == p.draw_data, "draw_data");
    assert!(c.transforms == p.transforms, "transforms");
    assert!(c.styles == p.styles, "styles");
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
