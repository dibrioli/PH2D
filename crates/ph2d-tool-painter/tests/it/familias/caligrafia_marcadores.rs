//! **Família `caligrafia_marcadores`** — J3, catálogos D e E. Na fábrica, curados e não aprovados (as diferenças que
//! sobram são de motor): o Marker Bullet (Wet) e o Marker Liner (Digital). Em estudo (reprovados, fora da fábrica):
//! Marker Chisel, Dry Marker, Broad Nib, Script, Shale. Fundido: o Monoline (Hard Round / Pixel 1px + Stabilizer). O
//! porquê de cada um: `docs/Painter/Pinceis/exames/caligrafia_marcadores.md`. Nada fora deste ficheiro (04_BANCADA §9).

use super::prelude::*;

/// A ponta LISA (sem a forma de cerdas do Wet: as franjas e os fiapos nas pontas que o Rebelle não tem).
const SEM_FORMA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// A tinta que ACABA de um lado da ponta (o Dry Marker): a rampa `Gradient` uma só vez através do dab (Size ½, Offset
/// ½ ⇒ `u ∈ [0, 1]`), ao longo do eixo LONGO da ponta (Angle 180 no referencial da pegada) — cheia em cima, nada em baixo.
const PONTA_QUE_ACABA: &str = r#"{"angle_deg": 180, "flow": false, "kind": "Gradient", "mapping": "ViewPlane", "offset": [0.5, 0.5], "params": [0.5, 0.5, 0.0, 0.0, 0.5, 0.5], "rake": false, "size": [0.5, 0.5], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

