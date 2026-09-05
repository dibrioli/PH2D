//! Gates da PILHA DE APARÊNCIA no desenho (v20) — o que ela emite, e o que ela **não** emite.
//!
//! ⚠️ O oráculo é a CODIFICAÇÃO do Vello (`n_paths` / `n_clips`), e não um pixel: é o mesmo oráculo
//! que os irmãos deste ficheiro usam, e ele responde exactamente às duas perguntas que uma pilha
//! levanta — *quantas marcas saíram* e *quantas camadas de composição foram abertas*.

use ph2d_vec_scene::{Paint, PaintEntry, Rgba8, StrokeSpec, VecPath, VecVertex};
use ph2d_vector::{Affine, VectorScene};

fn quadrado() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(10, 20, 30, 255))),
        stroke: Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0)),
        ..VecPath::default()
    }
}

/// Quantos caminhos e quantos recortes a codificação levou ao desenhar `p`.
fn conta(p: &VecPath) -> (u32, u32) {
    let mut t = VectorScene::new();
    crate::draw_path_tiled(p, Affine::IDENTITY, &mut t, None, None, None);
    let e = t.inner().encoding();
    (e.n_paths, e.n_clips)
}

/// ⭐⭐⭐ **CADA CAMADA LIGADA DESENHA, e uma camada NEUTRA não abre composição.**
///
/// ⚠️ As duas metades no mesmo gate de propósito: *desenha* sozinho ficaria verde num renderer que
/// abrisse uma camada de mistura por tinta (o custo que a pilha existe para não ter), e *não abre*
/// sozinho ficaria verde num renderer que não desenhasse nada.
#[test]
fn every_active_layer_draws_and_a_neutral_one_opens_no_composition() {
    let base = quadrado();
    let (paths_base, clips_base) = conta(&base);

    let mut com_pilha = quadrado();
    com_pilha.paints = vec![
        PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 128))),
        PaintEntry::stroke(StrokeSpec::new(Rgba8::new(255, 255, 255, 255), 3.0)),
    ];
    let (paths_pilha, clips_pilha) = conta(&com_pilha);
    assert_eq!(
        paths_pilha,
        paths_base + 2,
        "duas camadas ligadas = duas marcas a mais"
    );
    assert_eq!(
        clips_pilha, clips_base,
        "e nenhuma delas e' neutra? sao — entao NAO se abre camada de composicao nenhuma"
    );
}

/// ⭐ **UMA CAMADA COM OPACIDADE PRÓPRIA ABRE UMA COMPOSIÇÃO** — e é isso que a distingue de
/// baixar o alfa da cor: ela compõe a marca INTEIRA sobre o que está por baixo dentro da forma.
#[test]
fn a_layer_with_its_own_opacity_opens_one_composition() {
    let mut p = quadrado();
    let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)));
    e.opacity = ph2d_vec_scene::Opacity::new(0.4);
    p.paints = vec![e];
    let (_, clips) = conta(&p);
    let (_, clips_base) = conta(&quadrado());
    assert!(
        clips > clips_base,
        "a camada com opacidade propria tem de abrir composicao: {clips} contra {clips_base}"
    );
}

/// ⭐ **E COM MISTURA TAMBÉM**, mesmo opaca — a mistura é o outro motivo para compor.
#[test]
fn a_layer_with_a_blend_mode_opens_one_composition_even_when_opaque() {
    let mut p = quadrado();
    let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)));
    e.blend = ph2d_vec_scene::BlendMode::Multiply;
    p.paints = vec![e];
    let (_, clips) = conta(&p);
    let (_, clips_base) = conta(&quadrado());
    assert!(clips > clips_base, "{clips} contra {clips_base}");
}

/// **O OLHO: uma camada desligada não desenha, e os parâmetros ficam.**
///
/// ⚠️ Mutação que tem de sangrar: ignorar o `enabled`. O artista desarmaria uma camada e ela
/// continuaria a pintar — e o remédio dele seria apagá-la, perdendo a tinta.
#[test]
fn a_disabled_layer_draws_nothing_and_keeps_its_paint() {
    let mut p = quadrado();
    let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)));
    e.enabled = false;
    p.paints = vec![e];
    assert_eq!(
        conta(&p),
        conta(&quadrado()),
        "desligada, a camada nao emite marca nenhuma"
    );
    let ph2d_vec_scene::PaintKind::Fill(Paint::Solid(c)) = &p.paints[0].kind else {
        panic!("a tinta continua la'");
    };
    assert_eq!(c.r, 255, "e a cor dela nao se perdeu");
}

/// ⭐⭐⭐ **UM CONTORNO DE LARGURA ZERO NÃO É UMA CAMADA** — senão ele abriria composição e emitiria
/// uma marca invisível, que é custo por nada. É o mesmo teste (`width > 0`) que o chão já faz.
#[test]
fn a_zero_width_stroke_layer_is_not_a_layer() {
    let mut p = quadrado();
    p.paints = vec![PaintEntry::stroke(StrokeSpec::new(
        Rgba8::new(255, 255, 255, 255),
        0.0,
    ))];
    assert_eq!(conta(&p), conta(&quadrado()));
}

/// ⭐⭐⭐ **A CAIXA DA FORMA É A DO CONTORNO MAIS GORDO DA PILHA** — e não a do de base.
///
/// ⚠️ Ela dimensiona o scratch do FX e o rectângulo da camada de mistura: medi-la na base fazia um
/// traço extra largo ser **recortado**, que é o sintoma da ponta CEIFADA que o `standalone` já
/// documenta, uma tinta adiante.
///
/// Mutação que tem de sangrar: ler `path.stroke` em vez de percorrer o [`VecPath::paint_stack`].
#[test]
fn the_box_grows_with_the_widest_stroke_in_the_stack() {
    let estreita = crate::path_bounds_under(&quadrado(), Affine::IDENTITY).expect("ha' caixa");
    let mut p = quadrado();
    p.paints = vec![PaintEntry::stroke(StrokeSpec::new(
        Rgba8::new(255, 255, 255, 255),
        20.0,
    ))];
    let larga = crate::path_bounds_under(&p, Affine::IDENTITY).expect("ha' caixa");
    assert!(
        larga.x0 < estreita.x0 - 8.0 && larga.x1 > estreita.x1 + 8.0,
        "a caixa tem de crescer com o traco de 20: {larga:?} contra {estreita:?}"
    );
}

/// **O CONTROLO: sem pilha, o desenho é o de sempre** — sem ele os gates acima ficariam verdes
/// sobre um renderer que mudou o caminho comum.
#[test]
fn a_shape_without_a_stack_encodes_exactly_what_it_encoded() {
    let p = quadrado();
    assert!(p.paints.is_empty());
    let (n, c) = conta(&p);
    assert!(n >= 2, "preenchimento + traco: {n}");
    assert_eq!(c, 0, "e nenhum recorte");
}
