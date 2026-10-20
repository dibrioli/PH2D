//! **Família `canetas`** — J3, catálogo C (as canetas digitais). Aprovada: a Mapping Pen (C-5). O Studio Pen (C-4), o
//! Technical Pen (C-6), a Ballpoint (C-7) e o Dry Ink (C-11) foram REPROVADOS no exame e o Brush Pen (C-10) fundiu-se
//! no G-pen — o porquê, as receitas medidas e o que falta ao motor: `docs/Painter/Pinceis/exames/canetas.md`.
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//!
//! No Digital (o carimbo do `ph2d-painter-brush`). O tamanho de fábrica é o do preset de ORIGEM (o ficheiro do Krita,
//! CC-0).

use super::prelude::*;

pub const WET: &[Receita] = &[];

pub const MEIOS: &[ReceitaMeio] = &[
    // C-5 — o fio de largura FIXA (nenhuma pressão) e perfil gaussiano (Krita Ink-1 Precision): o perfil do traço é o
    // do dab (Pow4) com o passo SOLTO (a cápsula satura a orla e endurece-a); o início afina um pouco, como no Krita.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.2"),
            ("falloff", r#""Pow4""#),
            ("spacing", "0.08"),
            ("taper", r#"{"start":0.2,"tip_start":0.6,"opacity":0.0}"#),
        ],
        mao: &[("pressureSize", 0.0)],
        size_px: Some(3.0),
        opacity: Some(0.99),
        ..ReceitaMeio::digital("Ink", "Mapping", "Mapping Pen")
    },
];

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
    /// |Δ| médio, entre colunas vizinhas, da linha de cima do recto lento a ½ do pico: a borda rasgada.
    rugosidade: f64,
    /// O pico médio da rampa a pressão 0,05–0,25 ÷ a opacidade: a tinta que o traço leve deixa.
    leve: f64,
}

#[derive(Clone, Copy, Debug)]
enum M {
    Borda,
    Opacidade,
    RampaFio,
    RampaMeio,
    Rugosidade,
    Leve,
}

impl M {
    fn de(self, i: &Identidade) -> f64 {
        match self {
            M::Borda => i.borda,
            M::Opacidade => i.opacidade,
            M::RampaFio => i.rampa_fio,
            M::RampaMeio => i.rampa_meio,
            M::Rugosidade => i.rugosidade,
            M::Leve => i.leve,
        }
    }
    /// A tolerância contra a foto (a orla é inteira e escala com o macio).
    fn tol(self, foto: f64) -> f64 {
        match self {
            M::Borda => 0.5 + 0.35 * foto,
            M::Opacidade => 0.06,
            M::RampaFio | M::RampaMeio => 0.12,
            M::Rugosidade => 0.1 + 0.3 * foto,
            M::Leve => 0.15,
        }
    }
}

const CHEIO: f64 = 255.0 - 23.0;

impl Identidade {
    fn de(px: &[u8], w: usize) -> Self {
        let c = |x: usize, y: usize| {
            let p = &px[(y * w + x) * 4..(y * w + x) * 4 + 3];
            p.iter().map(|&v| (255.0 - f64::from(v)).abs()).fold(0.0, f64::max) / CHEIO
        };
        let col = |x: usize, y0: usize, y1: usize| -> Vec<f64> { (y0..y1).map(|y| c(x, y)).collect() };
        let pico = |v: &[f64]| v.iter().copied().fold(0.0, f64::max);
        let largura = |v: &[f64]| {
            let m = pico(v);
            if m > 0.02 { v.iter().filter(|&&a| a >= 0.25 * m).count() as f64 } else { 0.0 }
        };
        let borda = |v: &[f64]| {
            let m = pico(v);
            if m > 0.02 { v.iter().filter(|&&a| a > 0.1 * m && a < 0.9 * m).count() as f64 / 2.0 } else { 0.0 }
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
                if m > 0.02 { v.iter().position(|&a| a >= 0.5 * m).unwrap_or(0) as f64 } else { 0.0 }
            })
            .collect();
        let rugosidade = media(&topo.windows(2).map(|p| (p[1] - p[0]).abs()).collect::<Vec<_>>());
        let leve = media(&(85..=185).step_by(5).map(|x| pico(&col(x, 340, 460))).collect::<Vec<_>>());
        Self {
            borda: media(&rec.iter().map(|v| borda(v)).collect::<Vec<_>>()),
            opacidade,
            rampa_fio: rampa([130, 135, 140]) / forte,
            rampa_meio: rampa([305, 310, 315]) / forte,
            rugosidade,
            leve: leve / opacidade.max(1e-6),
        }
    }
}

