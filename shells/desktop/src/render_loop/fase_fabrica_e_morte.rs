//! **Fase-filha do quadro: A FÁBRICA E A MORTE** (TOP-20 #11 e #12, 2026-09-14) — os sinais deste
//! quadro fazem nascer cópias, o fora-do-ecrã colhe as que saíram, e um dreno único tira do mundo
//! quem morreu.
//!
//! ⚠️ **Ela é filha da [`super::fase_signal_outbox`] e corre DENTRO dela**, depois da tabela de
//! acções e antes do log — a ordem é a lei (ver o corpo). Nasceu à parte por tecto de LOC, e ficou
//! melhor por isso: o que ela faz tem nome próprio.
//!
//! ⚠️⚠️ **O prefixo `fase_` é LOAD-BEARING, e a falha sem ele é MUDA:** o texto emendado do quadro
//! (`frame_text::render_frame`, que é o oráculo de toda lei de ORDEM desta shell) colhe **só** as
//! funções `fn fase_*` do `render_loop/`. Uma fase-filha chamada de outra coisa **desaparece** dali,
//! e as leis de ordem que a atravessam deixam de ser medidas sem um único teste ficar vermelho.
//! *Foi assim que ela nasceu, e foi o gate de ordem da fábrica que o apanhou.*
//!
//! ⚠️ **O que ela NÃO faz:** decidir. Quem decide quem nasce é `ph2d_ecs::tick_factories`, quem
//! decide quem morre são o `tick_lifetimes` (na fase dos relógios) e o `reap_outside` — as três são
//! puras e não tocam no mundo. *A lei devolve factos, a ponte aplica-os.*

use super::*;

/// ⭐⭐⭐ **O RENASCIMENTO DA CORRIDA — a porta, e ela tem DOIS chamadores.**
///
/// Ela era o corpo do invariante *«parado e no início»* do [`crate::App::fase_fabrica_e_morte`], e
/// virou porta no dia em que um **verbo** passou a poder pedir o mesmo
/// ([`ph2d_ecs::SignalVerb::RestartRun`]). ⛔ *Uma lei escrita em dois sítios ainda não é uma lei —
/// só uma PORTA é*, e aqui o modo de falha da cópia é o caro: a segunda esqueceria uma das quatro
/// metades e a 2.ª corrida nasceria com o resto da primeira, **em silêncio**.
///
/// # ⚠️⚠️ E foi a MEDIÇÃO que obrigou a porta a existir
///
/// O invariante do rebobinar exige a corrida **PARADA** (`!a_correr && time <= 0`), e um recomeço
/// que continua a jogar nunca o satisfaz. ⇒ pôr `time = 0` e seguir deixaria as cópias na cena, os
/// relógios corridos e os contadores gastos — *«recomecei e continuo a perder»*. A porta é o que
/// torna as duas leituras a mesma.
///
/// As **quatro** metades, e nenhuma é opcional:
///
/// | metade | porquê |
/// |---|---|
/// | varrer quem nasceu | uma cópia da corrida anterior não é da nova |
/// | o estado vivo do mundo | relógios, contas, sementes, a vigia, o amortecimento da câmera |
/// | os SCRIPTS | a VM não mora no mundo, logo a porta do `ph2d-ecs` não os alcança |
/// | os EMISSORES | uma corrida de partículas também não é um componente |
///
/// # ⛔⛔⛔ E a QUINTA metade NÃO mora aqui — ela é do RECOMEÇO, e a assimetria é real
///
/// O que um **verbo** escreveu no mundo (`Hide`, `Show`, uma pose) não é estado vivo: é
/// **condução**, e o [`ph2d_preview_drive`] guarda o valor autorado por baixo dela. Devolvê-la é
/// obrigatório num recomeço — sem isso três luzes apagadas por `Hide` continuam apagadas e o dono
/// vê *«as vidas voltaram a três e o painel ficou às escuras»*.
///
/// ⚠️⚠️ **Mas ela NÃO pode entrar nesta porta**, e a razão é medida no modelo da casa: esta função
/// corre no invariante *«parado e no início»*, que é **todo quadro** em que o relógio está ali — e
/// ali quem conduz pode ser o **scrub da timeline**. Devolver as conduções nesse caminho faria um
/// arrasto da régua até ao zero **saltar o objecto para a pose autorada**. ⇒ ali quem trata da
/// condução é o `settle` + a captura do `post_frame_undo` (*«a corrida colapsa em UM passo»*), e
/// num recomeço não há captura nenhuma no meio. *Duas situações, duas respostas, e escrevê-las
/// iguais quebraria a que já funciona.*
fn renascer_a_corrida(
    sim: &mut ph2d_ecs::SimWorld,
    script: &mut Option<ph2d_script::ScriptHost>,
    particles: &mut ph2d_app_components::particles_bridge::ParticlesState,
    drive: &mut ph2d_preview_drive::PreviewDrive,
    motivo: ph2d_ecs::rewind_runtime::Renascimento,
) -> usize {
    let varridas = ph2d_app_components::factory_bridge::sweep_spawned(sim);
    let mut repostos = ph2d_ecs::rewind_runtime::rewind_runtime_state(sim.world_mut(), motivo);
    // ⭐ **Os SCRIPTS do artista renascem com eles** (TOP-20 #16): a VM não mora no mundo, então a
    // porta da família `Logic` não os alcança — a irmã dela é a da ponte, que também devolve a pose
    // que a corrida escreveu.
    if let Some(host) = script.as_mut() {
        repostos += ph2d_app_components::script_bridge::rewind(host, sim, drive);
    }
    // ⭐ **E os EMISSORES DE PARTÍCULAS** (TOP-20 #18): uma corrida de partículas não é um
    // componente (não está no mundo), então a porta da família `Logic` também não a alcança.
    repostos += particles.rewind();
    varridas + repostos
}

