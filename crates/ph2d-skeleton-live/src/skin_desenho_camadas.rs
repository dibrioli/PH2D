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

/// ⭐⭐⭐ **A camada do TRAÇO cortada do PRÓPRIO assado** — o traço É a borda do preenchimento (F55,
/// report do dono de 2026-10-04: assados à parte, contorno inteiro e trechos descolavam até `0,28`
/// numa dobra de `170°`). Cada ponta de um trecho ([`super::frente::cortes_dos_fechados`]) projecta-se
/// no pedaço do assado que vem do MESMO segmento da fonte (o nó `k` da fonte é, ao bit, um nó do
/// assado — `assa_a_pele_com_nos`), e o assado parte-se ali (de Casteljau). Os fechados inteiros à
/// vista entram inteiros; as riscas abertas, como estão. `None` quando os contornos não casam.
pub(super) fn traco_sobre_o_assado(
    d: &VecPath,
    nos: &[[f64; 2]],
    fonte: &VecPath,
    cortes: &[Option<Vec<super::frente::Trecho>>],
) -> Option<VecPath> {
    use super::frente::{avalia, cubica, recorta};
    use ph2d_vec_scene::VecVertex;
    if d.contour_count() != fonte.contour_count() || nos.len() != fonte.verts_all().count() {
        return None;
    }
    let mut pecas: Vec<ph2d_vec_scene::Contour> = Vec::new();
    let mut base = 0;
    for c in 0..fonte.contour_count() {
        let ((fv, fechado), (ov, _)) = (fonte.contour(c)?, d.contour(c)?);
        let m = fv.len();
        base += m;
        let trechos = match (fechado, cortes.get(c).and_then(Option::as_ref)) {
            (false, _) | (true, None) => {
                if ov.len() > 1 {
                    pecas.push(ph2d_vec_scene::Contour {
                        verts: ov.to_vec(),
                        closed: fechado,
                    });
                }
                continue;
            }
            (true, Some(t)) => t,
        };
        // O índice no assado do nó `k` da fonte.
        let mut idx = Vec::with_capacity(m);
        let mut de = 0;
        for k in 0..m {
            let alvo = nos[base - m + k];
            let i = (de..ov.len()).find(|&i| ov[i].anchor == alvo)?;
            idx.push(i);
            de = i;
        }
        let n = ov.len();
        let mut w = ov.to_vec();
        w.push(ov[0]);
        let projeta = |(u, q): (f64, [f64; 2])| -> f64 {
            #[expect(clippy::cast_precision_loss, reason = "contagem de nós")]
            let um = u.rem_euclid(m as f64);
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "segmento")]
            let k = (um.floor() as usize).min(m - 1);
            let fim = if k + 1 < m { idx[k + 1] } else { n };
            let dist = |j: usize, t: f64| {
                let p = avalia(&cubica(&w, j), t);
                (p[0] - q[0]).hypot(p[1] - q[1])
            };
            let (mut bj, mut bt) = (idx[k], 0.0);
            for j in idx[k]..fim.max(idx[k] + 1) {
                for i in 0..=32 {
                    let t = f64::from(i) / 32.0;
                    if dist(j, t) < dist(bj, bt) {
                        (bj, bt) = (j, t);
                    }
                }
            }
            let (mut a, mut b) = ((bt - 1.0 / 32.0).max(0.0), (bt + 1.0 / 32.0).min(1.0));
            for _ in 0..40 {
                let (x, y) = ((2.0 * a + b) / 3.0, (a + 2.0 * b) / 3.0);
                if dist(bj, x) < dist(bj, y) {
                    b = y;
                } else {
                    a = x;
                }
            }
            #[expect(clippy::cast_precision_loss, reason = "índice de segmento")]
            let u = bj as f64 + 0.5 * (a + b);
            u
        };
        #[expect(clippy::cast_precision_loss, reason = "contagem de nós")]
        let fim = n as f64;
        for &(a, b) in trechos {
            let (u0, u1) = (projeta(a), projeta(b));
            let verts: Vec<VecVertex> = if u0 < u1 {
                recorta(&w, u0, u1).into_iter().map(|(v, _)| v).collect()
            } else {
                let mut p: Vec<VecVertex> = recorta(&w, u0, fim).into_iter().map(|(v, _)| v).collect();
                let resto: Vec<VecVertex> = recorta(&w, 0.0, u1).into_iter().map(|(v, _)| v).collect();
                if let (Some(j), Some(r)) = (p.last_mut(), resto.first()) {
                    j.out_handle = r.out_handle;
                }
                p.extend(resto.into_iter().skip(1));
                p
            };
            pecas.push(ph2d_vec_scene::Contour {
                verts,
                closed: false,
            });
        }
    }
    let mut saida = d.clone();
    let mut it = pecas.into_iter();
    let primeiro = it.next()?;
    saida.verts = primeiro.verts;
    saida.closed = primeiro.closed;
    saida.subpaths = it.collect();
    Some(saida)
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