/// As FOTOS dos oráculos das canetas na cena `gestos` (A), medidas por `canetas_foto.py` (20/10): a da Mapping Pen e
/// as das suas IRMÃS do catálogo C (as canetas que ela não pode pintar), e a ESCALA a que a nossa se pinta (a largura do
/// recto lento do oráculo ÷ a nossa, a do `exame.sh`).
#[rustfmt::skip]
const FOTOS: [(&str, &str, Identidade); 5] = [
    ("Mapping Pen", "krita d) Ink-1 Precision", Identidade { borda: 9.0, opacidade: 0.998, rampa_fio: 1.0, rampa_meio: 1.0, rugosidade: 0.0, leve: 1.001 }),
    ("G-pen", "krita d) Ink-3 Gpen", Identidade { borda: 1.0, opacidade: 1.0, rampa_fio: 0.1, rampa_meio: 0.567, rugosidade: 0.0, leve: 0.99 }),
    ("Fineliner", "krita d) Ink-2 Fineliner", Identidade { borda: 1.0, opacidade: 1.0, rampa_fio: 0.789, rampa_meio: 0.895, rugosidade: 0.0, leve: 1.0 }),
    ("Ballpen", "mypaint deevad/ballpen", Identidade { borda: 13.0, opacidade: 0.687, rampa_fio: 0.807, rampa_meio: 0.833, rugosidade: 0.0, leve: 0.125 }),
    ("Pen Rough", "krita d) Ink-4 Pen Rough", Identidade { borda: 0.833, opacidade: 1.0, rampa_fio: 0.145, rampa_meio: 0.5, rugosidade: 0.145, leve: 0.99 }),
];

/// As canetas de fábrica da família: o nome, a escala do exame e o que faz a identidade.
#[rustfmt::skip]
const CANETAS: [(&str, f64, &[M]); 1] = [
    ("Mapping Pen", 1.167, &[M::Borda, M::RampaFio, M::RampaMeio, M::Opacidade, M::Rugosidade, M::Leve]),
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
    let gestos = bat["cenas"].as_array().unwrap().iter().find(|c| c["nome"] == "gestos").unwrap();
    let mut falhas = Vec::new();
    for (nome, escala, ms) in CANETAS {
        let mut cena = gestos.clone();
        for op in cena["ops"].as_array_mut().unwrap() {
            if let Some(d) = op.get("size_px").and_then(serde_json::Value::as_f64) {
                op["size_px"] = serde_json::json!(d * escala);
            }
        }
        let r = MEIOS.iter().find(|r| r.nome == nome).unwrap();
        let (parametros, texturas) = crate::presets_de_fabrica::parametros_do_meio(r);
        let preset = ph2d_brush_presets::Preset {
            parametros,
            origem: ph2d_brush_presets::Origem::Fabrica,
            miniatura: ph2d_brush_presets::Miniatura { largura: 1, altura: 1, rgba: vec![255; 4] },
            texturas,
        };
        let p = Pincel { nome: nome.to_owned(), fonte: Fonte::Preset { preset: Box::new(preset), relativo: nome.to_owned() } };
        let eu = Identidade::de(&pinta_cena(&p, None, &cena, tela, papel, 0).1, tela[0] as usize);
        let (_, oraculo, foto) = FOTOS.iter().find(|f| f.0 == nome).unwrap();
        println!("[canetas] {nome}: {eu:?}\n          foto {oraculo}: {foto:?}");
        for &m in ms {
            let (v, f) = (m.de(&eu), m.de(foto));
            if (v - f).abs() > m.tol(f) {
                falhas.push(format!("{nome}: {m:?} {v:.3} fora da foto {f:.3} ± {:.3} ({oraculo})", m.tol(f)));
            }
        }
        let distancia = |outra: &Identidade| -> f64 {
            ms.iter().map(|&m| (m.de(&eu) - m.de(outra)).abs() / m.tol(m.de(foto))).sum()
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
