//! Os gates da ponte do `Timer` — o que a lei pura não pode medir sozinha.
//!
//! ⚠️ **A lei pura tem os dela** ([`ph2d_ecs::timer`]): esta é a metade que só existe com um MUNDO
//! — a reconciliação dos dois vectores, o silêncio de um timer sem nome, e a propriedade pela qual
//! a wave inteira escolheu o desenho que escolheu.

use super::{start_autostart_timers, tick_timers};
use ph2d_ecs::{Name, SimWorld, Timer, TimerRuntime, Timers, Transform};

/// Um passo fixo de 1/60 s — o mesmo do produto.
const DT: f64 = 1.0 / 60.0;

/// O snapshot que o undo fotografa — **a porta do produto**, com o registo completo.
///
/// ⚠️ **A régua tem de ser esta e não o componente**: um componente não registado é descartado
/// por construção, e é isso que se está a medir. *Perguntar ao componente mediria a minha
/// intenção; perguntar ao snapshot mede o que o undo vê.*
fn snapshot(sim: &mut SimWorld) -> ph2d_ecs::scene::WorldSnapshot {
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut out = ph2d_ecs::scene::WorldSnapshot::default();
    ph2d_ecs::scene::world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &reg, &mut out)
        .expect("o snapshot do mundo de teste");
    out
}

/// ⚠️⚠️ **A entidade nasce SEM relógio, e isso é a correcção de 2026-09-08.** A 1.ª redacção
/// semeava um `TimerRuntime::default()` aqui, e com isso a fixtura fabricava metade do estado que
/// o produto tinha de produzir — *uma fixtura que arma o que o produto não arma mede a minha
/// intenção*. É assim que uma entidade sai do load, da paleta, de uma cópia e da cena de smoke.
fn mundo_com(timers: Vec<Timer>) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Alvo"), Timers(timers)))
        .id();
    (sim, e)
}

fn um(duration_us: u64, repeat: bool, signal: &str) -> Timer {
    Timer {
        name: "t".into(),
        duration_us,
        repeat,
        autostart: true,
        signal: signal.into(),
    }
}

/// **O relógio corre e o sinal sai** — o circuito inteiro, do tique à lista de sinais.
#[test]
fn a_timer_fires_through_the_bridge() {
    let (mut sim, e) = mundo_com(vec![um(100_000, true, "batida")]);
    start_autostart_timers(&mut sim);
    // 6 tiques de 1/60 s = 100 ms = exactamente um periodo.
    let sinais = tick_timers(&mut sim, 6, DT);
    assert_eq!(sinais.len(), 1, "o timer nao falou");
    assert_eq!(sinais[0].name, "batida");
    assert_eq!(sinais[0].fires, 1);
    assert_eq!(sinais[0].entity, e);
}

/// ⛔ **Um timer sem nome de sinal cumpre o período e CALA-SE.**
///
/// A lei da §11 — *um produtor sem nome não fala, em vez de falar com um nome vazio.* ⚠️ O gate da
/// lei pura mede o CAMPO; este mede a ponte, que é quem decide publicar.
#[test]
fn a_timer_with_no_signal_name_stays_silent_through_the_bridge() {
    let (mut sim, _) = mundo_com(vec![um(100_000, true, "")]);
    start_autostart_timers(&mut sim);
    assert!(
        tick_timers(&mut sim, 60, DT).is_empty(),
        "um timer sem nome falou — e o nome com que ele falaria seria vazio"
    );
}

