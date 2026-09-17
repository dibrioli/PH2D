//! ⭐⭐⭐ **O RELÓGIO DO GRUPO DAS FONTES** (ciclo 8, passo 5 —
//! [doc 113](../../../docs/Motion%20Nodes/113_ciclo_8_fontes_e_dados.md) §7).
//!
//! ⚠️⚠️ **Uma FONTE não se mede como um transforme, e a diferença não é de grau.** O relógio do
//! ciclo 7 punha o nó NO MEIO de uma cadeia com `n` objectos a entrar e perguntava o preço POR
//! OBJECTO. Aqui o nó é a CABEÇA: ele não recebe objectos, ele **faz** os objectos — e quantos ele
//! faz é governado, em cada um, por outra coisa (um ficheiro · um texto · uma gramática · uma taxa
//! · a vista). ⇒ **não existe um `n` do grupo**, e a tabela imprime a ALAVANCA de cada linha ao
//! lado da contagem. Uma coluna `n` só seria a média de sete perguntas diferentes.
//!
//! ⚠️⚠️ **Quatro coisas que uma tabela ingénua deste grupo mediria errado:**
//!
//! - **A MEMBRANA.** Cinco dos sete não fabricam nada: eles LEEM um external que a shell publicou
//!   (a lei do `source.shape`, do `source.text`, do `source.object`…). Um cook sem publicação lê o
//!   external **vazio** ⇒ *zero linhas, muito depressa* — a tabela mais rápida que este repo
//!   saberia imprimir, e sobre nada. ⇒ a sonda corre as membranas do PRODUTO **na ordem do
//!   quadro** ([`Membranas::publica`]), e a coluna `linhas` é a prova de que a publicação chegou.
//! - **OS DOIS RELÓGIOS.** *Publicar* e *cozer* são trabalhos diferentes sobre a mesma coisa, e um
//!   número só não diz qual — no §6 deste ciclo `6,05` dos `9,62 ms` eram a **republicação**.
//!   Depois da cura da W1 o primeiro é ~zero com a fonte parada, e isso **é** o resultado: mede-se
//!   para o poder afirmar.
//! - **A ROTA.** Uma fonte de FORMA VIVA recusa o dispositivo para a cadeia inteira
//!   (`graph_has_live_vector_source`) e as linhas dela saem como `VectorInstance`, não como
//!   quads — contar só `instances` leria **zero** sobre uma cena cheia de desenho.
//! - **O NEUTRO e o REGIME.** Metade do grupo nasce sem conteúdo (um `source.table` sem ficheiro é
//!   uma fonte de nada) e o emissor só tem a nuvem em regime depois de uma vida inteira de
//!   partícula ⇒ [`prepara`] arma cada um, e cronometra-se a **mediana** de [`AMOSTRAS`] quadros
//!   depois de [`AQUECE`] tiques.
//!
//! ⚠️ **Esta sonda mede o COZIMENTO de uma fonte, não o editor:** o `motion_bridge::dispatch` faz
//! antes disto o painel, os drenos e as intenções, e nada disso é do grupo.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture measure_the_source_group
//! ```
//! (imprime o `/proc/loadavg` na primeira linha — CLAUDE.md §5.0.)

use crate::motion_state::MotionState;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_editor_core::screens::layout::CenterSplit;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_nodegraph::graph::{Edge, NodeId};
use ph2d_render::Sprite;

const DT: f64 = 1.0 / 60.0;
/// Tiques antes de cronometrar. ⚠️ **O emissor é quem manda neste número:** a nuvem dele só está
/// em regime depois de uma vida inteira de partícula ([`VIDA`] segundos = `60` tiques), e uma
/// nuvem a meio de encher leria menos linhas por um preço menor.
const AQUECE: u64 = 120;
/// Quadros cronometrados depois do aquecimento; cita-se a MEDIANA.
const AMOSTRAS: u64 = 9;

