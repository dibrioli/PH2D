//! ⭐⭐⭐ **O `AudioSource2D` + o `AudioListener2D`** — o item **#4** do TOP-20, e a **primeira
//! reacção AUDÍVEL** do produto.
//!
//! # O que ele fecha
//!
//! Este repositório tem um subsistema de áudio completo — um mixer de 64 vozes com pool e roubo do
//! mais-velho-mais-fraco, um rack de **42 efeitos com 23 presets**, espectral, denoise por ML,
//! exportação Ogg/Opus, streaming — e **nada disso tem consumidor de CENA**. Medido em 2026-09-09:
//! `AudioSource2D` e `AudioListener2D` têm **zero** ocorrências na árvore, e a única linha que os
//! nomeia é a recusa escrita no [`crate::signal_actions`] a dizer que não existem.
//!
//! ⇒ *o rack é 100% editor-side*: um objecto da cena não pode fazer barulho. É esse o buraco.
//!
//! # ⚠️ As leis que este módulo herda, e onde cada uma foi paga
//!
//! - ⭐⭐ **A lei devolve FACTOS; quem toca é a ponte.** É a fronteira que o [`crate::timer`] e o
//!   [`crate::signal_actions`] já usam, e aqui ela é ainda mais dura: uma voz é um recurso do
//!   dispositivo, e o dispositivo vive na shell (o `cpal` nunca entra numa crate de lei). Este
//!   módulo decide **onde** o som está e **quanto** dele se ouve; ele nunca abre um ficheiro nem
//!   fala com o mixer.
//! - **O componente registado é CONFIG.** O que anda a 60 Hz — a voz viva, o que já tocou — fica
//!   **fora** dele, pela lei da física que o `Timer` pagou: *um contador dentro de um componente
//!   registado faz cada quadro com entrada virar um passo de undo*. ⇒ não há `VoiceId` aqui.
//! - **Vazio = calado.** Um `AudioSource2D` sem som nomeado não toca, em vez de tocar um silêncio.
//!   É o espelho exacto da lei do produtor de sinais (*«um produtor sem nome não fala, em vez de
//!   falar com um nome vazio»*).
//! - ⛔ **`autoplay` nasce DESLIGADO**, e é a mesma lei do `+` da tabela de acções: *um default que
//!   disparasse em alguma coisa faria anexar um componente MUDAR a cena*. Quem faz barulho é o
//!   artista, pelo verbo `PlaySound` da tabela de acções — que é exactamente o circuito que o
//!   levantamento desenha para o bloco 2–5.
//!
//! # ⚠️ O `sound` é um CAMINHO, e é o primeiro componente registado desta casa que guarda um
//!
//! Medido antes de escolher: **o índice de assets não conhece áudio** (nenhuma extensão de som no
//! `ph2d-asset`), e a única porta que o app tem para *«isto é um ficheiro de áudio»* é o
//! `decode_any` da shell (`AUDIO_IMPORT_EXTS` + `decode`), que já serve o editor de áudio, o
//! `audio.bands` do Motion e o diálogo de importação. ⇒ o componente nomeia o ficheiro e a ponte
//! resolve-o por essa porta, que é a mesma lei do alvo do `SignalActions`: *o modelo guarda o NOME,
//! a resolução é de quem tem o mundo*.
//!
//! ⚠️ **As duas consequências, declaradas:** mover o ficheiro parte o som (como a escultura, que
//! por isso tem um *Relink*), e **o projecto não embute o áudio**. As duas são o preço de não
//! inventar um índice de assets nesta wave; a cura de ambas é a mesma — pôr áudio no índice — e ela
//! é wave própria, com o `AssetId` por conteúdo que a F4 já usa para os pixels.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::{Entity, World};

/// Quantas vozes um único `AudioSource2D` pode ter a tocar ao mesmo tempo.
///
/// ⚠️ **De que recurso ele é:** do POOL do mixer, que tem `ph2d_audio::MAX_VOICES` = 64 vozes e
/// rouba a mais velha-mais-fraca quando enche. Um teto por-fonte existe para que **uma** fonte
/// disparada em rajada não coma o pool inteiro e cale toda a cena — que é a razão pela qual o
/// Godot tem o `max_polyphony`. `16` é um quarto do pool: chega para uma metralhadora e deixa três
/// quartos para o resto da cena.
pub const AUDIO_MAX_POLYPHONY: u8 = 16;

