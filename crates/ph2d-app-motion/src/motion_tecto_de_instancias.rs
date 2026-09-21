//! ⭐⭐⭐ **O TECTO DE INSTÂNCIAS POR NÓ** — ordem do dono, 2026-09-21: *«coloque um limite em todos
//! os nós que são usados para criar instâncias: nenhum deles pode gerar mais de 16384 objetos»*.
//!
//! ⚠️⚠️ **A POPULAÇÃO É MEDIDA, nunca uma lista escrita à mão.** *«Todos os nós que criam
//! instâncias»* é uma pergunta sobre COMPORTAMENTO, e a única resposta que não apodrece no primeiro
//! nó novo é cozinhar cada um e CONTAR o que ele emite. Uma lista de nomes ficaria muda exactamente
//! como o censo por prefixo de ficheiro desta casa já ficou (`CLAUDE.md` §5.0).
//!
//! ⛔ E ela é medida com os params no TECTO DECLARADO de cada um (o `ParamHardMax` se houver, senão
//! o `max` do `ParamUiHint`): a pergunta não é *«quanto ele emite de fábrica»*, é **«quanto ele
//! PODE emitir com o que o artista consegue escrever»**.

use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O tecto que o dono ordenou, lido da porta que o declara.
const TECTO: usize = ph2d_nodegraph::node::MAX_INSTANCIAS_POR_NO;

/// ⚠️⚠️ **O VALOR com que a sonda cozinha cada param, e porque ele é PEQUENO.**
///
/// ⛔⛔ **Duas corridas desta sonda tiveram de ser mortas** antes de este número existir. A 1.ª
/// cozinhava no tecto DECLARADO (o `motion.grid` declara `1 000 000` por eixo) e ficou `10` minutos
/// a `1 500 %` de CPU; a 2.ª, já em `--release` e limitada a `4 ×` o alvo, ficou outros `8`. ⭐ E a
/// causa não era a CONTAGEM: é que *«todo param no tecto»* põe também os params de **ITERAÇÃO** no
/// tecto — um solver a `65 536` sub-passos, um enxame a `65 536` vizinhos (`O(n²)`). *Pôr tudo no
/// máximo mede o pior caso de TODA grandeza, e só uma delas é a pergunta.*
///
/// ⇒ a sonda pergunta outra coisa, e barata: **este nó emite MAIS LINHAS DO QUE RECEBE?** Essa é a
/// definição de *«cria instâncias»*, ela é visível com valores modestos, e o TECTO é depois uma
/// propriedade do que o nó DECLARA — não do que ele consegue cozinhar numa sonda.
const VALOR_DA_SONDA: f32 = 12.0;

/// Quantas linhas a sonda alimenta a quem recebe instâncias.
const LINHAS_DE_ENTRADA: usize = 3;

/// O tecto de um param, como o artista o alcança: o digitável se houver, senão o topo do slider.
fn tecto_do_param(
    reg: &ph2d_node_registry::NodeRegistry,
    id: ph2d_nodegraph::node::NodeTypeId,
    param: &str,
) -> Option<f32> {
    if let Some(m) = reg.param_hard_max(id, param) {
        return Some(m);
    }
    reg.param_ui(id)?
        .iter()
        .find(|h| h.param == param)
        .map(|h| h.max)
}

/// Cozinha `no` no grafo e devolve `(linhas emitidas, entrou em pânico)`.
///
/// ⛔⛔ **A rede existe porque a 1.ª corrida deste censo MORREU no primeiro nó** — o `value.noise`
/// com os params no tecto declarado dava `attempt to add with overflow` (um `f32 → i32` satura, e
/// somar `1` ao saturado estoura). *Um censo que morre no primeiro acusado não mede os outros
/// 133*, e a lista dos que estouram é ela própria um achado que vale a corrida.
fn linhas(g: &Graph, reg: &ph2d_node_registry::NodeRegistry, no: NodeId) -> (usize, bool) {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut cook = Cook::new();
        cook.cook(g, reg, no, 0.0)
            .ok()
            .and_then(|out| out.first().map(|v| v.as_stream().count()))
            .unwrap_or(0)
    }));
    match r {
        Ok(n) => (n, false),
        Err(_) => (0, true),
    }
}

