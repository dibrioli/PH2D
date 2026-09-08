//! **AS FIGURAS DO TUTORIAL DO CICLO 3** — geradas COZINHANDO os nós, nunca desenhadas à mão
//! ([doc 103 §3](../../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)).
//!
//! ⚠️ **Um deformador não tem forma própria: ele tem uma FORMA DE MEXER**, e é isso que separa
//! estas figuras das dos dois ciclos anteriores. Uma nuvem de chegada não diz de onde cada
//! elemento veio — com deslocamento pequeno lê-se *«não fez nada»* e com deslocamento grande
//! lê-se *«outra grelha»*, **e as duas leituras estão erradas sobre o mesmo produto**. Por isso
//! o modo por omissão aqui é o **traço**: a folha de partida a cinzento, o resultado em cor, e
//! uma linha a ligar cada elemento ao seu. É a coerência entre traços vizinhos que se vê.
//!
//! ⛔⛔⛔ **E TRÊS dos treze não mexem num ponto — eles escrevem uma COLUNA que uma nuvem de
//! pontos não consegue mostrar.** O `motion.rotate` e o `motion.look_at` escrevem `rot`, o
//! `motion.scale` escreve `size`, e **nenhum deles toca em `P`**. A primeira redacção deste
//! gerador desenhava pontos, como os dois ciclos anteriores, e a régua leu **`0,00` de
//! deslocamento** nos três: *a figura não conseguia distinguir «o nó não fez nada» de «o nó fez
//! uma coisa a que eu sou cego»*, e num tutorial as duas leituras são o mesmo desastre — um
//! artista veria a grelha intacta debaixo do título «Rotate».
//!
//! ⇒ **o elemento é desenhado como uma MARCA, nunca como um ponto**: um traço curto centrado em
//! `P`, com a direcção de `rot` e o comprimento de `size`. As três colunas que este grupo
//! escreve entram todas na mesma figura, e a régua mede o que a marca faz — a **ponta** dela,
//! que é o único ponto derivado que se move quando qualquer uma das três muda.
//!
//! ⛔⛔ **E o traço NÃO SERVE a quem muda a contagem.** Ele emparelha o `i`-ésimo ponto de
//! partida com o `i`-ésimo de chegada; o `motion.mirror` devolve `2n` e o
//! `motion.kaleidoscope` devolve `n × segmentos`, então o emparelhamento ligaria elementos que
//! não têm nada a ver um com o outro e desenharia um novelo com cara de deformação. Esses dois
//! saem em **fantasma**: a folha de partida por baixo, o que saiu por cima, sem linhas.
//!
//! O enquadramento e o desenho saem da **mesma porta** do ciclo 2
//! ([`super::animadores_figures::moldura`] e [`svg`]) — duas cópias dariam dois tutoriais com
//! duas paletas.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture dump_deformadores_figures
//! ```

use super::animadores_figures::{liga, no_com};
use super::tutorial_draw::{cor, moldura};
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, SIZE_IDENTITY};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::NodeId;

/// ⚠️ **Relativo ao MANIFESTO, não ao processo** — `cargo test` corre com a cwd na raiz do
/// pacote, e um gerador que escreve no sítio errado diz «ok».
const DIR: &str = "../../docs/Motion Nodes/tutoriais/fig";

/// O instante em que tudo é cozido. Estes nós são todos `Pure` menos o `look_at` — nenhum deles
/// depende de `t` —, mas o número fica explícito para a figura não mudar se algum vier a ler o
/// relógio.
const T: f64 = 0.37;

/// A FOLHA que entra em toda figura: `13 × 13` a `12` de espaçamento, centrada na origem.
///
/// ⚠️ **Uma folha, não uma fila.** O ciclo 2 desenhava filas porque uma onda se lê ao longo de
/// uma; um deformador dobra uma ÁREA, e numa fila o `bend`, o `twist` e o `spherize` são
/// indistinguíveis uns dos outros.
const FOLHA: (f32, f32, f32) = (13.0, 13.0, 12.0);

/// O meio-comprimento da MARCA, em unidades de mundo. ⚠️ **Derivado do vão da folha**, e não um
/// número de desenho: uma marca com `0,35` do vão lê-se como uma marca e não toca a vizinha.
const MARCA: f32 = FOLHA.2 * 0.35;

