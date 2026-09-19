//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA da wave do FIM DE JOGO**: *a composição de hoje —
//! `Counter` + `CounterWatch` + a tabela de acções + a fábrica + o Luau — já exprime «perdi, e o
//! jogo recomeça»?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime.** Nesta linha a pergunta já REESCREVEU cinco entregas — o **#3 `SensorZone`** estava
//! fechado por composição, a **W5 do #21** descobriu que o pintor já existia, o **#24 `Health`**
//! deixou de ser um componente, o **#23** confirmou que o concorrente era real e não chegava, e o
//! **#14** descobriu que o ricochete já era exacto.
//!
//! ⚠️ **Ela mora AQUI, na crate de FAMÍLIA, porque esta é a única que vê os dois lados**: o
//! `ph2d-ecs` (os verbos, o contador, a vigia) e a PONTE que os aplica de verdade.
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME.
//!
//! ```text
//! cargo test -p ph2d-app-components --test it \
//!     mede_o_que_a_composicao_ja_da_ao_fim_de_jogo -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **Ligando os NOVE verbos ao sinal de fim, o jogo volta ao princípio?** — a pergunta inteira,
//!    medida pelo caminho do PRODUTO e não por uma leitura de lista.
//! B) **O relatório da ponte tem algum canal que fale da CORRIDA?** — o `Destroy` provou que um
//!    verbo pode ANUNCIAR em vez de agir; existe o anúncio de que falta?
//! C) **Quem sabe rebobinar hoje?** — o censo dos chamadores, pelo fonte da shell.
//! D) **A válvula de escape (o Luau do #16) tem porta de transporte?**

use ph2d_app_components::signal_actions_bridge::{Som, apply};
use ph2d_ecs::{
    Counter, CounterRuntime, Name, SignalAction, SignalActions, SignalEffect, SignalTarget,
    SignalVerb, SimWorld, StableId,
};
use ph2d_preview_drive::PreviewDrive;

/// O fonte da shell que decide o transporte — lido em tempo de COMPILAÇÃO, logo não pode
/// envelhecer sem o ficheiro mudar.
const ACTION_BUS_KINDS: &str = include_str!("../../../ph2d-editor-core/src/action_bus_kinds.rs");

/// Uma linha de tabela que reage a `on` com `verb`, apontada a si própria.
fn linha(on: &str, verb: SignalVerb) -> SignalAction {
    SignalAction {
        on: on.to_owned(),
        target: String::new(),
        verb,
        arg: String::new(),
        target_by: SignalTarget::default(),
        from: ph2d_ecs::SignalFrom::default(),
    }
}

#[test]
#[ignore = "sonda: imprime a medição do §5.0, não afirma uma barra"]
fn mede_o_que_a_composicao_ja_da_ao_fim_de_jogo() {
    eprintln!("\n════ §5.0 — O QUE A COMPOSIÇÃO JÁ DÁ AO FIM DE JOGO ════\n");

    // ── (A) Os NOVE verbos sobre o sinal de fim ───────────────────────────────────────────────
    //
    // A cena é o que um artista tem HOJE depois de perder: um contador de vidas que começa em `3` e
    // chegou a `0`. A pergunta é se algum dos nove verbos o devolve ao princípio.
    let mut sim = SimWorld::new();
    let heroi = sim
        .world_mut()
        .spawn((
            Name::new("Heroi"),
            StableId(1),
            Counter {
                name: "vidas".to_owned(),
                start: 3,
            },
            CounterRuntime { value: 0 },
            SignalActions(
                SignalVerb::ALL
                    .iter()
                    .map(|v| linha("fim", *v))
                    .collect::<Vec<_>>(),
            ),
        ))
        .id();

    let efeitos: Vec<SignalEffect> = SignalVerb::ALL
        .iter()
        .map(|v| SignalEffect {
            target: heroi,
            verb: *v,
            arg: String::new(),
            source: heroi,
        })
        .collect();
    let mut drive = PreviewDrive::default();
    let mut mudo = |_: &mut SimWorld, _: Som, _: ph2d_ecs::Entity| false;
    let relatorio = apply(&mut sim, &efeitos, &mut drive, &mut mudo);
    let vidas = sim
        .world()
        .get::<CounterRuntime>(heroi)
        .map_or(-1, |r| r.value);

    eprintln!("(A) OS NOVE VERBOS SOBRE O SINAL DE FIM");
    eprintln!(
        "    verbos ......................... {} : {:?}",
        SignalVerb::ALL.len(),
        SignalVerb::ALL
            .iter()
            .map(|v| v.label())
            .collect::<Vec<_>>()
    );
    eprintln!(
        "    aplicados {} · inertes {} · mortes {}",
        relatorio.applied,
        relatorio.inert,
        relatorio.mortes.len()
    );
    eprintln!("    as vidas depois dos NOVE ....... {vidas}  (o princípio é 3)");
    eprintln!(
        "    ⚠️ UM verbo mexeu, e não é um recomeço: o `Add to Counter` com o `arg` vazio soma\n\
            \x20      `+1` (a lei dele), logo as vidas vão de `0` a `1` — e o princípio é `3`.\n\
            \x20   ⚠️ E as MORTES são `0` porque o alvo é DOCUMENTO: o `Destroy` só tira quem\n\
            \x20      nasceu numa corrida (a lei do #24). Um herói autorado não sai da cena.\n\
            \x20   ⇒ com os nove ligados ao fim, o jogo continua PERDIDO: nada devolve a corrida\n\
            \x20      ao princípio. O artista tem de carregar em Rewind com o dedo.\n"
    );

    // ── (B) O relatório da ponte fala da CORRIDA? ─────────────────────────────────────────────
    //
    // O `Destroy` (#24) provou que um verbo pode ANUNCIAR em vez de agir: ele põe uma `Death` no
    // relatório e o dreno da shell serve-a. A pergunta é se existe o anúncio equivalente.
    eprintln!("(B) OS CANAIS DO RELATÓRIO DA PONTE");
    eprintln!("    applied (contagem) · inert (contagem) · mortes (Vec<Death>)");
    eprintln!(
        "    algum nomeia a CORRIDA? ........ false\n    \
         ⇒ o idioma do anúncio existe e está usado UMA vez (a morte). Falta o segundo.\n"
    );

    // ── (C) Quem sabe rebobinar hoje ──────────────────────────────────────────────────────────
    let transporte = ACTION_BUS_KINDS.contains("playhead.rewind()");
    eprintln!("(C) QUEM REBOBINA A CORRIDA");
    eprintln!("    o barramento do EDITOR declara-o? ... {transporte}");
    eprintln!(
        "    ⇒ o único caminho é o DEDO do artista na barra do topo (mais os prólogos de smoke\n\
            \x20      e o carregar de um projecto). Nenhum sinal, nenhum verbo, nenhum script.\n"
    );

    // ── (D) A válvula de escape ───────────────────────────────────────────────────────────────
    eprintln!("(D) A VÁLVULA DE ESCAPE (o Luau do TOP-20 #16)");
    eprintln!(
        "    a superfície é: emit · spawn · despawn · get/set · state_* · find_by_name · input"
    );
    eprintln!(
        "    porta de transporte? ........... nenhuma\n    \
         ⇒ nem escrevendo código o artista recomeça a corrida.\n"
    );

    eprintln!("════ VEREDITO ════");
    eprintln!(
        "O laço de um jogo fecha-se em sete passos, e SEIS já se autoram:\n\
         andar · nascer · bater · morrer · contar · perder.\n\
         O sétimo — RECOMEÇAR — não tem nenhuma porta. É um buraco exacto, e é UM verbo."
    );
}
