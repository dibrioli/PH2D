//! **Família `guache`** — J4, catálogo H (guache): Gouache Flat (+ Round, Dry, Wash em estudo; `docs/Painter/Pinceis/exames/guache.md`). Escreva aqui as receitas e os testes da família; nada
//! fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).

use super::prelude::*;

/// O pigmento da Gouache Flat (doc 44 §3, lote 3).
const PIGMENTO_DA_GUACHE: f64 = 100.0;
/// A curva da pressão → opacidade (prompt 45, doc 45 §2): a rampa da cena seca contra o Rebelle — 0,42 (0,43).
const CURVA_DA_GUACHE: f64 = 2.0;

pub const WET: &[Receita] = &[
    // A guache FLAT do Rebelle (`gouacheflat`, Gouache › Gouache Flat): a ponta do Wet Flat, mais alta que larga, e o
    // DOBRO da tinta (cobertura 0,50 × 0,32). A Gouache Round não entra: no Rebelle ela é o nosso Round (18 §4.3).
    Receita {
        meio: "Watercolor",
        grupo: "Gouache",
        nome: "Gouache Flat",
        water: 1.0,
        opacity: 1.0,
        pigment: PIGMENTO_DA_GUACHE,
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
            ("pressureOpacityCurve", CURVA_DA_GUACHE),
        ],
        grao: None,
        achatado: Some((0.2, 90)),
        pincel: &[("dab_quina", "1.0")],
        nasce_de: None,
    },
];

/// ⛔ Os REPROVADOS da J4 (`docs/Painter/Pinceis/exames/guache.md`): a receita medida fica para quando o motor que lhes
/// falta existir — sem ficheiro de preset (a fábrica são os aprovados e os pré-existentes).
pub const EM_ESTUDO: &[Receita] = &[
    // J4 H-1: a guache REDONDA do Rebelle (`Gouache/Gouache Round`): a ponta redonda dura, um terço da tinta da Flat (o
    // véu claro e liso, sem o grão do papel da Watercolor Round), e a pressão que estreita E esmaece.
    Receita {
        meio: "Watercolor",
        grupo: "Gouache",
        nome: "Gouache Round",
        water: 1.0,
        opacity: 1.0,
        pigment: 32.0,
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
            ("pressureSize", 0.8),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            // A guache seca DEPRESSA: a água sai a 0,97 por quadro (0,995 de omissão) — o azul que cruza o vermelho
            // molhado já não o empurra em manchas escuras dos lados (o Rebelle passa limpo). Régua 424 → 417/454.
            ("retention", 0.97),
        ],
        grao: None,
        achatado: None,
        // Nasce da Flat: a ponta volta a REDONDA (sem a quina nem o achatado dela).
        pincel: &[("dab_quina", "0.0"), ("dab_flatten", "0.0")],
        nasce_de: Some("Gouache Flat"),
    },
    // J4 H-4: a guache DILUÍDA do Rebelle (`Gouache/Gouache Wash`): a ponta quadrada (o corte recto) de lados nítidos
    // cujos dabs SOMAM — as pontas do traço entram e saem a esbater num diâmetro —, pouca tinta, SEM a orla da aguarela,
    // a pressão só na carga, e o tecto do tamanho do Rebelle (o recto de 200 px pinta 56).
    Receita {
        meio: "Watercolor",
        grupo: "Gouache",
        nome: "Gouache Wash",
        water: 1.0,
        opacity: 1.0,
        // 25 → 16: a régua 329 → 337/454 (o véu do Rebelle é mais claro). ⛔ O filme (`strokeFilm`) recusado: a ponta
        // quadrada vira um carimbo de quinas vivas (316/454).
        pigment: 16.0,
        pickup: 1.0,
        gate: 0.4,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            // Seca depressa (como a Round): régua 333 → 363/454, o cruzamento sobre o vermelho molhado sem manchas.
            ("retention", 0.95),
        ],
        grao: None,
        achatado: None,
        pincel: &[("dab_quina", "1.0"), ("dab_flatten", "0.0"), ("size_max_px", "56")],
        nasce_de: Some("Gouache Flat"),
    },
    // J4 H-3: a guache QUASE SECA do Rebelle (`Gouache/Gouache Dry`): o corpo do Wet Dry Brush (pouca água, as cerdas
    // finas, a orla) com metade da tinta, e cada FIO com a sua pressão ao longo do caminho — os fios somem e voltam em
    // troços (o arrasto partido), e a mão leve levanta-os primeiro.
    Receita {
        meio: "Watercolor",
        grupo: "Gouache",
        nome: "Gouache Dry",
        water: 0.2,
        opacity: 1.0,
        // 300 (o do Wet Dry Brush) → 70: régua 281 → 387/454 (o véu claro do Rebelle).
        pigment: 70.0,
        pickup: 1.6,
        gate: 0.4,
        evaporation: 1.5,
        edge: 56.25,
        tilt: DIREITA,
        bristle_size: 1.5,
        bristle_count: 1500.0,
        hardness: 0.0,
        size_px: None,
        extras: &[
            ("bristlePressure", 2.0),
            ("bristleNoiseLength", 60.0),
            ("bristleLaneWidth", 3.0),
            ("bristleLift", 1.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[],
        nasce_de: None,
    },
];

pub const MEIOS: &[ReceitaMeio] = &[];

// As formas (prompt 44, doc 44 §3): a ponta reta e a pressão que não estreita e ESMAECE (prompt 45, doc 45).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Gouache Flat", foto: "gouacheflat", raio: 19.5, metricas: &[(Cobertura, 0.504), (Ponta, 0.083), (Chato, 0.782), (RampaLargura, 0.872), (RampaCobertura, 0.43), (Largura, 39.0), (Pontas, 0.87)] },
];

/// Os reprovados da J4 ([`EM_ESTUDO`]) não têm ficheiro na fábrica, e nenhum tem o nome de um pincel dela.
#[test]
fn os_reprovados_do_guache_nao_estao_na_fabrica() {
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
