//! **§11 Animation** — a última das doze seções do Inspector do sprite
//! (spec [`08_animation_inline.md`](../../../docs/Sprite_projeto/08_animation_inline.md)).
//!
//! # ⛔ O «pool de frames» NÃO se reconstrói — ele já existe
//!
//! A spec §8.3 desenha um asset `SpriteFrames` com um `Vec<SpriteFrame>` próprio. Medido em
//! 2026-08-23, **construí-lo seria duplicar duas coisas que o app já tem**:
//!
//! - a **grelha** (`Sprite::hframes × vframes`, índice `Sprite::frame`) — o pool que a §4 Sprite
//!   Sheet autora hoje, e o único que o renderer amostra por quadro;
//! - a **folha autorada** (`ph2d_sprite_sheet::AuthoredSheet`) — regiões nomeadas e ordenadas.
//!
//! ⚠️ **E o segundo não é um sink vivo.** O `SpriteSheetRef` é *proveniência de autoria* («os meus
//! pixels são a região R da folha F»); mudá-lo não muda o que se desenha sem reatar a textura, que
//! é uma operação pesada. ⇒ **Um pool, um sink: a grelha e o `Sprite::frame`.** É também o que a
//! própria spec §8.9 escreve para o caso sem animador.
//!
//! ⇒ Uma animação é um **intervalo nomeado sobre as células que a sprite já tem**. Nada de asset
//! novo, nada de pixels duplicados, e a persistência vem do registro de componentes como no resto
//! do módulo. É o modelo do Aseprite (§8.2: *frames são um pool, animações são tags*) aplicado ao
//! pool que existe.
//!
//! # ⚠️ O tempo é INTEIRO, e a lei pura nunca vê um float
//!
//! [`advance`] recebe `dt_ticks: u64` (microssegundos) e faz tudo em aritmética inteira. A razão
//! está na spec §8.4 e é de determinismo: `elapsed_ms += dt * speed` em `f32` acumula erro que
//! **não é bit-idêntico entre sistemas** (o compilador emite `vfmadd` em x86_64 e não em aarch64),
//! e a divergência de um ULP acaba por mover o tique em que o frame avança — `Sprite::frame`
//! diverge, o UV do atlas diverge, e o hash de replay quebra.
//!
//! Quem converte é a shell, **uma vez**, a partir do `fixed_dt` do passo fixo — que é uma
//! constante. ⚠️ O passo fixo tem teto de 8 sub-passos, por isso [`advance`] é chamada com um `dt`
//! **constante e pequeno**: o laço de recuperação dentro dela é limitado por construção.
//!
//! # O que ficou de FORA, e porquê
//!
//! ⛔ **Duração por-FRAME arbitrária** (spec §8.3, `SpriteFrame::duration_ms`). Ela existe na spec
//! por paridade com o Aseprite, e **não há importador de `.ase`** — ninguém a produziria, e a
//! própria spec §8.8 diz que editá-la é do editor de timeline futuro, não do Inspector. O caso de
//! uso que ela nomeia — *anticipation hold*, o último frame parado mais tempo — é servido pelo
//! [`AnimationTag::hold_ms`], que a spec §8.6 já especifica. *Uma duração por frame sem quem a
//! escreva seria autoria sem consumidor, que é a dívida que este módulo passou o dia a pagar.*

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::SimComponent;

/// Máximo de tags por sprite.
///
/// ⚠️ **A spec §8.11 escreve 256, e este é 64 — de propósito, e a razão é a dela.** O motivo que
/// ela dá para o 256 é *«a contagem típica de animações é < 50»*; o que ela não pesa é que o
/// painel tem de **alcançar** cada uma, e um modelo que aceita o que o painel não mostra produz
/// estado inalcançável por gesto nenhum. Este módulo já pagou essa classe duas vezes (os quatro
/// modos mudos da §9 Sampling, e o cap de âncoras que o gate
/// `the_model_stops_exactly_where_the_panel_runs_out_of_rows` passou a prender).
///
/// 64 é o que a lista da §11 mostra, é o mesmo teto das âncoras, e está acima do «típico < 50»
/// que a própria spec usa para justificar o número maior.
pub const ANIM_TAGS_MAX: usize = 64;
/// Comprimento máximo do nome de uma tag, **em bytes UTF-8** — mesma lei (e mesma razão) do
/// [`crate::ANCHOR_NAME_MAX_BYTES`]: contar o que o postcard conta é o que mantém o hash estável.
pub const ANIM_NAME_MAX_BYTES: usize = 64;
/// Duração mínima de um frame, em ms (spec §8.11). Abaixo de 1 ms é absurdo — e o zero faria o
/// laço de recuperação de [`advance`] não terminar.
pub const FRAME_MS_MIN: u32 = 1;
/// Duração máxima de um frame, em ms (spec §8.11). Acima de 60 s é «estático».
pub const FRAME_MS_MAX: u32 = 60_000;

