#![forbid(unsafe_code)]
//! **O leitor do `.ase`/`.aseprite`** — o formato binário nativo do Aseprite, decodificado a partir
//! da especificação **pública** que o próprio projeto publica (`docs/ase-file-specs.md` no
//! repositório dele). ⚠️ **Clean-room a partir da SPEC, nunca do código**: o Aseprite é GPLv2, a
//! especificação é documentação, e ler um formato descrito não é obra derivada do programa que o
//! escreve. Nada aqui foi traduzido de fonte dele.
//!
//! # Porque uma crate-folha
//!
//! Ela não conhece o ECS, nem a GPU, nem o `Sprite` (ADR-0075: feature nova = **drop-crate**). A
//! entrada são **bytes**, a saída são **quadros RGBA8 + tags**, e é o shell que decide o que fazer
//! com eles. É a mesma fronteira que o `ph2d-asset::parse_atlas_meta` tem para o par
//! `.png` + `.json` — o outro caminho do Aseprite, que já existia.
//!
//! # O que este formato traz que o par `.png`+`.json` NÃO trazia
//!
//! ⚠️ **É por isto que ele foi pedido** (Enio, 2026-08-23: *«Precisamos Importar Aseprite (.ase)»*):
//! o par exportado traz **retângulos com nome**; o `.ase` traz a **AUTORIA** — as camadas, as
//! **tags** (nome, intervalo, direcção, repetições) e a **duração de cada quadro**. As tags são,
//! literalmente, o modelo da §11 Animation: um intervalo nomeado sobre as células que a sprite tem.
//!
//! ⚠️ **E a duração por-FRAME reabre uma recusa medida** (spec §8.12): a §11 guarda **um**
//! `frame_ms` por tag, e a recusa de então dizia *«não há quem produza durações por-quadro»*. Há:
//! é este importador. Esta crate **devolve o que o ficheiro diz** — a decisão de o modelo passar a
//! guardá-las é de produto, e o consumidor é que a toma (ver [`AseTag::uniform_duration_ms`]).
//!
//! # O que fica de fora, com o motivo
//!
//! Cada limite conhecido sai num [`AseDoc::notes`] — uma linha de texto que o shell mostra ao
//! artista. ⛔ *Um importador que ignora em silêncio é pior que um que recusa*: o desenho aparece
//! quase certo, e ninguém sabe porquê.
//!
//! * **Tilemaps** (cel tipo 3 / camadas tilemap) — outro modelo de dados, não uma folha de sprites.
//! * **`z-index` de cel** (Aseprite 1.3) — reordena cels dentro de um quadro; raro, e ignorá-lo é
//!   visível só num ficheiro que o use.
//! * **Modo de mistura de um GRUPO** — os grupos aqui são visibilidade e nada mais (*pass-through*,
//!   que é o default deles).

mod blend;
mod composite;
pub mod file;
mod read;
mod write;

pub use blend::BlendMode;
pub use file::{
    AseCel, AseCelBody, AseFile, AseFileFrame, AseLayer, AseSlice, AseSliceKey, AseTilemap,
    AseTileset, parse_file,
};
pub use write::{WriteError, write_file};

/// O pivot gravado numa key SEM pivot de uma slice com a flag (a flag é por slice): `0,0` é um pivot
/// real, por isso a ausência grava-se fora de qualquer canvas (o Trial aceita pivot fora sem clamp e
/// não o desenha: `P11_oraculos_arquivo.md`, `weird.badpivot`) e lê-se de volta como `None`.
pub const NO_PIVOT: (i32, i32) = (i32::MIN, i32::MIN);

/// Um documento `.ase` decodificado: os quadros já **compostos** em RGBA8 e a autoria.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AseDoc {
    pub width: u16,
    pub height: u16,
    /// Um por quadro, na ordem do ficheiro — e essa ordem **é** o índice que as tags referem.
    pub frames: Vec<AseFrame>,
    pub tags: Vec<AseTag>,
    /// O que o ficheiro trazia e este leitor não honrou, em linguagem de artista. Vazio = nada
    /// ficou por trás.
    pub notes: Vec<String>,
}

/// Um quadro composto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AseFrame {
    /// Quanto ele dura, em milissegundos — o número que o Aseprite guarda **por quadro**.
    pub duration_ms: u16,
    /// `width * height * 4` bytes, RGBA8 de alfa **RETO** (não pré-multiplicado), que é como o
    /// resto do app trata os pixels de um `.png`.
    pub rgba: Vec<u8>,
}

