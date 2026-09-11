//! ⭐⭐⭐ **O POLÍGONO DE `N` VÉRTICES AUTORADOS** (W132) — o irregular e o **côncavo**, que nem o
//! prisma (só regulares) nem o triângulo (três, e sempre convexo) alcançam.
//!
//! | gate | o defeito que ele apanha |
//! |---|---|
//! | `the_polygon_is_the_extrusion_of_its_own_contour` | uma **segunda** fórmula para a mesma superfície |
//! | `the_notch_is_really_a_notch` | a concavidade evaporar (um casco convexo em vez do contorno) |
//! | `growing_the_count_does_not_move_the_surface` | a peça **saltar** ao pedir mais um vértice |
//! | `the_count_goes_up_and_down_and_comes_back` | as duas leis da contagem não serem inversas |
//! | `the_fillet_rounds_the_rim_and_leaves_the_typed_corners_alone` | o filete deixar de ser do **aro** (e a nota que eu ia acreditar dizia o contrário) |
//! | `the_field_is_one_lipschitz_however_the_corners_lie` | a peça **rasgar** (medido pela DEFINIÇÃO) |

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn campo(p: Primitive) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça"),
    )
}

/// O contorno côncavo de referência — cinco vértices, o quarto reentrante.
fn concavo() -> Vec<[f32; 2]> {
    vec![
        [-0.38, -0.22],
        [0.36, -0.12],
        [0.14, 0.38],
        [0.02, 0.04],
        [-0.22, 0.28],
    ]
}

fn poly(pontos: Vec<[f32; 2]>, round: f32) -> Primitive {
    Primitive::Polygon {
        profile: ph2d_field::polygon_profile(pontos).expect("o contorno"),
        half_height: 0.12,
        round,
        chamfer: 0.0,
    }
}

/// ⭐⭐⭐ **UMA LEI, UM SÍTIO** — o campo do polígono é, ponto a ponto, o do `Extrude` do mesmo
/// contorno.
///
/// ⚠️ **É o gate que impede a segunda fórmula.** A distância a um polígono simples (o `min` sobre os
/// segmentos com o sinal do enrolamento) já era calculada nesta casa há waves, e escrever uma
/// segunda cópia dela para a primitiva nova seria a lei em dois sítios — *a que diverge é sempre a
/// mais nova, que é a que ninguém está a olhar*. A partilha também é o que traz de graça a
/// especialização por ladrilho e o filete do aro.
#[test]
fn the_polygon_is_the_extrusion_of_its_own_contour() {
    let profile = ph2d_field::polygon_profile(concavo()).expect("o contorno");
    let poligono = campo(Primitive::Polygon {
        profile: profile.clone(),
        half_height: 0.12,
        round: 0.03,
        chamfer: 0.0,
    });
    let extrusao = campo(Primitive::Extrude {
        profile,
        half_height: 0.12,
        round: 0.03,
        chamfer: 0.0,
    });
    let mut pior = 0.0_f64;
    for i in 0..24 {
        for j in 0..24 {
            for k in 0..6 {
                let p = [
                    -0.6 + 1.2 * f64::from(i) / 23.0,
                    -0.6 + 1.2 * f64::from(j) / 23.0,
                    -0.3 + 0.6 * f64::from(k) / 5.0,
                ];
                let d = (poligono.at(p[0], p[1], p[2]) - extrusao.at(p[0], p[1], p[2])).abs();
                pior = pior.max(d);
            }
        }
    }
    assert!(
        pior == 0.0,
        "o polígono e a extrusão do mesmo contorno divergem em {pior:.3e} — são DUAS fórmulas, e \
         só devia haver uma"
    );
}

