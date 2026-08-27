//! **A PORTA DE RENDER do padrão** (plano 33, W2) — e os gates dela CONTAM em vez de cronometrar.
//!
//! # Porque estes gates lêem o ENCODING
//!
//! Nenhum deles precisa de GPU, e nenhum lê um relógio. O `vello::Scene` expõe o `Encoding`
//! (`n_clips`, `styles`, `transforms`, `draw_data`) — um oráculo **exacto** para as três coisas que
//! esta wave promete: que a regra de preenchimento viaja, que a colocação viaja, e que o modo de
//! repetição viaja. É o mesmo precedente do
//! [`encode_cost_tests`](../../ph2d-vec-render/src/encode_cost_tests.rs) da crate irmã, cujo
//! cabeçalho regista que a 1ª versão dele **era uma razão de tempo e a mutação SOBREVIVEU**.
//!
//! # ⚠️ Os dois argumentos que estavam MORTOS no `fill_path`
//!
//! O [`VectorScene::fill_path`] chama `self.inner.fill(Fill::NonZero, transform, brush, None, path)`
//! — a regra **fixada** e o `brush_transform` **sempre `None`**. Os dois são exactamente o que um
//! padrão precisa, e é por isso que a porta é nova em vez de um caminho novo dentro da velha.

use super::*;
use std::sync::Arc;
use vello::kurbo::Affine;
use vello::peniko::{Extend, Fill, ImageQuality};

/// Um quadrado unitário — geometria mínima que ainda produz um caminho encodado.
fn square() -> BezPath {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    bp.line_to((1.0, 1.0));
    bp.line_to((0.0, 1.0));
    bp.close_path();
    bp
}

/// Um ladrilho 2x2 com quatro cores distintas.
fn tile() -> StableImage {
    let px: Vec<u8> = vec![
        255, 0, 0, 255, // vermelho
        0, 255, 0, 255, // verde
        0, 0, 255, 255, // azul
        255, 255, 0, 255, // amarelo
    ];
    StableImage::from_rgba(Arc::new(px), 2, 2).expect("2x2 RGBA")
}

/// Encoda UM preenchimento de padrão com os parâmetros dados e devolve a cena.
#[allow(clippy::too_many_arguments)]
fn encode(rule: Fill, brush_xf: Affine, x_ext: Extend, y_ext: Extend, alpha: f32) -> VectorScene {
    let mut s = VectorScene::new();
    s.fill_path_image(
        &square(),
        rule,
        Affine::IDENTITY,
        &tile(),
        brush_xf,
        x_ext,
        y_ext,
        ImageQuality::Medium,
        alpha,
    );
    s
}

fn styles(s: &VectorScene) -> Vec<u32> {
    s.inner()
        .encoding()
        .styles
        .iter()
        .map(|st| st.flags_and_miter_limit)
        .collect()
}

/// ⚠️ **A REGRA DE PREENCHIMENTO VIAJA** — e sem isto um padrão numa forma composta com `EvenOdd`
/// pintaria o buraco.
///
/// Não é hipótese: é exactamente a pedra em que o `fill_multipoint` tropeçou, e o comentário dele
/// (*"`VectorScene::push_clip` hardcodes NonZero, which would paint the gradient over a compound's
/// hole"*) é o registo. O `fill_path` tem o mesmo defeito e continua a tê-lo — esta porta é a que
/// não o tem.
#[test]
fn the_fill_rule_reaches_the_encoding() {
    let id = Affine::IDENTITY;
    let nz = encode(Fill::NonZero, id, Extend::Repeat, Extend::Repeat, 1.0);
    let eo = encode(Fill::EvenOdd, id, Extend::Repeat, Extend::Repeat, 1.0);
    // Controlo: a mesma regra duas vezes tem de dar o mesmo estilo — senão o teste abaixo mediria
    // ruído de encode em vez da regra.
    let nz2 = encode(Fill::NonZero, id, Extend::Repeat, Extend::Repeat, 1.0);
    assert_eq!(
        styles(&nz),
        styles(&nz2),
        "o encode tem de ser determinista"
    );
    assert_ne!(
        styles(&nz),
        styles(&eo),
        "a regra do caminho nao chegou ao encoding: um compound EvenOdd pintaria o buraco"
    );
}

/// ⭐⭐ **A COLOCAÇÃO VIAJA, e é ela que faz o padrão cavalgar a pose da forma.**
///
/// O Vello compõe `transform * brush_transform` (`vello-0.8.0/src/scene.rs:329`) — então passar a
/// colocação como `brush_transform` é o que põe o padrão em espaço de ÂNCORAS e o faz transformar
/// junto com o path, que é a lei que o `paint.rs` já escreveu para os gradientes.
///
/// ⚠️ Com `None` (o que o `fill_path` faz hoje) o padrão ficaria colado à TELA e escorregaria por
/// baixo da forma — que é precisamente o defeito da origem-da-régua do Illustrator.
#[test]
fn the_brush_transform_reaches_the_encoding() {
    let id = encode(
        Fill::NonZero,
        Affine::IDENTITY,
        Extend::Repeat,
        Extend::Repeat,
        1.0,
    );
    let moved = encode(
        Fill::NonZero,
        Affine::translate((17.0, -5.0)) * Affine::scale(3.0),
        Extend::Repeat,
        Extend::Repeat,
        1.0,
    );
    assert_ne!(
        id.inner().encoding().transforms,
        moved.inner().encoding().transforms,
        "a colocacao do padrao nao chegou ao encoding"
    );
}

