//! **AS FIGURAS do tutorial do ciclo 7** (a cor e o rasto — doc 112, passo 6).
//!
//! ⚠️ **Elas não são desenhos:** cada quadrado é uma peça que o motor devolveu ao correr a cena
//! `=118` do produto — com a POSIÇÃO, o TAMANHO, a COR e a ALFA que ele lhe deu. *Uma figura
//! desenhada à mão é a ilustração de uma teoria, não a saída dela.*
//!
//! ⚠️⚠️ **As duas primeiras pares pintam os DADOS** (a coluna `tint` vira a cor do quadrado, e a
//! alfa a opacidade dele): o assunto delas é a cor e o apagar da cauda, e um ponto na cor da
//! paleta seria cego aos dois. ⭐ **A paleta partilhada** ([`tutorial_draw`](super::tutorial_draw))
//! continua a dar o fundo, e o par do TEMPO usa o desenho dos outros ciclos inteiro (o repouso
//! como fantasma, e o traço até onde a peça está).
//!
//! ⛔⛔ **A ESCRITA VEM DEPOIS DAS ASSERÇÕES** — a lei que o ciclo 4 pagou.

use super::tutorial_draw as draw;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// As coordenadas de mundo ganham esta escala antes de virar a figura — a `moldura` partilhada
/// tem uma folga mínima de `6` unidades, pensada para os panos grandes dos outros ciclos.
const ESCALA: f32 = 100.0;
const DT: f64 = 1.0 / 60.0;

/// Corre a cena `=118` (pela porta `monta`, nunca uma cena montada à mão) `ticks` quadros e devolve,
/// por sink, o stream dele e o do `motion.move` a montante (a pose de REPOUSO), no último quadro.
fn colher(ticks: u64) -> Vec<(Stream, Stream)> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let (sinks, _) = crate::motion_demo_legend::monta("118", &mut doc, &reg);
    let repouso: Vec<NodeId> = sinks
        .iter()
        .map(|s| {
            ph2d_nodegraph::cook::upstream_cone(&doc.graph, *s)
                .into_iter()
                .find(|n| {
                    doc.graph
                        .node(*n)
                        .is_some_and(|i| i.type_name == "motion.move")
                })
                .expect("o pano")
        })
        .collect();
    let mut cook = Cook::new();
    let mut fora = Vec::new();
    for f in 0..ticks {
        let t = f as f64 * DT;
        fora.clear();
        for (s, r) in sinks.iter().zip(&repouso) {
            let a = cook.cook(&doc.graph, &reg, *s, t).expect("coze")[0]
                .as_stream()
                .clone();
            let b = cook.cook(&doc.graph, &reg, *r, t).expect("coze")[0]
                .as_stream()
                .clone();
            fora.push((a, b));
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("tique");
    }
    fora
}

fn vec2(s: &Stream, col: &str) -> Vec<[f32; 2]> {
    match s.get(col) {
        Some(Column::Vec2(v)) => v.clone(),
        outra => panic!("a coluna `{col}` devia ser Vec2: {outra:?}"),
    }
}

fn vec4(s: &Stream, col: &str) -> Vec<[f32; 4]> {
    match s.get(col) {
        Some(Column::Vec4(v)) => v.clone(),
        // O `tint` ausente é o branco opaco da identidade (a lowering da casa).
        None => vec![[1.0; 4]; s.count()],
        outra => panic!("a coluna `{col}` devia ser Vec4: {outra:?}"),
    }
}

fn escala(p: &[[f32; 2]]) -> Vec<[f32; 2]> {
    p.iter().map(|q| [q[0] * ESCALA, q[1] * ESCALA]).collect()
}

/// O canal linear (a coluna `tint`) no sRGB que um SVG espera.
fn srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round() as u8
}

/// Um quadrado por peça, com a COR, a ALFA e o TAMANHO que o motor lhe deu.
fn svg_pintado(s: &Stream, (cx, cy, w, h): (f32, f32, f32, f32)) -> String {
    const LARGURA_PX: f32 = 340.0;
    let (p, cor) = (escala(&vec2(s, "P")), vec4(s, "tint"));
    let lado: Vec<f32> = match s.get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|z| z[0] * ESCALA).collect(),
        _ => vec![ESCALA; p.len()],
    };
    let mut o = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.2} {:.2} {w:.2} {h:.2}\" \
         width=\"{LARGURA_PX:.0}\" height=\"{:.0}\" role=\"img\">\n\
         <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"{:.2}\" fill=\"{}\"/>\n",
        cx - w / 2.0,
        -cy - h / 2.0,
        LARGURA_PX * h / w,
        cx - w / 2.0,
        -cy - h / 2.0,
        w.min(h) / 26.0,
        draw::cor::FUNDO,
    );
    // A ordem é a do stream: a cauda primeiro, a cabeça por cima — a mesma do renderizador.
    for ((q, c), l) in p.iter().zip(&cor).zip(&lado) {
        o.push_str(&format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{l:.2}\" height=\"{l:.2}\" rx=\"{:.2}\" \
             fill=\"#{:02x}{:02x}{:02x}\" fill-opacity=\"{:.3}\"/>\n",
            q[0] - l / 2.0,
            -q[1] - l / 2.0,
            l * 0.18,
            srgb(c[0]),
            srgb(c[1]),
            srgb(c[2]),
            c[3].clamp(0.0, 1.0),
        ));
    }
    o.push_str("</svg>\n");
    o
}