/// O maior alcance autorável, em metros.
///
/// ⚠️ **É um teto de PAINEL, e não do motor:** a atenuação é aritmética em `f32` e não satura em
/// distância nenhuma que um jogo 2D use. O número vem da escala das cenas deste app — os objectos
/// das cenas de smoke vivem em `±2 m` e o `Transform` **já é metros** (não há porta de escala; a
/// única px→m é o `pixels_per_meter` do projecto) — então `1 km` é três ordens de grandeza acima do
/// que se vê, que é a folga que separa *«um teto»* de *«uma cerca que morde»*.
pub const AUDIO_MAX_DISTANCE_M: f32 = 1_000.0;

/// **Em que barramento do mixer um som de cena entra.**
///
/// ⚠️ **Ele atravessa a fronteira como TAG, nunca como o enum do mixer** — a lei do ADR-0029 que o
/// `SignalVerb` já paga: o `ph2d-ecs` é lei de cena e **não depende do `ph2d-audio`**. Quem volta a
/// fazer disto um `BusId` é a shell, que tem os dois lados.
///
/// ⛔ **O barramento `Ui` do mixer NÃO está aqui, e a ausência é a decisão.** Ele é a voz do
/// *chrome* — os quatro sons sintetizados do editor (o D1), que uma preferência do utilizador
/// (`~/.ph2d/prefs.txt`, `ui_sound=0`) desliga. Um som de JOGO encaminhado para lá seria calado por
/// uma preferência que fala de outra coisa, e o artista não teria como o saber. *Um controlo cujo
/// efeito depende de uma preferência que nomeia outro assunto é um controlo que mente.*
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioBus {
    /// Efeitos — o barramento por omissão de um som de cena.
    #[default]
    Sfx,
    /// Música e ambiente.
    Music,
    /// Voz / diálogo.
    Voice,
    /// Directo ao master, sem passar por um sub-barramento.
    Master,
}

impl AudioBus {
    /// ⚠️ **A ORDEM é o contrato**: a posição nesta lista é a tag, e é dela que o painel deriva as
    /// entradas do seletor e a ponte deriva o `BusId`. ⛔ Reordenar isto muda o barramento de toda
    /// cena já gravada, **e compila**.
    pub const ALL: [AudioBus; 4] = [
        AudioBus::Sfx,
        AudioBus::Music,
        AudioBus::Voice,
        AudioBus::Master,
    ];

    /// O rótulo que o artista lê. ⚠️ **Ele viaja no snapshot** — o painel não conhece este enum.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            AudioBus::Sfx => "SFX",
            AudioBus::Music => "Music",
            AudioBus::Voice => "Voice",
            AudioBus::Master => "Master",
        }
    }

    /// A posição em [`Self::ALL`].
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            AudioBus::Sfx => 0,
            AudioBus::Music => 1,
            AudioBus::Voice => 2,
            AudioBus::Master => 3,
        }
    }

    /// O inverso do [`Self::tag`]. ⚠️ **Uma tag desconhecida cai no default e não recusa** — ela
    /// chega de um ficheiro, e recusar o load inteiro por causa de um barramento seria trocar um
    /// som no sítio errado por um projecto que não abre.
    #[must_use]
    pub fn from_tag(tag: u8) -> AudioBus {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }
}

