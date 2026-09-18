//! Os gates da cena `=115` — o MATERIAL, e a prova de que ela ensina o que anuncia (doc 109 §7).
//!
//! ⚠️⚠️ **O cook é o da SHELL, não um `Cook` nu** — as bolas são `source.shape`, que lê um EXTERNAL
//! que a shell publica: num cook virgem ele emite **zero** e os gates mediriam nada. É o mesmo
//! precedente que a `=110` e a `=114` já nomeiam.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// O que uma corrida devolve por quadrante: a nuvem no princípio e no fim, e os ÂNGULOS no fim.
struct Corrida {
    inicio: Vec<Vec<[f32; 2]>>,
    fim: Vec<Vec<[f32; 2]>>,
    /// O `y` mais alto que a peça 0 alcançou DEPOIS do primeiro toque — a régua do salto.
    pico_depois_do_toque: Vec<f32>,
    rot: Vec<Vec<f32>>,
}

impl Corrida {
    /// O ângulo da peça 0 (as fileiras de cima têm uma bola só).
    fn rot0(&self, i: usize) -> f32 {
        self.rot[i].first().copied().unwrap_or(0.0)
    }
    /// Quanto a nuvem toda rodou, em módulo — a régua das taças.
    fn giro_total(&self, i: usize) -> f32 {
        self.rot[i].iter().map(|a| a.abs()).sum()
    }
    /// A largura da nuvem no fim.
    fn largura(&self, i: usize) -> f32 {
        let (lo, hi) = self.fim[i]
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), q| {
                (lo.min(q[0]), hi.max(q[0]))
            });
        hi - lo
    }
}

