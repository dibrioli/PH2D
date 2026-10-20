//! **Família `oleo_acrilica`** — J4, catálogo G (óleo, acrílica): Oil Bristle, Acrylic Round, Express Oil, Dabber.
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).

use super::prelude::*;

/// Os três óleos foram afinados a Spacing 0,01 (o pigmento por dab deles é o de 0,01): o pigmento por comprimento
/// mantém a tinta do Express Oil e da Acrylic Round com os dabs mais esparsos.
const PASSO_DOS_OLEOS: f64 = 0.01;

// ⭐ O 2.º lote (11/10, depois da reavaliação pedida pelo dono — 18 §3: pigmento alto, o relevo de volta
// e o grão no lugar das cerdas). Cada um medido contra a foto do seu preset-oráculo do Rebelle, seco, à
// largura dela; o tamanho é o do preset dele. `cada_preset_parece_a_sua_midia` guarda-os.
pub const WET: &[Receita] = &[
    // Oil & Acrylic › Bristle Flat: corpo cheio, estrias das cerdas, relevo.
    OIL_BRISTLE,
    // Oil & Acrylic › Round Hard (a acrílica do Rebelle é esta ferramenta).
    ACRYLIC_ROUND,
    // Express Oil › Hairy: claro e liso, tufos (prompt 23, aceite do dono 12/10). A claridade é POUCO
    // pigmento — com o Spacing 0,01 cada célula leva ~100 carimbos, por isso 60 e não 250; sem água (a água
    // assentava o pouco pigmento nos poros: grão 0,065 → 0,027); cerdas 4 = tufos sem sal.
    Receita {
        meio: "ExpressOil",
        grupo: "Round",
        nome: "Express Oil",
        water: 0.0,
        opacity: 0.6,
        pigment: 60.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 4.0,
        bristle_count: 900.0,
        hardness: 0.9,
        size_px: Some(64.0),
        // Os FIOS (W2, doc 48_medicao §3): as faixas do Hairy do Rebelle são quase CONSTANTES pelo traço (carris
        // 0,85) e a textura dele é o contraste entre elas ⇒ a pressão de cada fio quase constante (ruído 2000 px)
        // e forte, e a mão leve que levanta os fracos. Textura 3,2 → 7,9 (9,0), CV 0,009 (0,008), carris 0,88,
        // leve 11,5 (10,2), forte 7,1 (7,3); o período 47 % a 6 px do carimbo dele não se copia (03 §3.1).
        // ⛔ Sem o relevo de cada fio (W3, doc 49_medicao): o controlo 7,94 não se distingue do Rebelle 9,0 (o ruído
        // da foto, recto × rápido, é 0,9) — 1 px por ganho dava 8,79, dentro do ruído.
        extras: &[
            ("felt", 0.0),
            ("bristleStrength", 2.0),
            ("emboss", 0.2),
            ("bristlePressure", 2.5),
            ("bristleLaneWidth", 2.0),
            ("bristleLift", 0.2),
            ("bristleNoiseLength", 2000.0),
            ("pigmentSpacing", PASSO_DOS_OLEOS),
            ("pressureSizeCurve", 0.5),
            ("pressureOpacity", 0.75),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // `dabberc_032` (prompt 53): o Oil Bristle de cerdas largas, mais relevo e a ponta espalhada (jitter) — a esponja.
    // L13 (J2, 20/10): a esponja é REDONDA (o vertical do Rebelle tem a largura do deitado: 38/38) e morde o papel; a
    // lâmina de pé do Oil Bristle espalhada pelo Jitter era o «rasgado» (rugosidade 2,97 × 1,05). Grelha flatten 0 ×
    // jitter 0–0,5 × paperGate 0–1 à largura do Rebelle: 0,4 · 1 = 390/420 (era 346), rugosidade 0 ✗ (eram 20).
    Receita {
        grupo: "Dabber",
        nome: "Dabber",
        bristle_size: 2.0,
        hardness: 0.6,
        size_px: Some(51.0),
        gate: 1.0,
        achatado: Some((0.0, 90)),
        extras: &[
            ("felt", 0.05),
            ("bristleStrength", 2.0),
            ("emboss", 1.2),
            ("bristlePressure", 1.2),
            ("bristleLaneWidth", 1.5),
            ("bristleLift", 0.4),
            ("bristleNoiseSat", 0.3),
            // O grão claro da esponja: as duas fotos do Rebelle cercam-no (a da bateria: textura do recto ≤ ~11, 0,6 dá
            // 11,3 ✗; a do 52 seca: ≥ 9,6, o vale do Wet Round) — 0,55 dá 10,9 e 9,67 (0,25: 9,9 e 8,1 ✗).
            ("bristleNoiseLight", 0.55),
            ("bristleHeight", 0.75),
            ("pigmentSpacing", 0.0),
            ("capsuleMaxPx", 2.0),
            ("pressureSizeCurve", 0.25),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 3.0),
        ],
        pincel: &[("jitter", "0.4")],
        nasce_de: Some("Oil Bristle"),
        ..OIL_BRISTLE
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// ⚠️ Os três do óleo levam os valores que o DONO afinou a olho contra o Rebelle (11/10, «vamos deixar esses
// valores»); a régua reprovava Textura do Oil Bristle (8,46 contra 5,86) e da Acrylic Round (30,4 contra 3,13)
// e a Borda do Express Oil (0,66 contra 0,80) — saíram daqui, a régua não mede o que o olho dele vê (18 §2).
// O Chato e a Tinta (prompt 23, aceites 12/10) são os dois pontos que o dono VIU: a mutação «preset de antes»
// sangra cada um sozinho (Oil 0,950 · Express 0,906).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Oil Bristle", foto: "oil", raio: 20.5, metricas: &[(Cobertura, 0.909), (Borda, 0.872), (Chato, 0.317)] },
    Midia { preset: "Acrylic Round", foto: "oilround", raio: 19.5, metricas: &[(Cobertura, 0.880), (Borda, 0.595)] },
    Midia { preset: "Express Oil", foto: "express", raio: 21.0, metricas: &[(Cobertura, 0.700), (Tinta, 0.772)] },
    Midia { preset: "Dabber", foto: "dabber", raio: 18.5, metricas: &[(Cobertura, 0.840), (Textura, 10.715), (Borda, 0.588), (Orla, 5.0), (Largura, 36.0)] },
];

const ACRYLIC_ROUND: Receita = Receita {
    meio: "Oil",
    grupo: "Round",
    nome: "Acrylic Round",
    water: 0.05,
    opacity: 0.6,
    pigment: 500.0,
    pickup: 1.0,
    gate: 0.0,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 0.5,
    bristle_count: 4000.0,
    hardness: 0.5,
    size_px: Some(78.0),
    extras: &[
        ("felt", 0.0),
        ("bristleStrength", 2.0),
        ("pigmentSpacing", PASSO_DOS_OLEOS),
        ("pressureSizeCurve", 1.4),
        // A textura das cerdas dela anda com o dab: a 0,025 a escada a 103 px dobrava (6,2 → 12,5); à densidade
        // dela 6,1, a diferença de imagem 0,04 (doc 51 §3).
        ("capsuleSpacing", PASSO_DOS_OLEOS),
    ],
    grao: None,
    achatado: None,
    pincel: &[],
    nasce_de: None,
};

const OIL_BRISTLE: Receita = Receita {
    meio: "Oil",
    grupo: "Bristle",
    nome: "Oil Bristle",
    water: 0.1,
    opacity: 1.0,
    pigment: 2000.0,
    pickup: 1.0,
    gate: 0.0,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 1.0,
    bristle_count: 2000.0,
    hardness: 0.9,
    size_px: Some(103.0),
    // O chato (prompt 23, aceite do dono 12/10): lâmina de pé, 0,68 = 1 − 13/41 (o traço de pé e o
    // deitado da foto); o feltro fecha as falhas que a lâmina fina abria. Régua: `mede_familia_c.py`.
    // Os FIOS (W2, doc 48_medicao §3): cada fio claro ou escuro pela cor (o óleo satura: a pressão só abriria
    // falhas), a pressão própria e a mão leve que os levanta. Textura 2,75 → 3,85 (Rebelle 5,86: o resto são
    // as sombras do relevo de cada fio, W3), carris 0,00 → 3,03 (3,32), CV 0,017, período 2 %.
    // O RELEVO de cada fio sem escada (W3b, doc 50_medicao): a altura contínua através dos fios, acesa pelo
    // MESMO emboss cinza da tinta (o de 18/10 — degrau por fio, luz do Impasto, 1,25 px + emboss 1 — saiu pelo
    // smoke do dono: «muito pixelado»). Textura 3,85 → 5,65 (Rebelle 5,86), contraste 8,7 → 13,4 (12,1), a
    // escada no inclinado 2,2 · 1,9 (2,5 · 2,6); a 103 px 1,9 · 1,4 (2,5 · 2,3).
    extras: &[
        ("felt", 0.05),
        ("bristleStrength", 2.0),
        ("bristlePressure", 1.2),
        ("bristleLaneWidth", 1.5),
        ("bristleLift", 0.4),
        ("bristleNoiseSat", 0.3),
        ("bristleNoiseLight", 0.25),
        ("bristleHeight", 0.75),
        // O pigmento por DAB (doc 51 §3): ele satura (a tinta a 103 px 0,985 da de antes sem o peso) e o peso só lhe
        // dava degraus — a escada da curva 2,57 · 2,70 (tecto 2,50 · 2,55) contra 2,15 · 2,40 sem ele.
        ("pigmentSpacing", 0.0),
        // Os fios dele têm largura em px: a 103 px a escada lê 2,24 · 2,45 · 2,15 com a cápsula a 2 px, 3,06 a 2,06
        // e 4,71 a 3 (tecto 2,78 · 2,60 · 2,50; doc 51 §3).
        ("capsuleMaxPx", 2.0),
        ("pressureSizeCurve", 0.25),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 2.0),
        // As leis de escala do Wet (J2, 03_MOTORES §7 c/d; `as_leis_da_escala`): o relevo pelo diâmetro (o Bristle Flat
        // do Rebelle 15,6 → 11,3 de 41 a 202 px; o nosso crescia 15 → 17), a tinta que não escoa para a faixa molhada
        // (a lente 19/12 → 12/12) e a ponta recta e desfiada de cada fio (6,5 px a 200 no Rebelle).
        // O expoente 0,4: o através 14 → 11 de 41 a 202 px (o Rebelle 15,4 → 11,5); a ponta ρ 0,8: desfiado ~6 px a 200.
        ("emboss", 0.7),
        ("embossSizeExp", 0.4),
        ("embossSizeRef", 41.0),
        ("levelingOwnStroke", 1.0),
        ("bristleTipRagged", 0.8),
    ],
    grao: None,
    achatado: Some((0.68, 90)),
    // A lâmina é um RECTÂNGULO na pegada (a quina 1: o corte recto a toda a largura); o desfiado é a lei das cerdas
    // (`bristleTipRagged`, J2 §7 c).
    pincel: &[("dab_quina", "1.0")],
    nasce_de: None,
};
