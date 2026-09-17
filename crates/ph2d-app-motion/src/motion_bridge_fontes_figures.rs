//! **AS FIGURAS do tutorial do ciclo 8** (de onde vêm as coisas — doc 113, passo 6).
//!
//! ⚠️ **Elas não são desenhos:** cada quadrado é uma peça que o motor devolveu ao correr a cena
//! `=119` do PRODUTO, com a posição e o tamanho que ele lhe deu. *Uma figura desenhada à mão é a
//! ilustração de uma teoria, não a saída dela.*
//!
//! ⚠️⚠️ **E aqui há uma armadilha que as figuras dos ciclos anteriores não tinham: as MEMBRANAS.**
//! Cinco fontes deste grupo LEEM um external que a shell publica. Um gerador que cozesse sem
//! publicar escreveria SVGs **vazios** — e um ficheiro vazio lê-se, num PDF, como *«o pano não tem
//! nada»*. ⇒ [`colher`] publica pelas portas do produto, e há asserção de população antes de
//! escrever.
//!
//! ⭐ **O par do ZOOM é medido em PIXELS de ECRÃ**, e não em unidades de mundo: o assunto dele é
//! exactamente a diferença entre as duas réguas, e uma figura em unidades de mundo mostraria o
//! contrário do que a cena ensina (ali as peças da câmara *mudam* de tamanho — é o ecrã que as vê
//! iguais).
//!
//! ⛔⛔ **A ESCRITA VEM DEPOIS DAS ASSERÇÕES** — a lei que o ciclo 4 pagou.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib write_the_source_figures -- --ignored --nocapture
//! ```

use super::tutorial_draw as draw;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};

/// A altura da janela das figuras do zoom — o numerador do `zoom` publicado.
const JANELA_H: u32 = 900;
/// Os dois enquadramentos do par do zoom: quantas unidades de mundo a janela mostra.
const PERTO: f32 = 5.0;
const LONGE: f32 = 20.0;

/// Coze a cena `=119` com as membranas publicadas e devolve o stream de cada pano.
fn colher(altura_mundo: f32) -> Vec<Stream> {
    let mut m = MotionState::new();
    let (sinks, _) = crate::motion_demo_legend::monta("119", &mut m.doc, &m.registry);
    crate::motion_externals::publish_all(&mut m, 0.0);
    crate::motion_bridge::publish_editor_inputs(
        &mut m,
        &ph2d_render::Camera2d {
            height_world: altura_mundo,
            ..ph2d_render::Camera2d::default()
        },
        (0.0, 0.0),
        ph2d_editor_core::screens::layout::CenterSplit::None,
        ph2d_host::WindowSize::new(1600, JANELA_H),
    );
    sinks
        .iter()
        .map(|s| {
            m.pump
                .cook
                .cook(&m.doc.graph, &m.registry, *s, 0.0)
                .expect("coze")[0]
                .as_stream()
                .clone()
        })
        .collect()
}

