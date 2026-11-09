//! ⭐ **A cena responde à Bancada do editor de scripts** (plano 13 Onda B, item 10; dono 09/11:
//! «espiar a cena» e «mapa na Bancada»). A aba de código ocupa o lugar do canvas, por isso pede à
//! cena por recursos ECS (`ph2d_code_bancada::scene`): o objecto a ESPIAR manda no contorno de
//! proveniência (na cor dele), e o mapa recebe o contorno de MUNDO de cada objecto que corre o
//! script — pela MESMA porta que contorna um objecto apontado ([`crate::App::resolve_hover_outline`]).

use ph2d_code_bancada::{SceneOutlines, ScenePeek};

impl crate::App {
    /// Um quadro, depois do contorno de proveniência: o espiar ganha a ele; o mapa recebe os anéis.
    pub(crate) fn serve_bancada(&mut self) {
        let Some(g) = self.gfx.as_ref() else { return };
        let w = g.sim.world();
        let peek = w.get_resource::<ScenePeek>().and_then(|p| p.0);
        let wanted = w
            .get_resource::<SceneOutlines>()
            .map(|o| o.wanted.clone())
            .unwrap_or_default();
        if let Some((bits, _)) = peek {
            self.hovered_object = Some(bits);
            self.hover_outline = self.resolve_hover_outline(bits);
        }
        let got: Vec<(u64, Vec<Vec<[f32; 2]>>)> = wanted
            .into_iter()
            .map(|bits| {
                let world = self.resolve_hover_outline(bits);
                (bits, ph2d_vec_render::outline_rings(&world))
            })
            .collect();
        if let Some(g) = self.gfx.as_mut()
            && let Some(mut o) = g.sim.world_mut().get_resource_mut::<SceneOutlines>()
            && o.got != got
        {
            o.got = got;
        }
    }
}
