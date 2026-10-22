//! **Família `efeitos`** — J4, catálogo M (especiais, efeitos). O exame de cada um:
//! `docs/Painter/Pinceis/exames/efeitos.md`; as pranchas em `pranchas/efeitos/`.

use super::prelude::*;

pub const WET: &[Receita] = &[];

pub const MEIOS: &[ReceitaMeio] = &[DODGE, BURN, COLOR_GLAZE];

/// REPROVADO (J4): o revisor cego (média) — os rápidos do Krita param a meio e esvaem-se (o Curve dele não pinta o
/// fim de um traço rápido), o fio da recta do Krita é mais fino e claro que a soma das nossas cordas curtas, o pingo
/// escuro onde a caneta pousa, e a teia dele mais irregular (o centro da espiral cheio). A fita casa (o motor:
/// `wire_cordas`).
/// M-7 — a teia do Krita `v) Experimental Webs` (o motor Curve): o traço liga-se aos pontos que deixou para trás no
/// PERCURSO e, numa curva, as cordas cortam a quina e enchem o lado de dentro numa fita em degradê (forte na linha, a
/// esvair para dentro); numa recta as cordas caem sobre a linha e ela é só um fio. É o `LineKind::Wire` sem a linha
/// de dabs (`wire_connection_line` falso — o fio é a soma das cordas curtas), 24 cordas por dab (`wire_cordas`: a 4
/// eram quatro fios soltos, não a fita) numa janela de 2,5 diâmetros, cada corda de 1 px a 0,06 (a recta do Krita é um fio claro).
const SKETCHY_WEB: ReceitaMeio = ReceitaMeio {
    pincel: &[
        ("hardness", "1.0"),
        ("line_kind", "\"Wire\""),
        ("wire_history", "2.5"),
        ("wire_connection_line", "false"),
        ("wire_cordas", "24"),
        ("thread_width_px", "1.0"),
        ("thread_opacity", "0.06"),
        ("stabilizer", "0.0"),
    ],
    mao: &[("pressureSize", 0.0)],
    opacity: Some(1.0),
    ..ReceitaMeio::digital("Effects", "Sketch", "Sketchy Web")
};

/// Os EFEITOS do Rebelle (`Marker/Effect`; 06 §2 e §10.3, L7): um modo de mistura W3C em sRGB codificado a um α fixo por
/// PASSAGEM. O perfil do traço dele é o de um CONE somado ao longo do traço (a queda Linear: a ½ 0,57 · a 0,9 do raio
/// 0,07 do pico, o que o Rebelle mede; o Smooth dava 0,45 · 0,01 e a orla cortada a ±7 px que três revisores viram),
/// não o planalto de um tecto: dabs de fluxo baixo que SOMAM (Accumulate) num PLANO do traço (o tecto 1 — nos 8 bits da
/// tela os incrementos de < ½ nível por dab perdiam-se: o G do Color ficava 0, o R do Dodge 191). Medido a 40 px
/// (`exames/efeitos_foto.py`, o pico do véu e a largura a 50/25/10/4 %): Burn 0,095 · 18/28/34/36 contra o Rebelle
/// 0,091 · 20/25/30/36. No papel vazio pousa em Normal com o mesmo α (`blend_sobre_vazio`, L7-vazio; a bancada com
/// `PH2D_EXAME_CAMADA=1`). A pressão leva a força; o tamanho é o do artista. Dodge = ADDITIVE (Add), Burn = LINEAR_BURN
/// (o B do Rebelle cai 39 % e o R 10 %, que nenhuma fórmula a um α dá — decisão 7: fica 5 níveis acima), Color.
/// Hue ≡ Color sobre a tinta saturada (06 §2) e Overlay é um Color mais escuro na mesma bateria: fundem-se no Color
/// Glaze (o modo é uma linha do cartão); Luminosity e Saturation não se vêem (ΔE ≤ 1,5) e o Saturation do Rebelle não
/// segue fórmula nenhuma (decisão 7) — não são pincéis.
macro_rules! efeito {
    ($blend:literal, $fluxo:literal, $queda:literal, $curva:expr, $nome:literal) => {
        ReceitaMeio {
            pincel: &[
                ("hardness", "0.0"),
                ("falloff", $queda),
                ("custom_falloff", $curva),
                ("accumulate", "true"),
                ("flow", $fluxo),
                ("spacing", "0.1"),
                ("traco", r#"{"largura": 7.0, "molhada": 0.0, "queimada": 0.0, "tecto": 1.0}"#),
                ("blend_sobre_vazio", "true"),
                // Sem o estabilizador do Soft Round: ele atrasava o traço 3–4 px (o início e o fim depois dos do
                // Rebelle, que três revisores viram).
                ("stabilizer", "0.0"),
                ("blend", $blend),
            ],
            mao: &[("pressureSize", 0.0), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 1.0)],
            opacity: Some(1.0),
            ..ReceitaMeio::digital("Effects", "Effects", $nome)
        }
    };
}

