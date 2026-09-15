//! ⭐⭐⭐ **O G-BUFFER DO DISPOSITIVO NO VOCABULÁRIO DA CPU** — para a pintura correr onde já
//! corre, sem saber que a marcha mudou de sítio.
//!
//! ⚠️ **Este é o caminho que o [`crate::paint`] SUBSTITUI.** Ele fica por duas razões, e nenhuma é
//! inércia: é ele que o gate de paridade do G-buffer compara com o traçado de CPU, e é ele que
//! serve uma peça que o pintor do dispositivo ainda não saiba pintar.

use crate::trace::DeviceGbuffer;

impl DeviceGbuffer {
    /// ⭐⭐⭐ **O G-buffer do dispositivo no vocabulário da CPU** — para a pintura correr onde já
    /// corre, sem saber que a marcha mudou de sítio.
    ///
    /// ⚠️ **O `point` RECONSTRÓI-SE do `t`**, e é por isso que ele não atravessa o barramento: são
    /// mais `12 B` por pixel (`25 MB` a `1920×1080`) para uma conta que a CPU faz em microssegundos.
    /// *O que se lê de volta é o que não se pode derivar.*
    #[must_use]
    pub fn to_cpu(
        &self,
        cam: &ph2d_field_render::Orbit,
        screen: ph2d_field_render::Screen,
    ) -> (ph2d_field_render::Gbuffer, ph2d_field_render::Shadows) {
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
        let g = ph2d_field_render::Gbuffer {
            width: self.width,
            height: self.height,
            hit,
            normal: self.normal.clone(),
            point,
            edges,
        };
        let mut sh = ph2d_field_render::Shadows::default();
        sh.set_lamp(0, self.shadow.clone());
        // ⚠️ **A suavização é aplicada AQUI**, como o refinamento da CPU a aplica no publicar — ela
        // faz parte do que a oclusão entrega, e não do que ela calcula.
        sh.set_ambient(ph2d_field_render::blur_occlusion(&g, &self.ambient));
        (g, sh)
    }
}
