//! ⭐⭐ **O PREÇO DOS CORPOS MOLES NA CPU** (ciclo 9, W4/W5 — doc 114 §8).
//!
//! Irmão do [`crate::motion_rig_probe`] pelo tecto de LOC, como o relógio do ciclo 8 é irmão da
//! auditoria dele. Aquele diz **onde** cada nó corre e **porquê**; este diz **quanto custa**.
//!
//! ## A pergunta que esta sonda existe para responder
//!
//! A §8 do doc 114 mostra que o `motion.wave` **não tem bloqueador estrutural** — o passo dele é um
//! estêncil explícito, o caso embaraçosamente paralelo. Mas *«não tem bloqueador»* não é *«vale um
//! kernel»*: se o estêncil já couber num quadro na malha que o artista usa, escrever WGSL é gastar
//! uma wave para comprar nada. ⇒ **é esta escada que transforma a aposta numa decisão.**
//!
//! ## A ESCADA, e não um ponto
//!
//! Um número só diz *«custa X»*; a escada diz **como cresce**, que é a pergunta de quem decide um
//! tecto (§0.0). Um estêncil é `O(células)`, logo quadruplicar o lado deve quadruplicar o relógio —
//! e um desvio disso é ele próprio um achado.
//!
//! ## ⛔ E a escada bate num TECTO que ninguém mediu
//!
//! O `motion.wave` prende o lado em **`MAX_SIDE = 60`**, e a justificação escrita ao lado dele é
//! *«field cost is O(rows·cols)»* — que é uma **lei de crescimento e não um recurso**. O §0.0 pede
//! o contrário: *um limite legítimo diz de que recurso ele é, e traz a medição*.
//!
//! ⚠️⚠️ **E o doc 91 NÃO o cobre, apesar de nomear este nó** — ele auditou o `MAX_DT` do
//! `motion.wave` (e removeu-o, por inerte, em 2026-08-27); a palavra `MAX_SIDE` não aparece lá uma
//! única vez. *Uma auditoria de tectos responde pelos tectos que olhou*, e um nó que aparece numa
//! lista de dívida paga lê-se como um nó sem dívida. ⇒ esta escada é o instrumento do tecto que
//! ninguém olhou.
//!
//! ⚠️ **Nenhuma leitura daqui vale nada acima de `load ~5`** (`CLAUDE.md` §5.0). A sonda imprime o
//! `loadavg` na primeira linha, e quem a corre à mão corre-a pelo vigia
//! (`docs/Motion Nodes/ferramentas/medir_quando_calmo.sh`).

use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use std::time::Instant;

/// Tiques de aquecimento antes de medir — um solver com estado só entra em regime depois de o laço
/// dar algumas voltas, e medir o transitório mede outra coisa.
const AQUECE: usize = 60;
/// Quadros medidos; fica-se com a MEDIANA (a média deixa um pico de escalonador decidir a tabela).
const AMOSTRAS: usize = 9;
/// O passo de tempo de um quadro a 60 fps — o relógio que o artista tem.
const DT: f64 = 1.0 / 60.0;

/// Um nó do grupo, com a porta por onde o estado dele volta e a alavanca que o faz crescer.
struct Mole {
    tipo: &'static str,
    /// A porta `state` — o índice conta-se no manifesto do nó, nunca se adivinha.
    porta_estado: u16,
    /// `(param, param)` do tamanho: um lado, ou uma contagem só (o segundo é `None`).
    tamanho: (&'static str, Option<&'static str>),
    alavanca: &'static str,
}

/// Os quatro medidos — os três corpos moles mais o `motion.boids`.
///
/// ⛔⛔ **O `motion.boids` NÃO é aqui «o controlo do dispositivo», e a primeira redacção desta
/// sonda rotulou-o assim.** Este arnês coze com o [`Cook`] da **CPU**: o que ele mede do boids é a
/// rota LENTA dele, não a placa. *Um controlo que não percorre o mesmo caminho não controla nada.*
///
/// ⭐ Ele fica — com o rótulo certo — porque assim diz outra coisa, e mais útil: **quanto custa o
/// recuo** do único nó do grupo que tem para onde recuar.
const MOLES: &[Mole] = &[
    Mole {
        tipo: "motion.wave",
        porta_estado: 1,
        tamanho: ("rows", Some("cols")),
        alavanca: "estencil explicito 5-pontos",
    },
    Mole {
        tipo: "motion.soft_body",
        porta_estado: 2,
        tamanho: ("rows", Some("cols")),
        alavanca: "shape matching (reducoes globais)",
    },
    Mole {
        tipo: "motion.verlet_rope",
        porta_estado: 2,
        tamanho: ("count", None),
        alavanca: "relaxacao Gauss-Seidel",
    },
    Mole {
        tipo: "motion.boids",
        porta_estado: 2,
        tamanho: ("count", None),
        alavanca: "a rota LENTA de quem tem placa",
    },
];

