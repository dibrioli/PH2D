//! ⭐⭐ **O PADRÃO CAVALGA A POSE DA FORMA** — os gates do assunto (plano 33).
//!
//! Irmão do [`super::paint_pattern_tests`], e o corte é por RESPONSABILIDADE: aquele mede o **dado**
//! (o tamanho do `Paint`, o período da colmeia, a serialização, o enquadramento do `Clamp`); este
//! mede o que acontece ao padrão quando a **forma** se move, roda ou escala.
//!
//! ⚠️ É a lei que o `paint.rs` já escrevia para os gradientes — *a geometria de um preenchimento
//! vive no espaço das ÂNCORAS e transforma junto com o path* —, e o padrão é o único preenchimento
//! desta casa que conserva também a **orientação**.

use super::paint_pattern_tests::fill;
use super::{Paint, VecPath, VecPathId, VecScene, VecVertex};

/// Uma forma quadrada com o padrão de referência já vestido.
fn scene_with_pattern(f: super::PatternFill) -> (VecScene, VecPathId) {
    let mut scene = VecScene::default();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::Pattern(Box::new(f))),
        ..VecPath::default()
    });
    (scene, id)
}

/// ⭐⭐ **O padrão CONSERVA A ORIENTAÇÃO quando a forma roda** — e é o único preenchimento desta
/// casa que o faz.
///
/// O gradiente radial não pode: um radial do peniko **é circular** e não tem onde guardar um
/// ângulo, e é por isso que o `transform_fill_geometry` lhe passa um `radius_scale` médio. O padrão
/// tem o campo, então a sonda do afim (as imagens dos dois eixos unitários) dá-lhe a resposta exacta.
#[test]
fn rotating_the_shape_rotates_the_pattern_with_it() {
    let (mut scene, id) = scene_with_pattern(fill());
    let quarter = std::f64::consts::FRAC_PI_2;
    assert!(scene.rotate_path_by(id, quarter, [0.0, 0.0]));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert!(
        (p.angle - quarter).abs() < 1e-9,
        "o padrao nao rodou com a forma: {}",
        p.angle
    );
    assert!(
        (p.size[0] - 10.0).abs() < 1e-9 && (p.size[1] - 20.0).abs() < 1e-9,
        "uma rotacao nao pode mudar o tamanho do ladrilho: {:?}",
        p.size
    );
}

/// **Escalar a forma escala o ladrilho, POR EIXO** — e a escala não-uniforme é o fenómeno que a
/// fixtura tinha de conter (plano 33 §5.1).
#[test]
fn scaling_the_shape_scales_the_tile_per_axis() {
    let (mut scene, id) = scene_with_pattern(fill());
    assert!(scene.scale_path(id, 3.0, 0.5, [0.0, 0.0]));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert!(
        (p.size[0] - 30.0).abs() < 1e-9 && (p.size[1] - 10.0).abs() < 1e-9,
        "o ladrilho nao seguiu a escala por eixo: {:?}",
        p.size
    );
    assert!(
        p.angle.abs() < 1e-9,
        "uma escala positiva nao pode rodar o padrao"
    );
}

/// **Mover a forma move o padrão com ela** — a lei que o `paint.rs` já escreveu para os gradientes,
/// e o oposto do defeito da origem-da-régua do Illustrator.
#[test]
fn moving_the_shape_moves_the_pattern_with_it() {
    let (mut scene, id) = scene_with_pattern(fill());
    assert!(scene.translate_path(id, 7.0, -3.0));
    let Some(Paint::Pattern(p)) = &scene.paths()[0].fill else {
        panic!("especie trocada")
    };
    assert_eq!(p.origin, [7.0, -3.0]);
    assert!(p.angle.abs() < 1e-9 && (p.size[0] - 10.0).abs() < 1e-9);
}

