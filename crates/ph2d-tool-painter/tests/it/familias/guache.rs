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
        real_oil: false,
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
        real_oil: false,
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
            // J6: a guache DILUÍDA não cobre — VELA (o Rebelle: o cruzamento no vermelho molhado [208,195,210], mais
            // escuro que as duas; o nosso sem véu [227,208,217], com 0,3 [207,192,207]). Régua 355 → 363/456. ⛔ O
            // `wetCover` (a lei da J6) leva-o ao contrário (resto 0,73 → 0,31; o Rebelle 0,95); `felt` 0,05: 361.
            ("glaze", 0.3),
        ],
        grao: None,
        achatado: None,
        pincel: &[
            ("dab_quina", "1.0"),
            ("dab_flatten", "0.0"),
            ("size_max_px", "56"),
        ],
        nasce_de: Some("Gouache Flat"),
        real_oil: false,
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
        real_oil: false,
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
    assert!(
        na_fabrica.is_empty(),
        "reprovados na fábrica: {na_fabrica:?}"
    );
}

/// A cena `sobre` (foto A) pintada pelo preset `nome` com o mapa `engine` sobreposto: `(rgba, largura)`.
fn sobre_com(nome: &str, engine: &[(&str, f64)]) -> (Vec<u8>, usize) {
    use crate::exame_bancada::{Fonte, Pincel, bateria, pinceis, pinta_cena};
    let p = pinceis(nome).remove(0);
    let Fonte::Preset {
        mut preset,
        relativo,
    } = p.fonte
    else {
        unreachable!()
    };
    for (k, v) in engine {
        preset.parametros.engine.insert((*k).to_owned(), *v);
    }
    let p = Pincel {
        nome: p.nome,
        fonte: Fonte::Preset { preset, relativo },
    };
    let bat = bateria();
    let cena = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "sobre")
        .unwrap()
        .clone();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    (
        pinta_cena(&p, None, &cena, tela, papel, 0).1,
        tela[0] as usize,
    )
}

