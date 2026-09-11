//! **A RÉGUA DA VISTA DEPENDE DE ONDE SE PERGUNTA** — a medição que matou a
//! âncora do estêncil do alpha (Enio, 2026-08-09: *"a tinta da máscara projetada
//! no objeto não corresponde ao que realmente está sendo esculpido"*).
//!
//! A primeira versão do estêncil media o tamanho de um ladrilho em *unidades de
//! objeto por altura de tela* — um número que sai de
//! [`Camera3d::world_radius_for_screen_px`] e é **função da PROFUNDIDADE**: numa
//! câmera em perspectiva o mesmo pixel cobre mais mundo quanto mais longe. Quem
//! montava a régua tinha de escolher ONDE perguntar, e os dois consumidores
//! escolheram pontos diferentes: o dab no ACERTO, o preview no CENTRO da peça.
//!
//! ⚠️ **Esta sonda continua válida depois da cura, e é por isso que ela fica:**
//! ela não mede o estêncil — mede a CÂMERA, e o que ela reporta é permanente.
//! Ela é a evidência de que nenhuma âncora podia estar certa, e o motivo de o
//! `AlphaStencil` guardar hoje o frustum inteiro (olho + razão) em vez de uma
//! régua num ponto.
//!
//! Rodar: `cargo test -p ph2d-mesh-render --test measure_the_view_ruler_at_two_anchors -- --ignored --nocapture`

use ph2d_mesh_render::Camera3d;

const SIZE: (u32, u32) = (1920, 1080);

/// A cena do smoke: a mesma esfera e o mesmo enquadramento que o artista vê.
fn scene() -> (ph2d_mesh::Mesh, Camera3d) {
    let mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let aspect = SIZE.0 as f32 / SIZE.1 as f32;
    let mut cam = Camera3d {
        yaw: 0.6,
        pitch: 0.35,
        ..Camera3d::default()
    };
    cam.frame(mesh.bounds(), aspect);
    (mesh, cam)
}

/// A régua da vista: quantas unidades de objeto a ALTURA da tela abrange,
/// perguntada em `at`.
fn ruler(cam: &Camera3d, at: [f32; 3]) -> f32 {
    cam.world_radius_for_screen_px(at, SIZE.1 as f32, SIZE)
}

#[test]
#[ignore = "sonda de medição"]
fn measure_the_view_ruler_at_two_anchors() {
    let (mesh, cam) = scene();
    let centre = mesh.bounds().center();
    // O ponto do acerto: a superfície da esfera virada para a câmera.
    let eye = cam.eye();
    let dir = (eye - glam::Vec3::from(centre)).normalize();
    let front: [f32; 3] = (glam::Vec3::from(centre) + dir).into();
    // E o ponto de TRÁS, para dar o outro extremo do que um traço alcança.
    let back: [f32; 3] = (glam::Vec3::from(centre) - dir).into();

    let (rc, rf, rb) = (ruler(&cam, centre), ruler(&cam, front), ruler(&cam, back));

    println!("distancia da camera ao centro: {:.4}", cam.distance);
    println!("regua no CENTRO (o preview):   {rc:.4} unidades de objeto");
    println!("regua na FRENTE  (o dab):      {rf:.4}");
    println!("regua ATRAS:                   {rb:.4}");
    println!("razao preview/dab (frente):    {:.4}x", rc / rf);
    println!("razao preview/dab (atras):     {:.4}x", rc / rb);
    println!(
        "erro que uma ANCORA no centro daria: {:+.1}% na frente, {:+.1}% atras",
        (rc / rf - 1.0) * 100.0,
        (rc / rb - 1.0) * 100.0
    );
    println!(
        "razao frente/atras: {:.4}x — nenhuma escolha de ponto serve para os dois",
        rf / rb
    );
}
