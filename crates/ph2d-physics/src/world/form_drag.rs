//! **Arrasto de FORMA — a água sabe para onde o corpo está apontando** (W-FormDrag).
//!
//! O `Drag` da área (W-AreaDrag) é **viscosidade**: `v /= 1 + d·dt`, uniforme, igual em
//! toda direção. É o modelo certo para *xarope*, e é o que o rapier, a Unity e o Godot
//! chamam de damping — mas ele não sabe nada sobre a FORMA do corpo, então um tronco
//! atravessado na correnteza desce exatamente como um tronco de proa. E não é isso que
//! um tronco faz.
//!
//! Isto é a outra metade: **resistência de forma**. Cada aresta do polígono que está
//! voltada para o escoamento recebe uma força proporcional ao seu comprimento e à
//! componente da velocidade que a atravessa, aplicada **no meio daquela aresta**.
//!
//! ## ⚠️ O que ele NÃO faz: um corpo simétrico não vira cata-vento
//!
//! Eu construí isto esperando o cata-vento — *"o tronco atravessado gira para a
//! correnteza"* — e **medi zero torque em toda inclinação**. Não é bug: para um corpo
//! **simétrico**, o centro de pressão coincide com o centroide, que é o centro de massa,
//! e `r × F` some aresta por aresta. Uma flecha só se alinha porque as penas ficam
//! ATRÁS do centro de massa; um retângulo centrado não tem esse braço.
//!
//! O que ele **de fato** faz, e é o que vale:
//!
//! - **resiste por SECÇÃO** — o mesmo tronco sofre **4×** mais de través que de proa,
//!   porque a face longa (2,0) enfrenta o escoamento em vez da curta (0,5). Um `Drag`
//!   uniforme dá o mesmo número nas duas poses;
//! - **freia rotação pela FORMA** — o termo `ω × r` faz as arestas distantes do centro
//!   varrerem mais fluido, então um tronco comprido resiste a girar muito mais que uma
//!   bola de mesma área. O damping angular uniforme não sabe disso.
//!
//! A força é aplicada **por aresta, no ponto dela** de qualquer forma: é o que dá o
//! freio de rotação, e é o que permitiria o cata-vento no dia em que o centro de massa
//! deixar de ser o centroide (um lastro, um `MassOverride` deslocado).
//!
//! ## ⚠️ A força age ao longo da NORMAL, não da velocidade — e isso é o modelo inteiro
//!
//! `F = −n · k · L · (v·n) · |v|`. A primeira versão empurrava ao longo de `v` (parece
//! natural: arrasto se opõe ao movimento) e o torque saiu **exatamente zero** em toda
//! inclinação — não por bug, por **identidade**: forças todas paralelas entre si, sobre
//! um polígono fechado, se cancelam em torno do centro. O cata-vento não existiria.
//!
//! Pressão de fluido age **perpendicular à superfície**. Com cada aresta empurrada ao
//! longo da PRÓPRIA normal, as forças deixam de ser paralelas, e é essa não-paralelidade
//! que vira torque. É também o que produz **sustentação** de graça (a componente
//! perpendicular ao escoamento), que é por que uma placa inclinada desliza de lado
//! enquanto cai.
//!
//! Quadrático (`|v|` vezes `v·n`), a mesma lei do [`super::drag`] do MUNDO (`½ρCdA|v|v`)
//! e não a do damping. São modelos diferentes de propósito e o vocabulário segue: *Drag*
//! é viscosidade, *Shape Drag* é resistência de forma, e as duas rows coexistem porque as
//! duas existem na natureza.

use rapier2d::dynamics::{RigidBodyHandle, RigidBodySet};
use rapier2d::geometry::ColliderSet;
use rapier2d::na::{Point2, Vector2};

/// Amostras por aresta. ⚠️ **Duas, e não uma, por um motivo medido:** no MEIO da aresta
/// de um corpo simétrico a velocidade de rotação é exatamente tangencial à superfície,
/// então `v·n` dá zero e uma amostra única mede **freio de rotação nenhum**. O efeito
/// inteiro está no GRADIENTE ao longo da aresta — as pontas varrem fluido, o meio não.
/// Duas amostras já o capturam; mais só refina um número que ninguém lê.
const EDGE_SAMPLES: usize = 2;

