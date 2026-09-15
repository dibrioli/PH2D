//! Os gates da cena `=114` — e a prova de que ela ensina o que anuncia (doc 109 W3 e §5).
//!
//! ⚠️⚠️ **O cook é o da SHELL, não um `Cook` nu.** As peças são `source.shape`, que lê um EXTERNAL
//! que a shell publica (`motion_shape_gen::publish`): num cook virgem ele emite **zero**, e as duas
//! taças sairiam vazias com os gates a medir nada — o precedente que a cena `=110` já nomeia.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, Column};

/// As posições de cada metade num instante — uma nuvem por metade.
type PorMetade = Vec<Vec<[f32; 2]>>;

/// O que uma corrida devolve: as nuvens no instante zero e no fim, e as meias extensões de MUNDO
/// da caixa que a metade da direita declarou (lidas do stream COZIDO, não de uma constante).
struct Corrida {
    inicio: PorMetade,
    fim: PorMetade,
    meia_direita: Option<[f32; 2]>,
    /// O ÂNGULO de cada peça da direita no fim — vazio quando ninguém rodou (doc 109 §6).
    rot_direita: Vec<f32>,
}

/// Monta a cena num `MotionState`, deixa `mexe` ajustar o grafo, publica as formas e corre.
fn corre(secs: f64, mexe: impl FnOnce(&mut MotionState, &[NodeId])) -> Corrida {
    let mut state = MotionState::new();
    assert!(
        state.doc.graph.nodes().is_empty(),
        "o `MotionState` nasceu com nos: desligue o PH2D_GPU_COOK_DEMO para correr estes gates"
    );
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    let formas: Vec<NodeId> = state
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .collect();
    mexe(&mut state, &formas);
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let last = (secs * 60.0) as u64;
    let mut c = Corrida {
        inicio: vec![Vec::new(); sinks.len()],
        fim: vec![Vec::new(); sinks.len()],
        meia_direita: None,
        rot_direita: Vec::new(),
    };
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        for (i, sink) in sinks.iter().enumerate() {
            let s = state
                .pump
                .cook
                .cook(&state.doc.graph, &state.registry, *sink, t)
                .expect("cozinha")[0]
                .as_stream()
                .clone();
            if let Some(Column::Vec2(v)) = s.get("P") {
                if k == 0 {
                    c.inicio[i] = v.clone();
                }
                if k == last {
                    c.fim[i] = v.clone();
                }
            }
            if k == last
                && i == 1
                && let (Some(Column::Vec2(m)), Some(Column::Vec2(sz))) =
                    (s.get(COLLIDER_BOX_COLUMN), s.get("size"))
                && let (Some(m0), Some(s0)) = (m.first(), sz.first())
            {
                c.meia_direita = Some([m0[0] * s0[0].abs(), m0[1] * s0[1].abs()]);
            }
            if k == last
                && i == 1
                && let Some(Column::Scalar(r)) = s.get("rot")
            {
                c.rot_direita = r.clone();
            }
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    c
}

/// Uma corrida com `params` escritos no cartão da forma da DIREITA.
fn direita_com(params: &[(&'static str, f32)]) -> Corrida {
    corre(2.6, |state, formas| {
        for (nome, valor) in params {
            state.doc.graph.set_param(formas[1], *nome, *valor);
        }
    })
}

/// **A pilha da direita com a rotação TRAVADA** — a régua de geometria da cena.
///
/// ⚠️ **Travada de propósito:** desde o doc 109 §6 as peças TOMBAM, e um quadrado a 45° toca o
/// vizinho pela quina — o vão típico de uma pilha que roda vai até à DIAGONAL, e uma barra de
/// «encostado» medida sobre ela não separaria o encosto do ar que o dono fotografou. A rotação tem
/// gate próprio (`the_pieces_tumble_unless_the_card_locks_the_rotation`).
fn pilha_travada(params: &[(&'static str, f32)]) -> Vec<[f32; 2]> {
    let mut todos = vec![(param::LOCK_ROTATION, 1.0)];
    todos.extend_from_slice(params);
    direita_com(&todos).fim[1].clone()
}

/// A distância de cada peça ao vizinho MAIS PRÓXIMO dela — uma por peça.
fn vizinhos(p: &[[f32; 2]]) -> Vec<f32> {
    p.iter()
        .enumerate()
        .map(|(i, a)| {
            p.iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                .fold(f32::MAX, f32::min)
        })
        .collect()
}

/// A MEDIANA dessas distâncias — o vão TÍPICO da pilha (a régua que a 1.ª versão desta cena pagou:
/// o par mais próximo é um extremo, o vão típico é uma propriedade).
fn vizinho_mediano(p: &[[f32; 2]]) -> f32 {
    let mut v = vizinhos(p);
    v.sort_by(f32::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

/// A LARGURA da pilha: da peça mais à esquerda à mais à direita.
fn largura(p: &[[f32; 2]]) -> f32 {
    let (lo, hi) = p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
        (lo.min(q[0]), hi.max(q[0]))
    });
    hi - lo
}

const PECAS: usize = (ROWS * COLS) as usize;

/// ⭐⭐⭐ **Só a metade com `Collide` mantém as peças separadas — ENCOSTADAS, sem ar.**
///
/// ⚠️ O lado é o da caixa lida do STREAM cozido (`collider_box × |size|`), e não uma constante desta
/// cena: se a medição do contorno mudar, o gate segue o produto em vez de o contradizer.
///
/// ⛔⛔ **A barra de CIMA é o report do dono** (doc 109 §5, com foto: *«collider impreciso»*): o
/// círculo à volta do quadrado, a 1.ª redacção, assentava a `141 %` do lado — `41 %` de ar. Sem ela
/// o gate ficava verde sobre o defeito que o dono fotografou.
///
/// ⚠️ **E o CONTROLO é a outra metade**, senão o gate ficava verde sobre duas taças a fazer o mesmo.
#[test]
fn only_the_half_whose_shape_collides_keeps_the_pieces_apart() {
    // Com a rotação travada — ver [`pilha_travada`] para porquê.
    let c = direita_com(&[(param::LOCK_ROTATION, 1.0)]);
    assert_eq!(c.fim.len(), 2, "a cena tem duas metades");
    for (i, metade) in c.fim.iter().enumerate() {
        assert_eq!(
            metade.len(),
            PECAS,
            "a metade {i} tem de trazer as {PECAS} pecas -- vazia, a forma nao foi publicada"
        );
    }
    let m = c.meia_direita.expect("a metade da direita declara a caixa");
    let lado = 2.0 * m[0].max(m[1]);
    let (sem, com) = (vizinho_mediano(&c.fim[0]), vizinho_mediano(&c.fim[1]));
    eprintln!(
        "  vao tipico │ Collide off {sem:.4} · Collide on {com:.4} ({:.0}% do lado) · meia {m:?}",
        com / lado * 100.0
    );
    assert!(
        com >= lado * 0.85,
        "com Collide o vao tipico mede {com:.4} contra o lado declarado {lado:.4}"
    );
    assert!(
        com <= lado * 1.15,
        "a pilha tem AR: o vao tipico mede {com:.4} contra o lado {lado:.4} -- o defeito do report"
    );
    assert!(
        sem < com * 0.6,
        "sem Collide o vao tipico mede {sem:.4} contra {com:.4} -- as duas metades tem de contar \
         historias DIFERENTES"
    );
}

/// ⭐⭐ **As duas taças apanham o que cai — medido como DESCIDA de cada peça contra si própria.**
#[test]
fn both_bowls_actually_catch_what_falls() {
    let c = corre(2.6, |_, _| {});
    for (i, (a, b)) in c.inicio.iter().zip(&c.fim).enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de pecas")]
        let n = a.len().max(1) as f32;
        let descida = a.iter().zip(b).map(|(p, q)| p[1] - q[1]).sum::<f32>() / n;
        let dentro = b
            .iter()
            .filter(|p| p[1] < TACA_Y + TACA_R && p[1] > TACA_Y - TACA_R - 0.5)
            .count();
        eprintln!("  metade {i} │ descida media {descida:.3} │ na taca {dentro}");
        assert!(
            descida > 0.5,
            "na metade {i} as pecas desceram {descida:.3}"
        );
        assert!(dentro >= 20, "na metade {i} so' {dentro} ficaram na taca");
    }
}

/// ⭐⭐⭐ **O que a ordem do dono proíbe: NENHUM nó `Collide` na linha da simulação**, e a pergunta
/// da cena é UMA caixa no cartão da forma — ligada só à direita.
#[test]
fn the_scene_has_no_collide_node_and_only_the_right_shape_collides() {
    let mut state = MotionState::new();
    let _ = build(&mut state.doc, &state.registry).expect("a cena monta");
    let g = &state.doc.graph;
    assert_eq!(
        g.nodes()
            .iter()
            .filter(|n| n.type_name == "motion.collide")
            .count(),
        0,
        "o colisor e' da FORMA: nenhum `motion.collide` pode voltar a esta cena"
    );
    let colide: Vec<bool> = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| {
            g.node_param_overrides(n.id)
                .and_then(|o| o.get(param::COLLIDE).copied())
                .is_some_and(|v| v >= 0.5)
        })
        .collect();
    assert_eq!(
        colide,
        vec![false, true],
        "esquerda desligada, direita ligada"
    );
}

/// ⭐⭐⭐ **OS CONTROLOS QUE O ANÚNCIO MANDA MEXER FAZEM O QUE ELE DIZ.**
///
/// ⚠️ Um passo de smoke que manda mexer num controlo **afirma que ele tem efeito**. Todos no cartão
/// da forma da direita: desligar `Collide` (a pilha volta a borrão) · `Collider Width` e
/// `Collider Height` juntos (a pilha incha, monotonamente) · só `Collider Width` (a pilha ALARGA) ·
/// `Collider Shape = Circle` (os círculos também encostam, pelo raio que toca os lados) e
/// `Collider Radius` (incham).
#[test]
fn the_controls_the_announcement_names_do_what_it_says() {
    let ligado = pilha_travada(&[]);
    let desligado = pilha_travada(&[(param::COLLIDE, 0.0)]);
    let (on, off) = (vizinho_mediano(&ligado), vizinho_mediano(&desligado));
    eprintln!("  Collide     on {on:.4} · off {off:.4}");
    assert!(off < on * 0.6, "desligar a caixa tem de desfazer a pilha");

    let escalas = [0.6_f32, 1.0, 1.4];
    let v: Vec<f32> = escalas
        .iter()
        .map(|k| {
            vizinho_mediano(&pilha_travada(&[
                (param::COLLIDER_WIDTH, *k),
                (param::COLLIDER_HEIGHT, *k),
            ]))
        })
        .collect();
    eprintln!("  Width+Height {escalas:?} -> {v:?}");
    assert!(
        v[0] < v[1] && v[1] < v[2],
        "largura e altura juntas incham a pilha: {v:?}"
    );

    let larga = pilha_travada(&[(param::COLLIDER_WIDTH, 1.6)]);
    eprintln!(
        "  Width       1.0 -> largura {:.4} · 1.6 -> largura {:.4}",
        largura(&ligado),
        largura(&larga)
    );
    assert!(
        largura(&larga) > largura(&ligado) * 1.1,
        "so' a largura alarga a pilha: {:.4} contra {:.4}",
        largura(&larga),
        largura(&ligado)
    );

    let circulo = vizinho_mediano(&pilha_travada(&[(param::COLLIDER_SHAPE, 1.0)]));
    let circulo_maior = vizinho_mediano(&pilha_travada(&[
        (param::COLLIDER_SHAPE, 1.0),
        (param::COLLIDER_RADIUS, 1.4),
    ]));
    let lado = 2.0 * LADO;
    eprintln!(
        "  Circle      {circulo:.4} ({:.0}% do lado) · Radius 1.4 {circulo_maior:.4}",
        circulo / lado * 100.0
    );
    assert!(
        circulo >= lado * 0.85 && circulo <= lado * 1.15,
        "o circulo toca os lados do quadrado: {circulo:.4} contra {lado:.4}"
    );
    assert!(circulo_maior > circulo * 1.2, "o raio incha a pilha");
}

/// ⭐⭐⭐ **AS PEÇAS TOMBAM — e o botão `Lock Rotation` prende-as** (doc 109 §6, report do dono:
/// *«precisa destravar a rot. e colocar outro botão para travar rotação»*).
///
/// ⚠️ **As duas metades num gate**: *«tomba»* passa com uma cena que gira tudo sempre, e *«trava»*
/// passa com uma que nunca gira. E travada a coluna do ângulo **nem nasce** — a cena sai como saía
/// antes da rotação existir.
#[test]
fn the_pieces_tumble_unless_the_card_locks_the_rotation() {
    let solta = direita_com(&[]);
    let angulos = &solta.rot_direita;
    assert_eq!(
        angulos.len(),
        PECAS,
        "a metade que colide tem de trazer um angulo por peca"
    );
    let maior = angulos.iter().fold(0.0_f32, |m, a| m.max(a.abs()));
    let tortas = angulos.iter().filter(|a| a.abs() > 5.0).count();
    eprintln!("  rotacao │ maior {maior:.1}° · {tortas} de {PECAS} acima de 5°");
    assert!(
        maior > 15.0 && tortas >= 5,
        "as pecas tem de TOMBAR ao cair umas sobre as outras: maior {maior:.1}°, {tortas} tortas"
    );

    let travada = direita_com(&[(param::LOCK_ROTATION, 1.0)]);
    assert!(
        travada.rot_direita.is_empty(),
        "travada, a coluna do angulo nem nasce: {:?}",
        &travada.rot_direita[..travada.rot_direita.len().min(4)]
    );
}

/// O anúncio da cena, lido como TEXTO — para os nomes que o gate abaixo defende e os que o dono lê
/// não poderem divergir em silêncio.
const ANUNCIO: &str = include_str!("motion_state_demo_announce.rs");

/// As rows que o cartão `titulo` mostra, com os params como o dono os vê.
fn rows_do_cartao(m: &MotionState, titulo: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let v = snap
        .nodes
        .iter()
        .find(|v| v.display_name == titulo)
        .unwrap_or_else(|| {
            let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
            panic!("o anuncio manda clicar no cartao `{titulo}` e a cena tem: {havia:?}")
        });
    assert!(
        !snap.nodes.iter().any(|v| v.display_name == "Collide"),
        "apareceu um cartao `Collide` -- o colisor voltou para a linha da simulacao"
    );
    (
        v.params.iter().map(|c| c.hint.label).collect(),
        v.sections.iter().map(|s| s.title).collect(),
    )
}

/// ⭐⭐⭐ **CADA PASSO DO ANÚNCIO É POSSÍVEL NO APP.**
///
/// ⛔⛔ Um passo que manda clicar numa linha **afirma que ela está no cartão** — e esta casa já pagou
/// por escrever um passo impossível. Montada pelo ROTEADOR (`monta("114")`), que é o que o smoke
/// abre, e com os params carimbados no cartão como o dono os vê. ⚠️ O `Collider Radius` só existe
/// com `Circle`: o gate troca a forma como o anúncio manda, e confere que a largura sai.
#[test]
fn every_row_the_announcement_names_is_on_the_card() {
    let titulo = "Shape (Collide)";
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("114", &mut m.doc, &m.registry);
    let (rows, secs) = rows_do_cartao(&m, titulo);
    for linha in [
        "Collide",
        "Collider Shape",
        "Collider Width",
        "Collider Height",
    ] {
        assert!(
            rows.contains(&linha),
            "o anuncio manda mexer em `{linha}` no cartao `{titulo}`, e ele mostra {rows:?}"
        );
        assert!(
            ANUNCIO.contains(linha),
            "o gate defende `{linha}` e o anuncio nao a nomeia"
        );
    }
    assert!(
        !rows.contains(&"Collider Radius"),
        "com Box o raio nao aparece: {rows:?}"
    );
    assert!(
        secs.contains(&"Collision"),
        "o anuncio manda abrir a seccao `Collision`, e o cartao tem {secs:?}"
    );
    assert!(
        ANUNCIO.contains(titulo),
        "o anuncio tem de nomear o cartao `{titulo}`"
    );

    let direita = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .find(|id| {
            m.doc
                .graph
                .node_param_overrides(*id)
                .and_then(|o| o.get(param::COLLIDE).copied())
                .is_some_and(|v| v >= 0.5)
        })
        .expect("a forma da direita");
    m.doc.graph.set_param(direita, param::COLLIDER_SHAPE, 1.0);
    let (rows, _) = rows_do_cartao(&m, titulo);
    assert!(
        rows.contains(&"Collider Radius") && !rows.contains(&"Collider Width"),
        "com Circle o raio aparece e a largura sai: {rows:?}"
    );
    assert!(ANUNCIO.contains("Collider Radius"));
}

// ---------------------------------------------------------------------------------------------
// SONDA — *«as shapes que ficam embaixo no centro vibram muito após serem apertadas pelas shapes
// acima»* (report do dono, 2026-09-15, com foto e três setas no fundo do monte).
// ---------------------------------------------------------------------------------------------

/// **Quanto cada peça ainda MEXE depois de a pilha assentar, e ONDE ela está.**
///
/// ⚠️ A régua da cena (`vizinho_mediano`) mede o VÃO entre peças, que é uma fotografia: uma pilha a
/// tremer e uma pilha parada com o mesmo espaçamento leem-se **iguais** nela. O que o dono fotografou
/// é MOVIMENTO, e nenhuma régua desta cena o via — é por isso que os gates dela estão todos verdes
/// sobre o defeito.
///
/// A grandeza é o deslocamento por tique **ao longo da última meia janela**, em fracção do LADO da
/// peça (adimensional ⇒ comparável entre densidades). Uma peça assente lê `~0`.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_que_ainda_treme -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_que_ainda_treme() {
    let (tremor, fim, vel) = tremor_da_direita(2.0, 2.9);
    let lado = LADO;
    let mut ordem: Vec<usize> = (0..tremor.len()).collect();
    ordem.sort_by(|a, b| tremor[*b].total_cmp(&tremor[*a]));
    eprintln!("\n  peca |      x      y | tremor/lado | |vel|");
    eprintln!("  -----|---------------|-------------|-------");
    for i in ordem {
        eprintln!(
            "  {i:>4} | {:>6.3} {:>6.3} | {:>11.4} | {:>5.3}",
            fim[i][0],
            fim[i][1],
            tremor[i] / lado,
            vel[i]
        );
    }
    // A leitura que a foto pede: o tremor contra a ALTURA da peça no monte.
    let (mut baixo, mut alto) = (Vec::new(), Vec::new());
    let y_med = mediana(&fim.iter().map(|p| p[1]).collect::<Vec<_>>());
    for (i, p) in fim.iter().enumerate() {
        if p[1] < y_med {
            baixo.push(tremor[i] / lado);
        } else {
            alto.push(tremor[i] / lado);
        }
    }
    eprintln!(
        "\n  metade de BAIXO: mediana {:.4} · pior {:.4}",
        mediana(&baixo),
        baixo.iter().copied().fold(0.0_f32, f32::max)
    );
    eprintln!(
        "  metade de CIMA : mediana {:.4} · pior {:.4}",
        mediana(&alto),
        alto.iter().copied().fold(0.0_f32, f32::max)
    );
}

/// O deslocamento mediano por tique de cada peça da DIREITA entre `de` e `ate` segundos, a posição
/// final, e o módulo da velocidade final.
fn tremor_da_direita(de: f64, ate: f64) -> (Vec<f32>, Vec<[f32; 2]>, Vec<f32>) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior) = (Vec::<Vec<f32>>::new(), Vec::<[f32; 2]>::new());
    let (mut fim, mut vel) = (Vec::new(), Vec::new());
    let last = (ate * 60.0) as u64;
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(p)) = s.get("P") {
            if t >= de && anterior.len() == p.len() {
                passos.push(
                    p.iter()
                        .zip(&anterior)
                        .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                        .collect(),
                );
            }
            anterior = p.clone();
            if k == last {
                fim = p.clone();
            }
        }
        if k == last {
            vel = match s.get("vel") {
                Some(Column::Vec2(v)) => v.iter().map(|v| v[0].hypot(v[1])).collect(),
                _ => vec![f32::NAN; fim.len()],
            };
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let tremor = (0..fim.len())
        .map(|i| mediana(&passos.iter().map(|linha| linha[i]).collect::<Vec<_>>()))
        .collect();
    (tremor, fim, vel)
}

