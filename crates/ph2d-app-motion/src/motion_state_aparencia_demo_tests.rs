//! Os gates da cena `=118` — a do ciclo 7 (doc 112 §4-octies).
//!
//! ⚠️⚠️ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente**
//! (`CLAUDE.md` §5.0). O anúncio promete nove coisas; cada uma que se pode medir é um gate aqui —
//! incluindo as que dependem de o dono MEXER num controlo (os passos 3, 6, 8 e 9), que se medem
//! mexendo no MESMO param que a linha do cartão mexe.
//!
//! ⚠️ **Cada medição corre numa cena NOVA** ([`corre`]): três dos seis panos carregam estado por um
//! `pre`, e um `Cook` que já avançou os tiques para um pano entrega o outro a meio do caminho.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::Graph;

const DT: f64 = 1.0 / 60.0;

/// O nó do tipo `tipo` a montante do sink — pelo cone do PRODUTO (`upstream_cone`, que não
/// atravessa `pre`). ⚠️ Todo tipo que um gate procura é ÚNICO no pano onde o procura (os dois
/// osciladores do meio nunca são procurados), e o gate da contagem
/// (`each_pair_differs_by_what_the_announcement_says`) é o que impede o `find` de passar a escolher.
fn a_montante(g: &Graph, sink: NodeId, tipo: &str) -> Option<NodeId> {
    ph2d_nodegraph::cook::upstream_cone(g, sink)
        .into_iter()
        .find(|n| g.node(*n).is_some_and(|i| i.type_name == tipo))
}

/// Monta uma cena NOVA, aplica `mexe` ao pano `k`, coze `ticks` quadros e devolve o stream do sink
/// no último — e, se pedido, o do primeiro nó do tipo `tambem` a montante dele, no MESMO quadro.
fn corre(
    k: usize,
    ticks: u64,
    tambem: Option<&str>,
    mexe: impl FnOnce(&mut Graph, NodeId),
) -> (Stream, Option<Stream>) {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    let sink = sinks[k];
    mexe(&mut m.doc.graph, sink);
    let outro = tambem.map(|t| a_montante(&m.doc.graph, sink, t).expect("o no' pedido"));
    let mut fim = (Stream::default(), None);
    for f in 0..ticks {
        let t = f as f64 * DT;
        let s = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, sink, t)
            .expect("coze")[0]
            .as_stream()
            .clone();
        let o = outro.map(|n| {
            m.pump
                .cook
                .cook(&m.doc.graph, &m.registry, n, t)
                .expect("coze")[0]
                .as_stream()
                .clone()
        });
        fim = (s, o);
        m.pump
            .cook
            .advance_tick(&m.doc.graph, &m.registry, t)
            .expect("tique");
    }
    fim
}

/// O param `nome` do primeiro nó `tipo` a montante — o que a linha do cartão escreve.
fn poe(tipo: &'static str, nome: &'static str, v: f32) -> impl FnOnce(&mut Graph, NodeId) {
    move |g, sink| {
        let n = a_montante(g, sink, tipo).expect("o no' a mexer");
        g.set_param(n, nome, v);
    }
}

fn nada(_: &mut Graph, _: NodeId) {}

fn vec2(s: &Stream, col: &str) -> Vec<[f32; 2]> {
    match s.get(col) {
        Some(Column::Vec2(v)) => v.clone(),
        outra => panic!("a coluna `{col}` devia ser Vec2: {outra:?}"),
    }
}

fn vec4(s: &Stream, col: &str) -> Vec<[f32; 4]> {
    match s.get(col) {
        Some(Column::Vec4(v)) => v.clone(),
        outra => panic!("a coluna `{col}` devia ser Vec4: {outra:?}"),
    }
}

/// ⭐ **SÃO SEIS PANOS, e cada um parte das 36 peças que o anúncio nomeia.**
#[test]
fn the_scene_is_six_cloths_of_thirty_six() {
    let n = (LADO * LADO) as usize;
    for k in 0..6 {
        let (s, _) = corre(k, 1, None, nada);
        assert_eq!(s.count(), n, "pano {k} no primeiro quadro");
    }
}

