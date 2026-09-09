//! **Os gates da costura da secção TIMERS** — irmão de [`super::inspector_timer`] por CAP de LOC.
//!
//! ⚠️ **Só a shell vê as duas metades**: o painel é chrome e não depende do motor, e a lei do
//! relógio vive no `ph2d-ecs`. É aqui que se prova que o snapshot diz o que a cena tem e que o
//! commit escreve o que o clique pediu.

use super::*;
use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands, register_ecs_components};
use ph2d_ecs::{TimerRuntime, Transform};

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_render::register_render_components(&mut r);
    r
}

fn um(name: &str, duration_us: u64, signal: &str) -> Timer {
    Timer {
        name: name.into(),
        duration_us,
        repeat: true,
        autostart: true,
        signal: signal.into(),
    }
}

/// Um objecto com os timers dados. ⚠️ **Sem `TimerRuntime`** — é assim que a paleta o entrega.
fn objecto(sim: &mut SimWorld, timers: Vec<Timer>) -> Entity {
    sim.world_mut()
        .spawn((Transform::default(), Timers(timers)))
        .id()
}

fn edit(
    sim: &mut SimWorld,
    e: Entity,
    reg: &ComponentRegistry,
    ed: TimerFieldEdit,
) -> Option<Toast> {
    let queue = EditorCommandQueue::new();
    let t = apply_timer_edit(sim, e.to_bits(), &ed, &queue, reg);
    apply_editor_commands(sim.world_mut(), &queue, reg).expect("o commit aplica");
    t
}

fn info(sim: &SimWorld, e: Entity) -> InspectorTimerInfo {
    build_timer_info(sim.world(), e.to_bits(), 1).expect("o objecto tem Timers")
}

/// ⛔ **Um objecto SEM `Timers` não tem secção** — é o ADR-0166: o Inspector mostra o que o objecto
/// TEM, e anexar é da paleta. Um `Some` aqui poria uma secção vazia em todo objecto do projecto.
#[test]
fn an_object_without_the_component_has_no_section() {
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn((Transform::default(),)).id();
    assert!(
        build_timer_info(sim.world(), e.to_bits(), 1).is_none(),
        "um objecto sem Timers recebeu a seccao — ela apareceria em TODO objecto, vazia"
    );
}

/// **O snapshot diz o que a cena tem, com a duração em SEGUNDOS.**
#[test]
fn the_snapshot_reports_the_component_in_seconds() {
    let mut sim = SimWorld::default();
    let e = objecto(&mut sim, vec![um("Batida", 1_500_000, "batida")]);
    let i = info(&sim, e);
    assert_eq!(i.rows.len(), 1);
    assert_eq!(i.rows[0].name, "Batida");
    assert!(
        (i.rows[0].duration_s - 1.5).abs() < 1e-6,
        "a duracao nao chegou em segundos: {}",
        i.rows[0].duration_s
    );
    assert_eq!(i.rows[0].signal, "batida");
    assert!(i.rows[0].will_ever_fire());
    assert!(!i.rows[0].is_mute());
}

/// ⭐⭐ **A conversão é uma IDA E VOLTA** — segundos entram, microssegundos ficam, segundos saem.
///
/// ⚠️ **É o gate que separa este ficheiro de dois ficheiros:** a conversão vive nas duas pontas
/// do MESMO módulo de propósito, e é esta prova que a mantém honesta. Com ela em dois sítios, o
/// número que o artista lê e o que o motor aplica divergem no dia em que uma metade for corrigida.
///
/// **Mutação que deve sangrar:** trocar o `US_PER_S` de uma das duas pontas por `1_000.0`.
#[test]
fn the_duration_survives_the_round_trip_through_the_engine() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![um("t", 0, "s")]);
    edit(&mut sim, e, &reg, TimerFieldEdit::DurationSecs(0, 2.25));
    assert_eq!(
        sim.world().get::<Timers>(e).expect("timers").0[0].duration_us,
        2_250_000,
        "o commit nao escreveu microssegundos"
    );
    assert!(
        (info(&sim, e).rows[0].duration_s - 2.25).abs() < 1e-6,
        "o snapshot nao devolveu segundos"
    );
}