/// Monta a cena, deixa `mexe` ajustar os cartões e corre `secs` segundos.
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
    assert_eq!(formas.len(), 6, "seis quadrantes");
    mexe(&mut state, &formas);
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let last = (secs * 60.0) as u64;
    let n = sinks.len();
    let mut c = Corrida {
        inicio: vec![Vec::new(); n],
        fim: vec![Vec::new(); n],
        pico_depois_do_toque: vec![f32::MIN; n],
        rot: vec![Vec::new(); n],
    };
    // O `y` mínimo já visto por quadrante: o pico só conta DEPOIS de a peça ter descido.
    let mut fundo = vec![f32::MAX; n];
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
                if let Some(p) = v.first() {
                    fundo[i] = fundo[i].min(p[1]);
                    if p[1] < fundo[i] + 1e-4 {
                        c.pico_depois_do_toque[i] = p[1];
                    } else {
                        c.pico_depois_do_toque[i] = c.pico_depois_do_toque[i].max(p[1]);
                    }
                }
            }
            if k == last {
                c.rot[i] = match s.get("rot") {
                    Some(Column::Scalar(r)) => r.clone(),
                    _ => Vec::new(),
                };
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

/// Os índices dos quadrantes, na ordem de [`quadrantes`].
const RAMPA_GELO: usize = 0;
const RAMPA_ATRITO: usize = 1;
const QUEDA_MORTA: usize = 2;
const QUEDA_VIVA: usize = 3;
const TACA_GELO: usize = 4;
const TACA_ATRITO: usize = 5;

/// ⭐⭐⭐ **A RESPOSTA AO REPORT, NA CENA** (*«os círculos não rotacionam com a colisão»*): a mesma
/// bola na mesma rampa RODA com `Friction 1` e NÃO roda com `Friction 0`.
///
/// ⚠️ **E o `Friction 0` é o controlo que vale mais que a metade que roda**: sem ele o gate ficava
/// verde sobre uma cena que gira tudo sempre, que é o defeito oposto e igualmente mudo.
#[test]
fn the_ball_with_friction_rolls_and_the_icy_one_does_not() {
    let c = corre(2.0, |_, _| {});
    let (gelo, rola) = (c.rot0(RAMPA_GELO), c.rot0(RAMPA_ATRITO));
    let desceu = |i: usize| c.inicio[i][0][0] - c.fim[i][0][0];
    eprintln!(
        "  rampa │ gelo {gelo:.1}° (andou {:.2}) · atrito {rola:.1}° (andou {:.2})",
        desceu(RAMPA_GELO),
        desceu(RAMPA_ATRITO)
    );
    assert!(
        desceu(RAMPA_GELO) > 0.3 && desceu(RAMPA_ATRITO) > 0.3,
        "as duas bolas tem de DESCER a rampa"
    );
    // ⚠️ **`1e-3°` e não zero ao bit, e o número tem MECANISMO:** a alavanca da normal num disco é
    // zero em aritmética exacta, mas o `ponto − centro` de uma bola a `x = 1,35` é uma subtracção
    // de dois números `30×` maiores do que a diferença, e o que sobra é ruído de cancelamento
    // (`~1e-7` por tique). Medido nesta cena: **`0,0000075°` em 2 s** — sete milionésimos de grau.
    // ⛔ *Uma barra de zero exacto aqui mediria a aritmética, não a lei.*
    assert!(
        gelo.abs() < 1e-3,
        "com `Friction 0` a bola nao pode rodar, e rodou {gelo}°"
    );
    assert!(
        rola.abs() > 90.0,
        "com `Friction 1` a bola tem de RODAR (mais de um quarto de volta), e rodou {rola}°"
    );
    // ⭐ E no SENTIDO certo: a descer para `−x` ela roda no sentido ANTI-horário (graus > 0).
    assert!(
        rola > 0.0,
        "a rodar ao contrario do que a descida pede: {rola}"
    );
}

/// ⭐⭐ **E o que ela rodou é o que ROLAR pede** — o arco `andou / raio`, e não um número qualquer.
///
/// ⚠️ A barra é larga (`±35 %`) de propósito: a bola começa a derrapar e só depois agarra, então o
/// arco total fica ABAIXO do rolamento puro. O que este gate exclui é a outra ordem de grandeza —
/// um giro decorativo que não tem nada a ver com a distância percorrida.
#[test]
fn what_it_turns_is_what_rolling_asks_for() {
    let c = corre(2.0, |_, _| {});
    let (a, b) = (c.inicio[RAMPA_ATRITO][0], c.fim[RAMPA_ATRITO][0]);
    let andou = (a[0] - b[0]).hypot(a[1] - b[1]);
    let pedido = andou / RAIO * ph2d_contact::GRAUS;
    let razao = c.rot0(RAMPA_ATRITO) / pedido;
    eprintln!(
        "  rolamento │ andou {andou:.3} · pedia {pedido:.0}° · rodou {:.0}° ({razao:.2}x)",
        c.rot0(RAMPA_ATRITO)
    );
    assert!(
        (0.65..=1.35).contains(&razao),
        "o giro tem de ser o ARCO que a bola andou: pedia {pedido:.0}°, rodou {:.0}°",
        c.rot0(RAMPA_ATRITO)
    );
}

/// ⭐⭐ **A BOLA SALTITANTE SALTA, E A MORTA NÃO** — a outra metade do material.
#[test]
fn the_bouncy_ball_comes_back_up_and_the_dead_one_stays_down() {
    let c = corre(2.0, |_, _| {});
    let (morta, viva) = (
        c.pico_depois_do_toque[QUEDA_MORTA],
        c.pico_depois_do_toque[QUEDA_VIVA],
    );
    eprintln!("  salto │ Bounciness 0 pico {morta:.3} · 0,9 pico {viva:.3} (chao {CHAO_Y})");
    assert!(
        morta < CHAO_Y + RAIO + 0.05,
        "com `Bounciness 0` a bola tem de MORRER no chao, e subiu ate' {morta:.3}"
    );
    // ⚠️ **A barra sai da MEDIÇÃO e não do valor autorado:** a queda é de `0,75` e o salto medido
    // sobe `0,372` acima da pousada — metade da altura, e não os `81 %` que `0,9²` sugere, porque
    // o embate dura um tique inteiro de gravidade. *Uma barra derivada de `restitution²` mediria a
    // conta de cabeça em vez do que a cena faz.*
    assert!(
        viva > CHAO_Y + RAIO + 0.25,
        "com `Bounciness 0,9` ela tem de SALTAR, e o pico foi {viva:.3}"
    );
}

/// ⭐⭐⭐ **OS DOIS CONTROLOS QUE O ANÚNCIO MANDA MEXER FAZEM O QUE ELE DIZ.**
///
/// ⚠️ Um passo de smoke que manda arrastar um slider **afirma que ele tem efeito** — e esta casa já
/// pagou por escrever um passo impossível.
#[test]
fn the_two_material_sliders_do_what_the_announcement_says() {
    // `Friction` a 0 na bola que rolava: ela passa a escorregar sem virar.
    let travado = corre(2.0, |state, formas| {
        state
            .doc
            .graph
            .set_param(formas[RAMPA_ATRITO], param::FRICTION, 0.0);
    });
    eprintln!("  Friction 1 -> 0 │ rot {}°", travado.rot0(RAMPA_ATRITO));
    // A mesma barra de ruído do gate acima — ver o mecanismo lá.
    assert!(
        travado.rot0(RAMPA_ATRITO).abs() < 1e-3,
        "arrastar `Friction` a 0 tem de parar a rotacao, e sobrou {}°",
        travado.rot0(RAMPA_ATRITO)
    );

    // `Bounciness` a 0 na bola que saltava: ela passa a morrer no chão.
    let morta = corre(2.0, |state, formas| {
        state
            .doc
            .graph
            .set_param(formas[QUEDA_VIVA], param::BOUNCE, 0.0);
    });
    eprintln!(
        "  Bounciness 0,9 -> 0 │ pico {:.3}",
        morta.pico_depois_do_toque[QUEDA_VIVA]
    );
    assert!(
        morta.pico_depois_do_toque[QUEDA_VIVA] < CHAO_Y + RAIO + 0.05,
        "arrastar `Bounciness` a 0 tem de matar o salto"
    );
}

/// ⭐⭐⭐ **O QUE DIFERE ENTRE AS DUAS METADES DE UM PAR É UM PARAM DA FORMA — e mais nada.**
///
/// ⚠️ Se a diferença estivesse no OBSTÁCULO, a cena ensinaria que o material é do mundo. Ele é da
/// PEÇA, e é isso que este gate prende: os quatro `sim.collide` têm o mesmo atrito e a mesma
/// restituição, e os quatro `sim.zone` o mesmo relógio.
#[test]
fn only_the_shape_card_differs_between_the_halves_of_a_pair() {
    let mut state = MotionState::new();
    let _ = build(&mut state.doc, &state.registry).expect("a cena monta");
    let g = &state.doc.graph;
    let numero = |tipo: &str, p: &str| -> Vec<f32> {
        g.nodes()
            .iter()
            .filter(|n| n.type_name == tipo)
            .map(|n| {
                g.node_param_overrides(n.id)
                    .and_then(|o| o.get(p).copied())
                    .unwrap_or(f32::NAN)
            })
            .collect()
    };
    // ⚠️ **Por PAR, e não globalmente**: a taça é ice nas duas metades e a rampa é áspera nas
    // duas — o que a lei proíbe é o mundo mudar DENTRO de um par, que é onde a comparação vive.
    for p in ["restitution", "friction"] {
        let v = numero("sim.collide", p);
        assert_eq!(v.len(), 6, "seis obstaculos");
        for par in v.chunks(2) {
            assert!(
                par[0] == par[1],
                "o `{p}` do obstaculo tem de ser o MESMO nas duas metades de um par: {v:?}"
            );
        }
    }
    let dur = numero("sim.zone", "duration");
    assert!(
        dur.windows(2).all(|w| w[0] == w[1]),
        "o relogio tem de ser o mesmo: {dur:?}"
    );
    // E os cartões das formas diferem exactamente nos dois números da pergunta.
    let (at, sal) = (
        numero("source.shape", param::FRICTION),
        numero("source.shape", param::BOUNCE),
    );
    assert_eq!(at[RAMPA_GELO], 0.0);
    assert_eq!(at[RAMPA_ATRITO], 1.0);
    assert_eq!(sal[QUEDA_MORTA], 0.0);
    assert!(sal[QUEDA_VIVA] > 0.5);
}

/// O anúncio da cena, lido como TEXTO — os nomes que o gate defende e os que o dono lê não podem
/// divergir em silêncio.
const ANUNCIO: &str = include_str!("motion_state_demo_announce.rs");

/// ⭐⭐⭐ **CADA LINHA QUE O ANÚNCIO MANDA CLICAR ESTÁ NO CARTÃO.**
///
/// ⛔⛔ Um passo que manda clicar numa linha **afirma que ela está lá**. Montada pelo ROTEADOR
/// (`monta("115")`), que é o que o smoke abre.
#[test]
fn every_row_the_announcement_names_is_on_the_card() {
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta("115", &mut m.doc, &m.registry);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    for titulo in [
        "Friction 1: ROLA",
        "Bounciness 0,9: SALTA",
        "Entre bolas, Friction 1: ROLAM umas nas outras",
    ] {
        let v = snap
            .nodes
            .iter()
            .find(|v| v.display_name == titulo)
            .unwrap_or_else(|| {
                let havia: Vec<&str> = snap.nodes.iter().map(|v| v.display_name.as_str()).collect();
                panic!("o anuncio manda clicar no cartao `{titulo}` e a cena tem: {havia:?}")
            });
        let rows: Vec<&str> = v.params.iter().map(|c| c.hint.label).collect();
        let secs: Vec<&str> = v.sections.iter().map(|s| ph2d_i18n::tr(s.title)).collect();
        for linha in ["Friction", "Bounciness"] {
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
            secs.contains(&"Collision"),
            "o anuncio manda abrir a seccao `Collision`, e o cartao tem {secs:?}"
        );
        assert!(
            ANUNCIO.contains(titulo),
            "o anuncio tem de nomear o cartao `{titulo}`"
        );
    }
}

// ───────────────────── A terceira fileira: o material ENTRE SHAPES ─────────────────────

/// ⭐⭐⭐ **O MATERIAL VALE ENTRE AS PRÓPRIAS SHAPES** — 3.º report do dono (2026-09-13:
/// *«as propriedades entre as próprias shapes não funcionam»*).
///
/// ⚠️⚠️ **A taça é ESCORREGADIA nas duas metades**, e é isso que torna este gate uma afirmação
/// sobre peça-contra-peça: com um obstáculo áspero, metade da rotação viria da parede e o gate
/// ficaria verde sem provar nada sobre o par de bolas. Aqui `μ` contra a parede é `√(0 · μ) = 0`
/// nos dois lados, logo **todo** o giro medido nasce de bola contra bola.
#[test]
fn the_material_acts_between_the_shapes_themselves() {
    // ⛔⛔ **A PREMISSA PRIMEIRO, e ela é metade do gate.** Sem esta linha, dar atrito à parede
    // deixava tudo abaixo VERDE — e o que estaria a ser medido era o mundo a rodar as bolas, não
    // as bolas umas às outras. *Um gate que não prende a própria premissa afirma outra coisa.*
    // (Medido: a mutação que põe a taça áspera SOBREVIVEU a todos os outros gates desta cena.)
    {
        let mut m = MotionState::new();
        let _ = build(&mut m.doc, &m.registry).expect("a cena monta");
        let atritos: Vec<f32> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "sim.collide")
            .map(|n| {
                m.doc
                    .graph
                    .node_param_overrides(n.id)
                    .and_then(|o| o.get("friction").copied())
                    .unwrap_or(f32::NAN)
            })
            .collect();
        assert_eq!(
            (atritos[TACA_GELO], atritos[TACA_ATRITO]),
            (0.0, 0.0),
            "as duas tacas tem de ser ESCORREGADIAS para o giro medido ser de bola contra bola: \
             {atritos:?}"
        );
    }
    let c = corre(2.2, |_, _| {});
    let (gelo, atrito) = (c.giro_total(TACA_GELO), c.giro_total(TACA_ATRITO));
    eprintln!(
        "  entre bolas │ gelo {gelo:.1}° (largura {:.3}) · atrito {atrito:.1}°          (maior {:.1}°, largura {:.3})",
        c.largura(TACA_GELO),
        c.rot[TACA_ATRITO]
            .iter()
            .fold(0.0_f32, |m, a| m.max(a.abs())),
        c.largura(TACA_ATRITO)
    );
    assert_eq!(
        c.fim[TACA_GELO].len(),
        c.fim[TACA_ATRITO].len(),
        "as duas tacas tem de ter as MESMAS bolas"
    );
    assert!(
        c.fim[TACA_ATRITO].len() >= 9,
        "uma bola so' nunca toca noutra: {} pecas",
        c.fim[TACA_ATRITO].len()
    );
    assert!(
        gelo.abs() < 1.0,
        "com `Friction 0` entre bolas nada pode rodar, e rodou {gelo}°"
    );
    assert!(
        atrito > 200.0,
        "com `Friction 1` as bolas tem de ROLAR umas nas outras, e o giro total foi {atrito}°"
    );
}

