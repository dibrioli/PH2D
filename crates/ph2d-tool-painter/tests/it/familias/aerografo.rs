//! **Família `aerografo`** — J3, catálogo J: o Airbrush (Spray Can, Splatter, Stipple, Spray Drips e Grainy Shader
//! reprovados — `docs/Painter/Pinceis/exames/aerografo.md`).
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
        pigment: 10.5,
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
            // J3: o pó fica 0,8. DUAS revisões cegas leram o do Spray 2 mais GROSSO, de orla esfiapada; mais pó (0,85 ·
            // 0,9) só o torna mais RALO e tira as pontas do portão das mídias da barra 0,6 (0,53–0,59, nenhuma pressão
            // casa as duas). O grão do Rebelle pede uma GOTA maior — o motor (`exames/aerografo.md`, fila).
            ("dust", 0.8),
            MAIS_PERTO,
            ("pressureSize", 0.5),
            // As PONTAS do Rebelle (ordem do dono 19/10, doc 51 §4): a curva côncava alarga o traço leve mais cedo —
            // pela mediana de 21 tiragens do pó, pontas 0,580 → 0,680 (Rebelle 0,80), a rampa 0,640 → 0,704 (0,63).
            // Nenhuma intensidade com a curva recta casa as duas: as pontas seguem a rampa a −0,06.
            ("pressureSizeCurve", 0.7),
            ("pressureOpacity", 1.0),
            // J3 (`exames/aerografo.md`): a curva 1,2 e o pigmento 10,5 — a 1,5 e 9 o pó pousava 12 % menos que o
            // Spray 2 (opacidade 0,49 × 0,557, a cor mais clara em 9 traços) e as pontas da rampa 0,15→1→0,15
            // sumiam (0 × 0,61); régua 412 → 444/454.
            ("pressureOpacityCurve", 1.2),
        ],
        grao: None,
        achatado: None,
        pincel: &[("falloff", "\"Smooth\""), ("shape", SEM_FORMA)],
        nasce_de: None,
    },
];

pub const MEIOS: &[ReceitaMeio] = &[
    // J-2 Spray Can e J-5 Stipple: REPROVADOS na J3 (`docs/Painter/Pinceis/exames/aerografo.md`) — as revisões cegas
    // acharam o Krita em todas as linhas e listaram o que o motor de hoje não exprime: a GOTA de tamanho fixo em px (o
    // Krita não a escala com o pincel: a 5 px um cordão, a 200 px gotículas soltas), o VÉU contínuo por baixo das gotas
    // do Texture Spray (duas populações de partículas) e o spray que ACUMULA parado (Krita 0,24 → 1,00 a 10 → 200
    // eventos; o nosso Space não pousa nada parado). Saíram da receita e da fábrica; as receitas medidas estão no exame.
    //
    // J-3 Splatter: REPROVADO na J3 — duas revisões cegas pela descrição escolheram-no com confiança alta e «claramente
    // diferente», mas ambas listaram a mesma falta: num só traço não há a gama do PÓ às gotas GRANDES (o `jitter_scale`
    // sorteia o tamanho UNIFORME; os salpicos pedem uma lei de potência — muitas gotículas, poucas gotas). Saiu da
    // fábrica; a receita medida está no exame.
    //
    // J-6 Grainy Shader: REPROVADO na J3, FUNDIDO no Airbrush — a lei que o separava (o grão PRESO À TELA: passar
    // outra vez escurece os mesmos picos; portão medido: 0,58–1,00 dos vãos ficam × 0,04 no pó do Airbrush) existe,
    // mas a revisão cega pela descrição não o sustenta: fraco (Multiply, fluxo 0,3–0,6) as revisões 1–2 escolhem-no e
    // listam a sombra fraca, a borda esfarelada e o grão que enche; forte (fluxo 1, ou o modo Height) as revisões 3–4
    // escolhem o AIRBRUSH como «o sombreador granulado» e lêem o nosso como pontilhado de giz. Saiu da fábrica.
];

// O PÓ — a textura, a uniformidade e a orla larga são dele; o «período» não conta (a repetição do Rebelle varia 0,005,
// o controlo fica a 0,013 — doc 43 §2).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Airbrush", foto: "airbrush", raio: 15.0, metricas: &[(Cobertura, 0.381), (Textura, 22.705), (Borda, 0.537), (Orla, 12.0), (Cv, 0.088), (Cruzamento, 0.583), (RampaLargura, 0.63), (RampaCobertura, 0.34), (Largura, 26.0), (Pontas, 0.8)] },
];