/// O SINO do Dodge e do Burn: uma gaussiana σ 0,45 do raio, truncada no aro. Sobre o vermelho o escurecimento do Burn
/// do Rebelle é mais concentrado que o véu no papel (a 50/25/10 % 20 · 26 · 32 px): a Linear dava 18 · 28 · 34 — «uma
/// faixa larga e esbatida» (r4) — e o sino 18 · 24 · 30.
const SINO: &str = r#"[{"handle": "Auto", "id": 0, "x": 0, "y": 1.0}, {"handle": "Auto", "id": 1, "x": 0.15, "y": 0.941}, {"handle": "Auto", "id": 2, "x": 0.3, "y": 0.7823}, {"handle": "Auto", "id": 3, "x": 0.45, "y": 0.5701}, {"handle": "Auto", "id": 4, "x": 0.6, "y": 0.3566}, {"handle": "Auto", "id": 5, "x": 0.75, "y": 0.1799}, {"handle": "Auto", "id": 6, "x": 0.9, "y": 0.0554}, {"handle": "Auto", "id": 7, "x": 1, "y": 0.0}]"#;
/// A curva neutra (o Color Glaze lê a queda Linear; a curva só vai ao preset por ser um campo do meio).
const RECTA: &str = r#"[{"handle": "Auto", "id": 0, "x": 0.0, "y": 1.0}, {"handle": "Auto", "id": 1, "x": 1.0, "y": 0.0}]"#;

/// M-2 — clarear o que já está pintado (a luz por cima), em passagens.
const DODGE: ReceitaMeio = efeito!("\"Add\"", "0.019", "\"Custom\"", SINO, "Dodge");
/// M-2 — escurecer o que já está pintado (a sombra por cima, sem sujar a cor), em passagens.
const BURN: ReceitaMeio = efeito!("\"LinearBurn\"", "0.021", "\"Custom\"", SINO, "Burn");
/// M-3 — trocar a cor do que está pintado sem mexer no valor, em passagens.
const COLOR_GLAZE: ReceitaMeio = efeito!("\"Color\"", "0.012", "\"Linear\"", RECTA, "Color Glaze");

/// Reprovados nesta onda (o exame: `docs/Painter/Pinceis/exames/efeitos.md`) — fora da fábrica, a receita fica.
pub const EM_ESTUDO: &[ReceitaMeio] = &[GLOW, SKETCHY_WEB];