/// O RESTO do vermelho molhado sob o azul (cena `sobre` A, cruzamentos x 250 e 500 na faixa molhada y 150): a distância
/// de cor do cruzamento ao azul no papel (y 240) ÷ a do vermelho longe (x + 125) ao mesmo azul — `0` = o azul cobre,
/// `≈ 1` = o vermelho fica. E o vermelho longe (para o controlo: a faixa fora do azul não muda).
fn resto(px: &[u8], w: usize) -> (f64, [f64; 3]) {
    let caixa = |x: usize, y: usize| {
        let mut s = [0.0; 3];
        for yy in y - 4..=y + 4 {
            for xx in x - 4..=x + 4 {
                for (k, c) in s.iter_mut().enumerate() {
                    *c += f64::from(px[(yy * w + xx) * 4 + k]) / 81.0;
                }
            }
        }
        s
    };
    let d = |a: [f64; 3], b: [f64; 3]| (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f64>().sqrt();
    let (mut r, mut longe) = (0.0, [0.0; 3]);
    for x in [250, 500] {
        let (cz, az, rv) = (caixa(x, 150), caixa(x, 240), caixa(x + 125, 150));
        r += d(cz, az) / d(rv, az).max(1e-6) / 2.0;
        for k in 0..3 {
            longe[k] += rv[k] / 2.0;
        }
    }
    (r, longe)
}

/// ⭐ **O azul da guache COBRE o vermelho molhado** (`Knob::WetCover`, J6 lacuna 7, `exames/guache.md` «Motor J6»): no
/// Rebelle o cruzamento sobre a faixa ainda molhada é a cor do azul no papel (resto 0,11 na Flat, 0,33 na Round); a
/// mistura de sempre deixa o vermelho por baixo (a nossa Flat 0,36, púrpura). CONTROLO: a mesma Flat sem a lei; a
/// faixa longe do azul não muda.
#[test]
fn o_azul_da_guache_cobre_o_vermelho_molhado() {
    let [(com, w), (sem, _)] = std::thread::scope(|s| {
        let a = s.spawn(|| sobre_com("Gouache Flat", &[("wetCover", 1.0)]));
        let b = s.spawn(|| sobre_com("Gouache Flat", &[]));
        [a.join().unwrap(), b.join().unwrap()]
    });
    let ((rc, lc), (rs, ls)) = (resto(&com, w), resto(&sem, w));
    println!(
        "[cobre] Gouache Flat: resto {rc:.3} com a lei · {rs:.3} sem (Rebelle 0,11) · longe {lc:?} · {ls:?}"
    );
    assert!(
        rs > 0.28,
        "o controlo (a mistura de sempre) já cobre: resto {rs:.3}"
    );
    assert!(
        rc < 0.18,
        "o azul não cobre o vermelho molhado: resto {rc:.3} (Rebelle 0,11)"
    );
    assert!(
        (0..3).all(|k| (lc[k] - ls[k]).abs() < 2.0),
        "a lei mexeu na faixa longe do azul: {lc:?} contra {ls:?}"
    );
}

/// A Gouache Dry da receita (sem a tabela da largura: o diâmetro pedido é o pintado) com o mapa `engine` sobreposto,
/// num recto de 40 px (y 80) e num de 320 (y 400), foto molhada: `(rgba, largura)`.
fn rectos_da_guache_seca(engine: &[(&str, f64)]) -> (Vec<u8>, usize) {
    use crate::exame_bancada::{Fonte, Pincel, pinta_cena};
    let r = EM_ESTUDO.iter().find(|r| r.nome == "Gouache Dry").unwrap();
    let mut preset = ph2d_brush_presets::Preset {
        parametros: crate::presets_de_fabrica::parametros(r),
        origem: ph2d_brush_presets::Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas: Default::default(),
    };
    for (k, v) in engine {
        preset.parametros.engine.insert((*k).to_owned(), *v);
    }
    let p = Pincel {
        nome: r.nome.to_owned(),
        fonte: Fonte::Preset {
            preset: Box::new(preset),
            relativo: String::new(),
        },
    };
    let recto = |y: f64| -> Vec<serde_json::Value> {
        (0..294)
            .map(|i| serde_json::json!([60.0 + 3.0 * f64::from(i), y, 1.0]))
            .collect()
    };
    let cena = serde_json::json!({"nome": "fios", "ops": [
        {"op": "pincel", "quem": "preset"},
        {"op": "cor", "rgb": [23, 101, 179]},
        {"op": "traco", "nome": "r40", "size_px": 40, "passo": 3.0, "pts": recto(80.0)},
        {"op": "traco", "nome": "r320", "size_px": 320, "passo": 3.0, "pts": recto(400.0)},
        {"op": "simula", "quadros": 60},
        {"op": "foto", "nome": "A"},
    ]});
    (
        pinta_cena(&p, None, &cena, [1000, 640], [255; 3], 0).1,
        1000,
    )
}

/// A LARGURA das riscas ao comprido de um traço horizontal (miolo x 300..700, linhas `y ± meia`): o 1.º zero da
/// autocorrelação do perfil por linha (a escuridão média de cada linha, sem a sua média móvel de ¼ da tinta).
fn largura_das_riscas(px: &[u8], w: usize, y: usize, meia: usize) -> usize {
    let linha = |yy: usize| {
        (300..700)
            .map(|x| {
                let i = (yy * w + x) * 4;
                1.0 - (f64::from(px[i]) + f64::from(px[i + 1]) + f64::from(px[i + 2])) / 765.0
            })
            .sum::<f64>()
            / 400.0
    };
    let p: Vec<f64> = (y - meia..=y + meia).map(linha).collect();
    let tinta: Vec<usize> = (0..p.len()).filter(|&i| p[i] > 0.03).collect();
    let r = &p[tinta[0]..=*tinta.last().unwrap()];
    let k = (r.len() / 4).max(3);
    let c: Vec<f64> = (0..r.len())
        .map(|i| {
            let (a, b) = (i.saturating_sub(k / 2), (i + k / 2 + 1).min(r.len()));
            r[i] - r[a..b].iter().sum::<f64>() / (b - a) as f64
        })
        .collect();
    (1..c.len())
        .find(|&l| (0..c.len() - l).map(|i| c[i] * c[i + l]).sum::<f64>() <= 0.0)
        .unwrap_or(c.len())
}

/// ⭐ **Os FIOS crescem com o pincel** (`Knob::BristleLaneSizeExp`, J6 cerdas, `exames/guache.md` «Motor J6 — cerdas»):
/// a guache seca do Rebelle pinta riscas de ~2 px a 40 e de ~10 a 200 (a largura cresce com o diâmetro); os fios em px
/// fixos dão a mesma risca fina a todo tamanho (a «nuvem pontilhada» dos 200 px). CONTROLO: a mesma receita sem a lei.
#[test]
fn os_fios_da_guache_seca_crescem_com_o_pincel() {
    let [(com, w), (sem, _)] = std::thread::scope(|s| {
        let a = s.spawn(|| rectos_da_guache_seca(&[("bristleLaneSizeExp", 1.0)]));
        let b = s.spawn(|| rectos_da_guache_seca(&[]));
        [a.join().unwrap(), b.join().unwrap()]
    });
    let razao = |px: &[u8]| {
        let (a, b) = (
            largura_das_riscas(px, w, 80, 40),
            largura_das_riscas(px, w, 400, 200),
        );
        (a, b, b as f64 / a.max(1) as f64)
    };
    let ((c40, c320, rc), (s40, s320, rs)) = (razao(&com), razao(&sem));
    println!(
        "[fios] riscas 40 → 320 px: com a lei {c40} → {c320} ({rc:.2}×) · sem {s40} → {s320} ({rs:.2}×)"
    );
    // A lei = o diâmetro 8× → a risca ~8× (medido 9,3×); sem ela os fios ficam em 3 px e só a queda da borda alarga o
    // perfil (4,7×).
    assert!(
        rs < 6.0,
        "o controlo (fios em px fixos) já cresce: {s40} → {s320}"
    );
    assert!(
        rc > 7.0,
        "os fios não crescem com o pincel: {c40} → {c320} ({rc:.2}×, sem a lei {rs:.2}×)"
    );
}