pub const WET: &[Receita] = &[
    // E-1 Marker › Marker Bullet (Rebelle `Marker/Bullet`): chapado, quase sem água, multiplica (Glaze 1) — a lei da
    // tinta por pincel (prompt 42); a ponta LISA (a forma de cerdas punha franjas nas pontas e na borda), sem arrasto
    // (pickup 0, tipClean 1, drag 0: o Rebelle cruza o vermelho num rectângulo limpo; o arrasto levava 0,02–0,04 de
    // vermelho para fora da faixa — revisão cega r1) e o tecto do Rebelle (Max Size 100: a 200 px pinta 101). Fica no
    // Wet: as leis da tinta molhada (`a_lei_da_tinta.rs`) têm-no por sujeito. A curva da carga 3 (era 2; fecho da J3):
    // sem a forma de cerdas a ponta lisa cobria cheia desde a pressão 0,3 e a MÃO DO DONO só respondia 1,52 (R_MAO 2,5,
    // `leis_sistemicas::a_pressao_de_cada_preset_e_a_da_sua_origem`); a 3, 2,79 com a largura a 0,139 da do Rebelle.
    Receita {
        meio: "Marker",
        grupo: "Marker",
        nome: "Marker Bullet",
        water: 0.1,
        opacity: 1.0,
        pigment: 800.0,
        pickup: 0.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.75,
        size_px: None,
        extras: &[
            ("glaze", 1.0),
            ("tipClean", 1.0),
            ("drag", 0.0),
            ("pressureSizeCurve", 0.25),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 3.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[("shape", SEM_FORMA), ("size_max_px", "101")],
        nasce_de: None,
    },
];

/// Os REPROVADOS no exame (J3, `docs/Painter/Pinceis/exames/caligrafia_marcadores.md`): receitas medidas, fora da
/// fábrica (nenhum ficheiro), à espera do motor que falta a cada um. O Broad Nib (Wet).
pub const EM_ESTUDO_WET: &[Receita] = &[
    // D-3 Ink › Nib › Broad Nib (MyPaint `classic/calligraphy`): o bico CHATO a ângulo fixo (L3: eixo longo a 48°, T
    // 0,05·L) — a largura sai da DIREÇÃO (fino ao longo do bico, cheio através), a tinta lisa do Ink Bullet e o corte
    // recto da ponta (quina 0,8); a tinta cobre (glaze 0: o azul sobre o vermelho é azul, como no MyPaint) e não arrasta. O
    // Fountain Square do Rebelle NÃO é isto (L 25 × T 19: quase redondo).
    Receita {
        meio: "Ink",
        grupo: "Nib",
        nome: "Broad Nib",
        water: 0.1,
        opacity: 1.0,
        pigment: 2000.0,
        pickup: 0.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.6,
        size_px: None,
        extras: &[("glaze", 0.0), ("tipClean", 1.0), ("drag", 0.0)],
        grao: None,
        achatado: Some((0.95, 48)),
        pincel: &[("shape", SEM_FORMA), ("dab_quina", "0.8")],
        nasce_de: Some("Ink Bullet"),
    },
];

/// O traço como UMA camada que multiplica (L8): um traço não escurece a si mesmo (a espiral, o cruzamento consigo), dois
/// sim — o marcador do Rebelle (o «sobre»: azul sobre azul escurece, sobre o vermelho multiplica).
const CAMADA_DO_MARCADOR: &str =
    r#"{"largura": 7.0, "molhada": 0.0, "queimada": 0.0, "tecto": 0.99}"#;

/// A camada do Liner com o tecto POR DAB (`TracoComposto::tecto_por_dab`): a tinta do traço pára na opacidade da mão.
const CAMADA_DO_LINER: &str =
    r#"{"largura": 7.0, "molhada": 0.0, "queimada": 0.0, "tecto": 0.99, "tecto_por_dab": true}"#;

pub const MEIOS: &[ReceitaMeio] = &[
    // E-2 — o marcador de BISEL de borda firme (Rebelle `Marker/Liner`): a ponta fixa na tela (L3: eixo longo deitado — o
    // ajuste dá 4°, a régua prefere 0° —, L 0,94·D × T 0,37·D: achatado 0,55 com a borda macia, que alarga o lado fino; a
    // foto antiga lida como «fina de 18 px» era este bisel visto de lado), a borda que amacia com o tamanho (dureza 0,35:
    // 1 px a 40, ~6 a 200 — o Rebelle 2 e 8), o corte quase recto (quina 0,7), a camada que multiplica, a pressão que
    // afina pouco (0,4) e leva a tinta (0,8, com o tecto POR DAB: a camada pára na opacidade da mão) — o Rebelle fica
    // opaco no traço leve, mas a MÃO DO DONO responde (fecho da J3: sem a tinta pela pressão 1,33 < R_MAO 2,5 em
    // `leis_sistemicas::a_pressao_de_cada_preset_e_a_da_sua_origem`; com ela 2,87, a largura a 0,048). No Digital: o Size é o eixo LONGO
    // do bisel, como no Rebelle (a lei da largura do Wet promete o traço horizontal — o lado FINO —, e o slider a 40
    // pintava um bisel de 102 px de comprido).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.35"),
            ("dab_flatten", "0.55"),
            ("dab_angle_deg", "0"),
            ("dab_quina", "0.7"),
            ("blend", r#""Multiply""#),
            ("continuous_spacing", "true"),
            ("traco", CAMADA_DO_LINER),
        ],
        mao: &[
            ("pressureSize", 0.4),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.8),
        ],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Marker", "Marker", "Marker Liner")
    },
    // D-1 Monoline: FUNDIDO no Hard Round / Pixel 1px (`docs/Painter/Pinceis/exames/caligrafia_marcadores.md`): na
    // bateria fica a 0,066 do Pixel 1px; a identidade dele (o StreamLine) é a linha «Stabilizer» do cartão (0,95 tira
    // 55 % do tremor), não outro pincel.
];