fn cores_distintas(c: &[[f32; 4]]) -> usize {
    let mut v: Vec<[i32; 3]> = c
        .iter()
        .map(|p| [0, 1, 2].map(|k| (p[k] * 1000.0).round() as i32))
        .collect();
    v.sort_unstable();
    v.dedup();
    v.len()
}

/// ⭐⭐⭐ **EM CIMA: a esquerda é de UMA cor e a direita de uma cor POR PEÇA** (passo 2).
#[test]
fn the_top_pair_is_one_colour_against_one_per_piece() {
    let ce = vec4(&corre(0, 1, None, nada).0, "tint");
    let cd = vec4(&corre(1, 1, None, nada).0, "tint");
    assert_eq!(
        cores_distintas(&ce),
        1,
        "o pano do `Tint` tem de ser de UMA cor"
    );
    assert!(
        cores_distintas(&cd) >= 12,
        "o pano da `Color Ramp` tem {} cor(es) -- o arco-iris nao atravessa o pano",
        cores_distintas(&cd)
    );
    // A cor da esquerda é a que a cena escolheu, não o branco da identidade.
    let c = ce[0];
    assert!(
        (0..3).all(|k| (c[k] - LARANJA[k]).abs() < 1e-5),
        "o `Tint` devia pintar o laranja da cena, e pinta {c:?}"
    );
}

/// ⭐⭐⭐ **O PASSO 3: outra cor no `Tint` pinta o pano da esquerda INTEIRO, e não toca o da
/// direita.**
#[test]
fn recolouring_the_tint_repaints_only_the_left_cloth() {
    let esq = vec4(&corre(0, 1, None, poe("motion.tint", "g", 0.9)).0, "tint");
    assert!(
        esq.iter().all(|c| (c[1] - 0.9).abs() < 1e-5),
        "a cor nova tem de chegar a TODAS as pecas da esquerda"
    );
    // O da direita não tem `Tint`: mexer no da esquerda é o mesmo que não mexer.
    let dir_antes = vec4(&corre(1, 1, None, nada).0, "tint");
    let dir_depois = vec4(
        &corre(1, 1, None, |g, _| {
            let t = g
                .nodes()
                .iter()
                .find(|i| i.type_name == "motion.tint")
                .map(|i| i.id)
                .expect("o Tint da esquerda");
            g.set_param(t, "g", 0.9);
        })
        .0,
        "tint",
    );
    assert_eq!(dir_antes, dir_depois, "o pano da direita nao pode mudar");
}

/// A distância de cada linha ao centro da sua célula, num pano centrado em `(cx, cy)`.
fn raios_da_roda(p: &[[f32; 2]], cx: f32, cy: f32) -> Vec<f32> {
    p.iter()
        .map(|q| {
            let c = celula_de(q, cx, cy);
            ((q[0] - c[0]).powi(2) + (q[1] - c[1]).powi(2)).sqrt()
        })
        .collect()
}

/// O centro da célula a que uma posição pertence (a roda nunca sai dela: `RODA < VAO/2`).
fn celula_de(q: &[f32; 2], cx: f32, cy: f32) -> [f32; 2] {
    let meio = (LADO - 1.0) * 0.5;
    let (u, v) = ((q[0] - cx) / VAO + meio, (q[1] - cy) / VAO + meio);
    [cx + (u.round() - meio) * VAO, cy + (v.round() - meio) * VAO]
}

/// A banda do raio da roda. ⚠️ O «seno» do oscilador é a aproximação da casa (HR-5), logo a roda
/// não é um círculo exacto por construção — e a sonda `probe_the_cards_of_the_appearance_scene`
/// mediu-a a `±0,1 %` do `RODA` (`0,09999 .. 0,10010` com `RODA = 0,1`). A banda é dez vezes isso.
const BANDA_DA_RODA: (f32, f32) = (0.99, 1.01);

