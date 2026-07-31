//! **A EMENDA — onde dois colliders se encontram**, e o pivô que toda rota de
//! criação de joint usa como ponto de partida.
//!
//! A regra antiga era o ponto MÉDIO dos centros, e o doc dela dizia o que estava
//! aproximando: *"entre dois corpos que se TOCAM — um elo de corrente — o meio É o
//! pivô certo"*. Estes gates pinam as duas metades: que ela continua acertando
//! onde acertava, e que ela deixou de errar onde errava.

use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_physics_ecs::{Collider, ColliderPose, ColliderShape, seam_point};

fn boxy(hx: f32, hy: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: hx,
            half_y: hy,
        },
        ..Collider::default()
    }
}

fn at(x: f32, y: f32) -> Transform {
    Transform::from_translation(Vec2::new(x, y))
}

fn seam(a: (&Collider, Transform), b: (&Collider, Transform)) -> [f32; 2] {
    seam_point(
        &ColliderPose::resolve(a.0, &a.1),
        &ColliderPose::resolve(b.0, &b.1),
    )
}

/// **O caso que o desenho antigo ACERTAVA continua acertado.** Dois elos iguais
/// que se tocam: as duas bordas caem no mesmo ponto, que é o meio dos centros.
/// É esta linha que torna a troca segura para a corrente que já shipa.
#[test]
fn two_equal_touching_shapes_still_meet_at_the_midpoint_of_their_centres() {
    let c = boxy(0.25, 0.25);
    let p = seam((&c, at(0.0, 0.0)), (&c, at(0.5, 0.0)));
    assert!((p[0] - 0.25).abs() < 1e-5, "x = {}", p[0]);
    assert!((p[1]).abs() < 1e-5, "y = {}", p[1]);
}

/// **E o caso que ele ERRAVA agora acerta.** Uma cabeça pequena encostada no topo
/// de um tronco grande: a junta está no pescoço, e o meio dos centros cai DENTRO
/// do tronco — 0,15 m abaixo, o suficiente para a cabeça girar em torno de um
/// ponto no peito.
#[test]
fn a_small_head_on_a_big_torso_meets_at_the_neck_not_inside_the_chest() {
    let torso = boxy(0.25, 0.5);
    let head = boxy(0.2, 0.2);
    // Tronco em (0,3) ⇒ topo em 3,5. Cabeça em (0,3.7) ⇒ base em 3,5.
    let p = seam((&torso, at(0.0, 3.0)), (&head, at(0.0, 3.7)));
    assert!(
        (p[1] - 3.5).abs() < 1e-5,
        "a emenda saiu em y = {:.4}; a junta está em 3,5 e o MEIO dos centros em \
         3,35 (dentro do tronco)",
        p[1]
    );
}

/// **A ordem dos dois não muda a emenda** — ela é uma propriedade do PAR, e um
/// joint `A→B` e um `B→A` descrevem o mesmo lugar.
#[test]
fn the_seam_does_not_depend_on_which_body_is_asked_first() {
    let torso = boxy(0.25, 0.5);
    let head = boxy(0.2, 0.2);
    let ab = seam((&torso, at(0.0, 3.0)), (&head, at(0.0, 3.7)));
    let ba = seam((&head, at(0.0, 3.7)), (&torso, at(0.0, 3.0)));
    assert!((ab[0] - ba[0]).abs() < 1e-5 && (ab[1] - ba[1]).abs() < 1e-5);
}

/// **A silhueta é medida no frame do CORPO.** Uma caixa girada 90° apresenta a
/// outra meia-extensão à linha dos centros, e a emenda anda com ela — medir no
/// frame do mundo daria a caixa não-girada.
#[test]
fn the_seam_follows_the_bodys_rotation() {
    let tall = boxy(0.1, 0.6);
    let ball = Collider {
        shape: ColliderShape::Ball { radius: 0.1 },
        ..Collider::default()
    };
    // De pé, a caixa alcança 0,1 para o lado.
    let up = seam((&tall, at(0.0, 0.0)), (&ball, at(2.0, 0.0)));
    // Deitada, ela alcança 0,6 para o mesmo lado.
    let mut lying = at(0.0, 0.0);
    lying.rotation = std::f32::consts::FRAC_PI_2;
    let side = seam((&tall, lying), (&ball, at(2.0, 0.0)));
    assert!(
        side[0] > up[0] + 0.2,
        "a emenda não seguiu a rotação: de pé {:.3}, deitada {:.3}",
        up[0],
        side[0]
    );
}

/// **A ESCALA do corpo entra** (W6): dobrar a escala dobra o alcance da silhueta,
/// pela mesma porta `scaled_shape` que o solver e o contorno usam.
#[test]
fn the_seam_honours_the_bodys_scale() {
    let c = boxy(0.25, 0.25);
    let mut big = at(0.0, 0.0);
    big.scale = Vec2::new(2.0, 2.0);
    let plain = seam((&c, at(0.0, 0.0)), (&c, at(2.0, 0.0)));
    let scaled = seam((&c, big), (&c, at(2.0, 0.0)));
    // A meia-extensão vai de 0,25 para 0,50, e a emenda é o MEIO entre as duas
    // bordas ⇒ ela anda exatamente metade do ganho. Um número exato em vez de uma
    // barra: a barra que eu tinha escrito (`> +0,2`) era um palpite, e ela
    // reprovou sobre produto CERTO.
    assert!(
        (scaled[0] - plain[0] - 0.125).abs() < 1e-5,
        "escalar o corpo moveu a emenda {:.4} m; meia-extensão 0,25 → 0,50 move a \
         borda 0,25 e a emenda metade disso",
        scaled[0] - plain[0]
    );
}

/// **O OFFSET do collider entra** (W-Offset): a emenda é entre os COLLIDERS, e o
/// hitbox de um pé não está no centro do corpo.
#[test]
fn the_seam_is_between_the_colliders_not_the_body_origins() {
    let plain = boxy(0.25, 0.25);
    let shifted = Collider {
        offset: [0.5, 0.0],
        ..boxy(0.25, 0.25)
    };
    let a = seam((&plain, at(0.0, 0.0)), (&plain, at(2.0, 0.0)));
    let b = seam((&plain, at(0.0, 0.0)), (&shifted, at(2.0, 0.0)));
    assert!(
        (b[0] - a[0] - 0.25).abs() < 1e-4,
        "o offset do collider moveu a emenda {:.4} m; ele desloca o centro dele \
         0,5 m, e a emenda anda metade disso",
        b[0] - a[0]
    );
}

/// **Centros coincidentes não têm direção** — a resposta é o próprio ponto, sem
/// inventar um eixo (e sem dividir por zero).
#[test]
fn coincident_centres_answer_with_the_point_itself() {
    let c = boxy(0.25, 0.25);
    let p = seam((&c, at(1.0, 2.0)), (&c, at(1.0, 2.0)));
    assert!((p[0] - 1.0).abs() < 1e-6 && (p[1] - 2.0).abs() < 1e-6);
    assert!(p[0].is_finite() && p[1].is_finite());
}