/// REPROVADO (J4): dois revisores cegos (média) — os cruzamentos sobre o vermelho do MyPaint escurecem a quase preto (a
/// mistura ESPECTRAL do MyPaint 2: motor do Digital, na fila desde a Ballpoint da J3) e o caroço escuro onde a caneta
/// pousa (o `slow_tracking` dele empilha os dabs no arranque). A forma (núcleo + halo, a pressão, a cabeça do traço
/// rápido pelo atraso da velocidade) casa.
const GLOW: ReceitaMeio =
    // M-1 — o néon do MyPaint `experimental/glow` (CC0): um NÚCLEO saturado com um HALO largo e ralo à volta, no mesmo
    // traço. É a pegada dele (a queda do libmypaint, ISC: `1 − r²·(1/h − 1)` até `r² = h`, depois `h·(1 − r²)/(1 − h)`,
    // com h 0,12 — a 0,16 do ficheiro dava o núcleo largo de mais, razão halo ÷ núcleo 2,5 × 2,9) somada por dabs
    // densos (passo 0,09 = os 5,5 dabs por raio dele) a 0,45 cada: o centro satura, o halo fica num PATAMAR a ~¼ do
    // pico. Medido a 40 px (`exames/efeitos_foto.py`): pico 0,82 · núcleo 18 · halo 52 · patamar 0,26 no MyPaint;
    // a mesma razão a 5 e a 200 px. A pressão abre o traço (o raio dele × e^0,5 de 0 a 1: halo 36 → 52) e não o
    // escurece; a velocidade estreita-o (a ¼ do pico: o recto rápido 14 px e o vertical
    // rápido 22 × 36 do lento no MyPaint) e com ATRASO — o speed1_slowness 0,04 s dele: o traço nasce largo e afina (a
    // cabeça do recto rápido; o vertical curto acaba antes de o filtro chegar). 0,3 + 0,04 s: 16 · 22 px.
    ReceitaMeio {
        pincel: &[
            // A dureza 0,1 com a queda que a desconta (2.ª ronda): a velocidade tem por onde a AMACIAR — o recto
            // rápido do MyPaint (a dureza dele 0,05 a 2 500 px/s) é um fio fino; com a dureza 0 não havia como.
            ("hardness", "0.1"),
            ("falloff", "\"Custom\""),
            ("custom_falloff", NEON),
            ("accumulate", "true"),
            ("flow", "0.45"),
            ("spacing", "0.09"),
            // O passo pelo diâmetro de CADA dab (o `dabs_per_actual_radius` dele): o rápido que afina sem contas.
            ("passo_pela_pressao", "true"),
            // A mistura espectral do MyPaint 2 (2.ª ronda): o néon azul sobre o vermelho passa pelo grená escuro.
            ("pigment", "true"),
            ("pigment_mix", "1.0"),
            ("pigment_espectral", "true"),
        ],
        mao: &[
            ("pressureSize", 0.37),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.0),
            ("speedSize", 0.3),
            ("speedOpacity", 0.0),
            ("speedLag", 0.04),
            // O caroço onde a caneta pousa (2.ª ronda): a dureza dele a v = 0 (0,24 × 0,13 a 300 px/s) na nossa
            // queda — o núcleo 1,36× (√ da razão); a nossa dureza é um patamar: 0,27 a v = 0, 0 a 2 500 px/s.
            ("speedHardness", 0.059),
        ],
        size_px: Some(32.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Effects", "Glow", "Glow")
    };

/// A pegada do néon (a queda do libmypaint com dureza 0,12, em 8 pontos rectos — o tecto da curva é 8), DESCONTADA
/// da dureza 0,1 do preset: `curva(u) = néon(0,1 + 0,9·u)` — com a dureza 0,1 o dab é o néon de antes (fora do
/// patamar central de 0,1 do raio, que a soma dos dabs satura).
const NEON: &str = r#"[{"handle": "Vector", "id": 0, "x": 0.0, "y": 0.8534}, {"handle": "Vector", "id": 1, "x": 0.1111, "y": 0.7067}, {"handle": "Vector", "id": 2, "x": 0.2738, "y": 0.12}, {"handle": "Vector", "id": 3, "x": 0.4444, "y": 0.1023}, {"handle": "Vector", "id": 4, "x": 0.5556, "y": 0.0873}, {"handle": "Vector", "id": 5, "x": 0.6667, "y": 0.0695}, {"handle": "Vector", "id": 6, "x": 0.8333, "y": 0.0378}, {"handle": "Vector", "id": 7, "x": 1.0, "y": 0.0}]"#;

#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[];

/// A identidade de um EFEITO na cena `sobre` (foto A), pintada numa CAMADA sobre o papel com as faixas do Hard Round
/// — as contas de `docs/Painter/Pinceis/exames/efeitos_foto.py`: a cor no cruzamento 2 sobre a faixa de cima (x 500,
/// y 150, mediana 5×5) e o véu no papel (o pico do α do traço «papel», a mediana das colunas x 280..320).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Efeito {
    cima: [f64; 3],
    veu: f64,
}

