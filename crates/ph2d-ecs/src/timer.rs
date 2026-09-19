//! ⭐⭐⭐ **O `Timer`** — o item **#2** do TOP-20, e o **primeiro produtor de sinal barato** do
//! produto.
//!
//! # Porque ele é o segundo da fila
//!
//! O levantamento chama-lhe *«a melhor razão custo/benefício»* com veredito unânime das quatro
//! sínteses. A razão é estrutural: o `ph2d-runtime` publica sinais desde o R0, e **o único produtor
//! autorável que existe hoje é uma COLISÃO** (`SignalOnHit`). Sem um relógio, nada acontece por si
//! — todo evento de jogo precisa de dois corpos a tocarem-se.
//!
//! A spec do catálogo é de uma linha: *«nomeados, N por instância, `progress` 0–1, publica
//! Signal»*, e cada palavra dela tem uma consequência de desenho abaixo.
//!
//! # ⚠️ As leis que este módulo herda, e onde cada uma foi paga
//!
//! - **O relógio corre no PASSO FIXO e nunca vê um `f32`.** É a lei da §11 do Sprite: o replay tem
//!   de reproduzir o tique, e um acumulador em vírgula flutuante não é bit-idêntico entre sistemas.
//!   ⇒ tudo aqui é `u64` de **microssegundos**, a mesma unidade do [`crate::SpriteAnimator`].
//! - ⭐⭐⭐ **O componente registado é CONFIG; o relógio vive ao lado e NÃO é registado.** É a lei da
//!   física, escrita por ela: *components de CONFIG, nunca estado vivo de solver — o undo ordena
//!   por bytes*. Um contador que anda a cada tique dentro de um componente registado faz **cada
//!   quadro com input virar um passo de undo**, e isso está medido nesta casa (a auditoria da §11
//!   nomeia-o como família pré-existente, com o `SpriteAnimator` a pagá-lo).
//!
//!   ⛔ **A alternativa era o ledger do `preview_drive`, e foi RECUSADA por medição:** ele guarda
//!   **um** facto por `(entidade, driver)` e o `Driven` é `Copy`, então N timers por entidade
//!   exigiriam um array de tamanho fixo `[u64; TIMERS_MAX]` — `128 B` que **todas** as entradas do
//!   ledger passariam a pagar, porque o maior variante manda no tamanho do enum. *A separação
//!   CONFIG/vivo custa uma struct e não custa um byte a ninguém.*
//!
//!   ⚠️ **O `running` é VIVO, e não autorado** — foi o que esta separação revelou: o `advance`
//!   escreve-o (um *one-shot* que acaba PÁRA), logo pô-lo no componente registado seria um passo de
//!   undo por cada timer que termina. O que o artista autora é o **`autostart`**.
//! - **Um tique atrasado colapsa num sinal só, com a CONTAGEM dentro.** É a lei do `cycles` da §11
//!   e do `rows` do Motion: *o colapso é lossy, e o número é o que ele descarta.* Dez disparos que
//!   ninguém viu são ruído; **um** evento que diz *«dez»* devolve a informação sem multiplicar o
//!   sinal.
//! - **Acabar e repetir distinguem-se por serem NOMES diferentes**, autorados em dois campos — a
//!   lei dos toques da física e das tags de animação. ⛔ Não há campo de «fase».

use bevy_ecs::component::Component;

use crate::{Entity, World};
use serde::{Deserialize, Serialize};

/// Quantos timers uma entidade pode ter.
///
/// ⚠️ **O número sai do PAINEL, não de um palpite** — é a lei que o `ANIM_TAGS_MAX` já pagou:
/// *um modelo que aceita o que o painel não mostra produz estado inalcançável*. A secção do
/// Inspector desenha uma linha por timer dentro do dock, e `16` é o que cabe sem a secção sozinha
/// passar a altura útil da coluna.
pub const TIMERS_MAX: usize = 16;

/// O maior nome de timer, em bytes — o mesmo teto do nome de uma tag de animação, e pela mesma
/// razão: ele viaja no snapshot e é comparado por igualdade.
pub const TIMER_NAME_MAX_BYTES: usize = 64;

