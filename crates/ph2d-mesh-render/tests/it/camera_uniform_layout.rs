//! **O gate que o doc do `camera_uniform_bytes` prometia e que não existia.**
//!
//! ⚠️ As duas funções (`camera_uniform_bytes` e `view_proj_from_bytes`) são
//! `pub`, e o doc de uma delas afirma: *"existe para o gate provar que a
//! coluna-major do `glam` é a coluna-major que o WGSL espera — trocar isso
//! transpõe a cena e nada avisa"*. **Esse gate não existia**, e um `pub fn` sem
//! chamador não é código morto silencioso: é uma segunda resposta esperando
//! alguém chamá-la, com um doc-comment mentindo por cima sobre quem já a chama.
//!
//! Ele não pede device: o layout que o shader lê é um fato sobre os BYTES.

use ph2d_mesh_render::{Camera3d, camera_uniform_bytes, view_proj_from_bytes};

/// Uma câmera com os três eixos DIFERENTES entre si.
///
/// ⚠️ Uma câmera olhando de frente para a origem tem matriz quase simétrica, e
/// uma transposição nela é **indistinguível** da original em metade das
/// entradas — que é exatamente a forma de este gate passar sobre o defeito que
/// ele existe para pegar.
fn skewed() -> Camera3d {
    let mut cam = Camera3d::framing(
        ph2d_mesh::Aabb {
            min: [-1.0, -2.0, -3.0],
            max: [1.5, 0.5, 4.0],
        },
        0.8,
        1.7,
    );
    cam.orbit(0.7, -0.35);
    cam.pan(0.2, 0.1);
    cam
}

#[test]
fn the_bytes_the_shader_reads_are_the_matrix_glam_built() {
    let cam = skewed();
    let aspect = 1.7;
    let bytes = camera_uniform_bytes(&cam, aspect, (1920, 1080));
    let back = view_proj_from_bytes(&bytes);
    let want = cam.view_proj(aspect);

    for c in 0..4 {
        for r in 0..4 {
            let (a, b) = (back.col(c)[r], want.col(c)[r]);
            assert!(
                (a - b).abs() < 1e-6,
                "coluna {c} linha {r}: os bytes dizem {a}, o glam diz {b}"
            );
        }
    }
}

/// ⚠️ **A metade que dá dentes ao gate acima.** Sem ela, uma leitura
/// transposta ainda passaria em toda entrada da diagonal e em toda matriz
/// simétrica — o gate ficaria verde sobre a cena virada do avesso.
#[test]
fn a_transposed_reading_would_be_caught() {
    let cam = skewed();
    let want = cam.view_proj(1.7);
    let t = want.transpose();
    let mut worst = 0.0f32;
    for c in 0..4 {
        for r in 0..4 {
            worst = worst.max((want.col(c)[r] - t.col(c)[r]).abs());
        }
    }
    assert!(
        worst > 1e-3,
        "esta câmera é simétrica demais ({worst}): o gate irmão não teria como ver uma transposição"
    );
}

/// Os primeiros 128 bytes são **duas** matrizes, e o `view_proj` é a PRIMEIRA. Trocar a
/// ordem dos dois campos deixaria a segunda metade dos bytes intacta e a
/// primeira apontando para a matriz errada — a cena continuaria desenhando,
/// iluminada por uma vista que não é a dela.
#[test]
fn the_view_proj_comes_first_and_the_view_second() {
    let cam = skewed();
    let bytes = camera_uniform_bytes(&cam, 1.7, (1920, 1080));
    assert_eq!(bytes.len(), 144, "duas mat4 de 64 B + o viewport");

    let view = view_proj_from_bytes(&bytes[64..128]);
    let want = cam.view();
    for c in 0..4 {
        for r in 0..4 {
            assert!(
                (view.col(c)[r] - want.col(c)[r]).abs() < 1e-6,
                "a segunda metade tem de ser a matriz de VISTA"
            );
        }
    }
}
