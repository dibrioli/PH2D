//! ⭐⭐⭐ **A RETÍCULA COMO GRAFO** — os pares vizinhos, cada um UMA vez.
//!
//! É isto que o `Blur` e o `Smear` leem, e é aqui que a família se paga: numa
//! textura, dois texels vizinhos **na imagem** podem estar em pontas opostas da
//! peça, e é por isso que os dois param na costura. Aqui um par é um par **na
//! superfície**, sempre — inclusive **através** de uma aresta da malha, porque
//! ali as duas faces leem a mesma célula.
//!
//! # ⛔⛔ Cada par UMA vez, e a dedução de que a enumeração é completa
//!
//! Os pares de um triângulo de lado `L` vêm em **três famílias** (`Δi/Δj`,
//! `Δj/Δk`, `Δk/Δi`) com `L(L+1)/2` cada — `3L(L+1)/2` ao todo. Os
//! sub-triângulos **direitos** são `L(L+1)/2` e cada um tem exactamente um par
//! de cada família ⇒ **enumerar os direitos dá cada par exactamente uma vez**,
//! e os invertidos não acrescentam nenhum. *A contagem é a prova, e ela está
//! no gate.*
//!
//! ⚠️ **E o par que corre AO LONGO de uma aresta da malha é visto pelas duas
//! faces.** Quem o emitir duas vezes faz uma média pesar o dobro exactamente na
//! fronteira — um fio mais escuro ao longo de metade das arestas da peça. ⇒ só
//! a face DONA (`Topologia::dona_do_lado`) o emite.

use crate::{Tinta, sitio_quad, sitio_tri};

/// ⛔⛔ **EM QUE LADOS um ponto da retícula está — e são um CONJUNTO, nunca um.**
///
/// Um CANTO está em **dois** lados ao mesmo tempo. A 1.ª redacção devolvia
/// `Option<usize>` com o primeiro que casasse, e a `lado = 1` isso fez o par
/// `b→c` ler *«lado 0 contra lado 1»* e escapar à cerca do dono — **três dos
/// seis pares de um tetraedro saíam a dobrar**, e o gate apanhou-o à primeira.
/// *Uma classificação que devolve UM elemento de um conjunto não pode ser
/// intersectada.*
fn lados_tri(ijk: (u32, u32, u32)) -> u8 {
    u8::from(ijk.2 == 0) | u8::from(ijk.0 == 0) << 1 | u8::from(ijk.1 == 0) << 2
}

/// A irmã para QUADS — `a→b` é `j = 0`, `b→c` é `i = L`, `c→d` é `j = L`,
/// `d→a` é `i = 0`.
fn lados_quad(lado: u32, ij: (u32, u32)) -> u8 {
    u8::from(ij.1 == 0)
        | u8::from(ij.0 == lado) << 1
        | u8::from(ij.1 == lado) << 2
        | u8::from(ij.0 == 0) << 3
}

/// O lado COMUM a dois pontos, se existir. Dois pontos distintos da retícula
/// partilham no máximo um lado — dois lados obrigariam-nos a ser o mesmo canto.
fn lado_comum(a: u8, b: u8) -> Option<usize> {
    let c = a & b;
    if c == 0 {
        None
    } else {
        Some(c.trailing_zeros() as usize)
    }
}

impl Tinta {
    /// ⭐⭐ **Os pares vizinhos da retícula de um TRIÂNGULO, cada um uma vez.**
    ///
    /// `f(a, b)` recebe os índices GLOBAIS das duas amostras.
    pub fn para_cada_par_tri(&self, face: usize, cantos: &[u32], mut f: impl FnMut(u32, u32)) {
        let l = self.lado_da_face(face);
        let idx = |i: u32, j: u32, k: u32| self.indice_de(face, cantos, sitio_tri(l, i, j, k));
        for i in 0..l {
            for j in 0..(l - i) {
                let k = l - 1 - i - j;
                // Os três cantos do sub-triângulo DIREITO em `(i, j, k)`.
                let a = (i + 1, j, k);
                let b = (i, j + 1, k);
                let c = (i, j, k + 1);
                for (p, q) in [(a, b), (b, c), (c, a)] {
                    // ⛔ O par que corre ao longo de um lado da face só é emitido
                    // pela face DONA — ver o cabeçalho.
                    if let Some(s) = lado_comum(lados_tri(p), lados_tri(q))
                        && !self.topologia().dona_do_lado(face, s)
                    {
                        continue;
                    }
                    f(idx(p.0, p.1, p.2), idx(q.0, q.1, q.2));
                }
            }
        }
    }

    /// A irmã para QUADS — os pares horizontais e verticais da retícula.
    pub fn para_cada_par_quad(&self, face: usize, cantos: &[u32], mut f: impl FnMut(u32, u32)) {
        let l = self.lado_da_face(face);
        let idx = |i: u32, j: u32| self.indice_de(face, cantos, sitio_quad(l, i, j));
        let mut emite = |p: (u32, u32), q: (u32, u32), me: &Self| {
            if let Some(s) = lado_comum(lados_quad(l, p), lados_quad(l, q))
                && !me.topologia().dona_do_lado(face, s)
            {
                return;
            }
            f(idx(p.0, p.1), idx(q.0, q.1));
        };
        for j in 0..=l {
            for i in 0..=l {
                if i < l {
                    emite((i, j), (i + 1, j), self);
                }
                if j < l {
                    emite((i, j), (i, j + 1), self);
                }
            }
        }
    }
}
