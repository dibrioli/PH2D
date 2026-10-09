//! Os gates da cena `=132`: a fixtura CONTÉM o fenómeno — com a onda a meio, a laranja (`Index`)
//! cresce FILA A FILA e a azul (`From Point`) em ANÉIS — e a galáxia gira (a simulação corre), pela
//! BOMBA do produto.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};

fn vec2(s: &Stream, nome: &str) -> Vec<[f32; 2]> {
    match s.get(nome) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Corre a cena `tiques` quadros pela bomba e devolve, por coluna, o stream do `Stagger` (as
/// posições simuladas e o tamanho que a onda deu) e as posições da galáxia no primeiro quadro.
fn marcha(tiques: u64) -> [(Stream, Vec<[f32; 2]>); 2] {
    let mut m = MotionState::new();
    let colunas = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = colunas.iter().map(|c| c.saida).collect();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let mut inicio = [Vec::new(), Vec::new()];
    for t in 0..tiques {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
        if t == 0 {
            for (k, c) in colunas.iter().enumerate() {
                let s = m.pump.cook.peek(c.galaxia).expect("a galaxia cozeu")[0]
                    .as_stream()
                    .clone();
                inicio[k] = vec2(&s, "P");
            }
        }
    }
    // ⚠️ A régua lê a SAÍDA (o que é desenhado), não o `Stagger`: a 1.ª foto da cena mostrou as
    // 800 estrelas do mesmo tamanho com o `Stagger` a variar — o que chega ao ecrã é a pergunta.
    let ler = |c: &Coluna| {
        m.pump.cook.peek(c.saida).expect("a saida cozeu")[0]
            .as_stream()
            .clone()
    };
    let [a, b] = &colunas;
    let [ia, ib] = inicio;
    [(ler(a), ia), (ler(b), ib)]
}

/// As estrelas DENTRO da onda (a encolher) — `(fileira de partida, distância ao centro)` de cada.
fn na_onda(s: &Stream) -> Vec<(usize, f32)> {
    let (tam, p) = (vec2(s, "size"), vec2(s, "P"));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o lado da grelha"
    )]
    let lado = LADO as usize;
    // Na SAÍDA o tamanho é o da estrela vezes o do ponto, e a posição já levou o `offset_x` da
    // coluna: lê-se RELATIVO ao maior tamanho e ao centro da galáxia.
    let maior = tam.iter().map(|t| t[0]).fold(0.0_f32, f32::max);
    #[expect(clippy::cast_precision_loss, reason = "poucas estrelas")]
    let n = p.len().max(1) as f32;
    let c = p
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0] / n, a[1] + q[1] / n]);
    eprintln!(
        "  saida: {} estrelas, tamanho {:.3}..{maior:.3}",
        tam.len(),
        tam.iter().map(|t| t[0]).fold(f32::MAX, f32::min)
    );
    tam.iter()
        .zip(&p)
        .enumerate()
        .filter(|(_, (t, _))| t[0] < 0.5 * maior)
        .map(|(i, (_, q))| (i / lado, (q[0] - c[0]).hypot(q[1] - c[1])))
        .collect()
}

/// `(fileiras ocupadas, largura do anel em metros)` de um conjunto.
fn espalho(d: &[(usize, f32)]) -> (usize, f32) {
    let filas = d.iter().map(|x| x.0).max().unwrap_or(0) - d.iter().map(|x| x.0).min().unwrap_or(0);
    let r = |f: fn(f32, f32) -> f32, z: f32| d.iter().map(|x| x.1).fold(z, f);
    (filas, r(f32::max, 0.0) - r(f32::min, f32::MAX))
}

