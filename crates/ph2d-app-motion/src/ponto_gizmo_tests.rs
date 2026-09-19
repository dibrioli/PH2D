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
        resolve(&m, true, true).is_none(),
        "montada e nao cozida: sem tomadas, sem gizmo"
    );

    coze(&mut m, &sinks);

    let v = resolve(&m, true, true).expect("a cena so'-posicoes tem de dar gizmo");
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
        resolve(&m, true, true).is_some(),
        "o CONTROLO: com a ferramenta ha'"
    );
    assert!(resolve(&m, false, true).is_none(), "sem ela, nada");
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
        resolve(&m, true, true).is_none(),
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
                    rot: None,
                    escala: None,
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

// ─────────────────────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ AS DUAS LEIS DA ORDEM DO DONO DE 2026-09-19 — *«tamanho absoluto […] e precisam responder
// aos grafos (como o scale do oscilador)»*.
// ─────────────────────────────────────────────────────────────────────────────────────────────

use ph2d_vector::{Point, Shape};

/// Um `to_screen` de brincar com um zoom `z`: o MUNDO escala, a tela é a mesma.
/// A altura da área da cena nas medições — um canvas de 900 px, que é a ordem de grandeza do app.
const ALTURA: f64 = 900.0;

fn olho(z: f64) -> impl Fn([f32; 2]) -> Point {
    move |w| Point::new(f64::from(w[0]) * z + 600.0, f64::from(w[1]) * z + 400.0)
}

/// Um grupo de UM ponto na origem, com as colunas que o teste quiser.
fn um_ponto(rot: Option<f32>, escala: Option<f32>) -> PontoGizmoView {
    PontoGizmoView {
        grupos: vec![Grupo {
            node: ph2d_nodegraph::graph::NodeId(0),
            feicao: Feicao::Ponto,
            pontos: vec![[0.0, 0.0]],
            segmentos: Vec::new(),
            rot: rot.map(|r| vec![r]),
            escala: escala.map(|e| vec![e]),
            total: 1,
        }],
    }
}

/// A caixa do que foi traçado — a régua das duas leis.
fn caixa(v: &PontoGizmoView, z: f64) -> ph2d_vector::Rect {
    let (_, tracos) = crate::ponto_gizmo_overlay::caminhos(v, &olho(z), ALTURA);
    tracos.bounding_box()
}

/// ⭐ **A ESCALA DE UMA CENA REAL.** A `=120` autora `0,10`–`0,16`; a 1.ª redacção do gizmo passava
/// nos gates com `1` e **colapsava aqui** — por isso todo gate desta família mede com este número.
const REAL: f32 = 0.13;

/// ⭐⭐⭐ **LEI 1: O GLIFO NÃO MUDA COM O ZOOM.**
///
/// ⚠️ **A metade que prova que a régua VÊ o zoom é obrigatória** — com dois pontos, a DISTÂNCIA
/// entre eles tem de dobrar quando o zoom dobra. Sem ela, um `caminhos` que ignorasse o
/// `to_screen` por inteiro passaria: *uma régua que não vê o fenómeno acontecer não prova que ele
/// não aconteceu*.
#[test]
fn o_glifo_tem_tamanho_absoluto_e_a_geometria_segue_o_zoom() {
    // (a) UM ponto com a escala de uma cena real: a caixa é o glifo, e não pode mudar.
    let v = um_ponto(None, Some(REAL));
    let (a, b) = (caixa(&v, 1.0), caixa(&v, 8.0));
    assert!(
        (a.width() - b.width()).abs() < 1e-9 && (a.height() - b.height()).abs() < 1e-9,
        "o glifo mudou com o zoom: {a:?} contra {b:?}"
    );

    // (b) O CONTROLO: dois pontos, e a geometria TEM de seguir o zoom.
    let mut dois = um_ponto(None, Some(REAL));
    dois.grupos[0].pontos.push([10.0, 0.0]);
    // ⚠️ **A coluna tem de ter uma entrada POR PONTO.** Com uma só, o 2.º cai na identidade
    // (`size = 1`), que é uma pegada SETE vezes maior — e a régua mediria o glifo do vizinho em
    // vez da distância. *Foi assim que esta metade reprovou da 1.ª vez.*
    dois.grupos[0].escala = Some(vec![REAL, REAL]);
    dois.grupos[0].total = 2;
    let (l1, l8) = (caixa(&dois, 1.0).width(), caixa(&dois, 8.0).width());
    // As larguras são `10·z + glifo`; a diferença das duas mede o mundo, sem o glifo.
    let mundo1 = l1 - a.width();
    let mundo8 = l8 - a.width();
    assert!(
        (mundo8 / mundo1 - 8.0).abs() < 1e-6,
        "a GEOMETRIA tem de seguir o zoom: {mundo1} → {mundo8}"
    );
}

