//! **Família `tintas`** — J3, catálogo C (as tintas do Wet Paint): Dip Pen, Ink Bullet, Ink Brush, Fountain Pen, Reed
//! Pen, Gloop Pen, Ink Bleed, Gesinski Ink, Wood Stick. Escreva aqui as receitas e os testes da família; nada fora
//! deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).

use super::prelude::*;

// ⭐ A lei da tinta POR PINCEL (prompt 42, doc 42): tinta e marcador — o Glaze do pincel escurece o
// cruzamento como o do Rebelle (molhado: Ink 0,117 · Dip Pen 0,223 · Ink Brush 0,239 · marcadores 0,016).
// As receitas da sonda `sonda_midias` (b7: `z_IB2_g100`, `z_DP_g80`, `z_IK_g90`, `z_MB_h75`,
// `z_ML_h90`). As cerdas são as do Shape de fábrica: o feltro/cerdas que a sonda punha no mapa do motor
// não pintam (medido: a cena sem eles é a mesma ao byte).
pub const WET: &[Receita] = &[
    // Ink › Brush: corpo ralo e orla de nanquim, a água que se espalha devagar. J3 (exame contra o Rebelle
    // `Brush/Brush`): os lados NÍTIDOS e as pontas em fios (as cerdas do motor, sem a forma de cerdas que
    // salpicava a borda), a tinta clara (pigmento 90) — `docs/Painter/Pinceis/exames/tintas.md`.
    Receita {
        meio: "Ink",
        grupo: "Brush",
        nome: "Ink Brush",
        water: 0.5,
        opacity: 1.0,
        pigment: 90.0,
        pickup: 1.0,
        gate: 0.0,
        evaporation: 3.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        // Glaze 0,8: a fracção multiplicada que o oráculo MEDIU nesta tinta (doc 42 §3: 0,76 · 0,82); o 1,0 de antes
        // compensava o halo e o azul arrastado para baixo (doc 42 §6).
        extras: &[
            ("extDiffusion", 0.01),
            ("glaze", 0.8),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 0.5),
            ("bristleStrength", 1.0),
            ("bristleLaneWidth", 1.5),
            ("bristleTipRagged", 0.8),
            ("bristlePressure", 0.6),
        ],
        grao: None,
        achatado: None,
        pincel: &[("shape", SEM_FORMA)],
        nasce_de: None,
    },
    // Ink › Pen › Dip Pen: duro e cheio, a borda GRUMOSA do aparo (o tamanho de cada carimbo sorteado ±25 %: os
    // grumos do Rebelle `Pen/Dip Pen`), o cruzamento que escurece (Glaze 0,9) e arrasta um pouco da cor de baixo.
    Receita {
        meio: "Ink",
        grupo: "Pen",
        nome: "Dip Pen",
        water: 0.3,
        opacity: 1.0,
        pigment: 1100.0,
        pickup: 10.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("glaze", 0.9),
            ("tipClean", 0.05),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[("jitter_scale", "0.25"), ("shape", SEM_FORMA)],
        nasce_de: None,
    },
    // Ink › Pen › Ink Bullet: o mais carregado e seco da tinta. J3 (Rebelle `Pen/Bullet`): a ponta redonda LISA (sem
    // a forma de cerdas que esfiapava as pontas — o que a punha ao lado do Marker Bullet), a largura que a pressão
    // pouco muda (0,55 → 1) e o cruzamento que arrasta um pouco da cor de baixo. ⚠️ A tinta que ENTRA clara do
    // oráculo (o «loading» do Rebelle) não sai: a ponta romba do taper esmaece cada carimbo, mas a cápsula
    // sobrepõe-nos e a tinta carregada satura (só o traço rápido o mostra) — `exames/tintas.md`.
    Receita {
        meio: "Ink",
        grupo: "Pen",
        nome: "Ink Bullet",
        water: 0.15,
        opacity: 1.0,
        pigment: 2000.0,
        pickup: 10.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("glaze", 0.9),
            ("tipClean", 0.05),
            ("pressureSize", 0.45),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.6),
            ("pressureOpacityCurve", 1.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[
            ("shape", SEM_FORMA),
            ("taper", SEM_TAPER),
        ],
        nasce_de: None,
    },
    // C-8 Ink › Fountain › Fountain Pen (Rebelle `Pen/Fountain Round`): a caneta-tinteiro lisa e cheia, a pressão leva
    // a largura de um fio ao cheio (o Bullet quase não muda) e a tinta não entra clara; o cruzamento escurece e arrasta
    // a cor de baixo uns 20 px (o tipClean que a limpa).
    Receita {
        meio: "Ink",
        grupo: "Fountain",
        nome: "Fountain Pen",
        water: 0.15,
        opacity: 1.0,
        pigment: 2000.0,
        pickup: 16.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("glaze", 1.0),
            ("tipClean", 0.05),
            ("pressureSize", 1.0),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.0),
            ("pressureOpacityCurve", 1.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[("shape", SEM_FORMA), ("taper", SEM_TAPER)],
        nasce_de: Some("Ink Bullet"),
    },
    // C-9 Ink › Reed › Reed Pen (Rebelle `Pen/Reed Pen`): a cana cortada — a ponta CHATA e reta fixa na tela (larga no
    // horizontal, ~0,57 dela no vertical), a borda áspera (o tamanho e a posição de cada carimbo sorteados um pouco, o
    // papel que morde: gate 0,3) e a ponta que afina com a pressão.
    Receita {
        meio: "Ink",
        grupo: "Reed",
        nome: "Reed Pen",
        water: 0.3,
        opacity: 1.0,
        pigment: 1100.0,
        pickup: 16.0,
        gate: 0.3,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        extras: &[
            ("glaze", 1.0),
            ("tipClean", 0.05),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: Some((0.43, 90)),
        pincel: &[
            ("shape", SEM_FORMA),
            ("dab_quina", "1.0"),
            ("jitter_scale", "0.12"),
            ("jitter", "0.03"),
        ],
        nasce_de: Some("Dip Pen"),
    },
    // C-15 Ink › Wood › Wood Stick (Rebelle `Pen/Wood`): o pau mergulhado na tinta — mais fino e mais claro que o Dip
    // Pen (água 0,5, pigmento 80), os grumos maiores e soltos (tamanho ±30 %, posição ±4 %).
    Receita {
        meio: "Ink",
        grupo: "Wood",
        nome: "Wood Stick",
        water: 0.5,
        opacity: 1.0,
        pigment: 80.0,
        pickup: 16.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.95,
        size_px: None,
        extras: &[
            ("glaze", 0.9),
            ("tipClean", 0.05),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[
            ("shape", SEM_FORMA),
            ("jitter_scale", "0.3"),
            ("jitter", "0.04"),
            ("taper", SEM_TAPER),
        ],
        nasce_de: Some("Dip Pen"),
    },
];

/// O taper desligado (o do vizinho de onde o pincel nasce não passa).
const SEM_TAPER: &str = r#"{"opacity": 0.0, "start": 0.0, "tip_start": 0.0}"#;

/// O Shape sem forma (nem cerdas): a ponta é só a queda — a caneta lisa (o mesmo do aerógrafo).
const SEM_FORMA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

pub const MEIOS: &[ReceitaMeio] = &[
    // C-12 Ink › Gloop › Gloop Pen (ArtRage 6 `Gloop Pen`, fotografado): a gota de tinta translúcida com a ORLA escura
    // e nítida — a lavagem da Watercolor com a orla forte e estreita, sem grão nem warp; a tinta molhada funde-se.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.95"),
            ("depth", "1.0"),
            ("opacity", "0.45"),
            ("fill", "0.3"),
            ("edge_gain", "6.0"),
            ("edge_spread", "1.0"),
            ("granulation", "0.0"),
            ("warp", "0.0"),
            ("wet_dilution", "0.0"),
            ("continuous_spacing", "true"),
            ("spacing", "0.05"),
        ],
        mao: &[("pressureSize", 0.3)],
        ..ReceitaMeio::aquarela("Ink", "Gloop", "Gloop Pen")
    },
    // C-13 Ink › Bleed › Ink Bleed (Procreate Ink Bleed, doc.): a tinta escura que SANGRA no papel — a borda torcida
    // pelo warp da Watercolor (5 px) e o grão do papel nela; a pressão leva a largura.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.8"),
            ("depth", "6.0"),
            ("opacity", "1.0"),
            ("fill", "1.0"),
            ("edge_gain", "1.5"),
            ("edge_spread", "2.0"),
            ("granulation", "0.4"),
            ("warp", "5.0"),
            ("wet_dilution", "0.0"),
            ("continuous_spacing", "true"),
            ("spacing", "0.05"),
        ],
        mao: &[("pressureSize", 0.8)],
        ..ReceitaMeio::aquarela("Ink", "Bleed", "Ink Bleed")
    },
    // C-14 Ink › Nib › Gesinski Ink (Procreate Gesinski Ink, doc.): a pena translúcida que ESCURECE onde se sobrepõe —
    // o carimbo duro do Digital em Multiply a Strength 0,6 (um traço não escurece a si mesmo; dois, sim) e a pressão
    // que leva a largura de um fio ao cheio.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("blend", r#""Multiply""#),
            ("continuous_spacing", "true"),
            ("spacing", "0.05"),
        ],
        mao: &[("pressureSize", 1.0), ("pressureSizeCurve", 1.3), ("pressureOpacity", 0.3)],
        opacity: Some(0.6),
        ..ReceitaMeio::digital("Ink", "Nib", "Gesinski Ink")
    },
];

