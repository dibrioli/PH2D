//! ⭐⭐⭐ **A CONVERSÃO entre o enquadramento que o RENDERIZADOR pede e o que o DOCUMENTO
//! lembra** — uma porta, nos dois sentidos.
//!
//! # ⛔⛔ Por que são dois tipos e não um
//!
//! O [`ph2d_mesh_render::Framing`] vive na crate que desenha malhas (`wgpu`, matcaps, `imageio`) e
//! o [`ph2d_form_donation::baked_form::Recorte`] vive na folha que o **runtime** lê *«sem o módulo
//! 3D no build»* — a promessa que o cabeçalho daquela crate faz desde que existe. Importar um no
//! outro arrastaria o renderizador inteiro para dentro de quem só precisa de multiplicar
//! `base × luz`, e um objecto reaberto num binário sem escultura deixaria de acender.
//!
//! ⚠️ **A duplicação é deliberada, e é o precedente do [`ph2d_pose::pesos`] desta casa** (uma lei
//! escrita duas vezes porque a crate de baixo declara zero dependências). ⭐ O que a torna honesta
//! é **esta porta ser a única travessia** e a prova de ida-e-volta ao lado dela.

use ph2d_form_donation::baked_form::Recorte;
use ph2d_mesh_render::{Framing, ViewRegion};

/// O que o documento lembra, a partir do que o renderizador usou.
#[must_use]
pub(crate) fn a_guardar(f: Framing) -> Recorte {
    Recorte {
        aspect: f.aspect,
        origin: f.region.origin,
        size: f.region.size,
    }
}

/// ⭐⭐ **O que o renderizador pede, a partir do que o documento lembra** — e `None` quer dizer
/// *«a vista inteira»*, que é o que todo objecto assado antes de 2026-09-21 quer dizer.
///
/// ⚠️ **O `size` só é lido no braço do `None`**, e é isso que o torna honesto: ali o alvo É a
/// vista, logo o aspecto dele é o da vista. Com um recorte guardado o aspecto é o da vista que
/// existia no gesto, e o tamanho do alvo não tem opinião nenhuma sobre ele.
#[must_use]
pub(crate) fn a_usar(r: Option<Recorte>, size: (u32, u32)) -> Framing {
    r.map_or_else(
        || Framing::whole(size),
        |r| Framing {
            aspect: r.aspect,
            region: ViewRegion {
                origin: r.origin,
                size: r.size,
            },
        },
    )
}

#[cfg(test)]
#[path = "recorte_tests.rs"]
mod tests;