/// A escala de velocidade em Q16.16: `65536` = `1,0×`.
pub const SPEED_ONE_Q16: i32 = 65_536;
/// Teto da escala de velocidade (spec §8.11: `±100,0×`).
pub const SPEED_MAX_Q16: i32 = 100 * SPEED_ONE_Q16;

/// Por que uma tag foi recusada. ⚠️ Recusar em silêncio é pior que aceitar — o artista escreveria
/// um nome, veria a lista não mudar, e não saberia porquê. Irmã da [`crate::AnchorNameError`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnimTagError {
    /// Vazio — um identificador tem de identificar.
    Empty,
    /// Acima de [`ANIM_NAME_MAX_BYTES`].
    TooLong,
    /// Contém um caractere de controlo.
    ControlChar,
    /// Já existe uma tag com este nome nesta sprite.
    Duplicate,
    /// A lista está cheia ([`ANIM_TAGS_MAX`]) — nada a ver com o nome.
    ListFull,
}

/// Valida o nome de uma tag. ⚠️ A duplicação **não** é vista aqui: ela é propriedade da lista.
pub fn validate_tag_name(name: &str) -> Result<(), AnimTagError> {
    if name.is_empty() {
        return Err(AnimTagError::Empty);
    }
    if name.len() > ANIM_NAME_MAX_BYTES {
        return Err(AnimTagError::TooLong);
    }
    if name.chars().any(char::is_control) {
        return Err(AnimTagError::ControlChar);
    }
    Ok(())
}

/// Como uma tag percorre o seu intervalo (spec §8.3).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimDirection {
    #[default]
    Forward,
    Reverse,
    /// Vai e volta, e repete.
    PingPong,
    /// Volta e vai — o `PingPong` a começar do fim.
    PingPongReverse,
}

impl AnimDirection {
    /// A ordem em que os quatro aparecem na UI e no ficheiro. ⚠️ A posição **é** a tag: reordenar
    /// faria um projeto gravado ler outra direção, em silêncio.
    pub const ALL: [Self; 4] = [
        Self::Forward,
        Self::Reverse,
        Self::PingPong,
        Self::PingPongReverse,
    ];

    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(usize::from(tag)).copied().unwrap_or_default()
    }

    pub fn tag(self) -> u8 {
        Self::ALL.iter().position(|d| *d == self).unwrap_or(0) as u8
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Forward => "Forward",
            Self::Reverse => "Reverse",
            Self::PingPong => "Ping-Pong",
            Self::PingPongReverse => "Ping-Pong Rev",
        }
    }

    /// Começa a andar para trás? (`Reverse` e `PingPongReverse`.)
    fn starts_reversed(self) -> bool {
        matches!(self, Self::Reverse | Self::PingPongReverse)
    }

    /// Inverte no fim em vez de saltar para o início?
    fn bounces(self) -> bool {
        matches!(self, Self::PingPong | Self::PingPongReverse)
    }
}

