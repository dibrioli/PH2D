//! **AS FIGURAS DO TUTORIAL DO CICLO 2** — geradas COZINHANDO os nós, nunca desenhadas à mão
//! ([doc 103 §3](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)).
//!
//! ⚠️ **Um animador não tem forma: ele tem MOVIMENTO**, e é isso que separa estas figuras das
//! do ciclo 1. Ali uma nuvem num instante dizia tudo; aqui um instante só diz metade — por isso
//! há três maneiras de olhar, e cada figura declara a sua:
//!
//! - **`Instante`** — a fila num momento. Serve quando a onda se *vê na fila* (é o que o
//!   `phase_stagger` faz: o desfasamento por elemento desenha a onda no espaço).
//! - **`Rasto`** — N instantes sobrepostos. É a única que mostra o *caminho* (a órbita).
//! - **`Fantasma`** — a entrada por baixo e a saída por cima, no mesmo instante: mostra o que
//!   o nó **fez**, e não o que ele produziu.
//!
//! `cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture dump_animadores_figures`

use super::tutorial_draw::{moldura, svg};
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, NodeId};

/// ⚠️ **Relativo ao MANIFESTO, não ao processo** — `cargo test` corre com a cwd na raiz do
/// pacote, e um gerador que escreve no sítio errado diz «ok».
const DIR: &str = "../../docs/Motion Nodes/tutoriais/fig";
/// O instante em que as figuras de `Instante` são cozidas. ⚠️ Não é `0`: em `t = 0` metade
/// destes nós está no ponto neutro e a figura sairia chapada.
const T: f64 = 0.37;
pub(super) const DT: f64 = 1.0 / 60.0;

#[derive(Clone, Copy)]
enum Como {
    Instante,
    Rasto(usize),
    Fantasma(usize),
    /// ⭐ **O deslocamento DESENHADO** — um traço de onde o elemento estava até onde ficou.
    ///
    /// ⚠️⚠️ **Foi preciso porque as outras três não conseguem mostrar um CAMPO.** Um campo
    /// coerente é *vizinhos a andarem juntos*, e uma nuvem de pontos deslocados não diz de onde
    /// cada um veio: com feição pequena lê-se estática, com feição grande lê-se uma grade quase
    /// parada, e **as duas leituras estão erradas sobre o mesmo produto**. O traço torna a
    /// coerência visível — é ela que alinha os traços vizinhos.
    Campo,
}

