//! **AS FIGURAS DO TUTORIAL DO CICLO 4** (os campos — doc 107, passo 6).
//!
//! ⚠️ **Elas saem da SAÍDA DO PRODUTO, nunca de um desenho à mão** ([§5.0](../../../CLAUDE.md)):
//! cada figura coze a cadeia `grid → <campo> → motion.scale → output` e desenha os quadrados
//! **do tamanho que o cook lhes deu**. Se o campo não morder, a figura é uma grelha uniforme —
//! e é exactamente isso que o gate abaixo recusa.
//!
//! ⭐ **A régua é a DISPERSÃO dos tamanhos, e não um deslocamento**: um campo não move um
//! elemento, ele **pesa-o**. A figura do ciclo 3 media `Δposição` porque um deformador desloca;
//! copiar aquela régua para aqui leria `0,00` sobre um campo perfeito — a mesma cegueira que a
//! régua do `rot`/`size` cobrou naquele ciclo.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture write_the_field_figures
//! ```

use super::tutorial_draw as draw;
use crate::motion::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::graph::{Edge, NodeId};

/// O lado da grelha das figuras. ⚠️ Pequeno de propósito: a figura tem `169 px` no PDF, e uma
/// grelha de 80 seria um borrão cinzento onde nenhuma mancha se lê.
const LADO: f32 = 26.0;
/// O passo entre elementos, em unidades de mundo.
const PASSO: f32 = 0.1;
/// Quanto a mancha cresce — o mesmo `2,2` da cena `=112`, para a figura e o ecrã concordarem.
const CRESCE: f32 = 2.2;

/// Uma figura: o ficheiro, o nó do campo, e os params que a fazem falar.
struct Fig {
    file: &'static str,
    node: &'static str,
    params: &'static [(&'static str, f32)],
}

/// ⚠️ **Os params são os do TUTORIAL, não os defaults** — uma figura no ponto neutro de um knob
/// desenha o nó desligado, que é o defeito que a tabela de preços do ciclo 3 pagou.
static FIGS: &[Fig] = &[
    Fig {
        file: "campo_circle",
        node: "motion.falloff",
        params: &[("center_x", 0.6), ("radius", 0.9)],
    },
    Fig {
        file: "campo_rect",
        node: "motion.falloff",
        params: &[
            ("center_x", 0.6),
            ("radius", 0.9),
            ("shape", 1.0),
            ("rotation", 30.0),
        ],
    },
    Fig {
        file: "campo_box",
        node: "field.box",
        params: &[
            ("center_x", 0.6),
            ("width", 1.6),
            ("height", 0.9),
            ("rotation", 20.0),
            ("soft", 0.6),
        ],
    },
    Fig {
        file: "campo_sweep",
        node: "field.radial_sweep",
        params: &[
            ("radius", 1.4),
            ("start_angle", -50.0),
            ("end_angle", 50.0),
            ("inner_radius", 0.5),
            ("soft", 0.25),
        ],
    },
    Fig {
        file: "campo_rank",
        node: "field.index_range",
        params: &[("start", 0.15), ("end", 0.5), ("soft", 0.2)],
    },
];

/// Coze `grid → <campo> → scale` e devolve `(posição, tamanho)` por elemento.
fn colher(fig: &Fig) -> Vec<([f32; 2], f32)> {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", LADO);
    g.set_param(grid, "cols", LADO);
    g.set_param(grid, "gap_x", PASSO);
    g.set_param(grid, "gap_y", PASSO);
    let campo = g.add_node(fig.node.to_string());
    for (k, v) in fig.params {
        g.set_param(campo, *k, *v);
    }
    let cresce = g.add_node("motion.scale");
    g.set_param(cresce, "amount", CRESCE);
    let out = g.add_node("motion.output");
    let liga = |g: &mut ph2d_nodegraph::graph::Graph, a: NodeId, b: NodeId| {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("liga");
    };
    liga(g, grid, campo);
    liga(g, campo, cresce);
    liga(g, cresce, out);
    let saida = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, out, 0.0)
        .expect("coze");
    let st = saida[0].as_stream();
    let p = match st.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    let s = match st.get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    p.into_iter().zip(s).collect()
}

/// O SVG: **um quadrado por elemento, do tamanho que o cook lhe deu**.
///
/// ⚠️ **O desenho é próprio e a PALETA é partilhada**, que é a lei escrita no
/// [`draw`](crate::render_loop::motion_bridge_tutorial_draw): o ciclo 2 desenha nuvens de pontos, o 3 marcas
/// orientadas, e este quadrados — *duas paletas seriam dois tutoriais com duas caras*.
fn svg(cel: &[([f32; 2], f32)]) -> String {
    let pontos: Vec<[f32; 2]> = cel.iter().map(|(p, _)| *p).collect();
    let (cx, cy, w, h) = draw::moldura(&[&pontos]);
    let maior = cel.iter().fold(0.0f32, |a, (_, t)| a.max(*t));
    const LARGURA_PX: f32 = 340.0;
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.3} {:.3} {w:.3} {h:.3}\" \
         width=\"{LARGURA_PX:.0}\" height=\"{:.0}\" role=\"img\">\n\
         <rect x=\"{:.3}\" y=\"{:.3}\" width=\"{w:.3}\" height=\"{h:.3}\" rx=\"{:.3}\" fill=\"{}\"/>\n",
        cx - w / 2.0,
        -cy - h / 2.0,
        LARGURA_PX * h / w,
        cx - w / 2.0,
        -cy - h / 2.0,
        w.min(h) / 26.0,
        draw::cor::FUNDO,
    );
    // ⚠️ O y do mundo cresce para CIMA e o do SVG para baixo — a figura mostra o que o artista vê.
    for (p, t) in cel {
        let peso = t / maior.max(1e-6);
        // O lado desenhado segue o `size` que o campo pesou, com um piso para o quadrado mais
        // leve continuar a ser um quadrado e não um ponto invisível.
        let lado = (PASSO * 0.88 * peso).max(PASSO * 0.14);
        let cor = if peso > 0.55 {
            draw::cor::FORTE
        } else {
            draw::cor::TRACO
        };
        s.push_str(&format!(
            "<rect x=\"{:.3}\" y=\"{:.3}\" width=\"{lado:.3}\" height=\"{lado:.3}\" \
             fill=\"{cor}\" opacity=\"{:.2}\"/>\n",
            p[0] - lado * 0.5,
            -p[1] - lado * 0.5,
            0.35 + 0.65 * peso
        ));
    }
    s.push_str("</svg>\n");
    s
}