/// Os REPROVADOS do Digital (o porquê e o motor que falta: o relatório da família): medidos pelo portão a partir da
/// receita (`presets_de_fabrica::parametros_do_meio`), fora da fábrica.
pub const EM_ESTUDO: &[ReceitaMeio] = &[
    // E-3 — o bisel de pontas que ESMAECEM (Rebelle `Marker/Chisel`): a ponta do Liner com a queda inteira (dureza 0), o
    // corte recto (quina 0,8), as duas pontas que entram e saem pela tinta (taper romba 0,3 D, opacidade 1) e a pressão
    // que leva a TINTA, não a largura (a rampa do Rebelle é de largura constante e entra pela opacidade).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.0"),
            ("dab_flatten", "0.61"),
            ("dab_angle_deg", "0"),
            ("dab_quina", "0.8"),
            (
                "taper",
                r#"{"start":0.3,"tip_start":1.0,"opacity":1.0,"end":0.3,"tip_end":1.0}"#,
            ),
            ("blend", r#""Multiply""#),
            ("continuous_spacing", "true"),
            ("traco", CAMADA_DO_MARCADOR),
        ],
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 2.0),
        ],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Marker", "Marker", "Marker Chisel")
    },
    // E-4 — o marcador a ACABAR (Rebelle `Marker/Dry`): a ponta de bisel fixa de pé (L3: eixo a 100°, T 0,32·L; a 90° e
    // com a quina 0,8 as pontas saem rectas como as dele — a 100° saíam inclinadas), dabs soltos a 0,04, a
    // tinta que acaba de um lado da ponta (cheia em cima, nada em baixo — fixa na tela), translúcida; os dabs a fluxo
    // baixo SOMAM dentro da camada do traço (o perfil é o do dab; sem a camada o Multiply escurecia o traço consigo até
    // ao azul-marinho) e a pressão leva a tinta. O tecto do Rebelle: a 200 px pinta 71 (o bisel de ~110).
    ReceitaMeio {
        pincel: &[
            ("hardness", "1.0"),
            ("dab_flatten", "0.68"),
            ("dab_angle_deg", "90"),
            ("dab_quina", "0.8"),
            ("shape", PONTA_QUE_ACABA),
            ("blend", r#""Multiply""#),
            ("flow", "0.2"),
            ("continuous_spacing", "false"),
            ("spacing", "0.04"),
            (
                "traco",
                r#"{"largura": 7.0, "molhada": 0.0, "queimada": 0.0, "tecto": 0.9}"#,
            ),
            ("size_max_px", "110"),
        ],
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 2.0),
        ],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Marker", "Marker", "Dry Marker")
    },
    // D-2 — o brush lettering (Procreate Script, doc.): a ponta ELÍPTICA fixa com o eixo longo ao longo da diagonal
    // que SOBE (135° no referencial da pegada: fina a subir para a direita, cheia a descer — a 45° saía ao contrário),
    // a pressão que leva a largura de um fio ao cheio (1,0), as duas pontas afiladas até à ponta e esmaecidas (1 D, ponta
    // 0, opacidade 0,5: a 0,3 acabavam em barbatana) e o estabilizador forte (0,98: o tremor 2,72 → 0,97 px).
    ReceitaMeio {
        pincel: &[
            ("hardness", "1.0"),
            ("borda_px", "1.0"),
            ("continuous_spacing", "true"),
            ("dab_flatten", "0.75"),
            ("dab_angle_deg", "135"),
            ("stabilizer", "0.98"),
            (
                "taper",
                r#"{"start":1.0,"tip_start":0.0,"opacity":0.5,"end":1.0}"#,
            ),
        ],
        mao: &[("pressureSize", 1.0)],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Calligraphy", "Script", "Script")
    },
    // D-4 — o pincel de letras MACIO (Procreate Shale Brush, doc.): borda macia (dureza 0,4) mordida pelo grão do papel
    // (Grain fixo na tela, 6, fundo 0,8: o miolo cheio, o grão onde a borda é parcial — dureza 0,2 e fundo 1 lia-se
    // pastel), a pressão que leva a largura, as pontas afiladas e esmaecidas e o estabilizador 0,98 (tremor 1,02 px).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.4"),
            ("continuous_spacing", "true"),
            ("stabilizer", "0.98"),
            (
                "texture",
                r#"{"angle_deg": 0, "flow": false, "kind": "Grain", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [6.0, 6.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#,
            ),
            ("grain_depth", "0.8"),
            (
                "taper",
                r#"{"start":1.0,"tip_start":0.0,"opacity":0.5,"end":1.0}"#,
            ),
        ],
        mao: &[("pressureSize", 0.8)],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Calligraphy", "Texture", "Shale")
    },
];

// Orla 1 nos marcadores (como no Round): não conta. (O Liner saiu daqui: passou ao Digital, e o portão por foto é do Wet.)
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Marker Bullet", foto: "markerbullet", raio: 20.0, metricas: &[(Cobertura, 0.909), (Textura, 0.658), (Borda, 0.768), (Cruzamento, 0.016)] },
];

// ---- O portão da família (J3, `docs/Painter/Pinceis/exames/caligrafia_marcadores.md`) ----

