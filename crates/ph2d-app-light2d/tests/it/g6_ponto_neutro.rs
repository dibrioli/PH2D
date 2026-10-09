//! **G6 — o ponto neutro** (plano §5): uma cena SEM `Luz` dá o quadro de sempre, byte a byte — a
//! família presente e chamada pelas três portas, contra a mesma cadeia que nunca ouviu falar de luz.
//! Sprites, tonemap e a colagem do vetor entram no quadro comparado.
//!
//! ⛔ Ligar o passe sem a guarda reprova: uma cena com uma `Luz` APAGADA (intensidade 0) acende a
//! cadeia, e o quadro escurece — o controlo de que a igualdade não é vácua.

use bevy_ecs::world::World;
use ph2d_app_light2d::LuzDoQuadro;
use ph2d_ecs::{GlobalTransform, PresentWorld, SpriteEmissive, Transform};
use ph2d_light2d_ecs::{Luz, TipoDeLuz};
use ph2d_render::{LiftedInstances, Sprite};

use crate::comum::{Cadeia, H, W, afim, gpu, quad, quadrado_vetorial};

const COR: [u8; 4] = [200, 120, 60, 255];

/// A cena de sempre: um sprite e um quadrado vetorial. `luz` = acrescenta uma `Luz` APAGADA.
fn mundo(luz: bool) -> World {
    let mut w = World::new();
    let t = Transform::IDENTITY;
    let g = GlobalTransform::from_transform(t);
    // ⚠️ Uma sprite que BRILHA, de propósito: sem `Luz` ela não acende a cena (a guarda é a `Luz`).
    w.spawn((
        Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        SpriteEmissive(2.0),
        t,
        g,
    ));
    if luz {
        w.spawn((
            Luz {
                cor: [1.0; 3],
                intensidade: 0.0,
                tamanho_m: 0.25,
                alcance_m: f32::INFINITY,
                tipo: TipoDeLuz::Ponto,
            },
            t,
            g,
        ));
    }
    w
}

fn quadro(gpu: &ph2d_gpu::GpuContext, familia: Option<&World>) -> Vec<u8> {
    let mut c = Cadeia::nova(gpu);
    let tex = c.textura(COR);
    let mut extra = LiftedInstances::default();
    extra.push(quad(tex, [-1.0, 0.0]), None);
    let mut vetor = vello::Scene::new();
    quadrado_vetorial(&mut vetor, COR, [1.0, 0.0]);
    if let Some(w) = familia {
        let mut luz = LuzDoQuadro::new();
        luz.preparar(w, afim(), (W, H));
        luz.gravar(gpu, &mut PresentWorld::new(), &c.renderer);
        luz.ligar(gpu, &mut c.renderer, &mut c.colagem, &c.tonemap);
    }
    c.quadro(gpu, &mut PresentWorld::new(), &extra, &vetor)
}

#[test]
fn g6_sem_luz_o_quadro_e_o_de_sempre_byte_a_byte() {
    let Some(gpu) = gpu() else {
        eprintln!("g6: sem placa — saltado");
        return;
    };
    let w = mundo(false);
    let mut luz = LuzDoQuadro::new();
    assert!(
        !luz.preparar(&w, afim(), (W, H)),
        "uma cena sem `Luz` (só uma sprite que brilha) não acende a luz 2D"
    );
    let sempre = quadro(&gpu, None);
    let com_familia = quadro(&gpu, Some(&w));
    let diferentes = sempre
        .iter()
        .zip(&com_familia)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        diferentes, 0,
        "a família presente numa cena sem `Luz` mudou {diferentes} bytes do quadro de sempre"
    );
}

/// O controlo: a cadeia ACESA (uma `Luz` a zero — sala escura) muda o quadro. Sem isto, a igualdade
/// acima passaria também com um receptor que nunca multiplica.
#[test]
fn g6_com_a_luz_acesa_o_quadro_muda() {
    let Some(gpu) = gpu() else {
        eprintln!("g6: sem placa — saltado");
        return;
    };
    let w = mundo(true);
    let mut luz = LuzDoQuadro::new();
    assert!(
        luz.preparar(&w, afim(), (W, H)),
        "uma `Luz` apagada ACENDE a cena (escura)"
    );
    let sempre = quadro(&gpu, None);
    let acesa = quadro(&gpu, Some(&w));
    let diferentes = sempre.iter().zip(&acesa).filter(|(a, b)| a != b).count();
    assert!(
        diferentes > 100,
        "a luz acesa devia escurecer o sprite e o vetor — só {diferentes} bytes mudaram"
    );
}
