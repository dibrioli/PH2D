//! ⭐⭐⭐ **§5.0 DO CATAVENTO, BLOCO D — o preço de ACENDER, por quadro.**
//!
//! ⛔⛔ **Ele existe porque a 1.ª §5.0 desta obra media METADE DA CORRENTE.** Os blocos A–C vivem na
//! [`ph2d-mesh-render`] e respondem *«quanto custa RASTERIZAR a malha filha»* — e a corrente do
//! produto é **rasterizar E ACENDER**. *Uma régua que mede o primeiro elo e divide o orçamento por
//! ele devolve uma contagem de objectos que o app nunca vai ver*, e é a família que esta casa já
//! pagou meia dúzia de vezes.
//!
//! A tabela dos outros três blocos e o mecanismo inteiro vivem em
//! `docs/Render3d/17_a_rota_b_o_catavento.md`.
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-form-donation --release \
//!     mede_o_acender -- --ignored --nocapture --test-threads=1
//! ```
//!
//! ⚠️ **`--release`, `--test-threads=1` e o `loadavg` impresso ao lado** — as três leis que os
//! blocos A–C tiveram de aprender à segunda redacção.

use super::*;
use crate::lei_da_luz::Lei;
use ph2d_render::TextureAtlas;

fn placa() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static PARTILHADA: OnceLock<Option<GpuContext>> = OnceLock::new();
    PARTILHADA
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

/// Uma bola sintética do lado pedido — o que esta sonda mede é **por PIXEL**, logo a geometria só
/// precisa de encher a silhueta e de dar normais que variem.
fn bola(lado: u32) -> BakedForm {
    let n = (lado * lado) as usize;
    let (mut base, mut form, occ) = (vec![0u8; n * 4], vec![0f32; n * 4], vec![1f32; n]);
    let meio = f64::from(lado) * 0.5;
    let r = meio * 0.9;
    for y in 0..lado {
        for x in 0..lado {
            let i = (y * lado + x) as usize;
            let (dx, dy) = (f64::from(x) - meio, f64::from(y) - meio);
            let d2 = dx * dx + dy * dy;
            if d2 <= r * r {
                let nz = (1.0 - d2 / (r * r)).sqrt();
                form[i * 4] = (dx / r) as f32;
                form[i * 4 + 1] = (dy / r) as f32;
                form[i * 4 + 2] = nz as f32;
                form[i * 4 + 3] = 1.0;
                base[i * 4..i * 4 + 4].copy_from_slice(&[200, 120, 90, 255]);
            } else {
                form[i * 4 + 2] = 1.0;
            }
        }
    }
    BakedForm {
        size: (lado, lado),
        base,
        form,
        form_occ: occ,
        texture_id: 0,
        rig: LightRig::default(),
        lit_with: None,
        lei: Lei::Forma,
    }
}

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// ⭐⭐⭐ **BLOCO D — quanto custa ACENDER um sprite com forma, por quadro, nas DUAS leis.**
///
/// ⚠️ **As duas colunas não são curiosidade:** a `Tinta` é a lei que o app tinha até 21/09 e a
/// `Forma` é a que ele ship desde então — *se a rota B só couber no orçamento com a lei antiga, isso
/// é um facto sobre a obra e tem de estar escrito antes de ela começar*.
#[test]
#[ignore = "precisa de adapter"]
fn mede_o_acender_por_quadro_nas_duas_leis() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    eprintln!(
        "\n=== BLOCO D — acender um sprite com forma, por quadro (load {}) ===",
        carga()
    );
    eprintln!(
        "{:>6} | {:>13} | {:>13} | {:>12}",
        "lado", "Tinta", "Forma", "objs/quadro"
    );
    for lado in [128u32, 256, 512, 1024] {
        let bake_base = bola(lado);
        let atlas = TextureAtlas::new(&gpu, lado.max(256));
        let mut renderer =
            SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
        let mut tempos = [0f64; 2];
        for (k, lei) in [Lei::Tinta, Lei::Forma].into_iter().enumerate() {
            let mut b = BakedForm {
                texture_id: renderer
                    .acquire_individual(lado, lado, &bake_base.base)
                    .expect("slot"),
                base: bake_base.base.clone(),
                form: bake_base.form.clone(),
                form_occ: bake_base.form_occ.clone(),
                ..bake_base
            };
            let mut passes = PassesDaLuz::default();
            // aquecimento: a 1.ª acendida constrói o pipeline e as texturas.
            acende_com(lei, &gpu, &mut renderer, &mut passes, &b.rig, &b).expect("acende");
            tempos[k] = mediana(
                (0..7)
                    .map(|_| {
                        // ⚠️ **O carimbo é limpo a cada volta**: o produto salta a acendida quando
                        // o rig não mudou, e *uma sonda que medisse o salto media o memo e não a lei*.
                        b.lit_with = None;
                        let t = std::time::Instant::now();
                        acende_com(lei, &gpu, &mut renderer, &mut passes, &b.rig, &b)
                            .expect("acende");
                        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
                        t.elapsed().as_secs_f64() * 1e3
                    })
                    .collect(),
            );
        }
        eprintln!(
            "{lado:>6} | {:>10.3} ms | {:>10.3} ms | {:>12.0}",
            tempos[0],
            tempos[1],
            16.67 / tempos[1]
        );
    }
}
