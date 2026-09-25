//! **OS TETOS DE INSTÂNCIA SÃO O NÚMERO MEDIDO DE CADA UM** (CLAUDE.md §0, doc 89 folha 07, doc 112 §4).
//!
//! `motion.trail`, `fx.drop_shadow` e `fx.rgb_split` limitavam a MESMA grandeza — quantas linhas
//! um nó pode emitir no caminho de CPU — e por isso carregavam o mesmo teto. Eles são
//! **drop-crates** e não podem depender uns dos outros (ADR-0075), então a const é copiada em
//! cada um, exactamente como o `falloff_at` das behaviours.
//!
//! ⚠️⚠️ **Em 2026-09-16 os dois `fx.*` ganharam kernel e o recurso DELES mudou** (ciclo 7, doc 112
//! §4): quem decide o teto de um nó que corre no dispositivo é o dispositivo (`CLAUDE.md` §0.0).
//! O grupo partiu-se em DOIS, cada um com o seu literal medido — e *a afirmação antiga («os três
//! são um número») seria agora a catraca que segura o produto no caminho lento*.
//!
//! ⚠️ **Uma const copiada em N sítios é N respostas esperando divergir**, e a única coisa que a
//! mantém honesta é este gate: esta crate é a que vê todos. Quem medir de novo e mover um deles
//! move o GRUPO dele, ou fica vermelho aqui.
//!
//! ⚠️ **As medições vivem nas sondas** — `measure_instance_ceiling.rs` (CPU) e
//! `ph2d-gpu-cook/tests/it/gpu_cpu_parity_fx.rs::fx_row_ceiling_probe` (dispositivo) — e a tabela
//! está no doc-comment de cada const.

/// ⛔⛔⛔ **ESTE TECTO DEIXOU DE SER UM RECURSO EM 2026-09-22, E A DISTINÇÃO É O PONTO.**
///
/// Ele foi `262 144`, e esse número era **de um recurso**: `measure_lsystem_ceiling.rs` mediu
/// derivar + interpretar a `~24 ns` por elemento, `38,8 %` de um quadro a `262 145`. Hoje ele é
/// `32 767`, e o número vem de uma **ORDEM DO DONO** (*«nenhum [nó] pode gerar mais de 16384
/// objetos»*, 2026-09-21, e o limite DOBRADO para `32 768` no ciclo 12) — não de uma medição.
///
/// ⚠️⚠️ **Um limite legítimo diz DE QUE RECURSO ele é** (`CLAUDE.md` §0.0), e este não diz: ele é
/// uma decisão de PRODUTO sobre quantos objectos um nó pode pôr na cena. Escrevê-lo aqui sem esta
/// linha faria a próxima pessoa procurar a medição que o justifica — e não há nenhuma.
///
/// ⭐ **A medição de `262 144` NÃO foi apagada**: ela continua no `measure_lsystem_ceiling.rs` e
/// continua verdadeira sobre o RELÓGIO. O que mudou foi que o produto deixou de poder lá chegar.
///
/// ⚠️ Cada literal aqui é um sítio a mais, e é de propósito: sem eles o gate compararia as consts
/// **umas com as outras** e ficaria verde no dia em que alguém as movesse todas juntas por
/// engano — um oráculo que usa a coisa sob teste para computar o que espera é sempre verde.
///
/// ⛔⛔ **E é um LITERAL, e a 1.ª redacção desta linha violava o parágrafo de cima** (auditoria do
/// fecho, 2026-09-24): ela escrevia `MAX_INSTANCIAS_POR_NO - 1`, a MESMA expressão do
/// `MAX_MODULES` — a linha comparava um valor consigo mesmo e não podia reprovar.
const MEASURED_LSYSTEM_CEILING: usize = 32_767;

/// O teto MEDIDO no DISPOSITIVO para quem REÚNE (`fx_row_ceiling_probe`): ~1,8 ns por linha, e a
/// cadeia `grid → oscillator → fx → output` ocupa `5,66–5,89 ms` (`34–35 %` de um quadro) a
/// `3 145 728` linhas — o critério de «um terço», no recurso novo.
const MEASURED_FX_DEVICE_CEILING: usize = 3_145_728;