fn vec2(s: &Stream, col: &str) -> Vec<[f32; 2]> {
    match s.get(col) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **UM PAR, UM ENQUADRAMENTO.** Devolve um SVG por stream, os dois na MESMA moldura.
///
/// ⛔⛔ **Enquadrar cada figura por si mesma é uma régua que MENTE num par:** as duas saem a
/// escalas diferentes e o leitor compara desenhos que não são comparáveis — foi assim que a
/// primeira versão destas figuras pôs doze quadrados de `30 px` ao lado de doze quadrados de
/// `30 px` com alturas que só o viewBox distinguia.
///
/// `marca` diz o lado do quadrado em unidades de figura. ⚠️⚠️ **`None` (o lado vem da coluna
/// `size`) só é honesto para uma peça que o sink desenha como QUAD.** As peças de texto e de forma
/// são geometria VECTORIAL: ali o `size` é um factor de escala e não uma extensão, e desenhá-lo
/// como lado dá quadrados de cem pixels sobre posições separadas por dois — a figura que esta
/// função existe para não escrever. Para essas passa-se uma marca, e a legenda di-lo.
fn svg_par(sets: &[&Stream], escala: f32, marca: Option<f32>) -> Vec<String> {
    // ⚠️ **Cada pano é CENTRADO no seu próprio centro, e só a ESCALA é partilhada.** Os dois panos
    // de um par vivem em metades opostas da cena (`±COL_X`), então uma moldura sobre as posições
    // CRUAS daria duas figuras com o conteúdo encostado a um canto e metade do papel vazio.
    let pts: Vec<Vec<[f32; 2]>> = sets
        .iter()
        .map(|s| {
            let p = vec2(s, "P");
            #[expect(clippy::cast_precision_loss, reason = "dezenas de peças")]
            let n = p.len().max(1) as f32;
            let c = p
                .iter()
                .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
            let c = [c[0] / n, c[1] / n];
            p.iter()
                .map(|q| [(q[0] - c[0]) * escala, -(q[1] - c[1]) * escala])
                .collect()
        })
        .collect();
    let refs: Vec<&[[f32; 2]]> = pts.iter().map(std::vec::Vec::as_slice).collect();
    let (cx, cy, w, h) = draw::moldura(&refs);
    sets.iter()
        .zip(&pts)
        .map(|(s, p)| {
            let t = vec2(s, "size");
            let mut corpo = String::new();
            for (i, q) in p.iter().enumerate() {
                let lado = marca.unwrap_or_else(|| t.get(i).map_or(1.0, |z| z[0]) * escala);
                corpo.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{lado:.2}\" height=\"{lado:.2}\" rx=\"{:.2}\" fill=\"{}\"/>",
                    q[0] - lado * 0.5,
                    q[1] - lado * 0.5,
                    (lado * 0.12).min(3.0),
                    draw::cor::FORTE
                ));
            }
            // ⛔⛔ **O `moldura` devolve o CENTRO, não o canto** — usá-lo como `viewBox` desloca a
            // figura meia moldura, e o que se vê é a peça encostada à borda (ou fora dela). A
            // conversão é a mesma que o `draw::svg` faz, e por isso é ela que se copia.
            const LARGURA_PX: f32 = 340.0;
            format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.2} {:.2} {w:.2} {h:.2}\" \
                 width=\"{LARGURA_PX:.0}\" height=\"{:.0}\" role=\"img\">\
                 <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"{:.2}\" fill=\"{}\"/>{corpo}</svg>",
                cx - w / 2.0,
                cy - h / 2.0,
                LARGURA_PX * h / w,
                cx - w / 2.0,
                cy - h / 2.0,
                w.min(h) / 26.0,
                draw::cor::FUNDO
            )
        })
        .collect()
}

/// **O par do ZOOM, em PIXELS DE ECRÃ.** À esquerda as nove peças da câmara; à direita uma peça da
/// tabela, que tem o tamanho do MUNDO. `zoom` = pixels de ecrã por unidade de mundo.
fn svg_ecra(camara: &Stream, tabela: &Stream, zoom: f32) -> String {
    // ⛔⛔ **A moldura tem de caber a GRELHA no enquadramento mais apertado.** A 1.ª versão era
    // `360 × 210`: com zoom, as nove peças ficam `162 px` afastadas (o VÃO segue o zoom, só o
    // TAMANHO é que não) e a figura mostrava UMA peça debaixo de uma legenda que dizia «as nove».
    // *Uma figura que contradiz a própria legenda é pior que nenhuma.* `620 × 400` cabe a grelha
    // a `PERTO` (`2 × 162 + 40 = 364`) e deixa a peça da tabela ao lado.
    const W: f32 = 620.0;
    const H: f32 = 400.0;
    let quadrado = |x: f32, y: f32, lado: f32, cor: &str| {
        format!(
            "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{lado:.2}\" height=\"{lado:.2}\" rx=\"{:.2}\" fill=\"{cor}\"/>",
            x - lado * 0.5,
            y - lado * 0.5,
            (lado * 0.12).min(3.0)
        )
    };
    let cp = vec2(camara, "P");
    let ct = vec2(camara, "size");
    let centro = cp
        .iter()
        .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
    #[expect(clippy::cast_precision_loss, reason = "nove peças")]
    let n = cp.len().max(1) as f32;
    let centro = [centro[0] / n, centro[1] / n];
    let mut corpo = String::new();
    for (i, p) in cp.iter().enumerate() {
        let lado = ct.get(i).map_or(1.0, |z| z[0]) * zoom;
        corpo.push_str(&quadrado(
            W * 0.31 + (p[0] - centro[0]) * zoom,
            H * 0.5 - (p[1] - centro[1]) * zoom,
            lado,
            draw::cor::FORTE,
        ));
    }
    let lado_tabela = vec2(tabela, "size").first().map_or(1.0, |z| z[0]) * zoom;
    corpo.push_str(&quadrado(
        W * 0.84,
        H * 0.5,
        lado_tabela,
        draw::cor::FANTASMA,
    ));
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W} {H}\">\
         <rect width=\"{W}\" height=\"{H}\" fill=\"{}\"/>{corpo}</svg>",
        draw::cor::FUNDO
    )
}