/// **Uma animação: um intervalo nomeado de células, com o seu ritmo** (spec §8.2/§8.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationTag {
    /// Identificador único dentro da sprite (`idle`, `walk`, `attack`).
    pub name: String,
    /// Primeira célula, **inclusive**.
    pub from: u32,
    /// Última célula, **inclusive**.
    pub to: u32,
    /// Quanto cada frame desta tag dura, em ms. ⚠️ Absoluto, nunca um multiplicador de um FPS
    /// global (spec §8.8/§8.13-1) — é o que permite um *hold* de antecipação sem contas de cabeça.
    pub frame_ms: u32,
    pub direction: AnimDirection,
    /// `None` = repete para sempre; `Some(n)` = toca `n` ciclos e pára. `Some(1)` = uma vez.
    pub repeat: Option<u32>,
    /// Pausa no ÚLTIMO frame do ciclo, em ms (o padrão do Phaser, spec §8.6) — a «respiração» de
    /// um idle antes de recomeçar.
    pub hold_ms: u32,
    /// Pausa adicional **entre ciclos**, em ms. Só existe quando ainda vem outro ciclo.
    pub repeat_delay_ms: u32,
    /// **O nome do sinal que esta animação grita ao ACABAR** (spec §8.10). Vazio = calada.
    ///
    /// ⚠️ **Dois campos, e não um mais uma fase** — é a lei que a física já escreveu para os
    /// contatos (`SignalOrigin::Contact`): *acabar* e *dar a volta* não se distinguem por um
    /// campo, distinguem-se por serem **nomes diferentes, autorados em dois sítios**. Um
    /// consumidor casa numa string e nunca precisa perguntar a origem.
    ///
    /// ⚠️ **Vazio = calada, e é o default** — a maioria das animações não tem nada a dizer, e uma
    /// que grita por omissão enche o canal de eventos que ninguém pediu.
    pub signal_on_finish: String,
    /// **A duração de CADA célula do intervalo**, em ms — vazio = todas duram `frame_ms`.
    ///
    /// ⚠️ **É a recusa medida da spec §8.12, reaberta pelo importador de `.ase`.** Ela dizia *«não
    /// há quem produza durações por-quadro»*; o importador é exactamente quem as produz, e nos
    /// ficheiros reais elas **variam** (o `example.ase` de terceiros vai de 50 a 500 ms, e as três
    /// tags dele variam por dentro). *Quem move o número que tornava algo inalcançável tem de
    /// reconferir a nota.*
    ///
    /// ⚠️ **Indexado a partir do `from`** (`per_frame_ms[cell - lo]`), e **curto é válido**: uma
    /// entrada em falta — ou um `0` — cai no `frame_ms`. Um vetor que pode estar dessincronizado
    /// do intervalo seria estado inválido a guardar; com o fallback, **não há estado inválido**.
    /// Mexer no `from`/`to` não o invalida: ele reindexa-se sozinho.
    ///
    /// ⚠️ O `hold_ms` continua a ser a pausa do ÚLTIMO frame do ciclo, e soma-se a isto. Os dois
    /// resolvem coisas diferentes: este é o ritmo interno, aquele é a respiração entre voltas.
    pub per_frame_ms: Vec<u32>,
    /// O nome do sinal que esta animação grita ao **fechar um ciclo**. Vazio = calada.
    ///
    /// ⚠️ Um tique atrasado fecha vários ciclos de uma vez e sai **um** sinal, com a contagem
    /// dentro (`SignalOrigin::Animation::cycles`) — uma rajada de passos que ninguém deu é ruído,
    /// não um efeito.
    pub signal_on_loop: String,
}

impl AnimationTag {
    /// Uma tag nova sobre o intervalo dado, a 10 fps (o placeholder da spec §8.3).
    pub fn new(name: impl Into<String>, from: u32, to: u32) -> Self {
        Self {
            name: name.into(),
            from,
            to,
            frame_ms: 100,
            direction: AnimDirection::Forward,
            repeat: None,
            hold_ms: 0,
            repeat_delay_ms: 0,
            signal_on_finish: String::new(),
            signal_on_loop: String::new(),
            per_frame_ms: Vec::new(),
        }
    }

    /// **O intervalo REAL desta tag contra o pool que a sprite tem hoje.**
    ///
    /// ⚠️ **Derivado, nunca guardado válido.** A grelha muda quando o artista mexe em
    /// `hframes`/`vframes`, e uma tag gravada com `to = 11` sobre uma grelha que encolheu para 8
    /// células apontaria para fora. Guardar «isto é válido» faria a validade envelhecer em
    /// silêncio — a mesma lei do `kind()` de uma âncora.
    ///
    /// `None` quando a tag não alcança célula nenhuma (a grelha encolheu abaixo do `from`).
    #[must_use]
    pub fn resolve(&self, cells: u32) -> Option<(u32, u32)> {
        if cells == 0 || self.from >= cells {
            return None;
        }
        let lo = self.from.min(self.to);
        let hi = self.from.max(self.to).min(cells - 1);
        (lo <= hi).then_some((lo, hi))
    }

