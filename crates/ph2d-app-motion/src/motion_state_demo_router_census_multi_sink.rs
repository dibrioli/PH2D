//! **A SONDA DA ESCADA DO MULTI-SINK** — irmã por RESPONSABILIDADE (HR-18) das do
//! [`super::census`], que mede o que as cenas AUTORAM. Esta mede **o que levantar a escada
//! COMPRARIA**, que é outra pergunta e é a que decide se vale a pena levantá-la.
//!
//! ⛔⛔ **Por que ela existe, e o que o censo irmão NÃO responde.** O
//! [`super::census::motion_route_census`] conta CENAS: `83` de `126` caem para a CPU por terem
//! mais de um sink, e em `28` delas **todos** os sinks já seriam reclamados pelo dispositivo.
//! Esse número diz que a escada é *alcançável* — ele não diz que ela é *alta*.
//!
//! ⚠️⚠️ **E há uma medição deste módulo que sugeria o contrário:** a auditoria de 2026-09-22 mediu
//! a pior cena do catálogo em `16 384` linhas, e **uma só** acima de `10 000`. Se as `83` cenas
//! multi-sink forem todas pequenas, o dispositivo perde para a CPU nelas — *uma corrida de GPU
//! tem um custo fixo de subida e despacho que uma corrente de duzentas linhas não amortiza*, e a
//! escada seria uma ponte sobre uma poça.
//!
//! ⇒ a grandeza é a **CONTAGEM DE LINHAS**, não o relógio, e isso é deliberado: um relógio desta
//! workstation não vale nada acima de `load ~5` (`CLAUDE.md` §5.0) e uma contagem é a mesma em
//! qualquer máquina. *A pergunta «quanto trabalho há aqui» responde-se contando o trabalho.*
//!
//! ⭐⭐⭐ **O QUE ELA DEU (2026-09-22, com o tecto já dobrado), e ele corrige o titular da
//! auditoria:**
//!
//! ```text
//!   83 cenas multi-sink   ·   54 771 linhas SOMADAS em todas elas
//!   a maior              : 32 762 linhas  (cena =107)
//!   a segunda            :  1 800 linhas  (cena  =97)
//!   acima do joelho      : 1 de 83
//!   prontas (todos os sinks já seriam device) : 28   ·   dessas, acima do joelho : 1
//! ```
//!
//! ⛔⛔ **`83 de 126` conta CENAS, não TRABALHO** — e as duas leituras levam a decisões opostas.
//! `82` das `83` cabem em `1 800` linhas ou menos, onde o dispositivo não se paga. A escada tem,
//! no catálogo de demos, **UM** sujeito medido: a `=107`.
//!
//! ⭐⭐ **E esse sujeito é consequência directa da ordem do dono de 2026-09-22.** A `=107` deriva
//! a população do tecto de instâncias (`SIDE = LADO_MAX_DE_GRELHA`), logo dobrar o tecto
//! **dobrou o trabalho dela** — de `16 385` para `32 762` linhas — e ela está na CPU *só* por
//! causa desta escada, com os dois sinks dela já reclamáveis pelo dispositivo.
//!
//! ⚠️ **Isto NÃO diz que a escada não vale a pena**: as cenas de demo não são o produto. Um
//! documento de artista com dois sinks pode ter `2 × 32 768` objectos, e o tecto que os autoriza
//! foi levantado hoje. O que a sonda diz é onde está o sujeito **medido** — e que um titular de
//! `83` cenas não é um titular de `83` cenas de trabalho.
//!
//! `cargo test -p ph2d-app-motion --lib a_escada_do_multi_sink -- --ignored --nocapture`

use super::*;

/// Quantas linhas o sink `s` emite no instante `t`, ou `None` se ele estourar / não cozer.
///
/// ⚠️ **A rede existe pela razão que o censo do tecto de instâncias já pagou:** *um censo que
/// morre no primeiro acusado não mede os outros oitenta e dois*.
fn linhas_do_sink(
    g: &ph2d_nodegraph::graph::Graph,
    reg: &ph2d_node_registry::NodeRegistry,
    s: ph2d_nodegraph::graph::NodeId,
    t: f64,
) -> Option<usize> {
    let anterior = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut cook = ph2d_nodegraph::cook::Cook::new();
        cook.cook(g, reg, s, t)
            .ok()
            .and_then(|out| out.first().map(|v| v.as_stream().count()))
    }));
    std::panic::set_hook(anterior);
    r.ok().flatten()
}