/// A maior duração autorável, em microssegundos (**uma hora**).
///
/// ⚠️ **De que recurso ele é:** o `elapsed_ticks` é `u64` de microssegundos, e um `u64` só satura
/// depois de ~584 mil anos — o teto **não** é de representação. Ele é do PAINEL: uma caixa de
/// número que aceita um valor que ninguém consegue esperar produz estado inalcançável, e uma hora
/// é o maior intervalo que um jogo 2D autora sem ser um relógio de calendário (que é outro
/// componente).
pub const TIMER_MAX_US: u64 = 3_600 * 1_000_000;

/// **Um timer nomeado** — o que o artista autora, mais o relógio que o motor escreve.
///
/// ⚠️ **Os dois vivem no mesmo struct de propósito**, exactamente como no `SpriteAnimator`: separá-los
/// em dois componentes obrigaria toda leitura a juntar dois queries, e o ledger já resolve a
/// fronteira **por campo**, que é onde ela de facto está.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timer {
    /// O nome do timer — **para o artista o distinguir**, não para o sinal.
    ///
    /// ⚠️ **Não é o nome do sinal**, e a distinção é a mesma das tags de animação: um timer chama-se
    /// *«recarga»* e pode publicar *«arma_pronta»*. Confundir os dois obrigaria a renomear o
    /// componente para mudar o contrato.
    pub name: String,
    /// Quanto ele demora, em microssegundos. `0` = **nunca dispara** (ver [`advance`]).
    pub duration_us: u64,
    /// Repete para sempre, ou dispara uma vez e pára.
    pub repeat: bool,
    /// Começa a correr quando a cena abre. ⚠️ **É este o campo que o artista autora** — o
    /// *«está a correr agora»* é estado vivo e mora no [`TimerRuntime`].
    pub autostart: bool,
    /// O nome do sinal publicado a cada disparo. **Vazio = calado** — a lei da §11: um produtor sem
    /// nome não fala, em vez de falar com um nome vazio.
    pub signal: String,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            name: String::new(),
            // ⚠️ **Um segundo, e não zero**: `0` é o valor que NÃO dispara, e um timer acabado de
            // acrescentar que nasce mudo lê-se como um componente partido.
            duration_us: 1_000_000,
            repeat: false,
            autostart: true,
            signal: String::new(),
        }
    }
}

/// **O ESTADO VIVO de um timer** — o que o motor escreve, e o undo não fotografa.
///
/// ⚠️ **`Default` é «parado e no zero»**, e é o estado certo para um slot acabado de nascer: quem
/// o arma é o [`reconcile`], no mesmo passo em que o cria.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimerState {
    /// Microssegundos acumulados no período actual.
    pub elapsed_us: u64,
    /// Está a correr agora? ⚠️ **Vivo**: o [`advance`] escreve-o (um *one-shot* que acaba pára).
    pub running: bool,
    /// ⭐⭐⭐ **Este *one-shot* chegou ao FIM** — o facto que `running == false` não sabia dizer.
    ///
    /// # ⛔⛔ Porque ele tem de existir, medido
    ///
    /// O [`advance`] declara por escrito que *«um one-shot que chega ao fim pára e ZERA»*
    /// (`running = false`, `elapsed_us = 0`) ⇒ um timer **terminado** e um que **nunca começou**
    /// eram o mesmo estado, **bit a bit**, e o `progress()` dos dois lê `0,0`.
    ///
    /// É exactamente a ambiguidade que a auditoria da §11 Animation nomeou em 2026-08-23 —
    /// *«pausado» e «terminado» leem-se igual no `playing == false`, e não são a mesma coisa* — e
    /// que o `#14` desta linha pagou outra vez, quando o Inspector lia um EVENTO para pintar um
    /// ESTADO (⇒ `projectiles_finished()` ao lado de `projectile_done()`).
    ///
    /// ⚠️ **O `TimerOutcome::finished` é o ACONTECIMENTO e este é o FACTO.** O primeiro vive um
    /// tique — quem não o apanhar nunca mais o vê —, e este fica até alguém arrancar o relógio.
    ///
    /// ⭐ **Quem o pediu foi o suplente #22** ([`docs/Components/21_plano_tween.md`] §1.2): sem ele
    /// um *fade-out* que acaba faz o objecto **REAPARECER** no quadro seguinte, porque o motor não
    /// tem como distinguir *«ainda não comecei, não escrevas nada»* de *«acabei, fica onde está»*.
    ///
    /// ⚠️ **Uma PAUSA não o levanta** (o [`stop`] não lhe toca): parar a meio é o *Pause* do Godot,
    /// e ler isso como um fim faria um objecto pausado saltar para o valor final.
    /// ⚠️ **Um timer que REPETE nunca o levanta** — ele não acaba, por definição.
    pub finished: bool,
}

