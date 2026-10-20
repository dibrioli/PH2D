//! **Família `tintas`** — J3, catálogo C (as tintas do Wet Paint): Dip Pen, Ink Bullet, Ink Brush, Reed
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
    // salpicava a borda), as RISCAS de cerda ao longo do traço (o claro de cada fio sorteado, 0,25; o 2.º revisor cego
    // via «grão de pontinhos» onde o Rebelle tem riscas) — `docs/Painter/Pinceis/exames/tintas.md`.
    Receita {
        meio: "Ink",
        grupo: "Brush",
        nome: "Ink Brush",
        water: 0.5,
        opacity: 1.0,
        pigment: 150.0,
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
            ("bristleLaneWidth", 1.0),
            ("bristleNoiseLight", 0.25),
            ("bristleTipRagged", 0.8),
            ("bristlePressure", 0.6),
        ],
        grao: None,
        achatado: None,
        pincel: &[("shape", SEM_FORMA)],
        nasce_de: None,
    },
    // Ink › Pen › Dip Pen: duro e cheio, a borda GRUMOSA do aparo (o tamanho de cada carimbo sorteado ±18 %: os
    // grumos do Rebelle `Pen/Dip Pen`), o cruzamento que escurece (Glaze 0,9) e arrasta um pouco da cor de baixo.
    Receita {
        meio: "Ink",
        grupo: "Pen",
        nome: "Dip Pen",
        water: 0.3,
        opacity: 1.0,
        // 300 e não 1100: o pigmento a mais SATURA a cápsula e a ponta leve da rampa ficava opaca (a do Rebelle é clara);
        // abaixo de ~300 a borda clareia e sai do vale da foto `dippen` (Borda 0,84 a 150, 0,88 a 300; Rebelle 0,91).
        pigment: 300.0,
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
        pincel: &[("jitter_scale", "0.18"), ("shape", SEM_FORMA)],
        nasce_de: None,
    },
    // Ink › Pen › Ink Bullet: o mais carregado e seco da tinta. J3 (Rebelle `Pen/Bullet`): a ponta redonda LISA (sem
    // a forma de cerdas que esfiapava as pontas — o que a punha ao lado do Marker Bullet), a largura que a pressão
    // pouco muda (0,55 → 1) e o cruzamento que arrasta um pouco a cor. ⚠️ A tinta que ENTRA clara do Rebelle (o
    // «loading») não sai: a ponta romba do taper só se vê com o pigmento a ~200, e aí a entrada sai PONTILHADA (o 2.º
    // revisor cego pôs o nosso ao lado do Marker Liner) — a tinta lisa a 2000 fica; `exames/tintas.md`.
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
    // C-9 Ink › Reed › Reed Pen (Rebelle `Pen/Reed Pen`): a cana cortada — a ponta CHATA e reta fixa na tela (larga no
    // horizontal, ~0,57 dela no vertical), a borda áspera (o tamanho e a posição de cada carimbo sorteados um pouco, o
    // papel que morde: gate 0,6) e a ponta que afina com a pressão.
    Receita {
        meio: "Ink",
        grupo: "Reed",
        nome: "Reed Pen",
        water: 0.3,
        opacity: 1.0,
        pigment: 200.0,
        pickup: 16.0,
        gate: 0.6,
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
            ("pressureOpacityCurve", 2.5),
        ],
        grao: None,
        achatado: Some((0.43, 90)),
        pincel: &[
            ("shape", SEM_FORMA),
            ("dab_quina", "0.8"),
            ("jitter_scale", "0.15"),
            ("jitter", "0.04"),
        ],
        nasce_de: Some("Dip Pen"),
    },
    // C-15 Ink › Wood › Wood Stick (Rebelle `Pen/Wood`): o pau mergulhado na tinta — mais fino e mais claro que o Dip
    // Pen (água 0,5, pigmento 60 — a 80 o revisor cego via-o mais escuro que o Rebelle, a 40 mais claro — a tinta que
    // a pressão leve deixa transparente), os grumos maiores e soltos
    // (tamanho ±30 %, posição ±4 %).
    Receita {
        meio: "Ink",
        grupo: "Wood",
        nome: "Wood Stick",
        water: 0.5,
        opacity: 1.0,
        pigment: 60.0,
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
            ("taper", r#"{"opacity": 0.6, "start": 1.5, "tip_start": 1.0}"#),
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
    // e nítida que MULTIPLICA onde se sobrepõe — o traço do Digital como uma camada (L8: tecto 0,7 e a orla queimada
    // de 2,5 px) em Multiply; a pressão afina a ponta (a rampa do ArtRage nasce fina) e o traço rápido afina um pouco
    // (o ArtRage responde à velocidade; 0,25 virava agulha — 2.º revisor cego). A 1.ª receita (a lavagem da Watercolor) fundia a tinta molhada e clareava o vermelho
    // por baixo: `exames/tintas.md`.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.95"),
            ("blend", r#""Multiply""#),
            ("continuous_spacing", "true"),
            ("spacing", "0.05"),
            (
                "traco",
                r#"{"largura": 2.5, "molhada": 0.0, "queimada": 0.8, "tecto": 0.7}"#,
            ),
        ],
        mao: &[("pressureSize", 0.8), ("speedSize", 0.1)],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Ink", "Gloop", "Gloop Pen")
    },
    // C-13 Ink › Bleed › Ink Bleed (Procreate Ink Bleed, doc.): a tinta escura que SANGRA pelas fibras — a lavagem
    // da Watercolor de tinta cheia (depth 6) com a borda que segue o papel (paper_edge 1) e torcida pelo warp (5 px),
    // a orla leve do nanquim, e a tinta que puxa e se dilui na que ainda está molhada (wet_pull 0,5); a pressão leva a
    // largura. Sem o puxar, a matriz punha-a a 0,49 do Hard Round (um redondo opaco com a borda rugosa).
    ReceitaMeio {
        pincel: &[
            ("hardness", "1.0"),
            ("depth", "6.0"),
            ("opacity", "1.0"),
            ("fill", "1.0"),
            ("edge_gain", "1.5"),
            ("edge_spread", "2.0"),
            ("granulation", "0.4"),
            ("paper_edge", "1.0"),
            ("smooth_edges", "false"),
            ("warp", "5.0"),
            ("wet_pull", "0.5"),
            ("wet_dilution", "0.3"),
            ("continuous_spacing", "true"),
            ("spacing", "0.05"),
        ],
        mao: &[("pressureSize", 1.0)],
        ..ReceitaMeio::aquarela("Ink", "Bleed", "Ink Bleed")
    },
    // C-14 Ink › Nib › Gesinski Ink (Procreate Gesinski Ink, doc.): a pena translúcida que ESCURECE onde se sobrepõe —
    // o carimbo duro do Digital em Multiply com o tecto POR TRAÇO (L8, 0,55: um traço não escurece a si mesmo, nem na
    // espiral nem a 200 px; dois, sim) e a pressão que leva a largura de um FIO ao cheio (0,9: a ponta leve guarda um
    // fio, o 2.º revisor cego via-a sumir) e fica fina por mais tempo (curva 1,4).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.95"),
            ("blend", r#""Multiply""#),
            ("continuous_spacing", "true"),
            ("spacing", "0.05"),
            (
                "traco",
                r#"{"largura": 7.0, "molhada": 0.0, "queimada": 0.0, "tecto": 0.55}"#,
            ),
        ],
        mao: &[("pressureSize", 0.9), ("pressureSizeCurve", 1.4), ("pressureOpacity", 0.3)],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Ink", "Nib", "Gesinski Ink")
    },
];