/// **A duração SATURA no teto do motor**, em vez de dar a volta.
#[test]
fn a_duration_above_the_engine_ceiling_saturates() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![um("t", 0, "s")]);
    // Um dia inteiro, contra o teto de uma hora.
    edit(&mut sim, e, &reg, TimerFieldEdit::DurationSecs(0, 86_400.0));
    assert_eq!(
        sim.world().get::<Timers>(e).expect("timers").0[0].duration_us,
        TIMER_MAX_US,
        "a duracao passou o teto do motor"
    );
}

/// **O `+` cria um timer que já FUNCIONA** — um segundo, e não zero.
///
/// ⚠️ É a lei do `Default` do motor: `0` é o valor que NUNCA dispara, e um timer acabado de
/// acrescentar que nasce mudo lê-se como um componente partido — que é exactamente o report que
/// abriu esta wave.
#[test]
fn a_new_timer_is_born_able_to_fire() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![]);
    edit(&mut sim, e, &reg, TimerFieldEdit::Add);
    let i = info(&sim, e);
    assert_eq!(i.rows.len(), 1);
    assert!(
        i.rows[0].will_ever_fire(),
        "o timer acabado de criar nasceu inerte: {:?}",
        i.rows[0]
    );
    assert_eq!(i.rows[0].name, "Timer 1");
}

/// **Dois `+` dão dois nomes DIFERENTES** — a lista escolhe-se por nome.
#[test]
fn two_new_timers_do_not_share_a_name() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![]);
    edit(&mut sim, e, &reg, TimerFieldEdit::Add);
    edit(&mut sim, e, &reg, TimerFieldEdit::Add);
    let i = info(&sim, e);
    assert_eq!(i.rows.len(), 2);
    assert_ne!(
        i.rows[0].name, i.rows[1].name,
        "dois timers com o mesmo nome — a lista deixa de ser escolhivel"
    );
}

/// ⛔ **O `+` no tecto RECUSA COM VOZ** — um botão que não faz nada lê-se como partido.
#[test]
fn adding_past_the_cap_refuses_out_loud() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let cheio: Vec<Timer> = (0..TIMERS_MAX)
        .map(|n| um(&format!("t{n}"), 1_000_000, "s"))
        .collect();
    let e = objecto(&mut sim, cheio);
    let t = edit(&mut sim, e, &reg, TimerFieldEdit::Add);
    assert!(t.is_some(), "o tecto foi atingido em silencio");
    assert_eq!(
        sim.world().get::<Timers>(e).expect("timers").0.len(),
        TIMERS_MAX
    );
}

/// ⛔ **Um nome VAZIO é recusado com voz**; um nome de SINAL vazio é legítimo e passa.
///
/// ⚠️ As duas metades no mesmo gate porque a diferença entre elas é a lei: a lista escolhe-se pelo
/// nome do timer (uma linha em branco não se aponta), e um produtor sem nome de sinal cumpre o
/// período e cala-se — que é uma escolha, não um erro.
#[test]
fn an_empty_timer_name_is_refused_and_an_empty_signal_is_not() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![um("Batida", 1_000_000, "batida")]);

    let t = edit(&mut sim, e, &reg, TimerFieldEdit::Rename(0, String::new()));
    assert!(t.is_some(), "o nome vazio passou em silencio");
    assert_eq!(info(&sim, e).rows[0].name, "Batida", "o nome foi apagado");

    edit(&mut sim, e, &reg, TimerFieldEdit::Signal(0, String::new()));
    let i = info(&sim, e);
    assert!(i.rows[0].is_mute(), "o sinal vazio nao ficou");
    assert!(
        i.rows[0].will_ever_fire(),
        "calar um timer nao o pode parar"
    );
}

/// **Mexer num campo não repõe os outros** — ler-modificar-escrever, nunca reconstruir.
///
/// ⚠️ O commit escreve o componente INTEIRO (é o que o `SetComponent` faz), então reconstruí-lo do
/// snapshot perderia o que outra edição do mesmo quadro escreveu. Este gate é o que prende isso.
#[test]
fn editing_one_field_leaves_the_others_alone() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![um("Batida", 1_000_000, "batida")]);
    edit(&mut sim, e, &reg, TimerFieldEdit::Repeat(0, false));
    let i = info(&sim, e);
    assert!(!i.rows[0].repeat);
    assert_eq!(i.rows[0].name, "Batida", "o nome foi reposto");
    assert_eq!(i.rows[0].signal, "batida", "o sinal foi reposto");
    assert!(i.rows[0].autostart, "o autostart foi reposto");
    assert!((i.rows[0].duration_s - 1.0).abs() < 1e-6, "a duracao mudou");
}

