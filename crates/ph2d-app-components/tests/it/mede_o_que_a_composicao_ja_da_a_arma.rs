//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA da wave da ARMA** (`WeaponFire`, §6 do
//! levantamento, e o item que o `#14 ProjectileMotion` deixou **ABERTO** por escrito): *a
//! composição de hoje — o gatilho + a fábrica + o relógio + o contador + a tabela de acções — já
//! exprime «a arma do jogador»?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime.** Nesta linha a pergunta já REESCREVEU seis entregas — o **#3 `SensorZone`** estava
//! fechado por composição, a **W5 do #21** descobriu que o pintor já existia, o **#24 `Health`**
//! deixou de ser um componente, o **#23** confirmou que o concorrente era real e não chegava, o
//! **#14** descobriu que o ricochete já era exacto, e o **FIM DE JOGO** achou um buraco de um verbo.
//!
//! ⚠️ **O lado medido NÃO é um espantalho.** É a melhor composição que esta casa tem hoje, e ela
//! usa **cinco** componentes que esta mesma linha shipou: `SignalOnAction` (#18, 18/09) ·
//! `Factory` + `aim_from_spawner` (#11, e a mira foi acrescentada pelo #18) · `Timers` (#2) ·
//! `Counter` (#20) · `SignalActions` (#5).
//!
//! ⚠️ **Ela mora AQUI, na crate de FAMÍLIA, porque esta é a única que vê os dois lados**: o
//! `ph2d-ecs` (a fábrica, o relógio, o contador, os verbos) e o `ph2d-physics-ecs` (o
//! `ProjectileMotion`, que é a bala).
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME.
//!
//! ```text
//! cargo test -p ph2d-app-components --test it \
//!     mede_o_que_a_composicao_ja_da_a_arma -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **A MIRA** — a bala sai apontada para onde o herói aponta? (a metade que já fechou)
//! B) **A CADÊNCIA** — segurar o gatilho dá quantos tiros por segundo?
//! C) **A CADÊNCIA PELO RELÓGIO** — o desvio que um artista escreveria, e o que ele custa.
//! D) **AS MUNIÇÕES** — a arma pára quando o pente acaba?
//! E) **A RECARGA** — existe verbo que REPONHA um contador?
//! F) **A DISPERSÃO** — duas balas da mesma rajada podem sair em ângulos diferentes?

use ph2d_ecs::{
    ActionEdge, ActionSample, ActionTriggerRow, Counter, CounterRuntime, Factory, FactoryRuntime,
    Name, SignalVerb, SimWorld, StableId, Timer, TimerRuntime, Timers, Transform, tick_factories,
};
use ph2d_tags::TagTree;

/// O passo fixo da casa.
const DT_US: u64 = 1_000_000 / 60;
/// Um quarto de segundo entre tiros — a cadência que um artista escreveria.
const CADENCIA_US: u64 = 250_000;
/// O pente.
const PENTE: i64 = 6;
/// A identidade da receita.
const MESTRE: u64 = 7;