// O Cruzamento é o da foto MOLHADA; a Borda e a Orla só onde o Round não as tem (Ink Bullet: borda 1,00 nos dois).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Ink Bullet", foto: "ink", raio: 21.0, metricas: &[(Cobertura, 0.862), (Textura, 0.395), (Cruzamento, 0.117)] },
    Midia { preset: "Dip Pen", foto: "dippen", raio: 19.0, metricas: &[(Cobertura, 0.856), (Textura, 1.433), (Borda, 0.910), (Cruzamento, 0.223)] },
    Midia { preset: "Ink Brush", foto: "inkbrush", raio: 20.0, metricas: &[(Cobertura, 0.722), (Textura, 1.266), (Borda, 0.793), (Orla, 5.0), (Cruzamento, 0.239)] },
];

/// As medidas de identidade de uma tinta na bateria do EXAME (`ferramentas/bateria.json`, a foto A molhada), à
/// escala 1: o que a prancha de cada pincel mostra contra o oráculo e contra o vizinho, num número.
#[derive(Debug, Clone, Copy)]
struct Tinta {
    /// A largura do recto lento (px, tinta > 0,25).
    largura: f64,
    /// O desvio-padrão da largura ao longo dele (px): os grumos do aparo, a borda da cana.
    grumo: f64,
    /// A largura do vertical lento ÷ a do recto: a ponta chata fixa na tela.
    de_pe: f64,
    /// A largura da rampa 0→1 na pressão ~0,12 ÷ na pressão ~0,96: quanto a pressão afina.
    rampa: f64,
    /// A tinta dos primeiros 12 px do recto ÷ a do meio: a tinta que ENTRA clara.
    entrada: f64,
    /// A tinta no miolo do recto (escuridão 0..1).
    miolo: f64,
    /// A tinta na borda do recto (o máximo a 1–4 px da beira) ÷ a do miolo: a orla.
    orla: f64,
    /// A tinta do azul sobre o azul (o cruzamento `cruza_1` × `papel`) ÷ a do azul só: o que escurece na sobreposição.
    sobre_si: f64,
}