/// **O relógio dos timers de uma entidade** — ⛔ **NÃO registado, de propósito**.
///
/// ⚠️ Ele muda a 60 Hz. Registá-lo poria o undo a fotografá-lo e **cada quadro com entrada viraria
/// um passo** — a lei que a física declara desde sempre (*«components de CONFIG, nunca estado vivo
/// de solver»*) e que o `SpriteAnimator` paga por ter escolhido o contrário.
///
/// ⚠️ **O índice casa com o do [`Timers`]**, e é isso que os liga — não um nome. Um nome ligaria
/// dois vectores por uma string que o artista pode editar a meio, e o relógio saltaria de timer.
/// A ponte reconcilia o comprimento a cada tique (ver `render_loop::timer_tick`).
///
/// ⭐⭐⭐ **A AUSÊNCIA de `Serialize`/`Deserialize` é LOAD-BEARING, e foi uma prova de mutação que o
/// mostrou:** tentar registá-lo não dá um gate vermelho, dá **erro de compilação**
/// (`the trait bound TimerRuntime: serde::Serialize is not satisfied`). ⇒ o defeito que este
/// desenho evita — o relógio a virar um passo de undo por quadro — está travado pelo **TIPO**, e
/// nenhum gate é preciso para o segurar. *Derivar `Serialize` aqui por conveniência abriria a
/// porta em silêncio.*
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct TimerRuntime(pub Vec<TimerState>);

impl Timer {
    /// Quanto do período já passou, em `0,0..=1,0`, dado o estado vivo.
    ///
    /// ⚠️ **É DERIVADO, nunca guardado** — a mesma lei do índice de célula da §11. Guardá-lo daria
    /// duas respostas à mesma pergunta, e a que envelhece é a que o painel mostra.
    /// ⚠️ **Duração `0` devolve `0,0`**, e não `1,0`: um timer que nunca dispara não está *cheio*,
    /// está **parado**. É o `UiProgressBar` (#17 do catálogo) que lê isto.
    #[must_use]
    pub fn progress(&self, state: &TimerState) -> f32 {
        if self.duration_us == 0 {
            return 0.0;
        }
        (state.elapsed_us as f64 / self.duration_us as f64).min(1.0) as f32
    }
}

/// **Os timers de uma entidade** — o componente registado.
///
/// ⚠️ **Uma LISTA, porque a spec diz «N por instância»** e um componente ECS é único por entidade.
/// É o mesmo molde do [`crate::SpriteAnimations`] e do `NamedAnchorList`.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timers(pub Vec<Timer>);

/// O que um tique produziu num timer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimerOutcome {
    /// Quantas vezes ele disparou neste tique. `0` = calado.
    ///
    /// ⚠️ **Sempre um evento, com a contagem dentro** — a lei do `cycles`/`rows`. Um tique que
    /// apanha atraso fecha vários períodos de um timer que repete, e publicar um sinal por período
    /// daria uma rajada que ninguém pediu.
    pub fires: u32,
    /// O timer parou neste tique (um *one-shot* que chegou ao fim).
    pub finished: bool,
}

/// Quantos períodos um único tique pode fechar antes de o laço desistir.
///
/// ⚠️ **É uma REDE, não uma lei do produto** — a mesma do `advance` da §11, com a mesma razão
/// escrita: a duração mínima autorável é `1 µs` e o passo fixo tem teto de sub-passos, então na
/// prática ele corre poucas vezes. O contador existe para que uma mudança futura no relógio não
/// possa transformar isto num **congelamento silencioso**.
const FIRES_GUARD: u32 = 1024;

