//! **O QUE A FORMA CUSTA GUARDADA** — a medição que decide como o canal viaja no disco.
//!
//! Filho de [`super`], que possui o harness (o adapter, a cena, o readback). O corte é de ASSUNTO: o
//! pai pergunta *as duas luzes concordam sobre a mesma forma?*; aqui a pergunta é *a luz sobrevive à
//! forma guardada como IMAGEM?* — e ela só tem resposta honesta assando as duas e comparando bytes.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --release --bins sculpt3d::bake::light::form_bytes -- --ignored --nocapture
//! ```

use super::*;

/// A forma como um sprite a guardaria: normal em `RGB8` (`n·0,5 + 0,5`) e peso no alpha.
///
/// ⚠️ **Isto NÃO é a representação escolhida — é a que a indústria usa** (Unity *Secondary
/// Textures*, Godot `CanvasTexture::normal_texture`, todo mapa de normal assado do Blender), e a
/// pergunta que ela responde é se a LUZ sobrevive à viagem. O `renormalise` no fim é o que o
/// consumidor faria de qualquer jeito: `n·0,5 + 0,5` quantizado não devolve um vetor unitário.
fn quantise_form(form: &[f32]) -> Vec<f32> {
    let mut out = vec![0f32; form.len()];
    for (o, f) in out.chunks_exact_mut(4).zip(form.chunks_exact(4)) {
        let enc = |v: f32| ((v * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0 + 0.5).floor() / 255.0;
        let (x, y, z) = (
            enc(f[0]) * 2.0 - 1.0,
            enc(f[1]) * 2.0 - 1.0,
            enc(f[2]) * 2.0 - 1.0,
        );
        let len = (x * x + y * y + z * z).sqrt().max(1e-6);
        o[0] = x / len;
        o[1] = y / len;
        o[2] = z / len;
        o[3] = ((f[3].clamp(0.0, 1.0) * 255.0 + 0.5).floor()) / 255.0;
    }
    out
}

/// Assa uma vez com a forma dada e devolve os bytes.
fn bake_with(
    gpu: &GpuContext,
    rig: &ph2d_light::ResolvedRig,
    form: &[f32],
    base_rgb: [u8; 3],
) -> Vec<u8> {
    let n = (SIDE * SIDE) as usize;
    let mut base = vec![0u8; n * 4];
    for px in base.chunks_exact_mut(4) {
        px.copy_from_slice(&[base_rgb[0], base_rgb[1], base_rgb[2], 255]);
    }
    let (relief, cover, mat0, mat1) = neutral_planes(&base);
    let planes = BakePlanes {
        relief,
        cover,
        mat0,
        mat1,
        lamps: resolved_lamps(rig),
    };
    let src = upload_rgba(gpu, (SIDE, SIDE), &base);
    let mut pass = ImpastoLightPass::new(gpu);
    // ⚠️ Sem oclusão, e de propósito: esta sonda mede a QUANTIZAÇÃO DA NORMAL. Misturar o outro
    // plano faria os bytes divergentes terem duas causas e a tabela deixaria de responder à pergunta
    // que ela existe para fazer. (A do plano de oclusão não precisa de sonda: ela multiplica o difuso
    // direto, então um erro de `1/255` nele move o pixel em no máximo **um** nível, por aritmética —
    // `|Δ(albedo·mul·occ)| ≤ Δocc`.)
    let input = build_input((SIDE, SIDE), &planes, form, &[], SpecLut::get());
    let out = pass.run(gpu, &src, &input).expect("o passe aceitou");
    readback(gpu, out, SIDE, SIDE, 4)
}

/// **A FORMA SOBREVIVE GUARDADA COMO IMAGEM?** — o número que decide como o canal viaja no disco.
///
/// A forma é hoje `[f32; 4]` por texel: a 1024² são **16 MiB por sprite**, contra **4 MiB** em
/// `RGBA8`. A pergunta não é o tamanho (é aritmética), é se a luz que o artista aprovou muda ao
/// passar por 8 bits — e ela só tem uma resposta honesta: **assar as duas e comparar os bytes**.
#[test]
#[ignore = "precisa de adapter"]
fn measure_the_form_quantised_to_eight_bits() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let resolved = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");
    let form = renderer
        .form_plane(
            &gpu.device,
            &gpu.queue,
            &camera,
            (SIDE, SIDE),
            ph2d_mesh_render::Shade::default(),
            None,
        )
        .expect("a malha esta la'")
        .normal;
    let quantised = quantise_form(&form);

    let clay = [
        (CLAY[0] * 255.0 + 0.5) as u8,
        (CLAY[1] * 255.0 + 0.5) as u8,
        (CLAY[2] * 255.0 + 0.5) as u8,
    ];
    println!("\n=== A FORMA GUARDADA COMO IMAGEM (RGB8 + peso no alpha) ===");
    println!("  albedo | texels dentro |  bytes diferentes |  pior delta | delta medio");
    for (name, rgb) in [
        ("barro", clay),
        ("branco", [255u8; 3]),
        ("meio", [128u8; 3]),
    ] {
        let a = bake_with(&gpu, &resolved, &form, rgb);
        let b = bake_with(&gpu, &resolved, &quantised, rgb);
        let (mut inside, mut differing, mut worst, mut sum) = (0u64, 0u64, 0u8, 0u64);
        for i in 0..(SIDE * SIDE) as usize {
            if form[i * 4 + 3] <= 0.0 {
                continue;
            }
            inside += 1;
            let mut any = false;
            for k in 0..3 {
                let d = a[i * 4 + k].abs_diff(b[i * 4 + k]);
                if d > 0 {
                    any = true;
                }
                worst = worst.max(d);
                sum += u64::from(d);
            }
            if any {
                differing += 1;
            }
        }
        let mean = sum as f64 / (inside.max(1) * 3) as f64;
        println!("  {name:>6} | {inside:>13} | {differing:>17} | {worst:>11} | {mean:>11.4}");
    }
    let px = u64::from(SIDE) * u64::from(SIDE);
    println!(
        "  tamanho por sprite {SIDE}x{SIDE}: forma f32 {:.2} MiB · forma RGBA8 {:.2} MiB · base RGBA8 {:.2} MiB",
        (px * 16) as f64 / (1024.0 * 1024.0),
        (px * 4) as f64 / (1024.0 * 1024.0),
        (px * 4) as f64 / (1024.0 * 1024.0)
    );
    println!("  (a 1024x1024 isso e' 16,00 / 4,00 / 4,00 MiB -- o documento de UM objeto assado)");
}
