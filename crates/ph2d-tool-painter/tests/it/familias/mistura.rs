//! **Família `mistura`** — J4, catálogo L (mistura, borrador, borracha). O exame de cada um:
//! `docs/Painter/Pinceis/exames/mistura.md`. São FERRAMENTAS que não pintam: só se vêem sobre tinta, e a bancada pinta
//! as faixas da cena «sobre» com a base do meio delas (`exame_bancada::base_padrao`: o Hard Round no Digital — as
//! faixas lisas do Round Hard que o Rebelle usou nas fotos de SMUDGE/ERASER).

use super::prelude::*;

mod motor;
mod sonda_agua;

pub const WET: &[Receita] = &[];

/// L-1, o «Blender Soft» (a base das ferramentas da água: o resto das receitas delas não é lido — `ferramenta_da_agua`).
const BLENDER_SOFT: Receita = Receita {
    meio: "Blend",
    grupo: "Soft",
    nome: "Blender Soft",
    water: 1.0,
    opacity: 1.0,
    pigment: 78.0,
    pickup: 1.0,
    gate: 0.4,
    evaporation: 1.0,
    edge: 12.0,
    tilt: DIREITA,
    bristle_size: 2.0,
    bristle_count: 4000.0,
    hardness: 0.0,
    size_px: None,
    extras: &[("blendForce", 0.5), ("felt", 0.2)],
    grao: None,
    achatado: None,
    pincel: &[],
    nasce_de: Some("Wet Round"),
    real_oil: false,
};

/// O feltro dos Smear: o do Wet Round. O 0,2 do Blender Soft (de onde eles nascem) junta as cerdas e o Smear quase não
/// arrasta (janela 15, com o corpo a chegar à ferramenta — Smear Soft na cena `sobre`: fundo 59 a 0,2, 110 a 0,03 · 0,
/// contra o Rebelle 119; Shattered 25 → 17 contra 0).
const FELTRO_DO_SMEAR: f64 = 0.03;