/// ⭐⭐⭐ **SERVIR O RECOMEÇO** (`ph2d_ecs::SignalVerb::RestartRun`) — o sétimo passo do laço de um
/// jogo, servido aqui e em mais lado nenhum.
///
/// ⚠️⚠️ **DEPOIS de tudo, e é a única posição possível:** as mortes deste quadro já saíram, os
/// nascimentos já foram publicados, e o renascimento apaga exactamente o que a corrida produziu.
/// *Servi-lo antes faria o dreno da morte medir um mundo que já tinha sido refeito.*
///
/// ⚠️ **O relógio volta ao zero e o `playing` NÃO se toca**: um recomeço que parasse a corrida
/// seria *«acabou»*, não *«outra vez»* — o dono teria de carregar em Play. É a diferença entre este
/// verbo e o botão *Rewind* da barra, que pausa de propósito.
///
/// ⛔ E o invariante do rebobinar **não** serve este caso: ele exige a corrida PARADA, e esta
/// continua a jogar. É por isso que o renascimento é uma PORTA com dois chamadores.
///
/// ⚠️ **Ela nasceu de um CORTE** — a fase chegou a `213` linhas contra o tecto de `200` —, e o
/// corte é por RESPONSABILIDADE: *servir um pedido do quadro* tem nome próprio.
///
/// ⚠️⚠️ **O `#[allow]` dela mora COLADO a ela, uma dúzia de linhas abaixo** — a 1.ª redacção pôs a
/// cerca pura entre o atributo e o dono, e o `clippy` acusou o `servir_o_recomeco` com o atributo
/// ainda no ficheiro. *Um item novo colado a um atributo rouba-o ao dono*, que é a lei que o
/// `ph2d-preview-drive` desta casa já tem escrita (lá o roubo foi um `#[cfg(test)]`, e o produto
/// deixou de compilar).
/// ⭐⭐⭐ **A CERCA CONTRA O LAÇO, como função PURA** — *a corrida tem de ter CORRIDO*.
///
/// ⚠️ **Ela sai do corpo de propósito**, e é a lei que esta casa já escreve: *quando um gate precisa
/// de um device (ou de um `Playhead`, de uma VM e de um mundo) para medir uma decisão que não tem
/// pixel nenhum, a lei está no sítio errado.* Aqui a decisão são **dois números**, e assim ela tem
/// gate e prova de mutação a sério.
///
/// ⛔ **O número é DERIVADO:** a unidade de uma corrida é o PASSO FIXO. Uma corrida cuja vida
/// inteira é o tique que acabou de andar não é uma corrida — e é essa, exactamente, a assinatura do
/// laço (uma condição já verdade no tique `0` pede o recomeço em todo quadro, e o relógio nunca
/// passa dali).
#[must_use]
pub(crate) fn a_corrida_ja_correu(vida: f64, passo_fixo: f64) -> bool {
    vida > passo_fixo
}

