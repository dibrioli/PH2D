//! **AS FIGURAS do tutorial do ciclo 9** (coisas que se seguram — doc 114, passo 6).
//!
//! ⚠️ **Elas não são desenhos:** cada quadrado é uma peça que o motor devolveu ao correr a cena
//! `=120` do PRODUTO, com a posição e o tamanho que ele lhe deu. *Uma figura desenhada à mão é a
//! ilustração de uma teoria, não a saída dela.*
//!
//! ⚠️⚠️ **QUATRO dos seis panos precisam de TEMPO, e isso é próprio deste ciclo.** A corda e o
//! campo nascem em repouso e o alvo do IK começa no meio do varrimento — colher no tique zero
//! desenharia uma corda esticada, uma grelha lisa e um braço a apontar a direito, que é
//! exactamente a imagem que o tutorial existe para desmentir. ⇒ [`colher`] corre
//! [`TIQUES_DE_REGIME`] antes de ler.
//!
//! ⭐⭐ **O par do MEIO leva a LINHA que liga as juntas, e os outros não.** Cinco pontos soltos não
//! se leem como uma corrente — o olho não sabe qual é o pai de qual —, e o assunto daquela fileira
//! é precisamente *quem manda em quem*. ⛔ A linha não é decoração: ela é a ordem dos índices, que
//! é o que `parent` de facto diz.
//!
//! ⛔⛔ **O campo desenha-se com o TAMANHO que os dados dão, nunca com uma marca fixa.** A altura
//! da onda vive no canal `Size` (`height_channel = 0`), logo uma marca fixa entregaria uma grelha
//! LISA — a figura a contradizer a legenda que diz «anéis». *É a mesma armadilha que a régua dos
//! gates pagou: uma medida que lê só o `P` é cega a metade deste grupo.*
//!
//! ⛔⛔ **A ESCRITA VEM DEPOIS DAS ASSERÇÕES** — a lei que o ciclo 4 pagou.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib write_the_rig_figures -- --ignored --nocapture
//! ```

use super::tutorial_draw as draw;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};

/// Quantos tiques correr antes de colher. ⚠️ **Não é «um número grande»:** a `40` a corda já
/// desceu e balança, o campo tem dois anéis a caminho da borda, e o alvo do IK está fora do
/// centro — as três coisas que as legendas afirmam.
const TIQUES_DE_REGIME: usize = 40;
const DT: f64 = 1.0 / 60.0;

/// Os seis panos, pela ordem em que a cena os empilha.
const CORDA: usize = 0;
const CAMPO: usize = 1;
const FK: usize = 2;
const IK: usize = 3;
const PELE_IGUAL: usize = 4;
const PELE_QUINHAO: usize = 5;

/// A forma da MANGA — contada da cena, porque a `Ligacao::Malha` precisa das colunas e uma
/// segunda cópia desenharia diagonais no dia em que a cena mudasse de forma.
const PELE_COLS: usize = 3;
const PELE_ROWS: usize = 7;

/// Quantos OSSOS uma corrente de `OSSOS_JUNTAS` juntas publica — **derivado, não escrito**.
///
/// ⭐ Uma corrente de `n` juntas tem `n − 1` ossos, porque **a raiz é a única junta sem osso a
/// chegar a ela** (a lei do [`ph2d_node_rig_bones`]). Desde 2026-09-20 os dois panos do meio
/// passam por um `rig.bones` antes de vestirem a `Shape: Bone`, logo as figuras deles contam
/// PEÇAS e não juntas.
///
/// ⚠️ **A asserção que este número substitui dizia `5` e reprovou no dia da migração** — e foi
/// ela que a apanhou, o que é exactamente o trabalho dela: *um gerador de figuras que não conta
/// as peças escreve um SVG em branco e o PDF lê-se como produto partido*.
#[expect(
    clippy::cast_possible_truncation,
    reason = "cinco juntas, contadas da cena"
)]
const OSSOS_DA_CORRENTE: usize = crate::motion_state::rig_demo::OSSOS_JUNTAS as usize - 1;

