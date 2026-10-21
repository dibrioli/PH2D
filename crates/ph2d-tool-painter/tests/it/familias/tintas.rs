//! **Família `tintas`** — J3, catálogo C (as tintas do Wet Paint): Dip Pen, Ink Bullet, Ink Brush. Gloop Pen, Ink
//! Bleed, Gesinski Ink e Wood Stick saíram na 2.ª onda, o Reed Pen (reprovado) no fecho da J3
//! (`docs/Painter/Pinceis/exames/tintas.md`). Escreva aqui as receitas
//! e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).

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
        // 2.ª onda: o BORRÃO do cruzamento (o reservatório 0,2 com a ponta que se limpa — sem a limpeza o traço inteiro
        // ficava preto; a 0,1 não passava a faixa, Rebelle 11 px —, Glaze 0,4: o cruzamento do Rebelle é claro, L 53).
        extras: &[
            ("extDiffusion", 0.01),
            ("glaze", 0.4),
            ("leveling", 0.5),
            ("tipReservoir", 0.2),
            ("tipClean", 0.05),
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
    // Ink › Pen › Dip Pen: duro e cheio, a borda GRUMOSA do aparo (o tamanho de cada carimbo sorteado ±10 %: os
    // grumos do Rebelle `Pen/Dip Pen`), o cruzamento que escurece (Glaze 0,75) e arrasta um pouco da cor de baixo.
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
        // Fecho da J3 (a lei das camadas do dono, `a_lei_da_tinta::a_terceira_camada_da_mesma_tinta_nao_clareia`): a
        // borda DURA (1,0) com os grumos a ±10 % (grumo 1,15; o Bullet liso 0,42) — a 0,9 × ±18 % a orla de cada grumo
        // entrava no miolo de uma camada (p95 0,1476 contra o tecto 0,1451; a ±11 % ainda 0,1452).
        hardness: 1.0,
        size_px: None,
        // 2.ª onda: o BORRÃO (o reservatório 0,4 — a 0,8 só +2 px e o cruzamento ao preto) e o tecto do Rebelle (Max Size
        // 100 = 91 px). O Glaze 0,75 (fecho da J3): a 0,5 da 2.ª onda a 2.ª e a 3.ª camada da mesma tinta davam o MESMO
        // tom (medianas 0,1157 · 0,1157) — a lei do dono de 16/10; a 0,75 cada camada escurece (0,1037 · 0,1000 · 0,0967).
        extras: &[
            ("glaze", 0.75),
            ("tipClean", 0.05),
            ("leveling", 0.5),
            ("tipReservoir", 0.4),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.9),
            ("pressureOpacityCurve", 1.5),
        ],
        grao: None,
        achatado: None,
        pincel: &[("jitter_scale", "0.10"), ("shape", SEM_FORMA), ("size_max_px", "91")],
        nasce_de: None,
    },
    // Ink › Pen › Ink Bullet: o mais carregado e seco da tinta. J3 (Rebelle `Pen/Bullet`): a ponta redonda LISA (sem
    // a forma de cerdas que esfiapava as pontas — o que a punha ao lado do Marker Bullet), a largura que a pressão
    // pouco muda (0,55 → 1) e o cruzamento que arrasta um pouco a cor. ⚠️ A tinta que ENTRA clara do Rebelle (o
    // «loading») não sai: a ponta romba do taper só se vê com o pigmento a ~200, e aí a entrada sai PONTILHADA (o 2.º
    // revisor cego pôs o nosso ao lado do Marker Liner) — a tinta lisa fica; `exames/tintas.md`.
    // ⭐ Fecho da J3 — a MÃO DO DONO (`leis_sistemicas::a_pressao_de_cada_preset_e_a_da_sua_origem`, R_MAO 2,5): o Rebelle
    // quase não responde na faixa 0,24–0,86 (1,39) e a curadoria da J3 seguiu-o (1,46: «apertar muito e pouco variam
    // pouco», o que o dono reprovou a 18/10). A mão responde pela CARGA: pigmento 1000 (a 2000 a tinta SATURA desde a
    // pressão 0,3 e a carga não se vê), o chão 0,01 e a curva 4 — a pressão leve ainda pinta (rampa 0,51; sem chão a
    // tinta sumia abaixo de 0,15) e a cheia carrega (mão 2,89, a largura a 0,073 da do Rebelle). O Glaze 0,4 (era 0,5):
    // a 0,5 a tinta de várias camadas CLAREAVA com a água (`as_bordas_nao_esbranquicam…`, 1923 células; a 0,4, 4).
    Receita {
        meio: "Ink",
        grupo: "Pen",
        nome: "Ink Bullet",
        water: 0.15,
        opacity: 1.0,
        pigment: 1000.0,
        pickup: 10.0,
        gate: 0.0,
        evaporation: 1.0,
        edge: 0.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
        hardness: 0.9,
        size_px: None,
        // 2.ª onda: o borrão e o tecto (93 px) como o Dip Pen.
        extras: &[
            ("glaze", 0.4),
            ("tipClean", 0.05),
            ("leveling", 0.5),
            ("tipReservoir", 0.4),
            ("pressureSize", 0.45),
            ("pressureSizeCurve", 1.0),
            ("pressureOpacity", 0.99),
            ("pressureOpacityCurve", 4.0),
        ],
        grao: None,
        achatado: None,
        pincel: &[
            ("shape", SEM_FORMA),
            ("taper", SEM_TAPER),
            ("size_max_px", "93"),
        ],
        nasce_de: None,
    },
];