/// Uma grelha de [`LINHAS_DE_ENTRADA`] linhas, para alimentar quem multiplica.
///
/// ⚠️ **Três e não uma:** um nó que MULTIPLICA por um factor lê-se igual a um que passa adiante
/// quando a entrada tem uma linha só (`1 × k` contra `1`), e a régua é `saida > entrada`.
fn alimenta(g: &mut Graph) -> NodeId {
    let f = g.add_node("motion.grid".to_string());
    g.set_param(f, "rows", 1.0);
    #[expect(clippy::cast_precision_loss, reason = "três linhas")]
    g.set_param(f, "cols", LINHAS_DE_ENTRADA as f32);
    f
}

/// ⭐⭐⭐ **QUANTAS LINHAS CADA NÓ PODE EMITIR** — a medição que decide onde o tecto entra.
#[test]
#[ignore = "sonda de medição — corra à mão"]
fn censo_de_quem_cria_instancias() {
    let m = crate::motion_state::MotionState::new();
    let reg = &m.registry;
    let tipos: Vec<(String, ph2d_nodegraph::node::NodeTypeId, Vec<String>, usize)> = reg
        .manifests()
        .map(|man| {
            (
                man.name.to_string(),
                man.id,
                man.params.iter().map(|p| p.name.to_string()).collect(),
                man.inputs.len(),
            )
        })
        .collect();
    drop(m);

    let mut acusados: Vec<(String, usize)> = Vec::new();
    let mut estouram: Vec<String> = Vec::new();
    let mut medidos = 0usize;
    // ⚠️ O hook de pânico é silenciado: com 134 nós a serem cozinhados no tecto, o `stderr` de cada
    // um afogaria a tabela que a sonda existe para imprimir.
    let anterior = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let m = crate::motion_state::MotionState::new();
    let reg = &m.registry;
    for (nome, id, params, n_entradas) in &tipos {
        // Três arranjos: sozinho · com UMA linha na porta 0 · com UMA linha em todas as portas.
        let mut melhor = 0usize;
        let mut panicos = false;
        for arranjo in 0..3usize {
            let mut g = Graph::new();
            let n = g.add_node(nome.clone());
            match arranjo {
                1 if *n_entradas > 0 => {
                    let f = alimenta(&mut g);
                    let _ = g.connect(Edge {
                        from: (f, 0),
                        to: (n, 0),
                        delayed: false,
                    });
                }
                2 => {
                    for p in 0..(*n_entradas as u16) {
                        let f = alimenta(&mut g);
                        let _ = g.connect(Edge {
                            from: (f, 0),
                            to: (n, p),
                            delayed: false,
                        });
                    }
                }
                _ => {}
            }
            // Todos os params no tecto declarado.
            for p in params {
                if tecto_do_param(reg, *id, p).is_some() {
                    g.set_param(n, p.as_str(), VALOR_DA_SONDA);
                }
            }
            let (n_linhas, morreu) = linhas(&g, reg, n);
            melhor = melhor.max(n_linhas);
            panicos |= morreu;
        }
        medidos += 1;
        if panicos {
            estouram.push(nome.clone());
        }
        // ⭐ A régua: ele emite MAIS do que recebe? (`0` entradas ⇒ tudo o que ele emite é novo.)
        let entrada = if *n_entradas > 0 {
            LINHAS_DE_ENTRADA
        } else {
            0
        };
        if melhor > entrada {
            acusados.push((nome.clone(), melhor));
        }
    }
    // ⭐⭐⭐ **E QUAIS PARAMS COMANDAM A CONTAGEM** — a metade que decide ONDE o tecto entra.
    //
    // ⚠️ *«Emite mais do que recebe»* apanha duas famílias diferentes: quem MULTIPLICA por um
    // número que o artista escreve (o `motion.grid`, o `motion.clone`) e quem só JUNTA o que já
    // lhe chega (o `motion.combine`, cuja saída é a soma das entradas — e cada uma delas já veio
    // de um nó capado). *Capar o segundo seria capar a composição, não o gerador.*
    //
    // ⇒ a régua é a mesma do censo dos knobs da escultura: varia-se UM param e vê-se se a
    // CONTAGEM muda.
    let mut comandam: Vec<(String, Vec<String>)> = Vec::new();
    for (nome, id, params, n_entradas) in &tipos {
        if !acusados.iter().any(|(a, _)| a == nome) {
            continue;
        }
        let mut quais: Vec<String> = Vec::new();
        for alvo in params {
            let conta = |v: f32| -> usize {
                let mut g = Graph::new();
                let n = g.add_node(nome.clone());
                for pp in 0..(*n_entradas as u16) {
                    let f = alimenta(&mut g);
                    let _ = g.connect(Edge {
                        from: (f, 0),
                        to: (n, pp),
                        delayed: false,
                    });
                }
                for p in params {
                    if tecto_do_param(reg, *id, p).is_some() {
                        g.set_param(n, p.as_str(), VALOR_DA_SONDA);
                    }
                }
                g.set_param(n, alvo.as_str(), v);
                linhas(&g, reg, n).0
            };
            if conta(VALOR_DA_SONDA) != conta(VALOR_DA_SONDA * 2.0) {
                quais.push(alvo.clone());
            }
        }
        if !quais.is_empty() {
            comandam.push((nome.clone(), quais));
        }
    }

    acusados.sort_by_key(|(_, n)| std::cmp::Reverse(*n));

    eprintln!(
        "\n  ═══ QUEM CRIA INSTÂNCIAS (emite mais do que recebe; params a {VALOR_DA_SONDA}) ═══\n"
    );
    eprintln!(
        "  nós medidos: {medidos}   ·   acusados: {}\n",
        acusados.len()
    );
    eprintln!("   linhas emitidas |  nó");
    eprintln!("  -----------------|------------------------------");
    for (nome, n) in &acusados {
        eprintln!("  {n:>16} |  {nome}");
    }
    eprintln!(
        "\n  ═══ E QUAIS PARAMS COMANDAM A CONTAGEM ({} nós) ═══\n",
        comandam.len()
    );
    for (nome, quais) in &comandam {
        let tectos: Vec<String> = quais
            .iter()
            .map(|q| {
                tecto_do_param(
                    reg,
                    tipos.iter().find(|(n, ..)| n == nome).expect("achado").1,
                    q,
                )
                .map_or_else(|| format!("{q}=?"), |t| format!("{q}<={t:.0}"))
            })
            .collect();
        eprintln!("  {nome:<28} {}", tectos.join("  "));
    }
    eprintln!(
        "\n  ⚠️ O tecto que o dono ordenou é {TECTO} objectos por nó; os tectos acima são os que\n  \
         cada nó DECLARA hoje, e a diferença entre os dois é o que a ordem custa."
    );
    eprintln!(
        "\n  ⛔ E os que ESTOURAM com os params no tecto declarado ({}):",
        estouram.len()
    );
    for nome in &estouram {
        eprintln!("       {nome}");
    }
    eprintln!();
    std::panic::set_hook(anterior);
}

