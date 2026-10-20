//! **Família `basicos`** — J3, catálogo A (básicos digitais): Hard Round, Soft Round, Pixel 1px, Glaze Round (o Round
//! Pressure e o Flat Block-in foram reprovados — ver o exame).
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//! O exame de cada um: `docs/Painter/Pinceis/exames/basicos.md`.

use super::prelude::*;

pub const WET: &[Receita] = &[];

pub const MEIOS: &[ReceitaMeio] = &[
    // A-1 — o disco duro do Rebelle «Pencil/Digital/Round Hard»: borda de 1–2 px, opaco, o TAMANHO pela pressão
    // (Rebelle: `pressure_size` 1, a curva dele ≈ p^1,2), a opacidade não. Tamanho = o do preset dele (83,9 + os
    // 2 px de orla que ele pinta a mais).
    ReceitaMeio {
        // A cápsula (`continuous_spacing`): sem ela a ponta fina da rampa sai em contas de 1 px (revisão cega, J3). A
        // dureza 0,93 dá o píxel de AA do Rebelle ao tamanho de fábrica e a orla de 2–3 px a 200 (a 0,85 esfumava-a).
        pincel: &[("hardness", "0.93"), ("continuous_spacing", "true")],
        mao: &[("pressureSize", 0.98), ("pressureSizeCurve", 1.2)],
        size_px: Some(86.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Basic", "Hard", "Hard Round")
    },
    // A-2 — o disco macio do Rebelle «Round Soft»: sem borda, a OPACIDADE pela pressão (a curva dele chega a 1 aos
    // 0,8), o tamanho não. O perfil gaussiano dele é a queda `Sharp`; tamanho = o do preset (152,6) × 30/28.
    ReceitaMeio {
        pincel: &[("hardness", "0.0"), ("falloff", "\"Sharp\"")],
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 2.6),
            ("pressureCeiling", 0.8),
        ],
        size_px: Some(163.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Basic", "Soft", "Soft Round")
    },
    // A-3 Round Pressure: REPROVADO na J3 (`docs/Painter/Pinceis/exames/basicos.md`) — a opacidade pela pressão do Krita
    // é por TRAÇO (cada dab no máximo a sua opacidade: a cauda fraca sobe linear ao longo de 200 px); a nossa multiplica a
    // cobertura de cada dab e os dabs SOMAM-se até à Strength — três revisores cegos viram a cauda mais curta.
    // A-4 Flat Block-in: REPROVADO na J3 (`docs/Painter/Pinceis/exames/basicos.md`) — a 5 px o miolo desmaia (0,48 ×
    // 0,87 do Rebelle) e a 200 px a ponta esbate em metade do comprimento; a ponta recta que ele pedia ficou (L-quina).
    // A-5 — o píxel exacto (L18, «pixel-perfect»): binário, alinhado à grelha, sem pressão; 1 px (MyPaint `1pixel`).
    ReceitaMeio {
        pincel: &[("hardness", "1.0"), ("pixel", "2")],
        mao: &[("pressureSize", 0.0)],
        size_px: Some(1.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Basic", "Pixel", "Pixel 1px")
    },
    // A-6 — a velatura (MyPaint `detail_brush_large_glazing`): dabs duros quase transparentes que se SOMAM (o perfil
    // macio é a corda do disco), a tinta sobe devagar onde o traço se cruza e pára num TECTO (L8); entre traços
    // acumula. Tamanho = o do MyPaint à pressão cheia (14 px).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.8"),
            ("accumulate", "true"),
            ("flow", "0.03"),
            (
                "traco",
                r#"{"largura": 7.0, "molhada": 0.0, "queimada": 0.0, "tecto": 0.37}"#,
            ),
        ],
        mao: &[
            ("pressureSize", 0.7),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.3),
        ],
        size_px: Some(14.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Basic", "Glaze", "Glaze Round")
    },
];

// O portão por foto dos meios antigos é o desta família (o `MIDIAS` é a régua do Wet, contra o Wet Round).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[];

