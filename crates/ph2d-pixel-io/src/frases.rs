//! **As frases do leitor** (HR-15): o que o `abrir`/`ase::ler` avisa ou recusa, em inglês por chave
//! i18n. Moram com o leitor porque o Pixel (painel) e o import do Sprite Inspector (folha) as leem.

use ph2d_i18n::{tr, tr_with};

use crate::abrir::ErroAbrir;
use crate::ase::Aviso;

#[must_use]
pub fn aviso_ase(a: Aviso) -> String {
    match a {
        Aviso::Tilemap => tr("pixel.io.warn.tilemap").to_owned(),
        Aviso::TerrenosFicam => tr("pixel.io.warn.terrains").to_owned(),
        Aviso::FormaVaiComoImagem => tr("pixel.io.warn.grid_shape").to_owned(),
        Aviso::PaletaPorQuadro => tr("pixel.io.warn.palette_per_frame").to_owned(),
        Aviso::ModoDesconhecido => tr("pixel.io.warn.unknown_blend").to_owned(),
        Aviso::DuracaoCortada => tr("pixel.io.warn.duration_cut").to_owned(),
        Aviso::CelCorrompido => tr("pixel.io.warn.bad_cel").to_owned(),
    }
}

#[must_use]
pub fn erro_abrir(e: &ErroAbrir) -> String {
    match e {
        ErroAbrir::Formato(x) => tr_with("pixel.io.err.type", &[("e", x)]),
        ErroAbrir::Vazio => tr("pixel.io.err.empty").to_owned(),
        ErroAbrir::TamanhosDiferentes => tr("pixel.io.err.sizes").to_owned(),
        ErroAbrir::FolhaPartida => tr("pixel.io.err.split_sheet").to_owned(),
        ErroAbrir::Ler(d) => tr_with("pixel.io.err.detail", &[("e", d)]),
        ErroAbrir::Documento(_) => tr("pixel.io.err.invalid").to_owned(),
    }
}