impl Efeito {
    fn de(px: &[u8], w: usize) -> Self {
        let mediana = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let canal = |x: usize, y: usize, k: usize| f64::from(px[(y * w + x) * 4 + k]);
        let cima = [0, 1, 2].map(|k| {
            mediana(
                (498..=502)
                    .flat_map(|x| (148..=152).map(move |y| (x, y)))
                    .map(|(x, y)| canal(x, y, k))
                    .collect(),
            )
        });
        let veu = (280..=340)
            .map(|y| {
                mediana(
                    (280..=320)
                        .map(|x| ((255.0 - canal(x, y, 0)) / 232.0).clamp(0.0, 1.0))
                        .collect(),
                )
            })
            .fold(0.0, f64::max);
        Self { cima, veu }
    }

    /// A distância à foto em TOLERÂNCIAS, a pior das duas: 6 níveis por canal (o Linear Burn do Rebelle tira 39 % do
    /// azul e 10 % do vermelho, que nenhuma fórmula a um α dá — decisão 7: o resíduo do B fica 5) e 0,02 no véu (o
    /// Rebelle pousa o Burn a α 0,088 no papel e 0,107 na tinta; o nosso, um α só, fica 0,017 acima no papel).
    fn distancia(&self, outra: &Self) -> f64 {
        let c = (0..3)
            .map(|k| (self.cima[k] - outra.cima[k]).abs() / 6.0)
            .fold(0.0, f64::max);
        c.max((self.veu - outra.veu).abs() / 0.02)
    }
}

/// As fotos do Rebelle `Marker/Effect` (a cena `sobre`, A), medidas por `efeitos_foto.py` (21/10).
#[rustfmt::skip]
const FOTOS_EFEITO: [(&str, &str, Efeito); 3] = [
    ("Dodge", "rebelle MARKER/effect_dodge", Efeito { cima: [193.0, 9.0, 34.0], veu: 0.091 }),
    ("Burn", "rebelle MARKER/effect_burn", Efeito { cima: [170.0, 0.0, 11.0], veu: 0.091 }),
    ("Color Glaze", "rebelle MARKER/effect_color", Efeito { cima: [181.0, 4.0, 25.0], veu: 0.060 }),
];

/// O preset de uma receita da família (a fábrica ou o `EM_ESTUDO`), pela mesma porta do gerador — sem ficheiro.
fn preset_da_receita(nome: &str) -> ph2d_brush_presets::Preset {
    let r = MEIOS
        .iter()
        .chain(EM_ESTUDO)
        .find(|r| r.nome == nome)
        .unwrap();
    let (parametros, texturas) = crate::presets_de_fabrica::parametros_do_meio(r);
    ph2d_brush_presets::Preset {
        parametros,
        origem: ph2d_brush_presets::Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas,
    }
}

/// A cena `sobre` (foto A) de um preset numa CAMADA transparente sobre o papel, com as faixas do Hard Round, composta
/// sobre o papel (o que o artista vê) — a bancada com `PH2D_EXAME_CAMADA=1` e `PH2D_EXAME_BASE="Hard Round"`.
fn sobre_em_camada(nome: &str, preset: ph2d_brush_presets::Preset) -> Vec<u8> {
    use crate::exame_bancada::{Fonte, Pincel, bateria, pinceis, pinta_cena_com};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let sobre = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "sobre")
        .unwrap();
    let p = Pincel {
        nome: nome.to_owned(),
        fonte: Fonte::Preset {
            preset: Box::new(preset),
            relativo: String::new(),
        },
    };
    let base = pinceis("Hard Round").remove(0);
    let mut px = pinta_cena_com(&p, Some(&base), sobre, tela, papel, 0, true).1;
    for c in px.as_chunks_mut::<4>().0 {
        let a = u32::from(c[3]);
        for k in 0..3 {
            c[k] = ((u32::from(c[k]) * a + u32::from(papel[k]) * (255 - a) + 127) / 255) as u8;
        }
        c[3] = 255;
    }
    px
}