/// A identidade de um básico na cena `gestos` da bateria — as contas de `docs/Painter/Pinceis/exames/basicos_foto.py`
/// (a cobertura = máx|c − 255| ÷ (255 − 23), o azul cheio = 1; largura e orla relativas ao pico da coluna).
#[derive(Clone, Copy, Debug)]
struct Identidade {
    /// Píxeis da orla (10–90 % do pico) de cada lado do recto lento: 0 = binário (píxel), ~1 = duro, ≫ = macio.
    borda: f64,
    /// O pico do recto lento (1 = opaco).
    opacidade: f64,
    /// A largura na pressão ~0,3 ÷ na ~0,95 (rampa 0→1): o tamanho pela pressão.
    rampa_largura: f64,
    /// O pico na pressão ~0,3 ÷ na ~0,95: a opacidade pela pressão.
    rampa_cobertura: f64,
    /// O pico dentro da espiral (o traço que se cruza) ÷ a opacidade: o tecto do traço.
    tecto: f64,
}

#[derive(Clone, Copy, Debug)]
enum M {
    Borda,
    Opacidade,
    RampaLargura,
    RampaCobertura,
    Tecto,
}

impl M {
    fn de(self, i: &Identidade) -> f64 {
        match self {
            M::Borda => i.borda,
            M::Opacidade => i.opacidade,
            M::RampaLargura => i.rampa_largura,
            M::RampaCobertura => i.rampa_cobertura,
            M::Tecto => i.tecto,
        }
    }
    /// A tolerância contra a foto (a orla é inteira e escala com o macio).
    fn tol(self, foto: f64) -> f64 {
        match self {
            M::Borda => 0.5 + 0.35 * foto,
            M::Opacidade => 0.06,
            M::RampaLargura | M::RampaCobertura => 0.12,
            M::Tecto => 0.3,
        }
    }
}

const CHEIO: f64 = 255.0 - 23.0;

impl Identidade {
    fn de(px: &[u8], w: usize) -> Self {
        let c = |x: usize, y: usize| {
            let p = &px[(y * w + x) * 4..(y * w + x) * 4 + 3];
            p.iter()
                .map(|&v| (255.0 - f64::from(v)).abs())
                .fold(0.0, f64::max)
                / CHEIO
        };
        let col =
            |x: usize, y0: usize, y1: usize| -> Vec<f64> { (y0..y1).map(|y| c(x, y)).collect() };
        let pico = |v: &[f64]| v.iter().copied().fold(0.0, f64::max);
        let largura = |v: &[f64]| {
            let m = pico(v);
            if m > 0.02 {
                v.iter().filter(|&&a| a >= 0.25 * m).count() as f64
            } else {
                0.0
            }
        };
        let borda = |v: &[f64]| {
            let m = pico(v);
            if m > 0.02 {
                v.iter().filter(|&&a| a > 0.1 * m && a < 0.9 * m).count() as f64 / 2.0
            } else {
                0.0
            }
        };
        let media = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let rec: Vec<Vec<f64>> = (200..=420).step_by(20).map(|x| col(x, 30, 150)).collect();
        let opacidade = media(&rec.iter().map(|v| pico(v)).collect::<Vec<_>>());
        let leve: Vec<Vec<f64>> = [205, 210, 215].map(|x| col(x, 340, 460)).into();
        let forte: Vec<Vec<f64>> = [530, 535, 540].map(|x| col(x, 340, 460)).into();
        let lar = |vs: &[Vec<f64>]| media(&vs.iter().map(|v| largura(v)).collect::<Vec<_>>());
        let pic = |vs: &[Vec<f64>]| media(&vs.iter().map(|v| pico(v)).collect::<Vec<_>>());
        let espiral = (170..374)
            .flat_map(|y| (700..921).map(move |x| (x, y)))
            .map(|(x, y)| c(x, y))
            .fold(0.0, f64::max);
        Self {
            borda: media(&rec.iter().map(|v| borda(v)).collect::<Vec<_>>()),
            opacidade,
            rampa_largura: lar(&leve) / lar(&forte).max(1.0),
            rampa_cobertura: pic(&leve) / pic(&forte).max(1e-6),
            tecto: espiral / opacidade.max(1e-6),
        }
    }
}

