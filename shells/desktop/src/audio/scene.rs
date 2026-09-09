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
use ph2d_ecs::{AudioBus, AudioSource2D, Spatial};

/// Uma voz viva de um objecto.
struct Live {
    voice: VoiceId,
    /// Quando ela deixa de soar. `None` = em ciclo (só um `stop` a cala).
    ends_at: Option<Instant>,
}

/// **O livro das vozes da cena.**
#[derive(Default)]
pub(crate) struct SceneAudio {
    /// O que já foi descodificado, por caminho.
    ///
    /// ⚠️ **`None` guarda a FALHA**, e é isso que impede uma tentativa por quadro: um caminho que
    /// não abre não abre, e voltar a tentar 60 vezes por segundo daria uma linha de erro por
    /// quadro sobre um facto que não muda.
    decoded: BTreeMap<String, Option<SampleData>>,
    /// As vozes vivas, por `StableId`.
    voices: BTreeMap<u64, Vec<Live>>,
    /// Quem já NASCEU — a aresta que o `autoplay` lê.
    ///
    /// ⚠️ **É a lei do `reconcile` do timer, e ela foi paga por um report:** *«começar a tocar» é
    /// uma ARESTA, e a aresta é o nascimento*. Ler *«não está a tocar»* faria um som de uma vez só
    /// renascer a cada quadro depois de acabar.
    born: BTreeSet<u64>,
}

impl SceneAudio {
    /// **Este objecto acabou de nascer?** Marca-o como nascido no mesmo passo.
    pub(crate) fn take_birth(&mut self, key: u64) -> bool {
        self.born.insert(key)
    }

    /// Quantas vozes de cena estão vivas — o número que o smoke imprime.
    pub(crate) fn live_voices(&self) -> usize {
        self.voices.values().map(Vec::len).sum()
    }

    /// Quantos sons distintos já foram descodificados com sucesso.
    ///
    /// ⚠️ **Só o gate a consulta**, e é por isso que ela é `cfg(test)`: uma sonda `pub(crate)` que
    /// ninguém do produto lê é exactamente um `dead_code` a ser tolerado em silêncio.
    #[cfg(test)]
    pub(crate) fn decoded_ok(&self) -> usize {
        self.decoded.values().filter(|d| d.is_some()).count()
    }

    /// **TOCA o som de um objecto.** `false` = não havia o que tocar (sem ficheiro, ou ele não
    /// abre) ou o mixer recusou o comando.
    ///
    /// ⚠️ **A polifonia é honrada AQUI**, e da ponta mais VELHA: uma fonte com `max_polyphony = 1`
    /// (o default) **substitui** a voz anterior, que é o que um passo ou um clique de porta querem.
    /// Deixar acumular transformaria um sinal repetido numa parede de som.
    pub(crate) fn play(
        &mut self,
        eng: &mut AudioEngine,
        key: u64,
        cfg: &AudioSource2D,
        sp: Spatial,
    ) -> bool {
        if cfg.sound.trim().is_empty() {
            return false;
        }
        let Some(data) = self.sample(&cfg.sound) else {
            return false;
        };
        let pitch = if cfg.pitch > 0.0 { cfg.pitch } else { 1.0 };
        let params = PlayParams {
            gain: sp.gain,
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
        });
        true
    }

    /// **CALA** o que este objecto tem a soar. `false` = não tinha nada.
    pub(crate) fn stop(&mut self, eng: &AudioEngine, key: u64) -> bool {
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
    pub(crate) fn update(&self, eng: &AudioEngine, key: u64, sp: Spatial) {
        let Some(slot) = self.voices.get(&key) else {
            return;
        };
        for l in slot {
            let _ = eng.set_voice_gain(l.voice, sp.gain);
            let _ = eng.set_voice_pan(l.voice, sp.pan);
        }
    }

    /// Retira as vozes que já acabaram — ver o cabeçalho para porque isto é uma conta.
    pub(crate) fn retire(&mut self) {
        let agora = Instant::now();
        self.voices.retain(|_, slot| {
            slot.retain(|l| l.ends_at.is_none_or(|t| t > agora));
            !slot.is_empty()
        });
    }

    /// **Esquece quem já não está na cena** — e cala o que ele tinha a soar.
    ///
    /// ⚠️ **As DUAS metades, e a segunda é a que se esquece:** apagar do livro sem parar as vozes
    /// deixaria um som órfão a tocar para sempre, sem dono que o pudesse calar. É o modo de falha
    /// de um `Ctrl+Z` sobre um objecto que estava a soar.
    ///
    /// ⚠️ **O `born` também se limpa**, senão um objecto apagado e reposto (um undo) voltaria
    /// **sem** a aresta de nascimento e o `autoplay` dele nunca mais valeria.
    pub(crate) fn forget_absent(&mut self, eng: &AudioEngine, vivos: &BTreeSet<u64>) {
        let mortos: Vec<u64> = self
            .voices
            .keys()
            .copied()
            .filter(|k| !vivos.contains(k))
            .collect();
        for k in mortos {
            self.stop(eng, k);
        }
        self.born.retain(|k| vivos.contains(k));
    }

    /// A amostra deste caminho, descodificada uma vez só. ⚠️ Ver o doc do [`Self::decoded`].
    fn sample(&mut self, path: &str) -> Option<SampleData> {
        if let Some(cached) = self.decoded.get(path) {
            return cached.clone();
        }
        let out = std::fs::read(path)
            .ok()
            .and_then(|bytes| super::decode_any::decode(&bytes).ok());
        if out.is_none() {
            eprintln!("audio: a fonte de cena nao conseguiu abrir `{path}`");
        }
        self.decoded.insert(path.to_string(), out.clone());
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