/// A identidade de um pincel da família na bateria do EXAME (`ferramentas/bateria.json`, a foto A, escala 1) e na cena
/// do TREMOR (a mão que treme, para o estabilizador): o que a prancha mostrou contra o oráculo, num número.
#[derive(Debug, Clone, Copy)]
struct Letra {
    /// A largura do vertical lento ÷ a do recto lento: a ponta FIXA na tela (bisel deitado ≫ 1, de pé ≪ 1).
    de_pe: f64,
    /// Na curva S, a largura nos troços que SOBEM para a direita ÷ a dos que DESCEM: a largura pela direção (o bico
    /// a 45–48° é fino a subir).
    sobe_desce: f64,
    /// Píxeis da orla (10–90 % do miolo) na beira de cima do recto lento: ~1 duro, ≫ macio.
    orla: f64,
    /// A tinta da metade de CIMA do recto lento ÷ a da de baixo: a tinta que acaba de um lado da ponta.
    cima_baixo: f64,
    /// O azul sobre o azul (`cruza_1` × `papel`) ÷ o azul só: o que escurece na sobreposição.
    sobre_si: f64,
    /// O desvio rms (px) do centro da tinta à recta na cena do tremor: o que o estabilizador tira da mão.
    tremor: f64,
    /// A tinta da tampa do recto lento (x 46–58, no centro; o gesto começa em x 60) ÷ a do miolo: a ponta que ENTRA pela tinta.
    entrada: f64,
}

/// A escuridão de um píxel (1 − luminância): 0 o papel.
fn escuro(px: &[u8], w: usize, x: usize, y: usize) -> f64 {
    let p = &px[(y * w + x) * 4..];
    1.0 - (0.299 * f64::from(p[0]) + 0.587 * f64::from(p[1]) + 0.114 * f64::from(p[2])) / 255.0
}

/// Quantos píxeis de `(x, y)` ao longo de `(dx, dy)` (unitário), nos dois sentidos até `h`, passam de `lim`.
fn largura_na_normal(
    px: &[u8],
    w: usize,
    hh: usize,
    x: f64,
    y: f64,
    n: (f64, f64),
    h: f64,
    lim: f64,
) -> f64 {
    let mut c = 0.0;
    let mut t = -h;
    while t <= h {
        let (xi, yi) = ((x + n.0 * t).round(), (y + n.1 * t).round());
        if xi >= 0.0
            && yi >= 0.0
            && (xi as usize) < w
            && (yi as usize) < hh
            && escuro(px, w, xi as usize, yi as usize) > lim
        {
            c += 1.0;
        }
        t += 1.0;
    }
    c
}

/// A cena do tremor: um recto de 12 px de x 60 a 939 em y 100 com a mão que treme (4 px a 9 px de onda, 2 px a 3,1,
/// e ±1,2 px sorteados por uma lcg fixa) — o desenho de `docs/Painter/Pinceis/exames/caligrafia_marcadores/bateria_tremor.json`
/// (o da bancada, com outro sorteio).
fn cena_do_tremor() -> serde_json::Value {
    let mut s: u64 = 7;
    let mut sorteio = || {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((s >> 33) as f64 / f64::from(1u32 << 31)) * 2.4 - 1.2
    };
    let pts: Vec<serde_json::Value> = (0..294)
        .map(|i| {
            let x = 60.0 + 3.0 * f64::from(i);
            let dx = 3.0 * f64::from(i);
            let y = 100.0 + 4.0 * (dx / 9.0).sin() + 2.0 * (dx / 3.1).sin() + sorteio();
            serde_json::json!([x, y, 1.0])
        })
        .collect();
    serde_json::json!({"nome": "tremor", "ops": [
        {"op": "pincel", "quem": "preset"}, {"op": "cor", "rgb": [23, 101, 179]},
        {"op": "traco", "nome": "tremor", "size_px": 12, "passo": 3.0, "pts": pts},
        {"op": "foto", "nome": "A"}]})
}