/// As FOTOS dos oráculos na cena `gestos` (A), medidas por `basicos_foto.py` (20/10): o Rebelle onde existe; e a
/// ESCALA a que o nosso se pinta (a largura do recto lento do oráculo ÷ a nossa, a do `exame.sh`; o Hard Round ao
/// tamanho de fábrica, 86 px: a orla dele é relativa ao raio e a do Rebelle é 1 px em todos — medido a 40 e a 200).
#[rustfmt::skip]
const FOTOS: [(&str, &str, f64, Identidade); 4] = [
    ("Hard Round", "rebelle PENCIL/digital_round_hard", 2.1, Identidade { borda: 1.0, opacidade: 1.0, rampa_largura: 0.228, rampa_cobertura: 1.0, tecto: 1.0 }),
    ("Soft Round", "rebelle PENCIL/digital_round_soft", 1.071, Identidade { borda: 9.75, opacidade: 0.994, rampa_largura: 0.878, rampa_cobertura: 0.228, tecto: 1.007 }),
    ("Pixel 1px", "mypaint experimental/1pixel", 1.0, Identidade { borda: 0.0, opacidade: 1.0, rampa_largura: 1.0, rampa_cobertura: 1.0, tecto: 1.0 }),
    ("Glaze Round", "mypaint deevad/detail_brush_large_glazing", 2.1, Identidade { borda: 22.5, opacidade: 0.231, rampa_largura: 0.591, rampa_cobertura: 0.235, tecto: 1.85 }),
];

/// O que faz a identidade de cada um.
const IDENTIDADE: [(&str, &[M]); 4] = [
    (
        "Hard Round",
        &[M::Borda, M::RampaLargura, M::RampaCobertura],
    ),
    (
        "Soft Round",
        &[M::Borda, M::RampaLargura, M::RampaCobertura],
    ),
    ("Pixel 1px", &[M::Borda, M::RampaLargura, M::RampaCobertura]),
    (
        "Glaze Round",
        &[M::Opacidade, M::Tecto, M::RampaCobertura, M::Borda],
    ),
];

