//! **Hash dourado por receita de fábrica.** Vermelho = o motor mudou de bytes: ou sobe `ENGINE_V1`, ou o conserto está
//! errado. Hash da sonda de W0, só para comparação de ordem: `e262c4b2a017544c` (outros kernels, outra receita).

use ph2d_sfx_dsp::{pcm_hash, render};
use ph2d_sfx_recipe::{ENGINE_V1, Take, Takes, factory};

const DOURADOS: [(&str, u64); 8] = [
    ("Whoosh", 0x7040f32cf3a84a03),
    ("Laser", 0x3824b38df643d294),
    ("Metal Hit", 0x36659fb676bf5064),
    ("Wood Knock", 0x9c821bb160b510aa),
    ("Glass Ping", 0x4b4798b10ef15048),
    ("Drum Thump", 0x0b5aa9b796f46424),
    ("FM Bell", 0x37c39ff241adacf1),
    ("Impact", 0x33ecba068ec66193),
];
const DOURADO_GRAVACAO: u64 = 0xac79d980833fb4af;

/// Material de teste a 44,1 kHz, estéreo: exercita a troca de taxa e a mistura de canais.
fn material() -> Take {
    let s: Vec<f32> = (0..44_100)
        .map(|i| {
            let t = i as f32 / 44_100.0;
            let queda = 1.0 - t;
            let n = (i * 7919 % 1009) as f32 / 1009.0 - 0.5;
            if i % 2 == 0 {
                (t * 2_764.6).rem_euclid(1.0) * queda - 0.5 * queda
            } else {
                n * queda
            }
        })
        .collect();
    Take::new(44_100, 2, s)
}

#[test]
fn hash_dourado_das_receitas_de_fabrica() {
    let mut erros = Vec::new();
    for (nome, r) in factory::all() {
        let pcm = render(&r, r.seed, ENGINE_V1, &Takes::default()).unwrap();
        let h = pcm_hash(&pcm.samples);
        let esperado = DOURADOS.iter().find(|(n, _)| *n == nome).unwrap().1;
        if h != esperado {
            erros.push(format!("(\"{nome}\", 0x{h:016x}),"));
        }
    }
    assert!(erros.is_empty(), "mudaram:\n{}", erros.join("\n"));
    assert_eq!(factory::all().len(), DOURADOS.len());
}

#[test]
fn hash_dourado_da_receita_de_gravacao() {
    let mut takes = Takes::default();
    let id = takes.add(material());
    let r = factory::recorded(id, 1.0);
    let pcm = render(&r, r.seed, ENGINE_V1, &takes).unwrap();
    assert!(pcm.missing.is_empty());
    let h = pcm_hash(&pcm.samples);
    assert_eq!(h, DOURADO_GRAVACAO, "0x{h:016x}");
}

#[test]
fn nenhuma_receita_de_fabrica_passa_do_teto() {
    for (nome, r) in factory::all() {
        let pcm = render(&r, r.seed, ENGINE_V1, &Takes::default()).unwrap();
        let pico = pcm.samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(pico > 0.05 && pico <= 1.0, "{nome}: pico {pico}");
    }
}

#[test]
fn motor_desconhecido_nao_toca_outro_som() {
    let (_, r) = factory::all().remove(0);
    assert_eq!(
        render(&r, r.seed, ENGINE_V1 + 1, &Takes::default()),
        Err(ph2d_sfx_dsp::RenderError::UnknownEngine(ENGINE_V1 + 1))
    );
}

#[test]
fn material_que_falta_e_dito_e_a_camada_sai_em_silencio() {
    let r = factory::recorded([9; 32], 0.1);
    let pcm = render(&r, r.seed, ENGINE_V1, &Takes::default()).unwrap();
    assert_eq!(pcm.missing, vec![r.layers[0].id]);
    assert!(pcm.samples.iter().all(|s| *s == 0.0));
}
