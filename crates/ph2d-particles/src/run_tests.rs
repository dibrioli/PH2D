//! **A corrida contra o ORÁCULO** — o relógio do Godot 4.7.2 (MIT), e o motor real do Motion.
//!
//! # A fixtura do oráculo (cabeçalho)
//!
//! `godot --headless --fixed-fps 60 --script docs/Components/ferramentas/godot_particles_probe.gd`
//! (2026-09-16, `4.7.2.stable.arch_linux.ed1daf0bf`). Oito partículas, `use_fixed_seed`. Para cada
//! caso: o quadro do `finished` e o quadro de origem (o `add_child`, ou o `restart()`).
//!
//! ⚠️ **A comparação é em TEMPO, e a janela é `[0, 1 quadro]` — uma convenção MEDIDA, não uma
//! folga:** o oráculo declara o fim no quadro que CONTÉM a morte (`floor(morte·60) − 1` desde a
//! origem nos onze casos, com o quadro de origem a contar `1/60` — a fracção do quadro de
//! nascimento, `fract_delta`), e nós no primeiro passo DEPOIS dela (idade `≤` vida ainda vive, a
//! convenção do nó). Medido: `+1` quadro em nove casos, `0` nos dois de explosividade `1` (a morte
//! cai exactamente num quadro). ⛔ Nunca ANTES do oráculo.

use super::*;
use crate::node_registry;

const DT: f64 = 1.0 / 60.0;
const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

fn base() -> ParticleEmitter {
    ParticleEmitter {
        amount: 8,
        life: 1.0,
        speed: 1.0,
        spread: 0.0,
        gravity: [0.0, 0.0],
        space: ParticleSpace::Local,
        ..ParticleEmitter::default()
    }
}

/// O passo em que a corrida grita `finished`, com `acao(passo, run)` antes de cada passo.
fn fim(
    cfg: &ParticleEmitter,
    reg: &NodeRegistry,
    passos: u32,
    mut acao: impl FnMut(u32, &mut EmitterRun) -> Option<EmitterRun>,
) -> Option<u32> {
    let mut run = EmitterRun::born(cfg, reg, Pose::IDENTITY, UV);
    let mut origem = 0;
    for k in 1..=passos {
        if let Some(novo) = acao(k, &mut run) {
            run = novo;
            origem = k;
        }
        if run.step(reg, Pose::IDENTITY, DT) {
            return Some(k - origem);
        }
    }
    None
}

/// ⭐⭐⭐ **O relógio do oráculo, caso a caso** — `(nome, cfg, quadros do oráculo desde a origem)`.
#[test]
fn o_fim_bate_com_o_oraculo() {
    let reg = node_registry();
    let rajada = |life: f32, e: f32| ParticleEmitter {
        one_shot: true,
        life,
        explosiveness: e,
        ..base()
    };
    let casos: Vec<(&str, ParticleEmitter, u32)> = vec![
        ("vida 1, e 0", rajada(1.0, 0.0), 111),
        ("vida 1, e 0,5", rajada(1.0, 0.5), 85),
        ("vida 1, e 1", rajada(1.0, 1.0), 59),
        ("vida 0,5, e 0", rajada(0.5, 0.0), 55),
        ("vida 0,5, e 0,5", rajada(0.5, 0.5), 42),
        ("vida 0,5, e 1", rajada(0.5, 1.0), 29),
        (
            "pré 0,5",
            ParticleEmitter {
                prewarm: 0.5,
                ..rajada(1.0, 0.0)
            },
            81,
        ),
        (
            "pré 1,5",
            ParticleEmitter {
                prewarm: 1.5,
                ..rajada(1.0, 0.0)
            },
            21,
        ),
        (
            "velocidade 2",
            ParticleEmitter {
                time_scale: 2.0,
                ..rajada(1.0, 0.0)
            },
            55,
        ),
        (
            "1 partícula",
            ParticleEmitter {
                amount: 1,
                ..rajada(1.0, 0.0)
            },
            59,
        ),
    ];
    for (nome, cfg, oraculo) in casos {
        let nosso =
            fim(&cfg, &reg, 400, |_, _| None).unwrap_or_else(|| panic!("{nome}: nunca acabou"));
        // O oráculo conta a origem como o quadro 1 do relógio: o seu tempo é `(j + 1)/60`.
        let (t_nosso, t_oraculo) = (f64::from(nosso) * DT, f64::from(oraculo + 1) * DT);
        eprintln!("[oráculo] {nome:<16} nós {nosso:>3} passos · oráculo {oraculo:>3} quadros");
        let atraso = t_nosso - t_oraculo;
        assert!(
            (-1e-9..=DT + 1e-9).contains(&atraso),
            "{nome}: nós {nosso} passos, oráculo {oraculo} quadros"
        );
    }
}

