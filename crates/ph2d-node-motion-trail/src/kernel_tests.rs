//! Os gates do rastro no dispositivo que NÃO precisam de adaptador — as derivações são as leis do
//! `step`, a escolha da variante é a da CPU, e os literais do WGSL são as consts dela.

use super::*;
use crate::carry::default_for;
use crate::{ECHO_BLEND_LABELS, MAX_SPACING};
use ph2d_nodegraph::attr::Column;

/// Deriva `nome` com estes params e esta contagem viva.
fn deriva(nome: &str, params: &[(&str, f32)], vivos: u32) -> f32 {
    let d = DERIVADOS
        .iter()
        .find(|d| d.param == nome)
        .unwrap_or_else(|| panic!("`{nome}` não é derivado"));
    let param = |n: &str| {
        params
            .iter()
            .find(|(k, _)| *k == n)
            .map_or_else(|| MANIFEST.param_default(n).unwrap_or(0.0), |(_, v)| *v)
    };
    (d.derive)(&CountLawCtx {
        inputs: &[vivos, 7],
        param: &param,
        playhead: 0.0,
        dt: 0.0,
    })
}

/// ⭐ **As taxas e a matriz são as do `step`**, bit a bit, sobre a grelha de casos que as move:
/// o comprimento (e o tecto de instâncias, que o encurta pela contagem viva), o espaçamento, e os
/// alvos — incluindo os neutros e o lixo que a CPU engole.
#[test]
fn the_derived_rates_are_the_steps() {
    let mut casos = 0;
    for length in [8.0, 1.0, 32.0, 50.0, 2.6] {
        for spacing in [1.0, 3.0, 0.2, 40.0] {
            for vivos in [64u32, 1, 262_144, 100_000] {
                let params = [
                    ("length", length),
                    ("spacing", spacing),
                    ("fade", 0.37),
                    ("shrink", 1.4),
                    ("hue_shift", 72.0),
                    ("saturation", 0.2),
                    ("spin", -33.0),
                    (ALPHA_MAX, 0.6),
                ];
                let k = generations(length, vivos as usize);
                let janela = (k - 1) * spacing_of(spacing) + 1;
                assert_eq!(
                    deriva("tr_window", &params, vivos),
                    janela as f32,
                    "janela {length}/{spacing}/{vivos}"
                );
                let d = Decay {
                    alpha_max: 0.6,
                    fade: 0.37,
                    shrink: 1.4,
                    hue_shift: 72.0,
                    saturation: 0.2,
                    spin: -33.0,
                }
                .per_tick((janela - 1) as u32);
                for (nome, v) in [("fade", d.fade), ("shrink", d.shrink), ("spin", d.spin)] {
                    assert_eq!(
                        deriva(nome, &params, vivos).to_bits(),
                        v.to_bits(),
                        "{nome}"
                    );
                }
                let m = colour::compose(
                    colour::hue_rotation(d.hue_shift),
                    colour::saturation(d.saturation),
                );
                for (r, linha) in m.iter().enumerate() {
                    for (c, v) in linha.iter().enumerate() {
                        let nome = format!("tr_m{r}{c}");
                        assert_eq!(
                            deriva(&nome, &params, vivos).to_bits(),
                            v.to_bits(),
                            "{nome}"
                        );
                    }
                }
                assert_eq!(
                    deriva("tr_colour", &params, vivos),
                    f32::from(u8::from(m != colour::IDENTITY))
                );
                assert_eq!(deriva("tr_live", &params, vivos), vivos as f32);
                assert_eq!(identidade_de(&params, vivos), k <= 1, "identidade");
                casos += 1;
            }
        }
    }
    assert_eq!(casos, 5 * 4 * 4, "a grelha correu inteira");
    // ⚠️ E no NEUTRO a matriz é a identidade — o corpo não multiplica nada.
    assert_eq!(deriva("tr_colour", &[], 64), 0.0);
}

fn identidade_de(params: &[(&str, f32)], vivos: u32) -> bool {
    let param = |n: &str| {
        params
            .iter()
            .find(|(k, _)| *k == n)
            .map_or_else(|| MANIFEST.param_default(n).unwrap_or(0.0), |(_, v)| *v)
    };
    identidade(&CountLawCtx {
        inputs: &[vivos],
        param: &param,
        playhead: 0.0,
        dt: 0.0,
    })
}

/// **A escolha da variante é a lei da CPU**: o `rot` quando há giro, o modo quando há tag — e o
/// caso identidade é um passa-tudo sem tag.
#[test]
fn the_variants_follow_the_cpus_choices() {
    let tem = |k: &GpuKernel, col: &str| k.bindings.iter().any(|b| b.column == col);
    for spin in [0.0, 12.0, f32::NAN, f32::INFINITY] {
        for tag in [0.0, 2.0, 0.4, f32::NAN] {
            let p = |n: &str| match n {
                "spin" => spin,
                ECHO_BLEND => tag,
                _ => 0.0,
            };
            let k = GPU_KERNEL.resolve(&p);
            assert_eq!(
                tem(k, "rot"),
                spin.is_finite() && spin != 0.0,
                "spin {spin}"
            );
            assert_eq!(
                tem(k, BLEND_COLUMN),
                echo_blend_tag(tag).is_some(),
                "tag {tag}"
            );
            let id = IDENTIDADE_KERNEL.resolve(&p);
            assert_eq!(
                id.is_passthrough(),
                echo_blend_tag(tag).is_none(),
                "tag {tag}"
            );
        }
    }
    // O `Resampled` fica na CPU.
    let applicable = GPU_KERNEL.applicable.expect("declarado");
    assert!(applicable(&|n| if n == SOURCE { 0.0 } else { 1.0 }));
    assert!(!applicable(&|n| if n == SOURCE { 1.0 } else { 0.0 }));
}

/// **Os literais do WGSL são as consts da CPU** — e estão no texto compilado.
#[test]
fn the_kernels_literals_are_the_cpus_constants() {
    assert_eq!((ECHO_BLEND_LABELS.len() - 1) as f32, 6.0);
    assert!(COM_MODO.wgsl.contains("1.0, 6.0)"));
    assert!(SO_MODO.wgsl.contains("1.0, 6.0)"));
    assert_eq!(MAX_SPACING as f32, 16.0);
    assert!(PARTILHADO.contains("16.0)"));
}

/// **As identidades da junção são o `default_for` da CPU**, coluna a coluna e dimensão a
/// dimensão.
#[test]
fn the_join_identities_are_the_cpus() {
    let protos = [
        (Dim::Scalar, Column::Scalar(vec![0.0])),
        (Dim::Vec2, Column::Vec2(vec![[0.0; 2]])),
        (Dim::Vec3, Column::Vec3(vec![[0.0; 3]])),
        (Dim::Vec4, Column::Vec4(vec![[0.0; 4]])),
    ];
    for nome in ["size", "tint", "uv_rect", "id", "falloff", AGE] {
        for (dim, proto) in &protos {
            let cpu: Vec<f32> = match default_for(nome, proto) {
                Column::Scalar(v) => v,
                Column::Vec2(v) => v.concat(),
                Column::Vec3(v) => v.concat(),
                Column::Vec4(v) => v.concat(),
            };
            let disp = ConcatFill::of(IDENTIDADES, nome, *dim);
            assert_eq!(&disp[..cpu.len()], &cpu[..], "{nome} {dim:?}");
        }
    }
}
