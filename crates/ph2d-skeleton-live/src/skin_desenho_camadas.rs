//! ⭐⭐ **O DESENHO de uma forma presa, em CAMADAS** (A6, 2026-10-04) — numa dobra forte em que a
//! união do contacto NÃO corre (uma forma cujos contornos já se sobrepõem em repouso, as cópias de
//! um *Repeater*), o TRAÇO de um contorno fechado da parte de trás pintava por cima da frente. O
//! traço não se corta sem cortar o preenchimento, logo a forma sai em duas camadas: o preenchimento
//! de tudo e o traço só dos trechos à vista ([`super::frente`]).

use super::*;

/// O desenho fiel de uma forma presa, em coordenadas LOCAIS do caminho: a forma e, quando a dobra
/// tapa parte de um contorno fechado sem união, a camada do TRAÇO à vista (e a forma vai então sem
/// traço). Lê-se como a forma ([`std::ops::Deref`]).
#[derive(Clone, Debug)]
pub struct Desenhado {
    /// O caminho desenhado (preenchimento e, sem camada de traço, o traço).
    pub forma: VecPath,
    /// O traço à vista — contornos abertos, sem preenchimento.
    pub traco: Option<VecPath>,
}

impl std::ops::Deref for Desenhado {
    type Target = VecPath;
    fn deref(&self) -> &VecPath {
        &self.forma
    }
}

/// O que um quadro de pele entrega à GEOMETRIA VIVA: o desenho fiel de cada forma presa que o
/// pode ter (quem o põe no mundo é a [`funde`]).
pub type SkinDesenhado = BTreeMap<VecPathId, Desenhado>;

/// Os contornos ABERTOS de `d` (as riscas), para a camada do traço.
pub(super) fn abertos(d: &VecPath) -> impl Iterator<Item = ph2d_vec_scene::Contour> + '_ {
    (0..d.contour_count())
        .filter_map(|c| d.contour(c))
        .filter(|(v, fechado)| !fechado && v.len() > 1)
        .map(|(v, _)| ph2d_vec_scene::Contour {
            verts: v.to_vec(),
            closed: false,
        })
}

/// ⭐⭐ **O desenho fiel no MUNDO, dentro da geometria viva do quadro.**
///
/// `vivo` é o mapa que o desenho e o PICK lêem (`ph2d_vec_render::LiveGeometry`, o mesmo tipo
/// escrito por extenso para esta folha não depender da crate de desenho).
///
/// ⚠️ **Não ESCREVE por cima de outro produtor** — uma forma presa com Offset vivo, largura viva,
/// simetria, padrão ou contorno continua a mostrar o que aquele produtor cozeu dela (sobre os nós
/// do artista, como ontem). ⛔ Os dois juntos seriam uma escolha sem dono; o primeiro a escrever
/// ganha, e os outros produtores escrevem ANTES desta chamada.
pub fn funde(
    desenho: &SkinDesenhado,
    xforms: &VecXforms,
    vivo: &mut BTreeMap<VecPathId, Vec<VecPath>>,
) {
    for (id, p) in desenho {
        vivo.entry(*id).or_insert_with(|| {
            let xf = ph2d_vec_scene::xform_of(xforms, *id);
            std::iter::once(&p.forma)
                .chain(p.traco.as_ref())
                .map(|c| {
                    let mut mundo = c.clone();
                    ph2d_vec_scene::bake_xform(&mut mundo, &xf);
                    mundo
                })
                .collect()
        });
    }
}

#[cfg(test)]
#[path = "skin_desenho_camadas_tests.rs"]
mod tests;
