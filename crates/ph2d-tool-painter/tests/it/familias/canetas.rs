//! **Família `canetas`** — J3, catálogo C (as canetas digitais). Aprovadas: a Mapping Pen (C-5) e a Ballpoint (C-7, J4 efeitos).
//! O Studio Pen (C-4), o Technical Pen (C-6) e o Dry Ink (C-11) foram REPROVADOS no exame e o Brush Pen (C-10) fundiu-se
//! no G-pen — o porquê, as receitas medidas e o que falta ao motor: `docs/Painter/Pinceis/exames/canetas.md`.
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//!
//! No Digital (o carimbo do `ph2d-painter-brush`). O tamanho de fábrica é o do preset de ORIGEM (o ficheiro do Krita,
//! CC-0).

use super::prelude::*;

pub const WET: &[Receita] = &[];

pub const MEIOS: &[ReceitaMeio] = &[
    // C-5 — o fio que a pressão não muda e perfil gaussiano (Krita Ink-1 Precision, CC-0: o tamanho pela VELOCIDADE,
    // `1 → 0,526`): o perfil do traço é o do dab (Pow4) com o passo SOLTO (a cápsula satura a orla e endurece-a), e o
    // traço rápido sai ~5 % mais fino (L12, a velocidade da mão: o Krita 0,97 no recto, 0,93 no vertical); o início
    // afina um pouco (o Krita, medido: 0,82 → 0,91 → 1 da largura nos primeiros ~8 px do recto lento).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.16"),
            ("falloff", r#""Pow4""#),
            ("spacing", "0.08"),
            ("taper", r#"{"start":0.2,"tip_start":0.8,"opacity":0.0}"#),
        ],
        mao: &[("pressureSize", 0.0), ("speedSize", 0.025)],
        size_px: Some(3.0),
        opacity: Some(0.99),
        ..ReceitaMeio::digital("Ink", "Mapping", "Mapping Pen")
    },
    // APROVADA na J4 (`exames/efeitos.md` 2.ª ronda): a mistura ESPECTRAL do MyPaint no cruzamento e a dureza pela
    // pressão a 200 px. C-7 — a esferográfica (MyPaint `deevad/ballpen`, CC0): translúcida e macia, a tinta e o tamanho pela pressão, e
    // no traço LEVE a tinta não fura — esbate em fiapos; o traço compõe-se UMA vez (o plano) com o pigmento, que sobre
    // o papel é o over e sobre outra tinta escurece (`exames/canetas.md` 2.ª onda).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.0"),
            ("falloff", r#""Sharp""#),
            ("spacing", "0.16"),
            ("pigment", "true"),
            ("pigment_mix", "1.0"),
            // A lei do MyPaint 2 (J4 efeitos, `exames/efeitos.md` 2.ª ronda): o azul sobre o vermelho em violeta-cinza.
            ("pigment_espectral", "true"),
            (
                "traco",
                r#"{"largura":7.0,"molhada":0.0,"queimada":0.0,"tecto":0.99}"#,
            ),
        ],
        // A dureza pela pressão (J4 efeitos, o `hardness 0,2 + 0,3·p` do `deevad/ballpen`; 0,15 na nossa queda —
        // a 0,3 o traço engordava 9 %): o cheio de 200 px não sai em contas (0,044 → 0,006), o leve continua em fiapos.
        mao: &[
            ("pressureSize", 0.25),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.2),
            ("pressureJitter", 0.35),
            ("pressureHardness", 0.15),
        ],
        size_px: Some(5.0),
        opacity: Some(0.7),
        ..ReceitaMeio::digital("Ink", "Ballpoint", "Ballpoint")
    },
];

/// As receitas EM ESTUDO — medidas, com o portão, mas fora da fábrica (REPROVADAS no exame; o porquê em
/// `docs/Painter/Pinceis/exames/canetas.md`).
pub const EM_ESTUDO: &[ReceitaMeio] = &[];

#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[];

// ---- O portão da família (J3, `docs/Painter/Pinceis/exames/canetas.md`) ----