/// A PROVA DE MUTAÇÃO sem tocar nos ficheiros partilhados: `PH2D_MUTACAO_CALIGRAFIA` = `{"<preset>": {"brush.<campo>":
/// v, "<chave da mão>": v}}` sobrepõe-se ao preset lido (a regra de `exame_bancada::variantes`); sem a variável, nada.
fn muta(p: &mut crate::exame_bancada::Pincel) {
    let Ok(j) = std::env::var("PH2D_MUTACAO_CALIGRAFIA") else {
        return;
    };
    let v: serde_json::Value = serde_json::from_str(&j).expect("PH2D_MUTACAO_CALIGRAFIA: JSON");
    let (Some(m), crate::exame_bancada::Fonte::Preset { preset, .. }) =
        (v.get(&p.nome), &mut p.fonte)
    else {
        return;
    };
    for (chave, valor) in m.as_object().expect("um objecto por preset") {
        if let Some(campo) = chave.strip_prefix("brush.") {
            preset
                .parametros
                .brush
                .insert(campo.to_owned(), valor.clone());
        } else {
            preset
                .parametros
                .engine
                .insert(chave.clone(), valor.as_f64().expect("a mão é um número"));
        }
    }
}

/// O pincel `nome`: o da fábrica, ou — um reprovado `EM_ESTUDO` — o preset que a receita gravaria.
fn pincel_de(nome: &str) -> crate::exame_bancada::Pincel {
    use crate::exame_bancada::{Fonte, Pincel, pinceis};
    let Some(r) = EM_ESTUDO.iter().find(|r| r.nome == nome) else {
        return pinceis(nome).remove(0);
    };
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
    Pincel {
        nome: nome.to_owned(),
        fonte: Fonte::Preset {
            preset: Box::new(preset),
            relativo: nome.to_owned(),
        },
    }
}

