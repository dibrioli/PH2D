//! B12.3: um campo preguiçoso da grade por PEDAÇOS esparsos. O pedaço que ninguém escreveu não existe e lê o CHÃO
//! por valor ([`Index`] devolve `&chao`); a 1.ª escrita ([`IndexMut`]) aloca só o pedaço dela. Era um `Vec<T>` cheio
//! do neutro no 1.º toque (`plano_cheio`): material 52 + óleo 28 + aparência 28 B × 8,3 M células a 4K = 897 MB.
//! Pedaço = [`PEDACO`] células LINEARES (o índice da grade, passo `s`): `i >> LOG` e `i & MASCARA`, sem divisão — o
//! leitor por célula do ecrã paga um ramo. Um dab toca as linhas dele: o custo é ∝ linhas × largura, não à tela.
//! Doc: `docs/Painter/missao_4k/BACKLOG.md` §B12.3.

use std::ops::{Index, IndexMut};

/// `log2` das células de um pedaço (4096 = pouco mais de uma linha a 4K).
pub const LOG_PEDACO: u32 = 12;
/// Células por pedaço.
pub const PEDACO: usize = 1 << LOG_PEDACO;
const MASCARA: usize = PEDACO - 1;

/// Um campo de `n` células `T` que lê `chao` onde nada foi escrito. `n == 0` = o campo não existe (o `Vec` vazio de
/// antes: «nenhum pincel fora da fábrica tocou a grade»).
#[derive(Clone, Debug)]
pub struct CampoEsparso<T> {
    chao: T,
    n: usize,
    pedacos: Vec<Option<Box<[T]>>>,
}

impl<T: Copy> CampoEsparso<T> {
    /// O campo que não existe (`is_empty`).
    #[must_use]
    pub fn vazio(chao: T) -> Self {
        Self {
            chao,
            n: 0,
            pedacos: Vec::new(),
        }
    }

    /// `n` células a `chao`, sem escrever nenhuma (zero pedaços vivos).
    #[must_use]
    pub fn novo(chao: T, n: usize) -> Self {
        Self {
            chao,
            n,
            pedacos: (0..n.div_ceil(PEDACO)).map(|_| None).collect(),
        }
    }

    /// O denso `v` em pedaços; o pedaço em que `mesmo_bit(célula, chão)` vale para todas não se aloca. `mesmo_bit` é
    /// do chamador porque `==` de `f32` confunde `-0.0` com `0.0` e a paridade é ao bit.
    #[must_use]
    pub fn de_vec(chao: T, v: &[T], mesmo_bit: impl Fn(&T, &T) -> bool) -> Self {
        let pedacos = v
            .chunks(PEDACO)
            .map(|p| {
                if p.iter().all(|c| mesmo_bit(c, &chao)) {
                    None
                } else {
                    let mut b = vec![chao; PEDACO];
                    b[..p.len()].copy_from_slice(p);
                    Some(b.into_boxed_slice())
                }
            })
            .collect();
        Self {
            chao,
            n: v.len(),
            pedacos,
        }
    }

    /// O campo denso (o upload da placa, o leitor que ainda quer `&[T]`).
    #[must_use]
    pub fn para_vec(&self) -> Vec<T> {
        let mut out = Vec::with_capacity(self.n);
        if self.n > 0 {
            self.copia_intervalo(0, self.n - 1, &mut out);
        }
        out
    }

