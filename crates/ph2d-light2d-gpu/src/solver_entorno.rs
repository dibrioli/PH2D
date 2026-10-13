//! ⭐ **O ENTORNO na placa** (W5d) — a luz que vem de FORA da grade da vista: um 2.º solver mais grosso
//! e mais largo mascara o MIOLO (a grade da vista: a emissão de dentro dela não entra nas cascatas dele)
//! e a c0 dele soma-se à da vista na saída da holográfica. Módulo-filho do [`super`].

use super::*;

impl Solver {
    /// ⭐ W5d: a c0 do último quadro (as cascatas: rgb = a fluência na sonda, alfa = fora dos opacos),
    /// na grade das cascatas — para quem precisa de a medir.
    #[must_use]
    pub fn c0(&self) -> &wgpu::Texture {
        &self.texturas[PassList::indice_da_c0(&self.cfg)]
    }

    /// ⭐ W5d — este solver é o ENTORNO de outro (a luz que vem de FORA da grade daquele): a emissão
    /// dentro de `miolo_m` (`[x0, y0, x1, y1]`, metros — a grade da vista) não entra nas cascatas
    /// dele. Só a holográfica sabe; devolve se o aplicou.
    pub fn mascarar_miolo(&mut self, gpu: &GpuContext, miolo_m: Option<[f64; 4]>) -> bool {
        let g = self.cfg.grade;
        let Some(h) = self.hrc.as_mut() else {
            return false;
        };
        let em = |m: f64, k: usize| ((m - g.origem_m[k]) / g.texel_m) as f32;
        h.miolo(
            &gpu.queue,
            miolo_m.map(|[x0, y0, x1, y1]| [em(x0, 0), em(y0, 1), em(x1, 0), em(y1, 1)]),
        );
        true
    }

    /// ⭐ W5d — a c0 do `entorno` (o solver que mascarou o miolo desta grade) soma-se à c0 deste: a luz
    /// que vem de FORA da grade (o rebote e os corpos que brilham fora dela). Só com a holográfica
    /// nos dois; devolve se o estado pedido ficou ligado.
    pub fn somar_entorno(&mut self, gpu: &GpuContext, entorno: Option<&Solver>) -> bool {
        let g = self.cfg.grade;
        let Some(h) = self.hrc.as_mut() else {
            return false;
        };
        let placa = crate::Placa {
            device: &gpu.device,
            fabrica: &self.fabrica,
            conta: None,
        };
        match entorno.filter(|e| e.hrc.is_some()) {
            Some(e) => {
                let ge = e.cfg.grade;
                let vista = e.texturas[PassList::indice_da_c0(&e.cfg)]
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let desloc = [
                    ((g.origem_m[0] - ge.origem_m[0]) / ge.texel_m) as f32,
                    ((g.origem_m[1] - ge.origem_m[1]) / ge.texel_m) as f32,
                ];
                let escala = (g.texel_m / ge.texel_m) as f32;
                h.entorno(
                    placa,
                    &gpu.queue,
                    Some((&vista, [desloc[0], desloc[1], escala])),
                );
                true
            }
            None => {
                h.entorno(placa, &gpu.queue, None);
                entorno.is_none()
            }
        }
    }
}