    /// **A duração da célula `frame`**, em microssegundos, com o cap da spec imposto.
    ///
    /// ⚠️ **A lei já perguntava POR FRAME** — o `step_ticks` chama-a dentro do laço, com a célula
    /// que está no ecrã. Era só a resposta que era uniforme, e é por isso que a duração por-quadro
    /// coube numa função e não numa refactoração: *quando a pergunta certa já está no sítio certo,
    /// a feature é a resposta*.
    ///
    /// `lo` é o `from` **resolvido** (o intervalo pode ter sido cortado pela grelha de hoje), e é
    /// contra ele que o vetor se indexa.
    fn frame_ticks_at(&self, frame: u32, lo: u32) -> u64 {
        let ms = self
            .per_frame_ms
            .get(frame.saturating_sub(lo) as usize)
            .copied()
            // `0` é «não declarado», e não «instantâneo»: um zero real faria o laço de recuperação
            // do `advance` girar até ao guarda, e um ficheiro adulterado não pode fazer isso.
            .filter(|v| *v > 0)
            .unwrap_or(self.frame_ms);
        u64::from(ms.clamp(FRAME_MS_MIN, FRAME_MS_MAX)) * 1_000
    }

    /// Esta tag tem ritmo próprio por célula? (O que o painel mostra, e o que o importador diz.)
    #[must_use]
    pub fn has_per_frame_timing(&self) -> bool {
        self.per_frame_ms.iter().any(|&v| v > 0)
    }
}

/// **A biblioteca de animações de uma sprite.** Componente opcional: ausente = sem animações.
///
/// ⚠️ **Per-entidade, e não um asset partilhado** — a mesma escolha (e a mesma consequência) do
/// [`crate::NamedAnchorList`]. Partilhar entre cinquenta goblins é trabalho do **prefab**, que o
/// repositório já tem; um tipo de asset novo pediria variante no `Asset`, caminho de gravação
/// próprio no `ProjectFile`, e o `AssetDb` **não é persistido** (os pixels são re-inseridos no
/// load). Um componente registado grava-se com zero máquina nova.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimations(pub SmallVec<[AnimationTag; 4]>);

impl SpriteAnimations {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &AnimationTag> {
        self.0.iter()
    }

    pub fn get(&self, name: &str) -> Option<&AnimationTag> {
        self.0.iter().find(|t| t.name == name)
    }

    /// Insere, impondo nome, unicidade e contagem. **A porta ÚNICA de crescimento.**
    pub fn insert(&mut self, tag: AnimationTag) -> Result<(), AnimTagError> {
        validate_tag_name(&tag.name)?;
        if self.0.iter().any(|t| t.name == tag.name) {
            return Err(AnimTagError::Duplicate);
        }
        if self.0.len() >= ANIM_TAGS_MAX {
            return Err(AnimTagError::ListFull);
        }
        self.0.push(tag);
        Ok(())
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.0.len();
        self.0.retain(|t| t.name != name);
        self.0.len() != before
    }

    /// Um nome livre da forma `anim_N` — o default que a UI oferece.
    pub fn next_free_name(&self) -> String {
        for n in 0..=ANIM_TAGS_MAX {
            let candidate = format!("anim_{n}");
            if !self.0.iter().any(|t| t.name == candidate) {
                return candidate;
            }
        }
        String::from("anim")
    }
}

impl SimComponent for SpriteAnimations {}

/// **O estado de reprodução.** `SimComponent` — o replay tem de reproduzir o frame avançado.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimator {
    /// A tag a tocar. Vazio, ou um nome que a lista não tem, = [`AnimatorState::NoTag`].
    pub current: String,
    /// A tocar?
    pub playing: bool,
    /// Começa a tocar quando a cena abre.
    pub autoplay: bool,
    /// Escala de velocidade em Q16.16 — [`SPEED_ONE_Q16`] é `1,0×`. Negativo toca ao contrário.
    ///
    /// ⚠️ Inteiro, e não um `f32`, pela razão do doc de módulo: a multiplicação inteira é
    /// bit-idêntica entre sistemas e um acumulador em vírgula flutuante não é.
    pub speed_q16: i32,
    /// Direção que substitui a da tag. `None` = herda.
    pub direction_override: Option<AnimDirection>,
    /// `None` = herda o `repeat` da tag · `Some(true)` = força repetir para sempre ·
    /// `Some(false)` = força tocar uma vez.
    pub loop_override: Option<bool>,
    /// Microssegundos acumulados no frame atual.
    pub elapsed_ticks: u64,
    /// Estado interno do ping-pong: está na volta?
    pub pingpong_reverse: bool,
    /// Quantos ciclos já terminaram.
    pub repeat_count: u32,
}

