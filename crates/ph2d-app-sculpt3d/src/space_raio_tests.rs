//! ⭐ **O TECTO DO RAIO NA CENA** — a porta `radius_px` clampa o raio autorado contra a DIAGONAL da
//! vista (report de 2026-09-16: *«O radius máximo permitido é pouco»*).
//!
//! Irmão com GPU do `space_tests`, que é puro de propósito: a lei do tecto é a função livre
//! `radius_ceiling_px` (com gate nas `rulers`), e o que só uma cena pode provar é o ELO — a porta
//! que o dab, o anel e o painel leem usa essa lei e não outra.

use crate::Sculpt3dScene;

/// Abre a GPU, ou diz que não há nada a afirmar (cópia local, como nos irmãos).
macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

#[test]
#[ignore = "precisa de GPU"]
fn o_raio_da_cena_chega_a_diagonal_da_vista() {
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(16, 24, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    let (w, h) = s.viewport();
    s.radius_px = 1.0e6;
    assert_eq!(
        s.radius_px(),
        crate::radius_ceiling_px(w, h),
        "o raio pedido nao para na diagonal da vista"
    );
    #[allow(clippy::cast_precision_loss)]
    let altura = h as f32;
    assert!(
        s.radius_px() > 0.49 * altura,
        "o tecto da cena nao cobre a peca"
    );
    s.radius_px = 0.0;
    assert_eq!(s.radius_px(), crate::RADIUS_MIN_PX, "o piso do raio mudou");
}

/// ⭐ **O pincel que a app ARMA vem de um traço arrastado** — é ele que liga a atenuação do pincel
/// de plano (espec §14.4). ⛔ Sem isto cada dab de um traço denso valeria `1/0,57 = 1,75×` o que
/// devia, e a bancada — que liga o interruptor à mão — não o veria.
#[test]
#[ignore = "precisa de GPU"]
fn o_pincel_armado_vem_de_um_traco_arrastado() {
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(&gpu.device, ph2d_mesh::shapes::uv_sphere(16, 24, 1.0), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = ph2d_sculpt3d::Verb::Plane;
    let armado = s.armed_brush([0.0, 0.0, 1.0]);
    assert!(armado.traco_arrastado, "o pincel armado perdeu o arrasto");
    assert!(
        (armado.factor_do_traco() - 1.0).abs() > 0.1,
        "o pincel de plano armado nao atenua"
    );
}