/// ⭐⭐ **A cena ensina o que acontece** (`CLAUDE.md` §5.0): a onda é uma FAIXA de estrelas a
/// encolher (antes dela a estrela ainda mostra a volta anterior, depois dela já cresceu). A `1,5 s`
/// a faixa da laranja (`Index`) cabe em POUCAS fileiras e atravessa a galáxia de lado a lado (o
/// anel dela é largo); a da azul (`From Point`) é um ANEL estreito que corta muitas fileiras. Cada
/// lado é o CONTROLO do outro.
#[test]
fn the_wave_runs_in_rows_on_the_left_and_in_rings_on_the_right() {
    let [(esq, _), (dir, _)] = marcha(90);
    let (de, dd) = (na_onda(&esq), na_onda(&dir));
    let ((fe, ae), (fd, ad)) = (espalho(&de), espalho(&dd));
    eprintln!(
        "cena 132 a 1,5 s: laranja {} na onda, {fe} fileiras, anel {ae:.2} m | azul {} na onda, \
         {fd} fileiras, anel {ad:.2} m",
        de.len(),
        dd.len()
    );
    assert!(
        de.len() >= 20 && dd.len() >= 20,
        "ha' uma onda a passar nas duas"
    );
    assert!(fe <= 4, "a laranja: a onda e' uma faixa de fileiras ({fe})");
    assert!(
        ae > 3.0 * ad,
        "e atravessa a galaxia ({ae:.2} m, o anel da azul {ad:.2} m)"
    );
    assert!(
        ad < 0.25 * ALCANCE,
        "a azul: a onda e' um anel estreito ({ad:.2} m)"
    );
    assert!(
        fd >= fe + 4,
        "que corta muitas fileiras ({fd} contra {fe} da laranja)"
    );
}

/// A SIMULAÇÃO corre: em 3 s a galáxia gira (as estrelas deixam a grelha de partida), e não foge
/// nem colapsa (a extensão fica perto da de partida).
#[test]
fn the_galaxies_turn_and_keep_their_size() {
    let [(esq, p0), _] = marcha(180);
    let p = vec2(&esq, "P");
    assert_eq!(p.len(), p0.len());
    let andou = p
        .iter()
        .zip(&p0)
        .map(|(a, b)| (a[0] + COL_X - b[0]).hypot(a[1] - b[1]))
        .fold(0.0_f32, f32::max);
    // A saída já levou o `offset_x` da coluna: a extensão mede-se a partir do CENTRO da galáxia.
    let ext = |v: &[[f32; 2]]| {
        #[expect(clippy::cast_precision_loss, reason = "poucas estrelas")]
        let n = v.len().max(1) as f32;
        let c = v
            .iter()
            .fold([0.0_f32; 2], |a, q| [a[0] + q[0] / n, a[1] + q[1] / n]);
        v.iter()
            .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
            .fold(0.0_f32, f32::max)
    };
    eprintln!(
        "cena 132: a estrela que mais andou {andou:.3} m · extensao {:.2} -> {:.2}",
        ext(&p0),
        ext(&p)
    );
    assert!(andou > 0.3, "a galaxia gira: {andou}");
    assert!(
        (ext(&p) - ext(&p0)).abs() < 0.25 * ext(&p0),
        "nem foge nem colapsa"
    );
}

/// As duas colunas são IGUAIS menos o `Order` — a cena compara uma coisa só.
#[test]
fn the_two_columns_differ_only_in_the_order() {
    let mut m = MotionState::new();
    let [a, b] = monta(&mut m.doc, &m.registry).expect("a cena monta");
    let g = &m.doc.graph;
    for (x, y) in [(a.tempo, b.tempo), (a.stagger, b.stagger)] {
        let (mut px, mut py) = (
            g.node_param_overrides(x).cloned().unwrap_or_default(),
            g.node_param_overrides(y).cloned().unwrap_or_default(),
        );
        assert_ne!(
            px.remove("order"),
            py.remove("order"),
            "o `Order` e' a diferenca"
        );
        assert_eq!(px, py, "tudo o resto e' igual");
    }
}

/// A FONTE do tutorial deste ciclo — lida, para que os nomes do gate e os do texto não divirjam.
const TUTORIAL: &str = include_str!("../../../docs/Motion Nodes/tutoriais/src/10_o_ritmo.html");

