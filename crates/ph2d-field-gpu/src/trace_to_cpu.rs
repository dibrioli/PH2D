//! ⭐⭐⭐ **O G-BUFFER DO DISPOSITIVO NO VOCABULÁRIO DA CPU** — a porta que os gates de paridade
//! comparam com o traçado de CPU (o produto pinta o Matcap na placa, sem o trazer de volta).

use crate::trace::DeviceGbuffer;

impl DeviceGbuffer {
    /// ⭐⭐⭐ **O G-buffer do dispositivo no vocabulário da CPU**.
    ///
    /// ⚠️ **O `point` RECONSTRÓI-SE do `t`**, e é por isso que ele não atravessa o barramento: são
    /// mais `12 B` por pixel (`25 MB` a `1920×1080`) para uma conta que a CPU faz em microssegundos.
    /// *O que se lê de volta é o que não se pode derivar.*
    #[must_use]
    pub fn to_cpu(
        &self,
        cam: &ph2d_field_render::Orbit,
        screen: ph2d_field_render::Screen,
    ) -> ph2d_field_render::Gbuffer {
        let n = self.t.len();
        let mut hit = Vec::with_capacity(n);
        let mut point = Vec::with_capacity(n);
        let w = self.width as usize;
        // ⭐⭐⭐ **O PONTO reconstrói-se SEM normalizar a direcção** — ver
        // [`ph2d_field_render::Rays::point_at`] para a álgebra e a tabela. Medido a `1920×1080`
        // neste laço: **`37,87 → 5,75 ms`**, que era a maior fatia do quadro inteiro (`87,58`).
        let raios = cam.rays();
        for y in 0..self.height as usize {
            #[allow(clippy::cast_precision_loss)]
            let py = y as f32 + 0.5;
            for x in 0..w {
                let t = self.t[y * w + x];
                hit.push(t >= 0.0);
                #[allow(clippy::cast_precision_loss)]
                let (u, v) = screen.plane_at(x as f32 + 0.5, py);
                point.push(raios.point_at(u, v, t));
            }
        }
        let edges = self
            .edges
            .iter()
            .map(|e| ph2d_field_render::EdgePixel {
                pixel: e.pixel,
                hit: e.hit,
                normal: e.normal,
            })
            .collect();
        ph2d_field_render::Gbuffer {
            width: self.width,
            height: self.height,
            hit,
            normal: self.normal.clone(),
            point,
            edges,
        }
    }
}