/// **A FONTE de som de um objecto** — o componente registado.
///
/// ⚠️ **Um objecto tem UMA fonte, e não uma lista**, ao contrário do [`crate::Timers`] e do
/// [`crate::SignalActions`]. A razão é o que o artista de facto autora: um objecto **é** uma porta,
/// e o som dela é o som dela; dois sons diferentes no mesmo objecto são dois objectos (ou duas
/// linhas da tabela de acções a apontar para dois filhos). *Uma lista aqui multiplicaria o painel
/// por N para servir um caso que a composição já exprime.*
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioSource2D {
    /// O ficheiro de som. **Vazio = calado** — ver o doc do módulo.
    pub sound: String,
    /// Volume em decibéis, `0` = o som como ele foi gravado.
    ///
    /// ⚠️ **Decibéis e não um factor linear**, e não é preferência: a percepção de intensidade é
    /// logarítmica, então um slider linear passa metade do curso numa região que o ouvido não
    /// distingue. É o que o Godot e o Unreal expõem, pela mesma razão.
    pub volume_db: f32,
    /// Multiplicador de velocidade de leitura (`1` = o tom original, `2` = uma oitava acima).
    pub pitch: f32,
    /// Repetir para sempre em vez de parar no fim.
    ///
    /// ⚠️ **Aqui e não no ficheiro:** o mesmo `.wav` serve de passo (um disparo) e de motor a
    /// trabalhar (em ciclo), e quem sabe qual dos dois é este objecto é a CENA.
    pub looping: bool,
    /// Começa a tocar quando a cena abre.
    ///
    /// ⛔ **Nasce DESLIGADO** — ver o doc do módulo.
    pub autoplay: bool,
    /// A distância, em metros, a partir da qual esta fonte deixa de se ouvir.
    ///
    /// ⚠️ **`0` = INAUDÍVEL**, e não *«sem limite»*: um alcance de zero é um alcance de zero, e ler
    /// um campo vazio como *infinito* faria o valor por preencher ser o mais barulhento de todos.
    /// Quem quer *«ouve-se de todo o lado»* põe um número grande — o teto é o
    /// [`AUDIO_MAX_DISTANCE_M`].
    pub max_distance: f32,
    /// O expoente da queda de volume com a distância.
    ///
    /// `1` = queda linear até ao silêncio; `>1` cala mais depressa perto do fim (o que soa mais
    /// natural, porque a intensidade real cai com `1/d²`); `<1` mantém o som cheio quase até ao
    /// limite. ⚠️ **`0` faz a fonte soar no volume máximo até desaparecer de repente na borda** —
    /// é um degrau, e existe de propósito para quem quer uma zona audível de fronteira dura.
    pub attenuation: f32,
    /// Dentro deste raio (metros) a fonte não tem LADO — ela soa ao centro.
    ///
    /// ⭐⭐ **É o `non_spatialized_radius` do Unreal, e ele cura um defeito real, medido:** com o
    /// pan a ser a DIRECÇÃO (`dx / d`), uma fonte que passa rente ao ouvinte inverte o azimute num
    /// intervalo minúsculo e o pan **varre a faixa toda** de um lado ao outro — o *«pan pulando»*.
    /// Medido nesta casa: a passar a `5 cm` do ouvinte, o pior salto entre dois passos de `1 cm`
    /// vai de `0,20` sem raio para `0,01` com um raio de meio metro. Um raio de zero devolve o
    /// comportamento sem ele, **ao bit**. Ver [`spatialize`] para a forma da mistura.
    pub non_spatialized_radius: f32,
    /// Quanto do pan geométrico chega de facto à saída, `0..=1`.
    ///
    /// `0` = mono (o som tem posição para o VOLUME e não para o lado); `1` = o pan cheio.
    /// ⚠️ É o `panning_strength` do Godot, e serve para o caso comum de música de ambiente que
    /// deve atenuar com a distância sem andar de um ouvido para o outro.
    pub panning_strength: f32,
    /// Quantas vozes desta fonte podem soar ao mesmo tempo — ver [`AUDIO_MAX_POLYPHONY`].
    ///
    /// ⚠️ **`1` é o default e é o certo:** o caso comum é um som que **substitui** o anterior (um
    /// passo, um clique de porta). Uma fonte que acumula vozes por omissão transforma um sinal
    /// repetido numa parede de som.
    pub max_polyphony: u8,
    /// A posição em [`AudioBus::ALL`]. ⚠️ Guardado como TAG — ver o doc do [`AudioBus`].
    pub bus_tag: u8,
}

impl Default for AudioSource2D {
    fn default() -> Self {
        Self {
            sound: String::new(),
            volume_db: 0.0,
            pitch: 1.0,
            looping: false,
            autoplay: false,
            // ⚠️ **Dez metros**, e o número sai da ESCALA das cenas deste app: os objectos das cenas
            // de smoke vivem em `±2 m`, então dez metros é *«ouve-se de qualquer ponto da cena e
            // cala-se para lá dela»* — que é o que um default tem de significar. ⛔ Não é um teto
            // (esse é o `AUDIO_MAX_DISTANCE_M`), e um default que fosse o teto tornaria o campo
            // inerte por omissão.
            max_distance: 10.0,
            attenuation: 1.0,
            non_spatialized_radius: 0.0,
            panning_strength: 1.0,
            max_polyphony: 1,
            bus_tag: 0,
        }
    }
}

