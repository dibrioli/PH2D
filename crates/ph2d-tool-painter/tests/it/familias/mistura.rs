//! **Família `mistura`** — J4, catálogo L (mistura, borrador, borracha). O exame de cada um:
//! `docs/Painter/Pinceis/exames/mistura.md`. São FERRAMENTAS que não pintam: só se vêem sobre tinta, e a bancada pinta
//! as faixas da cena «sobre» com a base do meio delas (`exame_bancada::base_padrao`: o Hard Round no Digital — as
//! faixas lisas do Round Hard que o Rebelle usou nas fotos de SMUDGE/ERASER).

use super::prelude::*;

pub const WET: &[Receita] = &[];

pub const MEIOS: &[ReceitaMeio] = &[
    // L-8 — a borracha macia do Rebelle «Eraser/Default/Round Soft»: tira o ALFA em parte (uma passagem deixa ~0,73
    // da tinta), borda larga e macia; a pressão leva a força.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.0"),
            ("falloff", "\"Sharp\""),
            ("blend", "\"EraseAlpha\""),
        ],
        mao: &[("pressureSize", 0.0), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 1.0)],
        opacity: Some(0.26),
        ..ReceitaMeio::digital("Eraser", "Soft", "Kneaded Eraser")
    },
    // L-9 — a borracha dura do Rebelle «Eraser/Default/Square Hard»: tira TUDO, ponta quadrada, orla de ~2 px (a do
    // Rebelle: largura 39 px a 40, orla 2; a dureza 1 dava 40 · 0 — o corte binário; a 0,9 com a orla de 1 px, 40 · 2).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("borda_px", "1.0"),
            ("dab_quina", "1.0"),
            ("continuous_spacing", "true"),
            ("blend", "\"EraseAlpha\""),
        ],
        mao: &[("pressureSize", 0.0), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 1.0)],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Eraser", "Hard", "Hard Eraser")
    },
];

/// Reprovados nesta onda (o exame: `docs/Painter/Pinceis/exames/mistura.md`) — fora da fábrica, a receita e o portão
/// ficam para quando o motor/oráculo que lhes falta chegar.
pub const EM_ESTUDO: &[ReceitaMeio] = &[
    // REPROVADO (J4): o fio do arrasto. O V e o arrasto casam o Rebelle (o portão de baixo), mas o fio de 1–2 px que o
    // Rebelle leva CONTÍNUO até à faixa de baixo e que o traço «papel» puxa num rasto liso parte-se no nosso em pontos
    // (o campo de deslocamento re-amostrado a cada pingo perde o traço sub-píxel) — os dois revisores cegos viram-no.
    // L-3 — o dedo do Rebelle «Smudge/Default/Soft»: um CAMPO DE DESLOCAMENTO (o `smear_field` do Digital: cada dab
    // empurra o que está debaixo dele, a corrente relaia sem tecto) — a faixa abre-se num V e as paredes dele correm
    // para um fio que segue o gesto. A queda `Smooth` à dureza 0 dá o V do Rebelle no cruzamento 2 (a largura a y 130 ·
    // 140 · 160: 22 · 18 · 12 px × 21 · 15 · 9; a `Linear` 26 · 18 · 10 e o fio abaixo do papel 10 linhas × 130; a
    // `Smooth` 95); a Strength 1 (a 0,8–0,95 a corrente não relaia: o V encurta para um dente). A pressão leva a força
    // (`pressure_opacity_influence` 1 do preset dele), o tamanho não.
    ReceitaMeio {
        modo: Some("Smear"),
        pincel: &[("hardness", "0.0"), ("falloff", "\"Smooth\"")],
        mao: &[("pressureSize", 0.0), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 1.0)],
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Smudge", "Smudge", "Smudge Finger")
    },
    // REPROVADO (J4): a uma passagem à Strength 0,2 o desfoque casa o Krita (a orla abre 4 px), mas não se VÊ na
    // prancha (o revisor cego leu «nada pintado» nos dois) — sem um oráculo em que o desfoque se veja, não entra.
    // L-5 — o desfoque do Krita «x) Filter Blur»: o modo Blur do Digital, macio, que não tira tinta e só ABRE a orla do
    // que cruza; uma passagem é subtil (o artista repete). A Strength 0,2: a orla de cima da faixa abre 4 px no eixo,
    // como no Krita (0,4 → 6; 0,6 → 8; 1 → 10).
    ReceitaMeio {
        modo: Some("Blur"),
        pincel: &[("hardness", "0.0"), ("falloff", "\"Smooth\"")],
        mao: &[("pressureSize", 0.0), ("pressureOpacity", 1.0), ("pressureOpacityCurve", 1.0)],
        opacity: Some(0.2),
        ..ReceitaMeio::digital("Blend", "Soft", "Blur")
    },
];