/// ⭐⭐⭐ **LEI 2: O GLIFO RESPONDE AO GRAFO** — o `scale` do oscilador, à letra.
#[test]
fn a_coluna_de_escala_engorda_o_glifo() {
    let base = caixa(&um_ponto(None, Some(REAL)), 1.0).width();
    let gordo = caixa(&um_ponto(None, Some(REAL * 3.0)), 1.0).width();
    let magro = caixa(&um_ponto(None, Some(REAL * 0.5)), 1.0).width();
    assert!(
        (gordo / base - 3.0).abs() < 1e-6,
        "o triplo da escala tem de dar o triplo: {base} → {gordo}"
    );
    assert!(
        (magro / base - 0.5).abs() < 1e-6,
        "metade da escala tem de dar metade: {base} → {magro}"
    );
    // ⚠️ E é INDEPENDENTE do zoom: a mesma razão a 8x.
    let gordo8 = caixa(&um_ponto(None, Some(REAL * 3.0)), 8.0).width();
    assert!(
        (gordo8 - gordo).abs() < 1e-9,
        "a escala do grafo nao e' do zoom"
    );
    // ⛔⛔ **E A MAGNITUDE, que é o que o dono reprovou em 19/09.** Com a escala de uma cena real o
    // glifo tem de ser VISÍVEL; a 1.ª redacção entregava `0,26 px`, um borrão por baixo do próprio
    // traço. *Uma razão certa sobre uma magnitude invisível passa em todo gate de razão.*
    assert!(
        base >= 4.0,
        "um glifo de {base:.2} px sobre a escala de uma cena real e' um ponto de tinta"
    );
}

/// ⛔ **O PISO existe e nomeia o recurso** — abaixo da espessura do traço um símbolo não tem
/// interior. Sem ele, um `size = 0` apagaria o gizmo e o artista leria *«o nó parou»*.
#[test]
fn uma_escala_minuscula_nao_apaga_o_gizmo() {
    let quase_zero = caixa(&um_ponto(None, Some(1e-6)), 1.0).width();
    assert!(
        quase_zero > 0.0,
        "um glifo de largura zero e' um gizmo que desapareceu"
    );
    // ⚠️ E uma coluna ENVENENADA também não o apaga.
    let nan = caixa(&um_ponto(None, Some(f32::NAN)), 1.0).width();
    assert!(nan > 0.0, "NaN nao pode apagar o gizmo");
}

/// ⭐⭐ **A AGULHA SÓ EXISTE SE O GRAFO DER DIRECÇÃO, e ela GIRA.**
///
/// ⚠️ **As duas metades:** sem a coluna, a caixa é o anel (simétrica); com ela, a caixa cresce
/// para o lado para onde o ângulo aponta. *Uma agulha em toda nuvem seria ruído sobre um grafo que
/// nunca falou de direcção.*
#[test]
fn a_agulha_da_direccao_so_existe_quando_o_grafo_a_da() {
    let sem = caixa(&um_ponto(None, Some(REAL)), 1.0);
    let com = caixa(&um_ponto(Some(0.0), Some(REAL)), 1.0);
    assert!(
        com.width() > sem.width() + 1.0,
        "com `rot` tem de aparecer a agulha: {sem:?} → {com:?}"
    );
    // A `0°` ela aponta para a DIREITA (`x` cresce), a `180°` para a esquerda.
    let direita = caixa(&um_ponto(Some(0.0), Some(REAL)), 1.0);
    let esquerda = caixa(&um_ponto(Some(180.0), Some(REAL)), 1.0);
    assert!(
        direita.x1 > esquerda.x1 && esquerda.x0 < direita.x0,
        "a agulha tem de GIRAR: {direita:?} contra {esquerda:?}"
    );
    // ⚠️ E `90°` nao e' a mesma imagem que `0°` — a armadilha da CRUZ, que roda em si mesma.
    let noventa = caixa(&um_ponto(Some(90.0), Some(REAL)), 1.0);
    assert!(
        (noventa.width() - direita.width()).abs() > 1.0,
        "um glifo que roda em si mesmo nao mostra rotacao nenhuma"
    );
}

