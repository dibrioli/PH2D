//! **Família `basicos`** — J3, catálogo A (básicos digitais): Hard Round, Soft Round, Flat Block-in, Pixel 1px, Glaze
//! Round (o Flat Block-in entrou na 2.ª onda, com o motor; o Round Pressure continua reprovado — ver o exame).
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//! O exame de cada um: `docs/Painter/Pinceis/exames/basicos.md`.

use super::prelude::*;

pub const WET: &[Receita] = &[];

pub const MEIOS: &[ReceitaMeio] = &[
    // A-1 — o disco duro do Rebelle «Pencil/Digital/Round Hard»: borda de 1–2 px, opaco, o TAMANHO pela pressão
    // (Rebelle: `pressure_size` 1, a curva dele ≈ p^1,2), a opacidade não. Tamanho = o do preset dele (83,87): com a
    // orla centrada a largura medida casa a 40 e a 200 à escala 1.
    ReceitaMeio {
        // A cápsula (`continuous_spacing`): sem ela a ponta fina da rampa sai em contas de 1 px (revisão cega, J3). A
        // orla de 1 px FIXA e centrada na borda (`borda_px`, dureza 1; J3 2.ª onda) é o píxel de AA do Rebelle a 40 e a
        // 200 (0,37 · 0,22 → 0,95) e dá à linha de 5 px a área do disco; a dureza 0,93 relativa ao raio deixava um aro de
        // 2–3 px a 200 (0,15 · 0,59 · 0,91) e meio píxel de tinta a menos de cada lado a 5.
        pincel: &[
            ("hardness", "1.0"),
            ("borda_px", "1.0"),
            ("continuous_spacing", "true"),
        ],
        // A curva 1,4 (J3 2.ª onda): com a orla centrada a 1,2 engordava o terço fraco da rampa (2,4 × 1,9 px do
        // Rebelle, revisão cega r2); a 1,4 a largura casa a dele coluna a coluna (2 · 2 · 4 · 6 · 8 · 10 · 12 · 14).
        mao: &[("pressureSize", 0.98), ("pressureSizeCurve", 1.4)],
        size_px: Some(84.0),
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
    // A-3 Round Pressure: REPROVADO de novo na 2.ª onda (`docs/Painter/Pinceis/exames/basicos.md`). O motor que ele pedia
    // existe (`tecto_por_dab`: a rampa do Krita sobe desde o 1.º píxel, sem somar), mas (1) a matriz põe-no a 0,286 do
    // Hard Round (degrau < 0,40) e a 0,30 do Studio Pen, e o revisor cego não separou o Hard Round dele à pressão
    // cheia; (2) a curva de potência + tecto da mão faz um JOELHO aos 0,86 que o Krita (curva em S) não tem.
    // A-4 — a lâmina do Rebelle «Pencil/Digital/Flat»: FIXA na tela (L 0,98·D × T 0,4·D, o eixo longo de pé: larga no
    // horizontal, fina no vertical), a ponta quase recta, dabs a 0,28 que ACUMULAM (as pontas esbatem) e o sub-passo
    // FINO (J3 2.ª onda): a 5 px a lâmina de 2 px enche como a de 16 a 40 (0,48 → 0,99; o Rebelle 0,95), e a orla de
    // 1 px no mínimo dá-lhe o AA de área (0,85 · 1 · 1 · 1 · 1 · 0,85 contra 0,75 · 0,95… · 0,75). Sem pressão.
    // Tamanho = o do preset (35,1) × 0,995 (a largura medida igual a 40 e a 200 px).
    ReceitaMeio {
        pincel: &[
            // A queda relativa do Rebelle (2 px a 40, 7 a 200; a 0,85 saía 1 e 5 — a revisão cega r3 viu a borda seca).
            ("hardness", "0.8"),
            ("borda_px", "1.0"),
            ("dab_flatten", "0.6"),
            ("dab_angle_deg", "90"),
            ("dab_quina", "0.9"),
            ("accumulate", "true"),
            ("flow", "0.28"),
            ("continuous_spacing", "true"),
            ("sub_passo_fino", "true"),
        ],
        mao: &[("pressureSize", 0.0)],
        size_px: Some(35.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Basic", "Flat", "Flat Block-in")
    },
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
    /// A largura do recto lento ÷ a do vertical: a lâmina FIXA na tela (1 = redondo).
    chato: f64,
}

#[derive(Clone, Copy, Debug)]
enum M {
    Borda,
    Opacidade,
    RampaLargura,
    RampaCobertura,
    Tecto,
    Chato,
}

impl M {
    fn de(self, i: &Identidade) -> f64 {
        match self {
            M::Borda => i.borda,
            M::Opacidade => i.opacidade,
            M::RampaLargura => i.rampa_largura,
            M::RampaCobertura => i.rampa_cobertura,
            M::Tecto => i.tecto,
            M::Chato => i.chato,
        }
    }
    /// A tolerância contra a foto (a orla é inteira e escala com o macio).
    fn tol(self, foto: f64) -> f64 {
        match self {
            M::Borda => 0.5 + 0.35 * foto,
            M::Opacidade => 0.06,
            M::RampaLargura | M::RampaCobertura => 0.12,
            M::Tecto | M::Chato => 0.3,
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
        let ver = media(
            &[500, 525, 550].map(|y| largura(&(640..761).map(|x| c(x, y)).collect::<Vec<_>>())),
        );
        let hor = media(&rec.iter().map(|v| largura(v)).collect::<Vec<_>>());
        let espiral = (170..374)
            .flat_map(|y| (700..921).map(move |x| (x, y)))
            .map(|(x, y)| c(x, y))
            .fold(0.0, f64::max);
        // A orla em DUAS fases da grelha: o recto lento (o centro num texel inteiro) e o topo da curva em S (y ≈ 195,9):
        // numa borda que cai entre dois texels o AA de área de um disco duro é binário (J3 2.ª onda).
        let topo_s: Vec<Vec<f64>> = (295..=325).step_by(5).map(|x| col(x, 150, 250)).collect();
        let orla = |vs: &[Vec<f64>]| media(&vs.iter().map(|v| borda(v)).collect::<Vec<_>>());
        Self {
            borda: 0.5 * (orla(&rec) + orla(&topo_s)),
            opacidade,
            rampa_largura: lar(&leve) / lar(&forte).max(1.0),
            rampa_cobertura: pic(&leve) / pic(&forte).max(1e-6),
            tecto: espiral / opacidade.max(1e-6),
            chato: hor / ver.max(1.0),
        }
    }
}

/// As FOTOS dos oráculos na cena `gestos` (A), medidas por `basicos_foto.py` (20/10): o Rebelle onde existe; e a
/// ESCALA a que o nosso se pinta (a largura do recto lento do oráculo ÷ a nossa, a do `exame.sh`; o Hard Round ao
/// tamanho do preset do Rebelle, 83,87 px: numa borda alinhada à grelha o AA de área é binário, e a dele cai fora).
#[rustfmt::skip]
const FOTOS: [(&str, &str, f64, Identidade); 5] = [
    ("Hard Round", "rebelle PENCIL/digital_round_hard", 2.097, Identidade { borda: 1.179, opacidade: 1.0, rampa_largura: 0.228, rampa_cobertura: 1.0, tecto: 1.0, chato: 1.0 }),
    ("Soft Round", "rebelle PENCIL/digital_round_soft", 1.071, Identidade { borda: 9.839, opacidade: 0.994, rampa_largura: 0.878, rampa_cobertura: 0.228, tecto: 1.007, chato: 1.0 }),
    ("Flat Block-in", "rebelle PENCIL/digital_flat", 0.995, Identidade { borda: 2.077, opacidade: 0.996, rampa_largura: 1.0, rampa_cobertura: 1.0, tecto: 1.004, chato: 2.294 }),
    ("Pixel 1px", "mypaint experimental/1pixel", 1.0, Identidade { borda: 0.0, opacidade: 1.0, rampa_largura: 1.0, rampa_cobertura: 1.0, tecto: 1.0, chato: 1.0 }),
    ("Glaze Round", "mypaint deevad/detail_brush_large_glazing", 2.1, Identidade { borda: 23.821, opacidade: 0.231, rampa_largura: 0.591, rampa_cobertura: 0.235, tecto: 1.85, chato: 1.0 }),
];

/// O que faz a identidade de cada um.
const IDENTIDADE: [(&str, &[M]); 5] = [
    (
        "Hard Round",
        &[M::Borda, M::RampaLargura, M::RampaCobertura],
    ),
    (
        "Soft Round",
        &[M::Borda, M::RampaLargura, M::RampaCobertura],
    ),
    (
        "Flat Block-in",
        &[M::Chato, M::Borda, M::RampaLargura, M::Opacidade],
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

/// O `Hard Round` com as variações `campos` (o pincel) e `mao` (as linhas da caneta), a pintar os `tracos` numa
/// tela 1000×640 branca: a cobertura de cada píxel (o azul cheio = 1).
fn sonda(
    campos: &[(&str, serde_json::Value)],
    mao: &[(&str, f64)],
    tracos: &[serde_json::Value],
) -> Vec<f64> {
    use crate::exame_bancada::{Fonte, Pincel, pinta_cena};
    use serde_json::json;
    let mut p = pinceis_de("Hard Round");
    for (c, v) in campos {
        p.parametros.brush.insert((*c).to_owned(), v.clone());
    }
    for (c, v) in mao {
        p.parametros.engine.insert((*c).to_owned(), *v);
    }
    let pincel = Pincel {
        nome: "sonda".to_owned(),
        fonte: Fonte::Preset {
            preset: Box::new(p),
            relativo: String::new(),
        },
    };
    let mut ops = vec![
        json!({"op": "pincel", "quem": "preset"}),
        json!({"op": "cor", "rgb": [23, 101, 179]}),
    ];
    ops.extend(tracos.iter().cloned());
    ops.push(json!({"op": "foto", "nome": "A"}));
    let cena = json!({"nome": "sonda", "ops": ops});
    let px = pinta_cena(&pincel, None, &cena, [1000, 640], [255, 255, 255], 0).1;
    px.as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            p[..3]
                .iter()
                .map(|&v| 255.0 - f64::from(v))
                .fold(0.0, f64::max)
                / CHEIO
        })
        .collect()
}

/// Um recto horizontal de `d` px a `y`, de x 100 a 900 a cada 3 px (o passo do recto da bateria), à pressão `p`.
fn recto(d: f64, y: f64, p: f64) -> serde_json::Value {
    let pts: Vec<serde_json::Value> = (0..=266)
        .map(|i| serde_json::json!([100.0 + 3.0 * f64::from(i), y, p]))
        .collect();
    serde_json::json!({"op": "traco", "nome": "recto", "size_px": d, "pts": pts})
}

/// A coluna `x` da cobertura `c` (tela de 1000) entre `y0..y1`.
fn coluna(c: &[f64], x: usize, y0: usize, y1: usize) -> Vec<f64> {
    (y0..y1).map(|y| c[y * 1000 + x]).collect()
}

fn pico(v: &[f64]) -> f64 {
    v.iter().copied().fold(0.0, f64::max)
}

/// ⭐ **O tecto POR DAB** (J3 2.ª onda, A-3; `TracoComposto::tecto_por_dab`): com a opacidade pela pressão e o
/// Accumulate desligado, cada texel pára na opacidade do dab que o cobre — a lei do Krita (`Basic-5 Size Opacity`:
/// a rampa 0→1 sobe com a pressão, sem somar). Um recto de 40 px a pressão constante 0,4 (a curva 1): o pico é a
/// opacidade de UM dab (0,4); sem o tecto os dabs somam-se até ~1 (a Strength 1 não tinha tecto).
#[test]
fn o_tecto_por_dab_para_na_opacidade_da_mao() {
    use serde_json::json;
    let mao = [
        ("pressureSize", 0.0),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 1.0),
        ("pressureCeiling", 1.0),
    ];
    let mede = |tecto: bool| {
        let c = sonda(
            // Sem a orla em píxeis do Hard Round: ela também arma o tecto do traço (a área da orla).
            &[
                ("traco", json!({"tecto_por_dab": tecto})),
                ("borda_px", json!(0.0)),
            ],
            &mao,
            &[recto(40.0, 320.0, 0.4)],
        );
        pico(&coluna(&c, 500, 280, 360))
    };
    let (com, sem) = (mede(true), mede(false));
    println!("[basicos] tecto por dab a p 0,4: pico {com:.3} · sem ele {sem:.3}");
    assert!(
        (com - 0.4).abs() <= 0.04,
        "com o tecto por dab o pico é {com:.3}, não a opacidade de um dab (0,4)"
    );
    assert!(
        sem > 0.8,
        "o controlo: sem o tecto os dabs somavam ({sem:.3})"
    );
}

/// ⭐ **O sub-passo FINO acumula a lâmina fina como a larga** (J3 2.ª onda, A-4; `BrushSpec::sub_passo_fino`): a
/// lâmina do Flat (0,4·D de espessura, Accumulate a 0,28) leva o mesmo número de passagens por texel a 5 e a 40 px —
/// o Rebelle pinta-a cheia nos dois (0,95 × 1,00). Sem ele o piso de um texel do sub-passo deixa a de 5 px em DUAS
/// passagens (0,48).
#[test]
fn o_sub_passo_fino_acumula_a_lamina_fina_como_a_larga() {
    use serde_json::json;
    let mede = |fino: bool| {
        let c = sonda(
            &[
                ("dab_flatten", json!(0.6)),
                ("dab_angle_deg", json!(90)),
                ("dab_quina", json!(1.0)),
                ("hardness", json!(0.85)),
                ("accumulate", json!(true)),
                ("flow", json!(0.28)),
                ("continuous_spacing", json!(true)),
                ("sub_passo_fino", json!(fino)),
                ("borda_px", json!(0.0)),
            ],
            &[("pressureSize", 0.0)],
            &[recto(5.0, 100.0, 1.0), recto(40.0, 300.0, 1.0)],
        );
        (
            pico(&coluna(&c, 500, 90, 111)),
            pico(&coluna(&c, 500, 270, 331)),
        )
    };
    let ((c5, c40), (s5, s40)) = (mede(true), mede(false));
    println!("[basicos] lâmina 5 × 40 px: fino {c5:.3} × {c40:.3} · sem {s5:.3} × {s40:.3}");
    assert!(
        c5 >= 0.9 && (c5 - c40).abs() <= 0.06,
        "com o sub-passo fino a lâmina de 5 px pinta {c5:.3} e a de 40 {c40:.3}"
    );
    assert!(
        s5 < 0.6,
        "o controlo: sem ele a de 5 px desmaiava ({s5:.3})"
    );
    assert!(
        (s40 - c40).abs() < 1e-9,
        "a 40 px o sub-passo é o mesmo: {s40} × {c40}"
    );
}

/// A orla (10–90 % do pico) de cada lado de uma coluna, px.
fn orla(v: &[f64]) -> f64 {
    let m = pico(v);
    v.iter().filter(|&&a| a > 0.1 * m && a < 0.9 * m).count() as f64 / 2.0
}

/// ⭐ **A orla em PÍXEIS é a mesma a 40 e a 200 px** (J3 2.ª onda, A-1; `BrushSpec::borda_px`): o disco duro do
/// Rebelle tem UM píxel de AA a qualquer tamanho (a 200 px: 0,22 → 0,95). A dureza relativa ao raio dava ao Hard
/// Round um aro de 2–3 px a 200; com a orla de 1 px a queda corre sobre o mesmo píxel nos dois.
#[test]
fn a_orla_em_pixeis_e_a_mesma_a_40_e_a_200() {
    use serde_json::json;
    // `None` = o Hard Round de fábrica; senão a orla e a dureza.
    let mede = |arma: Option<(f64, f64)>| {
        let campos: Vec<_> = arma
            .map(|(b, h)| [("borda_px", json!(b)), ("hardness", json!(h))])
            .into_iter()
            .flatten()
            .collect();
        let c = sonda(
            &campos,
            &[],
            &[
                // A borda fora da grelha (+0,3 px): numa borda que cai entre dois texels o AA de área é binário. A
                // linha de 5 px ALINHADA: é aí que a orla para dentro perde meio texel de cada lado.
                recto(40.0, 100.3, 1.0),
                recto(200.0, 380.3, 1.0),
                recto(5.0, 560.0, 1.0),
            ],
        );
        // A tinta de uma coluna do recto de 5 px (px de azul cheio): a ÁREA do disco é 5.
        let tinta = [400, 450, 500, 550, 600]
            .map(|x| coluna(&c, x, 550, 571).iter().sum::<f64>())
            .iter()
            .sum::<f64>()
            / 5.0;
        let media = |y0, y1| {
            [400, 450, 500, 550, 600]
                .map(|x| orla(&coluna(&c, x, y0, y1)))
                .iter()
                .sum::<f64>()
                / 5.0
        };
        (media(70, 131), media(270, 491), tinta)
    };
    // O controlo é o Hard Round da 1.ª onda: a dureza 0,93 relativa ao raio, sem a orla.
    let ((o40, o200, o5), (s40, s200, s5), (f40, f200, f5)) =
        (mede(Some((1.0, 1.0))), mede(Some((0.0, 0.93))), mede(None));
    println!(
        "[basicos] orla 40 × 200 px (tinta a 5 px): borda 1 px {o40:.1} × {o200:.1} ({o5:.2}) · sem {s40:.1} × {s200:.1} ({s5:.2}) · o de fábrica {f40:.1} × {f200:.1} ({f5:.2})"
    );
    // A caixa do AA CENTRADA na borda: a linha de 5 px leva a área do disco (o Rebelle 5,2; sem ela 4 — meio píxel
    // perdido de cada lado).
    assert!(
        (o5 - 5.0).abs() <= 0.4 && (f5 - 5.0).abs() <= 0.4,
        "a tinta da linha de 5 px: {o5:.2} com a orla, {f5:.2} a de fábrica — não é a área (5)"
    );
    assert!(
        s5 < 4.5,
        "o controlo: sem a orla centrada a de 5 px perdia tinta ({s5:.2})"
    );
    // O Hard Round de fábrica É esta cura (`borda_px` 1,3): o mesmo píxel de AA nos dois tamanhos.
    assert!(
        f40 <= 1.0 && f200 <= 1.0,
        "o Hard Round de fábrica: {f40:.1} px a 40, {f200:.1} a 200 — voltou o aro"
    );
    // Um píxel PARCIAL de cada lado (o AA, nem binário nem aro) — os dabs que se sobrepõem não o enchem.
    assert!(
        (0.9..=1.0).contains(&o40) && (0.9..=1.0).contains(&o200),
        "com a orla de 1 px: {o40:.1} px a 40, {o200:.1} a 200"
    );
    assert!(
        s200 >= 1.75,
        "o controlo: a dureza relativa dava {s200:.1} px a 200"
    );
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