/// Coze a cena `=120` em regime e devolve o stream de cada pano.
///
/// ⚠️ **Sem membranas, ao contrário do ciclo 8:** as seis cadeias fabricam a CORRENTE a partir de
/// params, logo não há dados de fora a publicar — e a ausência é uma propriedade do grupo, não um
/// esquecimento (*um esqueleto não vem de fora*).
///
/// ⛔⛔ **Mas há uma coisa a publicar, e esta função esteve sem ela:** desde 2026-09-20 cada pano
/// veste uma forma (`source.shape` → `motion.duplicator`), e a geometria de uma forma é gerada no
/// QUADRO, nunca no cozimento. Sem o `publish` os seis panos cozem **VAZIOS** e as seis figuras
/// saem em branco — *e nenhum gate desta linha olha para uma figura*, que é a frase que o commit
/// das figuras já escrevia por outro motivo.
fn colher() -> Vec<Stream> {
    let mut m = MotionState::new();
    let (sinks, _) = crate::motion_demo_legend::monta("120", &mut m.doc, &m.registry);
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let mut t = 0.0f64;
    for _ in 0..TIQUES_DE_REGIME {
        for s in &sinks {
            let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, *s, t);
        }
        let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
        t += DT;
    }
    sinks
        .iter()
        .map(|s| {
            m.pump
                .cook
                .cook(&m.doc.graph, &m.registry, *s, t)
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
/// escalas diferentes e o leitor compara desenhos que não são comparáveis. A lei é a mesma do
/// ciclo 8, e por isso a forma é a mesma.
///
/// `marca` diz o lado do quadrado em unidades de figura; `None` faz o lado vir da coluna `size`,
/// que é o que o campo precisa. [`Ligacao`] diz que fios se desenham entre as peças.
fn svg_par(sets: &[&Stream], escala: f32, marca: Option<f32>, ligar: Ligacao) -> Vec<String> {
    // ⚠️ **Cada pano é CENTRADO no seu próprio centro, e só a ESCALA é partilhada** — os dois
    // panos de um par vivem em metades opostas da cena (`±COL_X`).
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
            let grossura = (w.min(h) / 42.0).max(1.0);
            let mut fio = |a: [f32; 2], b: [f32; 2]| {
                corpo.push_str(&format!(
                    "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"{}\" \
                     stroke-width=\"{grossura:.2}\" stroke-linecap=\"round\"/>",
                    a[0],
                    a[1],
                    b[0],
                    b[1],
                    draw::cor::TRACO
                ));
            };
            match ligar {
                Ligacao::Nenhuma => {}
                Ligacao::Corrente => {
                    for w in p.windows(2) {
                        fio(w[0], w[1]);
                    }
                }
                // ⭐ A MALHA: cada peça liga-se à vizinha da direita e à de baixo. ⚠️ O `cols` é a
                // largura da grelha que ENTROU no `rig.skin_deformer`, não um número escolhido —
                // com o valor errado a figura desenha diagonais e mostra uma malha que não existe.
                Ligacao::Malha(cols) => {
                    for (i, q) in p.iter().enumerate() {
                        // ⚠️ O `(i+1) % cols != 0` é o que impede o fio de saltar da última peça
                        // de uma fila para a primeira da seguinte — sem ele a malha ganha uma
                        // diagonal que atravessa a figura e mostra uma vizinhança que não existe.
                        if (i + 1) % cols != 0
                            && let Some(d) = p.get(i + 1)
                        {
                            fio(*q, *d);
                        }
                        if let Some(d) = p.get(i + cols) {
                            fio(*q, *d);
                        }
                    }
                }
            }
            for (i, q) in p.iter().enumerate() {
                let lado = marca.unwrap_or_else(|| t.get(i).map_or(1.0, |z| z[0]) * escala);
                corpo.push_str(&format!(
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{lado:.2}\" height=\"{lado:.2}\" \
                     rx=\"{:.2}\" fill=\"{}\"/>",
                    q[0] - lado * 0.5,
                    q[1] - lado * 0.5,
                    (lado * 0.12).min(3.0),
                    draw::cor::FORTE
                ));
            }
            // ⛔⛔ **O `moldura` devolve o CENTRO, não o canto** — usá-lo como `viewBox` desloca a
            // figura meia moldura. A conversão é a mesma que o `draw::svg` faz.
            const LARGURA_PX: f32 = 340.0;
            format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.2} {:.2} {w:.2} {h:.2}\" \
                 width=\"{LARGURA_PX:.0}\" height=\"{:.0}\" role=\"img\">\
                 <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"{:.2}\" \
                 fill=\"{}\"/>{corpo}</svg>",
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

/// **Que fios se desenham entre as peças de uma figura.**
///
/// ⛔⛔ **Não é decoração, e a figura da pele provou-o:** vinte e cinco pontos soltos leem-se como
/// RUÍDO, e os mesmos pontos com os fios da grelha leem-se como uma malha a deformar-se. *A
/// ligação é a informação que a nuvem de pontos perdeu* — numa corrente ela é o `parent`, numa
/// pele é a vizinhança da grelha —, e nenhum gate desta linha a via porque nenhum gate olha para
/// uma imagem.
#[derive(Clone, Copy)]
enum Ligacao {
    /// Nada: o pano é uma nuvem e a vizinhança não é o assunto (a corda desenha-se assim porque
    /// as peças dela já se tocam).
    Nenhuma,
    /// Peça `i` à peça `i+1` — a ordem dos índices, que é o que a corrente de juntas é.
    Corrente,
    /// A vizinhança de uma grelha de `cols` colunas: à direita e abaixo.
    Malha(usize),
}

/// A dispersão dos lados das peças de um pano — a testemunha de que o campo ONDULA.
fn dispersao_do_tamanho(s: &Stream) -> f32 {
    let t = vec2(s, "size");
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for z in &t {
        lo = lo.min(z[0]);
        hi = hi.max(z[0]);
    }
    if t.is_empty() { 0.0 } else { hi - lo }
}

#[test]
#[ignore = "gerador de figuras — corra à mão"]
fn write_the_rig_figures() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    std::fs::create_dir_all(&dir).expect("a pasta");
    let p = colher();
    assert_eq!(p.len(), 6, "a cena tem seis panos");

    // ⛔ As asserções ANTES de escrever: uma figura vazia lê-se, no PDF, como produto partido.
    assert_eq!(p[CORDA].count(), 20, "a corda tem vinte pontos");
    assert_eq!(p[CAMPO].count(), 121, "o campo e' 11x11");
    assert_eq!(
        p[FK].count(),
        OSSOS_DA_CORRENTE,
        "o pano do FK entrega OSSOS (uma corrente de n juntas tem n-1)"
    );
    assert_eq!(
        p[IK].count(),
        OSSOS_DA_CORRENTE,
        "o pano do IK entrega os mesmos ossos"
    );
    assert_eq!(
        p[PELE_IGUAL].count(),
        PELE_COLS * PELE_ROWS,
        "a pele e' a manga"
    );
    assert_eq!(
        p[PELE_QUINHAO].count(),
        PELE_COLS * PELE_ROWS,
        "a pele do quinhao e' a mesma manga"
    );

    // ⭐ **A legenda do campo diz «anéis», e a figura tem de os poder mostrar.** Sem dispersão de
    // tamanho a grelha sai LISA e a legenda passa a mentir.
    let d = dispersao_do_tamanho(&p[CAMPO]);
    assert!(
        d > 1e-3,
        "o campo saiu LISO (dispersao de lado {d:e}) — a figura contradiria a legenda"
    );

    // ⭐ **E os dois pares de comparação têm de DIFERIR**, senão as figuras do tutorial mostram
    // duas vezes a mesma coisa debaixo de duas legendas diferentes.
    // ⚠️⚠️ **Os panos vivem em metades opostas da cena** (`±COL_X`), logo comparar as posições
    // CRUAS mede a separação entre eles e não a diferença de FORMA — e daria `5,8` de mundo para
    // dois panos idênticos. O que se mede é cada um CENTRADO, que é exactamente o que a figura
    // mostra (o `svg_par` centra-os pela mesma razão).
    let centrado = |k: usize| {
        let q = vec2(&p[k], "P");
        #[expect(clippy::cast_precision_loss, reason = "dezenas de peças")]
        let n = q.len().max(1) as f32;
        let c = q
            .iter()
            .fold([0.0f32; 2], |a, r| [a[0] + r[0], a[1] + r[1]]);
        let c = [c[0] / n, c[1] / n];
        q.iter()
            .map(|r| [r[0] - c[0], r[1] - c[1]])
            .collect::<Vec<_>>()
    };
    let forma_dif = |a: usize, b: usize| {
        let (x, y) = (centrado(a), centrado(b));
        x.iter()
            .zip(&y)
            .map(|(u, v)| ((u[0] - v[0]).powi(2) + (u[1] - v[1]).powi(2)).sqrt())
            .fold(0.0f32, f32::max)
    };
    assert!(
        forma_dif(FK, IK) > 0.05,
        "as duas correntes tem a mesma FORMA — as duas figuras do meio seriam iguais"
    );
    assert!(
        forma_dif(PELE_IGUAL, PELE_QUINHAO) > 1e-3,
        "as duas peles tem a mesma FORMA — as duas figuras de baixo seriam iguais"
    );

    // ⚠️ **Três escalas, e cada uma é o tamanho do seu assunto:** a corda mede ~1,9 de mundo, a
    // corrente ~1,8 e a pele ~2,0 — o `100` do ciclo 8 servia a panos de dimensão parecida, e aqui
    // serve pela mesma razão. A MARCA muda: o campo lê o `size` (a onda vive lá) e os outros não.
    let cima = svg_par(&[&p[CORDA], &p[CAMPO]], 100.0, None, Ligacao::Nenhuma);
    let meio = svg_par(&[&p[FK], &p[IK]], 100.0, Some(11.0), Ligacao::Corrente);
    let baixo = svg_par(
        &[&p[PELE_IGUAL], &p[PELE_QUINHAO]],
        100.0,
        Some(9.0),
        Ligacao::Malha(PELE_COLS),
    );
    for (nome, svg) in [
        ("rig_corda.svg", cima[0].clone()),
        ("rig_campo.svg", cima[1].clone()),
        ("rig_fk.svg", meio[0].clone()),
        ("rig_ik.svg", meio[1].clone()),
        ("rig_pele_igual.svg", baixo[0].clone()),
        ("rig_pele_quinhao.svg", baixo[1].clone()),
    ] {
        std::fs::write(dir.join(nome), &svg).expect("escreve a figura");
        eprintln!("  ✓ {nome}  ({} bytes)", svg.len());
    }
}
