//! **Shipping targets** (W6) — the platforms an asset is delivered to, and what each one
//! actually costs.
//!
//! # A variant is a different CLIP, not a different container
//!
//! The Delivery section already prices one clip under one codec, and the lesson it exists to
//! teach is that **the codec moves Disk and never RAM**: a Vorbis file and a WAV file decode
//! to the same `f32` buffer, so compressing an asset shrinks the download and buys back
//! exactly zero memory (ADR-0118 found the same thing from the other side — the codec picker
//! "did not save a single byte of RAM").
//!
//! Which means a "platform variant" that only swapped the codec would print **the same RAM
//! figure on every platform** — the precise lie that ADR-0118 was written to kill.
//!
//! So a [`Platform`] is a **format** first and a codec second. Mobile does not get "the same
//! sound as Opus"; it gets *a different sound* — fewer channels, fewer samples per second —
//! and that is what moves the number that actually bites (HR-13 gives the whole audio
//! subsystem 30 MB on an iPad, and one music bed can eat it).
//!
//! # The table is not allowed to lie
//!
//! **Opus is a 48 kHz codec and nothing else** ([`ph2d_audio_opus::OPUS_RATE`]) — it does not
//! store 24 kHz, it resamples up. So a platform that promised "24 kHz Opus" would show a RAM
//! figure a quarter of the truth, and the asset would land in the game at full rate anyway.
//! `a_platform_cannot_lie_about_its_sample_rate` forbids it, which is why Mobile below ships
//! Vorbis rather than the codec that is otherwise better per bit.

use ph2d_audio::{AudioFormat, ChannelLayout};

use crate::Codec;

/// One shipping target: what the asset **becomes**, and what carries it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Platform {
    /// ⛔⛔ **O IDENTIFICADOR do alvo — não é o que a aba diz, e já foi as duas coisas.**
    ///
    /// Ele é o infixo do ficheiro exportado (`{stem}.mobile.ogg`, por `id.to_lowercase()`), logo
    /// **traduzi-lo mudaria o nome dos ficheiros que o artista exporta conforme a língua da
    /// interface** — e uma pipeline de build que os procura pelo nome deixaria de os achar.
    ///
    /// ⚠️ A palavra que a aba diz é a chave [`Platform::label_key`]. *Quando um `&str` tem dois
    /// papéis, a cura é PARTI-LO* — e enquanto ele se chamou `name` a régua da fronteira
    /// acusava-o com razão, porque um campo `name` com cara de língua é indistinguível de um
    /// rótulo de catálogo.
    pub id: &'static str,
    /// ⭐ **A chave da palavra que a aba diz**, resolvida por quem pinta (`ph2d_i18n::tr`).
    ///
    /// ⚠️ Ela deriva do ALVO e não do [`Platform::id`]: atá-la ao id ligaria a tabela de strings ao
    /// nome dos ficheiros exportados, e a próxima renomeação de um deles partiria o outro.
    pub label_key: &'static str,
    /// The rate the asset is conformed to. This, and the layout, are what move RAM.
    pub sample_rate: u32,
    /// Mono halves the memory a stereo asset holds — the cheapest real saving there is.
    pub layout: ChannelLayout,
    /// What carries it to disk.
    pub codec: Codec,
    /// Quality / bitrate scalar for the lossy codecs; ignored by the lossless ones.
    pub quality: f32,
}

impl Platform {
    /// The format the asset is conformed INTO before it is encoded — the format the game will
    /// hold it in, and therefore the one the RAM figure has to be computed from.
    pub fn format(&self) -> AudioFormat {
        AudioFormat::new(self.sample_rate, self.layout)
    }
}

/// The targets the Export Set writes, in the order the panel lists them.
///
/// Each one is a real trade, spelled out:
///
/// - **Mobile** — RAM is the scarce thing (HR-13: 30 MB for *all* audio on an iPad), so this is
///   the only profile that touches the audio itself: 24 kHz, mono. That is **a quarter** of the
///   memory of a 48 kHz stereo master, which no choice of codec could have bought. Vorbis rather
///   than Opus **because Opus cannot store 24 kHz** — see the module docs.
/// - **Desktop** — memory is not scarce, bandwidth still is: full rate, full stereo, and the
///   codec with the best quality per bit we have.
/// - **Console** — ships on a disc or an SSD and decodes for free. Lossless, and it keeps the
///   loop points and the cue markers, which neither lossy codec carries.
pub const PLATFORMS: [Platform; 3] = [
    Platform {
        id: "Mobile",
        label_key: "audio.platform.mobile",
        sample_rate: 24_000,
        layout: ChannelLayout::Mono,
        codec: Codec::OggVorbis,
        quality: 0.3,
    },
    Platform {
        id: "Desktop",
        label_key: "audio.platform.desktop",
        sample_rate: 48_000,
        layout: ChannelLayout::Stereo,
        codec: Codec::Opus,
        quality: 0.5,
    },
    Platform {
        id: "Console",
        label_key: "audio.platform.console",
        sample_rate: 48_000,
        layout: ChannelLayout::Stereo,
        codec: Codec::Wav16,
        quality: 1.0,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// **A platform cannot promise a rate its codec does not store.**
    ///
    /// Opus is 48 kHz only. A profile declaring "24 kHz Opus" would price the RAM at a quarter
    /// of what the game will actually hold, because the encoder resamples up on the way in and
    /// the decoder hands back 48 kHz — and the whole point of this table is that the RAM figure
    /// is *true*. This is the gate that stops someone "improving" Mobile to Opus one day, which
    /// is exactly the tempting and wrong thing to do.
    #[test]
    fn a_platform_cannot_lie_about_its_sample_rate() {
        for p in PLATFORMS {
            if p.codec == Codec::Opus {
                assert_eq!(
                    p.sample_rate,
                    ph2d_audio_opus::OPUS_RATE,
                    "{}: Opus stores only {} Hz, so a {} Hz profile would price its RAM at the \
                     wrong rate -- the asset lands in the game at 48 kHz whatever this says",
                    p.id,
                    ph2d_audio_opus::OPUS_RATE,
                    p.sample_rate
                );
            }
        }
    }

    /// **The profiles are actually different**, or the set is three names for one file.
    #[test]
    fn every_platform_is_a_distinct_target() {
        for (i, a) in PLATFORMS.iter().enumerate() {
            for b in PLATFORMS.iter().skip(i + 1) {
                assert!(
                    a.format() != b.format() || a.codec != b.codec,
                    "{} and {} are the same target",
                    a.id,
                    b.id
                );
            }
        }
    }

    /// **Mobile is the one that saves memory, and it saves it by a factor of four.**
    ///
    /// Not a style point: it is the only profile that conforms the audio, and conforming the
    /// audio is the only thing that moves RAM at all. If someone "simplifies" Mobile to full
    /// rate and stereo, the platform preview goes back to printing the same RAM number three
    /// times -- which is the exact lie this table exists to stop.
    #[test]
    fn only_mobile_actually_buys_memory_back() {
        let desktop = PLATFORMS.iter().find(|p| p.id == "Desktop").unwrap();
        let mobile = PLATFORMS.iter().find(|p| p.id == "Mobile").unwrap();
        let ram_per_sec = |p: &Platform| p.sample_rate as usize * p.layout.count() * 4;
        assert!(
            ram_per_sec(mobile) * 4 <= ram_per_sec(desktop),
            "Mobile holds {} B/s against Desktop's {} B/s -- it is supposed to be a QUARTER, \
             and RAM is the budget that actually bites on a phone",
            ram_per_sec(mobile),
            ram_per_sec(desktop)
        );
    }
}