fn mede_letra(nome: &str) -> Letra {
    use crate::exame_bancada::{bateria, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let cena = |n: &str| {
        bat["cenas"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["nome"] == n)
            .unwrap()
            .clone()
    };
    let mut p = pincel_de(nome);
    muta(&mut p);
    let (w, hh) = (tela[0] as usize, tela[1] as usize);
    let (_, g) = pinta_cena(&p, None, &cena("gestos"), tela, papel, 0);
    let (_, s) = pinta_cena(&p, None, &cena("sobre"), tela, papel, 0);
    let (_, tr) = pinta_cena(&p, None, &cena_do_tremor(), tela, papel, 0);
    // O recto lento (y 90, x 60..560): o miolo = o p90 da escuridão na faixa (a tinta que acaba não tem o pico no meio).
    let mut faixa: Vec<f64> = (150..470)
        .flat_map(|x| (50..131).map(move |y| (x, y)))
        .map(|(x, y)| escuro(&g, w, x, y))
        .collect();
    faixa.sort_by(f64::total_cmp);
    let miolo = faixa[faixa.len() * 9 / 10].max(1e-3);
    let lim = 0.25 * miolo;
    let col = |x: usize| (50..131).filter(|&y| escuro(&g, w, x, y) > lim).count() as f64;
    let largura = (150..470).map(col).sum::<f64>() / 320.0;
    let de_pe = (480..570)
        .map(|y| (650..751).filter(|&x| escuro(&g, w, x, y) > lim).count() as f64)
        .sum::<f64>()
        / 90.0
        / largura.max(1.0);
    // A curva S da bateria, pela tangente de cada ponto: sobe (dy < 0) × desce, nos troços de |inclinação| 25–65°.
    let pts: Vec<(f64, f64)> = bat["cenas"][0]["ops"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["nome"] == "curva_s")
        .unwrap()["pts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|q| (q[0].as_f64().unwrap(), q[1].as_f64().unwrap()))
        .collect();
    let (mut sobe, mut desce) = (Vec::new(), Vec::new());
    for k in 3..pts.len() - 3 {
        let (dx, dy) = (pts[k + 3].0 - pts[k - 3].0, pts[k + 3].1 - pts[k - 3].1);
        let l = dx.hypot(dy);
        let ang = (dy.abs() / dx.abs().max(1e-9)).atan().to_degrees();
        if !(25.0..=65.0).contains(&ang) {
            continue;
        }
        let larg = largura_na_normal(&g, w, hh, pts[k].0, pts[k].1, (-dy / l, dx / l), 45.0, lim);
        if dy < 0.0 {
            sobe.push(larg)
        } else {
            desce.push(larg)
        }
    }
    let media = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    let sobe_desce = media(&sobe) / media(&desce).max(1.0);
    // A orla de cima do recto lento: por coluna, as linhas entre 10 % e 90 % do miolo acima do centro.
    let orla = (150..470)
        .map(|x| {
            (50..90)
                .filter(|&y| {
                    let e = escuro(&g, w, x, y) / miolo;
                    (0.1..0.9).contains(&e)
                })
                .count() as f64
        })
        .sum::<f64>()
        / 320.0;
    // A tinta de cima ÷ a de baixo, à volta da linha do traço (y 90).
    let soma = |y0: usize, y1: usize| {
        (150..470)
            .flat_map(|x| (y0..y1).map(move |y| (x, y)))
            .map(|(x, y)| escuro(&g, w, x, y))
            .sum::<f64>()
    };
    let cima_baixo = soma(50, 90) / soma(91, 131).max(1e-3);
    let so = 0.5 * (escuro(&s, w, 250, 230) + escuro(&s, w, 400, 310));
    let sobre_si = escuro(&s, w, 250, 310) / so.max(1e-3);
    // O tremor: o centro da tinta por coluna (x 120..880, y 70..131) contra y 100.
    let mut q = (0.0_f64, 0.0_f64);
    for x in 120..880 {
        let (mut a, mut b) = (0.0, 0.0);
        for y in 70..131 {
            let e = escuro(&tr, w, x, y);
            a += e * y as f64;
            b += e;
        }
        if b > 0.5 {
            q.0 += (a / b - 100.0).powi(2);
            q.1 += 1.0;
        }
    }
    let tremor = (q.0 / q.1.max(1.0)).sqrt();
    let entrada = (46..58).map(|x| escuro(&g, w, x, 90)).sum::<f64>() / 12.0 / miolo;
    Letra {
        de_pe,
        sobe_desce,
        orla,
        cima_baixo,
        sobre_si,
        tremor,
        entrada,
    }
}

/// Os pincéis da família e os vizinhos que a matriz e as pranchas puseram perto.
const MEDIDOS: [&str; 9] = [
    "Marker Bullet",
    "Marker Liner",
    "Marker Chisel",
    "Dry Marker",
    "Script",
    "Shale",
    "Ink Bullet",
    "Hard Round",
    "Flat Block-in",
];

fn medidas() -> Vec<(&'static str, Letra)> {
    std::thread::scope(|s| {
        let h: Vec<_> = MEDIDOS
            .iter()
            .map(|&n| (n, s.spawn(move || mede_letra(n))))
            .collect();
        h.into_iter().map(|(n, h)| (n, h.join().unwrap())).collect()
    })
}

/// ⭐ **Cada letra e cada marcador é o SEU** (J3, `docs/Painter/Pinceis/exames/caligrafia_marcadores.md`): a identidade
/// que a prancha mostrou contra o oráculo, presa num número que o separa do vizinho — os da fábrica e os `EM_ESTUDO` (a
/// receita medida não se perde enquanto o motor que lhes falta não chega). A fronteira de cada regra é o meio entre o
/// pincel e o que ela separa, medidos na corrida de 21/10 (o relatório). O Broad Nib (Wet, em estudo) fica de fora: o
/// Wet não grava o preset sem a lei da largura — o número dele está no relatório (sobe ÷ desce 3,62 contra 0,99).
#[test]
fn cada_letra_e_marcador_tem_a_sua_identidade() {
    let ms = medidas();
    let m = |n: &str| ms.iter().find(|(k, _)| *k == n).unwrap().1;
    for (n, l) in &ms {
        println!("[letra] {n:>14}: {l:?}");
    }
    let mut falhas = Vec::new();
    let mut regra = |ok: bool, o_que: String| {
        if !ok {
            falhas.push(o_que);
        }
    };
    // As fronteiras: o meio entre o pincel e o que a regra separa (corrida de 21/10, `exames/caligrafia_marcadores.md`).
    let (bullet, liner, chisel, dry) = (
        m("Marker Bullet"),
        m("Marker Liner"),
        m("Marker Chisel"),
        m("Dry Marker"),
    );
    let (script, shale) = (m("Script"), m("Shale"));
    // O Bullet é redondo (1,00 contra o bisel 2,25) e multiplica onde se sobrepõe (1,14 contra a tinta 1,05).
    regra(
        bullet.de_pe < 1.6,
        format!(
            "Marker Bullet: de pé ÷ deitado {:.2} — a ponta deixou de ser redonda",
            bullet.de_pe
        ),
    );
    regra(
        bullet.sobre_si > 1.09,
        format!(
            "Marker Bullet: sobreposição {:.2} — não acumula",
            bullet.sobre_si
        ),
    );
    // O bisel deitado fixo na tela: o vertical 2,25 × o recto (contra o redondo 1,00) e a camada que multiplica (1,30).
    regra(
        liner.de_pe > 1.6,
        format!(
            "Marker Liner: de pé ÷ deitado {:.2} — o bisel não está deitado",
            liner.de_pe
        ),
    );
    regra(
        liner.sobre_si > 1.17,
        format!(
            "Marker Liner: sobreposição {:.2} — não multiplica",
            liner.sobre_si
        ),
    );
    regra(
        chisel.de_pe > 1.6,
        format!(
            "Marker Chisel: de pé ÷ deitado {:.2} — o bisel não está deitado",
            chisel.de_pe
        ),
    );
    // O Chisel entra pela tinta (a ponta romba que esmaece) e o Liner entra cheio.
    let meio_entrada = 0.5 * (chisel.entrada + liner.entrada);
    regra(
        chisel.entrada < liner.entrada - 0.2,
        format!(
            "Marker Chisel: entrada {:.2} contra o Liner {:.2} — a ponta não esmaece",
            chisel.entrada, liner.entrada
        ),
    );
    regra(
        liner.entrada > meio_entrada,
        format!(
            "Marker Liner: entrada {:.2} — a ponta esmaece como a do Chisel",
            liner.entrada
        ),
    );
    // A tinta que acaba de um lado (2,11 contra ~1,05), a ponta de pé (0,33 contra 1,00) e a camada que multiplica
    // (1,43 contra a lâmina do Flat Block-in, 1,00, que tem a mesma ponta de pé).
    regra(
        dry.cima_baixo > 1.58,
        format!(
            "Dry Marker: cima ÷ baixo {:.2} — a tinta não acaba de um lado",
            dry.cima_baixo
        ),
    );
    regra(
        dry.de_pe < 0.66,
        format!(
            "Dry Marker: de pé ÷ deitado {:.2} — a ponta não está de pé",
            dry.de_pe
        ),
    );
    regra(
        dry.sobre_si > 1.22,
        format!(
            "Dry Marker: sobreposição {:.2} — não multiplica",
            dry.sobre_si
        ),
    );
    // O Script: fino a subir (0,35 contra 1,00 do Hard Round) e a mão que treme sai lisa (0,96 px contra 2,72).
    regra(
        script.sobe_desce < 0.66,
        format!(
            "Script: sobe ÷ desce {:.2} — não é fino a subir",
            script.sobe_desce
        ),
    );
    regra(
        script.tremor < 1.84,
        format!(
            "Script: tremor {:.2} px — o estabilizador não age",
            script.tremor
        ),
    );
    // O Shale: a borda macia (3,64 px contra 1,00 do Script) e estabilizado (1,01 contra 2,72).
    regra(
        shale.orla > 2.32,
        format!("Shale: orla {:.2} px — a borda ficou dura", shale.orla),
    );
    regra(
        shale.tremor < 1.87,
        format!(
            "Shale: tremor {:.2} px — o estabilizador não age",
            shale.tremor
        ),
    );
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Os reprovados não estão na fábrica (J3): nenhum ficheiro com o nome de uma receita `EM_ESTUDO`.
#[test]
fn os_reprovados_da_familia_nao_estao_na_fabrica() {
    let b = ph2d_brush_presets::Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let nomes: Vec<&str> = EM_ESTUDO
        .iter()
        .map(|r| r.nome)
        .chain(EM_ESTUDO_WET.iter().map(|r| r.nome))
        .collect();
    let na_fabrica: Vec<String> = b
        .entradas
        .iter()
        .map(|e| e.preset.parametros.name.clone())
        .filter(|n| nomes.contains(&n.as_str()))
        .collect();
    assert!(
        na_fabrica.is_empty(),
        "reprovados na fábrica: {na_fabrica:?}"
    );
}