/// O taper desligado (o do vizinho de onde o pincel nasce não passa).
const SEM_TAPER: &str = r#"{"opacity": 0.0, "start": 0.0, "tip_start": 0.0}"#;

/// O Shape sem forma (nem cerdas): a ponta é só a queda — a caneta lisa (o mesmo do aerógrafo).
const SEM_FORMA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

// 2.ª onda (`exames/tintas.md`): o Gloop Pen, o Ink Bleed e o Gesinski Ink saíram — reprovados de novo pelo revisor
// cego (a orla granulada e a pressão do ArtRage; a capilaridade pelas fibras; a densidade viva sob o tecto: motor).
pub const MEIOS: &[ReceitaMeio] = &[];

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
    /// O desvio-padrão da largura do recto lento (px, tinta > 0,25): os grumos do aparo, a borda da cana.
    grumo: f64,
    /// A largura do vertical lento ÷ a do recto: a ponta chata fixa na tela.
    de_pe: f64,
    /// A largura da rampa 0→1 na pressão ~0,12 ÷ na pressão ~0,96: quanto a pressão afina.
    rampa: f64,
    /// A tinta dos primeiros 12 px do recto ÷ a do meio: a tinta que ENTRA clara.
    entrada: f64,
    /// A tinta no miolo do recto (escuridão 0..1).
    miolo: f64,
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
    Tinta { grumo, de_pe, rampa, entrada, miolo }
}