/// **L5 · L6 · L7 · C0** — desligar, religar, recomeçar e nunca ligar, contra o oráculo.
#[test]
fn ligar_desligar_e_recomecar_batem_com_o_oraculo() {
    let reg = node_registry();
    let cont = base();
    // Desligado no quadro 30 do oráculo ⇒ `finished` 82 (origem 1).
    let desligado = fim(&cont, &reg, 400, |k, r| {
        if k == 29 {
            r.stop();
        }
        None
    })
    .expect("L5: desligar acaba");
    eprintln!("[oráculo] L5 desligado no 30: nós {desligado} · oráculo 82 (origem 1 ⇒ 81)");
    // O oráculo: origem no quadro 1, logo o tempo do quadro 82 é `82/60`; o nosso passo `k` é `k/60`.
    assert!(
        (0..=1).contains(&(i64::from(desligado) - 82)),
        "L5: {desligado} contra 82"
    );
    // Religado no 45 ⇒ nunca acaba.
    let religado = fim(&cont, &reg, 300, |k, r| {
        if k == 29 {
            r.stop();
        }
        if k == 44 {
            r.start();
        }
        None
    });
    assert_eq!(
        religado, None,
        "L6: religar antes da última morte cancela o fim"
    );
    // Recomeçado no 30 ⇒ o fim conta desde o recomeço, como na rajada inteira (111).
    let rajada = ParticleEmitter {
        one_shot: true,
        ..base()
    };
    let recomecado = fim(&rajada, &reg, 400, |k, _| {
        (k == 30).then(|| EmitterRun::born(&rajada, &reg, Pose::IDENTITY, UV))
    })
    .expect("L7: a rajada recomeçada acaba");
    eprintln!("[oráculo] L7 recomeçado no 30: nós {recomecado} · oráculo 141 − 30 = 111");
    // O passo do recomeço já anda (`+1`); o oráculo: `141 − 30 = 111` quadros ⇒ tempo `112/60`.
    assert!(
        (0..=1).contains(&(i64::from(recomecado) + 1 - 112)),
        "L7: {recomecado}"
    );
    // Nunca ligado ⇒ nunca grita.
    let off = ParticleEmitter {
        emitting: false,
        ..base()
    };
    assert_eq!(fim(&off, &reg, 200, |_, _| None), None, "C0");
}

fn posicoes(run: &EmitterRun, pose: Pose) -> Vec<[f32; 2]> {
    let mut out = Vec::new();
    run.append_instances(&mut out, pose, 7);
    assert!(
        out.iter().all(|i| i.z_order == 7),
        "a profundidade do objecto"
    );
    out.iter().map(|i| i.world_pos).collect()
}

/// ⭐⭐⭐ **O espaço** — em `World` as partículas ficam onde nasceram (um rasto), em `Local` andam
/// com o objecto (um penacho preso).
#[test]
fn world_deixa_rasto_e_local_leva_o_penacho() {
    let reg = node_registry();
    for space in [ParticleSpace::World, ParticleSpace::Local] {
        let cfg = ParticleEmitter {
            amount: 30,
            life: 1.0,
            speed: 1.0,
            spread: 0.0,
            angle: 90.0,
            space,
            ..base()
        };
        let pose_em = |k: u32| Pose::at(f32::from(u16::try_from(k).unwrap()) * 0.05, 0.0);
        let mut run = EmitterRun::born(&cfg, &reg, pose_em(0), UV);
        for k in 1..=40 {
            run.step(&reg, pose_em(k), DT);
        }
        let agora = pose_em(40);
        let xs: Vec<f32> = posicoes(&run, agora).iter().map(|p| p[0]).collect();
        assert!(xs.len() > 10, "{space:?}: há partículas");
        let (lo, hi) = xs
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), &x| (l.min(x), h.max(x)));
        match space {
            ParticleSpace::Local => assert!(
                (lo - agora.translation[0]).abs() < 1e-4
                    && (hi - agora.translation[0]).abs() < 1e-4,
                "Local: todas no x de agora ({}), leu {lo}..{hi}",
                agora.translation[0]
            ),
            ParticleSpace::World => {
                assert!(
                    hi <= agora.translation[0] + 1e-3,
                    "World: nenhuma à frente do objecto"
                );
                assert!(
                    hi - lo > 0.8,
                    "World: um rasto de quase 1 m, leu {lo}..{hi}"
                );
            }
        }
    }
}

/// **Local roda o cone com o objecto** — um objecto de lado lança de lado.
#[test]
fn local_roda_o_cone_com_o_objecto() {
    let reg = node_registry();
    let cfg = ParticleEmitter {
        amount: 30,
        ..base()
    };
    let de_lado = Pose {
        translation: [0.0, 0.0],
        basis: [0.0, 1.0, -1.0, 0.0], // 90° — o «para cima» do objecto é o −x do mundo
    };
    let mut run = EmitterRun::born(&cfg, &reg, de_lado, UV);
    for _ in 0..30 {
        run.step(&reg, de_lado, DT);
    }
    let ps = posicoes(&run, de_lado);
    assert!(
        ps.iter().all(|p| p[0] <= 1e-4 && p[1].abs() < 1e-4),
        "{ps:?}"
    );
    assert!(ps.iter().any(|p| p[0] < -0.2), "saíram pelo −x: {ps:?}");
}