/// ⭐⭐⭐ **NO MEIO: as duas metades andam em RODA, e só a direita deixa CAUDA** (passo 5).
///
/// ⚠️ **A régua da roda é o RAIO**, não «mexe-se»: toda linha — a cabeça e cada eco — fica a `RODA`
/// do centro da sua célula.
#[test]
fn the_middle_pair_turns_and_only_the_right_leaves_a_tail() {
    let n = (LADO * LADO) as usize;
    // Uma volta inteira: a cauda (12 gerações, uma a cada 4 tiques) enche-se em 44.
    let (esq, _) = corre(2, 120, None, nada);
    let (dir, _) = corre(3, 120, None, nada);
    assert_eq!(esq.count(), n, "a esquerda nao tem rasto");
    assert_eq!(
        dir.count(),
        n * ECOS as usize,
        "a direita tem de ter a cabeca e mais {} ecos por peca",
        ECOS as usize - 1
    );
    for (cx, s) in [(-COL_X, &esq), (COL_X, &dir)] {
        let r = raios_da_roda(&vec2(s, "P"), cx, fileira_y(1));
        let (lo, hi) = r
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        assert!(
            lo > RODA * BANDA_DA_RODA.0 && hi < RODA * BANDA_DA_RODA.1,
            "pano em x={cx}: as linhas deviam estar a {RODA} do centro da celula, e estao entre \
             {lo} e {hi}"
        );
    }
}

/// O maior buraco, em graus, na roda de UMA célula (a da última linha — uma cabeça viva).
fn buraco_na_roda(s: &Stream) -> f32 {
    let p = vec2(s, "P");
    let alvo = celula_de(&p[p.len() - 1], COL_X, fileira_y(1));
    let mut a: Vec<f32> = p
        .iter()
        .filter(|q| celula_de(q, COL_X, fileira_y(1)) == alvo)
        .map(|q| (q[1] - alvo[1]).atan2(q[0] - alvo[0]).to_degrees())
        .collect();
    a.sort_by(f32::total_cmp);
    let mut g = 360.0 - (a[a.len() - 1] - a[0]);
    for w in a.windows(2) {
        g = g.max(w[1] - w[0]);
    }
    g
}

/// ⭐⭐⭐ **O PASSO 6: o `Length` no fim do slider FECHA o anel, e o `Tail Alpha` a 1 deixa a cauda
/// acesa.**
#[test]
fn the_longest_tail_closes_the_ring_and_full_alpha_stops_the_fade() {
    let max_len = MotionState::new()
        .registry
        .param_ui(ph2d_nodegraph::node::NodeTypeId::of("motion.trail"))
        .and_then(|h| h.iter().find(|h| h.param == "length"))
        .map(|h| h.max)
        .expect("o hint do Length");
    let (antes, _) = corre(3, 200, None, nada);
    let (depois, _) = corre(3, 200, None, |g, sink| {
        let tr = a_montante(g, sink, "motion.trail").expect("o Trail");
        g.set_param(tr, "length", max_len);
        g.set_param(tr, "fade", 1.0);
    });
    let (b0, b1) = (buraco_na_roda(&antes), buraco_na_roda(&depois));
    assert!(
        b0 > 90.0,
        "controlo: com o Length da cena a cauda NAO fecha o anel (buraco de {b0} graus)"
    );
    assert!(
        b1 < 30.0,
        "com o Length no fim a cauda devia fechar o anel, e fica um buraco de {b1} graus"
    );
    let menor = vec4(&depois, "tint")
        .iter()
        .map(|c| c[3])
        .fold(f32::MAX, f32::min);
    assert!(
        (menor - 1.0).abs() < 1e-4,
        "com o Tail Alpha a 1 nenhum eco se apaga -- o mais apagado tem alfa {menor}"
    );
    // O controlo: no default da cena a cauda APAGA-SE.
    let menor0 = vec4(&antes, "tint")
        .iter()
        .map(|c| c[3])
        .fold(f32::MAX, f32::min);
    assert!(
        menor0 < 0.5,
        "controlo: a cauda da cena apaga-se ({menor0})"
    );
}

/// Quantos tiques a fileira do tempo corre antes de se medir — a linha de atraso enche-se em
/// `ATRASO`.
const REGIME: u64 = 90;

/// O deslocamento vertical de cada peça em relação à pose de repouso (o `motion.move`).
fn subidas(k: usize, ticks: u64, mexe: impl FnOnce(&mut Graph, NodeId)) -> Vec<f32> {
    let (s, r) = corre(k, ticks, Some("motion.move"), mexe);
    let (p, q) = (vec2(&s, "P"), vec2(&r.expect("o repouso"), "P"));
    p.iter().zip(&q).map(|(a, b)| a[1] - b[1]).collect()
}