/// A identidade de uma caneta na cena `gestos` da bateria — as MESMAS contas de
/// `docs/Painter/Pinceis/exames/canetas_foto.py` (a cobertura = máx|c − 255| ÷ (255 − 23), o azul cheio = 1).
#[derive(Clone, Copy, Debug)]
struct Identidade {
    /// Píxeis da orla (10–90 % do pico) de cada lado do recto lento: ~1 = duro, ≫ = macio.
    borda: f64,
    /// O pico do recto lento (1 = opaco).
    opacidade: f64,
    /// A largura na pressão ~0,15 ÷ na ~0,95 (rampa 0→1): o fio do traço leve.
    rampa_fio: f64,
    /// A largura na pressão ~0,5 ÷ na ~0,95: a forma da curva (em S no G-pen, recta no fineliner).
    rampa_meio: f64,
    /// A largura na pressão ~0,3 ÷ na ~0,95: o pé do S (o G-pen ainda fino; a recta já a meio caminho).
    rampa_baixa: f64,
    /// A largura na pressão ~0,7 ÷ na ~0,95: o ombro do S (o G-pen já quase cheio; um tecto em quina não).
    rampa_alta: f64,
    /// |Δ| médio, entre colunas vizinhas, da linha de cima do recto lento a ½ do pico: a borda rasgada.
    rugosidade: f64,
    /// O pico médio da rampa a pressão 0,05–0,25 ÷ a opacidade: a tinta que o traço leve deixa.
    leve: f64,
    /// No traço leve (0,05–0,25): |Δ| médio entre colunas vizinhas ÷ a tinta média — furos nítidos ≫, fiapos borrados ≪.
    aspereza_leve: f64,
    /// O σ vertical da tinta no traço leve ÷ o do cheio (0,88–0,97): a pressão leve que ESPALHA a posição.
    espalha_leve: f64,
    /// No traço leve, o pico de cada coluna menos a recta que o ajusta, σ ÷ média: os BORRÕES (os fiapos da
    /// esferográfica; liso ≈ 0).
    borrao: f64,
}

#[derive(Clone, Copy, Debug)]
enum M {
    Borda,
    Opacidade,
    RampaFio,
    RampaMeio,
    RampaBaixa,
    RampaAlta,
    Rugosidade,
    Leve,
    AsperezaLeve,
    EspalhaLeve,
    Borrao,
}