/// **AS ORELHAS DA CENA** — de onde se ouve.
///
/// ⚠️ **Um marcador, e não um número.** A posição vem do [`crate::Transform`] da entidade que o
/// carrega, como tudo o resto — dar-lhe coordenadas próprias seria a segunda resposta à pergunta
/// *«onde está isto?»*, e a que envelhece é sempre a que o artista não vê.
///
/// ⚠️ **A cena pode não ter nenhum**, e o que acontece então está escrito no [`spatialize`]: o som
/// toca **sem posição** (ao centro, no volume cheio). ⛔ A alternativa — usar a origem do mundo como
/// ouvinte implícito — foi recusada por ser **silenciosa e errada**: uma cena inteira construída
/// longe da origem ficaria muda sem que nada o explicasse. *Um default invisível que cala o produto
/// é pior que uma ausência que o painel nomeia.*
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioListener2D;

/// O que a geometria diz ao mixer sobre uma fonte, neste instante.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spatial {
    /// `-1` = todo à esquerda · `0` = ao centro · `1` = todo à direita.
    pub pan: f32,
    /// Ganho LINEAR já com o `volume_db` dentro. `0` = inaudível.
    pub gain: f32,
}

/// Decibéis → ganho linear.
///
/// ⚠️ **`-80 dB` ou menos é ZERO exacto, e não um número muito pequeno.** É a convenção do Godot
/// (`linear_to_db`/`db_to_linear` saturam ali) e ela existe para que *«no mínimo do slider»* queira
/// dizer **silêncio** e não *«0,0001 do volume»* — que, somado por 64 vozes, se ouve.
#[must_use]
pub fn db_to_linear(db: f32) -> f32 {
    if db <= -80.0 {
        return 0.0;
    }
    10.0_f32.powf(db / 20.0)
}

/// ⭐⭐⭐ **A LEI DA POSIÇÃO** — o que o mixer tem de saber sobre esta fonte, agora.
///
/// `listener` é `None` quando a cena não tem [`AudioListener2D`] nenhum.
///
/// # As leis embutidas, cada uma com um gate
///
/// - **Sem ouvinte, sem posição:** `pan = 0` e o ganho é só o `volume_db`. ⛔ Ver o doc do
///   [`AudioListener2D`] para a alternativa recusada.
/// - **`max_distance == 0` é inaudível**, e não *«sem limite»* — ver o campo.
/// - **Fora do alcance, ganho ZERO exacto.** Uma cauda que tende a zero mantém a voz viva a gastar
///   uma vaga do pool para não se ouvir, e 64 delas calam a cena.
/// - ⭐⭐ **O raio não-espacializado é uma MISTURA, nunca um `if`.** A primeira redacção era
///   *«dentro do raio, pan zero»*, e isso troca um salto por outro: a fonte passa a saltar do
///   centro para o pan cheio ao **cruzar o raio**. ⇒ a força da espacialização sobe de `0` na
///   borda do raio a `1` a **dois** raios, que é a mesma distância outra vez — a zona de mistura
///   tem o tamanho da coisa que a define, e não um número escolhido. Com `raio = 0` o factor é `1`
///   em todo o lado e o resultado é **byte-idêntico** ao de não haver raio nenhum.
/// - ⭐⭐⭐ **O pan é a DIRECÇÃO, e não a distância** — `dx / d`, que é o seno do azimute. Uma
///   fonte um metro à direita está **toda** à direita, venha o alcance de dez metros ou de mil.
///
///   ⚠️ **A primeira redacção normalizava o pan pelo `max_distance`, e o GATE apanhou-a** — não por
///   soar mal, mas porque a metade dele que mede a FIXTURA não conseguiu produzir o fenómeno que o
///   raio existe para curar (pior salto `0,001` a atravessar o ouvinte). *Uma lei em que o defeito
///   não é reproduzível é uma lei em que a cura não faz nada.* E a leitura é a de sempre nesta
///   casa: o `max_distance` é um knob sobre **volume**, e usá-lo também para a direcção era
///   **um parâmetro com dois papéis** — exactamente o defeito do `deadzone` do Godot que o Input
///   Map desta casa já corrigiu com dois números.
#[must_use]
pub fn spatialize(source: [f32; 2], listener: Option<[f32; 2]>, cfg: &AudioSource2D) -> Spatial {
    let volume = db_to_linear(cfg.volume_db);
    let Some(ears) = listener else {
        return Spatial {
            pan: 0.0,
            gain: volume,
        };
    };
    if cfg.max_distance <= 0.0 {
        return Spatial {
            pan: 0.0,
            gain: 0.0,
        };
    }
    let dx = source[0] - ears[0];
    let dy = source[1] - ears[1];
    let d = dx.hypot(dy);
    if d >= cfg.max_distance {
        return Spatial {
            pan: 0.0,
            gain: 0.0,
        };
    }
    // A queda: `1` colado ao ouvinte, `0` na borda, com o expoente a dar-lhe a forma.
    let falloff = (1.0 - d / cfg.max_distance).clamp(0.0, 1.0); // CLAMP-OK: fracção adimensional
    let gain = volume * falloff.powf(cfg.attenuation.max(0.0));

    // A força da espacialização — a mistura do raio não-espacializado, ver o doc.
    let spread = if cfg.non_spatialized_radius > 0.0 {
        ((d - cfg.non_spatialized_radius) / cfg.non_spatialized_radius).clamp(0.0, 1.0) // CLAMP-OK: fracção adimensional
    } else {
        1.0
    };
    // O seno do azimute. ⚠️ **Em cima do ouvinte (`d == 0`) não há direcção nenhuma**, e o valor
    // é o centro — que é o único que não inventa um lado.
    let raw = if d > 0.0 { dx / d } else { 0.0 };
    let pan = raw * spread * cfg.panning_strength.clamp(0.0, 1.0); // CLAMP-OK: fracção adimensional
    Spatial { pan, gain }
}

