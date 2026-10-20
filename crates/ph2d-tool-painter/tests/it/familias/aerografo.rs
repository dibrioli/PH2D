//! **Família `aerografo`** — J3, catálogo J: Airbrush, Spray Can, Splatter, Spray Drips, Stipple, Grainy Shader.
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).

use super::prelude::*;

pub const WET: &[Receita] = &[
    // Airbrush › Spray › Airbrush: o PÓ (prompt 43) — gotas de 2 px, uma em quatro por dab, seco, sem cerdas;
    // contra o Rebelle «Airbrush/Default/Spray 2» (doc 43 §4: a mais perto do Rebelle das 7/7).
    Receita {
        meio: "Airbrush",
        grupo: "Spray",
        nome: "Airbrush",
        water: 0.0,
        opacity: 1.0,
        pigment: 9.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.3,
        size_px: None,
        // A pressão do Spray 2 (dono, smoke do P44: «como no Rebelle»): rampa 0,63/0,34. Com a opacidade que
        // POUSA (prompt 45) e a curva 1,5: 0,60/0,35. Os números LITERAIS dele (tamanho 0) esmaecem igual mas
        // não afinam (largura 1,00) — doc 45 §2.
        extras: &[
            ("drag", 0.0),
            ("dust", 0.8),
            MAIS_PERTO,
            ("pressureSize", 0.5),
            // As PONTAS do Rebelle (ordem do dono 19/10, doc 51 §4): a curva côncava alarga o traço leve mais cedo —
            // pela mediana de 21 tiragens do pó, pontas 0,580 → 0,680 (Rebelle 0,80), a rampa 0,640 → 0,704 (0,63).
            // Nenhuma intensidade com a curva recta casa as duas: as pontas seguem a rampa a −0,06.
            ("pressureSizeCurve", 0.7),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[("falloff", "\"Smooth\""), ("shape", SEM_FORMA)],
        nasce_de: None,
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// O PÓ — a textura, a uniformidade e a orla larga são dele; o «período» não conta (a repetição do Rebelle varia 0,005,
// o controlo fica a 0,013 — doc 43 §2).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Airbrush", foto: "airbrush", raio: 15.0, metricas: &[(Cobertura, 0.381), (Textura, 22.705), (Borda, 0.537), (Orla, 12.0), (Cv, 0.088), (Cruzamento, 0.583), (RampaLargura, 0.63), (RampaCobertura, 0.34), (Largura, 26.0), (Pontas, 0.8)] },
];

/// O Shape sem forma (nem cerdas): a ponta é só a queda — o aerógrafo.
const SEM_FORMA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