/// Uma tag do Aseprite — o intervalo nomeado que a §11 chama de animação.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AseTag {
    pub name: String,
    /// Primeiro quadro, **inclusive**.
    pub from: u16,
    /// Último quadro, **inclusive**.
    pub to: u16,
    /// `0` avança · `1` recua · `2` vai-e-volta · `3` volta-e-vai. ⚠️ A ordem é **a mesma** do
    /// `AnimDirection::ALL` da §11, e essa coincidência é do formato, não uma escolha nossa — quem
    /// converter deve na mesma passar pelo construtor de lá, e não pelo número.
    pub direction: u8,
    /// Quantas voltas a tag dá. `0` = para sempre (Aseprite 1.3; ficheiros mais antigos trazem 0).
    pub repeat: u16,
    /// A cor da tag no chunk (o campo deprecado que todos os leitores entendem).
    pub rgb: [u8; 3],
}

impl AseTag {
    /// **A duração que esta tag tem, se ela tiver uma só.** `Some(ms)` quando todos os quadros do
    /// intervalo duram o mesmo; `None` quando o artista variou a duração dentro da tag — que é
    /// exactamente o caso que o modelo de um `frame_ms` por tag não sabe exprimir.
    ///
    /// ⚠️ **Devolver `None` é a informação**, e não um erro: quem chama decide se aproxima (e
    /// avisa) ou se abre a wave que põe a duração por-quadro no modelo.
    #[must_use]
    pub fn uniform_duration_ms(&self, frames: &[AseFrame]) -> Option<u16> {
        let lo = usize::from(self.from.min(self.to));
        let hi = usize::from(self.from.max(self.to)).min(frames.len().saturating_sub(1));
        let span = frames.get(lo..=hi)?;
        let first = span.first()?.duration_ms;
        span.iter().all(|f| f.duration_ms == first).then_some(first)
    }

    /// A duração **mais comum** do intervalo — a aproximação honesta quando não há uma só.
    ///
    /// Empate resolve-se pelo menor valor, para o resultado não depender da ordem do mapa.
    #[must_use]
    pub fn dominant_duration_ms(&self, frames: &[AseFrame]) -> u16 {
        let lo = usize::from(self.from.min(self.to));
        let hi = usize::from(self.from.max(self.to)).min(frames.len().saturating_sub(1));
        let Some(span) = frames.get(lo..=hi) else {
            return 100;
        };
        let mut best = (0_usize, u16::MAX);
        for f in span {
            let n = span
                .iter()
                .filter(|g| g.duration_ms == f.duration_ms)
                .count();
            if n > best.0 || (n == best.0 && f.duration_ms < best.1) {
                best = (n, f.duration_ms);
            }
        }
        if best.0 == 0 { 100 } else { best.1 }
    }
}

/// Porque é que um ficheiro não pôde ser lido. ⚠️ Cada variante **nomeia o que estava errado** — um
/// «parse error» genérico obriga o artista a adivinhar se o ficheiro está corrompido, se é de uma
/// versão futura, ou se ele largou o ficheiro errado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AseError {
    /// Acabaram os bytes a meio de uma estrutura.
    Truncated(&'static str),
    /// O número mágico do cabeçalho não é `0xA5E0` — não é um `.ase`.
    NotAseprite,
    /// O número mágico de um quadro não é `0xF1FA`.
    BadFrameMagic(usize),
    /// Profundidade de cor que a spec não define (só 8, 16 e 32).
    UnknownColorDepth(u16),
    /// O ficheiro diz ter zero quadros.
    NoFrames,
    /// Um quadro comprimido não descomprime.
    BadZlib(usize),
    /// Largura ou altura zero — não há imagem nenhuma.
    EmptyCanvas,
}

impl std::fmt::Display for AseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Truncated(what) => write!(f, "the file ends in the middle of the {what}"),
            Self::NotAseprite => write!(f, "not an Aseprite file (bad magic number)"),
            Self::BadFrameMagic(i) => write!(f, "frame {i} has a bad magic number"),
            Self::UnknownColorDepth(d) => write!(f, "unknown colour depth: {d} bits per pixel"),
            Self::NoFrames => write!(f, "the file declares no frames"),
            Self::BadZlib(i) => write!(f, "frame {i} has a cel that does not decompress"),
            Self::EmptyCanvas => write!(f, "the canvas is empty (zero width or height)"),
        }
    }
}

impl std::error::Error for AseError {}

/// **Lê um `.ase` inteiro e compõe-o.** Função pura sobre bytes — não toca no disco, não fala com a GPU.
///
/// # Errors
/// [`AseError`], uma variante por causa nomeada.
pub fn parse(bytes: &[u8]) -> Result<AseDoc, AseError> {
    composite::compose(&parse_file(bytes)?)
}