/// **E a corrente que traz as colunas de facto as ENTREGA ao retrato** — a rota, não a porta.
#[test]
fn o_retrato_carrega_as_colunas_do_grafo() {
    let s = nuvem(3)
        .with("size", Column::Vec2(vec![[2.0, 4.0]; 3]))
        .with("rot", Column::Scalar(vec![10.0, 20.0, 30.0]));
    assert_eq!(
        super::escalas(&s, 3),
        Some(vec![3.0, 3.0, 3.0]),
        "a media dos eixos"
    );
    assert_eq!(super::rotacoes(&s, 3), Some(vec![10.0, 20.0, 30.0]));
    assert_eq!(super::escalas(&nuvem(3), 3), None, "sem coluna, sem escala");
    assert_eq!(
        super::rotacoes(&nuvem(3), 3),
        None,
        "sem coluna, sem agulha"
    );
}

/// ⛔⛔⛔ **A MARCA DE UM PONTO É UMA CRUZ, e isto é um VEREDITO DO DONO** (2026-09-19: *«vc piorou
/// os desenhos dos gizmos que estavam bons»*).
///
/// A 1.ª tentativa trocou a cruz por um anel com o argumento de que *«uma cruz rodada `90°` é a
/// MESMA cruz»* — verdadeiro, e a conclusão era errada: **quem mostra a rotação é a AGULHA**.
///
/// ⚠️⚠️ **A CAIXA NÃO SEPARA AS DUAS**: uma cruz de braço `b` e um anel de raio `b` têm a mesma
/// caixa, e foi por isso que a mutação `R4` **SOBREVIVEU** a todos os gates de tamanho. O que as
/// separa é o que elas SÃO: uma cruz é feita de RECTAS e **passa pelo centro**; um anel é feito de
/// CURVAS e tem um buraco no meio.
#[test]
fn a_marca_de_um_ponto_e_uma_cruz_e_nao_um_anel() {
    use ph2d_vector::PathEl;
    let v = um_ponto(None, Some(REAL));
    let (_, tracos) = crate::ponto_gizmo_overlay::caminhos(&v, &olho(1.0), ALTURA);
    let els: Vec<PathEl> = tracos.elements().to_vec();
    assert!(!els.is_empty(), "a fixtura tem de desenhar alguma coisa");
    assert!(
        !els.iter()
            .any(|e| matches!(e, PathEl::CurveTo(..) | PathEl::QuadTo(..))),
        "uma cruz nao tem curvas — isto e' um anel: {els:?}"
    );
    // O centro, onde o `olho(1.0)` põe a origem.
    //
    // ⚠️ **ATRAVESSA, e não «acaba em»** — a 1.ª redacção desta metade procurava um EXTREMO no
    // centro e reprovou sobre a cruz certa: os quatro extremos dela são as PONTAS dos braços.
    // *Um segmento que passa pelo centro é o que um anel nunca tem.*
    let c = olho(1.0)([0.0, 0.0]);
    let mut de = None;
    let mut atravessa = false;
    for e in &els {
        match e {
            PathEl::MoveTo(p) => de = Some(*p),
            PathEl::LineTo(p) => {
                if let Some(a) = de {
                    let (lo_x, hi_x) = (a.x.min(p.x), a.x.max(p.x));
                    let (lo_y, hi_y) = (a.y.min(p.y), a.y.max(p.y));
                    if (lo_x - 1e-9..=hi_x + 1e-9).contains(&c.x)
                        && (lo_y - 1e-9..=hi_y + 1e-9).contains(&c.y)
                    {
                        atravessa = true;
                    }
                }
                de = Some(*p);
            }
            _ => de = None,
        }
    }
    assert!(
        atravessa,
        "a cruz tem de ATRAVESSAR o centro; um anel deixa-o vazio: {els:?}"
    );
}

