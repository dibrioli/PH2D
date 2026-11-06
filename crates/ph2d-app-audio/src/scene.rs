//! ⭐⭐⭐ **O SOM DA CENA** — o livro das vozes que os objectos têm a soar (TOP-20 #4).
//!
//! # Porque este livro existe, e porque ele não é um componente
//!
//! Uma voz é um recurso do DISPOSITIVO: ela nasce quando o mixer aceita o comando e morre quando a
//! amostra acaba ou alguém a rouba. Nada disso é documento — e o `Timer` já pagou a lei que o diz:
//! *um componente registado que anda a 60 Hz faz cada quadro com entrada virar um passo de undo*.
//! ⇒ o que anda vive aqui, ao lado do dispositivo, e o componente guarda só o que o artista
//! escreveu.
//!
//! # ⚠️ A chave é o `StableId`, nunca a `Entity`
//!
//! O undo respawna tudo com bits novos. Um livro chaveado por `Entity` perderia as vozes de um
//! objecto sempre que alguém carregasse em `Ctrl+Z` — as vozes continuariam a soar, sem dono, e
//! ninguém as poderia calar. É a mesma lei que o alvo do `SignalActions` já paga, e é por isso que
//! `AudioSource2D` não tem de guardar identidade nenhuma.
//!
//! # ⚠️ «Acabou» é uma CONTA, e não uma pergunta ao mixer
//!
//! O lado de controlo do `ph2d-audio` **não sabe** se uma voz ainda soa: o `active_voices()` vive no
//! `AudioRenderer`, do outro lado do ring, na thread do `cpal`. ⇒ o livro calcula a duração da
//! amostra (`frames / taxa / pitch`) e retira a voz quando ela passa.
//!
//! ⚠️ **Os dois podem discordar, e a discordância é benigna:** o mixer rouba a voz mais
//! velha-mais-fraca quando o pool enche, então uma voz pode morrer antes da conta. O que o livro
//! faz com uma voz que já não existe é mandar-lhe um `set_voice_pan` que o mixer ignora. ⛔ A
//! alternativa — um canal de retorno por voz — é maquinaria de RT para uma resposta que ninguém
//! consulta.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use ph2d_audio::{AudioEngine, BusId, PlayParams, SampleData, VoiceId};
use ph2d_ecs::{AudioBus, AudioSource2D, ProjectSounds, SoundId, SoundSet, Spatial};
use ph2d_sound_set::{Jitter, Picker};

/// Uma voz viva de um objecto.
struct Live {
    voice: VoiceId,
    /// Quando ela deixa de soar. `None` = em ciclo (só um `stop` a cala).
    ends_at: Option<Instant>,
    /// O ganho sorteado neste disparo: [`SceneAudio::update`] reescreve o ganho a cada quadro e tem de o manter.
    jitter_gain: f32,
}

/// **O livro das vozes da cena.**
#[derive(Default)]
pub struct SceneAudio {
    /// O que já foi descodificado, pelo CONTEÚDO (GEF 21 E2: os bytes vêm da [`ProjectSounds`]).
    ///
    /// ⚠️ **`None` guarda a FALHA**, e é isso que impede uma tentativa por quadro: uns bytes que
    /// não descodificam não descodificam, e voltar a tentar 60 vezes por segundo daria uma linha
    /// de erro por quadro sobre um facto que não muda.
    decoded: BTreeMap<SoundId, Option<SampleData>>,
    /// As vozes vivas, por `StableId`.
    voices: BTreeMap<u64, Vec<Live>>,
    /// Quem já NASCEU — a aresta que o `autoplay` lê.
    ///
    /// ⚠️ **É a lei do `reconcile` do timer, e ela foi paga por um report:** *«começar a tocar» é
    /// uma ARESTA, e a aresta é o nascimento*. Ler *«não está a tocar»* faria um som de uma vez só
    /// renascer a cada quadro depois de acabar.
    born: BTreeSet<u64>,
    /// As vozes de DISPARO de quem saiu numa corrida — acabam de soar sem dono (GEF 21 E3).
    tails: Vec<Live>,
    /// O sorteio de cada objecto. Fora do componente: ele anda a cada disparo.
    pickers: BTreeMap<u64, Picker>,
    /// A corrida estava ligada no quadro anterior — a borda do Stop.
    em_corrida: bool,
    /// O som e o desvio do último disparo.
    ultimo: Option<(String, Jitter)>,
}

impl SceneAudio {
    /// **Este objecto acabou de nascer?** Marca-o como nascido no mesmo passo.
    pub fn take_birth(&mut self, key: u64) -> bool {
        self.born.insert(key)
    }

    /// Quantas vozes de cena estão vivas — o número que o smoke imprime.
    pub fn live_voices(&self) -> usize {
        self.voices.values().map(Vec::len).sum::<usize>() + self.tails.len()
    }

