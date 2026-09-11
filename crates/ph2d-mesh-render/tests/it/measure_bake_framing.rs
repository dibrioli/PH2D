//! **A projeção que o artista VÊ contra a projeção que o BAKE grava.**
//!
//! Report do Enio no smoke da W8.6 (`docs/3D/02.2`): *"o modelo vivo parece em perspectiva, o
//! modelo assado parece isométrico"*. O bake usa a MESMA `Camera3d` do escultor, mas rasteriza no
//! tamanho do SPRITE (quadrado) enquanto o viewport é largo — e `render_gbuffer` deriva o aspecto
//! de `size`, preservando o `fov_y`.
//!
//! ⚠️ **Isto é uma SONDA, não um gate.** Ela não afirma um número: ela imprime a silhueta que cada
//! aspecto produz, para a causa ser atribuída por medição em vez de por leitura de screenshot. O
//! gate nasce depois, quando houver uma lei a pinar.
//!
//! ```text
//! cargo test -p ph2d-mesh-render --release --test measure_bake_framing -- --ignored --nocapture
//! ```

use ph2d_mesh::shapes;
use ph2d_mesh_render::{Camera3d, MeshRenderer};

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("ph2d-mesh bake framing"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");
    Some((device, queue))
}

/// A silhueta que a forma cobre, em pixels: `(x0, y0, x1, y1)` do conjunto com peso > 0.
fn silhouette(plane: &[f32], size: (u32, u32)) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = size;
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) as usize) * 4;
            if plane[i + 3] > 0.0 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    (x0 != u32::MAX).then_some((x0, y0, x1, y1))
}

/// **O que cada aspecto faz com a MESMA câmera e a MESMA malha.**
///
/// A leitura decisiva não é o tamanho em pixels — é a **fração do quadro** que o modelo ocupa em
/// cada eixo. Se as duas frações forem iguais nos dois aspectos, a projeção não é a causa e o que
/// difere é o modelo de SOMBREAMENTO; se a horizontal divergir, o bake enquadra o que o artista não
/// vê.
#[test]
#[ignore = "precisa de adapter"]
fn what_each_aspect_does_to_the_same_camera() {
    let Some((device, queue)) = device() else {
        eprintln!("sem adapter: pulando");
        return;
    };
    let mesh = shapes::uv_sphere(48, 72, 1.0);
    let mut renderer = MeshRenderer::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    renderer.upload_at(&device, &queue, 0, &mesh, &[]);

    // A câmera do escultor: enquadrada no VIEWPORT largo, que é o que o `Sculpt3dScene::new` faz.
    let wide = (1600u32, 900u32);
    let aspect_wide = wide.0 as f32 / wide.1 as f32;
    let camera = Camera3d::framing(mesh.bounds(), core::f32::consts::FRAC_PI_4, aspect_wide);

    println!(
        "camera: distancia {:.3} fov_y {:.1}deg (enquadrada em {:.3})",
        camera.distance,
        camera.fov_y.to_degrees(),
        aspect_wide
    );
    for (label, size) in [
        ("viewport LARGO (o que o artista ve)", wide),
        ("sprite QUADRADO (o que o bake grava)", (1024u32, 1024u32)),
    ] {
        let Some(plane) = renderer.form_plane(
            &device,
            &queue,
            &camera,
            size,
            ph2d_mesh_render::Shade::default(),
            None,
        ) else {
            panic!("form_plane devolveu None");
        };
        let Some((x0, y0, x1, y1)) = silhouette(&plane.normal, size) else {
            panic!("silhueta vazia — a malha nao caiu no quadro");
        };
        let (sw, sh) = ((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32);
        println!(
            "{label}: {}x{} | silhueta {:.0}x{:.0} px | fracao do quadro {:.3} x {:.3} | \
             redondeza {:.3} | centro ({:.3}, {:.3})",
            size.0,
            size.1,
            sw,
            sh,
            sw / size.0 as f32,
            sh / size.1 as f32,
            sw / sh,
            (f32::from(u16::try_from(x0 + x1).unwrap_or(0)) * 0.5) / size.0 as f32,
            (f32::from(u16::try_from(y0 + y1).unwrap_or(0)) * 0.5) / size.1 as f32,
        );
    }
}