/// As FERRAMENTAS do Wet (o modo ao lado da receita, `receitas_de_fabrica::ReceitaFerramenta`).
pub const WET_FERRAMENTAS: &[ReceitaFerramenta] = &[
    // L-1 — o misturador do Rebelle «Blend/Default/Soft»: a ferramenta Blend do Wet (o mesmo modelo: mistura a tinta
    // MOLHADA sob a pegada e esbate-a para cima e para baixo, sem a arrastar longe; A = B). Nasce do Wet Round. O
    // feltro no máximo (0,2) junta as cerdas — sem ele a mistura sai em riscas verticais; a força 0,5, sobre a faixa
    // do Ink Brush e à largura do Rebelle (escala 0,45 — o «40» do Blend dele mistura ~33 px; o nosso 37): o vermelho
    // levado acima e abaixo da faixa (o verde mínimo a y 100·110·115 · 185·190·200·210) 138·56·36 · 125·139·177·207
    // contra o Rebelle 131·78·48 · 107·130·183·217, o alcance acima · abaixo 24 · 21 px contra 26 · 23 (a 0,4: 19 · 21,
    // o que os dois revisores da 1.ª prancha viram — «o original sobe mais»; a força 0,8: 25 · 16).
    ReceitaFerramenta {
        receita: BLENDER_SOFT,
        modo: "Blend",
    },
    // As do Rebelle (N4, dono 25/10: «presets para Smear e Blend»), medidas na cena `sobre` contra `rebelle_bateria/BLEND`
    // (`sonda_agua`): «Default/Textured» — força ×5 e o toque espalhado 0,35 (a rotação dele não age aqui); a nossa força BAIXA dá a orla
    // mais alta (0,3: 17 · 0,5: 13 · 1: 9) e o espalhamento abre-a: resto 0,986 · orla 15 · fundo 5 contra 0,986 · 19 · 20.
    ReceitaFerramenta {
        receita: Receita {
            meio: "Blend",
            grupo: "Textured",
            nome: "Blender Textured",
            extras: &[("blendForce", 0.3), ("felt", 0.2)],
            pincel: &[("jitter", "0.35")],
            nasce_de: Some("Blender Soft"),
            ..BLENDER_SOFT
        },
        modo: "Blend",
    },
    // «Default/Dots»: o Textured com o toque menos espalhado (0,125): 0,996 · 17 · 1 contra 0,981 · 19 · 20.
    ReceitaFerramenta {
        receita: Receita {
            meio: "Blend",
            grupo: "Dots",
            nome: "Blender Dots",
            extras: &[("blendForce", 0.3), ("felt", 0.2)],
            pincel: &[("jitter", "0.125")],
            nasce_de: Some("Blender Soft"),
            ..BLENDER_SOFT
        },
        modo: "Blend",
    },
    // «Default/Hard»: mais denso e a pressão → força 0,8 com piso 0,2 (o do Rebelle); a dureza não age nas ferramentas da
    // água (o motor usa a delas). Com o felt 0,2 a chegar à ferramenta (janela 15) a orla sobe: força 0,6 → 14 · 0,8 → 12
    // · 1 → 11, contra o Rebelle 8 ± 4,6 (resto 0,996 · fundo 3 contra 0,955 · 12).
    ReceitaFerramenta {
        receita: Receita {
            meio: "Blend",
            grupo: "Hard",
            nome: "Blender Hard",
            extras: &[
                ("blendForce", 1.0),
                ("felt", 0.2),
                ("pressureOpacity", 0.8),
                ("pressureFloor", 0.2),
            ],
            nasce_de: Some("Blender Soft"),
            ..BLENDER_SOFT
        },
        modo: "Blend",
    },
    // O Smear da água (o SMUDGE do Rebelle), à escala 0,5 da cena: «Default/Soft» — a pressão leva a força, não o tamanho.
    // O arrasto casa (fundo 110 contra 119); o canal LIMPO do Rebelle (resto 0,09 em 15 px) o motor não abre a nenhuma
    // escala ou força (o rasto só leva pigmento: a 0,6–0,7 o resto desce a 0,95–0,81, a largura sobe a 12–41).
    ReceitaFerramenta {
        receita: Receita {
            meio: "Smear",
            grupo: "Soft",
            nome: "Smear Soft",
            extras: &[
                ("smear", 1.0),
                ("pressureSize", 0.0),
                ("pressureOpacity", 1.0),
                ("felt", FELTRO_DO_SMEAR),
            ],
            nasce_de: Some("Blender Soft"),
            ..BLENDER_SOFT
        },
        modo: "Smear",
    },
    // «Default/Spikey»: a pressão leva o tamanho (1) e a força (0,7, piso 0,3) — difere do Soft só sob a caneta, como lá.
    ReceitaFerramenta {
        receita: Receita {
            meio: "Smear",
            grupo: "Spikey",
            nome: "Smear Spikey",
            extras: &[
                ("smear", 1.0),
                ("pressureSize", 1.0),
                ("pressureOpacity", 0.7),
                ("pressureFloor", 0.3),
                ("felt", FELTRO_DO_SMEAR),
            ],
            nasce_de: Some("Blender Soft"),
            ..BLENDER_SOFT
        },
        modo: "Smear",
    },
    // «Default/Flat»: a força do toque 0,95 (o arrasto 81 contra 79); a ponta chata dele não entra — o carimbo das
    // ferramentas da água é o redondo delas (censo `cada_ferramenta_da_agua_so_ve_o_que_le`: Shape e Marca inertes).
    ReceitaFerramenta {
        receita: Receita {
            meio: "Smear",
            grupo: "Flat",
            nome: "Smear Flat",
            extras: &[
                ("smear", 0.95),
                ("pressureSize", 0.3),
                ("pressureOpacity", 1.0),
                ("felt", FELTRO_DO_SMEAR),
            ],
            nasce_de: Some("Blender Soft"),
            ..BLENDER_SOFT
        },
        modo: "Smear",
    },
    // «Default/Shattered»: o toque espalhado 0,25 (a rotação inteira dele não age aqui); a opacidade aleatória dele (média ~½) é a força
    // 0,6: o arrasto 18 contra 0 (a 1: 40).
    ReceitaFerramenta {
        receita: Receita {
            meio: "Smear",
            grupo: "Shattered",
            nome: "Smear Shattered",
            extras: &[("smear", 0.6), ("felt", FELTRO_DO_SMEAR)],
            pincel: &[("jitter", "0.25")],
            nasce_de: Some("Blender Soft"),
            ..BLENDER_SOFT
        },
        modo: "Smear",
    },
    // NOSSO (dono 25/10: «Smear tem água em seu efeito»): o Smear que MOLHA o rasto (Water 0,9) — dissolve o assente e
    // abre a tinta (resto 0,59, largura 117 px); o Rebelle não tem.
    ReceitaFerramenta {
        receita: Receita {
            meio: "Smear",
            grupo: "Wet",
            nome: "Smear Wet",
            extras: &[
                ("smear", 1.0),
                ("smearWater", 0.9),
                ("felt", FELTRO_DO_SMEAR),
            ],
            nasce_de: Some("Blender Soft"),
            ..BLENDER_SOFT
        },
        modo: "Smear",
    },
];