/// ⭐⭐⭐ **ESCREVE AS FIGURAS, e RECUSA uma que não mostre o campo a morder.**
///
/// A régua é a **DISPERSÃO** dos tamanhos (`maior / menor`), não um deslocamento: *um campo não
/// move um elemento, pesa-o*. A barra é `1,5×`, o mesmo número que o gate da cena `=112` usa —
/// abaixo disso a mancha não se distingue do pano a olho na figura de `169 px` do PDF.
#[test]
#[ignore = "escreve ficheiros — corra à mão"]
fn write_the_field_figures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    std::fs::create_dir_all(&dir).expect("a pasta");
    eprintln!("\n  figura              │ células │ maior/menor │ barra │ ");
    let mut maus: Vec<String> = Vec::new();
    let colhidas: Vec<Vec<([f32; 2], f32)>> = FIGS.iter().map(colher).collect();
    for (fig, cel) in FIGS.iter().zip(&colhidas) {
        let (mut menor, mut maior) = (f32::MAX, f32::MIN);
        for (_, t) in cel {
            menor = menor.min(*t);
            maior = maior.max(*t);
        }
        let razao = maior / menor.max(1e-6);
        eprintln!(
            "  {:<19} │ {:>7} │ {razao:>11.2} │  1.50 │",
            fig.file,
            cel.len()
        );
        if razao <= 1.5 {
            maus.push(format!("`{}` (dispersao {razao:.2})", fig.file));
        }
    }
    assert!(
        maus.is_empty(),
        "{} figura(s) nao mostram o campo a MORDER -- uma grelha uniforme ensina que o no' nao \
         faz nada:\n  {}",
        maus.len(),
        maus.join("\n  ")
    );

    // ⛔⛔ **E DUAS FIGURAS IGUAIS SÃO UM KNOB QUE NINGUÉM LEU.**
    //
    // ⚠️ A dispersão acima é **cega à FORMA**: o `campo_circle` e o `campo_rect` leem os dois
    // `2,18` porque em ambos há um elemento no cheio e outro no vazio — a razão mede *«o campo
    // morde»*, nunca *«o campo tem a forma que o param pediu»*. Se o `shape` deixasse de ser
    // lido, as duas figuras ficariam idênticas e o gate de cima continuaria verde, com o
    // tutorial a mostrar duas vezes a mesma imagem debaixo de duas legendas diferentes.
    let mut vistas: Vec<(&str, Vec<u32>)> = Vec::new();
    for (fig, cel) in FIGS.iter().zip(&colhidas) {
        let assinatura: Vec<u32> = cel.iter().map(|(_, t)| t.to_bits()).collect();
        if let Some((outra, _)) = vistas.iter().find(|(_, a)| *a == assinatura) {
            panic!(
                "`{}` e `{outra}` desenham EXACTAMENTE a mesma coisa -- um dos params que as \
                 separa nao esta' a ser lido",
                fig.file
            );
        }
        vistas.push((fig.file, assinatura));
    }

    // ⛔⛔ **A ESCRITA VEM DEPOIS DAS DUAS ASSERÇÕES, e não é arrumação.** A 1.ª redacção
    // escrevia dentro do laço de medição: uma corrida VERMELHA deixava as figuras erradas no
    // disco, e o tutorial passava a mostrar duas vezes a mesma imagem debaixo de duas legendas
    // diferentes — sem que nada no repositório o dissesse. *Uma sonda que falha não pode deixar
    // o artefacto que ela reprovou.* (Medido: a corrida de mutação escreveu `campo_circle` e
    // `campo_rect` com o MESMO md5, e só o `md5sum` à mão os apanhou.)
    for (fig, cel) in FIGS.iter().zip(&colhidas) {
        std::fs::write(dir.join(format!("{}.svg", fig.file)), svg(cel)).expect("escrever");
    }

    let path = dir.join("params_campos.html");
    std::fs::write(&path, super::tutorial_table::derive(TABELA)).expect("a tabela");
    eprintln!("  tabela derivada │ {}", path.display());
}

/// As âncoras da tabela «o que cada controlo faz» — a mesma porta do ciclo 3.
static TABELA: &[(&str, &str)] = &[
    ("falloff", "motion.falloff"),
    ("box", "field.box"),
    ("radial-sweep", "field.radial_sweep"),
    ("index-range", "field.index_range"),
    ("remap", "field.remap"),
    ("combine", "field.combine"),
    ("shape", "field.shape"),
];