/// ⛔ **UM OSSO NUNCA É MAIS GORDO DO QUE O PRÓPRIO COMPRIMENTO** — sem esta cerca uma peça grande
/// numa cadeia curta desenha um losango mais largo do que longo, que já não se lê como osso.
///
/// ⚠️ Foi uma **mutação sobrevivente** (`R5`) que a pediu: apagar o limite não partia nada, porque
/// todos os outros gates medem PONTOS.
///
/// ⛔⛔ **E a PREMISSA deste gate MORREU no dia seguinte, o que é o gate a funcionar.** Ele media o
/// comprimento em pixels de **ECRÃ** (dois pontos a `20` de mundo com um olho de `z = 1` eram
/// `20 px`), e a cura do report *«os gizmos estão relativos ao zoom»* passou a cerca para pixels de
/// **REFERÊNCIA** — ali `20` de mundo são `1 800 px`, uma cadeia enorme, e a cerca deixa de morder.
/// *A fixtura tinha de mudar de regime junto com a lei.*
#[test]
fn um_osso_nunca_e_mais_gordo_do_que_o_proprio_comprimento() {
    // Curto no MUNDO (`0,25` ⇒ `22,5 px` de referência) com uma peça grande: é aqui que a cerca
    // manda.
    let comp_mundo = 0.25_f32;
    let v = PontoGizmoView {
        grupos: vec![Grupo {
            node: ph2d_nodegraph::graph::NodeId(0),
            feicao: Feicao::Osso,
            pontos: vec![[0.0, 0.0], [comp_mundo, 0.0]],
            segmentos: vec![[0, 1]],
            rot: None,
            // ⚠️ Uma peça ENORME: sem o limite a meia-largura pedia `225 px` sobre um osso de 22,5.
            escala: Some(vec![5.0, 5.0]),
            total: 2,
        }],
    };
    let (cheios, _) = crate::ponto_gizmo_overlay::caminhos(&v, &olho(1.0), ALTURA);
    let caixa = cheios.bounding_box();
    let comp_ref = f64::from(comp_mundo) * crate::ponto_gizmo_overlay::ppu_de_referencia(ALTURA);
    assert!(!cheios.is_empty(), "a fixtura tem de desenhar o osso");
    assert!(
        caixa.height() <= comp_ref + 1e-6,
        "um osso de {comp_ref:.1} px saiu com {:.1} px de gordura",
        caixa.height()
    );
}

/// ⛔⛔⛔ **O OSSO NÃO AFINA COM O ZOOM** — report do dono, 2026-09-19: *«os gizmos estão relativos
/// ao zoom»*, e ele tinha razão sobre esta feição.
///
/// A cerca que impede um osso de ser mais gordo do que longo era medida em pixels de **ECRÃ**:
/// afastar a câmara encolhia o comprimento, o limite mordia, e a cadeia **afinava**. Hoje ela é
/// medida no mundo, convertido no zoom de FÁBRICA.
///
/// ⚠️ **A fixtura tem de estar NO REGIME onde a cerca morde** (um osso curto com uma peça grande),
/// senão o gate mede o caso em que a cerca é inerte — *que é como esta mesma cerca já sobreviveu a
/// uma mutação*.
#[test]
fn o_osso_nao_afina_com_o_zoom() {
    let osso = |z: f64| {
        let v = PontoGizmoView {
            grupos: vec![Grupo {
                node: ph2d_nodegraph::graph::NodeId(0),
                feicao: Feicao::Osso,
                // 0,25 de mundo = 22,5 px de referência: a cerca (35 %) dá 7,9 px, e a pegada de
                // uma peça de `1,0` pediria 45 — logo a CERCA é quem manda.
                pontos: vec![[0.0, 0.0], [0.25, 0.0]],
                segmentos: vec![[0, 1]],
                rot: None,
                escala: Some(vec![1.0, 1.0]),
                total: 2,
            }],
        };
        let (cheios, _) = crate::ponto_gizmo_overlay::caminhos(&v, &olho(z), ALTURA);
        assert!(
            !cheios.is_empty(),
            "a fixtura tem de desenhar o osso a z={z}"
        );
        cheios.bounding_box().height()
    };
    let (perto, longe) = (osso(8.0), osso(0.5));
    assert!(
        (perto - longe).abs() < 1e-9,
        "a gordura do osso mudou com o zoom: {perto:.3} contra {longe:.3}"
    );
    // ⚠️ E o CONTROLO de que a cerca está mesmo a morder: sem ela a meia-largura seria a da peça
    // (`0,5 × 22,5 = 11,25` ⇒ altura `22,5`), e com ela é `7,9` ⇒ altura `15,75`.
    assert!(
        perto < 20.0,
        "a cerca nao esta' a morder nesta fixtura — o gate mede o caso inerte ({perto:.3})"
    );
}

/// ⛔⛔⛔ **O GIZMO SÓ EXISTE COM A LEI LIGADA** — *«os retângulos voltaram e os gizmos…»*: com a lei
/// desligada as peças desenham-se, e o gizmo por cima delas é ruído sobre arte correcta.
///
/// ⚠️ **E ele aparecia na configuração de FÁBRICA**, que é a lei da casa violada.
#[test]
fn com_a_lei_desligada_nao_ha_gizmo() {
    let mut m = MotionState::new();
    let sinks = crate::motion_demo_legend::monta("117", &mut m.doc, &m.registry).0;
    m.sinks = sinks.clone();
    m.pump.set_taps(&taps_for(&m));
    coze(&mut m, &sinks);
    assert!(
        resolve(&m, true, true).is_some(),
        "o CONTROLO: com a lei ligada ha' gizmo"
    );
    assert!(
        resolve(&m, true, false).is_none(),
        "com a lei desligada as pecas desenham-se, e o gizmo seria ruido por cima"
    );
}

