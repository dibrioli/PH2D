//! ⭐⭐⭐ **Smoke da VIGIA DO CONTADOR.** `PH2D_COUNTERWATCH_SMOKE=1`.
//!
//! # O que esta cena prova
//!
//! Até 2026-09-17 este app sabia **contar** (o `Counter`), sabia **somar** (`AddToCounter`) e sabia
//! **mostrar** a conta (o `UiLabel` do HUD) — e **nada reagia a um número**. Três vidas chegavam a
//! zero e o jogo não sabia.
//!
//! ```text
//!   HEROI    ▪▪▪ ─► ▪▪ ─► ▪ ─► ·   três regras no MESMO contador, uma por luz
//!            3     2     1    0    ──► a última diz «morri» ──► ele DESAPARECE
//!
//!   CONTROLO ▪▪▪ ─► ▪▪▪ ─► ▪▪▪     o contador dele desce igual e NENHUMA luz se apaga,
//!            3     2      1        porque não há quem REAJA ao número
//! ```
//!
//! # ⭐⭐ As TRÊS luzes são três regras, e é isso que a cena mostra de graça
//!
//! O herói tem **três** vigias sobre o mesmo contador, em limiares diferentes (`≤2`, `≤1`, `≤0`),
//! cada uma a apagar uma luz. ⇒ vê-se, sem ler um número, que uma vigia é *«quando este número
//! chegar a N»* e não *«quando ele mudar»*.
//!
//! # ⭐⭐⭐ O CONTROLO é metade da cena, e é ele que a torna legível
//!
//! Os dois objectos são **iguais em tudo** — mesmo contador, mesmo relógio, mesma tabela de acções
//! — menos numa linha: o da direita **não tem a vigia**. Sem ele, um dono que visse o herói
//! desaparecer não teria como saber se foi a vigia ou qualquer outra coisa da cena.
//!
//! # ⚠️ Os dois contadores têm NOMES DIFERENTES, e isso é load-bearing
//!
//! Um contador é somado **por nome em toda a cena** (a lei da porta [`ph2d_ecs::counter::soma`]) —
//! é o que faz duas moedas chamadas `moedas` valerem duas. ⇒ chamar os dois `vidas` daria **6**, e
//! nenhum dos dois chegaria a zero: a cena ensinaria que o componente não funciona.
//!
//! # ⚠️ O que provar
//!
//! - **O placar da esquerda desce `3 · 2 · 1 · 0`** e, ao chegar a `0`, o quadrado azul **some**.
//! - **O da direita desce igual e o quadrado cinzento fica lá.**
//! - ⭐ **Escolha o herói e role o painel até *Counter Watch*:** a regra está lá, e o campo *Now*
//!   mostra o número a descer ao vivo.
//! - ⭐⭐ **Escreva um `d` a mais no nome do contador.** A linha fica **laranja**, o título diz
//!   `(1 broken)`, e o painel escreve *«There is no counter called…»* — que é o defeito mais caro
//!   deste componente a dizer-se em voz alta em vez de ficar mudo.
//!
//! ⚠️ Se a linha `[counterwatch-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Compare, Counter, CounterRuntime, CounterWatch, CounterWatchRow, Name, SignalAction,
    SignalActions, SignalVerb, Timer, Timers, Transform, Visibility,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// O contador do herói.
pub const HEROI: &str = "vidas";
/// ⚠️ **O do controlo tem NOME PRÓPRIO** — ver o cabeçalho: contadores homónimos SOMAM.
pub const CONTROLO: &str = "vidas_controlo";
/// Quantas vidas cada um começa com.
pub const VIDAS: i64 = 3;
/// De quanto em quanto tempo se perde uma vida, em microssegundos.
const GOLPE_US: u64 = 1_200_000;
/// O prefixo do nome de cada luz de vida — o alvo que a tabela de acções nomeia.
pub const HEROI_LUZ: &str = "Vida do heroi ";
/// Ver [`HEROI_LUZ`].
pub const CONTROLO_LUZ: &str = "Vida do controlo ";

/// Uma regra *«quando as vidas do herói chegarem a `limiar`, diz `sinal`»*.
fn regra(limiar: i64, sinal: &str) -> CounterWatchRow {
    CounterWatchRow {
        counter: HEROI.into(),
        compare: Compare::AtMost,
        value: limiar,
        signal: sinal.into(),
        once: true,
        scope: Default::default(),
    }
}

/// Uma linha *«ao ouvir `sinal`, esconde o objecto chamado `alvo`»*.
fn apaga(alvo: &str, sinal: &str) -> SignalAction {
    SignalAction {
        on: sinal.into(),
        target: alvo.into(),
        verb: SignalVerb::Hide,
        ..SignalAction::default()
    }
}

