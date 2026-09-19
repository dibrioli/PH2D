//! ⭐⭐⭐ **A PONTE do `SignalActions`** — onde um sinal deixa de ser um toast e vira jogo
//! (TOP-20 #5).
//!
//! # O que ela fecha
//!
//! O `ph2d-runtime` publica sinais de **cinco** origens (timeline · contacto · sensor · animação ·
//! relógio) e, até 2026-09-09, tinha **três** consumidores — um toast, uma linha de terminal e a
//! máquina de estados de UI. Todos de diagnóstico. *Nada na cena reagia a um sinal.*
//!
//! # ⚠️ As DUAS metades, e porque a fronteira está onde está
//!
//! A [`ph2d_ecs::resolve_signal_actions`] é pura sobre o mundo: ela casa nomes, resolve alvos e
//! devolve efeitos. **Este** ficheiro aplica-os, e a razão de ele existir separado é o **undo**:
//!
//! - **Um relógio** vive no `TimerRuntime`, que **não é componente registado** ⇒ arrancá-lo não é
//!   um passo de `Ctrl+Z`, e não precisa de declaração nenhuma.
//! - **A visibilidade** vive na [`ph2d_ecs::Visibility`], que **é** registada ⇒ sem o ledger,
//!   **cada porta que abre seria um passo de undo**. É a mesma lei que o `preview_drive` já
//!   escreve: *o documento é o valor AUTORADO; o que um motor escreve agora é pré-visualização*.
//!
//! ⇒ os dois verbos escrevem por portas diferentes de propósito, e a diferença **não é
//! arbitrária** — ela é a de o componente estar ou não no registo.
//!
//! # ⚠️ QUANDO, no quadro
//!
//! Depois do dreno do outbox (senão os sinais deste quadro chegariam ao próximo) e antes do
//! `post_frame_undo` (senão a escrita não seria fotografada nem declarada). É a mesma janela que o
//! toast usa, e é por isso que ela é lida **do mesmo cursor de leitura** — um segundo `SignalReader`
//! ao lado, com o seu próprio cursor.

use ph2d_ecs::{SignalEffect, SignalVerb, SimWorld, TimerRuntime, Timers, Visibility};

use ph2d_preview_drive::{Driven, PreviewDrive};

/// ⭐⭐⭐ **UM SINAL LIDO, com quem o disse** — `(nome, quem falou, o outro lado)`.
///
/// # ⛔⛔ Ela tem NOME porque era aqui que a origem morria
///
/// Até 2026-09-19 a fase fazia `.map(|s| s.name.to_string())`, e o `SignalOrigin` — que tem catorze
/// variantes, **onze** delas com `source`, e um `Contact` com `source` **e** `other` — era
/// descartado nessa linha. *O dado era construído, publicado, lido e deitado fora uma linha antes de
/// ser preciso.* Com um nome, um gate consegue apanhá-la; dentro do `map` da fase, não.
///
/// ⚠️ **Quem responde «quem falou?» é a porta do barramento** ([`ph2d_runtime::SignalOrigin::quem`]),
/// cujo `match` é EXAUSTIVO: um `match` escrito aqui esqueceria a décima quinta origem em silêncio.
///
/// ⚠️ **`try_from_bits` e não `from_bits`:** os bits vêm de um sinal publicado num quadro anterior.
/// Esta metade confere a CODIFICAÇÃO; a **liveness** é conferida do outro lado, no `targets_of`,
/// porque o bevy recicla bits de entidades despawnadas.
pub fn lido(
    s: &ph2d_runtime::Signal,
) -> (String, Option<ph2d_ecs::Entity>, Option<ph2d_ecs::Entity>) {
    let de =
        |b: Option<ph2d_runtime::EntityBits>| b.and_then(|b| ph2d_ecs::Entity::try_from_bits(b.0));
    (
        s.name.to_string(),
        de(s.origin.quem()),
        de(s.origin.outro()),
    )
}

