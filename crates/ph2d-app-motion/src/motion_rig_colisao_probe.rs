//! ⭐⭐⭐ **A COLISÃO NO GRUPO DO CICLO 9 — o report do dono, medido** (doc 114 §11).
//!
//! Report (2026-09-17): *«Verlet rope com Shape e Collision ON não reconhece colisões entre as
//! próprias células. Talvez todos do grupo não aceitem colisão.»*
//!
//! ⚠️ **São DUAS perguntas e elas têm respostas diferentes**, e esta sonda separa-as porque
//! respondê-las juntas dá a resposta errada a uma delas:
//!
//! 1. **A corda colide consigo mesma?** (uma volta do laço atravessa a outra) — *não*, e é
//!    ESTRUTURAL: a relaxação dela tem exactamente duas restrições, `i↔i+1` (distância) e
//!    `i↔i+2` (flexão), e **nenhuma** `i↔j` para pares afastados.
//! 2. **O `Collide` do `source.shape` chega à corda?** — *não pode*, e é estrutural de outra
//!    maneira: o `source.shape` declara o colisor em COLUNAS, e quem as lê é a família `sim.*`
//!    (`sim.collide` · `sim.step` · `ph2d-contact`). A corda não tem porta por onde elas
//!    entrem — as três dela são `anchor_x`, `anchor_y` e `state`.
//!
//! ⭐⭐ **E há uma TERCEIRA pergunta, que é a que interessa ao produto:** *a composição já
//! exprime isto?* O `motion.collide` é um separador de discos a sério (PBD não-penetração), é
//! `Effect::Pure` e aceita qualquer nuvem — inclusive a da corda. Esta sonda mede se pô-lo a
//! jusante da corda **e no laço de estado dela** entrega auto-colisão, e a que preço.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib colisao_probe -- --ignored --nocapture
//! ```

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Pontos da corda medida. ⚠️ **Ímpar de propósito:** com um número par o ponto do meio do
/// chicote cai exactamente sobre um vizinho e a leitura do mínimo fica sobre um par adjacente,
/// que é o par que a restrição de distância governa — e não o que a pergunta é sobre.
const PONTOS: f32 = 25.0;
/// O comprimento da corda, em unidades de mundo.
const COMPRIMENTO: f32 = 4.0;
/// A gravidade e o quanto a âncora chicoteia. ⚠️ **O chicote é o que faz a corda DOBRAR sobre si
/// mesma** — uma corda pendurada em repouso é uma catenária e nunca se toca, logo medi-la em paz
/// responderia *«não há sobreposição»* sobre um arranjo que não a pode ter.
const GRAVIDADE: f32 = 9.0;
const CHICOTE_AMPLITUDE: f32 = 2.2;
const CHICOTE_PERIODO: f32 = 0.45;

/// O raio de disco com que se pergunta pela sobreposição — e com que o `motion.collide` separa.
///
/// ⚠️ **Ele é uma FRACÇÃO do segmento, não um número solto:** o segmento mede
/// `COMPRIMENTO/(PONTOS−1) ≈ 0,167`, e um raio maior que metade disso faria vizinhos ADJACENTES
/// violarem o disco por construção — a restrição de distância mantém-nos a um segmento, e a
/// pergunta ficaria sobre um par que a corda nunca teve intenção de separar.
const RAIO: f32 = 0.07;

const TIQUES: usize = 240;
const DT: f64 = 1.0 / 60.0;

/// Quantos índices de distância contam como VIZINHOS e por isso saem da conta.
///
/// ⚠️ **`2` e não `1`, e o número é o do SOLVER:** a corda governa `i↔i+1` pela distância e
/// `i↔i+2` pela flexão. Perguntar pela sobreposição de um par que uma restrição já posiciona
/// mediria a restrição, não a colisão.
const VIZINHOS: usize = 2;

fn no(g: &mut Graph, tipo: &str, x: f32, y: f32) -> NodeId {
    let n = g.add_node(tipo.to_string());
    g.set_pos(n, Pos { x, y });
    n
}