/// ⭐⭐⭐ **UM TIMER A CORRER NÃO É UM PASSO DE UNDO** — a razão inteira do desenho desta wave.
///
/// O relógio muda a **60 Hz**. Se ele vivesse no componente registado, o diff por-quadro do
/// `post_frame_undo` veria o mundo mudar sem entrada nenhuma e **cada quadro viraria um passo** —
/// exactamente a família que a auditoria da §11 mediu e não curou, com o `SpriteAnimator` a pagá-la.
///
/// ⚠️ **A régua é o SNAPSHOT, não o componente**: é ele que a captura fotografa, e um componente
/// não registado é descartado por construção. *Perguntar ao componente mediria a minha intenção;
/// perguntar ao snapshot mede o que o undo vê.*
///
/// **Mutação que deve sangrar:** registar o `TimerRuntime` em `register_ecs_components`.
#[test]
fn a_running_timer_never_reaches_the_undo_snapshot() {
    let (mut sim, _) = mundo_com(vec![um(100_000, true, "batida")]);
    start_autostart_timers(&mut sim);

    let antes = snapshot(&mut sim);
    // Meio periodo: o relogio ANDOU, e nao ha' disparo nenhum a confundir a leitura.
    let sinais = tick_timers(&mut sim, 3, DT);
    assert!(
        sinais.is_empty(),
        "a fixtura disparou — ela nao mede o RELOGIO a andar"
    );
    let depois = snapshot(&mut sim);

    assert_eq!(
        antes, depois,
        "o relogio de um timer chegou ao snapshot do undo: a 60 Hz, CADA QUADRO viraria um passo \
         que o artista nao deu"
    );
}

/// ⚠️ **E o CONTROLO da régua acima**: o que o artista autora **chega** ao snapshot.
///
/// Sem ele, um `Timers` que também não fosse registado passaria o gate anterior — e a duração, o
/// nome e o sinal evaporavam no primeiro `Ctrl+Z`. *Um zero de «não medido» e um de «correcto» são
/// o mesmo byte.*
#[test]
fn what_the_artist_authors_does_reach_the_snapshot() {
    let (mut sim, e) = mundo_com(vec![um(100_000, true, "batida")]);
    let antes = snapshot(&mut sim);
    if let Some(mut ts) = sim.world_mut().get_mut::<Timers>(e) {
        ts.0[0].duration_us = 250_000;
    }
    let depois = snapshot(&mut sim);
    assert_ne!(
        antes, depois,
        "mexer na DURACAO nao chegou ao snapshot — o que o artista autora tem de sobreviver ao \
         Ctrl+Z e ao ficheiro"
    );
}

/// **O runtime segue o comprimento da config** — acrescentar e remover timers pelo painel.
///
/// ⚠️ **Os dois vectores ligam-se pelo ÍNDICE, nunca pelo nome**: um nome é editável a meio, e o
/// relógio saltaria de timer debaixo da mão do artista.
#[test]
fn the_runtime_follows_the_config_length() {
    let (mut sim, e) = mundo_com(vec![um(100_000, true, "a")]);
    start_autostart_timers(&mut sim);
    if let Some(mut ts) = sim.world_mut().get_mut::<Timers>(e) {
        ts.0.push(um(200_000, true, "b"));
        ts.0.push(um(300_000, true, "c"));
    }
    let _ = tick_timers(&mut sim, 1, DT);
    assert_eq!(
        sim.world().get::<TimerRuntime>(e).expect("runtime").0.len(),
        3,
        "o relogio nao acompanhou os timers novos — eles nunca correriam"
    );
    if let Some(mut ts) = sim.world_mut().get_mut::<Timers>(e) {
        ts.0.truncate(1);
    }
    let _ = tick_timers(&mut sim, 1, DT);
    assert_eq!(
        sim.world().get::<TimerRuntime>(e).expect("runtime").0.len(),
        1,
        "o relogio ficou com estado de timers que ja' nao existem"
    );
}

/// ⭐⭐ **O `autostart` CRIA o relógio numa entidade que veio do ficheiro.**
///
/// O `Timers` viaja no `.ph2dproj` e o `TimerRuntime` **não**. Sem esta metade, um projecto
/// reaberto teria os timers todos mudos — e a query do tique nem sequer os veria.
#[test]
fn loading_a_project_gives_the_timers_a_clock() {
    let mut sim = SimWorld::default();
    // Como uma entidade sai do restore: com a CONFIG e sem relógio nenhum.
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Timers(vec![um(100_000, true, "x")])))
        .id();
    assert!(sim.world().get::<TimerRuntime>(e).is_none());
    start_autostart_timers(&mut sim);
    let rt = sim.world().get::<TimerRuntime>(e).expect(
        "o load nao deu relogio ao timer — a query do tique nem o veria, e o projecto reabria mudo",
    );
    assert!(rt.0[0].running, "o autostart nao armou o timer");
    assert_eq!(tick_timers(&mut sim, 6, DT).len(), 1, "ele nao correu");
}