pub const MEIOS: &[ReceitaMeio] = &[
    // L-8 — a borracha macia do Rebelle «Eraser/Default/Round Soft»: tira o ALFA em parte (uma passagem deixa ~0,73
    // da tinta), borda larga e macia; a pressão leva a força.
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.0"),
            ("falloff", "\"Sharp\""),
            ("blend", "\"EraseAlpha\""),
        ],
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
        ],
        opacity: Some(0.26),
        ferramenta: Some("Eraser"),
        ..ReceitaMeio::digital("Eraser", "Soft", "Kneaded Eraser")
    },
    // L-9 — a borracha dura do Rebelle «Eraser/Default/Square Hard»: tira TUDO, ponta quadrada, orla de ~2 px (a do
    // Rebelle: largura 39 px a 40, orla 2; a dureza 1 dava 40 · 0 — o corte binário; a 0,9 com a orla de 1 px, 40 · 2).
    // A mão do Procreate/Krita (dono 24/10: «a dura apaga a fundo e a pressão mexe no TAMANHO»): a pressão→opacidade
    // pedia 2–4 passadas com a caneta a 0,24–0,86; a pressão→tamanho é a de fábrica (0,9 = o raio de 10 % a 100 %).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.9"),
            ("borda_px", "1.0"),
            ("dab_quina", "1.0"),
            ("continuous_spacing", "true"),
            ("blend", "\"EraseAlpha\""),
        ],
        mao: &[("pressureSize", 0.9), ("pressureOpacity", 0.0)],
        opacity: Some(1.0),
        ferramenta: Some("Eraser"),
        ..ReceitaMeio::digital("Eraser", "Hard", "Hard Eraser")
    },
    // A borracha de TEXTURA (dono 24/10): apaga pelo grão — o miolo e a orla em manchas de ruído; a pressão no tamanho.
    // A Strength 0,9: uma passada tira 139 ± 39 do alfa (a 1 o grão não morde: 255 ± 0 — `sonda_ferramentas_de_fabrica`).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.6"),
            ("texture", GRAO_DA_BORRACHA),
            ("blend", "\"EraseAlpha\""),
        ],
        mao: &[("pressureSize", 0.9), ("pressureOpacity", 0.0)],
        opacity: Some(0.9),
        ferramenta: Some("Eraser"),
        ..ReceitaMeio::digital("Eraser", "Texture", "Texture Eraser")
    },
    // Os básicos do Blur, do Smear e do Clone (dono 24/10): o pincel de fábrica do slot de cada um (Spacing 0,05 —
    // estas ferramentas não leem o Continuous, `NAO_LE`), a borda macia; a pressão no tamanho.
    ReceitaMeio {
        pincel: &[("hardness", "0.0")],
        opacity: Some(1.0),
        ferramenta: Some("Blur"),
        ..ReceitaMeio::digital("Basic", "Blur", "Blur Basic")
    },
    ReceitaMeio {
        pincel: &[("hardness", "0.0")],
        opacity: Some(0.8),
        ferramenta: Some("Smear"),
        ..ReceitaMeio::digital("Basic", "Smear", "Smear Basic")
    },
    ReceitaMeio {
        pincel: &[("hardness", "0.5")],
        opacity: Some(1.0),
        ferramenta: Some("Clone"),
        ..ReceitaMeio::digital("Basic", "Clone", "Clone Basic")
    },
];