/// ⛔⛔⛔ **A ARTE DO DISPOSITIVO OBEDECE À LEI** — a metade que o caminho da GPU não tinha, e que o
/// dono cobrou com *«os retângulos voltaram»*.
#[test]
fn a_arte_do_device_obedece_a_lei() {
    let mut m = MotionState::new();
    let sinks = crate::motion_demo_legend::monta("117", &mut m.doc, &m.registry).0;
    m.sinks = sinks.clone();
    m.pump.set_taps(&taps_for(&m));
    coze(&mut m, &sinks);
    // Uma cena de posições: sem fronteira com aparência ⇒ a arte NÃO desenha.
    assert!(
        !a_arte_desenha(&m, true),
        "uma cena de posicoes nao pode desenhar arte no device"
    );
    // ⛔⛔ **E uma fronteira que EXISTE e não traz aparência** — sem esta metade o gate não
    // discrimina: sobre uma lista VAZIA, *«nenhuma fronteira conta»* e *«toda fronteira conta»*
    // dão a mesma resposta, e foi uma mutação (`Z5`) que o disse.
    let grid = fronteira_de_posicoes();
    assert!(
        !a_arte_desenha(&grid, true),
        "uma FRONTEIRA de posicoes tambem nao e' aparencia"
    );
    // ⚠️ E o CONTROLO: com a lei DESLIGADA ela desenha — senão isto apagaria o app inteiro.
    assert!(
        a_arte_desenha(&m, false),
        "com a lei desligada a arte tem de desenhar"
    );
}

/// ⛔⛔⛔ **E A METADE POSITIVA: uma cena COM aparência desenha no dispositivo.**
///
/// ⚠️ **Sem ela a irmã é VÁCUA, e foi uma mutação sobrevivente (`Z4`) que o disse:** trocar o
/// predicado por *«nenhuma fronteira tem aparência»* apaga toda cena de objectos no device e passa
/// a irmã à mesma. *Um gate que só mede o lado que cala nunca vê o lado que apaga o app.*
#[test]
fn uma_cena_com_aparencia_desenha_no_device() {
    use ph2d_nodegraph::graph::Graph;
    let mut m = MotionState::new();
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
    crate::motion_lsystem_testkit::publish_object_alpha(&mut m, "Bola", 0, false);

    // ⚠️ **A FRONTEIRA é o que viaja para o dispositivo** — cozê-la é o que o hand-off da rota
    // híbrida faz, e é a porta que o `a_arte_desenha` lê.
    let graph = m.doc.graph.clone();
    let MotionState { pump, registry, .. } = &mut m;
    pump.advance_or_scrub_to_nodes_scoped(
        &graph,
        registry,
        &[obj],
        0,
        |t| t as f64 / 60.0,
        &ph2d_nodegraph::cook::TimeScopes::new(),
    );
    assert!(
        !m.pump.boundary_streams().is_empty(),
        "a fixtura tem de COZER a fronteira, senao o gate mede o vazio"
    );
    assert!(
        a_arte_desenha(&m, true),
        "uma cena de OBJECTOS tem de continuar a desenhar no device"
    );
}

/// Um estado cuja FRONTEIRA está cozida e **não traz aparência** — a metade que discrimina
/// *«nenhuma conta»* de *«toda conta»*.
fn fronteira_de_posicoes() -> MotionState {
    use ph2d_nodegraph::graph::Graph;
    let mut m = MotionState::new();
    let mut g = Graph::new();
    let grelha = g.add_node("motion.grid");
    let saida = g.add_node("motion.output");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (grelha, 0),
        to: (saida, 0),
        delayed: false,
    })
    .expect("liga");
    m.doc.graph = g;
    m.sinks = vec![saida];
    let graph = m.doc.graph.clone();
    let MotionState { pump, registry, .. } = &mut m;
    pump.advance_or_scrub_to_nodes_scoped(
        &graph,
        registry,
        &[grelha],
        0,
        |t| t as f64 / 60.0,
        &ph2d_nodegraph::cook::TimeScopes::new(),
    );
    assert!(
        !m.pump.boundary_streams().is_empty(),
        "a fixtura tem de COZER a fronteira"
    );
    m
}