/// **Uma edição num índice que já não existe não faz nada, e não estoura.**
///
/// ⚠️ É alcançável: o painel publica o índice aberto e a shell aplica no quadro seguinte — entre os
/// dois, um `Ctrl+Z` pode ter encolhido a lista.
#[test]
fn an_edit_on_a_vanished_index_is_a_no_op() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![um("Batida", 1_000_000, "batida")]);
    assert!(edit(&mut sim, e, &reg, TimerFieldEdit::Remove(7)).is_none());
    assert!(edit(&mut sim, e, &reg, TimerFieldEdit::DurationSecs(7, 5.0)).is_none());
    assert_eq!(info(&sim, e).rows.len(), 1);
}

/// ⭐⭐ **O painel NUNCA vê o relógio vivo** — nem o lê, nem o escreve.
///
/// ⚠️ É a razão inteira do desenho desta família, medida do lado do Inspector: se o `TimerRuntime`
/// entrasse no snapshot, o painel repintaria a 60 Hz e o pedido seguinte seria poder mexer nele —
/// que é o caminho de volta ao passo de undo por quadro.
///
/// **Mutação que deve sangrar:** pôr o `elapsed_us` no `InspectorTimerRow`.
#[test]
fn the_panel_never_touches_the_live_clock() {
    let mut sim = SimWorld::default();
    let reg = registry();
    let e = objecto(&mut sim, vec![um("Batida", 1_000_000, "batida")]);
    // O relógio existe e está a meio de um período.
    sim.world_mut()
        .entity_mut(e)
        .insert(TimerRuntime(vec![ph2d_ecs::TimerState {
            elapsed_us: 700_000,
            running: true,
        }]));
    let antes = sim
        .world()
        .get::<TimerRuntime>(e)
        .cloned()
        .expect("runtime");

    // Todo campo que a secção sabe escrever.
    edit(&mut sim, e, &reg, TimerFieldEdit::Rename(0, "Outro".into()));
    edit(&mut sim, e, &reg, TimerFieldEdit::DurationSecs(0, 3.0));
    edit(&mut sim, e, &reg, TimerFieldEdit::Repeat(0, false));
    edit(&mut sim, e, &reg, TimerFieldEdit::Autostart(0, false));
    edit(&mut sim, e, &reg, TimerFieldEdit::Signal(0, "outro".into()));

    assert_eq!(
        sim.world()
            .get::<TimerRuntime>(e)
            .cloned()
            .expect("runtime"),
        antes,
        "uma edicao do Inspector mexeu no relogio vivo — o proximo passo e' ele voltar ao undo"
    );
    let i = info(&sim, e);
    assert_eq!(i.rows[0].name, "Outro");
    assert!(
        !i.rows[0].will_ever_fire(),
        "o autostart desligado nao pegou"
    );
}

/// ⭐⭐ **Os ids do painel cobrem o cap do MODELO.**
///
/// ⚠️ *Um modelo que aceita o que o painel não mostra produz estado inalcançável por gesto nenhum*
/// — é a lei que o `ANIM_TAGS_MAX` pagou, com o número a descer de 256 para 64. Aqui os dois
/// nascem iguais, e este gate é o que os prende.
#[test]
fn the_timer_row_ids_cover_the_model_cap() {
    assert_eq!(
        ph2d_editor::ids::INSP_TIMER_ROW.len(),
        TIMERS_MAX,
        "o painel desenha {} linhas para um modelo de {TIMERS_MAX} — os timers a mais seriam \
         inalcancaveis",
        ph2d_editor::ids::INSP_TIMER_ROW.len(),
    );
}

/// **A faixa do campo de duração é a do MOTOR.**
///
/// ⚠️ O painel não depende do `ph2d-ecs` (ADR-0029), então ele escreve o número literal. Este gate
/// é o que impede o literal de envelhecer: mover o `TIMER_MAX_US` sem mover o campo daria um
/// slider que pára antes do que o motor aceita, ou que passa dele e satura em silêncio.
#[test]
fn the_timer_duration_range_is_the_engines() {
    // O literal que o `populate_timer` escreve, em segundos.
    const PANEL_MAX_S: u64 = 3600;
    assert_eq!(
        TIMER_MAX_US / 1_000_000,
        PANEL_MAX_S,
        "o teto do painel e o do motor discordam"
    );
}