/// ⭐⭐⭐ **QUANTO TRABALHO ESTÁ PRESO ATRÁS DA ESCADA DO MULTI-SINK.**
///
/// Para cada cena com mais de um sink: quantos sinks, quantas linhas cada um coze, e se o plano
/// de CADA UM, sozinho, já seria reclamado inteiro pelo dispositivo.
///
/// ⚠️ **Ela IMPRIME e não julga** (é a lei das sondas deste módulo). O que ela entrega é a
/// DISTRIBUIÇÃO — porque uma média esconderia exactamente o que interessa: se o trabalho está
/// concentrado em duas cenas grandes ou espalhado por oitenta pequenas, a decisão é OUTRA.
#[test]
#[ignore = "sonda de escada, não um gate — `-- --ignored --nocapture`"]
fn a_escada_do_multi_sink_mede_se_e_alta() {
    /// O instante em que se coze. Não é `0.0`: um emissor a `t = 0` ainda não nasceu ninguém, e
    /// medir a população de uma simulação no tique zero mede o vazio dela.
    const INSTANTE: f64 = 1.0;
    /// Abaixo disto o dispositivo não se paga, e o número é do PRÓPRIO módulo: o
    /// `PAR_THRESHOLD` do cozimento em paralelo (`8 192`) é onde ESTA casa já mediu que vale a
    /// pena sair do caminho de um núcleo só. ⛔ Ele não é o joelho da GPU — é o único joelho
    /// medido que existe aqui, e usá-lo como régua é mais honesto do que escolher um.
    const JOELHO: usize = ph2d_nodegraph::attr::PAR_THRESHOLD;

    let mut linhas: Vec<(u32, usize, usize, bool)> = Vec::new(); // (cena, sinks, linhas, todos_gpu)
    for level in 1..=MAX_DEMO_LEVEL {
        let mut state = MotionState::new();
        let sinks =
            crate::motion_demo_legend::monta(&level.to_string(), &mut state.doc, &state.registry).0;
        if sinks.len() < 2 {
            continue;
        }
        let total: usize = sinks
            .iter()
            .filter_map(|&s| linhas_do_sink(&state.doc.graph, &state.registry, s, INSTANTE))
            .sum();
        let todos_gpu = sinks.iter().all(|&s| {
            ph2d_gpu_cook::plan(&state.doc.graph, &state.registry, &state.registry, s)
                .is_fully_gpu()
        });
        linhas.push((level, sinks.len(), total, todos_gpu));
    }

    linhas.sort_by_key(|&(_, _, n, _)| std::cmp::Reverse(n));
    let n_cenas = linhas.len();
    let soma: usize = linhas.iter().map(|&(_, _, n, _)| n).sum();
    let acima: Vec<_> = linhas.iter().filter(|&&(_, _, n, _)| n >= JOELHO).collect();
    let prontas: Vec<_> = linhas.iter().filter(|&&(_, _, _, gpu)| gpu).collect();
    let prontas_acima: Vec<_> = prontas
        .iter()
        .filter(|&&&(_, _, n, _)| n >= JOELHO)
        .collect();

    eprintln!("\n=== A ESCADA DO MULTI-SINK · {n_cenas} cenas, cozidas em t = {INSTANTE} ===");
    eprintln!("  joelho de referencia (PAR_THRESHOLD do cozimento): {JOELHO} linhas\n");
    eprintln!(
        "  {:>5} │ {:>5} │ {:>9} │ todos os sinks ja seriam device",
        "cena", "sinks", "linhas"
    );
    for &(level, n_sinks, n, gpu) in linhas.iter().take(15) {
        eprintln!(
            "  {level:>5} │ {n_sinks:>5} │ {n:>9} │ {}",
            if gpu { "sim" } else { "nao" }
        );
    }
    if n_cenas > 15 {
        eprintln!(
            "  … mais {} cenas, todas abaixo de {} linhas",
            n_cenas - 15,
            linhas[14].2
        );
    }
    eprintln!("\n  ───────────────────────────────────────────────────────────");
    eprintln!("  linhas somadas em TODAS as {n_cenas} cenas multi-sink : {soma}");
    eprintln!(
        "  a MAIOR cena multi-sink                          : {} linhas (cena {})",
        linhas.first().map_or(0, |&(_, _, n, _)| n),
        linhas.first().map_or(0, |&(l, _, _, _)| l)
    );
    eprintln!(
        "  cenas ACIMA do joelho ({JOELHO})                      : {}",
        acima.len()
    );
    eprintln!(
        "  cenas em que TODOS os sinks ja seriam device      : {}",
        prontas.len()
    );
    eprintln!(
        "  … dessas, ACIMA do joelho                        : {}",
        prontas_acima.len()
    );
    eprintln!("  ───────────────────────────────────────────────────────────");
    eprintln!(
        "  ⇒ a escada e' {} \n",
        if prontas_acima.is_empty() {
            "uma ponte sobre uma POCA nas cenas de demo: nenhuma cena pronta tem trabalho que o \
             dispositivo amortize. O que ela pode comprar vive no DOCUMENTO do artista, nao aqui."
        } else {
            "sobre um POCO: ha cenas prontas com trabalho acima do joelho medido."
        }
    );
}
