//! ⭐ **Os gates do ENQUADRAMENTO** (W46) — e a medição que escolheu a folga.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_render::Orbit;

/// Uma peça **longe da origem e pequena** — o caso que a câmera padrão perde inteiro.
fn far_part(cx: f32, r: f32) -> FieldDoc {
    let ball = |x: f32| Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: r }),
        mods: Vec::new(),
        verb: None,
    };
    FieldDoc::new(
        vec![
            ball(cx - r),
            ball(cx + r),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("a união")
}

/// Quantos pixels da peça tocam a **moldura** do quadro, e que fração dele ela cobre.
fn on_the_border(doc: &FieldDoc, cam: &Orbit, w: u32, h: u32) -> (usize, f64) {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = ph2d_field_render::trace(doc, &reg, cam, w, h);
    let mut border = 0;
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if g.hit[i] && (x == 0 || y == 0 || x == w - 1 || y == h - 1) {
                border += 1;
            }
        }
    }
    (border, f64::from(g.hits() as u32) / f64::from(w * h))
}

/// ⭐ **A VARREDURA que escolheu o `FRAME_MARGIN`** — ela IMPRIME e prende o critério.
///
/// ⚠️ O critério não é estético: *nenhum pixel da peça toca a moldura*. O que ele mede é se
/// enquadrar **corta** a peça — que é o defeito, e é o mesmo que o §34 já pagou na exportação.
/// Uma **esfera sozinha, longe da origem** — o pior caso para um bordo esférico: aqui o bordo **é** a
/// silhueta, e a folga não tem nada de conservador atrás dela.
///
/// ⚠️ A primeira varredura desta wave usou a união de duas esferas e deu **zero pixels na moldura em
/// TODAS as folgas, `0,90` incluída** — porque a união de duas bolas é muito menor do que a bola que
/// a contém. *Uma fixtura que concorda não prova nada*, pela terceira vez nesta linha.
fn lone_sphere(cx: f32, r: f32) -> FieldDoc {
    FieldDoc::new(
        vec![Node {
            xform: Xform::at(cx, 0.0, 0.0),
            kind: NodeKind::Leaf(Primitive::Sphere { radius: r }),
            mods: Vec::new(),
            verb: None,
        }],
        NodeId(0),
    )
    .expect("uma esfera")
}

#[test]
fn the_frame_margin_is_the_smallest_one_that_cuts_nothing() {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    for (what, doc) in [
        ("esfera sozinha (bordo = silhueta)", lone_sphere(3.0, 0.25)),
        ("união de duas (bordo folgado)", far_part(3.0, 0.25)),
    ] {
        let ball = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a peça tem bordo");
        println!("\n{what} — raio do bordo {:.3}", ball.radius);
        println!("folga | pixels na moldura | fração do quadro");
        for m in [0.9_f32, 1.0, 1.05, 1.1, 1.2, 1.4, 1.8] {
            let mut cam = Orbit::from_yaw_pitch(0.72, 0.52);
            cam.target = ball.center;
            cam.half_extent = ball.radius * m;
            let (border, cover) = on_the_border(&doc, &cam, 240, 180);
            println!("{m:>5.2} | {border:>17} | {:>15.1} %", cover * 100.0);
        }
    }
}

/// ⭐⭐ **A FOLGA ESCOLHIDA NÃO CORTA, E A ANTERIOR CORTA** — a barreira tem de ter dentes.
///
/// ⚠️ A segunda metade é o que separa *"o número funciona"* de *"qualquer número funcionaria"*: sem
/// ela, um `FRAME_MARGIN` de 5 passaria com a peça a ocupar 2 % do quadro. A varredura da wave está
/// no doc-comment da constante.
#[test]
fn the_chosen_margin_cuts_nothing_and_the_one_below_it_does() {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = lone_sphere(3.0, 0.25);
    let ball = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("bordo");

    let mut cam = Orbit::from_yaw_pitch(0.72, 0.52);
    crate::field3d_input::law::frame(&mut cam, ball);
    let (border, cover) = on_the_border(&doc, &cam, 240, 180);
    assert_eq!(
        border, 0,
        "a folga que shipa CORTA a peça: {border} pixels dela na moldura"
    );
    assert!(
        cover > 0.25,
        "…e não pode ter comprado isso afastando-se: a peça cobre {:.1} % do quadro",
        cover * 100.0
    );

    // O controle: uma folga apertada corta mesmo. Sem isto, o gate acima passa com qualquer número.
    let mut tight = Orbit::from_yaw_pitch(0.72, 0.52);
    tight.target = ball.center;
    tight.half_extent = ball.radius;
    assert!(
        on_the_border(&doc, &tight, 240, 180).0 > 0,
        "o controle falhou: a folga 1,00 tinha de cortar — se deixou de cortar, a lente mudou e a \
         tabela da constante tem de ser re-medida"
    );
}

/// ⭐⭐ **O `Home` ENCONTRA A PEÇA** — e antes desta wave ele não a encontrava.
///
/// ⚠️ Ele punha o alvo na **origem** e repunha o meio-alcance: uma peça longe dela continuava fora
/// do quadro **depois** da tecla. *A tecla que existe para desfazer «estou perdido» era a única que
/// não sabia onde a peça estava.* A lei é a da referência — no Blender, `Home` é *View All*.
#[test]
fn home_finds_a_part_that_is_far_from_the_origin() {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = lone_sphere(3.0, 0.25);

    // Onde a câmera estava: o enquadramento inicial, na origem. A peça não aparece.
    let start = Orbit::from_yaw_pitch(0.72, 0.52);
    assert_eq!(
        ph2d_field_render::trace(&doc, &reg, &start, 240, 180).hits(),
        0,
        "o controle: a esfera a x=3 não está no quadro da vista inicial"
    );

    // O `Home` — as duas metades, como a tecla as chama.
    let mut cam = start;
    crate::field3d_input::law::home(&mut cam);
    assert_eq!(
        ph2d_field_render::trace(&doc, &reg, &cam, 240, 180).hits(),
        0,
        "só repor a orientação NÃO encontra a peça — era este o defeito"
    );
    let ball = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("bordo");
    crate::field3d_input::law::frame(&mut cam, ball);
    let (border, cover) = on_the_border(&doc, &cam, 240, 180);
    assert!(
        cover > 0.25 && border == 0,
        "depois de enquadrar a peça tem de estar INTEIRA no quadro: {:.1} % e {border} pixels na \
         moldura",
        cover * 100.0
    );
}

/// ⚠️ **Enquadrar não gira** — ele responde *"onde e quão longe"*, nunca *"de que lado"*.
///
/// Sem esta separação, um pedido de enquadrar servido a meio de uma órbita arrancaria a peça da mão
/// do artista — e o `Home` deixaria de poder chamar os dois em ordem.
#[test]
fn framing_never_touches_the_orientation() {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = lone_sphere(3.0, 0.25);
    let ball = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("bordo");

    let mut cam = Orbit::from_yaw_pitch(0.72, 0.52);
    crate::field3d_input::law::orbit(&mut cam, 37.0, -14.0);
    let turned = cam.rotation;
    crate::field3d_input::law::frame(&mut cam, ball);
    assert_eq!(
        cam.rotation, turned,
        "enquadrar mexeu na orientação — o ângulo é do artista"
    );
    assert_eq!(cam.target, ball.center, "…e o alvo é o centro do bordo");
}