/// Monta `<nó> → motion.output` com a aresta de estado ATRASADA sobre si mesmo, que é como toda
/// simulação desta casa fecha o laço.
fn monta(m: &crate::motion_state::MotionState, mole: &Mole, lado: usize) -> (Graph, NodeId) {
    let mut g = Graph::default();
    let _ = m;
    let n = g.add_node(mole.tipo.to_string());
    #[expect(
        clippy::cast_precision_loss,
        reason = "um lado de grelha, sempre pequeno"
    )]
    let v = lado as f32;
    match mole.tamanho {
        (a, Some(b)) => {
            g.set_param(n, a, v);
            g.set_param(n, b, v);
        }
        // Uma contagem só recebe o QUADRADO do lado, para a escada ser comparável com as grelhas.
        (a, None) => g.set_param(n, a, v * v),
    }
    let saida = g.add_node("motion.output".to_string());
    g.connect(Edge {
        from: (n, 0),
        to: (saida, 0),
        delayed: false,
    })
    .expect("fio de saida");
    // ⚠️ **A aresta de estado é DELAYED** — sem isso o grafo tem um ciclo e o cozedor recusa-o.
    g.connect(Edge {
        from: (n, 0),
        to: (n, mole.porta_estado),
        delayed: true,
    })
    .expect("fio de estado");
    (g, saida)
}

/// A mediana de [`AMOSTRAS`] quadros em regime, em milissegundos.
fn mede(m: &crate::motion_state::MotionState, mole: &Mole, lado: usize) -> (f64, usize) {
    let (g, saida) = monta(m, mole, lado);
    let mut cook = Cook::new();
    let mut t = 0.0f64;
    for _ in 0..AQUECE {
        let _ = cook.cook(&g, &m.registry, saida, t);
        let _ = cook.advance_tick(&g, &m.registry, t);
        t += DT;
    }
    let mut ms = Vec::with_capacity(AMOSTRAS);
    let mut linhas = 0usize;
    for _ in 0..AMOSTRAS {
        let agora = Instant::now();
        if let Ok(v) = cook.cook(&g, &m.registry, saida, t) {
            linhas = v[0].as_stream().count();
        }
        ms.push(agora.elapsed().as_secs_f64() * 1e3);
        let _ = cook.advance_tick(&g, &m.registry, t);
        t += DT;
    }
    ms.sort_by(f64::total_cmp);
    (ms[ms.len() / 2], linhas)
}

/// ⭐⭐⭐ **A ESCADA DO PREÇO** — corra-a à mão, em RELEASE, com a máquina calma.
#[test]
#[ignore = "sonda de preço — corra à mão, em RELEASE, com a máquina calma"]
fn measure_the_soft_body_group() {
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!(
        "  regime = mediana de {AMOSTRAS} quadros depois de {AQUECE} tiques · um quadro de 60 fps tem 16,67 ms\n"
    );
    eprintln!(
        "  {:<20} │ {:>5} │ {:>8} │ {:>9} │ {:>9} │ a alavanca",
        "nó", "lado", "linhas", "quadro/ms", "ns/linha"
    );
    eprintln!(
        "  ---------------------|-------|----------|-----------|-----------|------------------"
    );
    let m = crate::motion_state::MotionState::new();
    for mole in MOLES {
        // ⚠️ `60` é o TECTO do `motion.wave` (`MAX_SIDE`), não um número escolhido: pedir `64`
        // devolve `3 600` células, não `4 096`. A escada pára ali de propósito — medir além do
        // tecto pela porta do produto é impossível, e é isso que esta linha torna visível.
        for lado in [16usize, 32, 60] {
            let (ms, linhas) = mede(&m, mole, lado);
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de linhas")]
            let ns = if linhas == 0 {
                "—".to_string()
            } else {
                format!("{:.1}", ms * 1e6 / linhas as f64)
            };
            eprintln!(
                "  {:<20} │ {lado:>5} │ {linhas:>8} │ {ms:>9.3} │ {ns:>9} │ {}",
                mole.tipo, mole.alavanca
            );
        }
        eprintln!(
            "  ---------------------|-------|----------|-----------|-----------|------------------"
        );
    }
    eprintln!(
        "\n  (a contagem de um nó de CONTAGEM é o lado ao QUADRADO, para a escada ser comparável\n   \
         com as grelhas. `ns/linha` só se compara entre linhas da MESMA contagem.)\n"
    );
}

/// ⚠️ **O GRUPO DESTA SONDA ESTÁ VIVO** — e as portas de estado são CONTADAS do manifesto.
///
/// ⛔ Sem isto, um nó que ganhasse uma porta nova passaria a ter a aresta de estado ligada à porta
/// ERRADA, e a sonda mediria um solver que nunca recebe o próprio passado — *uma simulação sem
/// estado é barata, e a tabela sairia rápida e falsa*.
#[test]
fn as_portas_de_estado_da_sonda_sao_as_do_manifesto() {
    let m = crate::motion_state::MotionState::new();
    assert_eq!(MOLES.len(), 4, "a sonda mede os 3 moles + o controlo");
    for mole in MOLES {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(mole.tipo);
        let man = m
            .registry
            .manifests()
            .find(|man| man.id == tid)
            .unwrap_or_else(|| panic!("`{}` nao esta' registado", mole.tipo));
        let porta = man
            .inputs
            .get(mole.porta_estado as usize)
            .unwrap_or_else(|| panic!("`{}` nao tem porta {}", mole.tipo, mole.porta_estado));
        assert_eq!(
            porta.name, "state",
            "a porta {} do `{}` chama-se `{}`, nao `state` — a sonda mediria o solver sem o \
             proprio passado",
            mole.porta_estado, mole.tipo, porta.name
        );
        for (p, _) in [mole.tamanho].iter().map(|(a, b)| (*a, *b)) {
            assert!(
                man.params.iter().any(|s| s.name == p),
                "`{}` nao tem o param de tamanho `{p}`",
                mole.tipo
            );
        }
    }
}