/// Aplica o arrasto de forma de uma zona a um corpo, por sub-passo.
pub(crate) fn apply(
    bodies: &mut RigidBodySet,
    colliders: &ColliderSet,
    body: RigidBodyHandle,
    k: f32,
    dt: f32,
) {
    let Some(rb) = bodies.get(body) else {
        return;
    };
    let Some(collider) = rb.colliders().first().and_then(|h| colliders.get(*h)) else {
        return;
    };
    let (pose, com) = (*rb.position(), *rb.center_of_mass());
    let (v, w) = (*rb.linvel(), rb.angvel());
    // Um corpo parado não é resistido por nada — e sair aqui evita acordá-lo à toa.
    if v.norm_squared() <= 0.0 && w == 0.0 {
        return;
    }
    let Some(poly) = super::buoyancy::local_polygon(collider.shape()) else {
        return;
    };
    let impulses = edge_impulses(&poly, &pose, com, v, w, k, dt);
    if let Some(b) = bodies.get_mut(body) {
        for (imp, at) in impulses {
            b.apply_impulse_at_point(imp, at, true);
        }
    }
}

/// O impulso de cada aresta voltada para o escoamento, e onde ele age.
///
/// Separado do `apply` porque é **pura** — dá para perguntar a ela *"que forças esta
/// forma sofre nesta pose?"* sem um mundo, que é como os gates a interrogam.
#[must_use]
fn edge_impulses(
    poly: &[Point2<f32>],
    pose: &rapier2d::na::Isometry2<f32>,
    com: Point2<f32>,
    linvel: Vector2<f32>,
    angvel: f32,
    k: f32,
    dt: f32,
) -> Vec<(Vector2<f32>, Point2<f32>)> {
    let mut out = Vec::with_capacity(poly.len() * EDGE_SAMPLES);
    for i in 0..poly.len() {
        let p = pose * poly[i];
        let q = pose * poly[(i + 1) % poly.len()];
        let edge = q - p;
        let len = edge.norm();
        if len <= 0.0 {
            continue;
        }
        // Normal EXTERNA de um polígono CCW: a aresta girada −90°. (`local_polygon`
        // devolve CCW para toda forma — é a convenção das tesselações que ele reusa.)
        let n = Vector2::new(edge.y, -edge.x) / len;
        let seg = len / EDGE_SAMPLES as f32;
        for k_i in 0..EDGE_SAMPLES {
            // O meio de cada sub-trecho — ver `EDGE_SAMPLES` para por que não basta um.
            let t = (k_i as f32 + 0.5) / EDGE_SAMPLES as f32;
            let at = Point2::from(p.coords + edge * t);
            // A velocidade DAQUELE ponto, não a do corpo: é o termo `ω × r` que faz um
            // corpo girando ser freado pela sua FORMA.
            let r = at - com;
            let v_at = linvel + Vector2::new(-angvel * r.y, angvel * r.x);
            let facing = v_at.dot(&n);
            // Só o que está virado PARA o escoamento resiste. O que está atrás está na
            // esteira, e somá-lo cancelaria o efeito inteiro — a resistência de forma é
            // justamente a assimetria entre frente e esteira.
            if facing <= 0.0 {
                continue;
            }
            // ⚠️ Ao longo de `-n` (para dentro do corpo), NÃO de `-v_at`: ver o
            // cabeçalho. Forças paralelas não geram torque nenhum.
            out.push((n * (-k * seg * facing * v_at.norm() * dt), at));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapier2d::na::Isometry2;

    /// Um retângulo 2×0,5 (um tronco), CCW.
    fn log() -> Vec<Point2<f32>> {
        vec![
            Point2::new(1.0, 0.25),
            Point2::new(-1.0, 0.25),
            Point2::new(-1.0, -0.25),
            Point2::new(1.0, -0.25),
        ]
    }

    #[test]
    fn a_broadside_body_is_resisted_more_than_an_edge_on_one() {
        // O fato inteiro: o MESMO tronco, a MESMA velocidade, resistido conforme a
        // secção que ele oferece. De través a face longa (2,0) enfrenta o escoamento;
        // de proa, só a curta (0,5). Quatro vezes a força.
        let flow = Vector2::new(0.0, -3.0);
        let sum = |pose: &Isometry2<f32>| {
            edge_impulses(&log(), pose, Point2::origin(), flow, 0.0, 1.0, 1.0)
                .iter()
                .fold(Vector2::zeros(), |a, (imp, _)| a + imp)
                .norm()
        };
        let broadside = sum(&Isometry2::identity());
        let edge_on = sum(&Isometry2::rotation(std::f32::consts::FRAC_PI_2));
        assert!(
            (broadside / edge_on - 4.0).abs() < 0.01,
            "de través o tronco tem de sofrer 4x a força de proa (2,0 contra 0,5 de \
             secção), e a razão saiu {}",
            broadside / edge_on
        );
    }

    #[test]
    fn a_symmetric_body_feels_no_torque_and_that_is_geometry_not_a_bug() {
        // ⚠️ O fato que eu esperava ao contrário. Construí este módulo prevendo o
        // cata-vento e medi **zero torque em toda inclinação** — porque num corpo
        // simétrico o centro de pressão é o centroide, que é o centro de massa, e
        // `r × F` some aresta por aresta. Uma flecha só se alinha porque as penas ficam
        // ATRÁS do centro de massa.
        //
        // Pinado para ninguém "consertar" de volta para um torque inventado, e para o
        // dia em que um lastro deslocar o centro de massa: aí ele aparece sozinho, e é
        // este gate que vai mudar de forma junto.
        let flow = Vector2::new(0.0, -3.0);
        for angle in [0.0f32, 0.3, 0.5, 1.0, 2.0] {
            let pose = Isometry2::rotation(angle);
            let torque = edge_impulses(&log(), &pose, Point2::origin(), flow, 0.0, 1.0, 1.0)
                .iter()
                .fold(0.0f32, |t, (imp, at)| t + at.x * imp.y - at.y * imp.x);
            assert!(
                torque.abs() < 1e-4,
                "corpo simétrico não sente torque em escoamento uniforme (ângulo \
                 {angle}, torque {torque})"
            );
        }
    }

    #[test]
    fn a_long_body_resists_spinning_far_more_than_a_compact_one() {
        // O segundo fato que só a FORMA conhece: o termo `ω × r` faz as arestas
        // distantes do centro varrerem mais fluido. Um damping angular uniforme dá o
        // mesmo freio para as duas formas; este dá o freio que a forma merece.
        let spin = |poly: &[Point2<f32>]| {
            edge_impulses(
                poly,
                &Isometry2::identity(),
                Point2::origin(),
                Vector2::zeros(),
                4.0,
                1.0,
                1.0,
            )
            .iter()
            .fold(0.0f32, |t, (imp, at)| t + at.x * imp.y - at.y * imp.x)
            .abs()
        };
        // Um quadrado de MESMA ÁREA que o tronco (2,0 × 0,5 = 1,0 ⇒ lado 1,0).
        let square = vec![
            Point2::new(0.5, 0.5),
            Point2::new(-0.5, 0.5),
            Point2::new(-0.5, -0.5),
            Point2::new(0.5, -0.5),
        ];
        let (long, compact) = (spin(&log()), spin(&square));
        assert!(
            long > compact * 2.0,
            "girando à mesma taxa, o tronco comprido tem de sofrer muito mais freio que \
             o quadrado de mesma área ({long} vs {compact})"
        );
    }

    #[test]
    fn a_tilted_plate_is_pushed_sideways_because_pressure_follows_the_normal() {
        // ⚠️ O gate que a força-ao-longo-da-normal precisa, e que faltava. Descoberto por
        // MUTAÇÃO: trocar a normal pela velocidade deixava os outros três verdes, porque
        // num corpo simétrico não há torque e a magnitude é a mesma — as duas
        // formulações só divergem na componente PERPENDICULAR ao escoamento.
        //
        // Essa componente é **sustentação**, e é o que faz uma placa inclinada planar de
        // lado enquanto cai (uma folha, um cartão). Com a força ao longo de `v` ela é
        // exatamente zero e nada plana.
        let pose = Isometry2::rotation(0.6);
        let side: f32 = edge_impulses(
            &log(),
            &pose,
            Point2::origin(),
            Vector2::new(0.0, -3.0),
            0.0,
            1.0,
            1.0,
        )
        .iter()
        .map(|(imp, _)| imp.x)
        .sum();
        assert!(
            side.abs() > 0.5,
            "uma placa a 0,6 rad tem de receber empurrão LATERAL (sustentação), e recebeu \
             {side} — sinal de que a pressão está seguindo a velocidade em vez da normal"
        );
    }

    #[test]
    fn the_wake_side_does_not_push_back() {
        // Só as arestas viradas para o escoamento contam. Somar as de trás cancelaria o
        // efeito inteiro — a resistência de forma É a assimetria entre frente e esteira.
        let imps = edge_impulses(
            &log(),
            &Isometry2::identity(),
            Point2::origin(),
            Vector2::new(0.0, -3.0),
            0.0,
            1.0,
            1.0,
        );
        assert_eq!(
            imps.len(),
            EDGE_SAMPLES,
            "descendo reto, só a face de BAIXO enfrenta o escoamento — as outras três \
             estão na esteira ou de canto"
        );
    }
}
