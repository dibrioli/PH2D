//! As cenas dos gates, em metros: 1 px da réplica = 1 cm, o quadro de 512 px = 5,12 m.

use ph2d_light2d_ecs::CenaDeLuz;

use crate::comum::{disco, oclusor, regular};

/// Os degraus: texel da c0 a 1, 2 e 4 px da réplica (grade 512², 256², 128²).
pub const DEGRAUS: [(&str, u32, f64); 3] = [
    ("Alto (c0 a 1 px)", 512, 0.01),
    ("Médio (c0 a 2 px)", 256, 0.02),
    ("Baixo (c0 a 4 px)", 128, 0.04),
];

pub const LUZ: [f32; 2] = [1.20, 2.56];
/// Raio da luz da réplica: 10 px de diâmetro.
pub const RAIO_LUZ: f32 = 0.05;
const L: f32 = 1.0;

/// O oclusor de 14 px de lado, girado (fora dos eixos: uma fixture alinhada não mede uma base).
fn oclusor_14px() -> ph2d_light2d_ecs::Oclusor {
    oclusor(
        regular([1.60, 2.50], 0.14 / std::f32::consts::SQRT_2, 4, 0.52),
        1.0,
    )
}

/// **Réplica da HRC Fig. 8** (Freeman et al., 2025): 512², luz de 10 px, oclusor de 14 px. A
/// disposição exata da figura não está nas notas da pesquisa; aqui a luz fica à esquerda e o
/// oclusor a 40 px dela, para a penumbra correr para a direita, como na figura.
pub fn penumbra() -> CenaDeLuz {
    CenaDeLuz {
        emissores: vec![disco(LUZ, RAIO_LUZ, [L; 3], f32::INFINITY)],
        oclusores: vec![oclusor_14px()],
        ceu: [0.0; 3],
        ..Default::default()
    }
}

/// **Faísca**: a mesma cena com uma luz de 2 px de diâmetro.
pub fn faisca() -> CenaDeLuz {
    CenaDeLuz {
        emissores: vec![disco(LUZ, 0.01, [L; 3], f32::INFINITY)],
        oclusores: vec![oclusor_14px()],
        ceu: [0.0; 3],
        ..Default::default()
    }
}

/// **Céu e vidro**: céu colorido, dois vidros de α 0,5 SOBREPOSTOS (τ = 0,25 na interseção), uma
/// parede opaca fina e girada, e uma luz colorida de alcance FINITO (a `queda` no caminho).
pub fn ceu_e_vidro() -> CenaDeLuz {
    let parede = vec![[1.00, 1.00], [1.12, 1.03], [0.62, 3.03], [0.50, 3.00]];
    CenaDeLuz {
        emissores: vec![disco([3.6, 3.6], 0.08, [1.0, 0.6, 0.2], 2.0)],
        oclusores: vec![
            oclusor(regular([2.6, 2.4], 0.4, 4, 0.3), 0.5),
            oclusor(regular([2.9, 2.2], 0.35, 3, 1.0), 0.5),
            oclusor(parede, 1.0),
        ],
        ceu: [0.05, 0.08, 0.12],
        ..Default::default()
    }
}

/// O lado da região do enxame (400 px da réplica), centrada no quadro.
const LADO_ENXAME: f32 = 4.0;
/// O raio do emissor ÚNICO; com N emissores, cada um tem `R/√N` — a área emissora total é a mesma.
const R_ENXAME: f32 = 1.0;

/// **Enxame** (G4, K3 — S2 do relatório): `n` discos de raio `R/√n` numa treliça `k×k`
/// (`k = ⌈√n⌉`) sobre a MESMA região, células escolhidas por um passo coprimo com `k²`. Mesma área emissora, mesma região: só a CONTAGEM muda. Com `n = 10 000` o
/// raio é 1 px, num canto de texel: cada disco possui 4 texels no Alto (cobertura π/4) — por isso
/// o G4 corre no Alto.
pub fn enxame(n: u32) -> CenaDeLuz {
    let k = (f64::from(n).sqrt().ceil()) as u32;
    let celulas = k * k;
    // O coprimo mais perto de 0,618·k² (o passo áureo): os n primeiros espalham-se pela região.
    let alvo = ((f64::from(celulas) * 0.618_034) as u32).max(1);
    let passo = (1..=alvo)
        .rev()
        .find(|p| gcd(*p, celulas) == 1)
        .unwrap_or(1);
    let espaco = LADO_ENXAME / k as f32;
    let raio = R_ENXAME / (n as f32).sqrt();
    let origem = (5.12 - LADO_ENXAME) / 2.0;
    let emissores = (0..n)
        .map(|i| {
            let c = (u64::from(i) * u64::from(passo) % u64::from(celulas)) as u32;
            let centro = [
                origem + ((c % k) as f32 + 0.5) * espaco,
                origem + ((c / k) as f32 + 0.5) * espaco,
            ];
            disco(centro, raio, [L; 3], f32::INFINITY)
        })
        .collect();
    CenaDeLuz {
        emissores,
        oclusores: vec![],
        ceu: [0.0; 3],
        ..Default::default()
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Texels onde a luz se vê SEM obstáculo (o gabarito é a fórmula do vazio) e longe dela.
pub fn regiao_livre(r: &[[f64; 3]], lado: u32, texel: f64) -> impl Fn(usize) -> bool + '_ {
    let a = f64::from(RAIO_LUZ);
    move |k| {
        let p = [
            (f64::from(k as u32 % lado) + 0.5) * texel,
            (f64::from(k as u32 / lado) + 0.5) * texel,
        ];
        let d = (p[0] - f64::from(LUZ[0])).hypot(p[1] - f64::from(LUZ[1]));
        let vazio = 2.0 * (a / d).asin();
        d > 3.0 * a && (r[k][0] - vazio).abs() <= 1e-9 * vazio
    }
}

/// **Enxame de luzes** (G4, K3 — a S2 do relatório isolada): a MESMA treliça de 100×100 octógonos
/// de raio 1 px na região do enxame, dos quais os `luzes` primeiros (na ordem do passo áureo) EMITEM
/// e os outros são oclusores pretos opacos. A geometria — o que a marcha vê — é idêntica para toda
/// contagem; só o número de luzes muda. (A dependência da GEOMETRIA é outro eixo, a S3: o
/// `enxame` de área emissora fixa mede-a na sonda.)
pub fn enxame_de_luzes(luzes: u32) -> CenaDeLuz {
    const K: u32 = 100;
    let celulas = K * K;
    let alvo = (f64::from(celulas) * 0.618_034) as u32;
    let passo = (1..=alvo)
        .rev()
        .find(|p| gcd(*p, celulas) == 1)
        .unwrap_or(1);
    let espaco = LADO_ENXAME / K as f32;
    let origem = (5.12 - LADO_ENXAME) / 2.0;
    let mut cena = CenaDeLuz {
        emissores: vec![],
        oclusores: vec![],
        ceu: [0.0; 3],
        ..Default::default()
    };
    for i in 0..celulas {
        let c = (u64::from(i) * u64::from(passo) % u64::from(celulas)) as u32;
        let centro = [
            origem + ((c % K) as f32 + 0.5) * espaco,
            origem + ((c / K) as f32 + 0.5) * espaco,
        ];
        let vertices = regular(centro, 0.01, 8, 0.2);
        if i < luzes {
            cena.emissores.push(ph2d_light2d_ecs::Emissor::Poligono {
                vertices,
                radiancia: [L; 3],
                alcance_m: f32::INFINITY,
            });
        } else {
            cena.oclusores.push(oclusor(vertices, 1.0));
        }
    }
    cena
}