fn liga(g: &mut Graph, de: NodeId, para: (NodeId, u16), atrasado: bool) {
    g.connect(Edge {
        from: (de, 0),
        to: para,
        delayed: atrasado,
    })
    .expect("fio");
}

/// O que se põe entre a corda e a saída — e, quando `no_laco`, também dentro do laço de estado.
#[derive(Clone, Copy, PartialEq)]
enum Arranjo {
    /// A corda nua: `rope → output`, com o estado a voltar da PRÓPRIA corda.
    Nua,
    /// `rope → collide → output`, com o estado a voltar da CORDA. O separador é um acabamento.
    CollideDepois,
    /// `rope → collide → output`, com o estado a voltar do COLLIDE. O separador entra no laço.
    CollideNoLaco,
}

/// Monta a corda chicoteada e devolve `(grafo, sink)`. `iteracoes` só é lido pelos arranjos que
/// têm `motion.collide`.
fn monta_com(arranjo: Arranjo, iteracoes: f32) -> (Graph, NodeId) {
    let (mut g, sink) = monta(arranjo);
    for n in g.nodes().iter().map(|x| x.id).collect::<Vec<_>>() {
        if g.node(n).is_some_and(|i| i.type_name == "motion.collide") {
            g.set_param(n, "iterations", iteracoes);
        }
    }
    (g, sink)
}

/// Monta a corda chicoteada e devolve `(grafo, sink)`.
fn monta(arranjo: Arranjo) -> (Graph, NodeId) {
    let mut g = Graph::default();
    let corda = no(&mut g, "motion.verlet_rope", 0.0, 0.0);
    g.set_param(corda, "count", PONTOS);
    g.set_param(corda, "length", COMPRIMENTO);
    g.set_param(corda, "gravity", GRAVIDADE);

    // O CHICOTE: a âncora varre de um lado ao outro depressa, e a corda dobra sobre si mesma.
    let lfo = no(&mut g, "value.lfo", -200.0, 0.0);
    g.set_param(lfo, "period", CHICOTE_PERIODO);
    g.set_param(lfo, "amplitude", CHICOTE_AMPLITUDE);
    liga(&mut g, lfo, (corda, 0), false);

    let saida_do_solver = match arranjo {
        Arranjo::Nua => corda,
        Arranjo::CollideDepois | Arranjo::CollideNoLaco => {
            let c = no(&mut g, "motion.collide", 200.0, 0.0);
            g.set_param(c, "radius", RAIO);
            g.set_param(c, "iterations", 8.0);
            g.set_param(c, "strength", 1.0);
            liga(&mut g, corda, (c, 0), false);
            c
        }
    };

    // ⚠️ **A aresta de estado é ATRASADA** — sem isso o grafo tem ciclo e o cozedor recusa-o.
    let volta = if arranjo == Arranjo::CollideNoLaco {
        saida_do_solver
    } else {
        corda
    };
    liga(&mut g, volta, (corda, 2), true);

    let out = no(&mut g, "motion.output", 400.0, 0.0);
    liga(&mut g, saida_do_solver, (out, 0), false);
    (g, out)
}

/// A folga com que um par que TOCA deixa de contar como sobreposto.
///
/// ⛔⛔ **Sem ela a régua acusa a própria CONVERGÊNCIA.** Um separador de não-penetração faz os
/// discos ficarem *a tocar* — o repouso dele é exactamente `2·RAIO` —, e em `f32` isso pousa em
/// `0,1399` sobre uma barra de `0,1400`. A 1.ª redacção desta sonda contava `d < 2·RAIO` estrito e
/// imprimia **7 pares sobrepostos** ao lado de *«100 % da barra»*: duas colunas da mesma medição a
/// contradizerem-se, e a leitura errada («não funciona») é a que se acredita.
///
/// ⚠️ **`2 %` não é um epsilon de vírgula flutuante** — é *«a penetração é visível?»*. Sobre um
/// disco de `0,07` são `0,0028` de mundo, e o que a corda nua faz é penetrar `0,066`.
const TOLERANCIA: f32 = 0.02;