#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[];

/// A identidade de uma ferramenta na cena `sobre` (foto A): o cruzamento 2 (x = 500) sobre a faixa de cima (y 120..180),
/// pelo verde (a faixa é vermelha: verde ~0, o papel 255) — as contas de `docs/Painter/Pinceis/exames/mistura_foto.py`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Identidade {
    /// O que fica da faixa no eixo (1 = intacta, 0 = papel).
    resto: f64,
    /// As colunas (x 420..580, y 150) que diferem da faixa intacta > 15 de 255.
    largura: f64,
    /// As colunas dessa linha com o resto entre 0,1 e 0,9, ÷ 2: a orla de cada lado.
    borda: f64,
    /// Quantos px abaixo da faixa (y > 180) o vermelho foi ARRASTADO (verde < 128 em x 480..520).
    fundo: f64,
    /// A orla de CIMA da faixa no eixo menos a da faixa intacta (x 200), em px: o desfoque abre-a.
    orla_v: f64,
}

#[derive(Clone, Copy, Debug)]
enum M {
    Resto,
    Largura,
    Borda,
    Fundo,
    OrlaV,
}

impl M {
    fn de(self, i: &Identidade) -> f64 {
        match self {
            M::Resto => i.resto,
            M::Largura => i.largura,
            M::Borda => i.borda,
            M::Fundo => i.fundo,
            M::OrlaV => i.orla_v,
        }
    }
    fn tol(self, foto: f64) -> f64 {
        match self {
            M::Resto => 0.08,
            M::Largura => 3.0 + 0.1 * foto,
            M::Borda => 1.5 + 0.2 * foto,
            M::Fundo => 25.0,
            M::OrlaV => 3.0 + 0.2 * foto.abs(),
        }
    }
}

impl Identidade {
    pub(crate) fn de(px: &[u8], w: usize) -> Self {
        let verde = |x: usize, y: usize| f64::from(px[(y * w + x) * 4 + 1]);
        let rgb = |x: usize, y: usize| {
            let i = (y * w + x) * 4;
            [px[i], px[i + 1], px[i + 2]].map(f64::from)
        };
        let mediana = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let y0 = 150;
        let refv = mediana((60..200).map(|x| verde(x, y0)).collect());
        let ref_rgb = [0, 1, 2].map(|k| mediana((60..200).map(|x| rgb(x, y0)[k]).collect()));
        let resto_de = |g: f64| ((255.0 - g) / (255.0 - refv).max(1.0)).clamp(0.0, 1.0);
        let linha: Vec<f64> = (420..=580).map(|x| resto_de(verde(x, y0))).collect();
        let resto = linha[78..83].iter().sum::<f64>() / 5.0;
        let largura = (420..=580)
            .filter(|&x| {
                let c = rgb(x, y0);
                (0..3).map(|k| (c[k] - ref_rgb[k]).abs()).fold(0.0, f64::max) > 15.0
            })
            .count() as f64;
        let borda = linha.iter().filter(|&&r| r > 0.1 && r < 0.9).count() as f64 / 2.0;
        let fundo = (y0 + 31..y0 + 150)
            .filter(|&y| (480..=520).any(|x| verde(x, y) < 128.0))
            .map(|y| (y - (y0 + 30)) as f64)
            .fold(0.0, f64::max);
        let orla = |x: usize| {
            (y0 - 50..=y0)
                .map(|y| resto_de(verde(x, y)))
                .filter(|&r| r > 0.1 && r < 0.9)
                .count() as f64
        };
        Self {
            resto,
            largura,
            borda,
            fundo,
            orla_v: orla(500) - orla(200),
        }
    }
}