/// O Shape sem forma (nem cerdas): a ponta é só a queda — o aerógrafo.
const SEM_FORMA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

// ── Os portões da família (J3, `docs/Painter/Pinceis/exames/aerografo.md`) ───────────────────────────────────────

const CHEIO: f64 = 255.0 - 23.0;

/// A identidade de um aerógrafo na cena `gestos` (A) — as contas de `docs/Painter/Pinceis/exames/aerografo_foto.py`.
#[derive(Clone, Copy, Debug)]
struct Identidade {
    /// As linhas da faixa do recto lento com perfil ≥ 0,25 do pico (px).
    largura: f64,
    /// A cobertura média dentro dessa faixa.
    cobertura: f64,
    /// A fracção do miolo (a metade central da faixa) com cobertura < 0,1: os VÃOS entre as gotas.
    vazio: f64,
    /// As manchas (cobertura ≥ 0,35, 8-vizinhança) por 100 px de recto.
    pontos: f64,
    /// A área mediana dessas manchas (px).
    gota: f64,
    /// O desvio-padrão do matiz (graus) dos píxeis com cobertura ≥ 0,5.
    matiz: f64,
    /// A cobertura da rampa 0→1 na pressão fraca ÷ na forte.
    rampa: f64,
}

#[derive(Clone, Copy, Debug)]
enum M {
    Largura,
    Cobertura,
    Vazio,
    Pontos,
    Gota,
    Matiz,
    Rampa,
}

impl M {
    fn de(self, i: &Identidade) -> f64 {
        match self {
            M::Largura => i.largura,
            M::Cobertura => i.cobertura,
            M::Vazio => i.vazio,
            M::Pontos => i.pontos,
            M::Gota => i.gota,
            M::Matiz => i.matiz,
            M::Rampa => i.rampa,
        }
    }
    /// A tolerância contra a foto (as contagens e as áreas são relativas).
    fn tol(self, foto: f64) -> f64 {
        match self {
            M::Largura => 4.0 + 0.12 * foto,
            M::Cobertura => 0.08,
            M::Vazio => 0.1,
            M::Pontos | M::Gota => 0.5 * foto.max(2.0),
            M::Matiz => 0.5 + 0.4 * foto,
            M::Rampa => 0.12,
        }
    }
}