// O Cruzamento é o da foto MOLHADA; a Borda e a Orla só onde o Round não as tem (Ink Bullet: borda 1,00 nos dois).
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Ink Bullet", foto: "ink", raio: 21.0, metricas: &[(Cobertura, 0.862), (Textura, 0.395), (Cruzamento, 0.117)] },
    Midia { preset: "Dip Pen", foto: "dippen", raio: 19.0, metricas: &[(Cobertura, 0.856), (Textura, 1.433), (Borda, 0.910), (Cruzamento, 0.223)] },
    // ⚠️ Sem a Orla (5 px na foto `inkbrush` de 5 gestos): era a franja da forma de CERDAS, que a bateria do EXAME
    // (Rebelle `Brush/Brush`, três cenas) desmente — o perfil do recto dela casa com os lados nítidos (régua: perfil ✓
    // nas 3 cenas) e o revisor cego leu a franja como «granulado/salpicado». `exames/tintas.md`.
    Midia { preset: "Ink Brush", foto: "inkbrush", raio: 20.0, metricas: &[(Cobertura, 0.722), (Textura, 1.266), (Borda, 0.793), (Cruzamento, 0.239)] },
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
        "Dip Pen", "Ink Bullet", "Ink Brush", "Reed Pen", "Wood Stick", "Gloop Pen", "Ink Bleed",
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
    // As fronteiras: o meio entre o pincel e o que a regra separa, medidos na corrida final (20/10, `exames/tintas.md`).
    let (dip, bullet, brush, reed) = (m("Dip Pen"), m("Ink Bullet"), m("Ink Brush"), m("Reed Pen"));
    let (wood, gloop, bleed, gesinski) = (m("Wood Stick"), m("Gloop Pen"), m("Ink Bleed"), m("Gesinski Ink"));
    // Os grumos do aparo (1,80) contra a ponta lisa do Bullet (0,42).
    regra(dip.grumo > 1.11, format!("Dip Pen: grumo {:.2} — a borda do aparo ficou lisa", dip.grumo));
    // O Bullet quase não afina (0,60) contra o Dip Pen (0,18) e o Marker Bullet (0,07); e é liso.
    regra(bullet.rampa > 0.39, format!("Ink Bullet: rampa {:.2} — a pressão afina-o como uma pena", bullet.rampa));
    regra(bullet.grumo < 1.11, format!("Ink Bullet: grumo {:.2} — a ponta deixou de ser lisa", bullet.grumo));
    // A tinta RALA do pincel (pigmento 150): entra clara (0,92 contra ≥ 0,98 das canetas) e o miolo é mais claro que o
    // do Dip Pen (0,61 contra 0,66); e os lados lisos.
    regra(brush.entrada < 0.95, format!("Ink Brush: entrada {:.2} — a tinta não entra rala", brush.entrada));
    regra(brush.miolo < 0.635, format!("Ink Brush: miolo {:.2} — a tinta é a carregada das canetas", brush.miolo));
    regra(brush.grumo < 1.11, format!("Ink Brush: grumo {:.2} — os lados ficaram grumosos", brush.grumo));
    // A ponta chata fixa: de pé 0,64 da largura deitada contra ~1,0 dos redondos.
    regra(reed.de_pe < 0.82, format!("Reed Pen: de pé ÷ deitado {:.2} — a cana ficou redonda", reed.de_pe));
    // O pau contra o Dip Pen: os grumos maiores (2,62 contra 1,80) e a tinta que ENTRA clara (0,88 contra 0,99).
    regra(wood.grumo > 2.21, format!("Wood Stick: grumo {:.2} — os grumos são os do Dip Pen", wood.grumo));
    regra(wood.entrada < 0.94, format!("Wood Stick: entrada {:.2} — a tinta não entra clara", wood.entrada));
    // A orla escura da gota (1,60 contra 1,00 da pena), o que multiplica (1,50) e a largura que a pressão leva pouco
    // (0,32 contra a pena Gesinski, 0,16).
    regra(gloop.orla > 1.24, format!("Gloop Pen: orla {:.2} — perdeu o aro", gloop.orla));
    regra(gloop.sobre_si > 1.36, format!("Gloop Pen: sobreposição {:.2} — não multiplica", gloop.sobre_si));
    regra(gloop.rampa > 0.24, format!("Gloop Pen: rampa {:.2} — a pressão afina-a como a pena", gloop.rampa));
    // A borda que sangra (2,84 contra o Hard Round, o vizinho da matriz, 0).
    regra(bleed.grumo > 1.35, format!("Ink Bleed: grumo {:.2} — a borda deixou de sangrar", bleed.grumo));
    // A pena translúcida que escurece na sobreposição (1,60 contra 1,13 do Bullet), clara (0,36), sem aro e que afina.
    regra(gesinski.sobre_si > 1.36, format!("Gesinski Ink: sobreposição {:.2} — não escurece", gesinski.sobre_si));
    regra(gesinski.miolo < 0.51, format!("Gesinski Ink: miolo {:.2} — a tinta ficou opaca", gesinski.miolo));
    regra(gesinski.orla < 1.24, format!("Gesinski Ink: orla {:.2} — tem o aro da gota", gesinski.orla));
    regra(gesinski.rampa < 0.24, format!("Gesinski Ink: rampa {:.2} — a pressão não a afina", gesinski.rampa));
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