/// ⭐⭐ **E as bolas NASCEM DENTRO da taça** — a armadilha que a `=114` nomeia por escrito: um
/// recipiente projecta para dentro tudo o que nasce fora, e a cena abriria com um SALTO antes da
/// queda, a cada volta do laço.
#[test]
fn every_ball_of_the_third_row_starts_inside_its_bowl() {
    let c = corre(0.0, |_, _| {});
    for (i, x) in [(TACA_GELO, -VAO), (TACA_ATRITO, VAO)] {
        let longe = c.inicio[i]
            .iter()
            .map(|q| (q[0] - x - TACA[0]).hypot(q[1] - TACA[1]))
            .fold(0.0_f32, f32::max);
        let cabe = TACA_R - TACA_RAIO;
        eprintln!("  taca {i} │ a mais longe nasce a {longe:.3}, e cabe ate' {cabe:.3}");
        assert!(
            longe < cabe,
            "uma bola nasce FORA da taca ({longe:.3} contra {cabe:.3}) e sera' projectada para \
             dentro no primeiro tique"
        );
    }
}

/// ⭐⭐⭐ **A FAIXA DO SALTO CHEGA AO PRODUTO, DE PONTA A PONTA** — quanto mais salto, mais alto a
/// bola volta, e o tecto entrega o máximo.
///
/// ⚠️ **A escada tem TRÊS degraus e não dois**: com dois, uma lei que saturasse a meio passaria o
/// gate se o degrau de baixo fosse `0`. Os três exigem que cada um seja estritamente mais alto que
/// o anterior.
///
/// ⚠️⚠️ **Os degraus derivam do [`ph2d_nodegraph::attr::BOUNCE_MAX`], nunca de literais.** A 1.ª
/// redacção era `[0, 1, BOUNCE_MAX]` — ela media *«o tecto novo passa do antigo»* quando o tecto
/// era `2`, e no dia em que o dono o reverteu para `1` (2026-09-15) os dois degraus de cima
/// **colapsaram no mesmo número** e o gate passou a exigir que uma altura fosse maior que ela
/// própria. *Uma escada escrita com o valor de ontem mede o produto de ontem.*
#[test]
fn the_bounce_reaches_the_new_ceiling_in_the_scene() {
    let tecto = ph2d_nodegraph::attr::BOUNCE_MAX;
    let alturas: Vec<f32> = [0.0_f32, tecto * 0.75, tecto]
        .into_iter()
        .map(|e| {
            corre(2.0, |state, formas| {
                state
                    .doc
                    .graph
                    .set_param(formas[QUEDA_VIVA], param::BOUNCE, e);
            })
            .pico_depois_do_toque[QUEDA_VIVA]
        })
        .collect();
    eprintln!(
        "  faixa do salto │ 0 → {:.3} · {:.2} → {:.3} · {tecto} → {:.3}",
        alturas[0],
        tecto * 0.75,
        alturas[1],
        alturas[2]
    );
    assert!(
        alturas[0] < alturas[1] && alturas[1] < alturas[2],
        "a altura tem de subir com o salto, monotonamente: {alturas:?}"
    );
    // ⭐ E a faixa chega ao produto de PONTA A PONTA: o tecto devolve o DOBRO do que o zero dá.
    // ⚠️ A barra é contra o `0`, e não contra o degrau do meio: numa faixa de `0..1` a resposta
    // satura perto do topo (medido: `0,9 → 0,572` e `1,0 → 0,510`), e exigir margem ali mediria a
    // saturação em vez da faixa.
    assert!(
        alturas[2] > alturas[0] * 2.0,
        "o tecto tem de devolver o dobro do que o zero da': {alturas:?}"
    );
}