/// O grão da Texture Eraser: o ruído do papel em manchas de ~16 px (Noise `Tiled`).
const GRAO_DA_BORRACHA: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [64.0, 64.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

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
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
        ],
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
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
        ],
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
                (0..3)
                    .map(|k| (c[k] - ref_rgb[k]).abs())
                    .fold(0.0, f64::max)
                    > 15.0
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
const FOTOS: [(&str, &str, Identidade); 5] = [
    ("Smudge Finger", "rebelle SMUDGE/default_soft", Identidade { resto: 0.004, largura: 15.0, borda: 2.5, fundo: 119.0, orla_v: -2.0 }),
    ("Kneaded Eraser", "rebelle ERASER/default_round_soft", Identidade { resto: 0.72, largura: 31.0, borda: 14.5, fundo: 0.0, orla_v: 26.0 }),
    ("Hard Eraser", "rebelle ERASER/default_square_hard", Identidade { resto: 0.0, largura: 39.0, borda: 2.0, fundo: 0.0, orla_v: -4.0 }),
    ("Blur", "krita x) Filter Blur", Identidade { resto: 1.0, largura: 0.0, borda: 0.0, fundo: 0.0, orla_v: 4.0 }),
    ("Blender Soft", "rebelle BLEND/default_soft", Identidade { resto: 0.984, largura: 0.0, borda: 0.0, fundo: 9.0, orla_v: 17.0 }),
];

/// A escala da ferramenta na cena (a do exame: a largura medida igual à do oráculo; as faixas da base à escala 1).
const ESCALA_DO_EXAME: [(&str, f64); 9] = [
    ("Blender Soft", 0.45),
    ("Blender Textured", 0.45),
    ("Blender Dots", 0.45),
    ("Blender Hard", 0.45),
    ("Smear Soft", 0.5),
    ("Smear Spikey", 0.5),
    ("Smear Flat", 0.5),
    ("Smear Shattered", 0.5),
    ("Smear Wet", 0.5),
];

/// O que faz a identidade de cada uma.
const IDENTIDADE: [(&str, &[M]); 5] = [
    ("Smudge Finger", &[M::Resto, M::Largura, M::Fundo]),
    (
        "Kneaded Eraser",
        &[M::Resto, M::Largura, M::Borda, M::Fundo],
    ),
    ("Hard Eraser", &[M::Resto, M::Largura, M::Borda, M::Fundo]),
    ("Blur", &[M::Resto, M::OrlaV, M::Fundo]),
    ("Blender Soft", &[M::Resto, M::OrlaV, M::Fundo]),
];

/// A cena `sobre` (foto A) pintada por `nome` sobre a base do meio dele (`exame_bancada::base_padrao`): os píxeis.
pub(crate) fn sobre_de(nome: &str, preset: Option<ph2d_brush_presets::Preset>) -> Vec<u8> {
    let escala = ESCALA_DO_EXAME
        .iter()
        .find(|e| e.0 == nome)
        .map_or(1.0, |e| e.1);
    sobre_a_escala(nome, preset, escala)
}

