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

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_fabrica_e_morte(
        &mut self,
        mut deaths: Vec<ph2d_ecs::Death>,
        camera_rect: Option<([f32; 2], [f32; 2])>,
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
        // ⭐⭐⭐ **QUEM MORRE por o VOO ter acabado** (TOP-20 #14) — ao lado do de cima, e pelo mesmo
        // desenho: a ponte do projéctil **anuncia**, e quem remove é este dreno. *Dois despachantes
        // de morte seriam duas respostas à pergunta «quando é que isto sai da cena?».*
        //
        // ⚠️⚠️ **E o filtro é a lei que protege o trabalho do artista:** um projéctil que ele pôs
        // na cena à mão é **documento**, e apagá-lo por ter percorrido o alcance destruiria autoria.
        // A porta é a [`ph2d_ecs::is_transient`] — *o que nasce numa corrida não é documento* —, e
        // ela já tem dois leitores nesta casa. Um projéctil de documento cujo voo acabou **pára** e
        // fica na cena.
        deaths.extend(
            physics
                .projectile_done()
                .iter()
                .map(|(e, _)| *e)
                .filter(|&e| ph2d_ecs::is_transient(sim.world(), e))
                .map(|entity| ph2d_ecs::Death {
                    entity,
                    // ⚠️ **Calada**: o sinal de morte de um projéctil é assunto do `Lifetime`, que
                    // já o autora. Inventar um aqui seria um segundo campo para a mesma coisa.
                    signal: String::new(),
                    why: ph2d_ecs::DeathCause::Spent,
                }),
        );
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
            let varridas = ph2d_app_components::factory_bridge::sweep_spawned(sim);
            if self.signal_readers.logging() && varridas > 0 {
                eprintln!("[fabrica] rebobinou: {varridas} copia(s) varrida(s)");
            }
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
            let mut repostos = ph2d_ecs::rewind_runtime::rewind_runtime_state(sim.world_mut());
            // ⭐ **E os SCRIPTS do artista renascem com eles** (TOP-20 #16): a VM não mora no mundo,
            // então a porta da família `Logic` não os alcança — a irmã dela é a da ponte, que também
            // devolve a pose que a corrida escreveu.
            if let Some(host) = script.as_mut() {
                repostos +=
                    ph2d_app_components::script_bridge::rewind(host, sim, &mut self.preview_drive);
            }
            // ⭐ **E os EMISSORES DE PARTÍCULAS renascem com eles** (TOP-20 #18): uma corrida de
            // partículas não é um componente (não está no mundo), então a porta da família `Logic`
            // também não a alcança.
            repostos += particles.rewind();
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
    }
}