/// **A lei pura** — avança `timer` por `dt_us` e devolve o que aconteceu.
///
/// ⚠️ **Nenhum `f32` entra aqui**, e é isso que faz o replay reproduzir o tique.
///
/// # As recusas embutidas (cada uma tem um gate)
///
/// - `duration_us == 0` **nunca dispara**. ⛔ A alternativa — disparar a cada tique — faria um
///   campo vazio no painel virar uma rajada de sinais, e um valor por preencher não pode ser um
///   gesto destrutivo.
/// - `running == false` **não acumula**, e isso é diferente de acumular sem disparar: uma pausa
///   guarda o progresso, e retomar continua de onde estava (o *Pause* do Godot, não o *Stop*).
/// - Um *one-shot* que chega ao fim **pára e zera**: o `progress` de um timer terminado é `0`, e o
///   próximo *start* é um período inteiro. ⚠️ Deixá-lo cheio faria o disparo seguinte ser imediato.
pub fn advance(timer: &Timer, state: &mut TimerState, dt_us: u64) -> TimerOutcome {
    let mut out = TimerOutcome::default();
    if !state.running || timer.duration_us == 0 || dt_us == 0 {
        return out;
    }
    state.elapsed_us = state.elapsed_us.saturating_add(dt_us);
    let mut guard = 0u32;
    while state.elapsed_us >= timer.duration_us {
        state.elapsed_us -= timer.duration_us;
        out.fires = out.fires.saturating_add(1);
        if !timer.repeat {
            state.running = false;
            state.elapsed_us = 0;
            // ⭐ O ACONTECIMENTO (vive um tique) e o FACTO (fica) — ver [`TimerState::finished`].
            // Sem o segundo, este estado é indistinguível de um timer que nunca arrancou.
            out.finished = true;
            state.finished = true;
            break;
        }
        guard += 1;
        if guard >= FIRES_GUARD {
            state.elapsed_us = 0;
            break;
        }
    }
    out
}

/// ⭐⭐⭐ **A LEI DO NASCIMENTO** — põe o relógio do tamanho da config e arma o que ACABOU de
/// nascer. Devolve `true` quando mexeu em alguma coisa.
///
/// # ⚠️ «Começar a correr» é uma ARESTA, e a aresta é o NASCIMENTO do slot
///
/// Um tique só vê estados, e `autostart` é um facto sobre um instante. A primeira redacção desta
/// função aplicava o `autostart` a **todo** slot que não estivesse a correr, e chamava-se
/// `arm_autostart` — a ideia era que ela corresse **uma vez, no load**. Duas coisas partiram:
///
/// - **No produto ninguém a chamava por quadro**, logo um `Timers` anexado pela paleta nascia
///   **inerte para sempre** — e o mesmo valia para toda entidade que não viesse do ficheiro (a
///   cena de smoke, uma cópia, um respawn do undo). *Os gates armavam à mão o que o produto não
///   armava, e por isso ficavam verdes sobre um componente morto.*
/// - **E chamá-la por quadro para curar isso seria pior:** um *one-shot* que termina põe
///   `running = false`, que é exactamente a condição que ela lia como *«por armar»* ⇒ ele
///   **renasceria a cada quadro** e o `Recarga` do smoke dispararia para sempre.
///
/// ⇒ a condição deixa de ser *«não está a correr»* e passa a ser *«este `TimerState` não existia
/// antes desta chamada»*. É a mesma lei nas duas granularidades, e por isso é **uma** função:
/// o relógio inteiro que nasce (load, paleta, cópia) e o slot apendado num objecto que já tinha
/// timers (o `+` do painel) são o mesmo facto.
///
/// ⚠️ **Ela é idempotente**, e é isso que a deixa correr por quadro: sem nada por nascer devolve
/// `false` sem tocar no vector — quem a chama usa isso para **não** marcar o componente como
/// alterado, que é o hábito que o `SpriteGrid` já teve de corrigir.
///
/// ⚠️ **Encolher também é nascimento ao contrário:** truncar deita fora o estado dos slots que já
/// não existem, senão um timer removido e reposto voltaria a correr a meio.
pub fn reconcile(timers: &Timers, rt: &mut TimerRuntime) -> bool {
    let nascidos = rt.0.len();
    if nascidos == timers.0.len() {
        return false;
    }
    rt.0.resize(timers.0.len(), TimerState::default());
    for (t, s) in timers.0.iter().zip(rt.0.iter_mut()).skip(nascidos) {
        *s = born(t);
    }
    true
}