/// [`sobre_de`] com a ferramenta à `escala` dada.
pub(crate) fn sobre_a_escala(
    nome: &str,
    preset: Option<ph2d_brush_presets::Preset>,
    escala: f64,
) -> Vec<u8> {
    use crate::exame_bancada::{Fonte, Pincel, base_padrao, bateria, pinceis, pinta_cena};
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
    // A ferramenta à escala do exame (os traços depois do `pincel: preset`); as faixas da base ficam.
    let mut da_ferramenta = false;
    for op in sobre["ops"].as_array_mut().unwrap() {
        if op["op"] == "pincel" {
            da_ferramenta = op["quem"] == "preset";
        }
        if da_ferramenta && let Some(d) = op.get("size_px").and_then(serde_json::Value::as_f64) {
            op["size_px"] = serde_json::json!(d * escala);
        }
    }
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
    let Fonte::Preset { preset: pr, .. } = &p.fonte else {
        unreachable!()
    };
    let meio = ph2d_tool_painter::meio_do_preset(&pr.parametros.media).unwrap();
    let base = pinceis(base_padrao(meio, p.modo())).remove(0);
    // A camada SOBRE o papel (o que o artista vê; a borracha só mexe no alfa) — a foto da bancada (`escreve_ppm`).
    let mut px = pinta_cena(&p, Some(&base), &sobre, tela, papel, 0).1;
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
    // Uma ferramenta do Wet: o ficheiro de fábrica (a receita do Wet parte dele, com a tabela de largura).
    if WET_FERRAMENTAS.iter().any(|f| f.receita.nome == nome) {
        return crate::midias_de_fabrica::de_fabrica(nome);
    }
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

/// As falhas da identidade de `nome` medida em `eu` contra a sua foto, e a lei dos IRMÃOS (estritamente mais perto,
/// em tolerâncias somadas, da SUA foto que da de cada uma das outras).
fn falhas_da_identidade(nome: &str, eu: &Identidade) -> Vec<String> {
    let (_, oraculo, foto) = FOTOS.iter().find(|f| f.0 == nome).unwrap();
    let ms = IDENTIDADE.iter().find(|f| f.0 == nome).unwrap().1;
    let mut falhas = Vec::new();
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
    for (outro, _, f2) in FOTOS.iter().filter(|f| f.0 != nome) {
        let d = distancia(f2);
        if propria >= d {
            falhas.push(format!(
                "{nome}: {propria:.2} tolerâncias da sua foto, {d:.2} da do {outro} — faz o irmão"
            ));
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
            .map(|&(nome, _, _)| {
                s.spawn(move || (nome, Identidade::de(&sobre_de(nome, None), 1000)))
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (nome, eu) in &medidas {
        let foto = FOTOS.iter().find(|f| f.0 == *nome).unwrap();
        println!(
            "[mistura] {nome}: {eu:?}\n          foto {}: {:?}",
            foto.1, foto.2
        );
        falhas.extend(falhas_da_identidade(nome, eu));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// As fotos das ferramentas da água (N4): `rebelle_bateria/{BLEND,SMUDGE}/default_*_sobre_A.png` por `mistura_foto.py`
/// (o cruzamento 2: a troca de ferramenta das fotos antigas só erra o tamanho do 1.º). O que cada uma MEDE: o Blend o
/// resto, a orla e o fundo; o Smear o arrasto (o canal limpo do SMUDGE o motor não abre — a receita do Smear Soft) e,
/// no Shattered, o resto (o dele quase não toca a faixa).
#[rustfmt::skip]
const FOTOS_DA_AGUA: [(&str, &str, Identidade, &[M]); 7] = [
    ("Blender Textured", "BLEND/default_textured", Identidade { resto: 0.986, largura: 0.0, borda: 0.0, fundo: 20.0, orla_v: 19.0 }, &[M::Resto, M::OrlaV, M::Fundo]),
    ("Blender Dots", "BLEND/default_dots", Identidade { resto: 0.981, largura: 0.0, borda: 0.0, fundo: 20.0, orla_v: 19.0 }, &[M::Resto, M::OrlaV, M::Fundo]),
    ("Blender Hard", "BLEND/default_hard", Identidade { resto: 0.955, largura: 0.0, borda: 0.0, fundo: 12.0, orla_v: 8.0 }, &[M::Resto, M::OrlaV, M::Fundo]),
    ("Smear Soft", "SMUDGE/default_soft", Identidade { resto: 0.091, largura: 15.0, borda: 3.5, fundo: 119.0, orla_v: -6.0 }, &[M::Fundo]),
    ("Smear Spikey", "SMUDGE/default_spikey", Identidade { resto: 0.091, largura: 15.0, borda: 3.5, fundo: 119.0, orla_v: -4.0 }, &[M::Fundo]),
    ("Smear Flat", "SMUDGE/default_flat", Identidade { resto: 0.091, largura: 13.0, borda: 1.5, fundo: 79.0, orla_v: -4.0 }, &[M::Fundo]),
    ("Smear Shattered", "SMUDGE/default_shattered", Identidade { resto: 0.996, largura: 0.0, borda: 0.0, fundo: 0.0, orla_v: -1.0 }, &[M::Resto, M::Fundo]),
];

/// ⭐ **Cada preset das ferramentas da água cai na foto do seu Rebelle** (N4, dono 25/10): na cena `sobre`, cada medida
/// que a foto dele faz cai na tolerância. E o «Smear Wet» (nosso, o Water do Smear): MOLHA o rasto — dissolve a faixa
/// que o Smear Soft só arrasta (o resto cai mais de 0,2 abaixo do dele).
#[test]
fn cada_preset_da_agua_cai_na_foto_do_seu_rebelle() {
    let nomes: Vec<&str> = FOTOS_DA_AGUA
        .iter()
        .map(|f| f.0)
        .chain(["Smear Wet"])
        .collect();
    let medidas: Vec<(&str, Identidade)> = std::thread::scope(|s| {
        let hs: Vec<_> = nomes
            .iter()
            .map(|&nome| s.spawn(move || (nome, Identidade::de(&sobre_de(nome, None), 1000))))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let de = |n: &str| medidas.iter().find(|m| m.0 == n).unwrap().1;
    let mut falhas = Vec::new();
    for (nome, oraculo, foto, ms) in &FOTOS_DA_AGUA {
        let eu = de(nome);
        println!("[agua] {nome}: {eu:?}\n       foto {oraculo}: {foto:?}");
        for &m in *ms {
            let (v, f) = (m.de(&eu), m.de(foto));
            if (v - f).abs() > m.tol(f) {
                falhas.push(format!(
                    "{nome}: {m:?} {v:.3} fora da foto {f:.3} ± {:.3} ({oraculo})",
                    m.tol(f)
                ));
            }
        }
    }
    let (molhado, soft) = (de("Smear Wet").resto, de("Smear Soft").resto);
    println!("[agua] Smear Wet: resto {molhado:.3} · Smear Soft {soft:.3}");
    if molhado > soft - 0.2 {
        falhas.push(format!(
            "Smear Wet: resto {molhado:.3} — não dissolve mais que o Smear Soft ({soft:.3})"
        ));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// ⭐ **Aplicar um preset de uma ferramenta da água não mexe no Pincel** (N4, dono 25/10): com o Pincel da água pintado
/// por um preset, cada preset de fábrica do Smear e do Blend põe a SUA ferramenta na mão — e, de volta ao Pincel, o
/// pincel, a mão do traço, os knobs do motor e a largura prometida são os de antes, ao bit.
#[test]
fn aplicar_um_preset_da_agua_nao_mexe_no_pincel() {
    use ph2d_tool_painter::{FerramentaDoPincel as F, PainterTool};
    let mut falhas = Vec::new();
    for f in WET_FERRAMENTAS {
        let nome = f.receita.nome;
        let mut t = PainterTool::default();
        t.preset_aplicar(&crate::midias_de_fabrica::de_fabrica("Ink Brush"))
            .unwrap();
        let antes = t.retrato_do_pincel_da_agua();
        t.preset_aplicar(&crate::midias_de_fabrica::de_fabrica(nome))
            .unwrap();
        let esperada = if f.modo == "Smear" {
            F::SmearMolhado
        } else {
            F::BlendMolhado
        };
        if t.ferramenta_dos_presets() != Some(esperada) {
            falhas.push(format!(
                "{nome}: na mão {:?}, não {esperada:?}",
                t.ferramenta_dos_presets()
            ));
        }
        t.set_paint_tool_mode("wet_paint");
        let depois = t.retrato_do_pincel_da_agua();
        if depois != antes {
            falhas.push(format!("{nome}: o Pincel mudou"));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
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
        (
            "Smudge Finger",
            "Strength 0,8",
            muta("Smudge Finger", &|p| p.volumes.opacity = Some(0.8)),
        ),
        // o dedo a PINTAR (sem o modo Smear): é um disco macio
        (
            "Smudge Finger",
            "modo Paint",
            muta("Smudge Finger", &|p| {
                p.paint_mode = Some("Paint".to_owned())
            }),
        ),
        // a borracha macia a tirar tudo
        (
            "Kneaded Eraser",
            "Strength 1",
            muta("Kneaded Eraser", &|p| p.volumes.opacity = Some(1.0)),
        ),
        // a borracha dura com a borda macia
        (
            "Hard Eraser",
            "dureza 0",
            muta("Hard Eraser", &|p| {
                p.brush.insert("hardness".to_owned(), json!(0.0));
            }),
        ),
        // o desfoque na Strength 1 (abre a orla 10 px, não 4)
        (
            "Blur",
            "Strength 1",
            muta("Blur", &|p| p.volumes.opacity = Some(1.0)),
        ),
        // o misturador sem força: não mistura nada
        (
            "Blender Soft",
            "blendForce 0",
            muta("Blender Soft", &|p| {
                p.engine.insert("blendForce".to_owned(), 0.0);
            }),
        ),
        // o misturador a PINTAR (o modo Paint): uma faixa azul por cima
        (
            "Blender Soft",
            "modo Paint",
            muta("Blender Soft", &|p| p.paint_mode = Some("Paint".to_owned())),
        ),
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
    assert!(
        sobreviveu.is_empty(),
        "mutações que o portão NÃO apanha: {sobreviveu:?}"
    );
}

/// A tela 256 × 128 cheia de vermelho opaco com o preset `p` aplicado pela porta do navegador, o pincel a 40 px.
fn borracha_na_tela(p: &ph2d_brush_presets::Preset) -> ph2d_tool_painter::PainterTool {
    use ph2d_editor_core::tool::RasterEditTool;
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_source([191u8, 0, 18, 255].repeat(256 * 128), 256, 128);
    assert_eq!(t.preset_aplicar(p), Ok(Vec::new()));
    t.set_brush_size_px(20.0);
    t
}

/// ⭐ **A borracha dura de fábrica apaga numa passada com a caneta** (dono 24/10, a lei do Procreate/Krita: «a dura apaga
/// a fundo e a pressão mexe no TAMANHO»): aplicada, põe a Borracha na mão; UMA passada à pressão 0,24 · 0,5 · 0,86 · 1
/// (a mão do dono e o rato) deixa o alfa 0 no centro, e a largura apagada CRESCE com a pressão. Com a pressão→opacidade
/// de antes a caneta pedia 2–4 passadas.
#[test]
fn a_borracha_dura_de_fabrica_apaga_numa_passada_com_a_caneta() {
    use ph2d_editor_core::tool::RasterEditTool;
    let p = preset_da_receita("Hard Eraser");
    let mut larguras = Vec::new();
    let mut falhas = Vec::new();
    for pressao in [0.24f32, 0.5, 0.86, 1.0] {
        let mut t = borracha_na_tela(&p);
        assert_eq!(
            t.ferramenta_dos_presets(),
            Some(ph2d_tool_painter::FerramentaDoPincel::Borracha),
            "a Hard Eraser não pôs a Borracha na mão"
        );
        crate::sonda_gestos::gesto(&mut t, &[[30.0, 64.0], [226.0, 64.0]], 3.0, |_| pressao);
        let (px, w, _) = t.run_full();
        let alfa = |y: usize| px[(y * w as usize + 128) * 4 + 3];
        let largura = (0..128).filter(|&y| alfa(y) < 128).count();
        if alfa(64) != 0 {
            falhas.push(format!(
                "pressão {pressao}: o centro ficou com alfa {} numa passada",
                alfa(64)
            ));
        }
        println!(
            "[borracha dura] pressão {pressao}: centro {} · largura {largura} px",
            alfa(64)
        );
        larguras.push(largura);
    }
    if !larguras.windows(2).all(|l| l[0] < l[1]) {
        falhas.push(format!("a largura não cresce com a pressão: {larguras:?}"));
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// ⭐ **Cada ferramenta tem a SUA biblioteca de fábrica** (dono 24/10, o padrão do Procreate; as da água, 25/10): os
/// presets de fábrica cuja ferramenta é a Borracha, o Blur, o Smear, o Clone, o Smear ou o Blend da água são EXACTAMENTE
/// os da lista, e cada um, aplicado, põe essa ferramenta na mão com o meio dela (Digital, ou o Wet Paint).
#[test]
fn cada_ferramenta_tem_os_seus_presets_de_fabrica() {
    use ph2d_tool_painter::{
        FerramentaDoPincel as F, PaintMedia, PainterTool, ferramenta_do_preset,
    };
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let esperado: [(F, &[&str]); 6] = [
        (
            F::Borracha,
            &["Hard Eraser", "Kneaded Eraser", "Texture Eraser"],
        ),
        (F::Blur, &["Blur Basic"]),
        (F::Smear, &["Smear Basic"]),
        (F::Clone, &["Clone Basic"]),
        (
            F::SmearMolhado,
            &[
                "Smear Flat",
                "Smear Shattered",
                "Smear Soft",
                "Smear Spikey",
                "Smear Wet",
            ],
        ),
        (
            F::BlendMolhado,
            &[
                "Blender Dots",
                "Blender Hard",
                "Blender Soft",
                "Blender Textured",
            ],
        ),
    ];
    for (f, nomes) in esperado {
        let mut achados: Vec<&str> = b
            .entradas
            .iter()
            .filter(|e| ferramenta_do_preset(&e.preset.parametros) == f)
            .map(|e| e.preset.parametros.name.as_str())
            .collect();
        achados.sort_unstable();
        assert_eq!(achados, nomes, "a biblioteca de fábrica de {f:?}");
        for e in b
            .entradas
            .iter()
            .filter(|e| ferramenta_do_preset(&e.preset.parametros) == f)
        {
            let mut t = PainterTool::default();
            assert_eq!(
                t.preset_aplicar(&e.preset),
                Ok(Vec::new()),
                "{}",
                e.relativo
            );
            assert_eq!(
                t.ferramenta_dos_presets(),
                Some(f),
                "{}: a ferramenta na mão",
                e.relativo
            );
            let meio = if f.ferramenta_da_agua().is_some() {
                PaintMedia::WetPaint
            } else {
                PaintMedia::Digital
            };
            assert_eq!(t.paint_media(), meio, "{}: o meio", e.relativo);
        }
    }
}
