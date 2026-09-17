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

/// ⭐ **O diagnóstico do dreno, e ele IMPRIME MESMO A ZERO** — é esse o caso que interessa: um
/// sinal que soa e não resolve efeito nenhum é o modo de falha MUDO desta tabela (um reactor sem
/// `StableId` não entra na consulta do `resolve`, e o toast aparece na mesma).
///
/// ⚠️ Vive fora da fase por TECTO DE FUNÇÃO — e é o sítio certo: ela não decide nada do quadro.
///
/// ⚠️ **A guarda entra AQUI e não na fase** — pelo tecto de função dela, e porque «imprimir ou não»
/// é assunto do diagnóstico, não do quadro.
fn diga_o_que_resolveu(ligado: bool, nomes: &[&str], efeitos: usize) {
    if ligado {
        eprintln!("[signal] {nomes:?} -> {efeitos} efeito(s)");
    }
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_signal_outbox(
        &mut self,
        anim_signals: Vec<sprite_anim_tick::AnimSignal>,
        timer_signals: Vec<timer_tick::TimerSignal>,
        deaths: Vec<ph2d_ecs::Death>,
        camera_rect: Option<([f32; 2], [f32; 2])>,
        ticks: u32,
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
            script,
            particles,
            sort_scratch,
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
        // ⭐ **A árvore vai junto** (TOP-20 #9, W3c): o `SignalTagFilter` de uma armadilha decide
        // AQUI, onde o `other` existe. Sem filtro nenhum a saída é byte-idêntica.
        for sig in physics.signal_events(sim, tags) {
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
        for sig in self.signals.read(&mut self.signal_readers.toast) {
            toasts.push(Toast::info(tr_with(
                "shell.fase_signal_outbox.signal",
                &[("sig", &(sig.name))],
            )));
        }
        // ⭐⭐⭐ **OS CÉREBROS** (TOP-20 #15) — eles leem os sinais e ANUNCIAM, e o que anunciam é
        // publicado aqui, **antes** de a tabela de acções ler. É isso que faz uma porta abrir no
        // MESMO quadro em que o botão é tocado.
        //
        // ⚠️ **Todas as máquinas leem a mesma fotografia**, tirada antes de qualquer uma avançar —
        // ver o cabeçalho de [`state_machine_tick`]. Uma emissão desta fase chega a quem a escuta
        // no quadro SEGUINTE, que é o que fecha a classe dos laços sem um `if`.
        {
            let ouvidos: Vec<String> = self
                .signals
                .read(&mut self.signal_readers.machine)
                .map(|s| s.name.to_string())
                .collect();
            let nomes: Vec<&str> = ouvidos.iter().map(String::as_str).collect();
            let anunciados = state_machine_tick::advance_machines(sim, &nomes);
            for sig in anunciados {
                self.signals
                    .publish(ph2d_runtime::Signal::from_state_machine(
                        &sig.name,
                        sig.entity.to_bits(),
                    ));
            }
        }
        // ⭐⭐⭐ **OS DOIS MOTORES DA JANELA DOS CÉREBROS** — os scripts do artista (#16) e os
        // emissores de partículas (#18). O corpo mora no irmão `motores_do_quadro` (tecto de LOC);
        // a ORDEM é esta, e é o que o texto emendado do quadro mede: os dois falam ANTES de a
        // tabela de acções ler.
        let relogio = motores_do_quadro::Relogio {
            playing: self.playhead.is_playing(),
            ticks,
            dt: self.fixed_step.fixed_dt(),
        };
        motores_do_quadro::scripts(
            sim,
            script,
            &mut self.preview_drive,
            &mut self.signals,
            &mut self.signal_readers.script,
            &relogio,
        );
        motores_do_quadro::particulas(
            sim,
            particles,
            sort_scratch,
            &mut self.signals,
            &mut self.signal_readers.particles,
            &relogio,
        );
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
                .read(&mut self.signal_readers.action)
                .map(|s| s.name.to_string())
                .collect();
            if !disparados.is_empty() {
                let nomes: Vec<&str> = disparados.iter().map(String::as_str).collect();
                // ⚠️ **A ÁRVORE DE TAGS entra aqui** (TOP-20 #9): uma linha com alvo por TAG pergunta
                // quem pertence à subárvore dela; uma por nome nunca a lê.
                let efeitos = ph2d_ecs::resolve_signal_actions(sim.world_mut(), tags, &nomes);
                diga_o_que_resolveu(self.signal_readers.logging(), &nomes, efeitos.len());
                if !efeitos.is_empty() {
                    let r = signal_actions::apply(
                        sim,
                        &efeitos,
                        &mut self.preview_drive,
                        self.audio.as_mut(),
                    );
                    if self.signal_readers.logging() {
                        eprintln!(
                            "[signal] {} accao(oes) aplicada(s), {} inerte(s)",
                            r.applied, r.inert
                        );
                    }
                }
            }
        }
        // ⭐⭐⭐ **A FÁBRICA** (TOP-20 #11) e **O DRENO DA MORTE** (#12) — ver
        // [`super::fase_fabrica_e_morte`]. ⚠️ **Aqui, nesta ordem, e não noutro sítio:** a fábrica lê os
        // MESMOS sinais que a tabela de acções (logo depois dela, para que um `SignalActions` que
        // arranque um timer não tenha de esperar um quadro), e a morte drena **por último**, que é
        // a lei que o oráculo mediu — um moribundo continua visível a toda consulta até ao fim do
        // quadro.
        self.fase_fabrica_e_morte(deaths, camera_rect);
        // ⭐ **E o DIAGNÓSTICO**, que é assunto próprio e mora no irmão (`fase_signal_log`).
        //
        // ⚠️ **Ela chama-se `fase_*` e isso NÃO é estilo:** o texto emendado do quadro colhe só
        // essas, e com outro nome ela desapareceria do oráculo de **toda** lei de ordem desta
        // shell, em silêncio — a armadilha que a wave da fábrica mediu e escreveu.
        self.fase_signal_log();
    }
}
