//! **Família `aguarela`** — J4, catálogo I (aguarela, sumi-e): Wet Wash, Wet Dry Brush, Wet Drip, Wet Liner, Wet Flat, Sumi-e.
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//! ⛔ O Wet Round NÃO é desta família: é o pincel sem preset e o controlo dos portões (`receitas_de_fabrica::RECEITAS`).

use super::prelude::*;

/// O pigmento do Wet Flat (doc 44 §3, lote 3). J4: 44 casava a cobertura do recto do Rebelle (régua 383 → 399/456) mas
/// tirava a rampa da pressão da origem (`leis_sistemicas`, 0,212 > 0,18) — fica 50 (`exames/aguarela.md` I-3).
const PIGMENTO_DO_FLAT: f64 = 50.0;
/// A curva da pressão → opacidade (prompt 45, doc 45 §2): a rampa da cena seca contra o Rebelle — 0,41 (0,42).
const CURVA_DO_FLAT: f64 = 1.75;

pub const WET: &[Receita] = &[
    // J4 (`exames/aguarela.md` I-2): NÃO aprovado, receita de antes. A cura medida contra o Rebelle `Wash Round` (dura
    // 0,9, papel 0, pigmento 5, orla 200, água por dab 2, largura fixa) leva a régua 349 → 384/454, mas dois revisores
    // cegos repetiram o miolo mosqueado e a orla oca na pressão fraca, e ela parte duas leis da J2 (`leis_sistemicas`:
    // o traço de 5 px cobre 0,77 do de 40 — Rebelle 0,98; a rampa a 0,277 da origem > 0,18).
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Wash",
        water: 0.4,
        opacity: 1.0,
        pigment: 21.0,
        pickup: 0.6,
        gate: 0.3,
        evaporation: 0.7,
        edge: 20.0,
        tilt: DIREITA,
        bristle_size: 3.0,
        bristle_count: 3000.0,
        hardness: 0.0,
        size_px: None,
        extras: &[
            ("pressureSizeCurve", 0.35),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Dry Brush",
        // J4: NÃO aprovado (`docs/Painter/Pinceis/exames/aguarela.md` I-5). A origem (Rebelle `Watercolor/Flat Dry`) é
        // um chato cheio: curá-lo para ela deu o Wet Flat sem esmaecer (régua 220 → 308/454, e o revisor cego emparelhou-o
        // com o Wet Flat em 6/6) — a receita fica a de antes, e a fusão vai ao coordenador.
        water: 0.2,
        opacity: 1.0,
        pigment: 300.0,
        pickup: 1.6,
        gate: 0.4,
        evaporation: 1.5,
        edge: 56.25,
        tilt: DIREITA,
        bristle_size: 1.5,
        bristle_count: 1500.0,
        hardness: 0.0,
        size_px: None,
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Drip",
        water: 1.0,
        opacity: 1.0,
        pigment: 63.0,
        pickup: 0.8,
        gate: 0.2,
        evaporation: 0.6,
        edge: 20.0,
        tilt: Tilt {
            on: true,
            ring: 8,
            spoke: 3,
        },
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.0,
        size_px: None,
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // ⭐ O 1.º da Fase C (prompt 17, 05 §4b «Presets de todas as mídias»): o Liner do Rebelle
    // (`Watercolor/Liner`: duro, carregado, papel 0,10 — o grão quase não fura, 28,86 px). Medido contra
    // `fixtures/rebelle/wcliner.png` (cobertura 0,65 · textura 1,7 %) à largura 38; o Wet Round dá 0,19 · 8,1.
    // J4 (`exames/aguarela.md` I-4): NÃO aprovado, receita de antes. A cura contra a bateria (dura 1, papel 0, orla
    // 200, pigmento 65: régua 322 → 376/454) foi lida como «grão e halo esfiapado» por dois revisores e sobe a textura
    // fora do vale da foto `wcliner` (6,28 contra 1,71; `cada_preset_parece_a_sua_midia`).
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Liner",
        water: 0.5,
        opacity: 1.0,
        pigment: 100.0,
        pickup: 1.0,
        gate: 0.05,
        evaporation: 1.0,
        edge: 100.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: Some(29.0),
        extras: &[],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
    // ⭐ As FORMAS (prompt 44, doc 44): o Wet Round com a ponta RETA (quina 1, dura 0,9 — a ponta macia lê-se
    // redonda), um pouco mais alta que larga e FIXA na tela (o rato do Rebelle não roda a ponta), e a pressão
    // que ESMAECE em vez de estreitar. Watercolor › Flat do Rebelle (`wcflat`).
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Wet Flat",
        water: 1.0,
        opacity: 1.0,
        pigment: PIGMENTO_DO_FLAT,
        pickup: 1.0,
        gate: 0.4,
        evaporation: 1.0,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("pressureSize", 0.3),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", CURVA_DO_FLAT),
        ],
        grao: None,
        achatado: Some((0.12, 90)),
        pincel: &[("dab_quina", "1.0")],
        nasce_de: None,
    },
];