/// `(maior espalhamento dentro de uma COLUNA, maior espalhamento dentro de uma LINHA)`.
fn espalhamentos(sobe: &[f32]) -> (f32, f32) {
    let lado = LADO as usize;
    let faixa = |v: Vec<f32>| {
        let (a, b) = v
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        b - a
    };
    let col = (0..lado)
        .map(|c| faixa((0..lado).map(|r| sobe[r * lado + c]).collect()))
        .fold(0.0, f32::max);
    let lin = (0..lado)
        .map(|r| faixa((0..lado).map(|c| sobe[r * lado + c]).collect()))
        .fold(0.0, f32::max);
    (col, lin)
}

/// ⭐⭐⭐ **EM BAIXO: a esquerda atrasa pela ORDEM, a direita pelo LUGAR** (passo 7).
///
/// ⚠️ **A régua é o espalhamento DENTRO de uma coluna:** pelo lugar, as seis linhas de uma coluna
/// chegam juntas (`0`); pela ordem, cada linha chega depois da anterior.
///
/// ⚠️⚠️ **E a ordem começa em BAIXO** — a 1.ª redacção do anúncio dizia *«como quem lê um
/// texto»*, e a sonda mediu a peça `0` no canto de BAIXO à esquerda (a grade é row-major a partir
/// do `y` menor). O gate prende a frase que ficou.
#[test]
fn the_bottom_pair_delays_by_order_and_by_place() {
    let (s, _) = corre(4, 1, None, nada);
    let p = vec2(&s, "P");
    let n = p.len();
    assert!(
        p[0][1] < p[n - 1][1] && p[0][0] < p[5][0],
        "a peca 0 tem de nascer em BAIXO a esquerda (o anuncio diz que a onda comeca pela linha \
         de baixo): {:?} · {:?} · {:?}",
        p[0],
        p[5],
        p[n - 1]
    );
    let (col_e, lin_e) = espalhamentos(&subidas(4, REGIME, nada));
    let (col_d, lin_d) = espalhamentos(&subidas(5, REGIME, nada));
    assert!(
        col_d < 1e-4,
        "pelo LUGAR as seis linhas de uma coluna tem de subir JUNTAS, e espalham {col_d}"
    );
    assert!(
        lin_d > ONDA * 0.3,
        "pelo LUGAR a onda tem de atravessar a linha, e espalha so' {lin_d}"
    );
    assert!(
        col_e > ONDA * 0.3,
        "pela ORDEM as linhas de uma coluna chegam em tempos diferentes, e espalham so' {col_e}"
    );
    assert!(lin_e > 0.0, "pela ORDEM a linha tambem ondula ({lin_e})");
}

/// ⭐⭐⭐ **O PASSO 8: `Delay By = Field` SEM campo faz a onda SUMIR** — o pano inteiro atrasa por
/// igual, e continua a subir e descer.
#[test]
fn field_mode_without_a_field_delays_the_whole_cloth_alike() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("cena");
    let scan = a_montante(&m.doc.graph, sinks[4], "motion.slit_scan").expect("o da esquerda");
    assert_eq!(
        m.doc.graph.label(scan),
        Some(NOME_ORDEM),
        "o passo nomeia ESTE cartao"
    );
    let campo = poe(
        "motion.slit_scan",
        ph2d_node_motion_slit_scan::RAMP,
        POR_CAMPO,
    );
    let (col, lin) = espalhamentos(&subidas(4, REGIME, campo));
    assert!(
        col < 1e-4 && lin < 1e-4,
        "sem campo o atraso e' o mesmo para todos e a onda some -- espalha {col} / {lin}"
    );
    // ⚠️ E o pano NÃO parou: dois instantes, duas alturas.
    let a = subidas(
        4,
        REGIME,
        poe(
            "motion.slit_scan",
            ph2d_node_motion_slit_scan::RAMP,
            POR_CAMPO,
        ),
    )[0];
    let b = subidas(
        4,
        REGIME + 20,
        poe(
            "motion.slit_scan",
            ph2d_node_motion_slit_scan::RAMP,
            POR_CAMPO,
        ),
    )[0];
    assert!(
        (a - b).abs() > ONDA * 0.3,
        "o pano parou de subir e descer ({a} e {b}) -- o passo 8 promete a onda a SUMIR, nao o \
         movimento"
    );
}