/// O objecto que a membrana resolve — ⚠️ **o NOME é a referência inteira** (a lei do
/// `source.object`: renomear o objecto move o nó com ele).
const OBJECTO: &str = "Peca";
/// Um ladrilho do atlas de demonstração (`TextureAtlas::dummy` semeia as chaves `0..16`).
const LADRILHO: u32 = 5;
/// Linhas do CSV que arma o `source.table` — a MESMA porta de fixture da sonda da costura (§3.1),
/// para as duas medições do ciclo falarem do mesmo ficheiro.
const LINHAS_CSV: usize = 100_000;
/// Gerações do `source.lsystem`. ⚠️ Não é um teto: é a alavanca desta linha, impressa ao lado dela.
const GERACOES: f32 = 6.0;
/// A taxa e a vida do emissor ⇒ a nuvem em regime é `RITMO × VIDA` partículas.
const RITMO: f32 = 10_000.0;
const VIDA: f32 = 1.0;

fn liga(m: &mut MotionState, de: NodeId, para: NodeId) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .expect("as portas encaixam");
}

/// **As membranas do produto, na ORDEM do quadro.**
///
/// ⚠️ **A ordem é load-bearing e não é minha:** o `publish_shapes` **limpa** a tabela de externos,
/// os objectos acrescentam, os valores do editor vêm por último no espaço reservado `$`, e o
/// `publish_all` corre post-drain/pre-cook (ver `fase_motion_bridge.rs` e `motion_externals`).
/// Publicar por outra ordem mediria uma tabela a que falta metade, **sem erro nenhum**.
struct Membranas {
    sim: SimWorld,
    cena: ph2d_vec_scene::VecScene,
    mapa: ph2d_vec_entities::entities::VecEntityMap,
    xforms: ph2d_vec_scene::VecXforms,
    atlas: ph2d_render::TextureAtlas,
}

impl Membranas {
    fn new(gpu: &GpuContext) -> Self {
        let mut sim = SimWorld::new();
        // O sujeito do `source.object`: um sprite NOMEADO, que é o que a membrana resolve.
        sim.world_mut().spawn((
            Name::new(OBJECTO),
            Sprite::atlas(LADRILHO, [0.7, 0.7], [1.0, 1.0, 1.0, 1.0]),
            Transform::IDENTITY,
        ));
        Self {
            sim,
            cena: ph2d_vec_scene::VecScene::new(),
            mapa: ph2d_vec_entities::entities::VecEntityMap::default(),
            xforms: ph2d_vec_scene::VecXforms::default(),
            atlas: ph2d_render::TextureAtlas::dummy(gpu),
        }
    }

    fn publica(&mut self, m: &mut MotionState, t: f64) {
        let Self {
            sim,
            cena,
            mapa,
            xforms,
            atlas,
        } = self;
        crate::motion_bridge::publish_shapes(m, sim, cena, mapa, xforms, None);
        let cozida = |_: ph2d_asset::LogicalTextureId| None;
        crate::motion_bridge::publish_objects(
            m,
            sim,
            crate::motion_bridge::Appearance {
                atlas,
                cooked: &cozida,
            },
            t,
        );
        crate::motion_bridge::publish_editor_inputs(
            m,
            &ph2d_render::Camera2d::default(),
            (0.0, 0.0),
            CenterSplit::None,
            WindowSize::new(1600, 900),
        );
        crate::motion_externals::publish_all(m, t);
    }
}

