//! O linter de pixel art (P13 bloco A, `docs/PixelArt/P13_plano.md` §4): «que vícios tem este cel?».
//!
//! [`lint`] compõe o quadro como a exportação o vê (sem camadas de referência) e corre as regras
//! ligadas em [`Regras`] sobre o RGBA. Cada regra escreve a sua lei no módulo dela, em `regra::*`.
//! Um pixel é OPACO quando `alfa > 0`; duas células têm a MESMA cor quando o RGBA é igual.

mod img;
mod regra;

use ph2d_pixel_core::{FrameId, Purpose, Sprite, compose_window};

pub use img::Img;

/// As regras do linter, na ordem em que o painel as lista.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Regra {
    /// Pixel solto, longe do desenho ([`regra::orfao`]).
    Orfao,
    /// Canto em L numa linha de 1 px ([`regra::canto_duplo`]).
    CantoDuplo,
    /// Degraus de linha sem ritmo ([`regra::degrau`]).
    Degrau,
    /// Faixa intermédia de 1 px entre duas fronteiras paralelas ([`regra::faixa`]).
    Faixa,
    /// Sombra que só segue o contorno ([`regra::almofada`]).
    Almofada,
    /// Mais cores do que o limite ([`regra::cores`]).
    CoresAMais,
}

impl Regra {
    pub const TODAS: [Regra; 6] = [
        Regra::Orfao,
        Regra::CantoDuplo,
        Regra::Degrau,
        Regra::Faixa,
        Regra::Almofada,
        Regra::CoresAMais,
    ];
}

/// Quão grave é o aviso (a ordem do painel: mais grave primeiro).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severidade {
    Baixa,
    Media,
    Alta,
}

/// Um vício achado: a regra, as células `[x, y]` (ordenadas por linha, depois coluna) e a
/// severidade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aviso {
    pub regra: Regra,
    pub celulas: Vec<[u32; 2]>,
    pub severidade: Severidade,
}

/// Que regras correm, e o limite de cores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Regras {
    pub orfao: bool,
    pub canto_duplo: bool,
    pub degrau: bool,
    pub faixa: bool,
    pub almofada: bool,
    pub cores_a_mais: bool,
    /// Quantas cores opacas distintas o cel pode usar antes de [`Regra::CoresAMais`].
    pub limite_cores: usize,
}

/// O limite de cores por omissão: o tamanho da DB32, a paleta de fábrica.
pub const LIMITE_CORES_PADRAO: usize = 32;

impl Default for Regras {
    fn default() -> Self {
        Self {
            orfao: true,
            canto_duplo: true,
            degrau: true,
            faixa: true,
            almofada: true,
            cores_a_mais: true,
            limite_cores: LIMITE_CORES_PADRAO,
        }
    }
}

impl Regras {
    #[must_use]
    pub fn ligada(&self, r: Regra) -> bool {
        match r {
            Regra::Orfao => self.orfao,
            Regra::CantoDuplo => self.canto_duplo,
            Regra::Degrau => self.degrau,
            Regra::Faixa => self.faixa,
            Regra::Almofada => self.almofada,
            Regra::CoresAMais => self.cores_a_mais,
        }
    }
}

/// Os vícios do quadro `frame` de `sprite`, composto como a exportação o vê.
#[must_use]
pub fn lint(sprite: &Sprite, frame: FrameId, regras: &Regras) -> Vec<Aviso> {
    let rgba = compose_window(sprite, frame, None, sprite.bounds(), Purpose::Export);
    lint_rgba(&Img::new(sprite.width(), sprite.height(), &rgba), regras)
}

/// O mesmo sobre um RGBA já composto (alfa reto, `w * h * 4` bytes).
#[must_use]
pub fn lint_rgba(img: &Img<'_>, regras: &Regras) -> Vec<Aviso> {
    let mut out = Vec::new();
    for r in Regra::TODAS {
        if !regras.ligada(r) {
            continue;
        }
        let (sev, grupos) = match r {
            Regra::Orfao => (Severidade::Baixa, regra::orfao::acha(img)),
            Regra::CantoDuplo => (Severidade::Media, regra::canto_duplo::acha(img)),
            Regra::Degrau => (Severidade::Media, regra::degrau::acha(img)),
            Regra::Faixa => (Severidade::Media, regra::faixa::acha(img)),
            Regra::Almofada => (Severidade::Alta, regra::almofada::acha(img)),
            Regra::CoresAMais => (
                Severidade::Baixa,
                regra::cores::acha(img, regras.limite_cores),
            ),
        };
        for mut celulas in grupos {
            celulas.sort_by_key(|c| (c[1], c[0]));
            celulas.dedup();
            out.push(Aviso {
                regra: r,
                celulas,
                severidade: sev,
            });
        }
    }
    out
}
