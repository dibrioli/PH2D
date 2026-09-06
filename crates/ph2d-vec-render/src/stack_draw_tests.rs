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
    crate::draw_path_tiled(p, Affine::IDENTITY, &mut t, crate::Derived::NONE);
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

/// ⛔ **SONDA (não é gate): quanto custa uma pilha funda?** — para o tecto de camadas sair de uma
/// medição e não de um palpite (§0.0).
#[test]
fn probe_the_cost_of_a_deep_stack() {
    for n in [1usize, 4, 16, 32, 64] {
        let mut p = quadrado();
        p.paints = (0..n)
            .map(|k| {
                let mut e = PaintEntry::stroke(StrokeSpec::new(
                    Rgba8::new(200, 100, 50, 255),
                    1.0 + k as f64,
                ));
                e.opacity = ph2d_vec_scene::Opacity::new(0.9);
                e
            })
            .collect();
        let t0 = std::time::Instant::now();
        let (paths, clips) = conta(&p);
        let dt = t0.elapsed();
        eprintln!(
            "[stack-cost] camadas={n:<3} caminhos={paths:<4} recortes={clips:<4} encode={:?}",
            dt
        );
    }
}

/// ⭐⭐⭐ **O DESLOCAMENTO DE UMA CAMADA SOFRE A POSE DA FORMA** — a ordem do afim.
///
/// ⚠️ **O caso de omissão NÃO testa isto:** com `offset = [0, 0]` as duas ordens dão o mesmo afim,
/// e com a forma sem rotação nem escala também. O que as separa é uma forma **rodada** — ali,
/// `translate ∘ transform` deixaria a sombra a andar no eixo do ECRÃ enquanto a forma roda por
/// baixo dela, e o artista veria a sombra descolar ao rodar o objecto.
///
/// A régua é o ponto para onde a origem da camada vai: sob `rot(90°)`, um deslocamento de `+1` em
/// `x` (o eixo da FORMA) tem de aterrar em `+1` no `y` do mundo.
#[test]
fn a_layers_offset_rides_the_shapes_pose() {
    let rodado = Affine::rotate(std::f64::consts::FRAC_PI_2);
    let onde = super::camada_xf(rodado, [1.0, 0.0]);
    let p = onde * ph2d_vector::Point::ZERO;
    assert!(
        (p.x - 0.0).abs() < 1e-9 && (p.y - 1.0).abs() < 1e-9,
        "a camada nao seguiu a pose: ({}, {}) — a ordem do afim esta' invertida",
        p.x,
        p.y
    );
    // O CONTROLO: o neutro devolve o afim de sempre, AO BIT — é o que mantém byte-idêntico o
    // desenho de toda pilha que não desloca nada.
    assert_eq!(
        super::camada_xf(rodado, [0.0, 0.0]).as_coeffs(),
        rodado.as_coeffs(),
        "o neutro nao e' byte-identico"
    );
}

/// ⭐⭐⭐ **A CAIXA DA FORMA COBRE A CAMADA DESLOCADA.**
///
/// ⛔ Sem isto ela é **recortada**: esta caixa dimensiona o scratch do FX e o rectângulo da camada
/// de mistura, e é a MESMA ponta CEIFADA que o `inflate_for_stroke` já documenta duas vezes — a
/// terceira vez que este ficheiro paga a conta, uma tinta adiante.
#[test]
fn the_shapes_box_covers_an_offset_layer() {
    let mut p = quadrado();
    p.paints = vec![{
        let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)));
        e.offset = [5.0, -3.0];
        e
    }];
    let r = ph2d_vector::Rect::new(0.0, 0.0, 10.0, 10.0);
    let inflada = crate::standalone::inflate_for_stroke(&p, Affine::IDENTITY, r);
    assert!(
        inflada.x1 >= r.x1 + 5.0 && inflada.y0 <= r.y0 - 3.0,
        "a caixa nao cobre a camada deslocada: {inflada:?}"
    );
    // O CONTROLO: a MESMA pilha no neutro devolve a caixa que a forma já dava.
    //
    // ⚠️ **A 1.ª redacção comparava com o rectângulo CRU e reprovou** — `quadrado()` tem traço de
    // base, então a caixa infla `2` em cada lado com pilha ou sem ela. *Um controlo que mede uma
    // grandeza que a feature não toca acusa o produto pelo que já era verdade.*
    let mut limpa = quadrado();
    limpa.paints = vec![PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)))];
    assert_eq!(
        crate::standalone::inflate_for_stroke(&limpa, Affine::IDENTITY, r),
        crate::standalone::inflate_for_stroke(&quadrado(), Affine::IDENTITY, r),
        "uma pilha sem deslocamento mudou a caixa"
    );
}

