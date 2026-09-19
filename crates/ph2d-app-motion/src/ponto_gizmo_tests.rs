//! Os portões do [`super::ponto_gizmo`] — a ordem do dono de 2026-09-17/19.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};

/// Coze pela porta do PRODUTO (a mesma que o quadro chama), e é ela que enche as tomadas.
fn coze(m: &mut MotionState, sinks: &[ph2d_nodegraph::graph::NodeId]) {
    let graph = m.doc.graph.clone();
    let MotionState { pump, registry, .. } = m;
    pump.advance_or_scrub_scoped(
        &graph,
        registry,
        sinks,
        0,
        |t| t as f64 / 60.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &ph2d_nodegraph::cook::TimeScopes::new(),
    );
}

fn nuvem(n: usize) -> Stream {
    Stream::new(n).with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
}

/// **A FEIÇÃO SAI DAS COLUNAS** — as três respostas, derivadas e não de uma lista de nomes de nó.
#[test]
fn a_feicao_sai_das_colunas() {
    assert_eq!(feicao_de(&nuvem(3)), Feicao::Ponto);
    assert_eq!(
        feicao_de(&nuvem(3).with("parent", Column::Scalar(vec![-1.0, 0.0, 1.0]))),
        Feicao::Osso
    );
    assert_eq!(
        feicao_de(&nuvem(3).with("rope_prev", Column::Vec2(vec![[0.0, 0.0]; 3]))),
        Feicao::Corda
    );
}

/// ⚠️ **E o `parent` GANHA do `rope_prev`** — ver o cabeçalho: pender de alguém é mais forte do que
/// ser consecutivo, e uma cadeia com ramos desenhada como corda liga pontos que não se tocam.
#[test]
fn o_parent_ganha_do_rope_prev() {
    let s = nuvem(3)
        .with("parent", Column::Scalar(vec![-1.0, 0.0, 1.0]))
        .with("rope_prev", Column::Vec2(vec![[0.0, 0.0]; 3]));
    assert_eq!(feicao_de(&s), Feicao::Osso);
}

/// **A RAIZ NÃO DESENHA UM OSSO, e um `parent` fora de alcance também não.**
///
/// ⚠️ A raiz declara-se com `-1`, e `as usize` sobre um negativo **satura em `0`** em Rust ⇒ sem a
/// comparação com `0.0` ANTES do cast, toda raiz desenharia um osso de si para si.
/// ⛔⛔ **A FIXTURA TEM DE TER UMA RAIZ FORA DO ÍNDICE `0`, e isto foi uma MUTAÇÃO SOBREVIVENTE.**
///
/// Com a raiz só em `0`, apagar a comparação `p < 0.0` **não é observável**: `-1.0 as usize` satura
/// em `0` e a cerca seguinte (`pi != i`) rejeita o `[0, 0]` na mesma. *As duas cercas só são
/// distinguíveis onde a saturação NÃO aterra no próprio elemento* — uma raiz em `2` com `parent =
/// -1` desenharia um osso **de `0` para `2`**, que é uma linha que o artista não autorou, e um
/// `parent = -3` faz o mesmo em qualquer índice.
#[test]
fn a_raiz_e_o_pai_fora_de_alcance_nao_desenham_osso() {
    // Duas cadeias: `0 → 1`, e uma raiz NOVA em `2` (`-1`), mais um pai fora de alcance em `4`.
    let s = nuvem(5).with("parent", Column::Scalar(vec![-1.0, 0.0, -1.0, -3.0, 99.0]));
    let segs = super::ossos(&s, 5);
    assert_eq!(
        segs,
        vec![[0, 1]],
        "so' o filho real desenha: as duas raizes e os dois pais invalidos saem"
    );
}

/// **UMA CORDA DE `n` PONTOS TEM `n − 1` SEGMENTOS**, e uma de um ponto não tem nenhum.
#[test]
fn a_corda_liga_os_consecutivos() {
    assert_eq!(super::corda(4), vec![[0, 1], [1, 2], [2, 3]]);
    assert!(super::corda(1).is_empty());
    assert!(super::corda(0).is_empty());
}