/// As FOTOS (a cena `sobre`, A), medidas por `mistura_foto.py` (21/10): o Rebelle refeito com o SET_BRUSH repetido na
/// troca (`fixtures/rebelle_mistura/`), o Krita onde o Rebelle não tem a ferramenta.
#[rustfmt::skip]
const FOTOS: [(&str, &str, Identidade); 4] = [
    ("Smudge Finger", "rebelle SMUDGE/default_soft", Identidade { resto: 0.004, largura: 15.0, borda: 2.5, fundo: 119.0, orla_v: -2.0 }),
    ("Kneaded Eraser", "rebelle ERASER/default_round_soft", Identidade { resto: 0.72, largura: 31.0, borda: 14.5, fundo: 0.0, orla_v: 26.0 }),
    ("Hard Eraser", "rebelle ERASER/default_square_hard", Identidade { resto: 0.0, largura: 39.0, borda: 2.0, fundo: 0.0, orla_v: -4.0 }),
    ("Blur", "krita x) Filter Blur", Identidade { resto: 1.0, largura: 0.0, borda: 0.0, fundo: 0.0, orla_v: 4.0 }),
];

/// O que faz a identidade de cada uma.
const IDENTIDADE: [(&str, &[M]); 4] = [
    ("Smudge Finger", &[M::Resto, M::Largura, M::Fundo]),
    ("Kneaded Eraser", &[M::Resto, M::Largura, M::Borda, M::Fundo]),
    ("Hard Eraser", &[M::Resto, M::Largura, M::Borda, M::Fundo]),
    ("Blur", &[M::Resto, M::OrlaV, M::Fundo]),
];

/// A cena `sobre` (foto A) pintada por `nome` sobre a base do meio dele (`exame_bancada::base_padrao`): os píxeis.
pub(crate) fn sobre_de(nome: &str, preset: Option<ph2d_brush_presets::Preset>) -> Vec<u8> {
    use crate::exame_bancada::{Fonte, Pincel, base_padrao, bateria, pinceis, pinta_cena};
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let sobre = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == "sobre")
        .unwrap();
    let p = match preset {
        Some(pr) => Pincel {
            nome: nome.to_owned(),
            fonte: Fonte::Preset {
                preset: Box::new(pr),
                relativo: String::new(),
            },
        },
        None => Pincel {
            nome: nome.to_owned(),
            fonte: Fonte::Preset {
                preset: Box::new(preset_da_receita(nome)),
                relativo: String::new(),
            },
        },
    };
    let base = pinceis(base_padrao(ph2d_tool_painter::PaintMedia::Digital)).remove(0);
    // A camada SOBRE o papel (o que o artista vê; a borracha só mexe no alfa) — a foto da bancada (`escreve_ppm`).
    let mut px = pinta_cena(&p, Some(&base), sobre, tela, papel, 0).1;
    for c in px.as_chunks_mut::<4>().0 {
        let a = u32::from(c[3]);
        for k in 0..3 {
            c[k] = ((u32::from(c[k]) * a + u32::from(papel[k]) * (255 - a) + 127) / 255) as u8;
        }
        c[3] = 255;
    }
    px
}

/// O preset de uma receita da família (a fábrica ou o `EM_ESTUDO`), pela mesma porta do gerador — sem ficheiro.
fn preset_da_receita(nome: &str) -> ph2d_brush_presets::Preset {
    let r = MEIOS.iter().chain(EM_ESTUDO).find(|r| r.nome == nome).unwrap();
    let (parametros, texturas) = crate::presets_de_fabrica::parametros_do_meio(r);
    ph2d_brush_presets::Preset {
        parametros,
        origem: ph2d_brush_presets::Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura { largura: 1, altura: 1, rgba: vec![255; 4] },
        texturas,
    }
}

/// As falhas da identidade de `nome` medida em `eu` contra a sua foto, e a lei dos IRMÃOS (estritamente mais perto,
/// em tolerâncias somadas, da SUA foto que da de cada uma das outras).
fn falhas_da_identidade(nome: &str, eu: &Identidade) -> Vec<String> {
    let (_, oraculo, foto) = FOTOS.iter().find(|f| f.0 == nome).unwrap();
    let ms = IDENTIDADE.iter().find(|f| f.0 == nome).unwrap().1;
    let mut falhas = Vec::new();
    for &m in ms {
        let (v, f) = (m.de(eu), m.de(foto));
        if (v - f).abs() > m.tol(f) {
            falhas.push(format!("{nome}: {m:?} {v:.3} fora da foto {f:.3} ± {:.3} ({oraculo})", m.tol(f)));
        }
    }
    let distancia = |outra: &Identidade| -> f64 {
        ms.iter().map(|&m| (m.de(eu) - m.de(outra)).abs() / m.tol(m.de(foto))).sum()
    };
    let propria = distancia(foto);
    for (outro, _, f2) in FOTOS.iter().filter(|f| f.0 != nome) {
        let d = distancia(f2);
        if propria >= d {
            falhas.push(format!("{nome}: {propria:.2} tolerâncias da sua foto, {d:.2} da do {outro} — faz o irmão"));
        }
    }
    falhas
}

