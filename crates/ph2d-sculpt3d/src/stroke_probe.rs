//! **O DIAGNÓSTICO — o que uma sonda pode PERGUNTAR ao traço sem o fazer
//! depositar nada.**
//!
//! Filho do [`super`], e o corte é *o que um dab FAZ* (lá) contra *o que se
//! consegue MEDIR dele* (aqui). É a casa da próxima sonda: o pai é o caminho
//! quente e cresce por conta própria, e um diagnóstico enfiado nele custa LOC a
//! quem não o usa.

use super::{Dab, SculptStroke};
use crate::brush::Brush;
use ph2d_mesh::Mesh;

impl SculptStroke {
    /// **DIAGNÓSTICO: que plano este dab ajustaria?** Devolve `(ponto, normal)`.
    ///
    /// ⚠️ **Ele existe para uma sonda não escrever a SEGUNDA resposta.** O
    /// atlas de divergência precisa comparar o nosso plano com o
    /// `area_center`/`area_normal` da referência, e a alternativa — re-derivar
    /// o ajuste do lado da sonda — é exatamente a forma que este módulo
    /// documenta como a que apodrece: duas leituras da mesma pergunta, e a
    /// medição passa a falar sobre a cópia. Aqui a rota é a MESMA
    /// (`verts_in_sphere` → `capture` → `fit_plane`), então o que a sonda mede
    /// é o que o dab usaria.
    ///
    /// ⚠️ **Ele CAPTURA** (é `&mut self`): sem a captura o `fit_plane` leria
    /// `base_pos` de vértices que este traço nunca viu.
    pub fn probe_plane(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        dab: &Dab,
    ) -> ([f32; 3], [f32; 3]) {
        mesh.verts_in_sphere(dab.center, dab.radius, &mut self.query, &mut self.footprint);
        for i in 0..self.footprint.len() {
            let v = self.footprint[i];
            self.capture(mesh, v);
        }
        let fit = self.fit_plane(mesh, brush, dab);
        (fit.point, fit.normal)
    }
}