/// A menor distância entre dois pontos que NÃO são vizinhos no solver, e quantos pares se
/// sobrepõem de facto (ver [`TOLERANCIA`]).
fn sobreposicao(p: &[[f32; 2]]) -> (f32, usize) {
    let barra = 2.0 * RAIO * (1.0 - TOLERANCIA);
    let mut menor = f32::MAX;
    let mut pares = 0usize;
    for i in 0..p.len() {
        for j in (i + VIZINHOS + 1)..p.len() {
            let d = ((p[i][0] - p[j][0]).powi(2) + (p[i][1] - p[j][1]).powi(2)).sqrt();
            menor = menor.min(d);
            if d < barra {
                pares += 1;
            }
        }
    }
    (menor, pares)
}

fn corre(m: &MotionState, arranjo: Arranjo) -> (f32, usize, usize) {
    corre_com(m, arranjo, 8.0)
}

fn corre_com(m: &MotionState, arranjo: Arranjo, iteracoes: f32) -> (f32, usize, usize) {
    let (g, sink) = monta_com(arranjo, iteracoes);
    let mut cook = Cook::new();
    let mut t = 0.0f64;
    let mut pior = f32::MAX;
    let mut pior_pares = 0usize;
    let mut n = 0usize;
    for _ in 0..TIQUES {
        if let Ok(v) = cook.cook(&g, &m.registry, sink, t) {
            let s = v[0].as_stream();
            if let Some(Column::Vec2(p)) = s.get("P") {
                n = p.len();
                let (menor, pares) = sobreposicao(p);
                // ⚠️ **O PIOR instante, não o último.** Uma corda chicoteada sobrepõe-se a meio do
                // golpe e volta a abrir — ler só o quadro final mediria o repouso dela.
                if menor < pior {
                    pior = menor;
                }
                pior_pares = pior_pares.max(pares);
            }
        }
        let _ = cook.advance_tick(&g, &m.registry, t);
        t += DT;
    }
    (pior, pior_pares, n)
}

/// A mediana do relógio de um quadro em regime, em milissegundos.
///
/// ⚠️ **Mediana de nove depois de aquecer**, a mesma forma do relógio do grupo (doc 114 §7): a
/// corda tem estado e um pico de escalonador não pode decidir a tabela.
fn relogio(m: &MotionState, arranjo: Arranjo, iteracoes: f32, pontos: f32) -> f64 {
    let (mut g, sink) = monta_com(arranjo, iteracoes);
    for n in g.nodes().iter().map(|x| x.id).collect::<Vec<_>>() {
        if g.node(n)
            .is_some_and(|i| i.type_name == "motion.verlet_rope")
        {
            g.set_param(n, "count", pontos);
        }
    }
    let mut cook = Cook::new();
    let mut t = 0.0f64;
    for _ in 0..60 {
        let _ = cook.cook(&g, &m.registry, sink, t);
        let _ = cook.advance_tick(&g, &m.registry, t);
        t += DT;
    }
    let mut ms = Vec::with_capacity(9);
    for _ in 0..9 {
        let agora = std::time::Instant::now();
        let _ = cook.cook(&g, &m.registry, sink, t);
        ms.push(agora.elapsed().as_secs_f64() * 1e3);
        let _ = cook.advance_tick(&g, &m.registry, t);
        t += DT;
    }
    ms.sort_by(f64::total_cmp);
    ms[ms.len() / 2]
}