/// ⭐⭐ **O que a tabela pede ao SOM** — o vocabulário da injecção (ver [`apply`]).
///
/// ⛔ **Um enum e não um `bool`:** *«toca»* e *«cala»* são dois verbos autorados, e um booleano
/// chamado `play` no sítio da chamada lê-se ao contrário com a mesma facilidade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Som {
    /// Toca o som do alvo.
    Toca,
    /// Cala o que o alvo tem a soar.
    Cala,
}

/// O que uma aplicação fez — para o log de diagnóstico, e para os gates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActionReport {
    /// Quantos efeitos chegaram a mexer em alguma coisa.
    pub applied: usize,
    /// Quantos não tinham onde pegar (o alvo não tem o componente que o verbo escreve).
    ///
    /// ⚠️ **Não é um erro, e é por isso que ele é CONTADO e não gritado:** ligar um sinal a um
    /// objecto sem relógio é uma configuração a meio, não uma avaria — e o painel é quem tem de o
    /// dizer, não um toast por quadro.
    pub inert: usize,
    /// ⭐⭐⭐ **Quem o [`SignalVerb::Destroy`] mandou sair** (suplente #24).
    ///
    /// ⚠️ **Ela sai daqui em vez de a ponte apagar**, e é a lei do despachante único: *«quando é
    /// que isto sai da cena?»* é uma pergunta só, e quem responde é o dreno da
    /// `fase_fabrica_e_morte` da shell, uma vez por quadro, depois de todos os produtores.
    pub mortes: Vec<ph2d_ecs::Death>,
}

/// ⭐⭐ **Aplica os efeitos deste quadro.**
///
/// ⚠️ **O `drive` é obrigatório na assinatura**, e não um `Option`: uma função-irmã «sem ledger»
/// seria a segunda porta pela qual o defeito volta — exactamente a lei que a `ProjectState::capture`
/// já escreve para si mesma.
///
/// # ⛔⛔ E o SOM entra por INJECÇÃO, e não por um `&mut AudioSystem`
///
/// Esta crate é uma **FAMÍLIA** e o `ph2d-app-audio` também: o gate
/// `architecture_no_dependency_climbs_a_layer` recusa a aresta, e a cura que ele PRESCREVE é
/// *«uma tabela para ser injectada pela composição»*. ⇒ a tabela de acções não sabe o que é um
/// mixer; ela sabe pedir *«toca o som deste objecto»*, e quem responde é a shell, que é dona dos
/// dois lados.
///
/// ⚠️ **O fecho devolve `false` quando não havia o que tocar** — a mesma leitura de sempre: um
/// editor sem dispositivo de som corre em silêncio de propósito (o `AudioSystem::new` devolve
/// `None` e a casa degrada como faz com o `gilrs`), e isso conta como **inerte**, nunca como erro.
pub fn apply(
    sim: &mut SimWorld,
    effects: &[SignalEffect],
    drive: &mut PreviewDrive,
    som: &mut dyn FnMut(&mut SimWorld, Som, ph2d_ecs::Entity) -> bool,
) -> ActionReport {
    let mut report = ActionReport::default();
    for fx in effects {
        let ok = match fx.verb {
            SignalVerb::StartTimer => set_timers(sim, fx, true),
            SignalVerb::StopTimer => set_timers(sim, fx, false),
            SignalVerb::Show => set_visible(sim, drive, fx, Some(false)),
            SignalVerb::Hide => set_visible(sim, drive, fx, Some(true)),
            SignalVerb::ToggleVisibility => set_visible(sim, drive, fx, None),
            // ⭐⭐⭐ **O SOM** (TOP-20 #4) — o verbo que a recusa deste enum nomeava como
            // inalcançável até 2026-09-09. ⚠️ `as_deref_mut` porque o laço passa por aqui N vezes
            // e um `Option<&mut _>` não é `Copy`.
            SignalVerb::PlaySound => som(sim, Som::Toca, fx.target),
            SignalVerb::StopSound => som(sim, Som::Cala, fx.target),
            // ⭐⭐⭐ **O PLACAR** (TOP-20 #20) — é este verbo que faz «bateu na moeda → +1 ponto»
            // fechar com o `SignalOnHit` que a física já publica, sem uma linha do artista.
            SignalVerb::AddToCounter => add_to_counter(sim, fx),
            // ⭐⭐⭐ **O VERBO QUE TIRA DA CENA** (suplente #24) — e ele **anuncia**, nunca apaga.
            SignalVerb::Destroy => match morte(sim, fx) {
                Some(d) => {
                    report.mortes.push(d);
                    true
                }
                None => false,
            },
        };
        if ok {
            report.applied += 1;
        } else {
            report.inert += 1;
        }
    }
    report
}