impl M {
    fn de(self, i: &Identidade) -> f64 {
        match self {
            M::Borda => i.borda,
            M::Opacidade => i.opacidade,
            M::RampaFio => i.rampa_fio,
            M::RampaMeio => i.rampa_meio,
            M::RampaBaixa => i.rampa_baixa,
            M::RampaAlta => i.rampa_alta,
            M::Rugosidade => i.rugosidade,
            M::Leve => i.leve,
            M::AsperezaLeve => i.aspereza_leve,
            M::EspalhaLeve => i.espalha_leve,
            M::Borrao => i.borrao,
        }
    }
    /// A tolerância contra a foto (a orla é inteira e escala com o macio).
    fn tol(self, foto: f64) -> f64 {
        match self {
            M::Borda => 0.5 + 0.35 * foto,
            M::Opacidade => 0.06,
            M::RampaFio | M::RampaMeio => 0.12,
            // A forma da curva: o pé e o ombro do S (os «joelhos» que a revisão cega viu passavam a 0,12).
            M::RampaBaixa | M::RampaAlta => 0.05,
            M::Rugosidade => 0.1 + 0.3 * foto,
            M::Leve => 0.15,
            M::AsperezaLeve => 0.05,
            M::EspalhaLeve => 0.12,
            M::Borrao => 0.1,
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
        let rampa = |xs: [usize; 3]| media(&xs.map(|x| largura(&col(x, 340, 460))));
        let forte = rampa([530, 535, 540]).max(1.0);
        let topo: Vec<f64> = (100..=500)
            .map(|x| {
                let v = col(x, 30, 150);
                let m = pico(&v);
                if m > 0.02 {
                    v.iter().position(|&a| a >= 0.5 * m).unwrap_or(0) as f64
                } else {
                    0.0
                }
            })
            .collect();
        let rugosidade = media(
            &topo
                .windows(2)
                .map(|p| (p[1] - p[0]).abs())
                .collect::<Vec<_>>(),
        );
        let leve = media(
            &(85..=185)
                .step_by(5)
                .map(|x| pico(&col(x, 340, 460)))
                .collect::<Vec<_>>(),
        );
        let banda = |x0: usize, x1: usize| -> (f64, f64) {
            let cols: Vec<Vec<f64>> = (x0..x1).map(|x| col(x, 340, 460)).collect();
            let tinta = media(&cols.iter().map(|v| media(v)).collect::<Vec<_>>());
            let dif = media(
                &cols
                    .windows(2)
                    .map(|p| {
                        media(
                            &p[0]
                                .iter()
                                .zip(&p[1])
                                .map(|(a, b)| (b - a).abs())
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            let sigma = |v: &Vec<f64>| {
                let w = v.iter().sum::<f64>() + 1e-9;
                let mu = v.iter().enumerate().map(|(y, a)| a * y as f64).sum::<f64>() / w;
                (v.iter()
                    .enumerate()
                    .map(|(y, a)| a * (y as f64 - mu).powi(2))
                    .sum::<f64>()
                    / w)
                    .sqrt()
            };
            (
                dif / tinta.max(1e-6),
                media(&cols.iter().map(sigma).collect::<Vec<_>>()),
            )
        };
        let ((aspereza_leve, sigma_leve), (_, sigma_cheio)) = (banda(85, 185), banda(500, 545));
        let picos: Vec<f64> = (85..185).map(|x| pico(&col(x, 340, 460))).collect();
        let n = picos.len() as f64;
        let (mx, my) = ((n - 1.0) / 2.0, media(&picos));
        let inclina = picos
            .iter()
            .enumerate()
            .map(|(x, y)| (x as f64 - mx) * (y - my))
            .sum::<f64>()
            / picos
                .iter()
                .enumerate()
                .map(|(x, _)| (x as f64 - mx).powi(2))
                .sum::<f64>();
        let resto: Vec<f64> = picos
            .iter()
            .enumerate()
            .map(|(x, y)| (y - my - inclina * (x as f64 - mx)).powi(2))
            .collect();
        let borrao = media(&resto).sqrt() / my.max(1e-6);
        Self {
            borda: media(&rec.iter().map(|v| borda(v)).collect::<Vec<_>>()),
            opacidade,
            rampa_fio: rampa([130, 135, 140]) / forte,
            rampa_meio: rampa([305, 310, 315]) / forte,
            rampa_baixa: rampa([205, 210, 215]) / forte,
            rampa_alta: rampa([405, 410, 415]) / forte,
            rugosidade,
            leve: leve / opacidade.max(1e-6),
            aspereza_leve,
            espalha_leve: sigma_leve / sigma_cheio.max(1e-6),
            borrao,
        }
    }
}

/// As FOTOS dos oráculos das canetas na cena `gestos` (A), medidas por `canetas_foto.py` (20/10): a da Mapping Pen e
/// as das suas IRMÃS do catálogo C (as canetas que ela não pode pintar), e a ESCALA a que a nossa se pinta (a largura do
/// recto lento do oráculo ÷ a nossa, a do `exame.sh`).
#[rustfmt::skip]
const FOTOS: [(&str, &str, Identidade); 5] = [
    ("Mapping Pen", "krita d) Ink-1 Precision", Identidade { borda: 9.0, opacidade: 0.998, rampa_fio: 1.0, rampa_meio: 1.0, rampa_baixa: 1.0, rampa_alta: 1.0, rugosidade: 0.0, leve: 1.001, aspereza_leve: 0.005, espalha_leve: 1.0, borrao: 0.002 }),
    ("G-pen", "krita d) Ink-3 Gpen", Identidade { borda: 1.0, opacidade: 1.0, rampa_fio: 0.1, rampa_meio: 0.567, rampa_baixa: 0.217, rampa_alta: 0.9, rugosidade: 0.0, leve: 0.99, aspereza_leve: 0.009, espalha_leve: 0.099, borrao: 0.003 }),
    ("Fineliner", "krita d) Ink-2 Fineliner", Identidade { borda: 1.0, opacidade: 1.0, rampa_fio: 0.789, rampa_meio: 0.895, rampa_baixa: 0.842, rampa_alta: 0.947, rugosidade: 0.0, leve: 1.0, aspereza_leve: 0.002, espalha_leve: 0.801, borrao: 0.0 }),
    ("Ballpoint", "mypaint deevad/ballpen", Identidade { borda: 13.0, opacidade: 0.687, rampa_fio: 0.807, rampa_meio: 0.833, rampa_baixa: 0.772, rampa_alta: 0.921, rugosidade: 0.0, leve: 0.125, aspereza_leve: 0.04, espalha_leve: 0.918, borrao: 0.28 }),
    ("Pen Rough", "krita d) Ink-4 Pen Rough", Identidade { borda: 0.833, opacidade: 1.0, rampa_fio: 0.145, rampa_meio: 0.5, rampa_baixa: 0.318, rampa_alta: 0.755, rugosidade: 0.145, leve: 0.99, aspereza_leve: 0.084, espalha_leve: 0.141, borrao: 0.018 }),
];

/// As canetas de fábrica da família: o nome, a escala do exame e o que faz a identidade.
#[rustfmt::skip]
const CANETAS: [(&str, f64, &[M]); 2] = [
    ("Mapping Pen", 1.273, &[M::Borda, M::RampaFio, M::RampaBaixa, M::RampaMeio, M::RampaAlta, M::Opacidade, M::Rugosidade, M::Leve]),
    ("Ballpoint", 1.462, &[M::Borda, M::Opacidade, M::RampaFio, M::RampaMeio, M::Leve, M::AsperezaLeve, M::EspalhaLeve, M::Borrao]),
];

/// ⭐ **Cada caneta reconhece-se pela foto do seu oráculo** (J3, `docs/Painter/Pinceis/exames/canetas.md`): a RECEITA
/// (não o ficheiro — `cada_preset_de_fabrica_e_a_sua_receita` prende-os) pintada na cena `gestos` da bateria à escala
/// do oráculo; cada medida da sua identidade cai na tolerância da foto, e — a lei dos IRMÃOS — a caneta fica
/// estritamente mais perto (em tolerâncias somadas) da SUA foto que da de cada uma das outras canetas do catálogo (o
/// G-pen, o fineliner, a esferográfica, a tinta áspera): o fio sem pressão não pinta nenhuma delas.
#[test]
fn cada_caneta_se_reconhece_pela_foto() {
    use crate::exame_bancada::{Fonte, Pincel, bateria, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let gestos = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "gestos")
        .unwrap();
    let mut falhas = Vec::new();
    for (nome, escala, ms) in CANETAS {
        let mut cena = gestos.clone();
        for op in cena["ops"].as_array_mut().unwrap() {
            if let Some(d) = op.get("size_px").and_then(serde_json::Value::as_f64) {
                op["size_px"] = serde_json::json!(d * escala);
            }
        }
        let r = MEIOS
            .iter()
            .chain(EM_ESTUDO)
            .find(|r| r.nome == nome)
            .unwrap();
        let (parametros, texturas) = crate::presets_de_fabrica::parametros_do_meio(r);
        let preset = ph2d_brush_presets::Preset {
            parametros,
            origem: ph2d_brush_presets::Origem::Fabrica,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![255; 4],
            },
            texturas,
        };
        let p = Pincel {
            nome: nome.to_owned(),
            fonte: Fonte::Preset {
                preset: Box::new(preset),
                relativo: nome.to_owned(),
            },
        };
        let eu = Identidade::de(
            &pinta_cena(&p, None, &cena, tela, papel, 0).1,
            tela[0] as usize,
        );
        let (_, oraculo, foto) = FOTOS.iter().find(|f| f.0 == nome).unwrap();
        println!("[canetas] {nome}: {eu:?}\n          foto {oraculo}: {foto:?}");
        for &m in ms {
            let (v, f) = (m.de(&eu), m.de(foto));
            if (v - f).abs() > m.tol(f) {
                falhas.push(format!(
                    "{nome}: {m:?} {v:.3} fora da foto {f:.3} ± {:.3} ({oraculo})",
                    m.tol(f)
                ));
            }
        }
        let distancia = |outra: &Identidade| -> f64 {
            ms.iter()
                .map(|&m| (m.de(&eu) - m.de(outra)).abs() / m.tol(m.de(foto)))
                .sum()
        };
        let propria = distancia(foto);
        for (outra, _, f2) in FOTOS.iter().filter(|f| f.0 != nome) {
            let d = distancia(f2);
            if propria >= d {
                falhas.push(format!("{nome}: {propria:.2} tolerâncias da sua foto, {d:.2} da do {outra} — pinta a irmã"));
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