/// ⭐⭐ **O ENTALHE é mesmo um entalhe** — o ponto fora do contorno mas dentro do casco convexo está
/// **fora** da peça.
///
/// ⚠️ É a propriedade que separa esta forma de todas as outras chapas da paleta: o prisma, a
/// estrela e o triângulo têm o contorno decidido pela fórmula, e nenhum deles admite uma quina que
/// entra. *Um gate que só medisse «a peça existe» ficaria verde sobre um casco convexo.*
#[test]
fn the_notch_is_really_a_notch() {
    let f = campo(poly(concavo(), 0.0));
    // O ponto entre o vértice reentrante e a corda que fecharia o casco: dentro do casco, fora da
    // peça.
    let fora = f.at(0.04, 0.20, 0.0);
    assert!(
        fora > 1.0e-3,
        "o ponto do entalhe leu {fora:.5} e devia estar FORA — o contorno virou casco convexo"
    );
    // E o miolo continua dentro, senão o gate acima ficaria verde sobre uma peça vazia.
    let dentro = f.at(0.0, -0.10, 0.0);
    assert!(
        dentro < -1.0e-3,
        "o miolo leu {dentro:.5} e devia estar DENTRO — a peça desapareceu"
    );
}

/// ⭐⭐⭐ **PEDIR MAIS UM VÉRTICE NÃO MEXE NA PEÇA** — a lei da contagem, medida no campo.
///
/// ⚠️ **A alternativa óbvia era regenerar um polígono regular**, e ela apagaria o trabalho do
/// artista a cada clique. A lei que shipa parte a aresta **mais longa** ao meio: o ponto novo cai em
/// cima da aresta antiga, logo a superfície é a mesma — e é isso que este gate mede, e não o
/// código que a produz.
#[test]
fn growing_the_count_does_not_move_the_surface() {
    let antes = campo(poly(concavo(), 0.0));
    let mais = ph2d_field::with_one_more_vertex(&concavo());
    assert_eq!(mais.len(), 6, "a lei tinha de acrescentar exactamente um");
    let depois = campo(poly(mais, 0.0));
    let mut pior = 0.0_f64;
    for i in 0..32 {
        for j in 0..32 {
            let p = [
                -0.6 + 1.2 * f64::from(i) / 31.0,
                -0.6 + 1.2 * f64::from(j) / 31.0,
            ];
            pior = pior.max((antes.at(p[0], p[1], 0.0) - depois.at(p[0], p[1], 0.0)).abs());
        }
    }
    assert!(
        pior < 1.0e-6,
        "a peça mexeu-se {pior:.3e} ao ganhar um vértice — o gesto de contar não pode mudar a forma"
    );
}

/// ⭐⭐ **SUBIR E DESCER A CONTAGEM É A IDENTIDADE** — e não por promessa: o vértice que nasce no meio
/// de uma aresta tem desvio de área **exactamente zero**, logo é o primeiro que a lei de descer tira.
///
/// ⚠️ **É isto que torna a linha da contagem segura de arrastar.** Uma lei de descer que tirasse *o
/// último* deixaria o par de gestos a destruir a forma um pedaço de cada vez, e o artista só daria
/// por isso ao largar.
#[test]
fn the_count_goes_up_and_down_and_comes_back() {
    let base = concavo();
    for _ in 0..4 {
        let ida = ph2d_field::with_one_more_vertex(&base);
        let volta = ph2d_field::with_one_fewer_vertex(&ida);
        assert_eq!(
            volta, base,
            "subir e descer a contagem devolveu outro polígono — as duas leis não são inversas"
        );
    }
}

