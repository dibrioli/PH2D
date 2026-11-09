//! **As frases do que o motor avisa ou recusa** (HR-15): os `Display` do `ph2d-pixel-io` são para
//! logs; o artista lê estas.

use std::path::PathBuf;

use ph2d_i18n::{tr, tr_with};
use ph2d_pixel_core::Sprite;
use ph2d_pixel_io::abrir::{Aberto, ErroAbrir};
use ph2d_pixel_io::ase::{AseErro, Aviso};
use ph2d_pixel_io::exporta::{ErroGravar, Export, Gravado};
use ph2d_pixel_io::saida::{AvisoSaida, ErroSaida, Formato};

/// As frases do leitor moram com ele (o import do Sprite Inspector, uma folha, também as lê).
pub use ph2d_pixel_io::frases::{aviso_ase, erro_abrir};

#[must_use]
pub fn aviso_saida(a: AvisoSaida) -> String {
    match a {
        AvisoSaida::GifReduziu(n) => tr_with("pixel.io.warn.gif_colors", &[("n", &n.to_string())]),
        AvisoSaida::GifAlfa => tr("pixel.io.warn.gif_alpha").to_owned(),
        AvisoSaida::Ase(x) => aviso_ase(x),
    }
}

fn erro_ase(e: &AseErro) -> String {
    match e {
        AseErro::QuadrosDemais(n) => tr_with("pixel.io.err.ase_frames", &[("n", &n.to_string())]),
        AseErro::CelForaDoFormato { .. } => tr("pixel.io.err.ase_cel").to_owned(),
        outro => tr_with("pixel.io.err.detail", &[("e", &outro.to_string())]),
    }
}

#[must_use]
pub fn erro_gravar(e: &ErroGravar) -> String {
    match e {
        ErroGravar::Saida(ErroSaida::GifGrandeDemais { .. }) => {
            tr("pixel.io.err.gif_size").to_owned()
        }
        ErroGravar::Saida(ErroSaida::NadaNaArea) => tr("pixel.io.error.nothing_in_area").to_owned(),
        ErroGravar::Saida(ErroSaida::GrandeDemais { .. }) => {
            tr("pixel.io.err.too_large").to_owned()
        }
        ErroGravar::Saida(ErroSaida::Ase(a)) => erro_ase(a),
        ErroGravar::Saida(ErroSaida::Formato(d)) => tr_with("pixel.io.err.detail", &[("e", d)]),
        ErroGravar::Disco(c, d) => tr_with(
            "pixel.io.err.disk",
            &[("path", &c.display().to_string()), ("e", d)],
        ),
        ErroGravar::AlvoSumiu => tr("pixel.io.err.target_gone").to_owned(),
        ErroGravar::Preset => tr("pixel.io.err.preset").to_owned(),
        ErroGravar::GodotSemQuadros => tr("pixel.io.err.godot_frames").to_owned(),
    }
}

/// As frases de um export gravado: `(é erro, texto)`.
#[must_use]
pub fn do_gravado(r: Result<Gravado, ErroGravar>) -> Vec<(bool, String)> {
    let g = match r {
        Ok(g) => g,
        Err(e) => return vec![(true, erro_gravar(&e))],
    };
    let path = g.ficheiros.first().map(|c| c.display().to_string());
    let path = path.unwrap_or_default();
    let n = g.ficheiros.len().to_string();
    let t = if g.ficheiros.len() > 1 {
        tr_with("pixel.io.saved_many", &[("n", &n), ("path", &path)])
    } else {
        tr_with("pixel.io.saved", &[("path", &path)])
    };
    let avisos = g.avisos.into_iter().map(|a| (false, aviso_saida(a)));
    std::iter::once((false, t)).chain(avisos).collect()
}

/// As frases de um ficheiro aberto: `(é erro, texto)`.
#[must_use]
pub fn do_aberto(nome: &str, avisos: Vec<Aviso>) -> Vec<(bool, String)> {
    let t = tr_with("pixel.io.opened", &[("name", &nome)]);
    let av = avisos.into_iter().map(|a| (false, aviso_ase(a)));
    std::iter::once((false, t)).chain(av).collect()
}

/// As frases `(é erro, texto)` e o documento com o nome, se abriu.
type Abertura = (
    Vec<(bool, String)>,
    Option<(Sprite, String, ph2d_pixel_io::ase::Colliders)>,
);

/// As frases de um ficheiro aberto e o documento, se abriu.
#[must_use]
pub fn do_abrir(r: Result<Aberto, ErroAbrir>) -> Abertura {
    match r {
        Ok((s, avisos, nome, c)) => (do_aberto(&nome, avisos), Some((s, nome, c))),
        Err(e) => (vec![(true, erro_abrir(&e))], None),
    }
}

/// O nome de partida no diálogo de gravar e a pasta: os do último export deste sprite.
#[must_use]
pub fn sugestao_de_nome(s: &Sprite, ext: &str) -> (String, Option<PathBuf>) {
    let velho = s.export_preset().map(|p| PathBuf::from(&p.path));
    let raiz = velho.as_ref().and_then(|c| c.file_stem());
    let raiz = raiz.map_or(tr("pixel.io.default_name").into(), |n| n.to_string_lossy());
    let pasta = velho.as_ref().and_then(|c| c.parent()).map(PathBuf::from);
    (format!("{raiz}.{ext}"), pasta)
}

/// A extensão do export e o nome do tipo no diálogo de gravar.
#[must_use]
pub fn tipo_do_export(e: &Export) -> (&'static str, &'static str) {
    let (ext, tipo) = match e {
        Export::Ficheiro(x) => match x.formato {
            Formato::Png => ("png", "pixel.io.kind_png"),
            Formato::Gif => ("gif", "pixel.io.kind_gif"),
            Formato::Apng => ("apng", "pixel.io.kind_apng"),
            Formato::Webp => ("webp", "pixel.io.kind_webp"),
            Formato::Ase => ("ase", "pixel.io.kind_ase"),
        },
        Export::Folha(_) | Export::Tiles(_) => ("png", "pixel.io.kind_png"),
    };
    (ext, tr(tipo))
}
