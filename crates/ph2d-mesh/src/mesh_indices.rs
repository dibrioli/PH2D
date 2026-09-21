//! **A LISTA DE ÍNDICES QUE A PLACA CONSOME** — e, por triângulo, de onde ele
//! veio.
//!
//! ⚠️ Irmão (`#[path]`) do [`super`], cortado pelo tecto de LOC (`723` contra
//! `700`) e pelo ASSUNTO: o `mesh.rs` é *o que uma malha É* e cresce quando ela
//! ganha um canal ou uma lei de vizinhança; **esta** travessia é *como ela se
//! entrega a um desenhador*, e cresceu quando a tinta fina precisou de saber em
//! que FACE cada triângulo mora.
//!
//! ⚠️ **Ela vê os campos privados da [`Mesh`] por ser um módulo FILHO** — e é
//! por isso que o corte não abriu uma porta nova na superfície da crate.

use super::Mesh;

impl Mesh {
    /// O buffer de índices que a GPU consome (quads triangulados).
    pub fn triangle_indices(&self, out: &mut Vec<[u32; 3]>) {
        self.triangle_indices_com_origem(out, None);
    }

    /// ⭐⭐⭐ **Os índices da GPU e, por triângulo, DE ONDE ELE VEIO** — a face
    /// e qual das metades de um quad, empacotados em `face << 1 | sub`.
    ///
    /// ⛔⛔ **Ela existe porque o `@builtin(primitive_index)` de um shader
    /// numera TRIÂNGULOS e a tinta mora nas FACES.** Um fragmento que queira
    /// ler a retícula precisa de saber em que face está e, num quad, em qual
    /// das duas metades da diagonal `a–c` — senão as baricêntricas que ele tem
    /// não se convertem no `(u, v)` da face.
    ///
    /// ⚠️ **Ela é a MESMA varredura da [`Self::triangle_indices`], que delega
    /// aqui** — e não uma segunda que por acaso concorda. *Duas travessias da
    /// mesma lista divergem no dia em que alguém mudar a diagonal por área ou
    /// por planaridade*, e a escolha da diagonal já tem dono único
    /// ([`crate::Face::tri_at`]).
    pub fn triangle_indices_com_origem(
        &self,
        out: &mut Vec<[u32; 3]>,
        mut origem: Option<&mut Vec<u32>>,
    ) {
        out.clear();
        out.reserve(self.triangle_count());
        if let Some(o) = origem.as_deref_mut() {
            o.clear();
            o.reserve(self.triangle_count());
        }
        for (fi, f) in self.faces.iter().enumerate() {
            for sub in 0..f.tri_count() {
                out.push(f.tri_at(sub));
                if let Some(o) = origem.as_deref_mut() {
                    o.push(((fi as u32) << 1) | sub as u32);
                }
            }
        }
    }
}