/// ⭐⭐⭐ **O FILETE É DO ARO, e a quina DIGITADA fica viva** — a lei medida, pinada.
///
/// ⛔⛔ **A primeira redacção deste gate afirmava o contrário** (*«um filete grande ABRE a peça: o
/// pescoço mais fino que `2·round` desaparece»*), porque eu li a nota que estava escrita no
/// `sd_extrude` em vez de medir. A medição refutou as duas metades: o pescoço **fica** (leu
/// `−0,035`, que é exactamente o campo sem filete nenhum) e a quina vertical **não se mexe**.
///
/// ⭐ O mecanismo é `(flat + r) − r = flat`: `flat + r` é um **minorante** da distância ao conjunto
/// erodido, e a igualdade falha exactamente numa quina convexa — compor um minorante com o inverso
/// do que ele minora não é a identidade. O que de facto arredonda é o **aro**, onde as duas
/// coordenadas (parede e laje) são ortogonais e o `√(a² + b²)` é a distância a sério.
///
/// ⇒ Este gate PINA a lei, para que ninguém a «corrija» por acidente: um filete que passasse a
/// mexer na quina do contorno seria uma forma diferente da que o `Extrude` shipa há waves, e as
/// duas têm de continuar a ser a mesma.
#[test]
fn the_fillet_rounds_the_rim_and_leaves_the_typed_corners_alone() {
    let quadrado = vec![[-0.2, -0.2], [0.2, -0.2], [0.2, 0.2], [-0.2, 0.2]];
    let vivo = campo(poly(quadrado.clone(), 0.0));
    let redondo = campo(poly(quadrado, 0.05));
    // A quina VERTICAL é a mesma nos dois — o filete não lhe toca.
    let (a, b) = (vivo.at(0.2, 0.2, 0.0), redondo.at(0.2, 0.2, 0.0));
    assert!(
        (a - b).abs() < 1.0e-6,
        "a quina digitada mexeu-se com o filete ({a:.5} para {b:.5}) — esta forma promete a quina \
         que o artista escreveu"
    );
    // E o ARO recua exactamente `√2·r − r`, que é o quarto de círculo perfeito.
    let aro = redondo.at(0.2, 0.0, 0.12);
    let esperado = f64::from(0.05_f32) * (2.0_f64.sqrt() - 1.0);
    assert!(
        (aro - esperado).abs() < 1.0e-4,
        "o aro leu {aro:.5} e o quarto de círculo de raio 0,05 dá {esperado:.5} — o filete deixou \
         de arredondar a aresta que ele arredonda"
    );
}

/// ⭐⭐⭐ **O CAMPO É `1`-LIPSCHITZ, com o contorno que vier** — a constante **pela definição**, e não
/// por diferença central.
///
/// ⚠️ **A régua é o quociente `|f(p) − f(q)| / ‖p − q‖` sobre pares**, e não `‖∇f‖`: a diferença
/// central **sobre-lê** num vinco convexo (ela mede a corda de um bico), o que faria este gate
/// acusar uma peça correcta. *A definição é a única régua que não tem esse ponto cego* — foi a
/// lição que a W131 pagou.
#[test]
fn the_field_is_one_lipschitz_however_the_corners_lie() {
    for (nome, pontos, round) in [
        ("côncavo", concavo(), 0.0),
        ("côncavo com filete", concavo(), 0.05),
        (
            "lasca fina",
            vec![[-0.40, -0.004], [0.40, 0.0], [-0.40, 0.004]],
            0.0,
        ),
    ] {
        let f = campo(poly(pontos, round));
        let mut pior = 0.0_f64;
        for i in 0..40 {
            for j in 0..40 {
                let p = [
                    -0.6 + 1.2 * f64::from(i) / 39.0,
                    -0.6 + 1.2 * f64::from(j) / 39.0,
                ];
                let h = 1.0e-3;
                for (dx, dy) in [(h, 0.0), (0.0, h), (h * 0.7, h * 0.7)] {
                    let a = f.at(p[0], p[1], 0.0);
                    let b = f.at(p[0] + dx, p[1] + dy, 0.0);
                    pior = pior.max((b - a).abs() / dx.hypot(dy));
                }
            }
        }
        assert!(
            pior <= 1.0 + 1.0e-3,
            "o campo do polígono «{nome}» tem constante de Lipschitz {pior:.4} — acima de 1 a marcha \
             passa a superfície e a peça rasga"
        );
    }
}
