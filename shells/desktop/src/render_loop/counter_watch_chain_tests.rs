//! ⭐⭐⭐ **A CORRENTE INTEIRA, sem janela nenhuma** — a única lente que prova que *a sequência leva
//! a algum lado*.
//!
//! ```text
//!   relógio ──«golpe»──► AddToCounter(-1) ──► o contador desce ──► VIGIA ──«morri»──► Hide
//! ```
//!
//! # ⚠️ Porque ele vive DENTRO do `render_loop`
//!
//! Os elos estão em três crates e **a shell é a única que os vê todos**: o relógio e a resolução
//! são da `ph2d-ecs`, a vigia é da `ph2d-app-components`, e quem APLICA um verbo é o
//! `render_loop::signal_actions`, que é **privado ao `render_loop`**.
//!
//! ⛔ **A alternativa era alargar dois módulos a `pub(crate)`, e seria a cura errada:** este repo
//! já tem quatro agulhas que nomeiam a VISIBILIDADE em vez da lei. *Um gate que precisa de ver o
//! que só um módulo vê mora nesse módulo* — e assim ele continua a partir-se alto no dia em que
//! alguém mudar a ordem do quadro.
//!
//! # ⛔ E o CONTROLO é metade do gate
//!
//! Sem ele, um `Hide` que apanhasse os dois — ou um verbo que escondesse toda a cena — leria como
//! aprovação.

use ph2d_ecs::{CounterRuntime, SimWorld, Visibility};
use ph2d_preview_drive::PreviewDrive;
use ph2d_tags::TagTree;

use super::{signal_actions, timer_tick};

/// Um passo fixo de 60 Hz.
const DT: f64 = 1.0 / 60.0;

/// Corre `quadros` quadros da corrente inteira, na ORDEM do quadro real.
///
/// ⚠️ **A ordem é a do `fase_signal_outbox`**: os relógios falam, a vigia fala a seguir (ela vê o
/// mundo como o quadro anterior o deixou), e a tabela de acções lê os dois. Trocar duas linhas
/// aqui mediria um programa que não existe.
fn corre(sim: &mut SimWorld, quadros: u32) {
    // ⛔⛔⛔ **A IDENTIDADE, e ela não é um detalhe do arnês — foi este gate que a descobriu.**
    // O `resolve_signal_actions` consulta `(Entity, &SignalActions, &StableId)`, logo **um mundo
    // sem identidade não reage a sinal nenhum** e não o diz: os relógios falam, a resolução
    // devolve zero e o contador fica parado. É a mesma família do defeito que as Tags pagaram em
    // 2026-09-14 (*«um `try_query` num mundo sem `StableId` responde NINGUÉM»*).
    //
    // ⚠️ No app quem a atribui é a shell; um arnês que a saltasse mediria **outro programa**.
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let tags = TagTree::new();
    let mut drive = PreviewDrive::default();
    let mut pendentes: Vec<String> = Vec::new();
    for _ in 0..quadros {
        // 1. os relógios
        let mut nomes: Vec<String> = timer_tick::tick_timers(sim, 1, DT)
            .into_iter()
            .map(|s| s.name)
            .collect();
        // 2. a VIGIA — ela fala antes de a tabela ler.
        let f = ph2d_app_components::counter_watch_bridge::frame(sim, true, 1);
        nomes.extend(f.disparos.into_iter().map(|(_, _, n)| n));
        // 3. o que ficou do quadro anterior chega agora (a vigia publica no outbox).
        nomes.append(&mut pendentes);
        if nomes.is_empty() {
            continue;
        }
        let refs: Vec<&str> = nomes.iter().map(String::as_str).collect();
        let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), &tags, &refs);
        if !efeitos.is_empty() {
            signal_actions::apply(sim, &efeitos, &mut drive, None);
        }
    }
}

fn escondido(sim: &SimWorld, e: ph2d_ecs::Entity) -> bool {
    sim.world().get::<Visibility>(e).is_some_and(|v| v.hidden)
}

fn vidas(sim: &SimWorld, e: ph2d_ecs::Entity) -> i64 {
    sim.world()
        .get::<CounterRuntime>(e)
        .map_or(i64::MIN, |c| c.value)
}

/// ⭐⭐⭐ **O herói perde as três vidas e DESAPARECE; o controlo perde as três e FICA.**
///
/// ⚠️ **As quatro asserções são quatro defeitos diferentes**, e nenhuma sozinha prova a wave: o
/// contador do herói a chegar a zero prova o relógio e o `AddToCounter`; o do controlo prova que o
/// mecanismo é o mesmo nos dois; o herói escondido prova a vigia; e o controlo **visível** prova
/// que foi a vigia e não outra coisa qualquer da cena.
///
/// **Mutações que devem sangrar:** apagar a chamada à vigia no laço · `soma.unwrap_or(0)` ·
/// apagar o `once` · pôr a vigia também no controlo.
#[test]
fn o_heroi_some_ao_chegar_a_zero_e_o_controlo_fica() {
    let mut sim = SimWorld::new();
    let (heroi, controlo) = ph2d_app_components::counter_watch_smoke::montar(sim.world_mut());

    // Antes de correr: os dois estão vivos e com as vidas todas.
    assert_eq!(
        vidas(&sim, heroi),
        ph2d_app_components::counter_watch_smoke::VIDAS
    );
    assert!(!escondido(&sim, heroi), "o heroi nao pode nascer escondido");

    // 3 vidas × 1,2 s = 3,6 s ⇒ 216 quadros, mais folga para o `Hide` chegar.
    corre(&mut sim, 300);

    assert!(
        vidas(&sim, heroi) <= 0,
        "o heroi nao chegou a zero (esta' em {}) — o relogio ou o AddToCounter nao correram",
        vidas(&sim, heroi)
    );
    assert!(
        vidas(&sim, controlo) <= 0,
        "o CONTROLO nao chegou a zero (esta' em {}) — sem isso ele nao e' um controlo",
        vidas(&sim, controlo)
    );
    assert!(
        escondido(&sim, heroi),
        "o heroi chegou a zero e NAO desapareceu — a vigia nao falou, ou o «morri» nao chegou"
    );
    assert!(
        !escondido(&sim, controlo),
        "o CONTROLO desapareceu — entao nao foi a vigia que escondeu o heroi"
    );
}

/// ⭐⭐ **Com o relógio da cena PARADO, ninguém perde nada e ninguém some.**
///
/// ⚠️ É a metade negativa, e ela tem valor próprio: um editor parado não pode matar o herói do
/// artista enquanto ele autora.
#[test]
fn com_o_relogio_parado_nada_acontece() {
    let mut sim = SimWorld::new();
    let (heroi, controlo) = ph2d_app_components::counter_watch_smoke::montar(sim.world_mut());
    let tags = TagTree::new();
    for _ in 0..300 {
        // ⚠️ `playing = false` — os relógios nem sequer avançam, e a vigia não avalia.
        let f = ph2d_app_components::counter_watch_bridge::frame(&mut sim, false, 0);
        assert!(f.disparos.is_empty(), "a vigia falou com o relogio parado");
        let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), &tags, &[]);
        assert!(efeitos.is_empty());
    }
    assert_eq!(
        vidas(&sim, heroi),
        ph2d_app_components::counter_watch_smoke::VIDAS
    );
    assert!(!escondido(&sim, heroi));
    assert!(!escondido(&sim, controlo));
}
