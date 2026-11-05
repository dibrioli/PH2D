//! A mesma receita de 8 camadas em duas variantes: kernels próprios sobre `ph2d-det-math` e FunDSP.
//! Só a sonda usa isto; o resultado e a decisão estão em `docs/SfxLab/02_W0_medicao.md`.

pub const SR: f32 = 48_000.0;
const LN_MILESIMO: f32 = -6.907_755;

/// FNV-1a sobre os bits de cada amostra: um bit diferente muda o resultado.
#[must_use]
pub fn hash(pcm: &[f32]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325_u64;
    for s in pcm {
        for b in s.to_bits().to_le_bytes() {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

#[derive(Clone, Copy, Debug)]
pub enum Camada {
    Modal {
        modos: u32,
        base: f32,
        t60: f32,
    },
    Ruido {
        hz: f32,
        q: f32,
        t60: f32,
    },
    Tom {
        hz: f32,
        queda: f32,
        t60: f32,
    },
    Fm {
        hz: f32,
        razao: f32,
        indice: f32,
        t60: f32,
    },
}

#[derive(Clone, Debug)]
pub struct Receita {
    pub semente: u64,
    pub frames: usize,
    /// (id estável, camada, ganho)
    pub camadas: Vec<(u64, Camada, f32)>,
}

/// O caso típico, com o fenômeno do plano: ressonância longa, ruído semeado, tons e FM.
#[must_use]
pub fn receita_tipica(frames: usize) -> Receita {
    Receita {
        semente: 0x5F3,
        frames,
        camadas: vec![
            (
                1,
                Camada::Modal {
                    modos: 32,
                    base: 180.0,
                    t60: 4.0,
                },
                0.2,
            ),
            (
                2,
                Camada::Modal {
                    modos: 32,
                    base: 730.0,
                    t60: 1.0,
                },
                0.15,
            ),
            (
                3,
                Camada::Ruido {
                    hz: 2_400.0,
                    q: 1.2,
                    t60: 0.5,
                },
                0.15,
            ),
            (
                4,
                Camada::Ruido {
                    hz: 9_000.0,
                    q: 0.7,
                    t60: 0.1,
                },
                0.1,
            ),
            (
                5,
                Camada::Tom {
                    hz: 1_760.0,
                    queda: 3.0,
                    t60: 0.4,
                },
                0.1,
            ),
            (
                6,
                Camada::Tom {
                    hz: 110.0,
                    queda: 0.5,
                    t60: 1.5,
                },
                0.1,
            ),
            (
                7,
                Camada::Fm {
                    hz: 440.0,
                    razao: 1.41,
                    indice: 3.0,
                    t60: 0.8,
                },
                0.1,
            ),
            (
                8,
                Camada::Fm {
                    hz: 66.0,
                    razao: 2.0,
                    indice: 8.0,
                    t60: 2.0,
                },
                0.1,
            ),
        ],
    }
}

/// O pior caso que o plano admite hoje: 8 camadas modais de 128 modos.
#[must_use]
pub fn receita_pior(frames: usize) -> Receita {
    let camadas = (0..8u32)
        .map(|i| {
            let base = 90.0 + 70.0 * i as f32;
            (
                u64::from(i) + 1,
                Camada::Modal {
                    modos: 128,
                    base,
                    t60: 4.0,
                },
                0.05,
            )
        })
        .collect();
    Receita {
        semente: 0x5F3,
        frames,
        camadas,
    }
}

pub mod kernels {
    use super::{Camada, LN_MILESIMO, Receita, SR};
    use ph2d_det_math::DetMath;

    const TAU: f32 = core::f32::consts::TAU;

    struct Rng(u64);
    impl Rng {
        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn bipolar(&mut self) -> f32 {
            const ESCALA: f32 = 1.0 / (1u32 << 24) as f32;
            ((self.next_u64() >> 40) as u32) as f32 * ESCALA * 2.0 - 1.0
        }
    }

    /// A semente da camada depende da receita e do id da própria camada: nunca da posição dela.
    fn semente_da_camada(receita: u64, id: u64) -> u64 {
        let mut r = Rng(receita ^ id.wrapping_mul(0xD6E8_FEB8_6659_FD93));
        r.next_u64()
    }

    fn queda_por_amostra(t60: f32) -> f32 {
        (LN_MILESIMO / (t60 * SR)).det_exp()
    }

    fn modal(mix: &mut [f32], ganho: f32, semente: u64, modos: u32, base: f32, t60: f32) {
        // Excitador de 2 ms, o mesmo para todos os modos.
        let mut rng = Rng(semente);
        let golpe: Vec<f32> = (0..96).map(|_| rng.bipolar()).collect();
        for k in 0..modos {
            let parcial = (k + 1) as f32;
            let f = base * parcial.det_powf(1.3);
            if f >= SR * 0.45 {
                break;
            }
            let t = t60 / (1.0 + 0.15 * k as f32);
            let raio = queda_por_amostra(t);
            let (a1, a2) = (2.0 * raio * (TAU * f / SR).det_cos(), -raio * raio);
            let amp = ganho / parcial;
            let (mut y1, mut y2) = (0.0f32, 0.0f32);
            for (i, m) in mix.iter_mut().enumerate() {
                let x = golpe.get(i).copied().unwrap_or(0.0);
                let y = a1 * y1 + a2 * y2 + x * 0.05;
                y2 = y1;
                y1 = y;
                *m += y * amp;
            }
        }
    }

    fn ruido(mix: &mut [f32], ganho: f32, semente: u64, hz: f32, q: f32, t60: f32) {
        let mut rng = Rng(semente);
        // SVF de Chamberlin/TPT com o coeficiente calculado uma vez.
        let g = (core::f32::consts::PI * hz / SR).det_tan();
        let k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let (mut ic1, mut ic2) = (0.0f32, 0.0f32);
        let queda = queda_por_amostra(t60);
        let mut env = ganho;
        for m in mix.iter_mut() {
            let x = rng.bipolar();
            let v3 = x - ic2;
            let v1 = a1 * (ic1 + g * v3);
            let v2 = ic2 + g * v1;
            ic1 = 2.0 * v1 - ic1;
            ic2 = 2.0 * v2 - ic2;
            *m += v1 * env;
            env *= queda;
        }
    }

    fn poly_blep(t: f32, dt: f32) -> f32 {
        if t < dt {
            let x = t / dt;
            x + x - x * x - 1.0
        } else if t > 1.0 - dt {
            let x = (t - 1.0) / dt;
            x * x + x + x + 1.0
        } else {
            0.0
        }
    }

    fn tom(mix: &mut [f32], ganho: f32, hz: f32, queda_t: f32, t60: f32) {
        let mut inc = hz / SR;
        let queda_f = (-queda_t / SR).det_exp();
        let queda = queda_por_amostra(t60);
        let (mut fase, mut env) = (0.0f32, ganho);
        for m in mix.iter_mut() {
            let saw = 2.0 * fase - 1.0 - poly_blep(fase, inc);
            *m += saw * env;
            fase += inc;
            if fase >= 1.0 {
                fase -= 1.0;
            }
            inc *= queda_f;
            env *= queda;
        }
    }

    fn fm(mix: &mut [f32], ganho: f32, hz: f32, razao: f32, indice: f32, t60: f32) {
        let (inc_p, inc_m) = (hz / SR, hz * razao / SR);
        let queda = queda_por_amostra(t60);
        let (mut fp, mut fm, mut env) = (0.0f32, 0.0f32, ganho);
        for m in mix.iter_mut() {
            let moda = (TAU * fm).det_sin() * indice * env.min(1.0);
            *m += (TAU * fp + moda).det_sin() * env;
            fp += inc_p;
            fm += inc_m;
            if fp >= 1.0 {
                fp -= 1.0;
            }
            if fm >= 1.0 {
                fm -= 1.0;
            }
            env *= queda;
        }
    }

    /// `render(receita) -> PCM` mono: o mesmo resultado em qualquer tamanho de bloco, porque não há bloco.
    #[must_use]
    pub fn render(r: &Receita) -> Vec<f32> {
        let mut mix = vec![0.0f32; r.frames];
        for &(id, camada, ganho) in &r.camadas {
            let s = semente_da_camada(r.semente, id);
            match camada {
                Camada::Modal { modos, base, t60 } => modal(&mut mix, ganho, s, modos, base, t60),
                Camada::Ruido { hz, q, t60 } => ruido(&mut mix, ganho, s, hz, q, t60),
                Camada::Tom { hz, queda, t60 } => tom(&mut mix, ganho, hz, queda, t60),
                Camada::Fm {
                    hz,
                    razao,
                    indice,
                    t60,
                } => fm(&mut mix, ganho, hz, razao, indice, t60),
            }
        }
        mix
    }

    /// A mesma pergunta do FunDSP: editar um vizinho muda a camada `alvo`?
    /// O vizinho tem ganho zero, então qualquer diferença no resultado vem da semente.
    #[must_use]
    pub fn vizinho(frames: usize) -> [u64; 4] {
        let alvo = (
            2,
            Camada::Ruido {
                hz: 2_400.0,
                q: 1.2,
                t60: 0.5,
            },
            0.5,
        );
        let viz = |c: Camada| (1, c, 0.0);
        let com = |camadas: Vec<(u64, Camada, f32)>| {
            super::hash(&render(&Receita {
                semente: 9,
                frames,
                camadas,
            }))
        };
        let tom = Camada::Tom {
            hz: 440.0,
            queda: 0.0,
            t60: 1.0,
        };
        let tom2 = Camada::Tom {
            hz: 880.0,
            queda: 0.0,
            t60: 1.0,
        };
        [
            com(vec![viz(tom), alvo]),
            com(vec![viz(tom2), alvo]),
            com(vec![
                viz(tom),
                viz(Camada::Fm {
                    hz: 1.0,
                    razao: 1.0,
                    indice: 1.0,
                    t60: 1.0,
                }),
                alvo,
            ]),
            com(vec![alvo, viz(tom)]),
        ]
    }
}

pub mod fd {
    use super::{Camada, Receita, SR};
    use fundsp::prelude32::*;

    fn camada(id: u64, semente: u64, c: Camada, ganho: f32) -> Net {
        let s = semente ^ id.wrapping_mul(0xD6E8_FEB8_6659_FD93);
        match c {
            Camada::Modal { modos, base, t60 } => {
                let golpe = noise().seed(s) * envelope(|t: f32| if t < 0.002 { 0.05 } else { 0.0 });
                let mut banco: Option<Net> = None;
                for k in 0..modos {
                    let parcial = (k + 1) as f32;
                    let f = base * pow(parcial, 1.3);
                    if f >= SR * 0.45 {
                        break;
                    }
                    let banda = 2.2 * (1.0 + 0.15 * k as f32) / t60;
                    let m = Net::wrap(Box::new(resonator_hz(f, banda) * (ganho / parcial)));
                    banco = Some(match banco {
                        Some(b) => b & m,
                        None => m,
                    });
                }
                Net::wrap(Box::new(golpe)) >> banco.expect("ao menos um modo")
            }
            Camada::Ruido { hz, q, t60 } => {
                let k = 6.907_755 / t60;
                Net::wrap(Box::new(
                    (noise().seed(s) >> bandpass_hz(hz, q))
                        * envelope(move |t: f32| ganho * exp(-t * k)),
                ))
            }
            Camada::Tom { hz, queda, t60 } => {
                let k = 6.907_755 / t60;
                Net::wrap(Box::new(
                    (envelope(move |t: f32| hz * exp(-t * queda)) >> saw().phase(0.0))
                        * envelope(move |t: f32| ganho * exp(-t * k)),
                ))
            }
            Camada::Fm {
                hz,
                razao,
                indice,
                t60,
            } => {
                let k = 6.907_755 / t60;
                let hzm = hz * razao;
                Net::wrap(Box::new(
                    (((sine_hz(hzm).phase(0.0) * (indice * hzm)) + hz) >> sine().phase(0.0))
                        * envelope(move |t: f32| ganho * exp(-t * k)),
                ))
            }
        }
    }

    #[must_use]
    pub fn monta(r: &Receita) -> Net {
        let mut net: Option<Net> = None;
        for &(id, c, g) in &r.camadas {
            let n = camada(id, r.semente, c, g);
            net = Some(match net {
                Some(a) => a + n,
                None => n,
            });
        }
        let mut n = net.expect("ao menos uma camada");
        n.set_sample_rate(f64::from(SR));
        n.allocate();
        n
    }

    /// Uma amostra por vez (código escalar).
    #[must_use]
    pub fn render_tick(r: &Receita) -> Vec<f32> {
        let mut n = monta(r);
        (0..r.frames).map(|_| n.get_mono()).collect()
    }

    /// Blocos de 64 (o caminho SIMD, o que um render offline usaria).
    #[must_use]
    pub fn render_bloco(r: &Receita) -> Vec<f32> {
        let mut n = monta(r);
        let mut out = vec![0.0f32; r.frames];
        let mut buf = BufferArray::<U1>::new();
        let entrada = BufferRef::empty();
        let mut i = 0;
        while i < r.frames {
            let t = Ord::min(r.frames - i, MAX_BUFFER_SIZE);
            n.process(t, &entrada, &mut buf.buffer_mut());
            out[i..i + t].copy_from_slice(&buf.channel_f32(0)[..t]);
            i += t;
        }
        out
    }

    fn hash_net(mut n: Net, frames: usize) -> u64 {
        n.set_sample_rate(f64::from(SR));
        n.allocate();
        let pcm: Vec<f32> = (0..frames).map(|_| n.get_mono()).collect();
        super::hash(&pcm)
    }

    /// As três formas do alvo: sem nada fixado · semente e fase fixadas · fixadas mais um envelope.
    /// Devolve, para cada forma, o hash com: vizinho base · parâmetro do vizinho mudado · nó a mais
    /// no vizinho · vizinho depois do alvo. O vizinho tem ganho zero.
    #[must_use]
    pub fn vizinho(frames: usize) -> [(&'static str, [u64; 4]); 3] {
        fn casos(alvo: &dyn Fn() -> Net, frames: usize) -> [u64; 4] {
            let v0 = || Net::wrap(Box::new(sine_hz(440.0) * 0.0));
            let v1 = || Net::wrap(Box::new(sine_hz(880.0) * 0.0));
            let v2 = || Net::wrap(Box::new((sine_hz(440.0) >> lowpass_hz(1_000.0, 1.0)) * 0.0));
            [
                hash_net(v0() + alvo(), frames),
                hash_net(v1() + alvo(), frames),
                hash_net(v2() + alvo(), frames),
                hash_net(alvo() + v0(), frames),
            ]
        }
        let livre = || Net::wrap(Box::new((noise() + sine_hz(330.0)) * 0.5));
        let fixo = || {
            Net::wrap(Box::new(
                (noise().seed(7) + sine_hz(330.0).phase(0.0)) * 0.5,
            ))
        };
        let env = || {
            Net::wrap(Box::new(
                (noise().seed(7) + sine_hz(330.0).phase(0.0))
                    * envelope(|t: f32| 0.5 * exp(-t * 4.0)),
            ))
        };
        [
            ("sem semente", casos(&livre, frames)),
            ("semente+fase fixadas", casos(&fixo, frames)),
            ("fixadas+envelope", casos(&env, frames)),
        ]
    }
}

const FRAMES_HASH: usize = 96_000;

/// As funções que o `corre.sh` chama em cada alvo (nativo, x86-64-v3, wasm32 no `node`).
#[unsafe(no_mangle)]
pub extern "C" fn hash_kernels() -> u64 {
    hash(&kernels::render(&receita_tipica(FRAMES_HASH)))
}
#[unsafe(no_mangle)]
pub extern "C" fn hash_fundsp_tick() -> u64 {
    hash(&fd::render_tick(&receita_tipica(FRAMES_HASH)))
}
#[unsafe(no_mangle)]
pub extern "C" fn hash_fundsp_bloco() -> u64 {
    hash(&fd::render_bloco(&receita_tipica(FRAMES_HASH)))
}
