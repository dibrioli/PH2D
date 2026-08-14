//! **O `pre` — o que um traço CONGELA no pen-down, e como ele o lê.**
//!
//! Filho do [`super`] para alcançar os planos congelados, e o corte é o assunto
//! que os outros três irmãos já pressupunham sem ter casa: o `growth` fala de
//! *quando a malha muda debaixo do gesto*, o `target` de *para onde cada verbo
//! aponta*, o `apply` de *o que se escreve* — e os três abrem o doc dizendo
//! «filho para alcançar o `pre` congelado». Aqui mora o `pre` em si.
//!
//! ⚠️ **A lei que estas trinta linhas guardam vale mais que elas:** o peso de um
//! dab é um **fato sobre a superfície CONGELADA**, nunca sobre a viva. Todo
//! ingrediente do `w` sai daqui — a máscara (`base_mask`), a posição que a
//! distância mede, a normal que o [`crate::FrontFace::Continuous`] pesa — e a
//! razão é sempre a mesma: o [`crate::Grip::Hold`] recomputa `accum = w` do zero
//! a cada dab, então qualquer termo que leia estado VIVO realimenta com a
//! deformação que ele próprio produziu. Foi o defeito do `b-mode` reportado em
//! 2026-08-13, e o gate que o pina é o
//! `the_weight_is_a_fact_about_the_frozen_surface`.

use super::SculptStroke;
use ph2d_mesh::{DEFAULT_MASK, Mesh};

impl SculptStroke {
    /// Guarda o `pre` de um vértice, se ainda não guardou. Idempotente.
    pub(super) fn capture(&mut self, mesh: &Mesh, v: u32) {
        let vi = v as usize;
        if self.stamp[vi] == self.epoch {
            return;
        }
        self.stamp[vi] = self.epoch;
        self.slot[vi] = self.touched.len() as u32;
        self.touched.push(v);
        self.base_pos.push(mesh.positions()[vi]);
        self.base_nrm.push(mesh.normals()[vi]);
        self.base_mask
            .push(mesh.masks().map_or(DEFAULT_MASK, |m| m[vi]));
        self.accum.push(0.0);
        // Alvo neutro: sem dab que vença, `lerp(base, base, 0)` não move nada.
        self.target.push(mesh.positions()[vi]);
    }

    /// A posição de `v` ANTES do traço.
    ///
    /// Um vértice não capturado nunca foi escrito por este traço, logo a posição
    /// viva dele **é** o `pre`. É isso que torna o Smooth barato: ele lê o anel
    /// inteiro sem obrigar a captura de vizinhos que ninguém vai mover.
    pub(super) fn base_pos_of(&self, mesh: &Mesh, v: u32) -> [f32; 3] {
        let vi = v as usize;
        if self.stamp[vi] == self.epoch {
            self.base_pos[self.slot[vi] as usize]
        } else {
            mesh.positions()[vi]
        }
    }
}