/// ⭐⭐⭐ **O RETRATO que responde ao report.**
#[test]
#[ignore = "sonda de investigação — corra à mão"]
fn colisao_probe_o_que_o_grupo_faz_com_um_colisor() {
    let m = MotionState::new();

    eprintln!("\n  ═══ 1 · A CORDA COLIDE CONSIGO MESMA? ═══\n");
    eprintln!(
        "  Uma corda de {PONTOS:.0} pontos, chicoteada pela âncora, {TIQUES} tiques.\n  \
         Discos de raio {RAIO} ⇒ dois pontos sobrepõem-se abaixo de {:.3}.\n",
        2.0 * RAIO
    );
    eprintln!(
        "  {:<26} │ {:>12} │ {:>14}",
        "arranjo", "menor vão", "pares sobrepostos"
    );
    eprintln!("  ---------------------------|--------------|---------------");
    for (a, nome) in [
        (Arranjo::Nua, "a corda NUA"),
        (Arranjo::CollideDepois, "+ collide DEPOIS"),
        (Arranjo::CollideNoLaco, "+ collide NO LAÇO"),
    ] {
        let (menor, pares, n) = corre(&m, a);
        assert_eq!(n, PONTOS as usize, "a corda tem de emitir {PONTOS} pontos");
        eprintln!("  {nome:<26} │ {menor:>12.4} │ {pares:>14}");
    }

    eprintln!("\n  ═══ 1-bis · ATÉ ONDE A COMPOSIÇÃO CHEGA — E AS DUAS MORADAS LADO A LADO ═══\n");
    eprintln!(
        "  A barra é {:.3}: abaixo dela dois discos estão um dentro do outro.\n\n  \
         ⚠️ As DUAS colunas são a W0 do doc 115 — a pergunta de onde o passe automático corre.\n  \
         DEPOIS = o separador é um acabamento, e o estado do solver volta da CORDA (a morada\n  \
         barata, e a que encaixa em «toda visualização passa pelo Duplicador»).\n  \
         NO LAÇO = a saída separada é REALIMENTADA no `rope.state`, que é o que existe hoje.\n",
        2.0 * RAIO
    );
    eprintln!(
        "  {:<12} │ {:>21} │ {:>21}",
        "iterações", "DEPOIS  (vão / pares)", "NO LAÇO (vão / pares)"
    );
    eprintln!("  -------------|-----------------------|----------------------");
    for it in [8.0f32, 16.0, 32.0, 64.0, 128.0] {
        let (md, pd, _) = corre_com(&m, Arranjo::CollideDepois, it);
        let (ml, pl, _) = corre_com(&m, Arranjo::CollideNoLaco, it);
        eprintln!("  {it:<12.0} │ {md:>13.4} / {pd:>5} │ {ml:>13.4} / {pl:>5}");
    }

    eprintln!("\n  ═══ 1-ter · O QUE A COMPOSIÇÃO CUSTA ═══\n");
    eprintln!(
        "  O `motion.collide` é `O(n²·iterações)` no caminho da CPU. Um quadro tem 16,67 ms.\n"
    );
    eprintln!(
        "  {:<10} │ {:>12} │ {:>12} │ {:>12}",
        "pontos", "corda só", "+ collide 32", "% de um quadro"
    );
    eprintln!("  -----------|--------------|--------------|-------------");
    for pontos in [25.0f32, 50.0, 100.0, 200.0] {
        let nua = relogio(&m, Arranjo::Nua, 8.0, pontos);
        let com = relogio(&m, Arranjo::CollideNoLaco, 32.0, pontos);
        eprintln!(
            "  {pontos:<10.0} │ {nua:>9.3} ms │ {com:>9.3} ms │ {:>11.1}%",
            com / 16.67 * 100.0
        );
    }

    eprintln!("\n  ═══ 2 · QUE PORTAS TEM CADA NÓ DO GRUPO ═══\n");
    eprintln!(
        "  O `source.shape` declara o colisor em COLUNAS. Para ele chegar a um solver, o solver\n  \
         precisa de uma PORTA por onde a corrente entre. Contado do manifesto:\n"
    );
    eprintln!("  {:<24} │ {:>7} │ portas de entrada", "nó", "entradas");
    eprintln!("  -------------------------|---------|------------------");
    for tipo in [
        "motion.verlet_rope",
        "motion.wave",
        "motion.soft_body",
        "motion.boids",
        "rig.skeleton",
        "rig.skin_deformer",
        "sim.collide",
    ] {
        let Some(man) = m.registry.manifests().find(|x| x.name == tipo) else {
            eprintln!("  {tipo:<24} │       ? │ (não registado)");
            continue;
        };
        let nomes: Vec<&str> = man.inputs.iter().map(|p| p.name).collect();
        eprintln!(
            "  {tipo:<24} │ {:>7} │ {}",
            man.inputs.len(),
            nomes.join(" · ")
        );
    }
    eprintln!();
}