/// ⭐⭐⭐ **A ROTA INTEIRA: cozer → tomada → retrato.**
///
/// ⚠️ **Ele monta a cena pela porta do PRODUTO** (`motion_bridge::dispatch` não é alcançável de um
/// teste, mas a bomba é): um gate que construísse o `PontoGizmoView` à mão afirmaria que a
/// geometria existe, **nunca que o quadro a produz** — a forma que esta casa já pagou quatro vezes.
#[test]
fn uma_grelha_sem_forma_vira_gizmo_de_pontos() {
    let mut m = MotionState::new();
    let sinks = crate::motion_demo_legend::monta("117", &mut m.doc, &m.registry).0;
    assert!(!sinks.is_empty(), "a fixtura tem de montar sinks");
    m.sinks = sinks.clone();

    // As tomadas, pela MESMA porta que o quadro usa.
    let taps = taps_for(&m);
    assert_eq!(taps, sinks, "o gizmo pede todos os sinks");
    m.pump.set_taps(&taps);

    // ⚠️ **Sem cozer não há retrato**, e é essa a metade que prova que ele lê o COZIDO.
    assert!(
        resolve(&m, true).is_none(),
        "montada e nao cozida: sem tomadas, sem gizmo"
    );

    coze(&mut m, &sinks);

    let v = resolve(&m, true).expect("a cena so'-posicoes tem de dar gizmo");
    assert!(!v.grupos.is_empty());
    for g in &v.grupos {
        assert_eq!(g.feicao, Feicao::Ponto, "uma grelha e' uma nuvem");
        assert!(!g.pontos.is_empty());
        assert!(g.segmentos.is_empty(), "uma nuvem nao tem segmentos");
        assert!(g.total >= g.pontos.len());
    }
}

/// ⭐⭐⭐ **E SEM A FERRAMENTA MOTION NA MÃO NÃO HÁ GIZMO** — *«…que não renderiza em runtime»*.
///
/// ⚠️ **A metade POSITIVA está no gate acima, e sem ela esta seria vácua:** uma `resolve` que
/// devolvesse sempre `None` passaria aqui.
#[test]
fn sem_a_ferramenta_motion_nao_ha_gizmo() {
    let mut m = MotionState::new();
    let sinks = crate::motion_demo_legend::monta("117", &mut m.doc, &m.registry).0;
    m.sinks = sinks.clone();
    m.pump.set_taps(&taps_for(&m));
    coze(&mut m, &sinks);
    assert!(
        resolve(&m, true).is_some(),
        "o CONTROLO: com a ferramenta ha'"
    );
    assert!(resolve(&m, false).is_none(), "sem ela, nada");
}

/// ⭐⭐ **UMA CORRENTE COM APARÊNCIA NÃO TEM GIZMO** — a mesma porta que o lowering usa.
///
/// ⚠️ Sem isto, uma cena com formas ganharia uma cruz em cima de cada peça desenhada.
/// ⛔⛔ **E ELE PERCORRE A ROTA — isto foi a SEGUNDA mutação sobrevivente.** A 1.ª redacção
/// perguntava a `tem_aparencia` directamente, e apagar o `continue` do `resolve` deixava-a VERDE:
/// *um gate que chama a porta afirma que ela responde bem, nunca que o produto a consulta.*
#[test]
fn quem_tem_aparencia_nao_ganha_gizmo() {
    use ph2d_nodegraph::graph::Graph;
    let mut m = MotionState::new();
    // Um grafo mínimo do PRODUTO: um objecto nomeado a entrar no sink.
    let mut g = Graph::new();
    let obj = g.add_node("source.object");
    let saida = g.add_node("motion.output");
    g.set_text_param(obj, "object", "Bola");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (obj, 0),
        to: (saida, 0),
        delayed: false,
    })
    .expect("liga");
    m.doc.graph = g;
    m.sinks = vec![saida];
    // A membrana publica a aparência, pela MESMA porta do shell.
    crate::motion_lsystem_testkit::publish_object_alpha(&mut m, "Bola", 0, false);
    m.pump.set_taps(&taps_for(&m));
    let sinks = m.sinks.clone();
    coze(&mut m, &sinks);

    // O CONTROLO: a corrente do sink de facto TEM aparência — senão isto media outro programa.
    let cozida = m
        .pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == saida)
        .map(|(_, s)| s.clone())
        .expect("a tomada do sink tem de estar cozida");
    assert!(
        ph2d_eval_motion::tem_aparencia(&cozida),
        "o CONTROLO: a corrente tem de trazer o ladrilho"
    );
    assert!(cozida.count() > 0, "e tem de ter linhas");

    assert!(
        resolve(&m, true).is_none(),
        "quem veio de uma forma desenha PIXEIS, nao gizmo"
    );
}