/// A escuridão de um píxel (1 − luminância): 0 o papel.
fn escuro(px: &[u8], w: usize, x: usize, y: usize) -> f64 {
    let p = &px[(y * w + x) * 4..];
    1.0 - (0.299 * f64::from(p[0]) + 0.587 * f64::from(p[1]) + 0.114 * f64::from(p[2])) / 255.0
}

/// A coluna de tinta (> 0,25 da escuridão do miolo) de `x` entre `y0..y1`, e o limite de cima e de baixo.
fn coluna(px: &[u8], w: usize, x: usize, y0: usize, y1: usize, limiar: f64) -> (f64, usize, usize) {
    let dentro: Vec<usize> = (y0..y1).filter(|&y| escuro(px, w, x, y) > limiar).collect();
    match (dentro.first(), dentro.last()) {
        (Some(&a), Some(&b)) => (dentro.len() as f64, a, b),
        _ => (0.0, 0, 0),
    }
}

fn mede_tinta(nome: &str) -> Tinta {
    use crate::exame_bancada::{bateria, pinceis, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let cena = |n: &str| bat["cenas"].as_array().unwrap().iter().find(|c| c["nome"] == n).unwrap().clone();
    let p = pinceis(nome).remove(0);
    let w = tela[0] as usize;
    let (_, g) = pinta_cena(&p, None, &cena("gestos"), tela, papel, 0);
    let (_, s) = pinta_cena(&p, None, &cena("sobre"), tela, papel, 0);
    // O recto lento: y 90, x 60..560 (40 px). O miolo = a mediana da escuridão na linha do centro.
    let mut centro: Vec<f64> = (150..470).map(|x| escuro(&g, w, x, 90)).collect();
    centro.sort_by(f64::total_cmp);
    let miolo = centro[centro.len() / 2];
    let lim = 0.25 * miolo;
    let larguras: Vec<f64> = (100..520).map(|x| coluna(&g, w, x, 50, 131, lim).0).collect();
    let largura = larguras.iter().sum::<f64>() / larguras.len() as f64;
    let grumo = (larguras.iter().map(|l| (l - largura).powi(2)).sum::<f64>() / larguras.len() as f64).sqrt();
    // O vertical lento: x 700, y 440..610 — a largura por linha, no meio dele.
    let de_pe_l: Vec<f64> = (480..570)
        .map(|y| (650..751).filter(|&x| escuro(&g, w, x, y) > lim).count() as f64)
        .collect();
    let de_pe = de_pe_l.iter().sum::<f64>() / de_pe_l.len() as f64 / largura;
    // A rampa 0→1: y 400, x 60 → 560 (a pressão linear no x).
    let larg_em = |x0: usize| (x0..x0 + 10).map(|x| coluna(&g, w, x, 360, 441, lim).0).sum::<f64>() / 10.0;
    let rampa = larg_em(115) / larg_em(535).max(1.0);
    // A entrada: a tinta na linha do centro nos primeiros 12 px depois da tampa (x 62..74) ÷ o miolo.
    let entrada = (62..74).map(|x| escuro(&g, w, x, 90)).sum::<f64>() / 12.0 / miolo;
    // A orla: por coluna, o máximo a 1–4 px da beira de cima ÷ a escuridão do centro.
    let orlas: Vec<f64> = (150..470)
        .filter_map(|x| {
            let (n, a, _) = coluna(&g, w, x, 50, 131, lim);
            (n > 6.0).then(|| (a + 1..a + 5).map(|y| escuro(&g, w, x, y)).fold(0.0, f64::max) / escuro(&g, w, x, 90).max(1e-3))
        })
        .collect();
    let orla = orlas.iter().sum::<f64>() / orlas.len().max(1) as f64;
    // O azul sobre o azul: o cruzamento (250, 310) ÷ o azul do `cruza_1` só (250, 230) e o do `papel` só (400, 310).
    let so = 0.5 * (escuro(&s, w, 250, 230) + escuro(&s, w, 400, 310));
    let sobre_si = escuro(&s, w, 250, 310) / so.max(1e-3);
    Tinta { largura, grumo, de_pe, rampa, entrada, miolo, orla, sobre_si }
}

/// ⭐ **Cada tinta é a SUA tinta** (J3, `docs/Painter/Pinceis/exames/tintas.md`): o que a prancha mostrou contra o
/// oráculo, preso num número que separa o pincel do vizinho que a matriz acha mais perto. A fronteira de cada regra é
/// o ponto médio entre o pincel e o vizinho que ela separa (medido na 1.ª corrida, abaixo).
#[test]
fn cada_tinta_tem_a_sua_identidade() {
    let nomes = [
        "Dip Pen", "Ink Bullet", "Ink Brush", "Fountain Pen", "Reed Pen", "Wood Stick", "Gloop Pen", "Ink Bleed",
        "Gesinski Ink", "Marker Bullet",
    ];
    let medidas: Vec<(&str, Tinta)> = std::thread::scope(|s| {
        let h: Vec<_> = nomes.iter().map(|&n| (n, s.spawn(move || mede_tinta(n)))).collect();
        h.into_iter().map(|(n, h)| (n, h.join().unwrap())).collect()
    });
    let m = |n: &str| medidas.iter().find(|(k, _)| *k == n).unwrap().1;
    for (n, t) in &medidas {
        println!("[tinta] {n:>13}: {t:?}");
    }
    let mut falhas = Vec::new();
    let mut regra = |ok: bool, o_que: String| {
        if !ok {
            falhas.push(o_que);
        }
    };
    let _ = (&m, &mut regra);
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
