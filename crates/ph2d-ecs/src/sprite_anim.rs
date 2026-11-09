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

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::SimComponent;

/// Máximo de tags por sprite. A spec §8.11 diz 256 («típico < 50»); 64 é o que a lista da §11 alcança
/// (o modelo não aceita o que o painel não mostra), o mesmo teto das âncoras.
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

    // ⛔ **A `label()` FOI APAGADA (2026-09-19)** — órfã provada, como a irmã do
    // [`crate::BlendMode`]. O `ph2d-panel-inspector` pinta estas quatro palavras da tabela dele
    // (`anim_rows.rs`, cujo doc já dizia *«espelha `ph2d_ecs::AnimDirection::label`»*), chaveadas
    // pelo `tag()` que o snapshot leva. *Um espelho declarado é uma cópia com um nome bonito.*

    /// Começa a andar para trás? (`Reverse` e `PingPongReverse`.)
    pub(crate) fn starts_reversed(self) -> bool {
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
    /// A cor desta animação no editor: índice da paleta, `0..ANIM_COLORS`. Mora na animação para não mudar ao
    /// reordenar nem ao apagar outra; quem a escolhe é [`SpriteAnimations::insert`].
    pub color: u8,
    /// A folha do intervalo `from..=to` (plano GEF 26): `0` = a do `Sprite`. Uma lista de quadros diz a folha de cada um.
    pub sheet: u8,
    /// Ping-pong com um número ÍMPAR de passadas do Aseprite ([`tag_do_aseprite`]): o último ciclo é
    /// meio — pára na ponta de CHEGADA, sem voltar. Só vale com o `repeat` da tag (sem `loop_override`).
    #[serde(default)]
    pub acaba_na_ponta_de_chegada: bool,
}

/// Quantas cores a paleta das animações tem; o editor dá um token a cada índice.
pub const ANIM_COLORS: u8 = 8;

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
            color: 0,
            sheet: 0,
            acaba_na_ponta_de_chegada: false,
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
}

/// **A biblioteca de animações de uma sprite.** Componente opcional: ausente = sem animações.
///
/// ⚠️ **Per-entidade, e não um asset partilhado** — a mesma escolha (e a mesma consequência) do
/// [`crate::NamedAnchorList`]. Partilhar entre cinquenta goblins é trabalho do **prefab**, que o
/// repositório já tem; um tipo de asset novo pediria variante no `Asset`, caminho de gravação
/// próprio no `ProjectFile`, e o `AssetDb` **não é persistido** (os pixels são re-inseridos no
/// load). Um componente registado grava-se com zero máquina nova.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimations(
    pub SmallVec<[AnimationTag; 4]>,
    /// O aviso de imagem trocada: o corte `[colunas, linhas]` que ficou da imagem de antes, enquanto a grelha é esta.
    /// Entra e sai pelo desfazer com a troca que o causou; refazer o corte à mão o cala.
    pub Option<[u32; 2]>,
);