/// **O TECTO CORTA E O TOTAL FICA** — a legenda continua a poder dizer quantas há.
#[test]
fn o_tecto_corta_e_o_total_fica() {
    let n = MAX_PONTOS + 500;
    let s = nuvem(n);
    assert_eq!(super::posicoes(&s).len(), MAX_PONTOS);
    assert_eq!(s.count(), n, "o total nao e' o que o tecto deixou");
}

/// ⭐⭐⭐ **O CUSTO DE PINTAR O GIZMO, por feição** — a medição que decide o [`MAX_PONTOS`] (§0.0:
/// *«antes de escrever qualquer limite, MEÇA»*).
///
/// **O recurso é o tempo de CODIFICAR os caminhos no quadro** (o mesmo do `MAX_CONTORNOS` do gizmo
/// do colisor), contra um orçamento de **`1,67 ms`** = 1/10 de um quadro de 60 Hz — a mesma régua
/// que o indicador da pose usa, e pela mesma razão: *um gizmo é um passageiro do quadro, não o
/// assunto dele*.
///
/// `cargo test -p ph2d-app-motion --release --lib -- --ignored --nocapture mede_o_custo_do_gizmo`
#[test]
#[ignore = "sonda de relogio, nao um gate"]
fn mede_o_custo_do_gizmo_de_pontos() {
    use ph2d_vector::VectorScene;
    eprintln!("\n=== CUSTO DE PINTAR O GIZMO ===");
    eprintln!("  (orcamento: 1,67 ms = 1/10 de um quadro de 60 Hz)\n");
    for feicao in [Feicao::Ponto, Feicao::Corda, Feicao::Osso] {
        for n in [256_usize, 1024, 4096, 16_384, 65_536] {
            #[expect(clippy::cast_precision_loss, reason = "coordenadas de sonda")]
            let pontos: Vec<[f32; 2]> = (0..n)
                .map(|i| [(i % 256) as f32 * 0.05, (i / 256) as f32 * 0.05])
                .collect();
            let segmentos = match feicao {
                Feicao::Ponto => Vec::new(),
                Feicao::Corda => super::corda(n),
                Feicao::Osso => (1..n).map(|i| [i - 1, i]).collect(),
            };
            let v = PontoGizmoView {
                grupos: vec![Grupo {
                    node: ph2d_nodegraph::graph::NodeId(0),
                    feicao,
                    pontos,
                    segmentos,
                    total: n,
                }],
            };
            let mut tempos: Vec<f64> = (0..9)
                .map(|_| {
                    let mut cena = VectorScene::new();
                    let t = std::time::Instant::now();
                    crate::ponto_gizmo_overlay::draw(
                        &v,
                        &ph2d_render::Camera2d::default(),
                        ph2d_editor_core::screens::layout::CenterSplit::None,
                        ph2d_host::WindowSize {
                            width: 1200,
                            height: 800,
                        },
                        &mut cena,
                    );
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            tempos.sort_by(f64::total_cmp);
            let med = tempos[tempos.len() / 2];
            eprintln!(
                "  {:<8} {n:>7} elementos │ {med:>7.3} ms │ {:>6.1} % do orcamento",
                format!("{feicao:?}"),
                med * 100.0 / 1.67
            );
        }
        eprintln!();
    }
}