/// A coluna cujo movimento é o do oscilador SEM atraso — a de onde a onda parte.
fn coluna_sem_atraso(mexe: impl FnOnce(&mut Graph, NodeId)) -> usize {
    let (s, o) = corre(5, REGIME, Some("motion.oscillator"), mexe);
    let (p, vivo) = (vec2(&s, "P"), vec2(&o.expect("o vivo"), "P"));
    let d = |c: usize| (p[c][1] - vivo[c][1]).abs();
    (0..LADO as usize)
        .min_by(|&a, &b| d(a).total_cmp(&d(b)))
        .expect("seis colunas")
}

/// ⭐⭐⭐ **O PASSO 9: `Invert` no `Falloff` troca o lado de onde a onda parte.**
///
/// ⚠️ A régua é **qual coluna anda com o oscilador sem atraso** — a da esquerda com o campo normal,
/// a da direita com ele invertido. *«A onda passa a correr para o outro lado» é isto, medido.*
#[test]
fn inverting_the_field_turns_the_wave_around() {
    assert_eq!(
        coluna_sem_atraso(nada),
        0,
        "com o campo normal a onda parte da ESQUERDA"
    );
    assert_eq!(
        coluna_sem_atraso(poe("motion.falloff", "invert", 1.0)),
        LADO as usize - 1,
        "com o campo invertido a onda parte da DIREITA"
    );
}

/// ⚠️⚠️ **O QUE MUDA EM CADA PAR — contado.**
///
/// ⛔ Sem isto alguém «afina» a cena mudando um param de um lado, os gates acima continuam verdes, e
/// o anúncio — que diz o que muda — passa a mentir.
#[test]
fn each_pair_differs_by_what_the_announcement_says() {
    let mut m = MotionState::new();
    let _ = build(&mut m.doc, &m.registry).expect("cena");
    let g = &m.doc.graph;
    let tipos: Vec<&str> = g.nodes().iter().map(|i| i.type_name.as_str()).collect();
    let conta = |t: &str| tipos.iter().filter(|x| **x == t).count();
    for (t, n) in [
        ("motion.grid", 6),
        ("motion.scale", 6),
        ("motion.move", 6),
        ("motion.output", 6),
        // Cima: UM de cada — a troca.
        ("motion.tint", 1),
        ("motion.color_ramp", 1),
        // Meio: dois osciladores por metade; baixo: um por metade.
        ("motion.oscillator", 6),
        // Meio: UM rasto.
        ("motion.trail", 1),
        // Baixo: dois slit-scans e UM campo.
        ("motion.slit_scan", 2),
        ("motion.falloff", 1),
    ] {
        assert_eq!(conta(t), n, "`{t}`: {tipos:?}");
    }
    assert_eq!(tipos.len(), 36, "nada alem do contado: {tipos:?}");
    // ⚠️ E os dois slit-scans diferem SÓ no `Delay By` (e no nome).
    let scans: Vec<_> = g
        .nodes()
        .iter()
        .filter(|i| i.type_name == "motion.slit_scan")
        .map(|i| i.id)
        .collect();
    let p = |n: NodeId| g.node_param_overrides(n).cloned().unwrap_or_default();
    let (a, b) = (p(scans[0]), p(scans[1]));
    assert_eq!(a.get("lag"), b.get("lag"), "o mesmo Lag");
    let so_ramp = |x: &std::collections::BTreeMap<String, f32>| {
        let mut x = x.clone();
        x.remove(ph2d_node_motion_slit_scan::RAMP);
        x
    };
    assert_eq!(so_ramp(&a), so_ramp(&b), "nada mais difere");
    assert_ne!(
        a.get(ph2d_node_motion_slit_scan::RAMP),
        b.get(ph2d_node_motion_slit_scan::RAMP),
        "o Delay By difere"
    );
}