    /// O som e o desvio do último disparo: a porta da sonda em `shells/game-scenes`.
    #[must_use]
    pub fn last_pick(&self) -> Option<(&str, Jitter)> {
        self.ultimo.as_ref().map(|(n, j)| (n.as_str(), *j))
    }

    /// Quantos sons distintos já foram descodificados com sucesso.
    ///
    /// ⚠️ **Só o gate a consulta**, e é por isso que ela é gateada: uma sonda que ninguém do
    /// produto lê é exactamente um `dead_code` a ser tolerado em silêncio.
    ///
    /// ⚠️ `cfg(test)` e já não `any(test, feature = "test-support")`: o gate que a lê (o da ponte do
    /// som, `audio_2d_tests.rs`) desceu da shell para esta crate em 2026-09-25, e a feature com ele.
    #[cfg(test)]
    pub fn decoded_ok(&self) -> usize {
        self.decoded.values().filter(|d| d.is_some()).count()
    }

    /// Quantas FALHAS de descodificação estão guardadas (o que impede a tentativa por quadro).
    #[cfg(test)]
    pub fn decoded_failed(&self) -> usize {
        self.decoded.values().filter(|d| d.is_none()).count()
    }

    /// **O Stop**: na borda em que a corrida acaba, tudo cala e esquece quem nasceu, para o Play seguinte
    /// arrancar o Autoplay de novo. Um objecto reposto pelo Stop volta com o mesmo `StableId`, então o
    /// [`Self::forget_absent`] não o veria sair.
    pub fn run_edge(&mut self, eng: &AudioEngine, corrida: bool) {
        if self.em_corrida && !corrida {
            let vozes = std::mem::take(&mut self.voices);
            for l in vozes.into_values().flatten().chain(self.tails.drain(..)) {
                let _ = eng.stop(l.voice);
            }
            self.born.clear();
            self.pickers.clear();
        }
        self.em_corrida = corrida;
    }