/// ⭐⭐⭐ **O CENSO NÃO PODE FICAR MUDO** — o gate que corre sempre, ao lado da sonda que se corre
/// à mão.
///
/// ⚠️ Ele não afirma o tecto (isso é a sonda, e a decisão é do dono): afirma que a POPULAÇÃO
/// existe. *Um censo que passe a medir zero nós lê-se exactamente como «ninguém cria instâncias»*,
/// que é a forma de falha muda que o `CLAUDE.md` §5.0 nomeia — e a cura dele é um piso.
#[test]
fn o_censo_de_quem_cria_instancias_ve_a_familia() {
    let m = crate::motion_state::MotionState::new();
    let reg = &m.registry;
    let tipos: Vec<(String, ph2d_nodegraph::node::NodeTypeId, usize)> = reg
        .manifests()
        .map(|man| (man.name.to_string(), man.id, man.inputs.len()))
        .collect();
    let mut cria = 0usize;
    let mut grid_visto = false;
    for (nome, _id, n_entradas) in &tipos {
        let mut g = Graph::new();
        let n = g.add_node(nome.clone());
        for p in 0..(*n_entradas as u16) {
            let f = alimenta(&mut g);
            let _ = g.connect(Edge {
                from: (f, 0),
                to: (n, p),
                delayed: false,
            });
        }
        let entrada = if *n_entradas > 0 {
            LINHAS_DE_ENTRADA
        } else {
            0
        };
        if linhas(&g, reg, n).0 > entrada {
            cria += 1;
            grid_visto |= nome == "motion.grid";
        }
    }
    assert!(
        cria >= 20,
        "o censo tem de ver a familia inteira, e viu {cria} nos de {}",
        tipos.len()
    );
    assert!(
        grid_visto,
        "controlo: o `motion.grid` e' o exemplo que o dono nomeou — se ele nao aparece, a regua \
         mede outra coisa"
    );
}