/// O teto MEDIDO no DISPOSITIVO para o RASTRO (`trail_row_ceiling_probe`): ~2,6 ns por linha (a
/// compactação e a junção), `5,15–5,46 ms` (`31–33 %`) a `2 097 152` linhas.
const MEASURED_TRAIL_DEVICE_CEILING: usize = 2_097_152;

/// **Cada tecto é o SEU número medido** — os dois `fx.*` partilham um (a mesma lei de custo),
/// o rastro e o L-System têm o deles. ⚠️ A tabela tem piso: uma linha apagada por engano
/// deixava um tecto livre de mover sem medição.
#[test]
fn the_instance_ceilings_agree_per_resource() {
    let tetos = [
        (
            "fx.drop_shadow",
            ph2d_node_fx_drop_shadow::MAX_INSTANCES,
            MEASURED_FX_DEVICE_CEILING,
            "fx_row_ceiling_probe",
        ),
        (
            "fx.rgb_split",
            ph2d_node_fx_rgb_split::MAX_INSTANCES,
            MEASURED_FX_DEVICE_CEILING,
            "fx_row_ceiling_probe",
        ),
        (
            "motion.trail",
            ph2d_node_motion_trail::MAX_INSTANCES,
            MEASURED_TRAIL_DEVICE_CEILING,
            "trail_row_ceiling_probe",
        ),
        // ⚠️ **O `source.lsystem` limita MÓDULOS e não linhas** — a tartaruga emite no máximo um
        // elemento por módulo (mais a raiz: `MAX_MODULES + 1` linhas, UMA de folga). Ele corre só
        // na CPU e o número é o da varredura dele. Até à W1d do ciclo 7 ele coincidia com o do
        // rastro e dos `fx.*` (*"duas medições independentes no mesmo sítio"*); os três ganharam
        // kernel e mediram o deles no dispositivo.
        (
            "source.lsystem",
            ph2d_node_source_lsystem::MAX_MODULES,
            MEASURED_LSYSTEM_CEILING,
            "measure_lsystem_ceiling",
        ),
    ];
    assert_eq!(tetos.len(), 4, "piso da tabela");
    for (name, c, medido, sonda) in tetos {
        assert_eq!(
            c, medido,
            "{name} carrega um teto de instancias que nao e' o medido: {c} contra {medido}. \
             Mover um exige medir ({sonda}) e mover o literal daqui, com a tabela ao lado."
        );
    }
}

/// **E o teto tem de estar ACIMA do caso que a nota antiga citava — medido no PRODUTO.**
///
/// ⚠️ A justificativa que shipava era *"4096 vivas × 32 ecos já é 131k quads"* — e sob o teto
/// de `65_536` esse pedido era **CLAMPADO em silêncio** para 16 gerações: o artista pedia 32 e
/// recebia metade, sem nada na tela a dizer porquê. Este gate pina que o caso deixou de ser
/// recusado, que é a metade da wave que o artista VÊ.
///
/// ⚠️ **Ele COZINHA o grafo em vez de comparar duas constantes** — uma comparação de consts é
/// dobrada pelo compilador (não pode falhar em tempo de execução) e, pior, seria cega ao
/// `MAX_LENGTH` do nó, que é privado e capa as gerações por outro caminho. O oráculo é o número
/// de linhas que o cook de facto emite.
#[test]
fn the_case_the_old_note_cited_now_fits() {
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");

    // 64 x 64 = 4096 vivas, o número exacto da nota antiga.
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 64.0);
    g.set_param(seed, "cols", 64.0);
    let tr = g.add_node("motion.trail");
    g.set_param(tr, "length", 32.0);
    g.connect(Edge {
        from: (seed, 0),
        to: (tr, 0),
        delayed: false,
    })
    .expect("feed");
    g.connect(Edge {
        from: (tr, 0),
        to: (tr, 1),
        delayed: true,
    })
    .expect("ring");

    let mut cook = Cook::new();
    let mut rows = 0usize;
    // A cauda leva `length` ticks a encher; 40 dá folga.
    for t in 0..40 {
        let t = f64::from(t);
        rows = cook.cook(&g, &reg, tr, t).expect("cooks")[0]
            .as_stream()
            .count();
        cook.advance_tick(&g, &reg, t).expect("advance");
    }
    assert_eq!(
        rows, 131_072,
        "4096 vivas x 32 ecos tinham de emitir 131072 linhas; {rows} significa que o teto de \
         instancias voltou a CLAMPAR em silencio o caso que a nota antiga citava"
    );
}