impl AudioSource2D {
    /// ⛔ **Esta fonte nunca se vai ouvir** — e o painel diz porquê.
    ///
    /// ⚠️ **Derivado, nunca guardado**, como o `never_fires` da tabela de acções. São as três
    /// maneiras de um som ficar mudo sem que nada esteja partido: sem ficheiro, sem alcance, ou no
    /// fundo do slider de volume.
    #[must_use]
    pub fn is_mute(&self) -> bool {
        self.sound.trim().is_empty() || self.max_distance <= 0.0 || self.volume_db <= -80.0
    }

    /// ⛔ **Ninguém a vai fazer tocar** — ela não arranca sozinha e nada a manda arrancar.
    ///
    /// ⚠️ **A segunda metade é do MUNDO e não está aqui:** *«nada a manda arrancar»* quer dizer
    /// que nenhuma linha de [`crate::SignalActions`] tem esta entidade por alvo com o verbo de
    /// tocar. Quem sabe isso é quem tem o mundo, e é por isso que esta função responde só pela
    /// metade local — o painel junta as duas.
    #[must_use]
    pub fn never_starts_by_itself(&self) -> bool {
        !self.autoplay
    }

    /// O barramento, já como enum.
    #[must_use]
    pub fn bus(&self) -> AudioBus {
        AudioBus::from_tag(self.bus_tag)
    }
}

/// **QUEM SÃO AS ORELHAS** — a entidade com [`AudioListener2D`], ou `None`.
///
/// ⚠️ **Com várias, ganha a de menor [`crate::StableId`]** — nunca a primeira da query. A ordem de
/// uma query é a ordem dos arquétipos e **muda quando um componente é inserido**; um ouvinte que
/// troca de objecto porque alguém anexou uma sprite noutro sítio é um defeito que não se
/// reproduz. É a mesma lei que o [`crate::signal_actions::resolve`] paga.
///
/// ⚠️ **Ela não recusa vários**, e é deliberado: recusar obrigaria a cena a estar correcta para se
/// ouvir alguma coisa, e uma cena a meio de ser montada não está. O painel é que diz — *«há N
/// ouvintes; este é o que manda»* — que é a informação, sem o castigo.
#[must_use]
pub fn listener_of(world: &mut World) -> Option<Entity> {
    world
        .query::<(Entity, &AudioListener2D, &crate::StableId)>()
        .iter(world)
        .map(|(e, _, s)| (s.0, e))
        .min_by_key(|(id, _)| *id)
        .map(|(_, e)| e)
}

/// Quantos ouvintes a cena tem — o número que o painel mostra quando não é `1`.
#[must_use]
pub fn listener_count(world: &mut World) -> usize {
    world.query::<&AudioListener2D>().iter(world).count()
}

#[cfg(test)]
#[path = "audio_2d_tests.rs"]
mod tests;