/// A mediana de uma amostra (vazia ⇒ `0`).
fn mediana(v: &[f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let mut s = v.to_vec();
    s.sort_by(f32::total_cmp);
    s[s.len() / 2]
}

/// **A DIREÇÃO do tremor, e se ele é oscilação ou deriva.**
///
/// ⚠️ As duas hipóteses previram coisas diferentes e é isto que as separa:
/// - **a velocidade só é cancelada ao longo da correcção LÍQUIDA** ⇒ uma peça apertada de LADO tem
///   `d` horizontal, a gravidade nunca é cancelada, e o tremor sai **vertical**;
/// - **oito varreduras de Jacobi não convergem numa ilha densa** ⇒ o tremor não tem direcção
///   preferida e cede a mais varreduras.
///
/// A segunda coluna é `|Σ passos| / Σ |passos|`: `0` = oscila no sítio, `1` = anda sempre para o
/// mesmo lado. *Vibrar e deslizar leem-se iguais num deslocamento por tique.*
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_a_direcao_do_tremor -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_a_direcao_do_tremor() {
    let (tremor, fim, eixo, deriva) = direcao_da_direita(2.0, 2.9);
    let mut ordem: Vec<usize> = (0..tremor.len()).collect();
    ordem.sort_by(|a, b| tremor[*b].total_cmp(&tremor[*a]));
    eprintln!("\n  peca |      x      y | tremor/lado | |dy|/(|dx|+|dy|) | deriva");
    eprintln!("  -----|---------------|-------------|------------------|-------");
    for i in ordem.into_iter().take(10) {
        eprintln!(
            "  {i:>4} | {:>6.3} {:>6.3} | {:>11.4} | {:>16.3} | {:>6.3}",
            fim[i][0],
            fim[i][1],
            tremor[i] / LADO,
            eixo[i],
            deriva[i]
        );
    }
}

/// Por peça: o passo mediano, a posição final, a fracção VERTICAL do movimento, e a deriva.
fn direcao_da_direita(de: f64, ate: f64) -> (Vec<f32>, Vec<[f32; 2]>, Vec<f32>, Vec<f32>) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior, mut fim) = (Vec::<Vec<[f32; 2]>>::new(), Vec::new(), Vec::new());
    let last = (ate * 60.0) as u64;
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(p)) = s.get("P") {
            if t >= de && anterior.len() == p.len() {
                passos.push(
                    p.iter()
                        .zip(&anterior)
                        .map(|(a, b): (&[f32; 2], &[f32; 2])| [a[0] - b[0], a[1] - b[1]])
                        .collect(),
                );
            }
            anterior = p.clone();
            if k == last {
                fim = p.clone();
            }
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let (mut tremor, mut eixo, mut deriva) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..fim.len() {
        let ds: Vec<[f32; 2]> = passos.iter().map(|linha| linha[i]).collect();
        tremor.push(mediana(
            &ds.iter().map(|d| d[0].hypot(d[1])).collect::<Vec<_>>(),
        ));
        let (ax, ay) = ds.iter().fold((0.0_f32, 0.0_f32), |(x, y), d| {
            (x + d[0].abs(), y + d[1].abs())
        });
        eixo.push(if ax + ay > 0.0 { ay / (ax + ay) } else { 0.0 });
        let soma = ds
            .iter()
            .fold([0.0_f32, 0.0], |a, d| [a[0] + d[0], a[1] + d[1]]);
        let total: f32 = ds.iter().map(|d| d[0].hypot(d[1])).sum();
        deriva.push(if total > 0.0 {
            soma[0].hypot(soma[1]) / total
        } else {
            0.0
        });
    }
    (tremor, fim, eixo, deriva)
}