/// O matiz HSV (graus) de um píxel sRGB — a conta do `colorsys.rgb_to_hsv`.
fn matiz_de(p: &[u8]) -> f64 {
    let [r, g, b] = [0, 1, 2].map(|k| f64::from(p[k]) / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    if mx == mn {
        return 0.0;
    }
    let d = mx - mn;
    let (rc, gc, bc) = ((mx - r) / d, (mx - g) / d, (mx - b) / d);
    let h = if r == mx {
        bc - gc
    } else if g == mx {
        2.0 + rc - bc
    } else {
        4.0 + gc - rc
    };
    (h / 6.0).rem_euclid(1.0) * 360.0
}

impl Identidade {
    fn de(px: &[u8], w: usize) -> Self {
        let c = |x: usize, y: usize| {
            px[(y * w + x) * 4..(y * w + x) * 4 + 3]
                .iter()
                .map(|&v| (255.0 - f64::from(v)).abs())
                .fold(0.0, f64::max)
                / CHEIO
        };
        let (x0, x1, y0, y1) = (150usize, 470usize, 0usize, 180usize);
        let prof: Vec<f64> = (y0..y1)
            .map(|y| (x0..x1).map(|x| c(x, y)).sum::<f64>() / (x1 - x0) as f64)
            .collect();
        let pico = prof.iter().copied().fold(0.0, f64::max);
        let dentro: Vec<usize> = (y0..y1).filter(|&y| prof[y - y0] >= 0.25 * pico).collect();
        let largura = dentro.len() as f64;
        let cobertura = dentro.iter().map(|&y| prof[y - y0]).sum::<f64>() / largura.max(1.0);
        let (ya, yb) = (dentro[0] as f64, dentro[dentro.len() - 1] as f64);
        let q = (yb - ya) / 4.0;
        let miolo: Vec<usize> = (y0..y1)
            .filter(|&y| (y as f64) >= ya + q && (y as f64) <= yb - q)
            .collect();
        let vaz = miolo
            .iter()
            .flat_map(|&y| (x0..x1).map(move |x| (x, y)))
            .filter(|&(x, y)| c(x, y) < 0.1)
            .count();
        let vazio = vaz as f64 / (miolo.len() * (x1 - x0)).max(1) as f64;
        // As manchas: 8-vizinhança na faixa, cobertura ≥ 0,35.
        let (fw, fh) = (x1 - x0, y1 - y0);
        let tinta: Vec<bool> = (0..fw * fh)
            .map(|i| c(x0 + i % fw, y0 + i / fw) >= 0.35)
            .collect();
        let mut visto = vec![false; fw * fh];
        let mut areas = Vec::new();
        for i in 0..fw * fh {
            if visto[i] || !tinta[i] {
                continue;
            }
            let (mut pilha, mut a) = (vec![i], 0usize);
            visto[i] = true;
            while let Some(j) = pilha.pop() {
                a += 1;
                let (u, v) = ((j % fw) as i64, (j / fw) as i64);
                for du in -1..=1 {
                    for dv in -1..=1 {
                        let (uu, vv) = (u + du, v + dv);
                        if uu >= 0 && vv >= 0 && (uu as usize) < fw && (vv as usize) < fh {
                            let k = vv as usize * fw + uu as usize;
                            if !visto[k] && tinta[k] {
                                visto[k] = true;
                                pilha.push(k);
                            }
                        }
                    }
                }
            }
            areas.push(a);
        }
        areas.sort_unstable();
        let pontos = areas.len() as f64 / ((x1 - x0) as f64 / 100.0);
        let gota = areas.get(areas.len() / 2).map_or(0.0, |&a| a as f64);
        let hs: Vec<f64> = (y0..y1)
            .step_by(2)
            .flat_map(|y| (x0..x1).step_by(2).map(move |x| (x, y)))
            .filter(|&(x, y)| c(x, y) >= 0.5)
            .map(|(x, y)| matiz_de(&px[(y * w + x) * 4..(y * w + x) * 4 + 3]))
            .collect();
        let matiz = if hs.is_empty() {
            0.0
        } else {
            let m = hs.iter().sum::<f64>() / hs.len() as f64;
            (hs.iter().map(|v| (v - m).powi(2)).sum::<f64>() / hs.len() as f64).sqrt()
        };
        let media = |xa: usize, xb: usize| {
            (xa..xb)
                .flat_map(|x| (340..460).map(move |y| (x, y)))
                .map(|(x, y)| c(x, y))
                .sum::<f64>()
                / ((xb - xa) * 120) as f64
        };
        Self {
            largura,
            cobertura,
            vazio,
            pontos,
            gota,
            matiz,
            rampa: media(150, 200) / media(480, 530).max(1e-6),
        }
    }
}

/// As FOTOS dos oráculos na cena `gestos` (A), medidas por `aerografo_foto.py` (20/10), e a ESCALA a que o nosso se
/// pinta (a do `exame.sh`: a largura do recto lento do oráculo ÷ a nossa). O Spray Can e o Stipple saíram da fábrica
/// (J3, reprovados): as fotos deles ficam como IRMÃOS — o Airbrush não pode pintar o spray nem o pontilhismo do Krita.
#[rustfmt::skip]
const FOTOS: [(&str, &str, f64, Identidade); 3] = [
    ("Airbrush", "rebelle AIRBRUSH/default_spray_2", 0.775, Identidade { largura: 32.0, cobertura: 0.393, vazio: 0.0, pontos: 11.88, gota: 2.0, matiz: 0.25, rampa: 0.131 }),
    ("Spray Can", "krita y)_Texture_Spray", 1.069, Identidade { largura: 113.0, cobertura: 0.435, vazio: 0.134, pontos: 177.19, gota: 2.0, matiz: 0.2, rampa: 0.245 }),
    ("Stipple", "krita v)_Texture_Pointillism", 1.0, Identidade { largura: 31.0, cobertura: 0.542, vazio: 0.225, pontos: 2.81, gota: 551.0, matiz: 3.62, rampa: 0.748 }),
];

/// Os pincéis da fábrica que se pintam e o que faz a identidade de cada um (os outros das [`FOTOS`] são só irmãos).
const IDENTIDADE: [(&str, &[M]); 1] = [(
    "Airbrush",
    &[
        M::Largura,
        M::Cobertura,
        M::Vazio,
        M::Pontos,
        M::Gota,
        M::Matiz,
        M::Rampa,
    ],
)];

/// A cena `gestos` da bateria com cada `size_px` × `escala`, pintada por `nome` (foto A).
fn gestos(nome: &str, escala: f64) -> Identidade {
    use crate::exame_bancada::{bateria, pinceis, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let mut cena = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "gestos")
        .unwrap()
        .clone();
    for op in cena["ops"].as_array_mut().unwrap() {
        if let Some(d) = op.get("size_px").and_then(serde_json::Value::as_f64) {
            op["size_px"] = serde_json::json!(d * escala);
        }
    }
    let p = pinceis(nome).remove(0);
    Identidade::de(
        &pinta_cena(&p, None, &cena, tela, papel, 0).1,
        tela[0] as usize,
    )
}

/// ⭐ **O aerógrafo reconhece-se pela foto do seu oráculo** (J3): pintado na cena `gestos`, cada medida da identidade
/// cai na tolerância da foto, e — a lei dos IRMÃOS — fica mais perto da SUA foto que da dos outros aerógrafos: o
/// Airbrush é névoa sem vãos, o spray do Krita milhares de gotículas com o núcleo aberto, o pontilhismo pontos grandes
/// e discretos de tom variado.
#[test]
fn cada_aerografo_se_reconhece_pela_foto() {
    let mut falhas = Vec::new();
    for (nome, ms) in IDENTIDADE {
        let (_, oraculo, escala, foto) = FOTOS.iter().find(|f| f.0 == nome).unwrap();
        let eu = &gestos(nome, *escala);
        println!("[aerografo] {nome}: {eu:?}\n            foto {oraculo}: {foto:?}");
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
        for (outro, _, _, f2) in FOTOS.iter().filter(|f| f.0 != nome) {
            let d = distancia(f2);
            if propria >= d {
                falhas.push(format!("{nome}: {propria:.2} tolerâncias da sua foto, {d:.2} da do {outro} — pinta o irmão"));
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// A cobertura (o azul cheio = 1) de `tracos` pintados por `nome` numa tela 1000×640 branca.
fn sonda(nome: &str, tracos: &[serde_json::Value]) -> Vec<f64> {
    use crate::exame_bancada::{pinceis, pinta_cena};
    use serde_json::json;
    let mut ops = vec![
        json!({"op": "pincel", "quem": "preset"}),
        json!({"op": "cor", "rgb": [23, 101, 179]}),
    ];
    ops.extend(tracos.iter().cloned());
    ops.push(json!({"op": "foto", "nome": "A"}));
    let px = pinta_cena(
        &pinceis(nome).remove(0),
        None,
        &json!({"nome": "sonda", "ops": ops}),
        [1000, 640],
        [255, 255, 255],
        0,
    )
    .1;
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

/// ⭐ **O aerógrafo PARADO pousa um sopro e não acumula** (J-1): a caneta parada no mesmo ponto, 10 e 200 eventos
/// (8 ms cada). O catálogo dizia «névoa que acumula enquanto se segura»; o Rebelle «Spray 2» (foto de 20/10,
/// `exames/aerografo/default_spray_2_parado_A.png`) dá 0,053 · 0,060 · 0,057 a 10 · 60 · 200 eventos — UM sopro, o
/// mesmo; o MyPaint `deevad/airbrush` 0 nos três. Quem acumula com o tempo é o spray do Krita (0,24 → 1,00), e esse
/// fica na fila (o Spray Can, ver o exame). O portão prende a lei do oráculo: o sopro existe e não cresce.
#[test]
fn o_aerografo_parado_pousa_um_sopro() {
    let parado = |n: usize| {
        let pts: Vec<serde_json::Value> = (0..n)
            .map(|i| serde_json::json!([500.0 + 0.01 * (i % 2) as f64, 320.0, 1.0]))
            .collect();
        let c = sonda(
            "Airbrush",
            &[serde_json::json!({"op": "traco", "nome": "parado", "size_px": 40.0, "pts": pts})],
        );
        (300..341)
            .flat_map(|y| (480..521).map(move |x| (x, y)))
            .map(|(x, y)| c[y * 1000 + x])
            .sum::<f64>()
            / 1681.0
    };
    let (pouco, muito) = (parado(10), parado(200));
    println!(
        "[aerografo] Airbrush parado: 10 eventos {pouco:.3} · 200 eventos {muito:.3} (Rebelle 0,053 · 0,057)"
    );
    assert!(
        pouco > 0.005,
        "o Airbrush parado não pousa nada: {pouco:.3}"
    );
    assert!(
        (muito - pouco).abs() <= 0.02,
        "o Airbrush parado acumula: {pouco:.3} → {muito:.3} (o Rebelle não)"
    );
}
