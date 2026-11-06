//! **A porta única do som aprovado:** a receita vira um som do projeto, sem limitador e sem solo. Só o gesto do artista
//! passa aqui; o motor que muda só muda o aviso ([`estado`]), nunca o som.

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_encode::{BitDepth, encode_wav};
use ph2d_ecs::ProjectSounds;
use ph2d_sfx_dsp::{RenderError, render};
use ph2d_sfx_recipe::{Approved, ENGINE_V1, Recipe, SAMPLE_RATE, SfxLibrary};

/// O buffer da receita num WAV de `f32`: o caminho de qualquer som do projeto.
#[must_use]
pub fn em_wav(amostras: Vec<f32>) -> Vec<u8> {
    let dados = SampleData::from_interleaved(
        amostras,
        AudioFormat {
            sample_rate: SAMPLE_RATE,
            channels: ChannelLayout::Mono,
        },
    );
    encode_wav(&dados, BitDepth::Float32).unwrap_or_else(|e| panic!("wav: {e}"))
}

/// A impressão digital de um som como o projeto o guardaria.
#[must_use]
pub fn impressao(amostras: &[f32]) -> [u8; 32] {
    ProjectSounds::id_of(&em_wav(amostras.to_vec()))
}

#[derive(Debug, PartialEq)]
pub enum AprovarErro {
    SemReceita,
    Render(RenderError),
}

/// Rende a receita `nome` e guarda o som. Reaprovar substitui o mesmo nome, e por isso o som nunca é dividido com outra
/// receita: reaprovar uma não pode mudar a outra. Devolve o nome do som.
///
/// # Errors
/// Receita ausente, ou o render recusou.
pub fn aprovar(
    lib: &mut SfxLibrary,
    sons: &mut ProjectSounds,
    nome: &str,
) -> Result<String, AprovarErro> {
    let r = lib.recipes.get(nome).ok_or(AprovarErro::SemReceita)?;
    let pcm = render(r, r.seed, r.engine, &lib.takes).map_err(AprovarErro::Render)?;
    let bytes = em_wav(pcm.samples);
    let blake3 = ProjectSounds::id_of(&bytes);
    let engine = r.engine;
    let ja = r.approved.as_ref().map(|a| a.sound.as_str());
    let som = match ja.filter(|s| sons.get(s).is_some()) {
        Some(s) => s.to_owned(),
        None => {
            let q = format!("{nome}.wav");
            if sons.get(&q).is_some_and(|s| s.id == blake3) {
                q
            } else {
                sons.free_name(&q)
            }
        }
    };
    sons.set(&som, bytes);
    if let Some(r) = lib.recipes.get_mut(nome) {
        r.approved = Some(Approved {
            sound: som.clone(),
            blake3,
            engine,
        });
    }
    Ok(som)
}

/// O que a barra diz do som aprovado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EstadoDoAprovado {
    Nenhum,
    Aprovado,
    Mudou,
    MotorVelho,
}

/// `atual` é a impressão do render mais novo. Compara o SOM, não a receita: mudar e voltar ao mesmo som é «Approved».
#[must_use]
pub fn estado(r: &Recipe, atual: Option<&[u8; 32]>) -> EstadoDoAprovado {
    match &r.approved {
        None => EstadoDoAprovado::Nenhum,
        Some(a) if a.engine != ENGINE_V1 => EstadoDoAprovado::MotorVelho,
        Some(a) if atual.is_some_and(|i| *i != a.blake3) => EstadoDoAprovado::Mudou,
        Some(_) => EstadoDoAprovado::Aprovado,
    }
}