/// **O que um slot ACABADO DE NASCER recebe** — a porta, com dois leitores.
///
/// ⚠️⚠️ **Ela existe porque REBOBINAR É RENASCER** (`crate::rewind_runtime`), e a resposta tinha de
/// ser a mesma nos dois sítios. Escrita duas vezes, o dia em que o `autostart` ganhasse um irmão
/// deixaria um Reset a devolver um timer que a cena nova arma — *e o modo de falha é mudo*.
///
/// ⚠️ **Não é o `Default`**: um `autostart` nasce **a correr**, e `TimerState::default()` é parado.
#[must_use]
pub fn born(timer: &Timer) -> TimerState {
    TimerState {
        elapsed_us: 0,
        running: timer.autostart,
        // ⚠️ **Nascer é nunca ter acabado** — e é isso que faz o rebobinar devolver um objecto com
        // o *fade* por correr, em vez de um preso no valor final da corrida anterior.
        finished: false,
    }
}

/// **ARRANCAR um timer** — ele passa a correr **do princípio**.
///
/// ⚠️ **Do princípio, e não de onde parou**, que é o que o `Timer.start()` do Godot faz e o que o
/// artista espera de um verbo chamado *start*. ⛔ Não há *resume*: retomar de onde se parou é outra
/// pergunta, e a lei do [`advance`] já a responde sozinha (uma pausa guarda o progresso, então
/// bastaria pôr `running` de volta). Um verbo para isso entra quando alguém o pedir, com o nome
/// que o distinga deste.
///
/// ⇒ é exactamente o que o [`reconcile`] faz a um slot que nasce, e de propósito: *arrancar é
/// arrancar*, venha do `autostart` ou de um sinal.
pub fn start(state: &mut TimerState) {
    state.running = true;
    state.elapsed_us = 0;
    // ⚠️ **Arrancar apaga o fim anterior** — senão um *one-shot* re-arrancado carregaria para
    // sempre a marca de que já tinha acabado, e quem lesse o facto ([`TimerState::finished`])
    // veria as duas coisas verdadeiras ao mesmo tempo.
    state.finished = false;
}

/// **PARAR um timer, guardando o progresso** — o *Pause* do Godot, não o *Stop*.
///
/// ⚠️ É a lei que o [`advance`] já declara do outro lado (*«`running == false` não acumula, e isso
/// é diferente de acumular sem disparar»*), e escrevê-la aqui é o que impede que um verbo de painel
/// zere o relógio por engano — o que tornaria *parar e voltar a arrancar* indistinguível de
/// *parar*.
///
/// ⚠️ **Ele NÃO toca no [`TimerState::finished`], e a ausência é a lei:** pausar não é acabar, e
/// quem lê o facto tem de poder responder *«parado a meio»* e *«chegou ao fim»* com bytes
/// diferentes — que é a razão inteira de aquele campo existir.
pub fn stop(state: &mut TimerState) {
    state.running = false;
}

/// ⭐⭐⭐ **ARMA os relógios de toda a cena** — a travessia do mundo que faltava a este módulo.
///
/// # ⚠️ Ela DESCEU da shell em 2026-09-19, e o sítio certo é aqui
///
/// Ela vivia no `render_loop::timer_tick` e é a **irmã exacta** da [`crate::reconcile_factories`] e
/// da [`crate::reconcile_lifetimes`], que já moram nesta crate: uma varredura sobre as portas deste
/// módulo, sem uma linha de composição. *Enquanto ela esteve na shell, um gate que precisasse de
/// armar relógios tinha de morar lá também* — e foi isso que a catraca `the_shell_only_shrinks`
/// acabou por cobrar.
///
/// ⚠️ **Só escreve quem tem trabalho** — o [`reconcile`] devolve `false` sem nada por nascer, e o
/// `insert` fica de fora. Um `insert` incondicional marcaria o componente como alterado a cada
/// quadro, que é o ruído que o `SpriteGrid` já teve de corrigir.
///
/// ⚠️ **A aresta é o NASCIMENTO do slot, nunca «não está a correr»** — que um *one-shot* terminado
/// satisfaz, e que o faria disparar para sempre. A lei vive no [`reconcile`]; aqui fica a travessia.
pub fn reconcile_timers(world: &mut World) {
    let alvos: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Timers)>();
        q.iter(world).map(|(e, _)| e).collect()
    };
    for e in alvos {
        let Some(timers) = world.get::<Timers>(e).cloned() else {
            continue;
        };
        let mut rt = world.get::<TimerRuntime>(e).cloned().unwrap_or_default();
        if reconcile(&timers, &mut rt) {
            world.entity_mut(e).insert(rt);
        }
    }
}

#[cfg(test)]
#[path = "timer_tests.rs"]
mod tests;