/// As falhas de `nome` medido em `eu`: fora da sua foto (1 tolerância), ou não estritamente mais perto da SUA foto que
/// da de cada irmão (a lei dos IRMÃOS).
fn falhas_do_efeito(nome: &str, eu: &Efeito) -> Vec<String> {
    let (_, oraculo, foto) = FOTOS_EFEITO.iter().find(|f| f.0 == nome).unwrap();
    let mut falhas = Vec::new();
    let propria = eu.distancia(foto);
    if propria > 1.0 {
        falhas.push(format!(
            "{nome}: {eu:?} a {propria:.2} tolerâncias da foto {foto:?} ({oraculo})"
        ));
    }
    for (outro, _, f2) in FOTOS_EFEITO.iter().filter(|f| f.0 != nome) {
        let d = eu.distancia(f2);
        if propria >= d {
            falhas.push(format!(
                "{nome}: {propria:.2} tolerâncias da sua foto, {d:.2} da do {outro} — faz o irmão"
            ));
        }
    }
    falhas
}

/// ⭐ **Cada efeito reconhece-se pela foto do Rebelle** (J4, `docs/Painter/Pinceis/exames/efeitos.md`): na cena
/// `sobre`, numa camada sobre o papel e sobre as faixas do Hard Round, o cruzamento com o vermelho cai na cor da foto
/// (o Dodge clareia e esfria, o Burn escurece, o Color troca o matiz sem mexer no valor) e o véu no papel no α dela
/// (o «On empty»: Normal ao mesmo α) — e cada um fica mais perto da SUA foto que da dos irmãos.
#[test]
fn cada_efeito_se_reconhece_pela_foto() {
    let medidas: Vec<(&str, Efeito)> = std::thread::scope(|s| {
        let hs: Vec<_> = FOTOS_EFEITO
            .iter()
            .map(|&(nome, _, _)| {
                s.spawn(move || {
                    (
                        nome,
                        Efeito::de(&sobre_em_camada(nome, preset_da_receita(nome)), 1000),
                    )
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (nome, eu) in &medidas {
        println!("[efeitos] {nome}: {eu:?}");
        falhas.extend(falhas_do_efeito(nome, eu));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// **A prova de mutação do portão, permanente** (J4): cada mutação do que FAZ a identidade de um efeito — o preset de
/// fábrica com um campo trocado — tem de sangrar `falhas_do_efeito`.
#[test]
fn o_portao_dos_efeitos_sangra_com_cada_mutacao() {
    use serde_json::json;
    let muta = |nome: &str, f: &dyn Fn(&mut ph2d_brush_presets::Parametros)| {
        let mut p = preset_da_receita(nome);
        f(&mut p.parametros);
        p
    };
    let campo = |k: &'static str, v: serde_json::Value| {
        move |p: &mut ph2d_brush_presets::Parametros| {
            p.brush.insert(k.to_owned(), v.clone());
        }
    };
    let mutacoes: Vec<(&str, &str, ph2d_brush_presets::Preset)> = vec![
        (
            "Dodge",
            "o modo Mix (pinta o azul)",
            muta("Dodge", &campo("blend", json!("Mix"))),
        ),
        (
            "Dodge",
            "sem «On empty» (sem véu no papel)",
            muta("Dodge", &campo("blend_sobre_vazio", json!(false))),
        ),
        (
            "Burn",
            "o fluxo × 3 (a velatura vira tinta)",
            muta("Burn", &campo("flow", json!(0.033))),
        ),
        (
            "Burn",
            "o modo Screen (clareia)",
            muta("Burn", &campo("blend", json!("Screen"))),
        ),
        (
            "Color Glaze",
            "o modo Luminosity",
            muta("Color Glaze", &campo("blend", json!("Luminosity"))),
        ),
    ];
    let sangrou: Vec<(&str, &str, Vec<String>)> = std::thread::scope(|s| {
        let hs: Vec<_> = mutacoes
            .into_iter()
            .map(|(nome, o_que, p)| {
                s.spawn(move || {
                    let eu = Efeito::de(&sobre_em_camada(nome, p), 1000);
                    println!("[efeitos] mutação {nome} · {o_que}: {eu:?}");
                    (nome, o_que, falhas_do_efeito(nome, &eu))
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let sobreviveu: Vec<String> = sangrou
        .iter()
        .filter(|(_, _, f)| f.is_empty())
        .map(|(n, o, _)| format!("{n} · {o}"))
        .collect();
    assert!(
        sobreviveu.is_empty(),
        "mutações que o portão NÃO apanha: {sobreviveu:?}"
    );
}

/// O cruzamento da cena `sobre` como o MyPaint o pinta (2.ª ronda, `exames/efeitos.md`): numa CAMADA transparente
/// composta sobre o papel (a superfície do oráculo, `fixtures/mypaint_bateria/README.md`), as faixas vermelhas com o
/// MESMO pincel, à escala do oráculo. Devolve a cor no meio do cruzamento 2 (x 500, y 150, mediana 5×5) e a do píxel
/// mais ESCURO à volta dele (y 120..180, x 440..560: o anel onde o néon ralo passa sobre o vermelho cheio).
fn cruzamento_do_mypaint(preset: ph2d_brush_presets::Preset, escala: f64) -> ([f64; 3], [f64; 3]) {
    use crate::exame_bancada::{Fonte, Pincel, bateria, pinta_cena_com};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let mut sobre = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "sobre")
        .unwrap()
        .clone();
    for op in sobre["ops"].as_array_mut().unwrap() {
        if let Some(d) = op.get("size_px").and_then(serde_json::Value::as_f64) {
            op["size_px"] = serde_json::json!(d * escala);
        }
    }
    let p = Pincel {
        nome: "cand".to_owned(),
        fonte: Fonte::Preset {
            preset: Box::new(preset),
            relativo: String::new(),
        },
    };
    let mut px = pinta_cena_com(&p, None, &sobre, tela, papel, 0, true).1;
    for c in px.as_chunks_mut::<4>().0 {
        let a = u32::from(c[3]);
        for k in 0..3 {
            c[k] = ((u32::from(c[k]) * a + u32::from(papel[k]) * (255 - a) + 127) / 255) as u8;
        }
    }
    let w = tela[0] as usize;
    let cor = |x: usize, y: usize| [0, 1, 2].map(|k| f64::from(px[(y * w + x) * 4 + k]));
    let centro = [0, 1, 2].map(|k| {
        let mut v: Vec<f64> = (498..=502)
            .flat_map(|x| (148..=152).map(move |y| (x, y)))
            .map(|(x, y)| cor(x, y)[k])
            .collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    });
    let escuro = (120..=180)
        .flat_map(|y| (440..=560).map(move |x| (x, y)))
        .map(|(x, y)| cor(x, y))
        .min_by(|a, b| a.iter().sum::<f64>().total_cmp(&b.iter().sum::<f64>()))
        .unwrap();
    (centro, escuro)
}

/// As fotos do MyPaint (`fixtures/mypaint_bateria`, a cena `sobre`): o pincel, a escala do exame, o meio do
/// cruzamento e o píxel mais escuro do anel (medidos por `scratchpad`/`exames/efeitos.md` 2.ª ronda, 21/10).
#[rustfmt::skip]
type CruzamentoMp = (&'static str, &'static str, f64, [f64; 3], [f64; 3]);
const CRUZAMENTOS_DO_MYPAINT: [CruzamentoMp; 2] = [
    (
        "Glow",
        "mypaint experimental/glow",
        1.636,
        [43.0, 73.0, 131.0],
        [75.0, 52.0, 90.0],
    ),
    (
        "Ballpoint",
        "mypaint deevad/ballpen",
        1.462,
        [64.0, 74.0, 121.0],
        [70.0, 71.0, 115.0],
    ),
];

/// As falhas de um cruzamento contra a foto do MyPaint: o meio a ±10 níveis por canal, e o anel ESCURO (o grená do
/// néon ralo sobre o vermelho) a ±25 na soma dos canais com o verde a ±12.
fn falhas_do_cruzamento(nome: &str, (centro, escuro): ([f64; 3], [f64; 3])) -> Vec<String> {
    let (_, oraculo, _, c_mp, e_mp) = CRUZAMENTOS_DO_MYPAINT.iter().find(|c| c.0 == nome).unwrap();
    let mut falhas = Vec::new();
    if (0..3).any(|k| (centro[k] - c_mp[k]).abs() > 10.0) {
        falhas.push(format!(
            "{nome}: o meio do cruzamento {centro:?} × {c_mp:?} ({oraculo})"
        ));
    }
    let soma = |c: &[f64; 3]| c.iter().sum::<f64>();
    if (soma(&escuro) - soma(e_mp)).abs() > 25.0 || (escuro[1] - e_mp[1]).abs() > 12.0 {
        falhas.push(format!(
            "{nome}: o anel escuro {escuro:?} × {e_mp:?} ({oraculo})"
        ));
    }
    falhas
}

fn receita_em_estudo(nome: &str) -> &'static ReceitaMeio {
    EM_ESTUDO
        .iter()
        .chain(super::canetas::MEIOS)
        .find(|r| r.nome == nome)
        .unwrap()
}

/// ⭐ **A mistura ESPECTRAL cruza como o MyPaint** (2.ª ronda; o `BrushSpec::pigment_espectral`): o Glow e a Ballpoint
/// sobre o vermelho — o meio do cruzamento violeta-cinza e o anel grená (o néon ralo sobre o vermelho cheio), que o
/// K–M de constante única pinta azul e claro. E a prova de mutação, permanente: a MESMA receita com a lei do K–M
/// (`pigment_espectral` falso — a lei de antes, o vermelho visto antes da cura) tem de sangrar.
#[test]
fn a_mistura_espectral_cruza_como_o_mypaint() {
    use serde_json::json;
    let casos: Vec<(&str, bool)> = CRUZAMENTOS_DO_MYPAINT
        .iter()
        .flat_map(|c| [(c.0, true), (c.0, false)])
        .collect();
    let medidas: Vec<(&str, bool, Vec<String>)> = std::thread::scope(|s| {
        let hs: Vec<_> = casos
            .into_iter()
            .map(|(nome, espectral)| {
                s.spawn(move || {
                    let mut p = crate::presets_de_fabrica::preset_do_meio(receita_em_estudo(nome));
                    if !espectral {
                        p.parametros
                            .brush
                            .insert("pigment_espectral".to_owned(), json!(false));
                    }
                    let escala = CRUZAMENTOS_DO_MYPAINT
                        .iter()
                        .find(|c| c.0 == nome)
                        .unwrap()
                        .2;
                    let m = cruzamento_do_mypaint(p, escala);
                    println!(
                        "[espectral] {nome} (espectral {espectral}): meio {:?} · anel {:?}",
                        m.0, m.1
                    );
                    (nome, espectral, falhas_do_cruzamento(nome, m))
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (nome, espectral, f) in medidas {
        match (espectral, f.is_empty()) {
            (true, false) => falhas.extend(f),
            (false, true) => falhas.push(format!("{nome}: a mutação «o K–M» NÃO sangrou")),
            _ => {}
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// As CONTAS do traço de 200 px (cena `tamanhos`, à escala `escala`): na linha do meio do traço (a de maior α médio
/// em y 380..500), a mediana de 7 linhas em x 300..700 — `(máx − mín) ÷ média`. O MyPaint `deevad/ballpen`: 0,0125.
fn contas_a_200_px(preset: ph2d_brush_presets::Preset, escala: f64) -> f64 {
    use crate::exame_bancada::{Fonte, Pincel, bateria, pinta_cena_com};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let mut cena = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "tamanhos")
        .unwrap()
        .clone();
    for op in cena["ops"].as_array_mut().unwrap() {
        if let Some(d) = op.get("size_px").and_then(serde_json::Value::as_f64) {
            op["size_px"] = serde_json::json!(d * escala);
        }
    }
    let p = Pincel {
        nome: "cand".to_owned(),
        fonte: Fonte::Preset {
            preset: Box::new(preset),
            relativo: String::new(),
        },
    };
    let px = pinta_cena_com(&p, None, &cena, tela, papel, 0, false).1;
    let w = tela[0] as usize;
    let alfa =
        |x: usize, y: usize| ((255.0 - f64::from(px[(y * w + x) * 4])) / 232.0).clamp(0.0, 1.0);
    let media_da_linha = |y: usize| (300..700).map(|x| alfa(x, y)).sum::<f64>() / 400.0;
    let yc = (380..500)
        .max_by(|&a, &b| media_da_linha(a).total_cmp(&media_da_linha(b)))
        .unwrap();
    let linha: Vec<f64> = (300..700)
        .map(|x| {
            let mut v: Vec<f64> = (yc - 3..=yc + 3).map(|y| alfa(x, y)).collect();
            v.sort_by(f64::total_cmp);
            v[3]
        })
        .collect();
    let m = linha.iter().sum::<f64>() / linha.len() as f64;
    (linha.iter().copied().fold(0.0, f64::max) - linha.iter().copied().fold(1.0, f64::min))
        / m.max(1e-6)
}

/// ⭐ **A esferográfica de 200 px não sai em CONTAS** (2.ª ronda; o revisor da J3: «contas a 200 px»): a dureza pela
/// pressão (`pressureHardness`, o `hardness 0,2 + 0,3·p` do `deevad/ballpen`) dá ao dab cheio um topo chato, e o
/// traço de 200 px fica liso como o do MyPaint (0,0125; ≤ 0,025). Mutação permanente: sem a linha (a dureza 0 de
/// antes, 0,044) tem de sangrar.
#[test]
fn a_esferografica_de_200_px_nao_sai_em_contas() {
    let receita = receita_em_estudo("Ballpoint");
    let escala = CRUZAMENTOS_DO_MYPAINT
        .iter()
        .find(|c| c.0 == "Ballpoint")
        .unwrap()
        .2;
    let (com, sem) = std::thread::scope(|s| {
        let a =
            s.spawn(|| contas_a_200_px(crate::presets_de_fabrica::preset_do_meio(receita), escala));
        let b = s.spawn(|| {
            let mut p = crate::presets_de_fabrica::preset_do_meio(receita);
            assert!(
                p.parametros.engine.remove("pressureHardness").is_some(),
                "a receita perdeu a linha"
            );
            contas_a_200_px(p, escala)
        });
        (a.join().unwrap(), b.join().unwrap())
    });
    println!(
        "[contas] Ballpoint 200 px: {com:.4} (sem a dureza pela pressão {sem:.4}; MyPaint 0,0125)"
    );
    assert!(
        com <= 0.025,
        "a 200 px a esferográfica sai em contas: {com:.4} (MyPaint 0,0125)"
    );
    assert!(
        sem > 0.025,
        "a mutação «sem a dureza pela pressão» NÃO sangrou: {sem:.4}"
    );
}

/// A bancada dos EM_ESTUDO (e da Ballpoint das `canetas`, cuja lei do pigmento é desta família): grava cada um, pela
/// porta do gerador, em `PH2D_EM_ESTUDO_DIR/<nome>.ph2d-brush.png` — o `exame.sh` aceita o caminho.
#[test]
#[ignore = "bancada: escreve os EM_ESTUDO em PH2D_EM_ESTUDO_DIR"]
fn escreve_os_em_estudo() {
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_EM_ESTUDO_DIR").expect("PH2D_EM_ESTUDO_DIR"));
    for r in EM_ESTUDO {
        let p = crate::presets_de_fabrica::preset_do_meio(r);
        let f =
            ph2d_brush_presets::gravar(&dir, &format!("{}.ph2d-brush.png", r.nome), &p).unwrap();
        println!("[em estudo] {}", f.display());
    }
}

/// Os reprovados não estão na fábrica: nenhum ficheiro com o nome de uma receita `EM_ESTUDO` — e cada um continua a
/// ser um preset que se gera.
#[test]
fn os_em_estudo_dos_efeitos_nao_estao_na_fabrica() {
    use crate::presets_de_fabrica::{parametros_do_meio, relativo_do_meio};
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    for r in EM_ESTUDO {
        let rel = relativo_do_meio(r);
        assert!(
            !raiz.join(&rel).exists(),
            "{}: em estudo mas na fábrica ({rel})",
            r.nome
        );
        assert_eq!(parametros_do_meio(r).0.name, r.nome);
    }
}