/// **O A/B das causas candidatas** — cada linha desliga UMA coisa e mede o mesmo tremor.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_que_cura_o_tremor -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_que_cura_o_tremor() {
    eprintln!("\n  caso                        | baixo p50 | baixo pior | cima p50");
    eprintln!("  ----------------------------|-----------|------------|---------");
    for (rotulo, params) in [
        ("como a cena shipa", &[][..]),
        ("rotacao TRAVADA", &[(param::LOCK_ROTATION, 1.0)][..]),
    ] {
        let (tremor, fim) = tremor_com(2.0, 2.9, params);
        let y = mediana(&fim.iter().map(|p| p[1]).collect::<Vec<_>>());
        let baixo: Vec<f32> = (0..fim.len())
            .filter(|i| fim[*i][1] < y)
            .map(|i| tremor[i] / LADO)
            .collect();
        let cima: Vec<f32> = (0..fim.len())
            .filter(|i| fim[*i][1] >= y)
            .map(|i| tremor[i] / LADO)
            .collect();
        eprintln!(
            "  {rotulo:<27} | {:>9.4} | {:>10.4} | {:>8.4}",
            mediana(&baixo),
            baixo.iter().copied().fold(0.0_f32, f32::max),
            mediana(&cima)
        );
    }
}

/// O tremor mediano por peça e a posição final, com `params` escritos no cartão da DIREITA.
fn tremor_com(de: f64, ate: f64, params: &[(&'static str, f32)]) -> (Vec<f32>, Vec<[f32; 2]>) {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    let formas: Vec<NodeId> = state
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .map(|n| n.id)
        .collect();
    for (nome, valor) in params {
        state.doc.graph.set_param(formas[1], *nome, *valor);
    }
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior, mut fim) = (Vec::<Vec<f32>>::new(), Vec::new(), Vec::new());
    let last = (ate * 60.0) as u64;
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Vec2(p)) = s.get("P") {
            if t >= de && anterior.len() == p.len() {
                passos.push(
                    p.iter()
                        .zip(&anterior)
                        .map(|(a, b): (&[f32; 2], &[f32; 2])| (a[0] - b[0]).hypot(a[1] - b[1]))
                        .collect(),
                );
            }
            anterior = p.clone();
            if k == last {
                fim = p.clone();
            }
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    let tremor = (0..fim.len())
        .map(|i| mediana(&passos.iter().map(|linha| linha[i]).collect::<Vec<_>>()))
        .collect();
    (tremor, fim)
}

/// **O SINAL do ângulo das piores peças, tique a tique** — 2 ciclos = sobre-correcção.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_angulo_tique_a_tique -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_angulo_tique_a_tique() {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let mut serie: Vec<Vec<f32>> = Vec::new();
    for k in 0..=174_u64 {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if t >= 2.6
            && let Some(Column::Scalar(r)) = s.get("rot")
        {
            serie.push(r.clone());
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    eprintln!("\n  tique |   rot[2] |   rot[3] |   rot[0]");
    eprintln!("  ------|----------|----------|---------");
    for (k, r) in serie.iter().enumerate().take(14) {
        eprintln!("  {k:>5} | {:>8.4} | {:>8.4} | {:>8.4}", r[2], r[3], r[0]);
    }
    // Quantas vezes o passo TROCA de sinal: um ciclo de 2 tiques troca em todos.
    for i in [2_usize, 3, 0] {
        let d: Vec<f32> = serie.windows(2).map(|w| w[1][i] - w[0][i]).collect();
        let trocas = d.windows(2).filter(|w| w[0] * w[1] < 0.0).count();
        eprintln!(
            "  peca {i}: {trocas} trocas de sinal em {} passos · |passo| p50 = {:.5}°",
            d.len(),
            mediana(&d.iter().map(|x| x.abs()).collect::<Vec<_>>())
        );
    }
}

/// **QUEM vibra: as peças pousadas na TAÇA, ou as pousadas noutras peças?**
///
/// ⚠️ É a pergunta que a foto faz e que nenhuma coluna anterior separava — *«em baixo»* e *«apoiada
/// no mundo»* são a mesma região nesta cena, e só uma delas é o mecanismo.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_quem_vibra_toca_a_taca -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_quem_vibra_toca_a_taca() {
    let (tremor, fim, _eixo, deriva) = direcao_da_direita(2.0, 2.9);
    // A taça da DIREITA: centro em (VAO/2 + …, TACA_Y), raio TACA_R. O centro em x sai da própria
    // nuvem — a cena desloca as duas metades, e uma constante aqui envelheceria com ela.
    let cx = fim.iter().map(|p| p[0]).sum::<f32>() / fim.len() as f32;
    let meia = LADO * std::f32::consts::SQRT_2 / 2.0;
    let mut na_taca = Vec::new();
    let mut solta = Vec::new();
    eprintln!("\n  peca | dist. a' parede | tremor/lado | deriva | apoio");
    eprintln!("  -----|-----------------|-------------|--------|-------");
    let mut ordem: Vec<usize> = (0..tremor.len()).collect();
    ordem.sort_by(|a, b| tremor[*b].total_cmp(&tremor[*a]));
    for i in ordem {
        let d = TACA_R - (fim[i][0] - cx).hypot(fim[i][1] - TACA_Y) - meia;
        let toca = d.abs() < meia;
        if deriva[i] < 0.5 {
            if toca {
                na_taca.push(tremor[i] / LADO);
            } else {
                solta.push(tremor[i] / LADO);
            }
        }
        eprintln!(
            "  {i:>4} | {d:>15.4} | {:>11.4} | {:>6.3} | {}",
            tremor[i] / LADO,
            deriva[i],
            if toca { "TAÇA" } else { "peças" }
        );
    }
    eprintln!(
        "\n  (só as que OSCILAM, deriva < 0,5)\n  apoiadas na TAÇA  : n={} · p50 {:.4} · pior {:.4}",
        na_taca.len(),
        mediana(&na_taca),
        na_taca.iter().copied().fold(0.0_f32, f32::max)
    );
    eprintln!(
        "  apoiadas em PEÇAS : n={} · p50 {:.4} · pior {:.4}",
        solta.len(),
        mediana(&solta),
        solta.iter().copied().fold(0.0_f32, f32::max)
    );
}

/// **SONDA — o que o ATRITO entre peças faz ao resto do tremor.**
///
/// ⚠️ A cena não escreve material nenhum nas peças, logo elas são **gelo** entre si (`μ = 0`, o
/// `Material::LISO`): um monte sem atrito não assenta, e isso é Física e não um defeito. Esta sonda
/// diz quanto do resíduo é isso.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_o_atrito_entre_pecas -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_o_atrito_entre_pecas() {
    eprintln!("\n  atrito das peças | baixo p50 | baixo pior | cima p50");
    eprintln!("  -----------------|-----------|------------|---------");
    for mu in [0.0_f32, 0.3, 0.6, 0.9] {
        let (tremor, fim) = tremor_com(2.0, 2.9, &[(param::FRICTION, mu)]);
        let y = mediana(&fim.iter().map(|p| p[1]).collect::<Vec<_>>());
        let baixo: Vec<f32> = (0..fim.len())
            .filter(|i| fim[*i][1] < y)
            .map(|i| tremor[i] / LADO)
            .collect();
        let cima: Vec<f32> = (0..fim.len())
            .filter(|i| fim[*i][1] >= y)
            .map(|i| tremor[i] / LADO)
            .collect();
        eprintln!(
            "  {mu:>16.2} | {:>9.4} | {:>10.4} | {:>8.4}",
            mediana(&baixo),
            baixo.iter().copied().fold(0.0_f32, f32::max),
            mediana(&cima)
        );
    }
}

