//! Gates da tradução do modo de mistura.

use super::{is_neutral, offered, vello_blend};
use ph2d_vec_scene::{BlendMode, MAX_BLEND_MODES};
use ph2d_vector::{Compose, Mix, VelloBlend};

fn todos() -> impl Iterator<Item = BlendMode> {
    (0..MAX_BLEND_MODES).map(BlendMode::from_u8)
}

/// ⭐⭐⭐ **A LISTA QUE O PAINEL OFERECE É A DOS MODOS QUE DESENHAM** — e as recusas são
/// exactamente três, pelo nome.
///
/// ⚠️ **Sem contagem literal**: o gate afirma o CONJUNTO recusado, então acrescentar um modo ao
/// vocabulário do app não vem editar este teste — ele só falha se um modo novo cair no lado errado
/// **em silêncio**, que é a única coisa que interessa aqui.
///
/// Mutação que tem de sangrar: dar tradução a um dos três (ele passa a ser oferecido e desenha
/// `Normal`, porque o shader do Vello não o tem).
#[test]
fn the_offered_list_is_derived_and_the_three_refusals_are_named() {
    let recusados: Vec<BlendMode> = todos().filter(|m| vello_blend(*m).is_none()).collect();
    assert_eq!(
        recusados,
        vec![
            BlendMode::LinearBurn,
            BlendMode::VividLight,
            BlendMode::LinearLight
        ],
        "os tres do Photoshop que o W3C nao tem sao os UNICOS sem traducao"
    );
    let oferecidos: Vec<BlendMode> = offered().collect();
    assert_eq!(
        oferecidos.len(),
        todos().count() - recusados.len(),
        "a lista oferecida e' o vocabulario menos as recusas — nada mais, nada menos"
    );
    assert!(
        !oferecidos.iter().any(|m| recusados.contains(m)),
        "um modo sem traducao chegou ao painel: ele gravaria e desenharia Normal"
    );
}

/// ⭐⭐⭐ **`Normal` É O CAMINHO NEUTRO, e mais nenhum oferecido é.**
///
/// ⚠️ As duas metades importam. A primeira é o que mantém byte-idêntico todo documento que nunca
/// tocou no campo (sem `Normal` neutro, cada forma da cena passaria a empurrar uma camada de
/// mistura). A segunda é o censo do **controlo morto**: um modo que se lesse neutro seria oferecido
/// no dropdown, gravado no documento, e **não desenharia nada de diferente**.
#[test]
fn normal_is_the_fast_path_and_no_other_offered_mode_is() {
    assert!(is_neutral(BlendMode::Normal), "Normal tem de ser o neutro");
    for m in offered().filter(|m| *m != BlendMode::Normal) {
        assert!(
            !is_neutral(m),
            "{m:?} oferece-se no painel e compõe-se como Normal — controlo morto"
        );
    }
    // Um modo SEM tradução conta como neutro de propósito (o painel não o oferece, mas um ficheiro
    // pode trazê-lo): desenhar Normal é a única resposta honesta que sobra.
    assert!(is_neutral(BlendMode::VividLight));
}

/// **OS TRÊS OPERADORES DE COMPOSIÇÃO não são modos de mistura** — e o `Clear` é `DestOut`.
///
/// ⚠️ **A distinção do `Clear` é o achado desta wave e vale um gate:** o `Compose::Clear` zera a
/// região INTEIRA da camada (um rectângulo de buraco no desenho, incluindo onde a forma não pinta),
/// e o que o nosso vocabulário promete é *"reduz o alfa do fundo pelo alfa da FONTE"* — o
/// `DestOut`. Os dois compilam, e só um faz o que a palavra diz.
#[test]
fn the_three_composition_operators_carry_a_neutral_mix() {
    for (m, c) in [
        (BlendMode::Add, Compose::Plus),
        (BlendMode::Behind, Compose::DestOver),
        (BlendMode::Clear, Compose::DestOut),
    ] {
        assert_eq!(
            vello_blend(m),
            Some(VelloBlend::new(Mix::Normal, c)),
            "{m:?} e' composicao, nao mistura"
        );
    }
}

/// **Os dezasseis modos de MISTURA traduzem-se um-a-um** — o conjunto do W3C, que é o do SVG, do
/// PDF, do CSS, do Illustrator e do Rive.
///
/// ⚠️ O gate percorre a tradução e conta quantos saem com `Compose::SrcOver`: é a definição de
/// «é um modo de mistura», e não uma lista escrita à mão a envelhecer ao lado da função.
#[test]
fn sixteen_modes_translate_as_pure_mix() {
    let mistura = todos()
        .filter_map(vello_blend)
        .filter(|b| b.compose == Compose::SrcOver)
        .count();
    assert_eq!(
        mistura, 16,
        "o conjunto de mistura do W3C tem dezasseis, e e' esse que o shader do Vello implementa"
    );
}