/// **Arma a fonte e devolve a ALAVANCA dela, por extenso.**
///
/// ⛔⛔ **Uma fonte nova ESTOURA aqui, de propósito.** O braço `_` não mede o default: metade
/// deste grupo nasce sem conteúdo, e medir o neutro devolveria uma linha rápida sobre um stream
/// vazio — exactamente o modo de falha que o cabeçalho nomeia. Quem acrescentar uma fonte escreve
/// aqui o que a enche.
fn prepara(m: &mut MotionState, no: NodeId, tipo: &str) -> String {
    match tipo {
        "motion.emitter" => {
            m.doc.graph.set_param(no, "rate", RITMO);
            m.doc.graph.set_param(no, "life", VIDA);
            format!("rate {RITMO}/s × life {VIDA}s")
        }
        "source.lsystem" => {
            m.doc
                .graph
                .set_param(no, ph2d_node_source_lsystem::param::GENERATIONS, GERACOES);
            format!("generations {GERACOES}")
        }
        "source.object" => {
            // ⚠️ O nome do param é o literal do nó (ele guarda-o privado); o VALOR é o `Name` da
            // entidade acima, e é a igualdade dos dois que faz a membrana responder.
            m.doc.graph.set_text_param(no, "object", OBJECTO);
            format!("o objecto «{OBJECTO}» da cena")
        }
        "source.shape" => "a forma do cartão (um MOLDE)".to_string(),
        "source.table" => {
            m.doc.graph.set_text_param(
                no,
                ph2d_node_source_table::FILE_KEY,
                super::fontes_costura::tabela(LINHAS_CSV),
            );
            format!("ficheiro de {LINHAS_CSV} linhas")
        }
        "source.text" => {
            let texto = "A fonte do texto e uma instancia por GLIFO. ".repeat(6);
            let n = texto.chars().filter(|c| !c.is_whitespace()).count();
            m.doc
                .graph
                .set_text_param(no, ph2d_node_source_text::TEXT_KEY, texto);
            format!("texto de ~{n} glifos")
        }
        "source.camera" => "a vista do editor (1 linha por construção)".to_string(),
        outro => panic!(
            "fonte NOVA sem alavanca escrita: `{outro}` — sem ela esta sonda mediria o neutro \
             dela (um stream vazio, muito depressa). Escreva aqui o que a enche."
        ),
    }
}

struct Linha {
    linhas: usize,
    /// As que saíram como VECTOR (uma forma viva não é um quad).
    vectores: usize,
    publicar: f64,
    cozer: f64,
    rota: &'static str,
    alavanca: String,
}

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// `X → scale → output`, pela ponte do produto. `tipo = None` é o CONTROLO: uma `motion.grid` de
/// `lado²`, cuja cadeia inteira vive na placa.
fn mede(gpu: &GpuContext, tipo: Option<&str>, lado: f32) -> Linha {
    let mut m = MotionState::new();
    let (cabeca, alavanca) = match tipo {
        Some(t) => {
            let n = m.doc.graph.add_node(t.to_string());
            let a = prepara(&mut m, n, t);
            (n, a)
        }
        None => {
            let g = m.doc.graph.add_node("motion.grid".to_string());
            m.doc.graph.set_param(g, "rows", lado);
            m.doc.graph.set_param(g, "cols", lado);
            (g, format!("{lado} × {lado}"))
        }
    };
    let s = m.doc.graph.add_node("motion.scale".to_string());
    let o = m.doc.graph.add_node("motion.output".to_string());
    liga(&mut m, cabeca, s);
    liga(&mut m, s, o);
    m.sinks = vec![o];
    let mut memb = Membranas::new(gpu);
    let escopos = ph2d_nodegraph::cook::TimeScopes::new();
    let (mut pub_ms, mut cozer_ms) = (Vec::new(), Vec::new());
    let (mut linhas, mut vectores) = (0usize, 0usize);
    for f in 0..AQUECE + AMOSTRAS {
        let t = f as f64 * DT;
        let t0 = std::time::Instant::now();
        memb.publica(&mut m, t);
        let t1 = std::time::Instant::now();
        // ⚠️ **As DUAS linhas do quadro do produto** (`motion_bridge` §cook): o dispositivo
        // responde, ou a bomba da CPU coze os sinks. Medir só a primeira leria uma recusa como
        // se ela fosse gratuita.
        let device = matches!(
            super::super::gpu::cook_gpu(&mut m, gpu, f, DT, &escopos),
            super::super::gpu::GpuOutcome::Handled
        );
        if device {
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            linhas = m.gpu_cook.instances().map_or(0, |b| b.len() as usize);
            vectores = 0;
        } else {
            for tick in crate::motion_bridge::ticks_owed(m.pump.last_cooked_tick(), f) {
                m.pump.advance_or_scrub_scoped(
                    &m.doc.graph,
                    &m.registry,
                    &m.sinks,
                    tick,
                    |t| t as f64 * DT,
                    m.default_uv_rect,
                    m.default_size,
                    &escopos,
                );
            }
            linhas = m.pump.instances.len();
            vectores = m.pump.vector_instances.len();
        }
        if f >= AQUECE {
            pub_ms.push((t1 - t0).as_secs_f64() * 1000.0);
            cozer_ms.push(t1.elapsed().as_secs_f64() * 1000.0);
        }
    }
    Linha {
        linhas,
        vectores,
        publicar: mediana(pub_ms),
        cozer: mediana(cozer_ms),
        rota: m.route_said.unwrap_or("dispositivo"),
        alavanca,
    }
}