impl SpriteAnimations {
    pub fn new() -> Self {
        Self(SmallVec::new(), None)
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
    pub fn insert(&mut self, mut tag: AnimationTag) -> Result<(), AnimTagError> {
        validate_tag_name(&tag.name)?;
        if self.0.iter().any(|t| t.name == tag.name) {
            return Err(AnimTagError::Duplicate);
        }
        if self.0.len() >= ANIM_TAGS_MAX {
            return Err(AnimTagError::ListFull);
        }
        tag.color = self.next_color();
        self.0.push(tag);
        Ok(())
    }

    /// A cor menos usada, a de menor índice no empate.
    fn next_color(&self) -> u8 {
        (0..ANIM_COLORS)
            .min_by_key(|c| self.0.iter().filter(|t| t.color == *c).count())
            .unwrap_or(0)
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

/// **A porta ÚNICA: uma tag no idioma do Aseprite vira uma [`AnimationTag`]** — o `.ase` importado
/// e o Pixel Sprite passam ambos aqui ([P14 §4](../../../docs/PixelArt/P14_plano.md)).
///
/// `duracoes_ms` é a duração de CADA quadro do documento inteiro. Uniforme no intervalo ⇒
/// `frame_ms` dessa duração e `per_frame_ms` vazio; senão `frame_ms` = a dominante (empate ⇒ a
/// menor; intervalo fora ⇒ 100) e `per_frame_ms` o do intervalo. `repeat 0` ⇒ para sempre.
/// `repeat` N conta PASSADAS (lei do Aseprite, medida no Trial: `ph2d_pixel_core::play`); o motor
/// conta CICLOS. No ping-pong, N passadas = `ceil(N/2)` ciclos e, N ímpar, o último é meio
/// ([`AnimationTag::acaba_na_ponta_de_chegada`]).
/// A direcção passa por [`AnimDirection::from_tag`], nunca por cast.
#[must_use]
pub fn tag_do_aseprite(
    nome: &str,
    from: u32,
    to: u32,
    direcao: u8,
    repeat: u16,
    duracoes_ms: &[u32],
) -> AnimationTag {
    let lo = from.min(to) as usize;
    let hi = (from.max(to) as usize).min(duracoes_ms.len().saturating_sub(1));
    let span = duracoes_ms.get(lo..=hi).unwrap_or(&[]);
    let uniforme = span
        .first()
        .filter(|d| span.iter().all(|g| g == *d))
        .copied();
    let ms = uniforme.unwrap_or_else(|| {
        let mut best = (0_usize, u32::MAX);
        for d in span {
            let n = span.iter().filter(|g| *g == d).count();
            if n > best.0 || (n == best.0 && *d < best.1) {
                best = (n, *d);
            }
        }
        if best.0 == 0 { 100 } else { best.1 }
    });
    let per_frame_ms = if uniforme.is_some() {
        Vec::new()
    } else {
        span.iter().map(|d| (*d).max(1)).collect()
    };
    let direction = AnimDirection::from_tag(direcao);
    let n = u32::from(repeat);
    let (ciclos, meio) = if direction.bounces() {
        (n.div_ceil(2), n % 2 == 1)
    } else {
        (n, false)
    };
    AnimationTag {
        frame_ms: ms.max(1),
        direction,
        repeat: (ciclos > 0).then_some(ciclos),
        per_frame_ms,
        acaba_na_ponta_de_chegada: meio,
        ..AnimationTag::new(nome, from, to)
    }
}

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
    /// A animação com que o jogo e o abrir do projeto começam (só em «Auto»). Separada do `current`: o ▶ do cartão é
    /// prévia e muda o `current`, nunca isto. `None` = começa no `current`.
    pub start: Option<String>,
}

/// A velocidade da §11 Animation: a faixa do motor (±100×). A lei do cartão Animation (`ph2d_app_sprite_anim`) e do
/// `ph2d.set` (`ph2d_app_components::property_laws`) — desceu para esta folha em 2026-10-28.
pub fn sanitize_sprite_animator(p: &mut SpriteAnimator) {
    p.speed_q16 = p.speed_q16.clamp(-SPEED_MAX_Q16, SPEED_MAX_Q16);
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
            start: None,
        }
    }

    /// A animação que vale ao começar: a escolhida, se a lista ainda a tem; senão o `current`, se a
    /// lista o tem; senão a 1.ª da lista (um `current` vazio ou de uma tag renomeada não toca nada).
    #[must_use]
    pub fn start_or_current<'a>(&'a self, lib: &'a SpriteAnimations) -> &'a str {
        [self.start.as_deref(), Some(self.current.as_str())]
            .into_iter()
            .flatten()
            .find(|n| lib.get(n).is_some())
            .or_else(|| lib.iter().next().map(|t| t.name.as_str()))
            .unwrap_or(&self.current)
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

#[path = "sprite_anim_lei.rs"]
mod lei;
pub(crate) use lei::advance_span_visiting;
pub use lei::{AnimOutcome, advance};

#[cfg(test)]
#[path = "sprite_anim_tests.rs"]
mod tests;
