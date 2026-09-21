//! Os gates do [`super::ViewRegion`] — e o que decide é o **PIXEL**, nunca a matriz.

use super::ViewRegion;
use crate::{Camera3d, Lens};
use glam::{Mat4, Vec3};

/// A vista inteira, em pixels: uma janela `1600×900` com o canvas todo.
const VISTA: [f32; 4] = [0.0, 0.0, 1600.0, 900.0];

fn camera(lens: Lens) -> Camera3d {
    Camera3d {
        yaw: 0.6,
        pitch: 0.35,
        distance: 4.0,
        lens,
        ..Camera3d::default()
    }
}

/// Onde `p` cai, em pixels da vista `rect`, projectado pelo `region` daquela vista.
///
/// ⚠️ **A régua passa pela PORTA DO PRODUTO** ([`Camera3d::view_proj_in`]) e não por uma segunda
/// montagem da matriz: um gate que monta a projecção à mão afirma sobre aritmética que o produto
/// não corre.
fn pixel(cam: &Camera3d, p: Vec3, rect: [f32; 4], region: ViewRegion) -> (f32, f32) {
    let aspect = VISTA[2] / VISTA[3];
    let clip = cam.view_proj_in(aspect, region) * p.extend(1.0);
    let ndc = clip.truncate() / clip.w;
    (
        rect[0] + (ndc.x + 1.0) * 0.5 * rect[2],
        rect[1] + (1.0 - ndc.y) * 0.5 * rect[3],
    )
}

/// ⭐ **A vista inteira é a IDENTIDADE**, e é isso que faz o caminho de omissão não pagar nada.
#[test]
fn a_vista_inteira_e_a_identidade() {
    assert_eq!(ViewRegion::FULL.to_clip(), Mat4::IDENTITY);
    assert!(ViewRegion::FULL.is_full());
    assert_eq!(ViewRegion::default(), ViewRegion::FULL);
}

/// ⚠️ Uma vista degenerada não tem fracção — e devolver uma dividiria por zero.
#[test]
fn uma_vista_degenerada_nao_tem_fraccao() {
    assert_eq!(
        ViewRegion::of_px([0.0, 0.0, 10.0, 10.0], [0.0, 0.0, 0.0, 900.0]),
        None
    );
    assert_eq!(ViewRegion::of_px([0.0, 0.0, 0.0, 10.0], VISTA), None);
}

/// ⚠️ O rectângulo IGUAL à vista é a vista inteira — e não «quase».
#[test]
fn o_rectangulo_igual_a_vista_e_a_vista_inteira() {
    let r = ViewRegion::of_px(VISTA, VISTA).expect("a vista cheia");
    assert!(r.is_full(), "{r:?}");
}

/// ⭐⭐⭐ **A LEI, e é ela que o report do dono pede: um ponto que cai no pixel `p` da vista cai
/// no MESMO pixel do sub-rectângulo.**
///
/// ⚠️ **A régua é o PIXEL DE JANELA dos dois lados**, e não o NDC: é a grandeza em que o artista
/// julga *«o assado está onde eu pus o objecto»*, e é a única que atravessa a mudança de aspecto
/// entre a vista e o recorte.
///
/// ⚠️ E ela corre nas **DUAS lentes**: a convergente é a que precisa do frustum assimétrico (um
/// *dolly* não a exprime), e a paralela é onde um erro de sinal passaria despercebido por simetria.
#[test]
fn o_recorte_poe_o_mundo_no_mesmo_pixel_da_vista() {
    for lens in Lens::ALL {
        let cam = camera(lens);
        // Um sub-rectângulo que não é centrado nem quadrado — os dois casos em que um sinal
        // trocado ou um aspecto herdado sobreviveriam.
        let sub = [240.0, 120.0, 520.0, 380.0];
        let region = ViewRegion::of_px(sub, VISTA).expect("o recorte");
        for p in [
            Vec3::ZERO,
            Vec3::new(0.7, 0.4, -0.3),
            Vec3::new(-0.5, -0.6, 0.2),
            Vec3::new(0.2, -0.9, 0.8),
        ] {
            let na_vista = pixel(&cam, p, VISTA, ViewRegion::FULL);
            let no_recorte = pixel(&cam, p, sub, region);
            assert!(
                (na_vista.0 - no_recorte.0).abs() < 1e-2
                    && (na_vista.1 - no_recorte.1).abs() < 1e-2,
                "{lens:?}: {p:?} cai em {na_vista:?} na vista e em {no_recorte:?} no recorte"
            );
        }
    }
}

/// ⭐⭐ **O CONTROLO — sem o recorte, o mesmo ponto cai noutro sítio, e por MUITO.**
///
/// ⚠️ **Sem esta metade a irmã acima passaria com a lei apagada:** com `FULL` dos dois lados as
/// duas chamadas são literalmente a mesma expressão, e a igualdade seria trivialmente verdadeira.
/// *É esta a distância que o dono fotografou.*
#[test]
fn sem_o_recorte_o_assado_sai_deslocado() {
    let cam = camera(Lens::Perspective);
    let sub = [240.0, 120.0, 520.0, 380.0];
    let p = Vec3::new(0.7, 0.4, -0.3);
    let na_vista = pixel(&cam, p, VISTA, ViewRegion::FULL);
    // O que a porta de assar fazia até 21/09: escrever o alvo inteiro, ou seja tratar o
    // sub-rectângulo como se fosse a vista.
    let sem_recorte = pixel(&cam, p, sub, ViewRegion::FULL);
    let erro = ((na_vista.0 - sem_recorte.0).powi(2) + (na_vista.1 - sem_recorte.1).powi(2)).sqrt();
    assert!(
        erro > 100.0,
        "o defeito nao esta na fixtura: {erro:.1} px entre {na_vista:?} e {sem_recorte:?}"
    );
}

/// ⚠️ **Um sprite que sobra para fora do viewport é um enquadramento LEGÍTIMO** — a lei tem de
/// valer com fracções fora de `[0,1]`, senão assar com a peça meio fora da tela assaria outra peça.
#[test]
fn um_recorte_que_sai_da_vista_ainda_obedece() {
    let cam = camera(Lens::Perspective);
    let sub = [-180.0, -90.0, 700.0, 620.0];
    let region = ViewRegion::of_px(sub, VISTA).expect("o recorte");
    assert!(region.origin[0] < 0.0 && region.origin[1] < 0.0);
    let p = Vec3::new(0.1, 0.2, 0.0);
    let na_vista = pixel(&cam, p, VISTA, ViewRegion::FULL);
    let no_recorte = pixel(&cam, p, sub, region);
    assert!(
        (na_vista.0 - no_recorte.0).abs() < 1e-2 && (na_vista.1 - no_recorte.1).abs() < 1e-2,
        "{na_vista:?} contra {no_recorte:?}"
    );
}