/// ⛔ Os REPROVADOS da J4 (`docs/Painter/Pinceis/exames/aguarela.md`): a receita medida fica para quando o motor que lhes
/// falta existir — sem ficheiro de preset (a fábrica são os aprovados e os pré-existentes).
pub const EM_ESTUDO: &[Receita] = &[
    // J4 I-7: a trincha CARREGADA do Rebelle (`Watercolor/Mop`): a lavagem sem o fio escuro, três vezes mais
    // pigmento, muita água (a espiral funde-se num disco), e a pressão estreita E esmaece. Régua 368/445; reprovado: o
    // miolo listrado e o anel claro por dentro da orla (o Rebelle é liso) não saem com valores (motor: o miolo liso).
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Mop",
        water: 1.0,
        opacity: 1.0,
        pigment: 15.0,
        pickup: 0.6,
        gate: 0.05,
        evaporation: 0.7,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 3.0,
        bristle_count: 3000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("waterPerDab", 1.5),
            ("pressureSize", 0.7),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: Some("Wet Wash"),
    },
    // J4 I-8: a trincha de CERDAS da aguarela do Rebelle (`Bristle/Bristle Flat`, 46 × 16 px): a lâmina FIXA na tela
    // (larga deitada, fina de pé; a espiral abre buracos), cheia, de borda direita — o Wet Flat achatado a 0,65. Régua
    // 282/454; reprovado: o pigmento é por COMPRIMENTO e a lâmina deitada varre 3× a área da de pé — o recto fica a 0,28
    // e o vertical a 0,43 (Rebelle 0,44 · 0,42); nenhum pigmento casa os dois (motor: o pigmento pela largura da pegada).
    Receita {
        meio: "Watercolor",
        grupo: "Fluid",
        nome: "Bristle Watercolor",
        water: 1.0,
        opacity: 1.0,
        pigment: 60.0,
        pickup: 1.0,
        gate: 0.4,
        evaporation: 1.0,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("pressureSize", 0.3),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", CURVA_DO_FLAT),
        ],
        grao: None,
        achatado: Some((0.65, 90)),
        pincel: &[("dab_quina", "1.0")],
        nasce_de: Some("Wet Flat"),
    },
    // J4 I-13: o Sumi-e do Rebelle (`Sumi-e/Sumi-e soft`): o pincel de cerdas da tinta (o Ink Brush) com ÁGUA — sem
    // o reservatório nem o vidrado da tinta, a ponta desfiada inteira, o pigmento que se dilui (45: régua 306 → 378/454,
    // vizinho Charcoal 1,01). Reprovado: o revisor cego achou o Rebelle em 6/6 — as cerdas dele são um PENTE de riscas
    // finas com um fio escuro de um lado e falhas brancas (a espiral mostra as voltas) e bordas duras a 200 px; o nosso
    // é liso, de pontas redondas e halo esfiapado (`bristleStrength` 1,6 + `bristleNoiseLight` 0,45: riscas largas e
    // ondeadas, régua 295) — motor: as cerdas em pente do Wet.
    Receita {
        meio: "Watercolor",
        grupo: "Brush",
        nome: "Sumi-e",
        water: 0.5,
        opacity: 1.0,
        pigment: 45.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 12.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("glaze", 0.0),
            ("tipReservoir", 0.0),
            ("leveling", 0.5),
            ("bristleTipRagged", 1.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: Some("Ink Brush"),
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// Medido contra `fixtures/rebelle/wcliner.png` à largura dela; as formas (prompt 44, doc 44 §3) e a pressão que
// ESMAECE (prompt 45, doc 45).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Wet Liner", foto: "wcliner", raio: 19.0, metricas: &[(Cobertura, 0.653), (Textura, 1.708)] },
    Midia { preset: "Wet Flat", foto: "wcflat", raio: 17.0, metricas: &[(Cobertura, 0.315), (Ponta, 0.096), (RampaLargura, 0.829), (RampaCobertura, 0.42), (Largura, 34.0), (Pontas, 0.9)] },
];

/// Os reprovados da J4 ([`EM_ESTUDO`]) não têm ficheiro na fábrica, e nenhum tem o nome de um pincel dela.
#[test]
fn os_reprovados_da_aguarela_nao_estao_na_fabrica() {
    let b = ph2d_brush_presets::Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let na_fabrica: Vec<&str> = b
        .entradas
        .iter()
        .map(|e| e.preset.parametros.name.as_str())
        .filter(|n| EM_ESTUDO.iter().any(|r| r.nome == *n))
        .collect();
    assert!(na_fabrica.is_empty(), "reprovados na fábrica: {na_fabrica:?}");
}