impl SpriteAnimator {
    /// Um animador parado na tag dada, à velocidade normal.
    pub fn new(current: impl Into<String>) -> Self {
        Self {
            current: current.into(),
            playing: false,
            autoplay: false,
            speed_q16: SPEED_ONE_Q16,
            direction_override: None,
            loop_override: None,
            elapsed_ticks: 0,
            pingpong_reverse: false,
            repeat_count: 0,
        }
    }

    /// A direção efetiva: a da tag, ou a que a substitui.
    pub fn direction(&self, tag: &AnimationTag) -> AnimDirection {
        self.direction_override.unwrap_or(tag.direction)
    }

    /// Quantos ciclos ainda faltam. `None` = infinitos.
    fn cycles_allowed(&self, tag: &AnimationTag) -> Option<u32> {
        match self.loop_override {
            Some(true) => None,
            Some(false) => Some(1),
            None => tag.repeat,
        }
    }

    /// **A reprodução gastou os ciclos que lhe eram permitidos?**
    ///
    /// ⚠️ **É isto que distingue *o artista pausou* de *a animação acabou*, e os dois leem-se
    /// igual no `playing == false`.** Sem a distinção, ligar de novo uma animação de uma volta é
    /// um gesto morto: [`advance`] volta a fechar o ciclo no primeiro passo e põe-se a `false`
    /// outra vez, no mesmo tique. Quem liga uma que acabou está a pedir para a **rever**.
    ///
    /// `false` para uma tag que repete para sempre — ela nunca acaba.
    #[must_use]
    pub fn is_finished(&self, tag: &AnimationTag) -> bool {
        self.cycles_allowed(tag)
            .is_some_and(|n| self.repeat_count >= n)
    }

    /// **Repõe o estado interno** — o que se faz ao trocar de tag ou ao rebobinar.
    ///
    /// ⚠️ Trocar de tag **tem** de o chamar: o `repeat_count` e o `pingpong_reverse` são do ciclo
    /// que estava a correr, e carregá-los para a tag nova faria a primeira volta dela começar a
    /// meio, ou nem chegar a acontecer.
    pub fn rewind(&mut self, tag: Option<&AnimationTag>) {
        self.elapsed_ticks = 0;
        self.repeat_count = 0;
        self.pingpong_reverse = tag.is_some_and(|t| self.direction(t).starts_reversed());
    }
}

impl SimComponent for SpriteAnimator {}

/// O que a montagem de estado desta sprite É — os três casos, nomeados.
///
/// ⚠️ **Três, e não dois** — a mesma lei do [`crate::MountState`]: uma tag que já não existe
/// comporta-se como «sem tag», mas tem de se poder **mostrar**, senão o artista vê a animação
/// parar e não tem o que ler.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnimatorState {
    /// Não há tag escolhida (nome vazio).
    NoTag,
    /// A tag existe e alcança células.
    Ready,
    /// O nome não está na lista, ou a tag não alcança célula nenhuma na grelha de hoje.
    Dangling,
}

/// **A célula por onde um ciclo desta tag COMEÇA**, dada a direção efetiva e a grelha de hoje.
///
/// ⚠️ **Rebobinar não é só zerar contadores — é pôr a IMAGEM no princípio.** Enquanto o
/// [`SpriteAnimator::rewind`] era a operação inteira, o botão *Rewind* repunha o ciclo e deixava a
/// sprite a mostrar a célula onde tinha parado: o artista carregava em «rebobinar» e nada se
/// mexia. Pior, com um `repeat` finito o gesto ficava **inerte** — a imagem estava na ponta, então
/// o primeiro passo de [`advance`] fechava logo o ciclo e parava outra vez.
///
/// `None` quando a tag não alcança célula nenhuma na grelha de hoje.
#[must_use]
pub fn entry_frame(animator: &SpriteAnimator, tag: &AnimationTag, cells: u32) -> Option<u32> {
    let (lo, hi) = tag.resolve(cells)?;
    Some(if animator.direction(tag).starts_reversed() {
        hi
    } else {
        lo
    })
}