/// ⭐ **Cada tinta é a SUA tinta** (J3, `docs/Painter/Pinceis/exames/tintas.md`): o que a prancha mostrou contra o
/// oráculo, preso num número que separa o pincel do vizinho que a matriz acha mais perto. A fronteira de cada regra é
/// o ponto médio entre o pincel e o vizinho que ela separa (medido na 1.ª corrida, abaixo).
#[test]
fn cada_tinta_tem_a_sua_identidade() {
    let nomes = [
        "Dip Pen", "Ink Bullet", "Ink Brush", "Marker Bullet",
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
    let (dip, bullet, brush) = (m("Dip Pen"), m("Ink Bullet"), m("Ink Brush"));
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
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// O BORRÃO do cruzamento (cena `sobre`, foto A, o vertical `cruza_1` em x 250): quantos px a MISTURA (≥ 3 píxeis por
/// linha numa janela de ±60) passa da borda de cima e de baixo da faixa centrada em `yc` — o `borrao.py` da 2.ª onda
/// (`exames/tintas.md`).
fn borrao(px: &[u8], yc: usize) -> (usize, usize, f64) {
    let w = 1000usize;
    let rgb = |x: usize, y: usize| {
        let i = (y * w + x) * 4;
        [f64::from(px[i]), f64::from(px[i + 1]), f64::from(px[i + 2])]
    };
    let mediana = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let cor_mediana = |pts: &[[f64; 3]]| [0, 1, 2].map(|c| mediana(pts.iter().map(|p| p[c]).collect()));
    // As bordas e o vermelho limpo da faixa, a meio entre os verticais (x 375).
    let vermelhos: Vec<(usize, [f64; 3])> = (yc - 50..yc + 50)
        .flat_map(|y| (372..379).map(move |x| (y, x)))
        .map(|(y, x)| (y, rgb(x, y)))
        .filter(|(_, c)| c[0] > 110.0 && c[2] < 110.0)
        .collect();
    let (topo, fundo) = (vermelhos.iter().map(|v| v.0).min().unwrap(), vermelhos.iter().map(|v| v.0).max().unwrap());
    // O azul limpo: o vertical sobre o papel entre as faixas.
    let azuis: Vec<[f64; 3]> = (240..280)
        .flat_map(|y| (210..291).map(move |x| rgb(x, y)))
        .filter(|c| c[2] > c[0] + 40.0 && c.iter().sum::<f64>() < 690.0)
        .collect();
    let azul = cor_mediana(&azuis);
    // A mistura vermelho × azul: o verde cai abaixo de 0,65 do azul limpo (o vermelho tem G 0; os grumos do próprio azul
    // não descem tanto) e não é o vermelho.
    let da_mistura = |y: usize| -> Vec<f64> {
        (190..311)
            .map(|x| rgb(x, y))
            .filter(|c| c.iter().sum::<f64>() < 690.0 && c[1] < 0.65 * azul[1] && c[0] < 110.0)
            .map(|c| c[0])
            .collect()
    };
    let mistura = |y: usize| da_mistura(y).len() >= 3;
    let acima = (1..90).take_while(|&k| mistura(topo - k)).count();
    let abaixo = (1..90).take_while(|&k| mistura(fundo + k)).count();
    // O vermelho da mistura acima da faixa (a mediana do R): o produto guarda o R do azul (Rebelle 26–39); a média
    // das duas cores subia-o para ~100 — o roxo.
    let r: Vec<f64> = (1..=acima).flat_map(|k| da_mistura(topo - k)).collect();
    (acima, abaixo, if r.is_empty() { 0.0 } else { mediana(r) })
}

/// ⭐⭐ **O borrão do cruzamento: a ponta de tinta LÍQUIDA é um reservatório** (J3 tintas, 2.ª onda; `Knob::TipReservoir`).
/// Rebelle, as cinco tintas na cena `sobre` (`borrao.py`, acima/abaixo): a mistura passa a faixa SECA 11–38 / 5–29 px e
/// a MOLHADA 11–50 / 2–29 (o pincel apanha a tinta, também a seca, debaixo da pegada inteira e pousa a mistura já
/// escurecida — o produto, R 26–39, nunca o roxo). Antes da lei o nosso dava 0 px acima (o rectângulo nítido dos
/// revisores cegos). ⚠️ Fica de fora a subida A MAIS da molhada no Dip e no Reed (45 e 40 contra 21 e 26 na seca): é a
/// mistura a escoar no azul molhado, física que o filme fino da tinta não exprime (`exames/tintas.md`, 2.ª onda). O
/// controlo: o Marker Bullet (sem a lei; o do Rebelle 0) continua nítido.
#[test]
fn o_borrao_da_tinta_transborda_a_faixa() {
    use crate::as_leis_da_escala::cena;
    use crate::exame_bancada::pinceis;
    let nomes = ["Dip Pen", "Ink Bullet", "Ink Brush", "Marker Bullet"];
    let medidas: Vec<(&str, (usize, usize, f64), (usize, usize, f64))> = std::thread::scope(|s| {
        let h: Vec<_> = nomes
            .iter()
            .map(|&n| {
                (n, s.spawn(move || {
                    let px = cena(&pinceis(n).remove(0), "sobre");
                    (borrao(&px, 150), borrao(&px, 470))
                }))
            })
            .collect();
        h.into_iter().map(|(n, h)| {
            let (m, sc) = h.join().unwrap();
            (n, m, sc)
        }).collect()
    });
    let mut falhas = Vec::new();
    for &(n, molhada, seca) in &medidas {
        println!("[borrão] {n:>13}: molhada acima/abaixo {molhada:?} · seca {seca:?}");
        if n == "Marker Bullet" {
            if seca.0 > 2 || molhada.0 > 2 {
                falhas.push(format!("{n}: o marcador (sem a lei) borrou {molhada:?} · {seca:?}"));
            }
            continue;
        }
        // O Rebelle (`borrao.py`, x 250): acima da molhada · acima da seca.
        let (reb_m, reb_s) = match n {
            "Dip Pen" => (45.0, 21.0),
            "Ink Bullet" => (40.0, 38.0),
            "Ink Brush" => (11.0, 11.0),
            _ => unreachable!("{n}"),
        };
        if (seca.0 as f64) < 0.4 * reb_s || seca.1 < 4 {
            falhas.push(format!("{n}: a faixa SECA não borra ({seca:?}; ≥ 0,4 × Rebelle {reb_s}) — o reservatório"));
        }
        if (molhada.0 as f64) < 0.25 * reb_m {
            falhas.push(format!("{n}: a MOLHADA borra pouco ({molhada:?}; ≥ 0,25 × Rebelle {reb_m}) — o reservatório"));
        }
        if seca.2 > 60.0 {
            falhas.push(format!("{n}: a mistura acima da seca é ROXA (R {:.0}; Rebelle 26–39) — o produto", seca.2));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
