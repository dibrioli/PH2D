//! Os portões do [`super::ponto_gizmo`] — a ordem do dono de 2026-09-17/19.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};

/// Coze pela porta do PRODUTO (a mesma que o quadro chama), e é ela que enche as tomadas.
fn coze(m: &mut MotionState, sinks: &[ph2d_nodegraph::graph::NodeId]) {
    coze_em(m, sinks, 0);
}

/// O mesmo, num QUADRO escolhido — para quem precisa de dois instantes do mesmo grafo.
fn coze_em(m: &mut MotionState, sinks: &[ph2d_nodegraph::graph::NodeId], quadro: u64) {
    let graph = m.doc.graph.clone();
    let MotionState { pump, registry, .. } = m;
    pump.advance_or_scrub_scoped(
        &graph,
        registry,
        sinks,
        quadro,
        |t| t as f64 / 60.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &ph2d_nodegraph::cook::TimeScopes::new(),
    );
}

fn nuvem(n: usize) -> Stream {
    Stream::new(n).with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
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
    let taps = taps_for(&m, true);
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
        assert!(!g.pontos.is_empty());
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
    m.pump.set_taps(&taps_for(&m, true));
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
    m.pump.set_taps(&taps_for(&m, true));
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
///
/// ⚠️ **A PREMISSA DESTE GATE MORREU PELA METADE em 2026-09-19** (report do dono: *«Gap y quebrou e
/// movimenta tudo em vez de criar espaço»*): ele media só que o tecto CORTA, e o corte era um
/// `take` — um PREFIXO. Numa grelha de `360 × 360` isso são as primeiras 11,4 fileiras de 360, uma
/// faixa na borda que **voa** quando o `gap_y` muda (medido: `15,8×` mais deslocamento do que
/// espaçamento). *Cortar quanto baste e cortar de onde deve são duas propriedades, e só a primeira
/// estava gateada.*
#[test]
fn o_tecto_corta_e_o_total_fica() {
    let n = MAX_PONTOS + 500;
    let s = nuvem(n);
    let am = super::Amostra::de(&s);
    assert_eq!(super::posicoes(&s, &am).len(), MAX_PONTOS);
    assert_eq!(s.count(), n, "o total nao e' o que o tecto deixou");
}

/// ⭐⭐⭐ **TODA COLUNA LÊ PELOS MESMOS ÍNDICES — o vector paralelo, gateado.**
///
/// ⚠️⚠️ **É o defeito MUDO que a amostra por passo introduz:** a `P`, a `size`, a `rot`, a forma e o
/// tamanho do gizmo são vectores paralelos, e uma delas colhida com outro passo daria ao elemento
/// `i` o tamanho do elemento `j` — **sem erro, sem aviso, e com toda régua de contagem verde**.
///
/// ⭐ **A régua amarra as duas colunas PELOS DADOS:** cada elemento leva o próprio índice em `P.x`
/// **e** em `size`, logo um desalinhamento de um único passo é visível como uma desigualdade —
/// ⛔ sem precisar de expor o `Amostra::indice`, que é o que tornaria o gate uma cópia da lei.
#[test]
fn toda_coluna_le_pelos_mesmos_indices() {
    let total = MAX_PONTOS * 3 + 7; // acima do tecto, e NÃO múltiplo dele
    #[allow(clippy::cast_precision_loss)]
    let marca: Vec<f32> = (0..total).map(|i| i as f32).collect();
    let s = Stream::new(total)
        .with("P", Column::Vec2(marca.iter().map(|&i| [i, 0.0]).collect()))
        .with("size", Column::Scalar(marca.clone()))
        .with("rot", Column::Scalar(marca.clone()));
    let am = super::Amostra::de(&s);
    let pontos = super::posicoes(&s, &am);
    let escala = super::escalas(&s, &am).expect("a coluna existe");
    let rot = super::rotacoes(&s, &am).expect("a coluna existe");
    assert_eq!(pontos.len(), MAX_PONTOS, "o tecto continua a cortar");
    assert_eq!(escala.len(), pontos.len(), "as colunas tem o mesmo tamanho");
    for k in 0..pontos.len() {
        let i = pontos[k][0]; // o índice de quem este glifo É
        assert!(
            (escala[k] - i).abs() < f32::EPSILON && (rot[k] - i).abs() < f32::EPSILON,
            "o glifo {k} e' o elemento {i} e leva os valores de outro: escala {}, rot {}",
            escala[k],
            rot[k]
        );
    }
    // ⛔ **O CONTROLO:** a amostra tem de SALTAR de facto, senão isto passaria sobre um prefixo —
    // e um prefixo alinha trivialmente.
    assert!(
        pontos[1][0] > 1.5,
        "o CONTROLO: com um prefixo o alinhamento e' trivial ({})",
        pontos[1][0]
    );
}

/// ⭐⭐⭐ **A METADE NOVA: A AMOSTRA VARRE A NUVEM INTEIRA, e não uma FAIXA dela.**
///
/// A régua é a do report — a mesma das duas sondas: **extensão contra centro**. Uma amostra
/// representativa tem o centro da nuvem e quase toda a extensão dela; um prefixo tem os dois
/// errados, e é isso que faz o `gap_y` ler-se como deslocamento.
///
/// ⚠️ **A fixtura é uma GRELHA `row-major` e tem de estar ACIMA do tecto** — é a forma exacta da
/// cena do dono, e a sonda que saiu limpa em 2026-09-19 usava `4 × 4`, que cabe no tecto e por isso
/// **nunca o engatava**. *Uma fixtura abaixo do tecto não testa o tecto.*
#[test]
fn a_amostra_varre_a_nuvem_e_nao_uma_faixa_dela() {
    // `row-major`, como a `motion.grid`: o índice `i` é a célula `(i / COLS, i % COLS)`.
    const COLS: usize = 360;
    const ROWS: usize = 360;
    let total = ROWS * COLS;
    let pontos: Vec<[f32; 2]> = (0..total)
        .map(|i| {
            let (r, c) = (i / COLS, i % COLS);
            #[allow(clippy::cast_precision_loss)]
            [
                c as f32 - (COLS as f32 - 1.0) * 0.5,
                r as f32 - (ROWS as f32 - 1.0) * 0.5,
            ]
        })
        .collect();
    let s = Stream::new(total).with("P", Column::Vec2(pontos.clone()));
    let am = super::Amostra::de(&s);
    let vistos = super::posicoes(&s, &am);
    assert_eq!(vistos.len(), MAX_PONTOS, "o tecto continua a cortar");

    let medir = |v: &[[f32; 2]]| {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for q in v {
            lo = lo.min(q[1]);
            hi = hi.max(q[1]);
        }
        (hi - lo, (hi + lo) / 2.0)
    };
    let (ext_todo, cen_todo) = medir(&pontos);
    let (ext_visto, cen_visto) = medir(&vistos);

    // O CENTRO: a nuvem é centrada na origem, e a amostra tem de o ser também.
    assert!(
        cen_visto.abs() <= 1.0,
        "a amostra esta' descentrada: centro {cen_visto} contra {cen_todo} da nuvem — \
         e' uma FAIXA, nao uma amostra"
    );
    // A EXTENSÃO: ela tem de varrer quase tudo. ⚠️ A barra é `95 %` e não `100 %` porque o último
    // índice amostrado é `(n−1)·total/n`, que fica a menos de uma fileira do fim por construção.
    assert!(
        ext_visto >= ext_todo * 0.95,
        "a amostra so' varre {ext_visto} de {ext_todo}"
    );
    // ⛔ **O CONTROLO**: o prefixo que esta wave substituiu reprova nas duas metades — sem ele um
    // `Amostra` que devolvesse a nuvem inteira passaria e não se saberia que a régua vê o defeito.
    let prefixo: Vec<[f32; 2]> = pontos.iter().take(MAX_PONTOS).copied().collect();
    let (ext_pref, cen_pref) = medir(&prefixo);
    assert!(
        cen_pref.abs() > 1.0 && ext_pref < ext_todo * 0.95,
        "o CONTROLO tem de reprovar: o prefixo mede centro {cen_pref} e extensao {ext_pref}"
    );
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
    {
        for n in [256_usize, 1024, 4096, 16_384, 65_536] {
            #[expect(clippy::cast_precision_loss, reason = "coordenadas de sonda")]
            let pontos: Vec<[f32; 2]> = (0..n)
                .map(|i| [(i % 256) as f32 * 0.05, (i / 256) as f32 * 0.05])
                .collect();
            let v = PontoGizmoView {
                grupos: vec![Grupo {
                    node: ph2d_nodegraph::graph::NodeId(0),
                    pontos,
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
                "  {n:>7} elementos │ {med:>7.3} ms │ {:>6.1} % do orcamento",
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
            pontos: vec![[0.0, 0.0]],
            rot: rot.map(|r| vec![r]),
            escala: escala.map(|e| vec![e]),
            total: 1,
        }],
    }
}

/// A caixa do que foi traçado — a régua das duas leis.
fn caixa(v: &PontoGizmoView, z: f64) -> ph2d_vector::Rect {
    crate::ponto_gizmo_overlay::caminhos(v, &olho(z), ALTURA).bounding_box()
}

/// ⭐ **A ESCALA DE UMA CENA REAL.** A `=120` autora `0,10`–`0,16`; a 1.ª redacção do gizmo passava
/// nos gates com `1` e **colapsava aqui** — por isso todo gate desta família mede com este número.
const REAL: f32 = 0.13;

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
        super::escalas(&s, &super::Amostra::inteira(3)),
        Some(vec![3.0, 3.0, 3.0]),
        "a media dos eixos"
    );
    assert_eq!(
        super::rotacoes(&s, &super::Amostra::inteira(3)),
        Some(vec![10.0, 20.0, 30.0])
    );
    assert_eq!(
        super::escalas(&nuvem(3), &super::Amostra::inteira(3)),
        None,
        "sem coluna, sem escala"
    );
    assert_eq!(
        super::rotacoes(&nuvem(3), &super::Amostra::inteira(3)),
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
    let tracos = crate::ponto_gizmo_overlay::caminhos(&v, &olho(1.0), ALTURA);
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

    // ⛔⛔ **E ela tem de ter BRAÇO — esta metade nasceu de uma mutação SOBREVIVENTE.**
    //
    // Com o braço a `0` os quatro segmentos colapsam no centro, e as duas metades acima ficam
    // **verdes**: um segmento degenerado não tem curvas e «atravessa» o centro trivialmente.
    // *Um zero de «não medido» e um de «perfeito» são o mesmo byte* — e no ecrã o artista não
    // vê marca nenhuma, que é o report do dono (*«coloque gizmos de pequenos pontos visíveis»*)
    // reaberto com a suíte a passar.
    //
    // ⚠️ **A barra é o PISO DO GLIFO e não um número escolhido:** ele é a tolerância com que uma
    // curva é achatada, abaixo da qual o traçador pode não emitir nada. A extensão de uma cruz é
    // `2 ×` o braço, logo `2 × GLIFO_MIN_PX` é o mínimo que qualquer braço legítimo produz.
    let (mut lo_x, mut hi_x) = (f64::MAX, f64::MIN);
    let (mut lo_y, mut hi_y) = (f64::MAX, f64::MIN);
    for e in &els {
        if let PathEl::MoveTo(p) | PathEl::LineTo(p) = e {
            lo_x = lo_x.min(p.x);
            hi_x = hi_x.max(p.x);
            lo_y = lo_y.min(p.y);
            hi_y = hi_y.max(p.y);
        }
    }
    let piso = 2.0 * crate::ponto_gizmo_overlay::GLIFO_MIN_PX;
    assert!(
        hi_x - lo_x >= piso && hi_y - lo_y >= piso,
        "a cruz colapsou: extensao {:.3} x {:.3} contra um piso de {piso:.3}",
        hi_x - lo_x,
        hi_y - lo_y
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
    m.pump.set_taps(&taps_for(&m, true));
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

/// ⛔⛔⛔ **COM A LEI DESLIGADA O GIZMO NÃO PEDE TOMADA NENHUMA** — e isto não é conforto.
///
/// Na rota do **DISPOSITIVO** a bomba não marcha, logo uma tomada obriga o `cook_taps_only` a
/// cozinhar aquele sink **na CPU**. *Um gizmo que não é desenhado não pode cobrar o cozimento de
/// que ele precisaria* — e a 1.ª redacção pedia a tomada SEMPRE.
///
/// ⚠️ **O preço está MEDIDO e é pequeno** (`0,216 ms` a `102 400` linhas — ver [`taps_for`]): esta
/// cerca fica pela LEI (zero custo de fábrica), e não por um relógio.
#[test]
fn com_a_lei_desligada_o_gizmo_nao_pede_tomadas() {
    let mut m = MotionState::new();
    let sinks = crate::motion_demo_legend::monta("117", &mut m.doc, &m.registry).0;
    m.sinks = sinks.clone();
    assert!(
        !taps_for(&m, true).is_empty(),
        "o CONTROLO: com a lei ligada ele pede os sinks"
    );
    assert!(
        taps_for(&m, false).is_empty(),
        "com a lei desligada ele nao pode cobrar um cozimento de CPU"
    );
}