struct Fig {
    file: &'static str,
    node: &'static str,
    params: &'static [(&'static str, f32)],
    /// `(linhas, colunas, espaçamento)` da fila que entra.
    feed: (f32, f32, f32),
    /// Um nó ENTRE a grade e o da figura — o `delay` precisa de uma onda a que chegar tarde.
    pre: Option<(&'static str, &'static [(&'static str, f32)])>,
    como: Como,
    /// Figuras do mesmo grupo partilham a moldura, para se poderem comparar a olho.
    grupo: &'static str,
}

/// A fila comum das figuras de onda: 41 pontos numa linha, 12 de espaçamento.
const FILA: (f32, f32, f32) = (1.0, 41.0, 12.0);
/// ⭐ **Uma onda inteira ao longo da fila** — e o número não é escolhido: o `phase_stagger` é o
/// avanço de fase **por elemento, em ciclos**, então uma volta repartida por 40 vãos é `1/40`.
/// ⚠️⚠️ **A primeira redacção pôs `1,0` aqui** — um ciclo inteiro por elemento, que deixa os 41
/// **na mesma fase**: a fila desloca-se em bloco e não desenha onda nenhuma. As quatro ondas
/// bipolares passavam assim (uma fila deslocada ainda «mexeu»), e quem acusou foi o `Spike`, que
/// a essa fase vale zero para todos. *A régua tinha de ser a VARIAÇÃO ao longo da fila, não o
/// deslocamento — um bloco deslocado é exactamente o que ela não pode aceitar.*
const PASSO: f32 = 1.0 / (FILA.1 - 1.0);
const ONDA: &[(&str, f32)] = &[
    ("channel", 1.0),
    ("amplitude", 78.0),
    ("frequency", 0.7),
    ("phase_stagger", PASSO),
];

const FIGS: &[Fig] = &[
    // As cinco formas do Oscillator — a mesma fila, o mesmo instante, só o `wave` muda.
    Fig {
        file: "osc_sine",
        node: "motion.oscillator",
        params: &[
            ("wave", 0.0),
            ("channel", 1.0),
            ("amplitude", 78.0),
            ("frequency", 0.7),
            ("phase_stagger", PASSO),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "onda",
    },
    Fig {
        file: "osc_tri",
        node: "motion.oscillator",
        params: &[
            ("wave", 1.0),
            ("channel", 1.0),
            ("amplitude", 78.0),
            ("frequency", 0.7),
            ("phase_stagger", PASSO),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "onda",
    },
    Fig {
        file: "osc_square",
        node: "motion.oscillator",
        params: &[
            ("wave", 2.0),
            ("channel", 1.0),
            ("amplitude", 78.0),
            ("frequency", 0.7),
            ("phase_stagger", PASSO),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "onda",
    },
    Fig {
        file: "osc_saw",
        node: "motion.oscillator",
        params: &[
            ("wave", 3.0),
            ("channel", 1.0),
            ("amplitude", 78.0),
            ("frequency", 0.7),
            ("phase_stagger", PASSO),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "onda",
    },
    Fig {
        file: "osc_spike",
        node: "motion.oscillator",
        params: &[
            ("wave", 4.0),
            ("channel", 1.0),
            ("amplitude", 78.0),
            ("frequency", 0.7),
            ("phase_stagger", PASSO),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "onda",
    },
    // O tremor e o campo.
    Fig {
        file: "wiggle",
        node: "motion.wiggle",
        params: &[
            ("channel", 1.0),
            ("amplitude", 78.0),
            ("frequency", 1.2),
            ("seed", 5.0),
            ("octaves", 2.0),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "livre",
    },
    // ⚠️ **A escala do campo é a feição, e ela estava ao contrário na 1.ª tentativa:** com
    // `scale = 0,5` o ruído tem feição MENOR que o vão da grade e a figura sai como estática de
    // televisão — que é o oposto do que um campo COERENTE é. O que se vê aqui é vizinhos a
    // andarem juntos, que é a razão de o nó existir.
    // ⚠️ **E a segunda tentativa falhou por outro lado:** com o deslocamento MAIOR que o vão da
    // grade os pontos **cruzam-se**, e um campo perfeitamente coerente volta a ler-se como
    // estática. *A figura de um campo precisa das duas coisas — feição grande E vão maior que a
    // amplitude*, senão ela ensina ruído branco sobre um nó que não o produz.
    // ⚠️⚠️ **E a feição não é só o `scale`: são as OITAVAS.** O `at` devolve `(px·scale, py·scale)`
    // e o `fbm` soma `octaves` camadas com `lacunarity = 2` — com as 3 de omissão a camada mais
    // fina tem **4×** a frequência da base, então uma grade que mostra 2,4 períodos da base
    // mostra 9,6 da última e volta a ler-se aleatória. A figura pede **uma** oitava.
    Fig {
        file: "noise",
        node: "motion.noise",
        params: &[
            ("channel", 4.0),
            ("amplitude", 5.5),
            ("scale", 0.02),
            ("octaves", 1.0),
            ("seed", 3.0),
        ],
        feed: (16.0, 16.0, 9.0),
        pre: None,
        como: Como::Campo,
        grupo: "campo",
    },
    // ⭐ As TRÊS ordens do Stagger — o que a W2 acrescentou, e o que ela mediu que não precisa
    // de entrada própria: `Reverse` e `From Edges` são estas espelhadas pelo toggle que já havia.
    Fig {
        file: "stag_index",
        node: "motion.stagger",
        params: &[
            ("channel", 1.0),
            ("min", 0.0),
            ("max", 150.0),
            ("order", 0.0),
            ("ease_curve", 2.0),
            ("ease_dir", 2.0),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "stagger",
    },
    Fig {
        file: "stag_center",
        node: "motion.stagger",
        params: &[
            ("channel", 1.0),
            ("min", 0.0),
            ("max", 150.0),
            ("order", 1.0),
            ("ease_curve", 2.0),
            ("ease_dir", 2.0),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "stagger",
    },
    Fig {
        file: "stag_random",
        node: "motion.stagger",
        params: &[
            ("channel", 1.0),
            ("min", 0.0),
            ("max", 150.0),
            ("order", 2.0),
            ("seed", 7.0),
            ("ease_curve", 2.0),
            ("ease_dir", 2.0),
        ],
        feed: FILA,
        pre: None,
        como: Como::Instante,
        grupo: "stagger",
    },
    // A órbita só se vê no RASTO: num instante ela é uma fila rodada.
    Fig {
        file: "orbit",
        node: "motion.orbit",
        params: &[("speed", 120.0)],
        feed: (3.0, 3.0, 62.0),
        pre: None,
        como: Como::Rasto(180),
        grupo: "orbita",
    },
    // ⭐ O atraso: a onda que entra (fantasma) e a que sai depois de o passado pesar.
    Fig {
        file: "delay",
        node: "motion.delay",
        params: &[("channel", 1.0), ("mode", 2.0), ("ticks", 14.0)],
        feed: FILA,
        pre: Some(("motion.oscillator", ONDA)),
        como: Como::Fantasma(48),
        grupo: "atraso",
    },
];

/// As oito secções da tabela de controlos, na ordem em que o tutorial as apresenta.
const TABELA: [(&str, &str); 8] = [
    ("oscillator", "motion.oscillator"),
    ("lfo", "value.lfo"),
    ("wiggle", "motion.wiggle"),
    ("noise", "motion.noise"),
    ("stagger", "motion.stagger"),
    ("orbit", "motion.orbit"),
    ("spring", "motion.spring"),
    ("delay", "motion.delay"),
];

pub(super) fn liga(
    m: &mut MotionState,
    de: NodeId,
    dp: u16,
    para: NodeId,
    pp: u16,
    atrasado: bool,
) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, dp),
            to: (para, pp),
            delayed: atrasado,
        })
        .expect("as portas encaixam");
}

pub(super) fn no_com(m: &mut MotionState, tipo: &str, params: &[(&str, f32)]) -> NodeId {
    let id = m.doc.graph.add_node(tipo.to_string());
    for (p, v) in params {
        m.doc.graph.set_param(id, *p, *v);
    }
    id
}

/// `out --pre--> state`, quando o nó tem a porta de realimentação (a convenção sequencial).
pub(super) fn realimenta(m: &mut MotionState, id: NodeId) {
    use ph2d_nodegraph::cook::OpResolver;
    let tid = m.doc.graph.node(id).expect("no'").type_id();
    let tem = m
        .registry
        .resolve(tid)
        .is_some_and(|op| op.manifest().inputs.iter().any(|p| p.name == "state"));
    if tem {
        liga(m, id, 0, id, 1, true);
    }
}

/// Monta `grade [→ pre] [→ nó] → output`. `com_no = false` deixa o fantasma: a mesma cadeia
/// sem o nó da figura.
fn monta(m: &mut MotionState, fig: &Fig, com_no: bool) -> NodeId {
    let (r, c, g) = fig.feed;
    let feed = no_com(
        m,
        "motion.grid",
        &[("rows", r), ("cols", c), ("gap_x", g), ("gap_y", g)],
    );
    let mut ultimo = feed;
    if let Some((tipo, params)) = fig.pre {
        let p = no_com(m, tipo, params);
        liga(m, ultimo, 0, p, 0, false);
        ultimo = p;
    }
    if com_no {
        let n = no_com(m, fig.node, fig.params);
        liga(m, ultimo, 0, n, 0, false);
        realimenta(m, n);
        ultimo = n;
    }
    let out = m.doc.graph.add_node("motion.output".to_string());
    liga(m, ultimo, 0, out, 0, false);
    out
}

/// Coze **e faz o tique virar**.
///
/// ⚠️⚠️ **UMA ARESTA `delayed` NÃO SE CARREGA SOZINHA:** é o [`Cook::advance_tick`] que passa a
/// saída deste tique para a porta `state` do seguinte. Sem ele o `motion.delay` devolve o valor
/// vivo **para sempre** — a figura dele saiu **byte-idêntica** à cadeia sem o nó, e ele parecia
/// um controlo morto quando o morto era o meu laço. *Um nó com estado testado sem virar o tique
/// mede o primeiro tique dele, que é a identidade por desenho.*
pub(super) fn pontos(cook: &mut Cook, m: &MotionState, out: NodeId, t: f64) -> Vec<[f32; 2]> {
    let r = cook.cook(&m.doc.graph, &m.registry, out, t);
    let p = match r {
        Ok(v) => match v.first().map(|c| c.as_stream()).and_then(|s| s.get("P")) {
            Some(Column::Vec2(p)) => p.clone(),
            _ => Vec::new(),
        },
        Err(_) => Vec::new(),
    };
    let _ = cook.advance_tick(&m.doc.graph, &m.registry, t);
    p
}

/// **O que se DESENHA · o FANTASMA · o ÚLTIMO instante** — os três são precisos, e por motivos
/// diferentes: o primeiro é a figura, o segundo é a régua (e às vezes também se desenha), e o
/// terceiro é o que se compara com a régua quando a figura é um rasto de muitos instantes.
type Colheita = (Vec<[f32; 2]>, Vec<[f32; 2]>, Vec<[f32; 2]>);

/// `(o que se desenha, o fantasma, o ÚLTIMO instante)`.
///
/// ⚠️ **O fantasma é colhido SEMPRE, mesmo quando não se desenha** — ele é a régua de *«este
/// animador fez alguma coisa?»*, e sem ela a verificação de figura chapada seria **vácua** numa
/// figura cuja altura vem da própria grade (o campo de ruído é 2-D: ele tem altura mesmo
/// parado).
fn colhe(fig: &Fig) -> Colheita {
    let tiques = match fig.como {
        Como::Instante | Como::Campo => 1,
        Como::Rasto(n) | Como::Fantasma(n) => n,
    };
    let mut m = MotionState::new();
    let out = monta(&mut m, fig, true);
    let mut cook = Cook::new();
    let mut desenho: Vec<[f32; 2]> = Vec::new();
    let mut ultimo: Vec<[f32; 2]> = Vec::new();
    for k in 0..tiques {
        // ⚠️ Um `Cook` só, a avançar — é assim que o app corre, e um nó com estado (a órbita
        // não tem, o atraso tem) responde outra coisa a saltos.
        let t = if tiques == 1 { T } else { k as f64 * DT };
        ultimo = pontos(&mut cook, &m, out, t);
        if matches!(fig.como, Como::Rasto(_)) {
            desenho.extend(ultimo.iter().copied());
        }
    }
    if !matches!(fig.como, Como::Rasto(_)) {
        desenho = ultimo.clone();
    }
    // O fantasma: a MESMA cadeia sem o nó da figura, corrida os mesmos tiques.
    let mut mg = MotionState::new();
    let og = monta(&mut mg, fig, false);
    let mut cg = Cook::new();
    let mut fantasma = Vec::new();
    for k in 0..tiques {
        let t = if tiques == 1 { T } else { k as f64 * DT };
        fantasma = pontos(&mut cg, &mg, og, t);
    }
    (desenho, fantasma, ultimo)
}

#[test]
#[ignore = "gerador de figuras do tutorial"]
fn dump_animadores_figures() {
    let dir = std::path::Path::new(DIR);
    std::fs::create_dir_all(dir).expect("a pasta das figuras");
    let colhidas: Vec<Colheita> = FIGS.iter().map(colhe).collect();
    let grupos: std::collections::BTreeSet<&str> = FIGS.iter().map(|f| f.grupo).collect();
    // A moldura de cada grupo — e ela é também o DENOMINADOR da régua abaixo.
    let mut molduras: std::collections::BTreeMap<&str, (f32, f32, f32, f32)> = Default::default();
    for grupo in &grupos {
        let conjuntos: Vec<&[[f32; 2]]> = (0..FIGS.len())
            .filter(|&i| FIGS[i].grupo == *grupo)
            .flat_map(|i| [colhidas[i].0.as_slice(), colhidas[i].1.as_slice()])
            .collect();
        molduras.insert(grupo, moldura(&conjuntos));
    }
    // ⚠️ **A tabela imprime-se ANTES de a régua acusar.** Uma sonda que estoura na primeira
    // figura má obriga a uma corrida por figura — e cada uma custa aqui um build de release.
    eprintln!("\n  figura       │   pontos │ moldura │  variacao │ barra");
    let mut maus: Vec<String> = Vec::new();
    for (i, fig) in FIGS.iter().enumerate() {
        let (fortes, fantasma, ultimo) = &colhidas[i];
        assert!(
            !fortes.is_empty(),
            "a figura `{}` saiu VAZIA — uma figura vazia ensina o contrario do que acontece",
            fig.file
        );
        assert_eq!(
            ultimo.len(),
            fantasma.len(),
            "`{}`: as duas cadeias contam igual",
            fig.file
        );
        // ⚠️ **E uma figura em que o animador não fez NADA é o mesmo defeito com outra cara.**
        // A régua é a distância à MESMA cadeia sem ele — não a altura da figura, que num campo
        // 2-D vem da grade e estaria lá com o nó desligado.
        //
        // ⚠️⚠️ **E a régua errou uma SEGUNDA vez, no mesmo sítio:** a 1.ª versão desta linha
        // media a *magnitude* do deslocamento, que é **cega ao sinal** — e a onda QUADRADA põe
        // metade da fila em `+A` e a outra metade em `−A`, logo `|d|` vale o mesmo para os 41 e
        // a figura mais legível do conjunto foi acusada de não distinguir ninguém.
        // *Quando a grandeza tem direcção, uma norma responde a outra pergunta.*
        //
        // ⚠️ **E as duas maneiras de olhar fazem afirmações diferentes, logo pedem réguas
        // diferentes.** Um `Instante` afirma *«a fila desenha a forma»* — mede-se a VARIAÇÃO do
        // deslocamento ao longo dela. Um `Rasto` afirma *«isto anda»* — e ali a variação é a
        // errada: a órbita **fecha a volta**, então no último instante ela está onde começou e
        // uma régua de instante lê `4,35` sobre um círculo inteiro percorrido. Ali a pergunta é
        // o quanto o rasto se AFASTA da fila parada.
        let espalha = |eixo: usize| -> f32 {
            let d = ultimo.iter().zip(fantasma).map(|(a, b)| a[eixo] - b[eixo]);
            let (lo, hi) = d.fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v), h.max(v)));
            hi - lo
        };
        let mexeu = if matches!(fig.como, Como::Rasto(_)) {
            fortes
                .iter()
                .map(|p| {
                    fantasma
                        .iter()
                        .map(|g| ((p[0] - g[0]).powi(2) + (p[1] - g[1]).powi(2)).sqrt())
                        .fold(f32::MAX, f32::min)
                })
                .fold(0.0f32, f32::max)
        } else {
            espalha(0).max(espalha(1))
        };
        // ⚠️ **O denominador é a MOLDURA da figura, não a fila.** A figura é ajustada ao quadro,
        // então o que decide se o artista vê alguma coisa é a fracção do QUADRO — e foi por a
        // 1.ª régua usar a largura da fila que o campo de ruído, desenhado numa moldura oito
        // vezes maior por partilhar o grupo do wiggle, era acusado de estar parado.
        let (_, _, mw, mh) = molduras[fig.grupo];
        let lado = (mw * mh).sqrt();
        let barra = lado * 0.04;
        eprintln!(
            "  {:<12} │ {:>8} │ {lado:>7.0} │ {mexeu:>9.2} │ {barra:>5.2}",
            fig.file,
            fortes.len()
        );
        if mexeu <= barra {
            maus.push(format!(
                "`{}` (variacao {mexeu:.2} numa moldura de {lado:.0})",
                fig.file
            ));
        }
    }
    assert!(
        maus.is_empty(),
        "{} figura(s) nao mostram o animador a distinguir os elementos — uma fila deslocada em \
         BLOCO nao desenha nada:\n  {}",
        maus.len(),
        maus.join("\n  ")
    );

    for grupo in &grupos {
        let m = molduras[grupo];
        for i in (0..FIGS.len()).filter(|&i| FIGS[i].grupo == *grupo) {
            let path = dir.join(format!("{}.svg", FIGS[i].file));
            // ⚠️ O fantasma **desenha-se** só onde ele é a mensagem (o atraso e o campo); nas
            // outras ele serviu de régua e sai da figura, senão a fila parada rouba a atenção
            // da que anda.
            let campo = matches!(FIGS[i].como, Como::Campo);
            let sombra: &[[f32; 2]] = match FIGS[i].como {
                Como::Fantasma(_) | Como::Campo => &colhidas[i].1,
                _ => &[],
            };
            std::fs::write(&path, svg(&colhidas[i].0, sombra, m, campo)).expect("escrever");
            eprintln!("  escrito │ {}", path.display());
        }
    }
    super::animadores_spring_fig::dump_spring_curves(dir);
    let path = dir.join("params_animadores.html");
    std::fs::write(&path, super::tutorial_table::derive(&TABELA)).expect("a tabela");
    eprintln!("  tabela derivada │ {}", path.display());
}