/// ⭐⭐⭐ **O facto de morte que o [`SignalVerb::Destroy`] produz** — ou `None`, e aí é inerte.
///
/// # ⚠️⚠️ A fronteira é FORÇADA, não escolhida
///
/// Ele só tira quem [`ph2d_ecs::is_transient`] — quem **nasceu numa corrida**. A razão não é
/// preferência: apagar um objecto do DOCUMENTO durante a corrida **tira-o do documento** (a captura
/// vê-o sumido e o `Ctrl+Z` herda a remoção), que é a lei *«o que acontece numa corrida não é
/// documento»* invertida. O precedente é do TOP-20 #14, com a frase inteira escrita na
/// `fase_fabrica_e_morte` da shell: *«um projéctil que ele pôs na cena à mão é documento, e apagá-lo
/// destruiria autoria»*. Este é o **quarto** leitor daquela porta.
///
/// # ⭐⭐ E a recusa NÃO vira aviso de linha no painel, com mecanismo
///
/// A linha `Destroy` de um objecto de documento **está certa**: ela é a receita, e as CÓPIAS que a
/// fábrica dela põe na cena são transientes. *O mesmo alvo muda de resposta entre o original e a
/// cópia* ⇒ um aviso na linha diria que está partido exactamente o que está a funcionar. O que
/// sobra é o [`ActionReport::inert`] e o diagnóstico, que é onde ele pertence.
fn morte(sim: &SimWorld, fx: &SignalEffect) -> Option<ph2d_ecs::Death> {
    if !ph2d_ecs::is_transient(sim.world(), fx.target) {
        if std::env::var_os("PH2D_SIGNAL_LOG").is_some() {
            let nome = ph2d_ecs::signal_actions::name_of(sim.world(), fx.target)
                .unwrap_or_else(|| "?".to_string());
            eprintln!(
                "[signal] Destroy recusado em «{nome}»: e' do DOCUMENTO (so' sai quem nasceu numa corrida)"
            );
        }
        return None;
    }
    Some(ph2d_ecs::Death {
        entity: fx.target,
        // ⚠️ **Calada**: o sinal de morte é assunto do `Lifetime`, que já o autora. Inventar um
        // aqui seria um segundo campo para a mesma coisa — a frase que a ponte do projéctil já tem.
        signal: String::new(),
        why: ph2d_ecs::DeathCause::Killed,
    })
}

/// **Soma ao contador do alvo.** `arg` vazio ou ilegível = `1`.
///
/// ⚠️ **Escreve no [`ph2d_ecs::CounterRuntime`], nunca na config** — o valor vivo de um contador
/// não é documento (ver o doc do `Counter`), logo isto NÃO passa pelo ledger e NÃO entra no undo.
/// *É a mesma fronteira que o `TimerRuntime` já tinha; a diferença é que aqui ela é a feature.*
///
/// ⚠️ **Somar `0` é INERTE**, e o relatório conta-o como tal: um valor que não move nada não pode
/// ler-se como aplicado. Um alvo sem `Counter` também é inerte — e o painel di-lo.
fn add_to_counter(sim: &mut SimWorld, fx: &ph2d_ecs::SignalEffect) -> bool {
    let quanto: i64 = {
        let t = fx.arg.trim();
        if t.is_empty() {
            1
        } else {
            t.parse().unwrap_or(1)
        }
    };
    if quanto == 0 {
        return false;
    }
    let world = sim.world_mut();
    if world.get::<ph2d_ecs::Counter>(fx.target).is_none() {
        return false;
    }
    let inicio = world
        .get::<ph2d_ecs::Counter>(fx.target)
        .map_or(0, |c| c.start);
    let mut ent = match world.get_entity_mut(fx.target) {
        Ok(e) => e,
        Err(_) => return false,
    };
    if let Some(mut rt) = ent.get_mut::<ph2d_ecs::CounterRuntime>() {
        rt.value = rt.value.saturating_add(quanto);
        if std::env::var_os("PH2D_SIGNAL_LOG").is_some() {
            eprintln!("[signal] contador += {quanto} -> {}", rt.value);
        }
    } else {
        // ⚠️ O vivo NASCE do `start` da config, e não de zero — a mesma lei do rebobinar.
        ent.insert(ph2d_ecs::CounterRuntime {
            value: inicio.saturating_add(quanto),
        });
    }
    true
}