/// ⭐ **Cada básico reconhece-se pela foto do seu oráculo** (J3, `docs/Painter/Pinceis/exames/basicos.md`): pintado na
/// cena `gestos` da bateria, cada medida da sua identidade cai na tolerância da foto, e — a lei dos IRMÃOS — o pincel
/// fica estritamente mais perto (em tolerâncias somadas) da SUA foto que da de cada um dos outros três: o Hard Round
/// sem a pressão no tamanho (é o píxel), o píxel com AA (é o Hard), o Soft com borda (é o Hard), a velatura opaca.
/// Mutação (J3): o Round Pressure sem a opacidade pela pressão sangrou (rampa 1,00 × 0,267; «pinta o Hard Round»).
#[test]
fn cada_basico_se_reconhece_pela_foto() {
    use crate::exame_bancada::{bateria, pinceis, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let gestos = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "gestos")
        .unwrap();
    let medidas: Vec<Identidade> = std::thread::scope(|s| {
        let hs: Vec<_> = FOTOS
            .iter()
            .map(|&(nome, _, escala, _)| {
                // A cena à escala do oráculo: cada `size_px` × escala.
                let mut cena = gestos.clone();
                for op in cena["ops"].as_array_mut().unwrap() {
                    if let Some(d) = op.get("size_px").and_then(serde_json::Value::as_f64) {
                        op["size_px"] = serde_json::json!(d * escala);
                    }
                }
                s.spawn(move || {
                    let p = pinceis(nome).remove(0);
                    Identidade::de(
                        &pinta_cena(&p, None, &cena, tela, papel, 0).1,
                        tela[0] as usize,
                    )
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for ((nome, oraculo, _, foto), (n2, ms)) in FOTOS.iter().zip(IDENTIDADE) {
        assert_eq!(nome, &n2);
        let eu = &medidas[FOTOS.iter().position(|f| f.0 == *nome).unwrap()];
        println!("[basicos] {nome}: {eu:?}\n          foto {oraculo}: {foto:?}");
        for &m in ms {
            let (v, f) = (m.de(eu), m.de(foto));
            if (v - f).abs() > m.tol(f) {
                falhas.push(format!(
                    "{nome}: {m:?} {v:.3} fora da foto {f:.3} ± {:.3} ({oraculo})",
                    m.tol(f)
                ));
            }
        }
        let distancia = |outra: &Identidade| -> f64 {
            ms.iter()
                .map(|&m| (m.de(eu) - m.de(outra)).abs() / m.tol(m.de(foto)))
                .sum()
        };
        let propria = distancia(foto);
        for (outro, _, _, f2) in FOTOS.iter().filter(|f| f.0 != *nome) {
            let d = distancia(f2);
            if propria >= d {
                falhas.push(format!("{nome}: {propria:.2} tolerâncias da sua foto, {d:.2} da do {outro} — pinta o irmão"));
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Um dab só do `Hard Round` com as variações `campos` no pincel, a `diametro` px no meio da tela: a cobertura.
fn um_dab(campos: &[(&str, serde_json::Value)], diametro: f64) -> (Vec<f64>, usize) {
    use crate::exame_bancada::{Fonte, Pincel, pinta_cena};
    let mut p = pinceis_de("Hard Round");
    for (c, v) in campos {
        p.parametros.brush.insert((*c).to_owned(), v.clone());
    }
    let pincel = Pincel {
        nome: "sonda".to_owned(),
        fonte: Fonte::Preset {
            preset: Box::new(p),
            relativo: String::new(),
        },
    };
    let cena = serde_json::json!({"nome": "ponto", "ops": [
        {"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": [23, 101, 179]},
        {"op": "traco", "nome": "ponto", "size_px": diametro, "pts": [[500.0, 320.0, 1.0], [500.5, 320.0, 1.0]]},
        {"op": "foto", "nome": "A"}]});
    let px = pinta_cena(&pincel, None, &cena, [1000, 640], [255, 255, 255], 0).1;
    let c = px
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            p[..3]
                .iter()
                .map(|&v| 255.0 - f64::from(v))
                .fold(0.0, f64::max)
                / CHEIO
        })
        .collect();
    (c, 1000)
}

fn pinceis_de(nome: &str) -> ph2d_brush_presets::Preset {
    crate::midias_de_fabrica::de_fabrica(nome)
}

/// ⭐ **A ponta RETA pinta um RECTÂNGULO no Digital** (J3, A-4): a quina (`dab_quina` 1, prompt 44) chegava ao pincel
/// mas o kernel de faixas da CPU (`ph2d-painter-brush/src/dab/bands.rs`) media a orla pela norma do disco — o Flat
/// Block-in saía com as pontas redondas (o da lâmina do Rebelle é recto). Um dab de 200 px com quina 1 e sem
/// achatar: o canto a 0,85·r (no disco, a 1,2·r — fora) é tinta; sem a quina continua papel (o disco de sempre).
#[test]
fn a_ponta_reta_do_digital_pinta_um_rectangulo() {
    use serde_json::json;
    let base = [
        ("dab_flatten", json!(0.0)),
        ("dab_angle_deg", json!(0)),
        ("dab_quina", json!(1.0)),
        ("hardness", json!(0.95)),
        ("continuous_spacing", json!(false)),
        ("accumulate", json!(false)),
        ("flow", json!(1.0)),
    ];
    let canto = |campos: &[(&str, serde_json::Value)]| {
        let (c, w) = um_dab(campos, 200.0);
        let at = |x: usize, y: usize| c[y * w + x];
        // r = 100: os quatro cantos a 0,85·r em cada eixo, e o centro.
        (
            [(585, 405), (415, 405), (585, 235), (415, 235)].map(|(x, y)| at(x, y)),
            at(500, 320),
        )
    };
    let (quadrado, centro) = canto(&base);
    let mut sem: Vec<_> = base.to_vec();
    sem.push(("dab_quina", json!(0.0)));
    let (disco, _) = canto(&sem);
    println!(
        "[basicos] quina 1: cantos {quadrado:?} centro {centro:.2} · quina 0: cantos {disco:?}"
    );
    assert!(centro > 0.95, "o dab não pintou ({centro:.2})");
    assert!(
        quadrado.iter().all(|&v| v > 0.9),
        "a quina 1 não chega ao canto: {quadrado:?} — o Digital pinta um disco"
    );
    assert!(
        disco.iter().all(|&v| v < 0.05),
        "sem a quina o canto devia ser papel: {disco:?}"
    );
}

/// ⭐ **O Pixel 1px é o ÚNICO básico que pinta o píxel exacto** (J3, A-5): a matriz e a revisão cega comparam à
/// largura de 40 px, onde uma ponta de píxel é um disco duro — a identidade dele mora no 1 px. Uma recta inclinada
/// (dy/dx = 0,16) a 1 px: o Pixel 1px deixa ZERO píxeis parciais e UM píxel por coluna (pixel-perfect, sem quinas
/// em L); o Hard Round e o Soft Round ao mesmo tamanho deixam orla anti-aliased e colunas de 0 ou 2 píxeis.
#[test]
fn o_pixel_1px_e_o_unico_que_pinta_o_pixel_exacto() {
    use crate::exame_bancada::{pinceis, pinta_cena};
    let pts: Vec<serde_json::Value> = (0..=100)
        .map(|i| {
            let x = 100.0 + 5.0 * f64::from(i);
            serde_json::json!([x, 100.0 + 0.16 * (x - 100.0), 1.0])
        })
        .collect();
    let cena = serde_json::json!({"nome": "linha", "ops": [
        {"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": [23, 101, 179]},
        {"op": "traco", "nome": "linha", "size_px": 1.0, "pts": pts}, {"op": "foto", "nome": "A"}]});
    let mede = |nome: &str| {
        let px = pinta_cena(
            &pinceis(nome).remove(0),
            None,
            &cena,
            [1000, 640],
            [255, 255, 255],
            0,
        )
        .1;
        let c = |x: usize, y: usize| {
            px[(y * 1000 + x) * 4..(y * 1000 + x) * 4 + 3]
                .iter()
                .map(|&v| 255.0 - f64::from(v))
                .fold(0.0, f64::max)
                / CHEIO
        };
        let (mut parciais, mut colunas_de_um) = (0, 0);
        for x in 120..580 {
            let col: Vec<f64> = (80..200).map(|y| c(x, y)).collect();
            parciais += col.iter().filter(|&&v| v > 0.05 && v < 0.95).count();
            colunas_de_um += usize::from(col.iter().filter(|&&v| v >= 0.95).count() == 1);
        }
        (parciais, colunas_de_um)
    };
    let (p_px, um_px) = mede("Pixel 1px");
    println!("[basicos] Pixel 1px: {p_px} parciais, {um_px}/460 colunas com um píxel");
    assert_eq!(p_px, 0, "a ponta de píxel deixou {p_px} píxeis parciais");
    assert_eq!(
        um_px, 460,
        "a linha de 1 px tem colunas com 0 ou 2 píxeis (quinas em L): {um_px}/460"
    );
    for outro in ["Hard Round", "Soft Round"] {
        let (p, um) = mede(outro);
        println!("[basicos] {outro} a 1 px: {p} parciais, {um}/460 colunas com um píxel");
        assert!(
            p > 0 && um < 440,
            "{outro} a 1 px pinta o píxel exacto ({p} parciais, {um}/460): o Pixel 1px não seria distinto"
        );
    }
}