/// **SONDA — a partir de que `Rolling Friction` a bola TRAVA na rampa** (doc 109 §7.10).
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_the_rolling_ball_on_the_ramp -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_the_rolling_ball_on_the_ramp() {
    eprintln!("\n  Rolling │ desceu (unidades em 2 s)   [rampa de 12°, tan = 0,213]");
    for rolar in [0.0_f32, 0.1, 0.2, 0.25, 0.3, 0.5, 1.0, 1.5] {
        let c = corre(2.0, |s, formas| {
            s.doc.graph.set_param(formas[1], param::ROLLING, rolar);
        });
        let (a, b) = (c.inicio[1][0], c.fim[1][0]);
        let d = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        eprintln!("  {rolar:>7.2} │ {d:>8.3}");
    }
}

/// ⭐⭐⭐ **O `Rolling Friction` TRAVA a bola na própria rampa** (doc 109 §7.10) — a ponta que o §7.7
/// nomeava (*«ela rola para sempre»*), agora com o número do PRODUTO e não só o da lei.
///
/// ⭐⭐ **E o valor a que ela trava não foi escolhido: é `tan(12°) = 0,213`**, a condição clássica de
/// equilíbrio ao rolamento num plano inclinado (`μr ≥ tan θ`). Medido na cena (sonda
/// `probe_the_rolling_ball_on_the_ramp`): `0,20 → desce 0,170` · **`0,25 → 0,087`** · `1,50 → 0,086`
/// — ou seja, ela prende entre `0,20` e `0,25`, exactamente onde a conta manda, e daí para cima o
/// número não muda mais nada.
///
/// ⚠️ **A régua é a DISTÂNCIA percorrida, não o ângulo:** uma bola travada ainda assenta os `0,02`
/// de folga com que nasce pousada, e isso são `0,086` que não são descida nenhuma.
#[test]
fn the_rolling_friction_locks_the_ball_on_the_ramp() {
    let desceu = |rolar: f32| {
        let c = corre(2.0, |s, formas| {
            s.doc.graph.set_param(formas[1], param::ROLLING, rolar);
        });
        let (a, b) = (c.inicio[1][0], c.fim[1][0]);
        ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
    };
    let solta = desceu(0.0);
    assert!(
        solta > 1.0,
        "o controlo: sem rolamento a bola desce a rampa toda, e desceu {solta}"
    );
    let presa = desceu(0.5);
    assert!(
        presa < 0.15,
        "com `Rolling Friction 0,5` (acima de tan 12° = 0,213) ela tem de FICAR onde está, \
         e andou {presa}"
    );
    assert!(
        solta > presa * 5.0,
        "e a diferença tem de ser à vista: {solta} contra {presa}"
    );
}
