//! Os gates da porta do carimbo preparado — ver o cabeçalho do [`super`].

use super::PreparedFill;
use crate::VectorScene;
use vello::kurbo::{Affine, BezPath, Point};
use vello::peniko::{Brush, Color, Fill};

/// Uma estrela de `pontas` pontas, fechada — a forma do carimbo do Motion.
fn estrela(pontas: usize) -> BezPath {
    let mut p = BezPath::new();
    for i in 0..(pontas * 2) {
        #[expect(
            clippy::cast_precision_loss,
            reason = "uma contagem de pontas, sempre pequena"
        )]
        let a = (i as f64) * std::f64::consts::PI / (pontas as f64);
        let r = if i % 2 == 0 { 0.5 } else { 0.22 };
        let pt = Point::new(r * a.cos(), r * a.sin());
        if i == 0 {
            p.move_to(pt);
        } else {
            p.line_to(pt);
        }
    }
    p.close_path();
    p
}

/// A tinta da cópia `i` — **diferente em cada cópia**, que é o caso que a rota do fragmento do
/// Vello (`Scene::append`) não sabe servir e esta porta serve.
fn tinta(i: usize) -> Brush {
    #[expect(clippy::cast_precision_loss, reason = "um indice de copia")]
    let t = (i % 7) as f32 / 7.0;
    Brush::Solid(Color::new([t, 1.0 - t, 0.5, 1.0]))
}

/// A pose da cópia `i`.
fn pose(i: usize) -> Affine {
    #[expect(clippy::cast_precision_loss, reason = "um indice de copia")]
    Affine::translate((i as f64 * 0.01, (i % 5) as f64 * 0.02))
}

/// ⭐⭐⭐ **A ROTA PREPARADA ESCREVE OS MESMOS BYTES QUE O `Scene::fill`.**
///
/// ⚠️ **É a única prova que vale.** Um desvio de um byte num fluxo do Vello não devolve erro
/// nenhum — devolve **outro desenho**, e num quadro de `102 400` cópias ninguém o vê a olho. Por
/// isso o gate compara os **seis** fluxos (tags e dados do caminho, tags e dados de desenho,
/// transformações, estilos) **e** os dois contadores, e não uma imagem.
///
/// ⚠️ As tintas são DIFERENTES por cópia de propósito: é aí que a rota do `Scene::append` (que
/// carrega o pincel assado no fragmento) deixaria de servir, e é a razão de esta porta existir.
///
/// ⚠️ Mutação que tem de sangrar: trocar a ordem `estilo → caminho` por `caminho → estilo`;
/// esquecer o `n_path_segments`; usar `Fill::EvenOdd` de um lado.
#[test]
fn o_carimbo_preparado_escreve_os_mesmos_bytes() {
    const N: usize = 64;
    for (pontas, fill) in [
        (5usize, Fill::NonZero),
        (3, Fill::EvenOdd),
        (11, Fill::NonZero),
    ] {
        let bez = estrela(pontas);
        let preparada = PreparedFill::new(&bez, fill);

        let mut hoje = VectorScene::new();
        for i in 0..N {
            hoje.inner_mut().fill(fill, pose(i), &tinta(i), None, &bez);
        }

        let mut carimbo = VectorScene::new();
        for i in 0..N {
            carimbo.fill_prepared(&preparada, pose(i), &tinta(i));
        }

        let (a, b) = (hoje.inner().encoding(), carimbo.inner().encoding());
        assert_eq!(
            a.path_tags.len(),
            b.path_tags.len(),
            "{pontas} pontas: o fluxo de tags do caminho tem outro tamanho"
        );
        assert!(
            a.path_tags == b.path_tags,
            "{pontas} pontas: o fluxo de tags do caminho difere"
        );
        assert_eq!(a.path_data, b.path_data, "{pontas} pontas: coordenadas");
        assert_eq!(
            a.draw_data, b.draw_data,
            "{pontas} pontas: dados de desenho"
        );
        assert_eq!(
            a.draw_tags.len(),
            b.draw_tags.len(),
            "{pontas} pontas: tags de desenho"
        );
        assert_eq!(a.styles, b.styles, "{pontas} pontas: estilos");
        assert_eq!(
            a.transforms.len(),
            b.transforms.len(),
            "{pontas} pontas: transformacoes"
        );
        assert_eq!(a.n_paths, b.n_paths, "{pontas} pontas: n_paths");
        assert_eq!(
            a.n_path_segments, b.n_path_segments,
            "{pontas} pontas: n_path_segments"
        );
        // CONTROLO: a cena tem mesmo conteúdo — sem isto os `assert_eq!` acima comparariam
        // dois vazios e ficariam verdes a afirmar NADA.
        assert!(
            a.n_path_segments as usize >= N * pontas,
            "{pontas} pontas: controlo — a cena de referencia tem de conter {N} copias"
        );
    }
}

/// ⚠️ **Uma forma sem um único segmento deixa a transformação e o estilo escritos e NÃO encoda o
/// pincel** — o que o `Scene::fill` faz quando o `encode_shape` devolve `false`.
///
/// Sem esta metade a porta divergiria exactamente no caso que ninguém encena: uma geometria
/// degenerada no meio de um lote desalinha os fluxos a partir dali, e o desenho seguinte fica com a
/// cor do anterior.
#[test]
fn uma_forma_vazia_segue_a_mesma_lei_do_fill() {
    let vazia = BezPath::new();
    let preparada = PreparedFill::new(&vazia, Fill::NonZero);
    assert!(preparada.is_empty(), "controlo: a forma tem de estar vazia");

    let mut hoje = VectorScene::new();
    hoje.inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, &tinta(1), None, &vazia);

    let mut carimbo = VectorScene::new();
    carimbo.fill_prepared(&preparada, Affine::IDENTITY, &tinta(1));

    let (a, b) = (hoje.inner().encoding(), carimbo.inner().encoding());
    assert!(a.path_tags == b.path_tags, "tags do caminho");
    assert_eq!(a.draw_tags.len(), b.draw_tags.len(), "tags de desenho");
    assert_eq!(a.styles, b.styles, "estilos");
    assert_eq!(a.n_paths, b.n_paths, "n_paths");
    // CONTROLO: o `fill` de hoje também não encoda pincel nenhum aqui.
    assert_eq!(a.draw_tags.len(), 0, "controlo: nenhum objecto de desenho");
}