fn relogio(sinal: &str) -> Timers {
    Timers(vec![Timer {
        name: "golpe".into(),
        duration_us: GOLPE_US,
        repeat: true,
        autostart: true,
        signal: sinal.into(),
    }])
}

/// Uma linha *«ao ouvir S, soma `-1` ao meu contador»*.
fn tira_uma_vida(sinal: &str) -> SignalAction {
    SignalAction {
        on: sinal.into(),
        // ⚠️ **Alvo vazio = ESTE objecto** — a lei da tabela do #5.
        target: String::new(),
        verb: SignalVerb::AddToCounter,
        arg: "-1".into(),
        ..SignalAction::default()
    }
}

pub fn counter_watch_smoke(cx: &mut crate::scene_ctx::SceneCtx) {
    let (heroi, controlo) = montar(cx.sim.world_mut());
    eprintln!(
        "[counterwatch-smoke] heroi «{HEROI}» e controlo «{CONTROLO}», {VIDAS} vidas cada, \
         uma por {:.1} s — o da ESQUERDA tem a vigia · {heroi:?} / {controlo:?}",
        GOLPE_US as f64 / 1e6
    );
}

/// ⭐⭐ **Monta a cena no mundo e devolve `(herói, controlo)`.**
///
/// ⚠️ **Ela existe separada da [`counter_watch_smoke`] para os GATES a poderem chamar** — a lei que
/// a cena da fábrica escreveu: *o oráculo de uma cena é o que ela MONTA, e isso mede-se sem janela
/// nenhuma*.
pub fn montar(world: &mut bevy_ecs::world::World) -> (ph2d_ecs::Entity, ph2d_ecs::Entity) {
    // ── O CHÃO, PRIMEIRO ─────────────────────────────────────────────────────
    // ⚠️ **A lei da ordem de raiz** (15/09): o `assign_missing_root_order` congela as raízes pela
    // ordem de criação, logo o fundo tem de nascer ANTES de tudo o que se vê por cima dele.
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, -2.6)),
        Sprite::atlas(WHITE_TILE_KEY, [14.0, 0.5], [0.18, 0.18, 0.22, 1.0]),
        Name::new("Chao"),
    ));

    // ── AS LUZES DE VIDA ─────────────────────────────────────────────────────
    // ⚠️ **Nascem antes dos donos** (a lei da ordem de raiz) e **com `Visibility` explícita**: o
    // verbo `Hide` lê a visibilidade de ANTES para a declarar ao ledger, logo um alvo sem o
    // componente é **inerte** — o defeito que o gate da corrente apanhou.
    for (lado, x0, cor) in [
        (HEROI_LUZ, -3.2_f32, [0.30, 0.75, 1.0, 1.0]),
        (CONTROLO_LUZ, 3.2_f32, [0.70, 0.70, 0.72, 1.0]),
    ] {
        for i in 0..VIDAS {
            #[allow(clippy::cast_precision_loss)]
            let dx = (i as f32 - 1.0) * 0.55;
            world.spawn((
                Transform::from_translation(Vec2::new(x0 + dx, 1.5)),
                Sprite::atlas(WHITE_TILE_KEY, [0.38, 0.38], cor),
                Name::new(format!("{lado}{}", i + 1)),
                Visibility::visible(),
            ));
        }
    }

    // ── O HERÓI: três vidas, e TRÊS REGRAS sobre o mesmo contador ────────────
    let heroi = world
        .spawn((
            Transform::from_translation(Vec2::new(-3.2, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.6, 1.6], [0.30, 0.55, 0.95, 1.0]),
            Name::new("Heroi"),
            // ⛔⛔⛔ **A `Visibility` EXPLÍCITA, e ela não é decoração — o gate da corrente apanhou-a.**
            // O verbo `Hide` lê a visibilidade de ANTES para a declarar ao ledger, logo um alvo **sem**
            // o componente é **inerte**: a vigia falava, o «morri» chegava, e o herói ficava lá. ⚠️ O
            // relatório da tabela conta-o como `inert` e **não grita** — de propósito, porque uma
            // configuração a meio não é uma avaria. ⇒ quem quer ser escondido nasce com o componente,
            // que é o que a cena irmã do TOP-20 #5 já faz.
            Visibility::visible(),
            Counter {
                name: HEROI.into(),
                start: VIDAS,
                keep_on_restart: false,
            },
            CounterRuntime { value: VIDAS },
            relogio("golpe_heroi"),
            SignalActions(vec![
                tira_uma_vida("golpe_heroi"),
                // ⭐ Uma luz por regra — o alvo é o NOME, a lei da tabela do #5.
                apaga(&format!("{HEROI_LUZ}3"), "luz3"),
                apaga(&format!("{HEROI_LUZ}2"), "luz2"),
                apaga(&format!("{HEROI_LUZ}1"), "morri"),
                // ⭐⭐⭐ **O elo que esta wave trouxe:** a vigia diz «morri», e a tabela esconde-o.
                SignalAction {
                    on: "morri".into(),
                    target: String::new(),
                    verb: SignalVerb::Hide,
                    ..SignalAction::default()
                },
                // ⭐⭐ **O MESMO sinal move DOIS verbos** — e esta linha nasceu de uma MEDIÇÃO, não de
                // um capricho: sem ela o relógio continuava a bater depois da morte e o contador do
                // herói descia para `−1`, `−2`, `−3`… *Um morto que continua a perder vidas ensina o
                // contrário do que a cena diz.* Com ela, ele **congela em `0`**.
                SignalAction {
                    on: "morri".into(),
                    target: String::new(),
                    verb: SignalVerb::StopTimer,
                    ..SignalAction::default()
                },
            ]),
            // ⭐⭐⭐ **TRÊS REGRAS sobre o MESMO contador**, em limiares diferentes. ⚠️ `once` ligado
            // nas três: cada luz apaga-se uma vez. Sem ele uma regra voltaria a falar se o contador
            // subisse e descesse outra vez — o certo para *«dez moedas»* e o errado para *«morreste»*.
            CounterWatch(vec![regra(2, "luz3"), regra(1, "luz2"), regra(0, "morri")]),
        ))
        .id();

    // ── O CONTROLO: tudo igual, MENOS a vigia ────────────────────────────────
    let controlo = world
        .spawn((
            Transform::from_translation(Vec2::new(3.2, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.6, 1.6], [0.55, 0.55, 0.58, 1.0]),
            Name::new("Controlo (sem vigia)"),
            // ⚠️ **Ele também nasce com ela**, e é obrigatório: se só o herói a tivesse, o controlo
            // ficaria visível **por lhe faltar o componente** em vez de por lhe faltar a vigia — e a
            // cena provaria a coisa errada.
            Visibility::visible(),
            Counter {
                name: CONTROLO.into(),
                start: VIDAS,
                keep_on_restart: false,
            },
            CounterRuntime { value: VIDAS },
            relogio("golpe_controlo"),
            // ⛔⛔⛔ **A linha de esconder é a MESMA, e o NOME que ela espera é OUTRO — e isto foi
            // MEDIDO, não escolhido.** Com os dois à escuta de `"morri"`, o controlo **desaparecia
            // junto com o herói**: um sinal é **global por NOME** e uma tabela reage a ele venha de
            // quem vier. ⇒ o controlo espera por `"morri_controlo"`, que **ninguém nesta cena diz** —
            // porque quem o diria era a vigia que ele não tem.
            //
            // ⭐ *É esta a forma certa do controlo:* ele tem tudo o que o herói tem — relógio, contador,
            // linha de esconder — e o que lhe falta é **quem DIGA que ele morreu**.
            // ⭐⭐ **A MESMA fiação, palavra por palavra** — três linhas a apagar três luzes e uma a
            // esconder-se —, e os nomes que elas esperam **ninguém os diz nesta cena**: quem os diria
            // era a vigia que ele não tem.
            SignalActions(vec![
                tira_uma_vida("golpe_controlo"),
                apaga(&format!("{CONTROLO_LUZ}3"), "luz3_controlo"),
                apaga(&format!("{CONTROLO_LUZ}2"), "luz2_controlo"),
                apaga(&format!("{CONTROLO_LUZ}1"), "morri_controlo"),
                SignalAction {
                    on: "morri_controlo".into(),
                    target: String::new(),
                    verb: SignalVerb::Hide,
                    ..SignalAction::default()
                },
            ]),
        ))
        .id();

    // ⛔⛔⛔ **NÃO HÁ PLACAR DE TEXTO, e a foto foi quem o decidiu.** Um `UiLabel` **troca o que
    // um texto MOSTRA; ele não cria o texto** — sem um `VecShape::Text` autorado por baixo, o
    // rótulo desenha-se como um **anel vazio**, e um anel que ninguém sabe explicar ensina menos
    // que nada. ⇒ o número vive onde esta wave o pôs: a linha *Now* da secção do Inspector.
    //
    // ⭐ E as três LUZES dizem-no sem texto nenhum — que é melhor: elas mostram que uma vigia
    // dispara **a um limiar**, e um número a descer não mostraria isso.

    (heroi, controlo)
}

#[cfg(test)]
#[path = "counter_watch_smoke_tests.rs"]
mod tests;