fn cores_distintas(s: &Stream) -> usize {
    let mut v: Vec<[i32; 3]> = vec4(s, "tint")
        .iter()
        .map(|p| [0, 1, 2].map(|k| (p[k] * 1000.0).round() as i32))
        .collect();
    v.sort_unstable();
    v.dedup();
    v.len()
}

/// O maior espalhamento vertical, em relação ao repouso, DENTRO de uma coluna do pano.
fn espalha_na_coluna(s: &Stream, r: &Stream) -> f32 {
    let (p, q) = (vec2(s, "P"), vec2(r, "P"));
    let lado = 6;
    (0..lado)
        .map(|c| {
            let v: Vec<f32> = (0..lado)
                .map(|l| p[l * lado + c][1] - q[l * lado + c][1])
                .collect();
            v.iter().copied().fold(f32::MIN, f32::max) - v.iter().copied().fold(f32::MAX, f32::min)
        })
        .fold(0.0, f32::max)
}

/// ⭐⭐ **Escreve as seis figuras do tutorial 7, com as asserções que as impedem de contar a mesma
/// história.**
///
/// ```text
/// cargo test -p ph2d-app-motion --lib write_the_appearance_figures -- --ignored --nocapture
/// ```
#[test]
#[ignore = "gerador de figuras — corra à mão"]
fn write_the_appearance_figures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    std::fs::create_dir_all(&dir).expect("a pasta");

    // 100 tiques: a cauda do meio cheia (44) e a linha de atraso de baixo cheia (30).
    let pan = colher(100);
    assert_eq!(pan.len(), 6, "a cena tem seis sinks");
    let (uma, arco) = (cores_distintas(&pan[0].0), cores_distintas(&pan[1].0));
    let (sem, com) = (pan[2].0.count(), pan[3].0.count());
    let (ordem, lugar) = (
        espalha_na_coluna(&pan[4].0, &pan[4].1),
        espalha_na_coluna(&pan[5].0, &pan[5].1),
    );
    eprintln!(
        "
  cores distintas     │ Tint {uma}  ·  Color Ramp {arco}
  linhas              │ sem rasto {sem}  ·  com rasto {com}
  espalha na coluna   │ ordem {ordem:.4}  ·  lugar {lugar:.6}"
    );
    assert!(
        uma == 1 && arco >= 12,
        "as figuras da COR contam a mesma historia"
    );
    assert!(com > sem * 4, "as figuras do RASTO contam a mesma historia");
    assert!(
        ordem > 0.05 && lugar < 1e-4,
        "as figuras do TEMPO contam a mesma historia"
    );

    // A escrita, só agora — ⛔ ver o cabeçalho do módulo.
    // Cada figura centra-se no SEU pano, com o tamanho de quadro do MAIOR do par — as duas de um par
    // leem-se à mesma escala.
    let quadro_de = |k: usize| {
        let (cx, cy, w, h) = draw::moldura(&[&escala(&vec2(&pan[k].0, "P"))]);
        // ⚠️ A moldura conta CENTROS; um quadrado passa meio lado para lá do dele.
        let lado = match pan[k].0.get("size") {
            Some(Column::Vec2(v)) => v.iter().map(|z| z[0]).fold(0.0, f32::max) * ESCALA,
            _ => 0.0,
        };
        (cx, cy, w + lado, h + lado)
    };
    let pinta = |i: usize, par: [usize; 2]| {
        let (a, b) = (quadro_de(par[0]), quadro_de(par[1]));
        let (cx, cy, _, _) = quadro_de(i);
        svg_pintado(&pan[i].0, (cx, cy, a.2.max(b.2), a.3.max(b.3)))
    };
    let tempo = |i: usize| {
        let (fortes, fantasma) = (escala(&vec2(&pan[i].0, "P")), escala(&vec2(&pan[i].1, "P")));
        let quadro = draw::moldura(&[&fortes, &fantasma]);
        draw::svg(&fortes, &fantasma, quadro, true)
    };
    let figuras = [
        ("cor_uma", pinta(0, [0, 1])),
        ("cor_por_peca", pinta(1, [0, 1])),
        ("rasto_sem", pinta(2, [2, 3])),
        ("rasto_com", pinta(3, [2, 3])),
        ("tempo_ordem", tempo(4)),
        ("tempo_lugar", tempo(5)),
    ];
    for i in 0..figuras.len() {
        for j in (i + 1)..figuras.len() {
            assert_ne!(
                figuras[i].1, figuras[j].1,
                "`{}` e `{}` sairam iguais",
                figuras[i].0, figuras[j].0
            );
        }
    }
    for (nome, svg) in &figuras {
        std::fs::write(dir.join(format!("{nome}.svg")), svg).expect("escreve");
    }
    eprintln!("  ✓ {} figuras em {}", figuras.len(), dir.display());
}