#[allow(clippy::too_many_arguments)]
fn servir_o_recomeco(
    recomecar: bool,
    playhead: &mut ph2d_core::Playhead,
    passo_fixo: f64,
    falar: bool,
    sim: &mut ph2d_ecs::SimWorld,
    script: &mut Option<ph2d_script::ScriptHost>,
    particles: &mut ph2d_app_components::particles_bridge::ParticlesState,
    drive: &mut ph2d_preview_drive::PreviewDrive,
) {
    if !recomecar || !playhead.is_playing() {
        return;
    }
    // ⛔⛔⛔ **A CERCA CONTRA O LAÇO, e o número é DERIVADO e não escolhido.**
    //
    // Uma condição que já é verdade quando a corrida começa — um `Counter Watch` escrito
    // `pontos AtLeast 0`, por exemplo — pede o recomeço em **todo** quadro: o relógio nunca passa
    // do primeiro tique, nada avança, e o dono vê um app **congelado** sem uma linha de erro. É a
    // mesma classe do laço `a → b → a` que o doc do `SignalActions` recusa por escrito, com o
    // relógio no lugar do sinal.
    //
    // ⭐ **A cerca é «a corrida tem de ter CORRIDO»**, e a unidade de uma corrida é o PASSO FIXO —
    // não um segundo escolhido, não um contador de recomeços por janela. Uma corrida cuja vida
    // inteira é o tique que acabou de andar **não é uma corrida**, e é exactamente essa a
    // assinatura do laço: no caso patológico o relógio lê `fixed_dt` em todo quadro, e no legítimo
    // lê os segundos que o dono jogou.
    //
    // ⚠️ **E ela FALA.** Um recomeço recusado em silêncio é indistinguível de um verbo partido — a
    // lei que os pincéis da escultura pagaram três vezes.
    let vida = playhead.time();
    if !a_corrida_ja_correu(vida, passo_fixo) {
        eprintln!(
            "[recomecar] RECUSADO: a corrida tem {vida:.4} s, que e' o primeiro tique — a condicao \
             que pede o recomeco ja' e' verdade quando ela COMECA, e servir isto congelaria o app \
             sem dizer porque"
        );
        return;
    }
    playhead.rewind();
    // ⭐⭐ **`Recomecar` e não `Rebobinar`, e a diferença é UMA grandeza:** aqui o relógio volta ao
    // zero e a corrida **continua a jogar**, logo é o único motivo em que um contador marcado
    // sobrevive (*«outra vida, mesma pontuação»*). O invariante do transporte, lá em baixo, passa
    // o outro.
    let repostos = renascer_a_corrida(
        sim,
        script,
        particles,
        drive,
        ph2d_ecs::rewind_runtime::Renascimento::Recomecar,
    );
    // ⛔⛔⛔ **A «QUINTA METADE» FOI CONSTRUÍDA, MEDIDA E RETIRADA** (report do dono, 19/09).
    //
    // Ela devolvia ao autorado **tudo** o que estivesse a ser conduzido, para curar *«as vidas
    // voltaram a três e as luzes ficaram às escuras»*. Duas medições derrubaram-na:
    //
    // 1. ⛔ **Ela não cura o caso que a motivou.** O que um verbo escreve é pré-visualização, e o
    //    `settle` de cada quadro esquece quem não foi declarado ⇒ um `Hide` sobrevive **dois
    //    quadros** e depois é DOCUMENTO. Um recomeço cinco segundos depois já não tem o que
    //    devolver. *Nem ele nem o `Rewind` desfazem um facto do documento — só o `Ctrl+Z`.*
    // 2. ⛔⛔ **E ela LUTAVA contra o artista.** As linhas `Show` que a tabela corre no MESMO sinal
    //    já tinham escrito, e o «autorado» que o ledger guardava por baixo delas era o **apagado**
    //    ⇒ a devolução **desfazia o `Show`**. *Uma porta que devolve «o que estava antes» não sabe
    //    distinguir o que a corrida escreveu do que o artista acabou de mandar escrever.*
    //
    // ⭐ E o que ela ia comprar já estava pago: a POSE de um corpo volta sozinha, porque a ponte da
    // física rebobina quando o tique recua (`rewind_to` → `rebuild_from_rest`).
    if falar {
        eprintln!("[recomecar] a corrida voltou ao inicio ({repostos} estado(s) reposto(s))");
    }
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    /// ⚠️ **O `recomecar` é um PARÂMETRO e não um campo do `App`**, e é deliberado: um verbo novo
    /// que produza um pedido tem de o passar por aqui, e esquecê-lo é **erro de compilação**. É a
    /// mesma forçagem que o abanão da câmera pagou (o offset é argumento do passe da vista, não uma
    /// leitura de componente).
    pub(super) fn fase_fabrica_e_morte(
        &mut self,
        mut deaths: Vec<ph2d_ecs::Death>,
        camera_rect: Option<([f32; 2], [f32; 2])>,
        recomecar: bool,
    ) {
        // ⚠️ **O mapa documento↔entidade é do `App`, não do `AppGfx`** — é o mesmo empréstimo
        // disjunto que o `sync_instances` faz.
        let vec_entities = &mut self.vec.entities;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            component_registry,
            vec_scene,
            tags,
            physics,
            script,
            particles,
            ..
        } = FrameGfx::of(gfx);
        let registry: &ph2d_ecs::scene::ComponentRegistry = component_registry;

        // ⭐⭐⭐ **A CORRIDA É QUANDO O RELÓGIO ANDA**, e esta é a metade que a define.
        //
        // ⚠️ **Sem isto uma fábrica enche a cena enquanto o artista EDITA** — não há modo de jogo
        // neste app e os relógios do passo fixo correm sempre, logo um `Timer` com `autostart`
        // publicaria o sinal dela no primeiro quadro depois de ser anexada. É o mesmo gate que a
        // física usa (`playhead.is_playing()`), e é o que faz *Play → nascem · Stop → congelam ·
        // rebobinar → somem* — o modelo que o Godot e a Unity dão com um modo de jogo separado.
        //
        // ⚠️ **O cursor é lido em TODO quadro, mesmo parado** (o `read` corre, o `for` é que não
        // age): um cursor que só andasse a tocar acumularia `missed` e, ao carregar no play,
        // entregaria de uma vez a janela inteira que o outbox ainda guarda — a cena nasceria com
        // uma rajada de sinais que aconteceram antes. É a lei que o `ui_signal_reader` já escreve.
        let a_correr = self.playhead.is_playing();
        // **QUEM NASCE** — os nomes disparados neste quadro, lidos com o cursor da fábrica.
        let disparados: Vec<String> = self
            .signals
            .read(&mut self.signal_readers.factory)
            .map(|s| s.name.to_string())
            .collect();
        let disparados = if a_correr { disparados } else { Vec::new() };
        let mut nascidos: Vec<(u64, String, u32)> = Vec::new();
        let mut esgotadas: Vec<(u64, String)> = Vec::new();
        if !disparados.is_empty() {
            let nomes: Vec<&str> = disparados.iter().map(String::as_str).collect();
            let tick = ph2d_ecs::tick_factories(sim.world_mut(), tags, &nomes);
            if !tick.births.is_empty() {
                let relatorio = ph2d_app_components::factory_bridge::apply_births(
                    sim,
                    registry,
                    &mut ph2d_app_components::instance_docs::OwnedDocs {
                        vec_scene,
                        vec_entities,
                    },
                    &tick.births,
                    self.fixed_step.tick_count(),
                );
                if self.signal_readers.logging()
                    && (relatorio.nasceram > 0 || relatorio.recusadas > 0)
                {
                    eprintln!(
                        "[fabrica] {} copia(s) nascida(s), {} recusada(s)",
                        relatorio.nasceram, relatorio.recusadas
                    );
                }
            }
            // ⚠️ Os bits da fábrica lêem-se ANTES do dreno da morte, que pode apagar entidades.
            for (e, nome, n) in tick.spawned {
                nascidos.push((e.to_bits(), nome, n));
            }
            for (e, nome) in tick.exhausted {
                esgotadas.push((e.to_bits(), nome));
            }
        }

        // **QUEM MORRE por sair do ecrã DO JOGO** — ⛔ nunca da vista do editor (ver
        // `ph2d_ecs::DestroyOutside`). Sem câmera de jogo na cena, este colhedor não mede nada.
        if let Some((centro, meia)) = camera_rect {
            deaths.extend(ph2d_ecs::reap_outside(sim.world_mut(), centro, meia));
        }
        // ⭐⭐⭐ **QUEM MORRE por causa da FÍSICA** — o voo acabou (TOP-20 #14), a vida chegou a zero
        // ou quem bateu some (plano 28, W2). A ponte **anuncia** e já filtra por `is_transient`
        // (uma peça posta à mão é documento, e fica); este dreno é quem remove. Ver
        // `ph2d_physics_ecs::PhysicsBridge::mortes_anunciadas`.
        deaths.extend(physics.mortes_anunciadas(sim.world()));
        // Os sinais de morte saem ANTES do dreno: depois dele a entidade já não existe, e o nome
        // dela viajaria vazio.
        let mortes: Vec<(u64, String)> = deaths
            .iter()
            .filter(|d| !d.signal.is_empty())
            .map(|d| (d.entity.to_bits(), d.signal.clone()))
            .collect();
        let tiradas = ph2d_app_components::factory_bridge::apply_deaths(sim, &deaths);
        if self.signal_readers.logging() && tiradas > 0 {
            eprintln!("[fabrica] {tiradas} copia(s) sairam da cena");
        }

        // ⭐⭐ **E a outra metade: com o relógio NO INÍCIO e parado, não há corrida — logo não há
        // nada nascido.** É a varredura do «rebobinar», escrita como um INVARIANTE em vez de um
        // gancho num botão: o transporte tem mais de um caminho até ao zero (o botão, o arrasto da
        // régua, o reset do documento), e um gancho em cada um é a lista que envelhece.
        //
        // ⚠️ **Ela não tem custo com a cena parada:** a query é sobre quem tem a marca, e sem
        // corrida não há ninguém com ela.
        if !a_correr && self.playhead.time() <= 0.0 {
            // ⭐⭐⭐ **E o ESTADO VIVO de toda a gente volta ao tique 0** (TOP-20 #15, W0) — o
            // relógio de um `Timer`, a conta e a SEMENTE de uma `Factory`, a vida de um
            // `Lifetime` e o amortecimento de uma `GameCamera`.
            //
            // ⛔⛔ **Sem isto, o que o undo não fotografa o undo também não REPÕE:** um timer que
            // correu continuava corrido, uma fábrica com `Max Total` gasto **recusava-se a
            // produzir na 2.ª corrida**, e uma fábrica aleatória dava uma corrida DIFERENTE a
            // cada rebobinar — o report do dono sobre os projécteis (2026-09-15), um nível acima.
            //
            // ⚠️ **Aqui, dentro do MESMO invariante**, e não num gancho próprio: o transporte tem
            // mais de um caminho até ao zero (o botão, o arrasto da régua, o reset do documento),
            // e um gancho em cada um é a lista que envelhece.
            // ⭐⭐ **`Rebobinar`: aqui o destino é o DOCUMENTO e NADA o atravessa** — nem um
            // contador marcado com `keep_on_restart`. Passar `Recomecar` aqui faria a régua
            // arrastada até ao princípio mostrar a pontuação da corrida anterior.
            let repostos = renascer_a_corrida(
                sim,
                script,
                particles,
                &mut self.preview_drive,
                ph2d_ecs::rewind_runtime::Renascimento::Rebobinar,
            );
            if self.signal_readers.logging() && repostos > 0 {
                eprintln!("[rebobinar] {repostos} estado(s) vivo(s) reposto(s)");
            }
        }

        // ⚠️ **Os sinais são publicados no fim, e chegam ao consumidor no quadro SEGUINTE** — o
        // dreno já correu acima. É o mesmo atraso que a física declara por escrito, e a alternativa
        // (drenar duas vezes) abriria a porta a um laço `nascer → nascer` dentro de um quadro.
        for (bits, nome, n) in nascidos {
            self.signals
                .publish(ph2d_runtime::Signal::from_spawn(&nome, bits, n));
        }
        for (bits, nome) in esgotadas {
            self.signals
                .publish(ph2d_runtime::Signal::from_spawn(&nome, bits, 0));
        }
        for (bits, nome) in mortes {
            self.signals
                .publish(ph2d_runtime::Signal::from_death(&nome, bits));
        }

        servir_o_recomeco(
            recomecar,
            &mut self.playhead,
            self.fixed_step.fixed_dt(),
            self.signal_readers.logging(),
            sim,
            script,
            particles,
            &mut self.preview_drive,
        );
    }
}

/// ⭐ O gate da CERCA contra o laço do recomeço — módulo irmão, porque `tests/it/` é outro binário
/// e não alcança o `render_loop`.
#[cfg(test)]
#[path = "a_cerca_do_laco_do_recomeco_tests.rs"]
mod cerca_tests;