/// ⭐ **Cada ferramenta da mistura reconhece-se pela foto do seu oráculo** (J4, `docs/Painter/Pinceis/exames/mistura.md`):
/// na cena `sobre` da bateria, sobre as faixas lisas do Hard Round, cada medida da identidade cai na tolerância da foto, e
/// cada uma fica mais perto da SUA foto que das das irmãs — o dedo arrasta o vermelho ~120 px abaixo da faixa (a borracha
/// não arrasta nada), a borracha macia deixa ~0,72 da tinta (a dura nenhuma, numa orla de 1–2 px), o desfoque não tira
/// tinta e só abre a orla da faixa.
#[test]
fn cada_ferramenta_da_mistura_se_reconhece_pela_foto() {
    let medidas: Vec<(&str, Identidade)> = std::thread::scope(|s| {
        let hs: Vec<_> = FOTOS
            .iter()
            .map(|&(nome, _, _)| s.spawn(move || (nome, Identidade::de(&sobre_de(nome, None), 1000))))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (nome, eu) in &medidas {
        let foto = FOTOS.iter().find(|f| f.0 == *nome).unwrap();
        println!("[mistura] {nome}: {eu:?}\n          foto {}: {:?}", foto.1, foto.2);
        falhas.extend(falhas_da_identidade(nome, eu));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// **A prova de mutação do portão, permanente** (J4): cada mutação do valor que FAZ a identidade de uma ferramenta — o
/// preset de fábrica com um campo trocado, pintado pela mesma porta — tem de sangrar `falhas_da_identidade`. Sem isto o
/// portão de cima podia passar por medir tolerâncias largas demais.
#[test]
fn o_portao_da_mistura_sangra_com_cada_mutacao() {
    use serde_json::json;
    let muta = |nome: &str, f: &dyn Fn(&mut ph2d_brush_presets::Parametros)| {
        let mut p = preset_da_receita(nome);
        f(&mut p.parametros);
        p
    };
    let mutacoes: Vec<(&str, &str, ph2d_brush_presets::Preset)> = vec![
        // o dedo com a Strength 0,8: a corrente não relaia (o V encurta para um dente)
        ("Smudge Finger", "Strength 0,8", muta("Smudge Finger", &|p| p.volumes.opacity = Some(0.8))),
        // o dedo a PINTAR (sem o modo Smear): é um disco macio
        ("Smudge Finger", "modo Paint", muta("Smudge Finger", &|p| p.paint_mode = Some("Paint".to_owned()))),
        // a borracha macia a tirar tudo
        ("Kneaded Eraser", "Strength 1", muta("Kneaded Eraser", &|p| p.volumes.opacity = Some(1.0))),
        // a borracha dura com a borda macia
        ("Hard Eraser", "dureza 0", muta("Hard Eraser", &|p| {
            p.brush.insert("hardness".to_owned(), json!(0.0));
        })),
        // o desfoque na Strength 1 (abre a orla 10 px, não 4)
        ("Blur", "Strength 1", muta("Blur", &|p| p.volumes.opacity = Some(1.0))),
    ];
    let sangrou: Vec<(&str, &str, Vec<String>)> = std::thread::scope(|s| {
        let hs: Vec<_> = mutacoes
            .into_iter()
            .map(|(nome, o_que, p)| {
                s.spawn(move || {
                    let eu = Identidade::de(&sobre_de(nome, Some(p)), 1000);
                    println!("[mistura] mutação {nome} · {o_que}: {eu:?}");
                    (nome, o_que, falhas_da_identidade(nome, &eu))
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
    assert!(sobreviveu.is_empty(), "mutações que o portão NÃO apanha: {sobreviveu:?}");
}