/// ⭐⭐⭐ **O MODO DE REPETIÇÃO VIAJA — e é isto que torna o ladrilhado NATIVO.**
///
/// Medido ao nível do bit (plano 33 §0.1): o `vello_encoding` empacota `x_extend` e `y_extend` em
/// `sample_alpha` (bits 10-11 e 8-9) e o `fine.wgsl` lê-os e honra-os. Este gate prova que os
/// nossos chegam lá — sem ele, todo padrão seria `Pad` e o ladrilho apareceria **uma vez** com as
/// bordas esticadas pelo resto da forma.
#[test]
fn the_extend_mode_reaches_the_encoding() {
    let id = Affine::IDENTITY;
    let data = |x, y| {
        encode(Fill::NonZero, id, x, y, 1.0)
            .inner()
            .encoding()
            .draw_data
            .clone()
    };
    let pad = data(Extend::Pad, Extend::Pad);
    assert_ne!(pad, data(Extend::Repeat, Extend::Repeat), "Repeat = Pad");
    assert_ne!(pad, data(Extend::Reflect, Extend::Reflect), "Reflect = Pad");
    // ⚠️⚠️ **Os DOIS eixos são independentes — e a 1ª redacção deste gate NÃO o media.**
    //
    // Ela comparava `(Repeat, Pad)` contra `(Pad, Repeat)`. Com os dois extends escritos pelo MESMO
    // argumento (o que um `.with_extend(x)` faz), o primeiro vira `(Repeat, Repeat)` e o segundo
    // `(Pad, Pad)` — que **ainda diferem**, e o gate passava. A mutação sobreviveu, e nomeou o
    // defeito: aquilo media *"o x importa"*, não *"o y tem vida própria"*.
    //
    // A régua certa **prende o x** e mexe só no y. ⛔ *O dano vive um passo à frente do que o gate
    // da feature olha* — é a quinta vez que esta linha o paga.
    assert_ne!(
        data(Extend::Repeat, Extend::Pad),
        data(Extend::Repeat, Extend::Repeat),
        "o eixo Y foi ignorado: os dois extends sao escritos pelo mesmo argumento"
    );
    assert_ne!(
        data(Extend::Pad, Extend::Repeat),
        data(Extend::Repeat, Extend::Repeat),
        "o eixo X foi ignorado"
    );
}

/// **A alfa do padrão viaja.** ⚠️ A fileira *Fill Opacity* que o painel já tem escreve a alfa da
/// COR do estilo da ferramenta, que um padrão não tem — a alfa dele é a do pincel de imagem.
#[test]
fn the_pattern_alpha_reaches_the_encoding() {
    let id = Affine::IDENTITY;
    let full = encode(Fill::NonZero, id, Extend::Repeat, Extend::Repeat, 1.0);
    let half = encode(Fill::NonZero, id, Extend::Repeat, Extend::Repeat, 0.5);
    assert_ne!(
        full.inner().encoding().draw_data,
        half.inner().encoding().draw_data,
        "a alfa do padrao nao chegou ao encoding"
    );
}

/// ⭐ **O preenchimento com padrão NÃO empurra camada nenhuma** — é o kill-criterion do plano 33 §6.
///
/// A rota antiga de imagem desta casa (o `fill_multipoint`) rasteriza um buffer na CPU, empurra um
/// `push_clip` e faz um blit; um padrão não precisa de nada disso. Uma camada por forma com padrão
/// seria o desenho errado, e a resposta é achar porquê — ⛔ não subir a barra.
#[test]
fn a_pattern_fill_pushes_no_clip_layer() {
    let s = encode(
        Fill::NonZero,
        Affine::IDENTITY,
        Extend::Repeat,
        Extend::Repeat,
        1.0,
    );
    assert_eq!(s.inner().encoding().n_clips, 0, "o padrao empurrou camada");
    assert_eq!(s.inner().encoding().n_open_clips, 0);
    assert_eq!(
        s.inner().encoding().n_paths,
        1,
        "um preenchimento com padrao e' UM caminho encodado, como uma cor chapada"
    );
    // Controlo: a rota de clip existe e é distinguível — senão este gate passaria numa cena vazia.
    // ⚠️ **`n_clips` conta DOIS por camada** (o registo de abertura e o de fecho), medido aqui: um
    // `push_clip_with_rule` + `pop_layer` dá `2`, não `1`. O número não é o que se adivinha — e é
    // por isso que o controlo existe.
    let mut clipped = VectorScene::new();
    clipped.push_clip_with_rule(&square(), Fill::NonZero);
    assert_eq!(
        clipped.inner().encoding().n_open_clips,
        1,
        "aberta e por fechar"
    );
    clipped.pop_layer();
    assert_eq!(clipped.inner().encoding().n_clips, 2);
    assert_eq!(clipped.inner().encoding().n_open_clips, 0);
}