/// ⭐⭐⭐ **UMA FORMA NEUTRA NÃO EMPURRA CAMADA, e uma com opacidade EMPURRA.**
///
/// ⚠️ **O oráculo é o `n_clips` da cena do Vello** — uma camada É um clip na codificação dele —, e
/// não uma bandeira nossa: a pergunta é *o que foi encodado*, que é o que a placa vai executar.
///
/// ⛔ **A metade neutra é a que paga o produto:** sem ela, toda forma de toda cena passaria a
/// empurrar uma camada de mistura (memória de blend por tile, no fine do Vello) para compor
/// exactamente o que o `SrcOver` já compõe de graça.
#[test]
fn only_a_non_neutral_object_pushes_a_layer() {
    use ph2d_vec_scene::{Opacity, Paint, Rgba8, VecPath, VecScene, VecVertex, VecXforms};
    use ph2d_vector::{Affine, VectorScene};

    let encode = |mutate: &dyn Fn(&mut VecPath)| {
        let mut scene = VecScene::default();
        let mut p = VecPath {
            verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            fill: Some(Paint::solid(Rgba8::new(10, 20, 30, 255))),
            ..VecPath::default()
        };
        mutate(&mut p);
        scene.push_path(p);
        let mut target = VectorScene::new();
        crate::dispatch(
            &scene,
            &ph2d_vec_scene::VecViewState::default(),
            &VecXforms::new(),
            &crate::LiveGeometry::new(),
            &crate::FxImages::new(),
            &crate::WidgetSkins::new(),
            &crate::PatternTiles::new(),
            &crate::BrushArts::new(),
            Affine::IDENTITY,
            &mut target,
        );
        let enc = target.inner().encoding().clone();
        (enc.n_clips, enc.draw_data.clone())
    };

    let (clips_neutro, tinta_neutra) = encode(&|_| {});
    assert_eq!(
        clips_neutro, 0,
        "uma forma opaca em Normal nao pode custar uma camada"
    );

    // ⚠️ **`2` é UMA camada**: a codificação do Vello conta os dois extremos do clip (o `begin` e
    // o `end`), e é por isso que este gate compara contra o NEUTRO em vez de contra um número
    // escrito à mão — o que interessa é *houve camada ou não*.
    let (clips_meio, _) = encode(&|p| p.opacity = Opacity::new(0.5));
    assert_eq!(clips_meio, 2, "meia opacidade compoe-se numa camada");

    let (clips_mistura, tinta_mistura) = encode(&|p| p.blend = BlendMode::Multiply);
    assert_eq!(clips_mistura, 2, "um modo de mistura compoe-se numa camada");
    // ⚠️ **`ends_with`, e não igualdade:** a camada acrescenta o registo dela (o par mistura +
    // alfa) ANTES da tinta da forma. O que este gate afirma é que a **cor autorada sai intacta** —
    // a mistura é composição, não tinta —, e é exactamente a diferença entre esta wave e o atalho
    // que ela recusou (escrever a opacidade nas cores).
    assert!(
        tinta_mistura.ends_with(&tinta_neutra),
        "a mistura mexeu na TINTA da forma: {tinta_mistura:?} nao termina em {tinta_neutra:?}"
    );
}

/// ⭐⭐⭐ **COM FILTRO, A CAMADA É A CAIXA DA IMAGEM — não a da forma.**
///
/// ⚠️ **É a metade que faz a estrela do report de ontem desvanecer com o brilho:** a imagem de FX
/// **substitui** o desenho e é MAIOR que a forma (a pilha tem alcance — um halo, uma sombra), então
/// medir a forma recortaria o halo contra a borda da camada. O sintoma seria um brilho com uma
/// aresta recta, que é pior que nenhum brilho.
///
/// ⚠️ E sem filtro a caixa é a da FORMA, pela mesma porta que dimensiona o scratch do FX — uma
/// segunda medição aqui seria a superfície pela qual a camada recorta a arte que o FX não recorta.
#[test]
fn a_filtered_object_layers_over_the_images_box() {
    use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecScene, VecVertex, VecXforms};
    use ph2d_vector::Affine;

    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(10, 20, 30, 255))),
        ..VecPath::default()
    });
    let (xf, live) = (VecXforms::new(), crate::LiveGeometry::new());

    let sem = super::layer_rect(
        &scene,
        &xf,
        &live,
        &crate::FxImages::new(),
        id,
        Affine::IDENTITY,
    )
    .expect("a forma tem caixa");

    let mut fx = crate::FxImages::new();
    let halo = (-40.0, -40.0, 50.0, 50.0);
    fx.insert(
        id,
        crate::FxImage {
            image: ph2d_vector::StableImage::from_rgba(std::sync::Arc::new(vec![0u8; 4]), 1, 1)
                .expect("1x1"),
            rect: halo,
        },
    );
    let com = super::layer_rect(&scene, &xf, &live, &fx, id, Affine::IDENTITY).expect("com imagem");
    assert_eq!(com, halo, "a camada tem de cobrir a IMAGEM inteira");
    assert!(
        com.0 < sem.0 && com.2 > sem.2,
        "a fixtura tem de ter a imagem MAIOR que a forma, senao o gate nao mede nada: {com:?} vs \
         {sem:?}"
    );
}
