//! **OS TETOS DE INSTÂNCIA SÃO UM NÚMERO POR RECURSO** (CLAUDE.md §0, doc 89 folha 07, doc 112 §4).
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

/// O teto MEDIDO no caminho de CPU: a linha emitida custa ~10–28 ns nos nós que só correm lá, e
/// este é o ponto em que **um** nó passa a ocupar cerca de um terço de um quadro de 60 fps.
///
/// ⚠️ Este literal é um sítio a mais, e é de propósito: sem ele o gate compararia as consts
/// **umas com as outras** e ficaria verde no dia em que alguém as movesse todas juntas por
/// engano — um oráculo que usa a coisa sob teste para computar o que espera é sempre verde.
const MEASURED_CPU_CEILING: usize = 262_144;

/// O teto MEDIDO no DISPOSITIVO (`fx_row_ceiling_probe`): ~1,8 ns por linha, e a cadeia
/// `grid → oscillator → fx → output` ocupa `5,66–5,89 ms` (`34–35 %` de um quadro) a
/// `3 145 728` linhas — o MESMO critério de «um terço», no recurso novo.
const MEASURED_DEVICE_CEILING: usize = 3_145_728;

/// Os nós que só correm na CPU carregam o teto da CPU; os que têm kernel carregam o do
/// dispositivo. ⚠️ **As duas metades têm piso** (um grupo esvaziado por engano ficaria verde).
#[test]
fn the_instance_ceilings_agree_per_resource() {
    let device = [
        ("fx.drop_shadow", ph2d_node_fx_drop_shadow::MAX_INSTANCES),
        ("fx.rgb_split", ph2d_node_fx_rgb_split::MAX_INSTANCES),
    ];
    assert_eq!(device.len(), 2, "piso do grupo do dispositivo");
    for (name, c) in device {
        assert_eq!(
            c, MEASURED_DEVICE_CEILING,
            "{name} tem kernel e carrega um teto que nao e' o medido no dispositivo: {c} contra \
             {MEASURED_DEVICE_CEILING}. Mover um exige medir (fx_row_ceiling_probe) e mover o \
             grupo, com a tabela ao lado."
        );
    }
    let ceilings = [
        ("motion.trail", ph2d_node_motion_trail::MAX_INSTANCES),
        // ⚠️ **O `source.lsystem` limita MÓDULOS e não linhas — e é a MESMA grandeza um nível
        // acima.** A tartaruga emite no máximo um elemento por módulo (mais a raiz), então o
        // orçamento da cadeia é o que decide quantas linhas este nó entrega ao caminho de CPU.
        // A varredura dele (`measure_lsystem_ceiling.rs`) chegou a este número por outro
        // caminho — 38,8 % de um quadro para derivar — e concorda com o *"cerca de um terço"*
        // que decidiu os outros três. ⭐ *Duas medições independentes no mesmo sítio.*
        //
        // ⚠️ Ele emite no máximo `MAX_MODULES + 1` linhas: a raiz que a tartaruga planta antes
        // do primeiro símbolo não vem da cadeia, logo não é contada pelo orçamento dela. UMA
        // linha de folga, escrita aqui para o «mesmo número» não ser lido como mais do que é.
        ("source.lsystem", ph2d_node_source_lsystem::MAX_MODULES),
    ];
    assert_eq!(ceilings.len(), 2, "piso do grupo da CPU");
    for (name, c) in ceilings {
        assert_eq!(
            c, MEASURED_CPU_CEILING,
            "{name} carrega um teto de instancias que ninguem mediu junto com o grupo da CPU: \
             {c} contra {MEASURED_CPU_CEILING}. Eles limitam a MESMA grandeza (linhas emitidas \
             no caminho de CPU) — mover um exige medir e mover o grupo, com a tabela ao lado. \
             (Um no' que ganhou kernel muda de grupo: doc 112 §4.)"
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