/// ⭐⭐⭐ **UMA CAMADA DILATADA DESENHA A GEOMETRIA DELA, E NÃO A DA FORMA.**
///
/// Pedido do Enio, 2026-09-05: *"o offset do cad, contraindo e dilatando"*.
///
/// ⚠️ **A régua é a CONTAGEM DE SEGMENTOS que chegou ao codificador**, e não a de marcas: uma
/// camada dilatada emite exactamente UMA marca, como qualquer outra — o que muda é a geometria.
/// Um gate que contasse caminhos ficaria verde sobre um renderer que ignorasse a dilatação por
/// inteiro.
///
/// A fixtura é construída para a régua: a forma é um QUADRADO e a geometria dilatada é um
/// TRIÂNGULO. Se o renderer usar a dilatada, o codificador recebe **menos** segmentos.
#[test]
fn a_dilated_layer_draws_its_own_geometry_and_not_the_shapes() {
    let mut p = quadrado();
    let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)));
    e.dilate = 4.0;
    p.paints = vec![e];

    let mut triangulo = VecPath {
        verts: [[-4.0, -4.0], [14.0, -4.0], [5.0, 14.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Solid(Rgba8::new(255, 0, 0, 255))),
        ..VecPath::default()
    };
    triangulo.id = p.id;
    let mut mapa = crate::DilatedPaints::new();
    mapa.insert((p.id, 0), triangulo);

    let (marcas_com, segs_com) = conta_dilatada(&p, Some(&mapa));
    let (marcas_sem, segs_sem) = conta_dilatada(&p, None);
    assert_eq!(
        marcas_com, marcas_sem,
        "uma camada dilatada emite as MESMAS marcas — o que muda e' a geometria"
    );
    assert_eq!(
        segs_com + 1,
        segs_sem,
        "a camada nao desenhou a geometria DILATADA: o triangulo tem um segmento a MENOS que o \
         quadrado, e a contagem nao mudou ({segs_com} contra {segs_sem})"
    );
}

/// **O CONTROLO: uma forma SEM dilatação codifica exactamente o que codificava**, mesmo com o mapa
/// presente — é o que mantém byte-idêntico o desenho de todo documento que não lhe toca.
#[test]
fn a_shape_with_no_dilated_layer_encodes_exactly_what_it_encoded() {
    let mut p = quadrado();
    p.paints = vec![PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)))];
    let mapa = crate::DilatedPaints::new();
    assert_eq!(
        conta_dilatada(&p, Some(&mapa)),
        conta_dilatada(&p, None),
        "um mapa VAZIO nao pode mudar nada"
    );
}

/// Marcas emitidas + segmentos codificados, com (ou sem) o mapa de dilatação.
fn conta_dilatada(p: &VecPath, dilated: Option<&crate::DilatedPaints>) -> (u32, u32) {
    let mut t = VectorScene::new();
    crate::draw_path_tiled(
        p,
        Affine::IDENTITY,
        &mut t,
        crate::Derived {
            dilated,
            ..crate::Derived::NONE
        },
    );
    let e = t.inner().encoding();
    (e.n_paths, e.n_path_segments)
}

/// ⭐⭐ **A CAIXA COBRE UMA CAMADA QUE CRESCE** — e não infla por uma que encolhe.
///
/// ⚠️ As duas metades: sem a primeira o anel de CAD sai **recortado** na borda do scratch (a
/// quarta vez que este ficheiro paga a conta); sem a segunda toda forma com um offset negativo
/// pagaria scratch por uma geometria que fica DENTRO dela.
#[test]
fn the_box_covers_a_growing_layer_and_not_a_shrinking_one() {
    let base = crate::path_bounds_under(&quadrado(), Affine::IDENTITY).expect("ha' caixa");

    let mut cresce = quadrado();
    let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)));
    e.dilate = 6.0;
    cresce.paints = vec![e];
    let r = crate::path_bounds_under(&cresce, Affine::IDENTITY).expect("ha' caixa");
    assert!(
        r.x0 <= base.x0 - 6.0 && r.x1 >= base.x1 + 6.0,
        "a caixa nao cobre a camada crescida: {r:?} contra {base:?}"
    );

    let mut encolhe = quadrado();
    let mut e2 = PaintEntry::fill(Paint::Solid(Rgba8::new(255, 0, 0, 255)));
    e2.dilate = -6.0;
    encolhe.paints = vec![e2];
    assert_eq!(
        crate::path_bounds_under(&encolhe, Affine::IDENTITY).expect("ha' caixa"),
        base,
        "encolher fica DENTRO da forma — inflar por ele e' folga por nada"
    );
}