/// Que estado a reprodução desta sprite tem, dada a biblioteca e o tamanho do pool.
#[must_use]
pub fn animator_state(
    animator: &SpriteAnimator,
    tags: Option<&SpriteAnimations>,
    cells: u32,
) -> AnimatorState {
    if animator.current.is_empty() {
        return AnimatorState::NoTag;
    }
    match tags.and_then(|l| l.get(&animator.current)) {
        Some(t) if t.resolve(cells).is_some() => AnimatorState::Ready,
        _ => AnimatorState::Dangling,
    }
}

/// O que um passo de [`advance`] produziu — o que a shell publica como sinal (spec §8.10).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct AnimOutcome {
    /// O frame mudou neste passo.
    pub frame_changed: bool,
    /// Quantos ciclos fecharam neste passo (normalmente 0 ou 1).
    pub looped: u32,
    /// A animação chegou ao fim e parou.
    pub finished: bool,
}

/// **A LEI.** Avança `frame` e o estado interno em `dt_ticks` microssegundos.
///
/// `frame` é o índice **absoluto** de célula, e é o que a shell escreve em `Sprite::frame`.
/// Devolve o que aconteceu, para os sinais.
///
/// ⚠️ **Tudo inteiro.** Ver o doc do módulo: um acumulador em vírgula flutuante diverge entre
/// sistemas e leva o hash de replay com ele.
///
/// ⚠️ **O `hold_ms` e o `repeat_delay_ms` são DURAÇÃO do último frame**, e não uma fase à parte.
/// A spec §8.4 desenha um `in_hold_phase` no estado; medido, ele é um estado a mais para o mesmo
/// resultado observável — durante os dois é o último frame que está no ecrã. Dobrá-los na duração
/// dá a mesma imagem com menos uma coisa que pode ficar dessincronizada. O que os distingue
/// continua a existir: o `repeat_delay_ms` **só conta quando ainda vem outro ciclo**.
pub fn advance(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    frame: &mut u32,
    cells: u32,
    dt_ticks: u64,
) -> AnimOutcome {
    let mut out = AnimOutcome::default();
    let Some((lo, hi)) = tag.resolve(cells) else {
        return out;
    };
    // Um `frame` fora do intervalo é o estado normal ao trocar de tag: entra pela ponta certa.
    if *frame < lo || *frame > hi {
        *frame = if animator.pingpong_reverse { hi } else { lo };
        out.frame_changed = true;
    }
    if !animator.playing || animator.speed_q16 == 0 || dt_ticks == 0 {
        return out;
    }
    // Um intervalo de UMA célula não tem para onde avançar; deixá-lo entrar no laço faria os
    // ciclos contarem à velocidade do relógio, e um `repeat` finito terminaria num instante.
    if lo == hi {
        return out;
    }

    // ⚠️ **A magnitude e o SENTIDO viajam separados.** Uma velocidade negativa toca ao contrário —
    // ela não faz o tempo andar para trás. Somar um delta negativo ao `elapsed` obrigaria a
    // «desavançar» frames com o resto do frame anterior, que é um segundo modelo de tempo.
    let speed = animator.speed_q16.clamp(-SPEED_MAX_Q16, SPEED_MAX_Q16);
    let magnitude = (dt_ticks as u128 * u128::from(speed.unsigned_abs()) / 65_536) as u64;
    animator.elapsed_ticks = animator.elapsed_ticks.saturating_add(magnitude);

    let mut guard = 0u32;
    loop {
        let step = step_ticks(animator, tag, *frame, lo, hi);
        if animator.elapsed_ticks < step {
            break;
        }
        animator.elapsed_ticks -= step;
        step_one(animator, tag, frame, lo, hi, speed < 0, &mut out);
        if !animator.playing {
            animator.elapsed_ticks = 0;
            break;
        }
        // Rede: `frame_ticks` é ≥ 1 ms e o passo fixo tem teto de 8 sub-passos, então este laço
        // corre poucas vezes. O contador existe para que uma mudança futura no relógio não possa
        // transformá-lo num congelamento silencioso.
        guard += 1;
        if guard >= 1024 {
            animator.elapsed_ticks = 0;
            break;
        }
    }
    out
}