/// ⭐⭐⭐ **O GATE QUE FALTAVA — ninguém arma nada à mão, e o timer corre na mesma.**
///
/// Todos os outros gates deste ficheiro chamam [`start_autostart_timers`] antes de tiquear, e por
/// isso ficaram **verdes** sobre um produto em que **nenhum caminho de quadro a chamava**: o único
/// chamador era o load. Medido em 2026-09-08, `PH2D_TIMER_SMOKE=1` montava a cena e produzia
/// **zero** sinais em 15 s, e um `Timers` anexado pela paleta nascia inerte para sempre.
///
/// ⇒ este gate faz o que o artista faz: põe a config na entidade e **não faz mais nada**.
///
/// **Mutação que deve sangrar:** apagar a chamada a `start_autostart_timers` no topo do
/// [`tick_timers`].
#[test]
fn a_timer_that_nobody_armed_by_hand_still_runs() {
    let (mut sim, e) = mundo_com(vec![um(100_000, true, "batida")]);
    assert!(
        sim.world().get::<TimerRuntime>(e).is_none(),
        "a fixtura ja' trazia relogio — ela nao mede o que a paleta produz"
    );
    // ⛔ Nenhuma chamada a `start_autostart_timers`: quem tem de a fazer e' o produto.
    let sinais = tick_timers(&mut sim, 6, DT);
    assert_eq!(
        sinais.len(),
        1,
        "um Timers posto na entidade e mais nada NAO correu — e' o estado em que a paleta, a \
         copia e a cena de smoke deixam toda entidade"
    );
    assert_eq!(sinais[0].name, "batida");
}

/// ⛔⛔ **E a ponte NÃO re-arma um *one-shot* que terminou** — a metade que a cura podia partir.
///
/// A forma barata de curar o gate acima seria chamar a lei antiga («arma todo slot que não corre»)
/// por quadro; ela é satisfeita por um one-shot terminado, que passaria a **disparar para sempre**.
/// É o `Recarga` da cena de smoke, e é o que o dono vê primeiro.
///
/// **Mutação que deve sangrar:** trocar a condição de nascimento por `if !s.running` no
/// `ph2d_ecs::timer_reconcile`.
#[test]
fn the_bridge_never_re_arms_a_finished_one_shot() {
    let (mut sim, _) = mundo_com(vec![um(100_000, false, "arma_pronta")]);
    let mut total = 0usize;
    // Dez segundos de quadros a 1/60 s, com o periodo em 100 ms: cem oportunidades de repetir.
    for _ in 0..600 {
        total += tick_timers(&mut sim, 1, DT).len();
    }
    assert_eq!(
        total, 1,
        "o one-shot falou {total} vezes — 'uma vez' e' a unica coisa que ele promete"
    );
}

/// ⭐ **Um timer ACRESCENTADO a um objecto vivo começa a correr**, sem tocar no vizinho.
///
/// É o `+` do painel de timers (W3) medido na ponte: o relógio já existe, e o que nasce é **um
/// slot**. Sem esta metade o segundo timer de um objecto seria inerte enquanto o primeiro corre.
#[test]
fn a_timer_appended_to_a_live_object_starts_running() {
    let (mut sim, e) = mundo_com(vec![um(100_000, true, "a")]);
    assert_eq!(
        tick_timers(&mut sim, 6, DT).len(),
        1,
        "o primeiro nao correu"
    );
    if let Some(mut ts) = sim.world_mut().get_mut::<Timers>(e) {
        ts.0.push(um(100_000, true, "b"));
    }
    let nomes: Vec<String> = tick_timers(&mut sim, 6, DT)
        .into_iter()
        .map(|s| s.name)
        .collect();
    assert!(
        nomes.iter().any(|n| n == "b"),
        "o timer acrescentado nasceu inerte; sairam {nomes:?}"
    );
}