    /// **Qual som toca agora**: a única porta do sorteio. Com sons ligados no `set` ele manda; sem, o `sound`
    /// de sempre. O sorteio de cada objecto tem semente própria, e o `Stop` recomeça-o.
    fn pick_sound<'a>(
        &mut self,
        key: u64,
        cfg: &'a AudioSource2D,
        set: Option<&'a SoundSet>,
    ) -> Option<(&'a str, Jitter)> {
        let Some(s) = set.filter(|s| s.has_sounds()) else {
            let nome = cfg.sound.trim();
            return (!nome.is_empty()).then_some((nome, Jitter::NONE));
        };
        let picker = self
            .pickers
            .entry(key)
            .or_insert_with(|| Picker::with_seed(key ^ 0xA5A5_5A5A_C3C3_3C3C));
        let i = picker.pick(&s.candidates(), s.strategy, s.no_repeat)?;
        let jitter = picker.jitter(s.pitch_jitter_semitones, s.gain_jitter_db);
        Some((s.sounds[i].name.as_str(), jitter))
    }

    /// **TOCA o som de um objecto.** `false` = não havia o que tocar (sem nome, um nome que os
    /// sons do projecto não têm, ou bytes que não descodificam) ou o mixer recusou o comando.
    ///
    /// ⚠️ **A polifonia é honrada AQUI**, e da ponta mais VELHA: uma fonte com `max_polyphony = 1`
    /// (o default) **substitui** a voz anterior, que é o que um passo ou um clique de porta querem.
    /// Deixar acumular transformaria um sinal repetido numa parede de som.
    pub fn play(
        &mut self,
        eng: &mut AudioEngine,
        key: u64,
        cfg: &AudioSource2D,
        sp: Spatial,
        set: Option<&SoundSet>,
        sons: Option<&ProjectSounds>,
    ) -> bool {
        let Some((nome, jitter)) = self.pick_sound(key, cfg, set) else {
            return false;
        };
        self.ultimo = Some((nome.to_owned(), jitter));
        let Some(data) = sons.and_then(|l| self.sample(l, nome)) else {
            return false;
        };
        let pitch = if cfg.pitch > 0.0 { cfg.pitch } else { 1.0 } * jitter.pitch;
        let params = PlayParams {
            gain: sp.gain * jitter.gain,
            pan: sp.pan,
            pitch,
            looping: cfg.looping,
            bus: bus_of(cfg.bus()),
            ..PlayParams::default()
        };
        let fmt = data.format();
        let secs = fmt.frames_to_secs(data.frame_count() as u64) / f64::from(pitch);
        let Ok(voice) = eng.play(data, params) else {
            return false;
        };
        let slot = self.voices.entry(key).or_default();
        // ⚠️ **Corta pela ponta VELHA antes de pôr a nova** — senão o teto seria N+1 por um quadro,
        // e uma rajada num `max_polyphony = 1` deixaria sempre duas a soar.
        let teto = cfg.max_polyphony.max(1) as usize;
        while slot.len() >= teto {
            let velha = slot.remove(0);
            let _ = eng.stop(velha.voice);
        }
        slot.push(Live {
            voice,
            ends_at: if cfg.looping {
                None
            } else {
                Instant::now().checked_add(Duration::from_secs_f64(secs.max(0.0)))
            },
            jitter_gain: jitter.gain,
        });
        true
    }

    /// **CALA** o que este objecto tem a soar. `false` = não tinha nada.
    pub fn stop(&mut self, eng: &AudioEngine, key: u64) -> bool {
        let Some(slot) = self.voices.remove(&key) else {
            return false;
        };
        let havia = !slot.is_empty();
        for l in slot {
            let _ = eng.stop(l.voice);
        }
        havia
    }

    /// Reescreve o ganho e o pan das vozes vivas deste objecto — o que faz o som **seguir** quem
    /// se move.
    pub fn update(&self, eng: &AudioEngine, key: u64, sp: Spatial) {
        let Some(slot) = self.voices.get(&key) else {
            return;
        };
        for l in slot {
            let _ = eng.set_voice_gain(l.voice, sp.gain * l.jitter_gain);
            let _ = eng.set_voice_pan(l.voice, sp.pan);
        }
    }

    /// Retira as vozes que já acabaram — ver o cabeçalho para porque isto é uma conta.
    pub fn retire(&mut self) {
        let agora = Instant::now();
        self.voices.retain(|_, slot| {
            slot.retain(|l| l.ends_at.is_none_or(|t| t > agora));
            !slot.is_empty()
        });
        self.tails.retain(|l| l.ends_at.is_some_and(|t| t > agora));
    }

    /// **Esquece quem já não está na cena** — e cala o que ele tinha a soar.
    ///
    /// ⚠️ **As DUAS metades, e a segunda é a que se esquece:** apagar do livro sem parar as vozes
    /// deixaria um som órfão a tocar para sempre, sem dono que o pudesse calar. É o modo de falha
    /// de um `Ctrl+Z` sobre um objecto que estava a soar.
    ///
    /// ⚠️ **O `born` também se limpa**, senão um objecto apagado e reposto (um undo) voltaria
    /// **sem** a aresta de nascimento e o `autoplay` dele nunca mais valeria.
    ///
    /// ⭐ **Numa corrida (`deixa_acabar`) a voz de DISPARO acaba de soar** (GEF 21 E3): a moeda que
    /// toca e sai no mesmo quadro tem de se ouvir. A de ciclo cala-se sempre — sem dono, ninguém a
    /// pararia. Fora da corrida (o `Ctrl+Z`) cala-se tudo, como antes.
    pub fn forget_absent(&mut self, eng: &AudioEngine, vivos: &BTreeSet<u64>, deixa_acabar: bool) {
        let mortos: Vec<u64> = self
            .voices
            .keys()
            .copied()
            .filter(|k| !vivos.contains(k))
            .collect();
        for k in mortos {
            let Some(slot) = self.voices.remove(&k) else {
                continue;
            };
            for l in slot {
                if deixa_acabar && l.ends_at.is_some() {
                    self.tails.push(l);
                } else {
                    let _ = eng.stop(l.voice);
                }
            }
        }
        self.born.retain(|k| vivos.contains(k));
        self.pickers.retain(|k, _| vivos.contains(k));
    }

    /// A amostra deste caminho, descodificada uma vez só. ⚠️ Ver o doc do [`Self::decoded`].
    fn sample(&mut self, sons: &ProjectSounds, name: &str) -> Option<SampleData> {
        let som = sons.get(name)?;
        if let Some(cached) = self.decoded.get(&som.id) {
            return cached.clone();
        }
        let out = ph2d_audio_decode::decode_any::decode(&som.bytes).ok();
        if out.is_none() {
            eprintln!("audio: o som do projecto `{name}` nao descodifica");
        }
        self.decoded.insert(som.id, out.clone());
        out
    }
}

/// A tag da cena → o barramento do mixer.
///
/// ⚠️ **A tradução vive AQUI**, do lado que conhece os dois enums — é a lei do ADR-0029 que o
/// `SignalVerb` já paga. ⛔ E o `BusId::Ui` não é alcançável a partir de nenhuma tag: ele é a voz do
/// chrome, que uma preferência do utilizador desliga.
fn bus_of(b: AudioBus) -> BusId {
    match b {
        AudioBus::Sfx => BusId::Sfx,
        AudioBus::Music => BusId::Music,
        AudioBus::Voice => BusId::Voice,
        AudioBus::Master => BusId::Master,
    }
}
