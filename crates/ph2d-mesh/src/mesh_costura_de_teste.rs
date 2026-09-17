//! ⭐ **A COSTURA DE TESTE da malha** — e ela tem UM item, de propósito.
//!
//! Filho (`#[path]`) do [`super`] pelo mesmo motivo do [`super::planes`]: ela
//! escreve um campo PRIVADO, e a privacidade em Rust alcança os descendentes do
//! módulo que o declara.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, e é o que impede esta porta de crescer:**
//! o pai diz *o que uma malha é*; aqui fica o que só existe para uma BANCADA
//! poder reproduzir um oráculo. Um ficheiro próprio torna visível, num `ls`, tudo
//! o que o produto NÃO faz.

use super::Mesh;

impl Mesh {
    /// **PREGA as normais de vértice** — escreve-as, e mais nada.
    ///
    /// ⚠️⚠️ **Ela existe porque um ORÁCULO pode ter um caminho que não as
    /// refresca, e o nosso refresca sempre.** Os corpora por script dos pincéis
    /// de clean-room dão os dabs um a um e o alvo, nesse caminho, **não**
    /// recalcula as normais entre eles — o cabeçalho de cada fixtura di-lo com
    /// todas as letras. Corrido pelo caminho do dab, o nosso
    /// [`Mesh::refresh_region`] recalcula-as, e a cadeia de oito dabs do pincel
    /// afiado desvia `4,45e-2` por uma razão **que não é a lei**.
    ///
    /// ⛔ **É `test-support` e nunca produto:** no app as normais seguem a
    /// superfície — é isso que faz um traço arrastado ser o que é —, e um
    /// caminho de produto que as pregasse entregaria um pincel que deixa de ver
    /// o barro que ele próprio moveu.
    pub fn pregar_normais_para_teste(&mut self, normais: &[[f32; 3]]) {
        assert_eq!(
            normais.len(),
            self.normals.len(),
            "pregar normais de outra malha"
        );
        self.normals.copy_from_slice(normais);
    }
}