/// **SONDA — os cartões que a cena `=118` pinta, e as medidas de onde saem as bandas.**
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn probe_the_cards_of_the_appearance_scene() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("118", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    eprintln!("\n  cartão             | linhas");
    for v in &snap.nodes {
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        eprintln!("  {:<18} | {}", v.display_name, rows.join(" · "));
    }
    let (s, _) = corre(4, 1, None, nada);
    let p = vec2(&s, "P");
    eprintln!(
        "\n  peca 0 {:?} · peca 5 {:?} · peca 30 {:?} · peca 35 {:?}",
        p[0], p[5], p[30], p[35]
    );
    let (dir, _) = corre(3, 120, None, nada);
    let r = raios_da_roda(&vec2(&dir, "P"), COL_X, fileira_y(1));
    let (lo, hi) = r
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    eprintln!("  raio da roda: {lo} .. {hi} (RODA {RODA})");
    let (col_e, lin_e) = espalhamentos(&subidas(4, REGIME, nada));
    let (col_d, lin_d) = espalhamentos(&subidas(5, REGIME, nada));
    eprintln!("  espalhamento ORDEM col {col_e} lin {lin_e} · LUGAR col {col_d} lin {lin_d}");
    eprintln!(
        "  buraco na roda: cena {} graus\n",
        buraco_na_roda(&corre(3, 200, None, nada).0)
    );
}

/// ⭐⭐ **O PASSO 6 DO TUTORIAL: `Tail Size` a 1 — as cópias deixam de encolher.**
#[test]
fn a_full_tail_size_stops_the_shrink() {
    let (s, _) = corre(3, 120, None, poe("motion.trail", "shrink", 1.0));
    let lados: Vec<f32> = vec2(&s, "size").iter().map(|z| z[0]).collect();
    assert!(
        lados.iter().all(|l| (l - PECA).abs() < 1e-5),
        "com o Tail Size a 1 toda copia tem o tamanho da peca ({PECA}); o menor e' {}",
        lados.iter().copied().fold(f32::MAX, f32::min)
    );
    // O controlo: no default da cena a cauda ENCOLHE.
    let (s0, _) = corre(3, 120, None, nada);
    let menor = vec2(&s0, "size")
        .iter()
        .map(|z| z[0])
        .fold(f32::MAX, f32::min);
    assert!(
        menor < PECA * 0.9,
        "controlo: a cauda da cena encolhe ({menor})"
    );
}

/// ⭐⭐ **O PASSO 10 DO TUTORIAL: com o campo em `Circle`, os CANTOS mexem-se primeiro e o CENTRO
/// chega por último.**
///
/// ⚠️ A régua é a do passo 9: a peça sem atraso anda COM o oscilador. Um canto está fora do raio
/// do círculo (campo `0`, atraso `0`); as quatro peças do meio estão perto do centro (campo perto de
/// `1`, atraso perto do `Lag` inteiro).
#[test]
fn a_circular_field_makes_the_corners_lead_and_the_centre_follow() {
    let lado = LADO as usize;
    let canto = 0;
    let meio = (lado / 2 - 1) * lado + lado / 2 - 1;
    let mut atraso_do_meio = 0.0f32;
    for ticks in [REGIME, REGIME + 15] {
        let (s, o) = corre(
            5,
            ticks,
            Some("motion.oscillator"),
            poe("motion.falloff", "shape", 0.0),
        );
        let (p, vivo) = (vec2(&s, "P"), vec2(&o.expect("o vivo"), "P"));
        assert!(
            (p[canto][1] - vivo[canto][1]).abs() < 1e-5,
            "o canto esta' fora do circulo e nao atrasa -- anda {} longe do vivo",
            (p[canto][1] - vivo[canto][1]).abs()
        );
        atraso_do_meio = atraso_do_meio.max((p[meio][1] - vivo[meio][1]).abs());
    }
    assert!(
        atraso_do_meio > ONDA * 0.1,
        "as pecas do meio tem de chegar atrasadas -- andam so' {atraso_do_meio} longe do vivo"
    );
}

/// ⭐⭐ **O PASSO 12 DO TUTORIAL: o `Tint` em `Gradient` pinta mais do que uma cor.**
#[test]
fn the_tint_gradient_mode_paints_more_than_one_colour() {
    let c = vec4(
        &corre(0, 1, None, poe("motion.tint", "mode", 1.0)).0,
        "tint",
    );
    assert!(
        cores_distintas(&c) >= 12,
        "o Tint em Gradient devia atravessar o pano, e pinta {} cor(es)",
        cores_distintas(&c)
    );
}