/// ⭐ **UM ELEMENTO, com as três colunas que este grupo escreve** — onde ele está, para onde
/// aponta, e quão grande é.
#[derive(Clone, Copy)]
struct Marca {
    p: [f32; 2],
    /// Graus — a coluna `rot`, que é o que o `motion.rotate` e o `motion.look_at` escrevem.
    rot: f32,
    /// O factor da coluna `size` (a identidade é `[1, 1]`), que é o que o `motion.scale` escreve.
    size: [f32; 2],
}

impl Marca {
    /// **A PONTA da marca** — o ponto derivado que se move quando QUALQUER uma das três colunas
    /// muda, e por isso a única régua que serve aos treze nós de uma vez.
    ///
    /// ⚠️ O seno e o cosseno aqui são de DESENHO (uma figura de documentação), não do produto —
    /// a HR-5 é sobre o que o motor computa por elemento num quadro.
    fn ponta(self) -> [f32; 2] {
        let (s, c) = (self.rot.to_radians().sin(), self.rot.to_radians().cos());
        [
            self.p[0] + MARCA * self.size[0] * c,
            self.p[1] + MARCA * self.size[1] * s,
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Como {
    /// Há emparelhamento: o `i`-ésimo que entrou virou o `i`-ésimo que saiu.
    Emparelha,
    /// A contagem muda — não há par (ver o cabeçalho).
    Fantasma,
}

/// ⛔⛔ **A FOLHA DESENHA-SE COMO MALHA, e não como uma nuvem com traços de deslocamento.**
///
/// A primeira redacção ligava cada elemento à sua posição de partida. Na figura do
/// `motion.twist` isso saiu um **NOVELO** — a `150°` os vizinhos vão para lados diferentes e os
/// traços cruzam-se todos. Tirado o traço, saiu **pior**: uma dispersão de marcas horizontais
/// que se lê como ruído, porque *a torção vive na relação entre VIZINHOS e nenhuma marca sozinha
/// a contém*.
///
/// ⇒ o que se desenha é a **grelha deformada**: cada elemento ligado ao vizinho da direita e ao
/// de baixo. É a figura que toda referência usa para um warp, e ela mostra as três coisas de uma
/// vez — para onde a folha foi, como ela se dobrou, e (pelas marcas) para onde cada elemento
/// aponta.
///
/// ⚠️ **Só existe malha quando a contagem não muda** — o `motion.mirror` devolve `2n` e o
/// `motion.kaleidoscope` `n × segmentos`, e ali o `i + 1` já não é o vizinho da direita de nada.
/// Esses saem em marcas soltas, que é o desenho honesto para *«apareceram mais»*.
fn arestas(n: usize) -> Vec<(usize, usize)> {
    let cols = FOLHA.1 as usize;
    let mut v = Vec::new();
    for i in 0..n {
        if (i + 1) % cols != 0 && i + 1 < n {
            v.push((i, i + 1));
        }
        if i + cols < n {
            v.push((i, i + cols));
        }
    }
    v
}

struct Fig {
    file: &'static str,
    node: &'static str,
    params: &'static [(&'static str, f32)],
    como: Como,
}

/// ⚠️ **Os valores são escolhidos para o EFEITO ser legível numa figura de `340 px`**, e é por
/// isso que eles não são os defaults: metade deste grupo nasce na identidade, e uma figura de um
/// nó no ponto neutro é a folha de partida outra vez. A régua no fim do ficheiro mede
/// exactamente isso.
static FIGS: &[Fig] = &[
    Fig {
        file: "def_move",
        node: "motion.move",
        params: &[("dx", 46.0), ("dy", 18.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_rotate",
        node: "motion.rotate",
        // ⚠️ `60°` e não `30°`: a régua da marca aprova os dois, mas numa figura de `340 px` a
        // meia-volta lê-se de relance e o quarto não. *A figura tem de convencer, não passar.*
        params: &[("angle", 60.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_scale",
        node: "motion.scale",
        // O dobro — pela mesma razão do `rotate` acima.
        params: &[("amount", 2.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_transform",
        node: "motion.transform",
        // O CISALHAMENTO da W3 — o terço do afim que faltava, e o que a figura existe para
        // mostrar: a folha inclina-se, e `1` é 45°.
        params: &[("skew_x", 0.5), ("pivot_mode", 2.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_mirror",
        node: "motion.mirror",
        // Horizontal, com um vão — o par lê-se como par. ⚠️ `2n`: fantasma.
        params: &[("axis", 1.0), ("offset", 90.0)],
        como: Como::Fantasma,
    },
    Fig {
        file: "def_look_at",
        node: "motion.look_at",
        // Sem alvo ligado o modo `Point` aponta para a origem: cada elemento VIRA-SE, e a
        // posição não muda. ⚠️ É por isso que esta figura é a excepção da régua (ver o fim).
        params: &[("target_x", 0.0), ("target_y", 0.0), ("strength", 1.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_bend",
        node: "motion.bend",
        params: &[("angle", 110.0), ("amount", 1.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_twist",
        node: "motion.twist",
        // ⭐ O modo `Centroid` da W1 — o eixo no meio do que está a passar, sem coordenada
        // digitada.
        params: &[
            ("angle", 150.0),
            ("amount", 1.0),
            (ph2d_nodegraph::pivot::PARAM, 2.0),
        ],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_spherize",
        node: "motion.spherize",
        params: &[("amount", 1.0), ("radius", 90.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_four_point_warp",
        node: "motion.four_point_warp",
        // O keystone: o topo aperta-se.
        params: &[("tl_dx", 34.0), ("tr_dx", -34.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_bezier_warp",
        node: "motion.bezier_warp",
        // ⭐ A fronteira CURVA (W5a): as duas tangentes de cima sobem, e o topo arqueia — que é
        // exactamente o que o irmão de cima não consegue fazer.
        params: &[("top_a_dy", 40.0), ("top_b_dy", 40.0)],
        como: Como::Emparelha,
    },
    Fig {
        file: "def_kaleidoscope",
        node: "motion.kaleidoscope",
        // ⚠️ `n × segmentos`: fantasma.
        params: &[("segments", 6.0), ("reflect", 1.0)],
        como: Como::Fantasma,
    },
    Fig {
        file: "def_spline_wrap",
        node: "motion.spline_wrap",
        // ⚠️ **A curva é ESCRITA aqui desde 2026-09-08.** Este nó passou a nascer INERTE (ordem
        // do dono: *«melhor nascer inerte com um botão para selecionar um path»*), e a figura
        // dele saiu **`0,00` de deslocamento** na primeira corrida depois disso — a régua deste
        // ficheiro apanhou-a antes de ela chegar ao PDF. *Uma fixtura que herda um default mede
        // o default, e um default que muda leva a figura com ele.*
        params: &[
            ("height_scale", 1.0),
            ("p0x", -3.0),
            ("p0y", -1.5),
            ("p1x", -1.0),
            ("p1y", 2.0),
            ("p2x", 1.0),
            ("p2y", -2.0),
            ("p3x", 3.0),
            ("p3y", 1.5),
        ],
        como: Como::Emparelha,
    },
];

/// As âncoras da tabela de controlos — a mesma ordem do grupo no doc 106.
static TABELA: &[(&str, &str)] = &[
    ("move", "motion.move"),
    ("rotate", "motion.rotate"),
    ("scale", "motion.scale"),
    ("transform", "motion.transform"),
    ("mirror", "motion.mirror"),
    ("look-at", "motion.look_at"),
    ("bend", "motion.bend"),
    ("twist", "motion.twist"),
    ("spherize", "motion.spherize"),
    ("four-point-warp", "motion.four_point_warp"),
    ("bezier-warp", "motion.bezier_warp"),
    ("kaleidoscope", "motion.kaleidoscope"),
    ("spline-wrap", "motion.spline_wrap"),
];

/// Monta `folha [→ nó] → output`. `com_no = false` deixa o fantasma: a MESMA folha sem o nó.
fn monta(m: &mut MotionState, fig: &Fig, com_no: bool) -> NodeId {
    let (r, c, g) = FOLHA;
    let folha = no_com(
        m,
        "motion.grid",
        &[("rows", r), ("cols", c), ("gap_x", g), ("gap_y", g)],
    );
    let mut ultimo = folha;
    if com_no {
        let n = no_com(m, fig.node, fig.params);
        liga(m, ultimo, 0, n, 0, false);
        ultimo = n;
    }
    let out = m.doc.graph.add_node("motion.output".to_string());
    liga(m, ultimo, 0, out, 0, false);
    out
}

/// Coze e lê as TRÊS colunas — `P`, `rot` e `size`.
///
/// ⚠️ **Uma coluna ausente é a identidade dela**, e não um erro: uma folha de grelha chega sem
/// `rot` e sem `size`, e é o nó que os põe lá. Ler `0` e `[1, 1]` é a mesma lei que o `eval` de
/// cada um destes nós corre.
fn marcas(m: &MotionState, out: NodeId) -> Vec<Marca> {
    let mut cook = Cook::new();
    let Ok(v) = cook.cook(&m.doc.graph, &m.registry, out, T) else {
        return Vec::new();
    };
    let Some(s) = v.first().map(|c| c.as_stream()) else {
        return Vec::new();
    };
    let p = match s.get("P") {
        Some(Column::Vec2(p)) => p.clone(),
        _ => return Vec::new(),
    };
    let rot = match s.get("rot") {
        Some(Column::Scalar(r)) => r.clone(),
        _ => Vec::new(),
    };
    let size = match s.get("size") {
        Some(Column::Vec2(z)) => z.clone(),
        _ => Vec::new(),
    };
    p.iter()
        .enumerate()
        .map(|(i, q)| Marca {
            p: *q,
            rot: rot.get(i).copied().unwrap_or(0.0),
            size: size.get(i).copied().unwrap_or(SIZE_IDENTITY),
        })
        .collect()
}

/// `(o que saiu, a folha de partida)`.
fn colhe(fig: &Fig) -> (Vec<Marca>, Vec<Marca>) {
    let mut m = MotionState::new();
    let out = monta(&mut m, fig, true);
    let saiu = marcas(&m, out);
    let mut mg = MotionState::new();
    let og = monta(&mut mg, fig, false);
    let partida = marcas(&mg, og);
    (saiu, partida)
}

/// **UMA FOLHA DE MARCAS** — a de partida a cinzento, a de chegada em cor, e o traço entre
/// elas quando o emparelhamento existe.
///
/// ⚠️ **O enquadramento e a paleta saem da porta do ciclo 2** ([`moldura`] e [`cor`]); o que é
/// próprio deste ficheiro é o DESENHO, porque o assunto é outro — ver o cabeçalho.
/// ⚠️ **A MARCA só é desenhada onde ela FALA.** Três dos treze escrevem `rot` ou `size` e é para
/// eles que a marca existe; nos outros dez ela é sempre o mesmo traço horizontal, e numa malha
/// torcida dez marcas iguais são **ruído sobre a única coisa que a figura tem para dizer**. A
/// pergunta é feita à colheita — *alguma marca virou ou cresceu?* —, nunca autorada por figura.
fn marcas_falam(saiu: &[Marca], partida: &[Marca]) -> bool {
    saiu.len() == partida.len()
        && saiu
            .iter()
            .zip(partida)
            .any(|(a, b)| a.rot != b.rot || a.size != b.size)
}

fn svg_marcas(
    saiu: &[Marca],
    partida: &[Marca],
    (cx, cy, w, h): (f32, f32, f32, f32),
    malhado: bool,
) -> String {
    let falam = marcas_falam(saiu, partida) || !malhado;
    const LARGURA_PX: f32 = 340.0;
    // A espessura segue a média geométrica do quadro, como no ciclo 2.
    let e = (w * h).sqrt() / 150.0;
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.2} {:.2} {w:.2} {h:.2}\" \
         width=\"{LARGURA_PX:.0}\" height=\"{:.0}\" role=\"img\">\n\
         <rect x=\"{:.2}\" y=\"{:.2}\" width=\"{w:.2}\" height=\"{h:.2}\" rx=\"{:.2}\" fill=\"{}\"/>\n",
        cx - w / 2.0,
        -cy - h / 2.0,
        LARGURA_PX * h / w,
        cx - w / 2.0,
        -cy - h / 2.0,
        w.min(h) / 26.0,
        cor::FUNDO,
    ); // O y do mundo cresce para CIMA e o do SVG para baixo.
    let malha = |s: &mut String, ms: &[Marca], c: &str, esp: f32| {
        for (a, b) in arestas(ms.len()) {
            s.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" \
                 stroke=\"{c}\" stroke-width=\"{esp:.2}\" stroke-linecap=\"round\"/>\n",
                ms[a].p[0], -ms[a].p[1], ms[b].p[0], -ms[b].p[1]
            ));
        }
    };
    // Onde a marca não fala, o elemento é um PONTO — o vértice da malha, e nada mais.
    let marca = |s: &mut String, m: &Marca, c: &str, esp: f32| {
        if !falam {
            s.push_str(&format!(
                "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" fill=\"{c}\"/>\n",
                m.p[0],
                -m.p[1],
                esp * 0.8
            ));
            return;
        }
        let t = m.ponta();
        // O traço é centrado em `P`: a marca vai da ponta oposta à ponta.
        let o = [2.0 * m.p[0] - t[0], 2.0 * m.p[1] - t[1]];
        s.push_str(&format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" \
             stroke=\"{c}\" stroke-width=\"{esp:.2}\" stroke-linecap=\"round\"/>\n",
            o[0], -o[1], t[0], -t[1]
        ));
    };
    // A folha de PARTIDA, por baixo: a malha fina e as marcas.
    if malhado {
        malha(&mut s, partida, cor::FANTASMA, e * 0.55);
    }
    for m in partida {
        marca(&mut s, m, cor::FANTASMA, e);
    }
    // E a de CHEGADA, por cima.
    if malhado {
        malha(&mut s, saiu, cor::TRACO, e * 0.8);
    }
    for m in saiu {
        marca(&mut s, m, cor::FORTE, e * 1.15);
    }
    s.push_str("</svg>\n");
    s
}

/// ⚠️ **A moldura de uma figura deste ciclo é QUADRADA, e a do ciclo 2 não é.** Lá as figuras
/// são filas — largas e baixas — e um quadrado desperdiçaria 70 % de cada uma; aqui elas são
/// folhas, e o que se ganha é o ALINHAMENTO: impressas três por linha, treze molduras de
/// proporções diferentes deixam as legendas em degraus e a página lida como desarrumada. *A
/// moldura é a forma do que ela mostra — e treze folhas são treze quadrados.*
fn quadrada((cx, cy, w, h): (f32, f32, f32, f32)) -> (f32, f32, f32, f32) {
    let lado = w.max(h);
    (cx, cy, lado, lado)
}

/// Todos os pontos que a figura ocupa — as marcas contam pelas DUAS pontas, senão uma marca
/// grande na borda sai cortada.
fn extremos(ms: &[Marca]) -> Vec<[f32; 2]> {
    let mut v = Vec::with_capacity(ms.len() * 2);
    for m in ms {
        let t = m.ponta();
        v.push(t);
        v.push([2.0 * m.p[0] - t[0], 2.0 * m.p[1] - t[1]]);
    }
    v
}

#[test]
#[ignore = "gerador de figuras do tutorial"]
fn dump_deformadores_figures() {
    let dir = std::path::Path::new(DIR);
    std::fs::create_dir_all(dir).expect("a pasta das figuras");
    let colhidas: Vec<(Vec<Marca>, Vec<Marca>)> = FIGS.iter().map(colhe).collect();

    // ⚠️ **UMA moldura por FIGURA, e não uma partilhada pelo grupo.** No ciclo 2 as figuras de
    // onda partilhavam o quadro para se poderem comparar a olho; aqui elas não se comparam — o
    // caleidoscópio ocupa seis vezes a área da folha, e uma moldura comum deixaria as outras
    // doze do tamanho de um selo.
    eprintln!("\n  figura              │  marcas │ moldura │ deslocamento │ barra │ desenho");
    let mut maus: Vec<String> = Vec::new();
    for (i, fig) in FIGS.iter().enumerate() {
        let (saiu, partida) = &colhidas[i];
        assert!(
            !saiu.is_empty(),
            "a figura `{}` saiu VAZIA — uma figura vazia ensina o contrario do que acontece",
            fig.file
        );
        let (es, ep) = (extremos(saiu), extremos(partida));
        let m = quadrada(moldura(&[es.as_slice(), ep.as_slice()]));
        let lado = (m.2 * m.3).sqrt();
        // ⚠️ **A régua mede a PONTA da marca, não a posição** — e essa é a correcção que este
        // gerador teve de pagar antes de escrever a primeira figura. Medindo `P`, o
        // `motion.rotate`, o `motion.scale` e o `motion.look_at` liam **`0,00`**: eles escrevem
        // `rot` e `size` e nunca tocam num ponto. *Uma régua que só vê uma das três colunas
        // acusa de inerte um nó que está a trabalhar*, e a figura que ela aprovaria mostraria a
        // grelha intacta debaixo do título «Rotate». A ponta move-se quando qualquer uma das
        // três muda.
        //
        // ⚠️ E ela continua a medir contra a MESMA cadeia sem o nó — não a altura da figura,
        // que numa folha 2-D já existe com o nó desligado.
        //
        // ⚠️ **E ela olha o EMPARELHAMENTO quando ele existe.** Com a contagem a mudar não há
        // par, e a pergunta passa a ser *a nuvem cresceu?* — a caixa de chegada contra a de
        // partida —, que é exactamente o que o espelho e o caleidoscópio fazem.
        let mexeu = if saiu.len() == partida.len() {
            saiu.iter()
                .zip(partida)
                .map(|(a, b)| {
                    let (t, g) = (a.ponta(), b.ponta());
                    ((t[0] - g[0]).powi(2) + (t[1] - g[1]).powi(2)).sqrt()
                })
                .fold(0.0f32, f32::max)
        } else {
            let cx = moldura(&[es.as_slice()]);
            let cp = moldura(&[ep.as_slice()]);
            (cx.2 - cp.2).abs().max((cx.3 - cp.3).abs())
        };
        // ⚠️⚠️ **A BARRA é a MARCA, não a moldura — e a primeira redacção usava a moldura.**
        // Ela vinha do ciclo 2, onde a figura afirma *«a fila desloca-se»* e o denominador certo
        // é o quadro. Aqui a figura desenha as DUAS marcas sobrepostas, e o que o olho compara é
        // uma com a outra: o que ela afirma é *«a marca cinzenta e a colorida distinguem-se»*.
        // Medida contra a moldura, uma rotação de 30° sobre uma marca de `8,4` numa folha de
        // `169` lia `1,3 %` e era acusada — sobre uma figura em que cada marca vira visivelmente
        // no ecrã. *Quando a figura sobrepõe dois estados, o denominador é o TAMANHO deles.*
        let barra = MARCA * 0.35;
        eprintln!(
            "  {:<19} │ {:>7} │ {lado:>7.0} │ {mexeu:>12.2} │ {barra:>5.2} │ {}",
            fig.file,
            saiu.len(),
            match (fig.como == Como::Emparelha, marcas_falam(saiu, partida)) {
                (true, true) => "malha + marcas",
                (true, false) => "malha",
                (false, _) => "marcas soltas",
            }
        );
        if mexeu <= barra {
            maus.push(format!(
                "`{}` (deslocamento {mexeu:.2} numa moldura de {lado:.0})",
                fig.file
            ));
        }
    }
    assert!(
        maus.is_empty(),
        "{} figura(s) nao mostram o deformador a DEFORMAR — uma folha que nao se mexe ensina \
         que o no' nao faz nada:\n  {}",
        maus.len(),
        maus.join("\n  ")
    );

    for (i, fig) in FIGS.iter().enumerate() {
        let (saiu, partida) = &colhidas[i];
        let m = quadrada(moldura(&[
            extremos(saiu).as_slice(),
            extremos(partida).as_slice(),
        ]));
        let path = dir.join(format!("{}.svg", fig.file));
        std::fs::write(
            &path,
            svg_marcas(saiu, partida, m, fig.como == Como::Emparelha),
        )
        .expect("escrever");
        eprintln!("  escrito │ {}", path.display());
    }

    let path = dir.join("params_deformadores.html");
    std::fs::write(&path, super::tutorial_table::derive(TABELA)).expect("a tabela");
    eprintln!("  tabela derivada │ {}", path.display());
}
