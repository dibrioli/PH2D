//! ⭐ W23 B12 — **quem está dentro de uma região**: a porta ÚNICA do rectângulo de selecção e do laço
//! (`docs/MiroClone/17_bloco6_plano.md` §2.1). A mesma política nos dois — só os vivos e à vista
//! ([`BoardDoc::live`]), nunca os travados (o Miro), e quem é de um grupo traz o grupo — e a regra de
//! quem entra muda com a região: o rectângulo leva quem tem a CAIXA toda dentro (o Excalidraw); o
//! laço, quem tem [`LASSO_COVER`] do contorno real dentro (o Miro).

use ph2d_board_model::{BoardDoc, ElementId};

use crate::Editor;

/// O laço leva quem tem pelo menos esta fracção dentro (a área; numa linha, o comprimento). O Miro:
/// *«an object is selected only if at least 90% of it is inside the lasso»* (help, «Working with
/// objects»). `1.0` = o «contain» do Excalidraw (o oráculo corre com ele).
pub const LASSO_COVER: f64 = 0.9;

/// Uma região de selecção, no mundo.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Region<'a> {
    /// `[x0, y0, x1, y1]`.
    Rect([f64; 4]),
    /// O laço (fecha-se sozinho) e a cobertura que ele pede.
    Lasso(&'a [[f64; 2]], f64),
}

fn meets(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] <= b[2] && b[0] <= a[2] && a[1] <= b[3] && b[1] <= a[3]
}

impl Editor {
    /// O que um laço pelo rasto `trail` apanha em `doc` (a mesma porta do gesto; as sondas e as cenas).
    pub fn lasso_pick(&mut self, doc: &BoardDoc, trail: &[[f64; 2]]) -> Vec<ElementId> {
        self.lasso_pick_covering(doc, trail, LASSO_COVER)
    }

    /// [`Self::lasso_pick`] com outra cobertura (`1.0` = o «contain» do Excalidraw: o gate do oráculo).
    pub fn lasso_pick_covering(
        &mut self,
        doc: &BoardDoc,
        trail: &[[f64; 2]],
        cover: f64,
    ) -> Vec<ElementId> {
        self.inside(doc, Region::Lasso(trail, cover))
    }

    /// ⭐ Os elementos que `region` apanha, já com os grupos deles.
    pub(crate) fn inside(&mut self, doc: &BoardDoc, region: Region<'_>) -> Vec<ElementId> {
        self.routes.sync(doc);
        let routes = &self.routes;
        let lasso = match region {
            Region::Lasso(l, _) => ph2d_board_geom::Lasso::new(l),
            Region::Rect(_) => None,
        };
        if matches!(region, Region::Lasso(..)) && lasso.is_none() {
            return Vec::new();
        }
        let hits: Vec<ElementId> = doc
            .live()
            .filter(|el| !el.locked())
            .filter(|el| {
                let route = el.connector().map(|_| routes.get(el.id));
                let b = match route {
                    Some(r) => r.map_or([f64::NAN; 4], |r| r.bbox),
                    None => el.aabb(),
                };
                match region {
                    Region::Rect(r) => b[0] >= r[0] && b[1] >= r[1] && b[2] <= r[2] && b[3] <= r[3],
                    Region::Lasso(_, cover) => {
                        let lasso = lasso.as_ref().expect("conferido acima");
                        b.iter().all(|v| v.is_finite())
                            && meets(b, lasso.bbox())
                            && lasso.cover(el, b, route.flatten().map(|r| &r.path)) >= cover
                    }
                }
            })
            .map(|el| el.id)
            .collect();
        hits.into_iter().flat_map(|id| self.pick(doc, id)).collect()
    }
}