/// **A gravidade puxa e o amortecimento trava** — o laço de forças chega às partículas.
#[test]
fn a_gravidade_puxa_e_o_amortecimento_trava() {
    let reg = node_registry();
    let lateral = ParticleEmitter {
        amount: 60,
        life: 2.0,
        angle: 0.0,
        speed: 2.0,
        gravity: [0.0, -9.8],
        ..base()
    };
    let mut run = EmitterRun::born(&lateral, &reg, Pose::IDENTITY, UV);
    for _ in 0..60 {
        run.step(&reg, Pose::IDENTITY, DT);
    }
    let ps = posicoes(&run, Pose::IDENTITY);
    let mais_velha = ps[0];
    assert!(
        mais_velha[1] < -3.0,
        "caiu ~4,9 m num segundo: {mais_velha:?}"
    );
    assert!(
        (mais_velha[0] - 2.0).abs() < 0.1,
        "e andou 2 m de lado: {mais_velha:?}"
    );

    let travado = ParticleEmitter {
        gravity: [0.0, 0.0],
        damping: 2.0,
        ..lateral
    };
    let mut run = EmitterRun::born(&travado, &reg, Pose::IDENTITY, UV);
    for _ in 0..60 {
        run.step(&reg, Pose::IDENTITY, DT);
    }
    let x = posicoes(&run, Pose::IDENTITY)[0][0];
    assert!(
        x > 0.3 && x < 1.9,
        "o amortecimento encurta o caminho de 2 m: {x}"
    );
}

/// ⭐⭐ **Cor e tamanho ao longo da vida** — a mais velha tem a cor e o tamanho finais.
#[test]
fn cor_e_tamanho_ao_longo_da_vida() {
    let reg = node_registry();
    let cfg = ParticleEmitter {
        amount: 60,
        life: 1.0,
        size: 0.2,
        size_end: 0.25,
        color: [1.0, 0.0, 0.0, 1.0],
        color_end: [0.0, 0.0, 1.0, 0.0],
        ..base()
    };
    let mut run = EmitterRun::born(&cfg, &reg, Pose::IDENTITY, UV);
    for _ in 0..90 {
        run.step(&reg, Pose::IDENTITY, DT);
    }
    let mut out = Vec::new();
    run.append_instances(&mut out, Pose::IDENTITY, 0);
    let (velha, nova) = (out[0], out[out.len() - 1]);
    assert!(
        velha.tint[2] > 0.9 && velha.tint[0] < 0.1,
        "a velha é azul: {:?}",
        velha.tint
    );
    assert!(
        nova.tint[0] > 0.9 && nova.tint[2] < 0.1,
        "a nova é vermelha: {:?}",
        nova.tint
    );
    assert!(
        velha.size[0] < 0.07,
        "a velha encolheu para ~0,05: {:?}",
        velha.size
    );
    assert!(
        (nova.size[0] - 0.2).abs() < 0.02,
        "a nova tem o tamanho de nascer: {:?}",
        nova.size
    );
}

/// **A mesma definição dá as mesmas partículas** — o replay bit a bit.
#[test]
fn a_mesma_definicao_da_as_mesmas_particulas() {
    let reg = node_registry();
    let cfg = ParticleEmitter {
        amount: 50,
        spread: 60.0,
        speed_random: 0.5,
        gravity: [0.0, -9.8],
        seed: 3,
        space: ParticleSpace::World,
        ..base()
    };
    let corre = || {
        let mut run = EmitterRun::born(&cfg, &reg, Pose::at(1.0, 2.0), UV);
        for k in 0..50u16 {
            run.step(&reg, Pose::at(1.0 + f32::from(k) * 0.01, 2.0), DT);
        }
        let mut out = Vec::new();
        run.append_instances(&mut out, Pose::IDENTITY, 0);
        out.iter()
            .map(|i| (i.world_pos[0].to_bits(), i.world_pos[1].to_bits()))
            .collect::<Vec<_>>()
    };
    assert_eq!(corre(), corre());
}

/// **Todo grafo que o compilador monta é bem tipado** — cada combinação de módulos.
#[test]
fn todo_grafo_compilado_e_valido() {
    let reg = node_registry();
    for space in [ParticleSpace::World, ParticleSpace::Local] {
        for gravity in [[0.0, 0.0], [0.0, -9.8]] {
            for damping in [0.0, 1.0] {
                for size_end in [1.0, 0.5] {
                    for color_end in [[1.0; 4], [0.0, 0.0, 1.0, 1.0]] {
                        let cfg = ParticleEmitter {
                            space,
                            gravity,
                            damping,
                            size_end,
                            color_end,
                            ..base()
                        };
                        let c = compile(&cfg, &Emission::born(&cfg));
                        c.graph
                            .validate(&reg)
                            .unwrap_or_else(|v| panic!("{cfg:?}: {v:?}"));
                    }
                }
            }
        }
    }
}