/// A PALAVRA que a linha mostra (o instantâneo pode trazer a chave ou já a palavra).
fn palavra(l: &str) -> String {
    if l.starts_with("node.") {
        ph2d_i18n::tr(l).to_string()
    } else {
        l.to_string()
    }
}

/// As linhas que o cartão mostra, com o nó num estado dado.
fn linhas(m: &MotionState, no: NodeId) -> Vec<&'static str> {
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let v = snap
        .nodes
        .iter()
        .find(|v| v.id == no.0)
        .expect("o cartao esta no instantaneo");
    v.params.iter().map(|c| c.hint.label).collect()
}

/// Um passo do tutorial: o estado do cartão `Time` e as linhas que ele tem de mostrar.
type Passo = (&'static [(&'static str, f32)], &'static [&'static str]);

/// ⭐⭐⭐ **CADA PASSO DO TUTORIAL É POSSÍVEL NO APP** (doc 103 §1, passo 7): cada linha que o texto
/// manda mexer está no cartão `Time` da fila de cima NO ESTADO em que o passo a pede — o
/// `Origin X` só existe com `From Point`, o `Columns` só com `Grid`, o `Damping` só com `Spring`.
#[test]
fn every_row_the_rhythm_tutorial_names_is_on_the_card() {
    let mut m = MotionState::new();
    let [laranja, _] = monta(&mut m.doc, &m.registry).expect("a cena monta");
    let t = laranja.tempo;
    let passos: &[Passo] = &[
        (&[], &["Delay", "Order", "Ease", "Direction"]),
        (&[("order", 4.0)], &["Origin X", "Origin Y", "Reach"]),
        (&[("order", 3.0)], &["Columns", "From", "Axis"]),
        (&[("order", 2.0)], &["Seed"]),
        (&[("ease_curve", 12.0)], &["Steps"]),
        (&[("ease_curve", 11.0)], &["Period"]),
        (&[("ease_curve", 13.0)], &["Damping"]),
        (
            &[("ease_curve", 14.0)],
            &["Handle 1 X", "Handle 1 Y", "Handle 2 X", "Handle 2 Y"],
        ),
    ];
    for (estado, pedidas) in passos {
        let mut mm = MotionState::new();
        mm.doc = m.doc.clone();
        for &(p, v) in *estado {
            mm.doc.graph.set_param(t, p, v);
        }
        let rows: Vec<String> = linhas(&mm, t).iter().map(|l| palavra(l)).collect();
        for l in *pedidas {
            assert!(
                rows.iter().any(|r| r == l),
                "o tutorial manda mexer em `{l}` (estado {estado:?}) e o cartao mostra {rows:?}"
            );
            assert!(
                TUTORIAL.contains(&format!("<code>{l}</code>")),
                "o gate defende `{l}` e o texto nao a nomeia"
            );
        }
    }
    // E o controlo: com `Delay = 0` a onda some do cartão (a nota do tutorial).
    m.doc.graph.set_param(t, "delay", 0.0);
    let rows: Vec<String> = linhas(&m, t).iter().map(|l| palavra(l)).collect();
    assert!(
        !rows.iter().any(|r| r == "Order"),
        "Delay a zero esconde a onda: {rows:?}"
    );
}

/// Cada figura existe, e o tutorial abre a cena deste ciclo.
#[test]
fn the_rhythm_tutorial_opens_this_scene_and_its_figures_exist() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    let mut n = 0;
    for pedaco in TUTORIAL.split("src=\"../fig/").skip(1) {
        let nome = pedaco.split('"').next().expect("o nome da figura");
        assert!(
            dir.join(nome).is_file(),
            "o tutorial mostra `{nome}` e a figura nao existe"
        );
        n += 1;
    }
    assert_eq!(n, 2, "duas figuras, achei {n}");
    assert!(TUTORIAL.contains("PH2D_GPU_COOK_DEMO=132"));
    assert!(crate::motion_state::demo_router::is_cycle_scene("132"));
}