    /// Células (o `len` do `Vec` de antes).
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.n
    }

    /// O campo não existe.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// O valor de quem nunca foi escrito.
    #[inline]
    #[must_use]
    pub fn chao(&self) -> T {
        self.chao
    }

    /// A célula `i`, `None` fora do campo (o `.get(i).copied()` do `Vec`).
    #[inline]
    #[must_use]
    pub fn get(&self, i: usize) -> Option<T> {
        (i < self.n).then(|| self[i])
    }

    /// Pedaços alocados.
    #[must_use]
    pub fn pedacos_vivos(&self) -> usize {
        self.pedacos.iter().filter(|p| p.is_some()).count()
    }

    /// Os bytes que os pedaços vivos ocupam (e que a 1.ª escrita de cada um escreveu).
    #[must_use]
    pub fn bytes_vivos(&self) -> usize {
        self.pedacos_vivos() * PEDACO * std::mem::size_of::<T>()
    }

    /// As células por valor, por ordem.
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        (0..self.n).map(move |i| self[i])
    }

    /// Os pedaços por ordem: `Some(células)` (o último cortado em `n`) ou `None` (todo o chão). O leitor da tela inteira
    /// sem o ramo por célula do [`Index`] (o `iter` a 4K: 8,3 M ramos).
    pub fn pedacos(&self) -> impl Iterator<Item = Option<&[T]>> + '_ {
        self.pedacos.iter().enumerate().map(move |(k, p)| {
            p.as_deref()
                .map(|p| &p[..(self.n - k * PEDACO).min(PEDACO)])
        })
    }

    /// Acrescenta a `out` as células `a..=b` (o `extend_from_slice(&v[a..=b])` do `Vec`); o pedaço ausente dá o chão.
    pub fn copia_intervalo(&self, a: usize, b: usize, out: &mut Vec<T>) {
        assert!(
            a <= b && b < self.n,
            "intervalo {a}..={b} fora de {}",
            self.n
        );
        let mut i = a;
        while i <= b {
            let (k, o) = (i >> LOG_PEDACO, i & MASCARA);
            let fim = (b - i).min(PEDACO - 1 - o);
            match &self.pedacos[k] {
                Some(p) => out.extend_from_slice(&p[o..=o + fim]),
                None => out.extend(std::iter::repeat_n(self.chao, fim + 1)),
            }
            i += fim + 1;
        }
    }

    /// Escreve `v` a partir da célula `a` (o `copy_from_slice` do `Vec`); aloca os pedaços tocados.
    pub fn escreve_intervalo(&mut self, a: usize, v: &[T]) {
        assert!(
            a + v.len() <= self.n,
            "escrita {a}+{} fora de {}",
            v.len(),
            self.n
        );
        let mut feito = 0;
        while feito < v.len() {
            let i = a + feito;
            let (k, o) = (i >> LOG_PEDACO, i & MASCARA);
            let m = (v.len() - feito).min(PEDACO - o);
            self.pedaco_mut(k)[o..o + m].copy_from_slice(&v[feito..feito + m]);
            feito += m;
        }
    }

    /// O [`Self::escreve_intervalo`] que não aloca o pedaço ausente quando o trecho dele é todo o chão
    /// (`mesmo_bit`): o restauro de uma janela neutra não acorda a tela.
    pub fn escreve_intervalo_poupando(
        &mut self,
        a: usize,
        v: &[T],
        mesmo_bit: impl Fn(&T, &T) -> bool,
    ) {
        assert!(
            a + v.len() <= self.n,
            "escrita {a}+{} fora de {}",
            v.len(),
            self.n
        );
        let mut feito = 0;
        while feito < v.len() {
            let i = a + feito;
            let (k, o) = (i >> LOG_PEDACO, i & MASCARA);
            let m = (v.len() - feito).min(PEDACO - o);
            let trecho = &v[feito..feito + m];
            if self.pedacos[k].is_some() || !trecho.iter().all(|c| mesmo_bit(c, &self.chao)) {
                self.pedaco_mut(k)[o..o + m].copy_from_slice(trecho);
            }
            feito += m;
        }
    }

    /// Copia as células `faixa` de `o` (o mesmo `len` e o mesmo chão): o pedaço ausente nos dois não se aloca, e o
    /// ausente só na origem volta ao chão no destino.
    pub fn copia_faixa_de(&mut self, o: &Self, faixa: std::ops::Range<usize>) {
        debug_assert_eq!(self.n, o.n);
        let (mut i, fim) = (faixa.start.min(self.n), faixa.end.min(self.n));
        while i < fim {
            let (k, d) = (i >> LOG_PEDACO, i & MASCARA);
            let m = (fim - i).min(PEDACO - d);
            match (&o.pedacos[k], self.pedacos[k].is_some()) {
                (Some(p), _) => self.pedaco_mut(k)[d..d + m].copy_from_slice(&p[d..d + m]),
                (None, true) => self.pedaco_mut(k)[d..d + m].fill(o.chao),
                (None, false) => {}
            }
            i += m;
        }
    }

    /// Volta a não existir (o `clear` do `Vec`).
    pub fn clear(&mut self) {
        self.n = 0;
        self.pedacos.clear();
    }

    fn pedaco_mut(&mut self, k: usize) -> &mut [T] {
        let chao = self.chao;
        self.pedacos[k].get_or_insert_with(|| vec![chao; PEDACO].into_boxed_slice())
    }
}

impl<T: Copy> Index<usize> for CampoEsparso<T> {
    type Output = T;

    #[inline]
    fn index(&self, i: usize) -> &T {
        assert!(i < self.n, "célula {i} fora de {}", self.n);
        match &self.pedacos[i >> LOG_PEDACO] {
            Some(p) => &p[i & MASCARA],
            None => &self.chao,
        }
    }
}

impl<T: Copy> IndexMut<usize> for CampoEsparso<T> {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut T {
        assert!(i < self.n, "célula {i} fora de {}", self.n);
        &mut self.pedaco_mut(i >> LOG_PEDACO)[i & MASCARA]
    }
}

/// Igualdade LÓGICA (célula a célula), não de alocação: o pedaço ausente iguala o vivo cheio do chão.
impl<T: Copy + PartialEq> PartialEq for CampoEsparso<T> {
    fn eq(&self, o: &Self) -> bool {
        self.n == o.n && self.iter().eq(o.iter())
    }
}

#[cfg(test)]
#[path = "campo_esparso_tests.rs"]
mod tests;