#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE, com adaptador e com a máquina calma"]
fn measure_the_source_group() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adaptador — esta sonda corre a ponte do produto, que coze na placa");
    };
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!(
        "  X -> scale -> output pela ponte do produto · regime = mediana de {AMOSTRAS} quadros \
         depois de {AQUECE} tiques\n"
    );
    eprintln!(
        "  {:<16} │ {:>9} │ {:>8} │ {:>6} │ {:>6} │ {:>8} │ a alavanca / a rota",
        "nó", "linhas", "publicar", "cozer", "quadro", "ns/linha"
    );
    eprintln!(
        "  -----------------|-----------|----------|--------|--------|----------|-------------------"
    );
    let imprime = |rotulo: &str, l: &Linha| {
        let total = l.linhas + l.vectores;
        let contagem = if l.vectores > 0 {
            format!("{total} (vec)")
        } else {
            format!("{total}")
        };
        let quadro = l.publicar + l.cozer;
        let ns = if total == 0 {
            "—".to_string()
        } else {
            format!("{:.1}", quadro * 1e6 / total as f64)
        };
        eprintln!(
            "  {rotulo:<16} │ {contagem:>9} │ {:>8.2} │ {:>6.2} │ {quadro:>6.2} │ {ns:>8} │ {}",
            l.publicar, l.cozer, l.alavanca
        );
        if l.rota != "dispositivo" {
            eprintln!(
                "  {:<16} │ {:>9} │          │        │        │          │ ⛔ {}",
                "", "", l.rota
            );
        }
    };
    let grupo = crate::motion_fontes_probe::grupo();
    assert!(
        grupo.len() >= 6,
        "piso de populacao do grupo das fontes: {grupo:?}"
    );
    let mut maior = 0usize;
    let mut linhas = Vec::new();
    for no in &grupo {
        let l = mede(&gpu, Some(no), 0.0);
        assert!(
            l.linhas + l.vectores > 0,
            "`{no}` nao emitiu linha nenhuma — a membrana dele nao publicou, e uma linha de ZERO \
             mede um stream vazio (ver o cabecalho)"
        );
        maior = maior.max(l.linhas + l.vectores);
        linhas.push((*no, l));
    }
    for (no, l) in &linhas {
        imprime(no, l);
    }
    // ⭐ O CONTROLO, no fim e com a contagem do MAIOR do grupo: é ele que separa *«o que custa
    // esta fonte»* de *«o que custam estas linhas»*.
    #[expect(clippy::cast_precision_loss, reason = "o lado de uma grade")]
    let lado = (maior as f64).sqrt().round() as f32;
    let c = mede(&gpu, None, lado);
    eprintln!(
        "  -----------------|-----------|----------|--------|--------|----------|-------------------"
    );
    imprime("motion.grid", &c);
    eprintln!(
        "\n  (o CONTROLO é a última linha: a mesma contagem com a cadeia INTEIRA na placa.\n   \
         `publicar` = as membranas do produto na ordem do quadro · `cozer` = o dispositivo, ou a \
         bomba da CPU quando ele recusa.\n   `(vec)` = as linhas saíram como forma VIVA, não como \
         quad. Um quadro de 60 fps tem 16,67 ms.)\n"
    );
}
