//! **Fase do quadro: O OUTBOX DE SINAIS** — a §11 Animation, os timers e a física publicam no MESMO outbox
//! da timeline, o painel autorado publica os apertos dele, e o DRENO corre depois dos produtores: o toast,
//! a tabela nome → acção e o log (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **A ordem é a lei, e o gate dela lê o texto emendado do quadro**: o quadro de sinais vira na
//! `fase_timeline_containers`, a timeline publica na `fase_timeline_drain`, o mundo anda na
//! `fase_physics_step`, e só DEPOIS disto tudo esta fase publica e drena. Os dois `Vec` de sinais são os
//! que a `fase_fixed_step_clocks` produziu neste quadro, consumidos aqui.

use super::*;
use ph2d_i18n::tr_with;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_signal_outbox(
        &mut self,
        anim_signals: Vec<sprite_anim_tick::AnimSignal>,
        timer_signals: Vec<timer_tick::TimerSignal>,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            physics,
            tags,
            ..
        } = FrameGfx::of(gfx);

        // **E a FÍSICA publica no MESMO outbox dos sinais da timeline** (W-Signal).
        // Os quatro canais de leitura dela existem desde o W7 e nenhum fazia nada
        // ACONTECER; o que faltava era o publicador, não o consumidor — a nota do
        // dreno da timeline, acima, já nomeia *gameplay* como um dos consumidores
        // diferidos DAQUELA saída (ADR-0143 + ADR-0075).
        //
        // ⚠️ **Duas fontes, um consumidor, e é aqui que elas se encontram.** A
        // física não importa o tipo de sinal da timeline: fazer o motor de colisão
        // depender do editor de animação para dizer *"algo bateu"* é o oposto do
        // ADR-0075. Cada uma publica o seu, e o SHELL — que já é o dono do
        // consumidor — funde as duas.
        //
        // ⚠️ **Aqui e não lá em cima:** o dreno da timeline roda ANTES do dispatch
        // da física, então ler os sinais de física ali entregaria os do quadro
        // ANTERIOR. Um atraso de um quadro é invisível num toast e deixa de ser
        // invisível no dia em que o consumidor for som.
        // **E A §11 ANIMATION publica no MESMO outbox** (spec §8.10). O tique produziu os fatos
        // lá em cima; aqui eles viram eventos, na janela certa — depois do virar do quadro e antes
        // do dreno.
        for sig in anim_signals {
            self.signals.publish(ph2d_runtime::Signal::from_animation(
                &sig.name,
                sig.entity.to_bits(),
                sig.cycles,
            ));
        }
        for sig in timer_signals {
            self.signals.publish(ph2d_runtime::Signal::from_timer(
                &sig.name,
                sig.entity.to_bits(),
                sig.fires,
            ));
        }
        for sig in physics.signal_events(sim) {
            self.signals.publish(ph2d_runtime::Signal::from_contact(
                &sig.name,
                sig.source.to_bits(),
                sig.other.to_bits(),
            ));
        }
        // **O PAINEL AUTORADO publica no MESMO outbox** — a ponte que faltava.
        //
        // ⚠️ Ela fecha DOIS defeitos de uma vez, e o menor deles é o vazamento: a fila de intents
        // era a única do app sem ponte (physics, sculpt3d, tokens, motion e timeline todas
        // drenam), então ela **crescia sem teto** com o painel aberto — um arrasto de slider
        // empurrava um intent com duas `String` por quadro. O maior é que **todo botão autorado
        // era um controle MORTO**: um aperto não tem valor no store, então este canal é o único
        // que o carrega, e ninguém o lia.
        //
        // ⛔⛔ **E o dreno tinha UM BRAÇO SÓ.** Esta linha dizia *"só o `Fired` vira sinal … os
        // outros são drenados e descartados"*, e a segunda metade era o defeito a descrever-se a
        // si mesmo: as variantes `Choice` e `Text` caíam fora e morriam no fim do quadro, o que
        // mata **seis famílias de widget** de uma vez (Tabs, SegmentedAdaptive, RadioGroup,
        // Dropdown, TextInput, NumberInput). O sintoma é *«o chip acende e nada muda»*.
        //
        // ⚠️ O `match` com um braço nomeado por variante — incluindo as caladas, com o motivo — e
        // a escolha entre dar carga à origem e compor o NOME vivem no irmão, junto dos gates.
        //
        // ⚠️ **A CHAMADA fica AQUI e não lá**, e isso é medido: o gate
        // `the_authored_intent_queue_has_a_drain_and_it_runs_before_the_signal_drain` lê a ORDEM
        // do quadro pela POSIÇÃO deste literal dentro deste arquivo (virar < drenar < ler). Levar
        // a chamada para o irmão apaga a única lente que aquela lei tem.
        for intent in ph2d_panel_authored::drain_intents() {
            authored_intents::publish(&mut self.signals, &intent);
        }
        // **O DRENO — o único lugar do app onde um sinal encontra quem escuta.**
        //
        // Ele roda DEPOIS dos dois produtores, e é por isso que a entrega é no MESMO quadro:
        // um atraso de um quadro é invisível num toast e deixa de ser invisível no dia em que
        // o consumidor for SOM. (A saída ainda guarda o quadro anterior, então um consumidor
        // futuro que rode cedo demais recebe atrasado em vez de nunca — a rede, não a licença.)
        //
        // ⚠️ **Cada consumidor tem cursor próprio e lê a saída com `&self`**, então ele segura
        // `&mut` no PRÓPRIO estado enquanto lê — é isso que um barramento de handlers boxeados
        // não permite, e provavelmente por que o `ph2d-script::messaging` tem zero consumidores
        // desde que nasceu.
        for sig in self.signals.read(&mut self.signal_toast_reader) {
            toasts.push(Toast::info(tr_with(
                "shell.fase_signal_outbox.signal",
                &[("sig", &(sig.name))],
            )));
        }
        // ⭐⭐⭐ **O CONSUMIDOR QUE FAZ ALGUMA COISA** (TOP-20 #5) — a tabela nome → acção.
        //
        // ⚠️ **Aqui, e não noutro sítio do quadro:** depois do dreno (senão os sinais deste quadro
        // só chegariam ao próximo) e antes do `post_frame_undo` (senão a escrita não seria
        // fotografada nem declarada ao ledger). É a mesma janela do toast, por construção.
        //
        // ⚠️ **A resolução é pura e a aplicação não** — ver o cabeçalho de
        // [`signal_actions`]: escrever uma `Visibility` é escrever um componente REGISTADO, e sem
        // o `preview_drive` cada porta que abre viraria um passo de `Ctrl+Z`.
        {
            let disparados: Vec<String> = self
                .signals
                .read(&mut self.signal_action_reader)
                .map(|s| s.name.to_string())
                .collect();
            if !disparados.is_empty() {
                let nomes: Vec<&str> = disparados.iter().map(String::as_str).collect();
                // ⚠️ **A ÁRVORE DE TAGS entra aqui** (TOP-20 #9): uma linha com alvo por TAG pergunta
                // quem pertence à subárvore dela; uma por nome nunca a lê.
                let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), tags, &nomes);
                if !efeitos.is_empty() {
                    let r = signal_actions::apply(
                        sim,
                        &efeitos,
                        &mut self.preview_drive,
                        self.audio.as_mut(),
                    );
                    if self.signal_log_reader.is_some() {
                        eprintln!(
                            "[signal] {} accao(oes) aplicada(s), {} inerte(s)",
                            r.applied, r.inert
                        );
                    }
                }
            }
        }
        if let Some(reader) = self.signal_log_reader.as_mut() {
            for sig in self.signals.read(reader) {
                match sig.origin {
                    ph2d_runtime::SignalOrigin::Timeline { t } => {
                        eprintln!("[signal] {} <- timeline @ {t:.3}s", sig.name);
                    }
                    ph2d_runtime::SignalOrigin::Contact { source, other } => {
                        eprintln!(
                            "[signal] {} <- fisica, {} tocou {}",
                            sig.name, source.0, other.0
                        );
                    }
                    ph2d_runtime::SignalOrigin::Control => {
                        eprintln!("[signal] {} <- controle autorado", sig.name);
                    }
                    ph2d_runtime::SignalOrigin::Motion { tick, rows } => {
                        eprintln!(
                            "[signal] {} <- grafo motion, tique {tick}, {rows} linha(s)",
                            sig.name
                        );
                    }
                    ph2d_runtime::SignalOrigin::Animation { source, cycles } => {
                        eprintln!(
                            "[signal] {} <- animacao da sprite {}, {cycles} ciclo(s)",
                            sig.name, source.0
                        );
                    }
                    ph2d_runtime::SignalOrigin::Timer { source, fires } => {
                        eprintln!(
                            "[signal] {} <- timer do objecto {}, {fires} periodo(s)",
                            sig.name, source.0
                        );
                    }
                }
            }
        }
    }
}