/// Arranca ou pára os timers do alvo. `arg` vazio = **todos**; senão, os que têm aquele nome.
///
/// ⚠️ **A lei de arrancar e de parar vive no `ph2d-ecs`** ([`ph2d_ecs::timer_start`] /
/// [`ph2d_ecs::timer_stop`]), e não aqui: *arrancar é do princípio, parar guarda o progresso*, e
/// escrevê-la neste ficheiro daria uma segunda resposta ao lado da que o `autostart` já usa.
///
/// ⚠️ **Um nome que não casa com timer nenhum devolve `false`** — é a configuração a meio que o
/// [`ActionReport::inert`] conta.
fn set_timers(sim: &mut SimWorld, fx: &SignalEffect, start: bool) -> bool {
    let Some(timers) = sim.world().get::<Timers>(fx.target).cloned() else {
        return false;
    };
    // ⚠️ **Os índices saem da CONFIG antes de tocar no relógio**: o `Timers` e o `TimerRuntime`
    // ligam-se pelo índice, e ler os dois ao mesmo tempo pediria dois empréstimos do mundo.
    let alvos: Vec<usize> = timers
        .0
        .iter()
        .enumerate()
        .filter(|(_, t)| fx.arg.is_empty() || t.name == fx.arg)
        .map(|(i, _)| i)
        .collect();
    if alvos.is_empty() {
        return false;
    }
    let Some(mut rt) = sim.world_mut().get_mut::<TimerRuntime>(fx.target) else {
        // O relógio ainda não nasceu — ele nasce na reconciliação do quadro seguinte, e o sinal
        // deste quadro perde-se. ⚠️ Só acontece no quadro em que o componente é anexado.
        return false;
    };
    let mut tocou = false;
    for i in alvos {
        let Some(s) = rt.0.get_mut(i) else { continue };
        if start {
            ph2d_ecs::timer_start(s);
        } else {
            ph2d_ecs::timer_stop(s);
        }
        tocou = true;
    }
    tocou
}

/// Escreve a visibilidade do alvo **pelo ledger**. `None` = inverter o que está lá.
///
/// ⚠️⚠️ **As DUAS metades são obrigatórias**, e esquecer a segunda é o defeito silencioso: escrever
/// sem `driven` faria a `settle` do fim do quadro esquecer o facto, e o valor de pré-visualização
/// viraria **documento** — que é exactamente o passo de undo que este caminho existe para não ter.
fn set_visible(
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    fx: &SignalEffect,
    hide: Option<bool>,
) -> bool {
    let Some(antes) = sim.world().get::<Visibility>(fx.target).map(|v| v.hidden) else {
        return false;
    };
    let depois = hide.unwrap_or(!antes);
    // ⚠️ **Declara-se mesmo sem mudança**, e é deliberado: a `settle` esquece quem **não** foi
    // declarado neste quadro, então calar-se num quadro em que o valor coincide devolveria o
    // facto ao documento no meio de uma corrida.
    drive.driven(fx.target, Driven::Visible(antes), Driven::Visible(depois));
    Driven::Visible(depois).write(sim, fx.target);
    true
}

/// Os gates das LEIS desta ponte — módulo irmão, por tecto de LOC.
#[cfg(test)]
#[path = "signal_actions_bridge_leis_tests.rs"]
mod tests;

/// ⭐ Os gates do VERBO QUE TIRA DA CENA e da ORIGEM que atravessa a leitura (suplente #24).
#[cfg(test)]
#[path = "signal_actions_bridge_tests.rs"]
mod destroy_tests;