/// A receita da bala — um mestre com `ProjectileMotion`, como o `#14` a autora.
fn bala(sim: &mut SimWorld) -> u64 {
    sim.world_mut().spawn((
        Name::new("Bala"),
        StableId(MESTRE),
        ph2d_physics_ecs::ProjectileMotion {
            initial_speed: 12.0,
            face_velocity: true,
            ..Default::default()
        },
    ));
    MESTRE
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0, não afirma uma barra"]
#[allow(clippy::too_many_lines)]
fn mede_o_que_a_composicao_ja_da_a_arma() {
    eprintln!("\n════ §5.0 — O QUE A COMPOSIÇÃO JÁ DÁ À ARMA DO JOGADOR ════\n");

    let tags = TagTree::default();

    // ── (A) A MIRA ────────────────────────────────────────────────────────────────────────────
    //
    // O herói aponta a 90° (para cima) e a fábrica mora nele. A pergunta é se a cópia nasce
    // apontada — a metade que o gatilho (#18) fechou.
    let mut sim = SimWorld::new();
    let master = bala(&mut sim);
    let angulo = std::f32::consts::FRAC_PI_2;
    sim.world_mut().spawn((
        Name::new("Heroi"),
        StableId(1),
        Transform {
            translation: ph2d_core::Vec2::new(2.0, 0.0),
            rotation: angulo,
            ..Default::default()
        },
        Factory {
            master,
            on_signal: "fire".to_owned(),
            aim_from_spawner: true,
            ..Default::default()
        },
        FactoryRuntime::default(),
    ));
    let t = tick_factories(sim.world_mut(), &tags, &["fire"]);
    eprintln!("(A) A MIRA — a bala sai apontada?");
    eprintln!(
        "    nascimentos ..................... {}\n    \
         a pose .......................... {:?}\n    \
         o ângulo ........................ {:?}  (o herói aponta {angulo:.4})",
        t.births.len(),
        t.births.first().map(|b| b.at),
        t.births.first().and_then(|b| b.aim)
    );
    eprintln!(
        "    ⭐ FECHADO: o `aim_from_spawner` nasceu com o gatilho (#18), e o `ProjectileMotion`\n\
         \x20      converte `initial_speed` + o ângulo do corpo numa velocidade, uma vez, no lançamento.\n"
    );

    // ── (B) A CADÊNCIA, pela rota directa ─────────────────────────────────────────────────────
    //
    // A linha do gatilho é `Hold` (é o que um artista escreve para «segurar para disparar») e ela
    // fala em TODO tique enquanto a tecla está em baixo.
    let segurada = ActionSample {
        pressed: true,
        just_pressed: false,
        just_released: false,
    };
    let linha = ActionTriggerRow {
        action: "fire".to_owned(),
        edge: ActionEdge::Hold,
        signal: "fire".to_owned(),
    };
    let fala_por_tique = ph2d_ecs::signal_on_action::fala(&linha, segurada);
    let mut nascidas = 0usize;
    for _ in 0..60 {
        let fired: Vec<&str> = if fala_por_tique {
            vec!["fire"]
        } else {
            Vec::new()
        };
        nascidas += tick_factories(sim.world_mut(), &tags, &fired).births.len();
    }
    eprintln!("(B) A CADÊNCIA — segurar o gatilho um segundo");
    eprintln!("    a linha `Hold` fala por tique ... {fala_por_tique}");
    eprintln!("    balas num segundo ............... {nascidas}");
    eprintln!(
        "    ⛔ **{nascidas} tiros por segundo.** Não há cadência: a fábrica nasce a cada sinal e o\n\
         \x20      gatilho fala a cada tique. Uma arma é, antes de tudo, um RITMO.\n"
    );

    // ── (C) O DESVIO PELO RELÓGIO ─────────────────────────────────────────────────────────────
    //
    // O que um artista escreveria para curar (B): carregar ARRANCA um relógio repetidor, largar
    // PÁRA-o, e é o relógio que alimenta a fábrica. Medimos o que isso custa.
    let mut sim2 = SimWorld::new();
    let m2 = bala(&mut sim2);
    let arma = sim2
        .world_mut()
        .spawn((
            Name::new("Heroi"),
            StableId(1),
            Transform::default(),
            Timers(vec![Timer {
                name: "cadence".to_owned(),
                duration_us: CADENCIA_US,
                repeat: true,
                autostart: false,
                signal: "shot".to_owned(),
            }]),
            TimerRuntime::default(),
            Factory {
                master: m2,
                on_signal: "shot".to_owned(),
                ..Default::default()
            },
            FactoryRuntime::default(),
        ))
        .id();
    ph2d_ecs::reconcile_timers(sim2.world_mut());
    // o dedo carrega no tique 0 e não larga
    {
        let mundo = sim2.world_mut();
        let mut rt = mundo.get_mut::<TimerRuntime>(arma).unwrap();
        ph2d_ecs::timer_start(&mut rt.0[0]);
    }
    let mut primeira: Option<u32> = None;
    let mut total = 0usize;
    for q in 0..60u32 {
        let mut disparos: Vec<String> = Vec::new();
        {
            let mundo = sim2.world_mut();
            let cfg = mundo.get::<Timers>(arma).unwrap().clone();
            let mut rt = mundo.get_mut::<TimerRuntime>(arma).unwrap();
            for (t, st) in cfg.0.iter().zip(rt.0.iter_mut()) {
                if ph2d_ecs::timer_advance(t, st, DT_US).fires > 0 {
                    disparos.push(t.signal.clone());
                }
            }
        }
        let refs: Vec<&str> = disparos.iter().map(String::as_str).collect();
        let n = tick_factories(sim2.world_mut(), &tags, &refs).births.len();
        if n > 0 && primeira.is_none() {
            primeira = Some(q);
        }
        total += n;
    }
    eprintln!("(C) A CADÊNCIA PELO RELÓGIO — o desvio que cura (B)");
    eprintln!(
        "    o custo em autoria .............. 1 componente + 2 linhas de tabela (Start/Stop)"
    );
    eprintln!(
        "    a PRIMEIRA bala sai no tique .... {:?}  ({:.3} s depois de carregar)",
        primeira,
        f64::from(primeira.unwrap_or(0)) / 60.0
    );
    eprintln!("    balas num segundo ............... {total}  (o pedido eram 4)");
    eprintln!(
        "    ⛔ **A primeira bala chega ATRASADA um período inteiro.** Um relógio dispara no FIM\n\
         \x20      do período e uma arma dispara no INÍCIO: carregar e não acontecer nada durante um\n\
         \x20      quarto de segundo lê-se como um botão que não funciona.\n"
    );

    // ── (D) AS MUNIÇÕES ───────────────────────────────────────────────────────────────────────
    //
    // O pente é um `Counter`, e a tabela desconta um por tiro. A pergunta é se alguma coisa liga o
    // contador à fábrica.
    let mut sim3 = SimWorld::new();
    let m3 = bala(&mut sim3);
    let dono = sim3
        .world_mut()
        .spawn((
            Name::new("Heroi"),
            StableId(1),
            Transform::default(),
            Counter {
                name: "ammo".to_owned(),
                start: PENTE,
            },
            CounterRuntime { value: PENTE },
            Factory {
                master: m3,
                on_signal: "fire".to_owned(),
                ..Default::default()
            },
            FactoryRuntime::default(),
        ))
        .id();
    let tiros = 10usize;
    let mut saidas = 0usize;
    for _ in 0..tiros {
        saidas += tick_factories(sim3.world_mut(), &tags, &["fire"])
            .births
            .len();
        // a linha da tabela: `on fire → Add to Counter (-1)`
        let mundo = sim3.world_mut();
        let mut c = mundo.get_mut::<CounterRuntime>(dono).unwrap();
        c.value -= 1;
    }
    let restante = ph2d_ecs::counter::soma(sim3.world(), "ammo");
    eprintln!("(D) AS MUNIÇÕES — a arma pára quando o pente acaba?");
    eprintln!("    o pente ......................... {PENTE}");
    eprintln!("    gatilhos ........................ {tiros}");
    eprintln!("    balas que SAÍRAM ................ {saidas}");
    eprintln!("    o contador depois ............... {restante:?}");
    eprintln!(
        "    ⛔ **A arma dispara {saidas} balas de um pente de {PENTE}, e o contador vai a NEGATIVO.**\n\
         \x20      Nada liga o contador à fábrica: o `alive_max`/`total_max` dela contam a CORRIDA\n\
         \x20      inteira, e um pente é um número que se REPÕE.\n"
    );

    // ── (E) A RECARGA ─────────────────────────────────────────────────────────────────────────
    let escrevem: Vec<&str> = SignalVerb::ALL
        .iter()
        .filter(|v| matches!(v, SignalVerb::AddToCounter))
        .map(|v| v.label())
        .collect();
    eprintln!("(E) A RECARGA — existe verbo que REPONHA um contador?");
    eprintln!(
        "    verbos ao todo .................. {}",
        SignalVerb::ALL.len()
    );
    eprintln!("    os que escrevem um contador ..... {escrevem:?}");
    eprintln!(
        "    ⛔ **Um só, e ele SOMA.** Recarregar é pôr o pente no cheio, e somar o tamanho do pente\n\
         \x20      só acerta quando ele está a ZERO — recarregar com três balas dentro dá NOVE.\n\
         \x20      ⇒ a recarga é INEXPRIMÍVEL por qualquer caminho de hoje.\n"
    );

    // ── (F) A DISPERSÃO ───────────────────────────────────────────────────────────────────────
    let mut sim4 = SimWorld::new();
    let m4 = bala(&mut sim4);
    sim4.world_mut().spawn((
        Name::new("Cacadeira"),
        StableId(1),
        Transform::default(),
        Factory {
            master: m4,
            on_signal: "fire".to_owned(),
            aim_from_spawner: true,
            burst: 3,
            ..Default::default()
        },
        FactoryRuntime::default(),
    ));
    let t4 = tick_factories(sim4.world_mut(), &tags, &["fire"]);
    let angulos: Vec<Option<f32>> = t4.births.iter().map(|b| b.aim).collect();
    eprintln!("(F) A DISPERSÃO — três balas da mesma rajada");
    eprintln!("    os ângulos ...................... {angulos:?}");
    eprintln!(
        "    ⛔ **Os três são o MESMO.** Uma rajada sai como uma bala só, e uma caçadeira é\n\
         \x20      exactamente a diferença.\n"
    );

    eprintln!("════ VEREDITO ════");
    eprintln!(
        "A MIRA está fechada (o #18 pagou-a). O que falta é tudo o que faz de um spawner uma ARMA:\n\
         o RITMO (60 tiros/s, ou o primeiro atrasado), o PENTE que acaba e se RECARREGA, e o\n\
         ESPALHAMENTO. São três perguntas que a composição não responde — e as três são do MESMO\n\
         objecto, que é o que faz disto um componente e não três."
    );
}
