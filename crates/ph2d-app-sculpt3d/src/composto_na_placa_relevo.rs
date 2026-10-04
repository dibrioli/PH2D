//! ⭐⭐⭐ **O RELEVO BORRADO NA PLACA** (`docs/3D/30` §20) — filho (`#[path]`) de [`super`]: a dobra
//! do relevo através dos ajustes corre na CPU (é barata), e o passa-baixo dela — o calor da retícula,
//! que a CPU não paga a `32x` — corre no compositor da peça, o MESMO polinómio e o mesmo passo da cor.

use super::*;
use ph2d_mesh_colors::Tinta;
use ph2d_tool_painter::{AdjustWindow, Neighbourhood, gaussian_sigma};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// A retícula da peça na placa como vizinhança da dobra.
struct VizinhancaNaPlaca<'g> {
    gpu: &'g GpuContext,
    compositor: Mutex<&'g mut LayerCompositor>,
    janela: AdjustWindow,
    falhou: AtomicBool,
}

impl VizinhancaNaPlaca<'_> {
    fn calor(&self, radius: f32, campo: &mut [[f32; 4]]) {
        let feito = self
            .compositor
            .lock()
            .is_ok_and(|mut c| c.surface_heat_field(self.gpu, campo, gaussian_sigma(radius)));
        if !feito {
            self.falhou.store(true, Ordering::Relaxed);
        }
    }
}

impl Neighbourhood for VizinhancaNaPlaca<'_> {
    fn window(&self) -> AdjustWindow {
        self.janela
    }
    fn blur4(&self, radius: f32, buf: &mut [[f32; 4]]) {
        self.calor(radius, buf);
    }
    fn blur1(&self, radius: f32, buf: &mut [f32]) {
        let mut c: Vec<[f32; 4]> = buf.iter().map(|&x| [x, 0.0, 0.0, 0.0]).collect();
        self.calor(radius, &mut c);
        for (b, v) in buf.iter_mut().zip(&c) {
            *b = v[0];
        }
    }
    fn blur2(&self, radius: f32, a: &mut [f32], b: &mut [f32]) {
        let mut c: Vec<[f32; 4]> = a
            .iter()
            .zip(b.iter())
            .map(|(&x, &y)| [x, y, 0.0, 0.0])
            .collect();
        self.calor(radius, &mut c);
        for ((x, y), v) in a.iter_mut().zip(b.iter_mut()).zip(&c) {
            *x = v[0];
            *y = v[1];
        }
    }
    fn image_plane(&self) -> bool {
        false
    }
}

impl CompostosDaCena {
    /// ⭐⭐ **O relevo da peça dobrado através dos ajustes, com o calor na placa** — `false` se a
    /// placa não o fez (sem vizinhança, a superfície recusada): quem chama dobra na CPU.
    pub(crate) fn dobra_o_relevo(
        &mut self,
        gpu: &GpuContext,
        peca: ObjectId,
        pilha: &PilhaDaPeca,
        tinta: &mut Tinta,
    ) -> bool {
        let c = self
            .por_peca
            .entry(peca)
            .or_insert_with(|| CompostoNaPlaca::novo(gpu));
        if c.garante_superficie(gpu, pilha).is_err() {
            return false;
        }
        let (l, h) = dobra(pilha.amostras());
        let nb = VizinhancaNaPlaca {
            gpu,
            compositor: Mutex::new(&mut c.compositor),
            janela: AdjustWindow::full(l, h),
            falhou: AtomicBool::new(false),
        };
        let r = pilha.dobra_o_relevo_com(&nb);
        if nb.falhou.load(Ordering::Relaxed) {
            return false;
        }
        if r.is_some() || tinta.tem_relevo() {
            tinta.com_relevo(r);
        }
        pilha.relevo_dobrado();
        true
    }
}
