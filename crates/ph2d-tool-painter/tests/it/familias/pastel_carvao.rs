//! **Família `pastel_carvao`** — J4, catálogo F (carvão, giz, pastel): Charcoal, Soft Pastel, Oil Pastel, Square Hard,
//! Wax Scrape. Escreva aqui as receitas e os testes da família; nada fora deste ficheiro
//! (`docs/Painter/Pinceis/04_BANCADA.md` §9). O `CHARCOAL` é base PARTILHADA (o Scratch da família `lapis` nasce dele).

use super::prelude::*;

pub const WET: &[Receita] = &[
    // Pencil › Charcoal: pouca carga, orla larga. Prompt 35 (14/10), contra o Rebelle «Charcoal/Charcoal» a
    // 37 px vistos (nota 12,03 → 3,35): o Noise 0,15 era uma mancha (25 px a y = 40, 36 a 450) — Noise 48; o
    // carvão do Rebelle é RALO (miolo 0,64). A orla do Rebelle (15 px a 21 de largura, 25 px de subida a 38)
    // não cabe na dureza: a 0,5 o portão das mídias lia 7, e nenhuma dureza/papel/carga/arrasto passa de 9
    // (1 755 variantes). O perfil Sharp (p², a cauda longa) e o jitter 0,2 dão 10 e a poeira do carvão:
    // dureza 0,1, papel 0, carga 24, arrasto 0,02 — nota 3,31 → 0,79.
    CHARCOAL,
    // Pastel › Pastel Dry › Dry Thin: grão fino e regular. Prompt 35 (14/10), contra o Rebelle NO TEMPO e à
    // mesma largura vista (`ferramentas/pastel_regua.py`, nota 4,76 → 1,38): o grão Noise 0,4 era uma mancha
    // de ~640 px (a largura ia de 15 a 34 px conforme a ALTURA na tela) — Noise 100 é o grão de 2–4 px; o
    // papel 0,8 ondulava a borda; a carga 400 enchia o miolo (0,98 contra 0,91).
    SOFT_PASTEL,
    // Pastel › Pastel Oil › Oil Pastel 2 (o giz de cera): grão mais fino e mais carregado que o seco (o
    // Voronoi passava as métricas, mas a largura variava até 42 % com a posição — medido 11/10). Prompt 35
    // (14/10), contra o Rebelle «Pastel Oil/Oil Pastel 2» a 77 px vistos (nota 5,06 → 1,41): o Noise 0,15
    // também era mancha (42 px a y = 40, 68 a 450) — Noise 100. Com o arrasto na ponta seca: o portão das
    // mídias reprovava Textura (9,7 contra > 12,6) e Orla (4 contra > 6); o arrasto as BAIXA (0,3: Textura
    // 12,9 só com papel 1,2) e não faz as estrias do Rebelle (a prancha: ele alisa). Carga 100, jitter 0,1,
    // arrasto 0,05 — nota 1,45 → 1,05. ABERTO: as estrias ao longo do traço não vêm de nenhum destes botões.
    Receita {
        meio: "Pastel",
        grupo: "Oil",
        nome: "Oil Pastel",
        water: 0.0,
        opacity: 1.0,
        pigment: 100.0,
        pickup: 1.0,
        gate: 0.8,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.0,
        size_px: Some(77.0),
        extras: &[
            ("felt", 0.0),
            ("drag", 0.05),
            MAIS_PERTO,
            ("pressureSizeCurve", 0.5),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 2.0),
        ],
        grao: Some(("Noise", 100.0)),
        achatado: None,
        pincel: &[("jitter", "0.1")],
        nasce_de: None,
    },
    // `sq_007` (prompt 53): o pastel de ponta RETA, o papel todo aberto.
    Receita {
        nome: "Square Hard",
        pigment: 60.0,
        gate: 1.0,
        size_px: Some(35.0),
        extras: &[
            ("felt", 0.0),
            ("drag", 0.0),
            MAIS_PERTO,
            ("pressureSizeCurve", 0.25),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 2.0),
        ],
        pincel: &[("dab_quina", "1.0")],
        nasce_de: Some("Soft Pastel"),
        ..SOFT_PASTEL
    },
    // `wx_007` (prompt 53): a cera raspada — muita carga sobre o papel aberto.
    Receita {
        grupo: "Scrape",
        nome: "Wax Scrape",
        pigment: 150.0,
        gate: 1.0,
        size_px: Some(141.0),
        extras: &[
            ("felt", 0.0),
            ("drag", 0.0),
            MAIS_PERTO,
            ("pressureSizeCurve", 0.35),
            ("pressureOpacity", 0.75),
            ("pressureOpacityCurve", 1.5),
        ],
        pincel: &[("dab_quina", "0.0")],
        nasce_de: Some("Soft Pastel"),
        ..SOFT_PASTEL
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// O Cruzamento do Wax Scrape não entra (0,166 do controlo, abaixo de 3× o ruído desta régua); o raio do Wax por grelha.
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Charcoal", foto: "charcoal", raio: 10.5, metricas: &[(Cobertura, 0.384), (Borda, 0.378), (Orla, 15.0)] },
    Midia { preset: "Soft Pastel", foto: "pastel", raio: 8.5, metricas: &[(Cobertura, 0.752), (Borda, 0.443), (Orla, 6.0)] },
    Midia { preset: "Oil Pastel", foto: "oilpastel2", raio: 15.0, metricas: &[(Cobertura, 0.654), (Textura, 17.354), (Borda, 0.417), (Orla, 11.0)] },
    Midia { preset: "Square Hard", foto: "squarehard", raio: 15.50, metricas: &[(Cobertura, 0.705), (Textura, 20.884), (Borda, 0.856), (Cv, 0.077), (Cruzamento, 0.579), (Largura, 31.0)] },
    Midia { preset: "Wax Scrape", foto: "waxscrape", raio: 19.5, metricas: &[(Cobertura, 0.763), (Textura, 16.691), (Borda, 0.586), (Largura, 36.0)] },
];

const SOFT_PASTEL: Receita = Receita {
    meio: "Pastel",
    grupo: "Dry",
    nome: "Soft Pastel",
    water: 0.0,
    opacity: 1.0,
    pigment: 100.0,
    pickup: 1.0,
    gate: 0.4,
    evaporation: 1.0,
    edge: 0.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 2000.0,
    hardness: 0.2,
    size_px: Some(29.0),
    extras: &[
        ("felt", 0.0),
        ("drag", 0.0),
        MAIS_PERTO,
        ("pressureSizeCurve", 0.5),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 0.5),
    ],
    grao: Some(("Noise", 100.0)),
    achatado: None,
    pincel: &[],
    nasce_de: None,
};