/// ⭐⭐⭐ **A ESTAMPA DO CONTORNO cavalga a CANETA; a do preenchimento cavalga a FORMA** — e as duas
/// leis coincidem **ao bit** em todo afim conforme.
///
/// # O que esta redacção substitui, e porquê
///
/// A anterior chamava-se `..._exactly_like_the_fills` e exigia igualdade **em todo afim**. Ela
/// curou o defeito de que a estampa do traço não seguia a forma de todo — e escreveu de mais:
/// report do Enio de 2026-08-30, **"funciona, mas a proporção muda no stroke (estica/achata o
/// tile)"**.
///
/// ⚠️ **Duas leis colidiam dentro do mesmo traço.** A estampa herdava o afim CRU (a lei do
/// preenchimento, *"o padrão está colado à forma"*) e a faixa herdava o `√|det|` (a lei da caneta,
/// decisão do dono no bug #27, *"quando engrossa, engrossa por igual nos dois eixos"*). ⇒ uma banda
/// que não esticou com um motivo que esticou. Medido: sob `(3, 1)` o ladrilho ia a **3,00×** o
/// aspecto autorado enquanto a banda ficava redonda.
///
/// # A régua tem DUAS metades, e a de cima é a que protege o caso comum
///
/// ⭐ **Conforme ⇒ as duas tintas concordam ao bit.** É o que garante que rodar, transladar ou
/// escalar por igual não muda um pixel — a esmagadora maioria dos gestos.
///
/// ⛔ **Não-conforme ⇒ o traço NÃO acompanha o esticão**, e a divergência tem sentido: o aspecto do
/// ladrilho do traço fica no autorado, o do preenchimento estica. *Sem a metade de baixo, uma
/// redacção que uniformizasse os DOIS passaria — e o preenchimento tem de esticar.*
#[test]
fn the_strokes_pattern_follows_the_pen_and_the_fills_follows_the_shape() {
    let forma = |scene: &mut VecScene| {
        scene.push_path(VecPath {
            verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            fill: Some(Paint::Pattern(Box::new(fill()))),
            stroke: Some({
                let mut s = crate::StrokeSpec::new(crate::Rgba8::new(9, 9, 9, 255), 1.0);
                s.paint = crate::StrokePaint::Pattern(Box::new(fill()));
                s
            }),
            ..VecPath::default()
        })
    };
    let tintas = |scene: &VecScene| {
        let path = &scene.paths()[0];
        let Some(Paint::Pattern(f)) = &path.fill else {
            panic!("o preenchimento trocou de especie")
        };
        let s = path
            .stroke
            .as_ref()
            .and_then(crate::StrokeSpec::pattern)
            .expect("o traco continua estampado");
        (f.clone(), s.clone(), path.stroke.as_ref().unwrap().width)
    };
    let aut = fill().size;
    let asp = |s: [f64; 2]| s[0] / s[1];

    // ⭐ METADE DE CIMA — conforme (rotação + escala UNIFORME): concordam AO BIT.
    let mut a = VecScene::default();
    let id = forma(&mut a);
    assert!(a.rotate_path_by(id, std::f64::consts::FRAC_PI_2, [0.0, 0.0]));
    assert!(a.scale_path(id, 3.0, 3.0, [0.0, 0.0]));
    let (f, s, w) = tintas(&a);
    assert_eq!(
        (s.angle, s.size, s.origin),
        (f.angle, f.size, f.origin),
        "sob um afim CONFORME as duas tintas divergiram - o caso comum mudou"
    );
    assert!(
        (w - 3.0).abs() < 1e-12,
        "a largura do traco nao acompanhou a escala uniforme: {w} contra 3,0 - o ladrilho triplicou \
         e a banda ficou parada"
    );

    // ⛔ METADE DE BAIXO — NÃO-conforme `(3, 1)`: o preenchimento estica, o traço NÃO.
    let mut b = VecScene::default();
    let id = forma(&mut b);
    assert!(b.scale_path(id, 3.0, 1.0, [0.0, 0.0]));
    let (f, s, w) = tintas(&b);
    assert!(
        (asp(f.size) / asp(aut) - 3.0).abs() < 1e-9,
        "o PREENCHIMENTO devia esticar 3x o aspecto e foi a {:.4}x - ele esta' colado a' forma",
        asp(f.size) / asp(aut)
    );
    assert!(
        (asp(s.size) - asp(aut)).abs() < 1e-9,
        "o TRACO esticou o ladrilho ({:.4} contra o autorado {:.4}) - e' o report de 30/08: a banda \
         nao estica e o motivo dentro dela estica",
        asp(s.size),
        asp(aut)
    );
    // ⭐ E a banda e o motivo movem-se pelo MESMO factor — que é a lei inteira.
    let k = 3.0f64.sqrt();
    assert!(
        (w - k).abs() < 1e-12 && (s.size[0] / aut[0] - k).abs() < 1e-9,
        "a banda ({w}) e o ladrilho ({}) discordaram - os dois tem de seguir o sqrt(|det|) = {k}",
        s.size[0] / aut[0]
    );
}