#[test]
#[ignore = "gerador de figuras — corra à mão"]
fn write_the_source_figures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    std::fs::create_dir_all(&dir).expect("a pasta");
    let perto = colher(PERTO);
    let longe = colher(LONGE);
    assert_eq!(perto.len(), 6, "a cena tem seis panos");

    // ⛔ As asserções ANTES de escrever: uma figura vazia lê-se, no PDF, como produto partido.
    let n = |k: usize| perto[k].count();
    assert_eq!(n(0), 3, "a palavra «OLA» sao tres pecas, deu {}", n(0));
    assert_eq!(n(1), 1, "a forma e' UMA peca, deu {}", n(1));
    assert_eq!(n(2), n(3), "os dois panos da tabela leem as mesmas linhas");
    assert!(n(2) >= 12, "as doze linhas do ficheiro, deu {}", n(2));
    // O par do zoom: as peças da câmara medem os MESMOS pixels nos dois enquadramentos.
    let px = |s: &Stream, altura: f32| {
        vec2(s, "size").first().map_or(0.0, |z| z[0])
            * (f32::from(u16::try_from(JANELA_H).expect("janela")) / altura)
    };
    let (a, b) = (px(&perto[5], PERTO), px(&longe[5], LONGE));
    assert!(
        (a - b).abs() < 1e-2 && a > 1.0,
        "a peca da camara tem de medir os mesmos pixels nos dois: {a} e {b}"
    );
    // ...e a da tabela NÃO: sem esse controlo a figura do par não mostra diferença nenhuma.
    let t = |s: &Stream, altura: f32| {
        vec2(s, "size").first().map_or(0.0, |z| z[0])
            * (f32::from(u16::try_from(JANELA_H).expect("janela")) / altura)
    };
    assert!(
        (t(&perto[2], PERTO) / t(&longe[2], LONGE) - 4.0).abs() < 0.1,
        "a peca da tabela tem de encolher 4x entre os dois enquadramentos"
    );

    // ⚠️ O par de cima leva MARCA (as peças dele são geometria vectorial — ver `svg_par`); o da
    // tabela leva o tamanho dos DADOS, porque ali a peça é o quad que o sink desenha.
    let cima = svg_par(&[&perto[0], &perto[1]], 100.0, Some(12.0));
    let meio = svg_par(&[&perto[2], &perto[3]], 100.0, None);
    for (nome, svg) in [
        ("fonte_texto.svg", cima[0].clone()),
        ("fonte_forma.svg", cima[1].clone()),
        ("tabela_linhas.svg", meio[0].clone()),
        ("tabela_grafico.svg", meio[1].clone()),
        (
            "zoom_perto.svg",
            svg_ecra(
                &perto[5],
                &perto[2],
                f32::from(u16::try_from(JANELA_H).expect("janela")) / PERTO,
            ),
        ),
        (
            "zoom_longe.svg",
            svg_ecra(
                &longe[5],
                &longe[2],
                f32::from(u16::try_from(JANELA_H).expect("janela")) / LONGE,
            ),
        ),
    ] {
        std::fs::write(dir.join(nome), svg).expect("escreve a figura");
        eprintln!("  escrita: {nome}");
    }
}