/// O `|Δrot|` mediano por tique de CADA peça da direita, na janela assente.
fn balanco_angular(de: f64, ate: f64) -> Vec<f32> {
    let mut state = MotionState::new();
    let sinks = build(&mut state.doc, &state.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let sink = sinks[1];
    let (mut passos, mut anterior) = (Vec::<Vec<f32>>::new(), Vec::<f32>::new());
    let last = (ate * 60.0) as u64;
    for k in 0..=last {
        #[expect(clippy::cast_precision_loss, reason = "um indice de tique")]
        let t = k as f64 / 60.0;
        let s = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if let Some(Column::Scalar(r)) = s.get("rot") {
            if t >= de && anterior.len() == r.len() {
                passos.push(
                    r.iter()
                        .zip(&anterior)
                        .map(|(a, b)| (a - b).abs())
                        .collect(),
                );
            }
            anterior = r.clone();
        }
        state
            .pump
            .cook
            .advance_tick(&state.doc.graph, &state.registry, t)
            .expect("avanca");
    }
    (0..anterior.len())
        .map(|i| mediana(&passos.iter().map(|linha| linha[i]).collect::<Vec<_>>()))
        .collect()
}

/// **SONDA — a faixa do balanço angular, para a barra do gate sair de um VALE medido.**
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_a_faixa_do_balanco -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao"]
fn probe_a_faixa_do_balanco() {
    let mut b = balanco_angular(2.0, 2.9);
    b.sort_by(f32::total_cmp);
    eprintln!("\n  |Δrot| mediano por tique, as 25 peças em ordem:");
    for (i, v) in b.iter().enumerate() {
        eprintln!("  {i:>3} | {v:>8.4}°");
    }
}

/// ⭐⭐⭐ **A PILHA ASSENTA EM VEZ DE ZUMBIR** — o gate do 4.º report do dono (2026-09-15:
/// *«as shapes que ficam embaixo no centro vibram muito após serem apertadas pelas shapes acima»*,
/// com três setas no fundo do monte). Mecanismo e tabela: [doc 109 §8].
///
/// ⚠️⚠️ **A régua da cena não podia ver isto:** o `vizinho_mediano` mede o VÃO, que é uma
/// fotografia — *uma pilha a tremer e uma pilha parada com o mesmo espaçamento leem-se iguais nela*,
/// e os quatro gates da `=114` estavam verdes sobre o defeito. Esta mede **movimento**: o `|Δrot|`
/// mediano por tique de cada peça, na janela em que a pilha já devia estar assente.
///
/// ⭐ **A barra sai de um VALE MEDIDO, com os dois lados:** com a lei antiga (a rotação como
/// empurrão de ângulo) as 25 peças leem `… 0,38 · 1,24 · 2,53 · 3,50 · 3,96 · 4,16°` — **cinco**
/// acima de `1,0` —, e com a moeda certa (`spin`) a pior lê **`0,87°`**. ⛔ Não é um número
/// escolhido: é o vazio entre o lado que o dono reprovou e o lado curado.
///
/// ⚠️ **É um gate DETERMINÍSTICO, não de relógio** — ele não divide dois tempos nem conta
/// alocações, logo não é candidato à família de flakes de carga do `CLAUDE.md` §5.0.
#[test]
fn the_pile_settles_instead_of_buzzing() {
    /// Graus por tique. Ver o vale acima.
    const BARRA: f32 = 1.0;
    let balanco = balanco_angular(2.0, 2.9);
    assert!(
        balanco.len() >= 20,
        "piso de populacao: a metade da direita tem de ter peças ({})",
        balanco.len()
    );
    let piores: Vec<(usize, f32)> = balanco
        .iter()
        .enumerate()
        .filter(|(_, v)| **v > BARRA)
        .map(|(i, v)| (i, *v))
        .collect();
    assert!(
        piores.is_empty(),
        "peças a zumbir acima de {BARRA}°/tique: {piores:?} — doc 109 §8"
    );
}