/// Quanto o frame ATUAL dura, com o hold e o atraso entre ciclos dobrados nele.
fn step_ticks(animator: &SpriteAnimator, tag: &AnimationTag, frame: u32, lo: u32, hi: u32) -> u64 {
    let base = tag.frame_ticks_at(frame, lo);
    if !is_cycle_end(animator, tag, frame, lo, hi) {
        return base;
    }
    let hold = u64::from(tag.hold_ms) * 1_000;
    let more_to_come = animator
        .cycles_allowed(tag)
        .is_none_or(|n| animator.repeat_count + 1 < n);
    let delay = if more_to_come {
        u64::from(tag.repeat_delay_ms) * 1_000
    } else {
        0
    };
    base + hold + delay
}

/// Este frame é o ÚLTIMO do ciclo atual?
fn is_cycle_end(
    animator: &SpriteAnimator,
    tag: &AnimationTag,
    frame: u32,
    lo: u32,
    hi: u32,
) -> bool {
    let dir = animator.direction(tag);
    if dir.bounces() {
        // Num ping-pong o ciclo fecha ao voltar à ponta de PARTIDA.
        let start_at_hi = dir.starts_reversed();
        if animator.pingpong_reverse == start_at_hi {
            // A ir na direção de partida: o ciclo fecha na outra ponta? Não — só depois da volta.
            false
        } else {
            frame == if start_at_hi { hi } else { lo }
        }
    } else if dir.starts_reversed() {
        frame == lo
    } else {
        frame == hi
    }
}

/// Um passo de frame, na direção efetiva. Fecha o ciclo quando chega à ponta.
fn step_one(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    frame: &mut u32,
    lo: u32,
    hi: u32,
    play_backwards: bool,
    out: &mut AnimOutcome,
) {
    let dir = animator.direction(tag);
    // O sentido de marcha: o da tag, invertido pela volta do ping-pong e pela velocidade negativa.
    let mut going_back = dir.starts_reversed();
    if dir.bounces() && animator.pingpong_reverse != dir.starts_reversed() {
        going_back = !going_back;
    }
    if play_backwards {
        going_back = !going_back;
    }

    let at_end = if going_back {
        *frame == lo
    } else {
        *frame == hi
    };
    if !at_end {
        *frame = if going_back { *frame - 1 } else { *frame + 1 };
        out.frame_changed = true;
        return;
    }

    // Chegou à ponta.
    //
    // ⚠️ **Uma animação que NÃO repete pára no ÚLTIMO frame, e não volta ao primeiro.** É o que o
    // Godot e o Phaser fazem, e é o que um *attack* precisa: a pose final é o resultado do gesto.
    // Saltar para o início e só então parar deixaria a sprite na pose de repouso — o oposto.
    let will_continue = animator
        .cycles_allowed(tag)
        .is_none_or(|n| animator.repeat_count + 1 < n);

    if dir.bounces() {
        let start_at_hi = dir.starts_reversed();
        let closes_here = animator.pingpong_reverse != start_at_hi;
        if closes_here && !will_continue {
            close_cycle(animator, tag, out);
            return;
        }
        animator.pingpong_reverse = !animator.pingpong_reverse;
        *frame = if going_back { lo + 1 } else { *frame - 1 };
        out.frame_changed = true;
        if closes_here {
            close_cycle(animator, tag, out);
        }
        return;
    }
    if will_continue {
        *frame = if going_back { hi } else { lo };
        out.frame_changed = true;
    }
    close_cycle(animator, tag, out);
}

/// Fecha um ciclo: conta-o, e pára se já não há mais.
fn close_cycle(animator: &mut SpriteAnimator, tag: &AnimationTag, out: &mut AnimOutcome) {
    animator.repeat_count = animator.repeat_count.saturating_add(1);
    out.looped += 1;
    if let Some(n) = animator.cycles_allowed(tag)
        && animator.repeat_count >= n
    {
        animator.playing = false;
        out.finished = true;
    }
}

#[cfg(test)]
#[path = "sprite_anim_tests.rs"]
mod tests;
