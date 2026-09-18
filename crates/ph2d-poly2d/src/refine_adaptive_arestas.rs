//! ⭐⭐⭐ **O LIVRO DAS ARESTAS** — irmão do [`super`] pelo tecto de LOC, com o corte por
//! RESPONSABILIDADE: ali mora a OBRA (a bissecção, a cadeia, a fila), aqui **o que se sabe de uma
//! aresta e como se procura por ela**.
//!
//! ⚠️ **Ele é o sítio da cura de custo de 2026-09-16 (F6-t)**, e o porquê de cada escolha está no
//! doc de cada item: o índice pela ponta menor no lugar da árvore, os dois donos num par FIXO no
//! lugar de um `Vec`, e a ordem TOTAL que impede a cadeia de girar numa grelha.

/// A chave canónica de uma aresta — sempre `(menor, maior)`, para os dois donos concordarem.
pub(super) fn chave(u: u32, v: u32) -> (u32, u32) {
    if u < v { (u, v) } else { (v, u) }
}

/// O que se sabe do meio de uma aresta: onde ele fica, para onde o campo o leva, e quanto isso
/// difere da corda que o desenho pinta.
///
/// ⛔ **Os atributos NÃO vivem aqui, de propósito.** Guardá-los custaria um `Vec` por aresta
/// (~`1,5 ×` peças, por quadro, por imagem) para um valor que se recalcula por **aritmética pura**
/// no momento do corte — e a recomputação é bit-a-bit a mesma, porque é a mesma expressão sobre as
/// mesmas pontas.
#[derive(Clone, Copy)]
pub(super) struct Meio {
    pub(super) rest: [f64; 2],
    pub(super) posed: [f64; 2],
    pub(super) desvio: f64,
}

/// ⭐⭐ **TUDO O QUE SE SABE DE UMA ARESTA, NUMA ENTRADA SÓ** — quem a possui e onde é o meio dela.
///
/// ⛔⛔ **As duas metades viviam em DOIS mapas, e a sonda de custo disse porque não podem:** a
/// decisão de *não* refinar custava `0,52 µs` por peça contra `0,096` da lei uniforme (`5,4×`),
/// e o preço era o livro de contas — dois `BTreeMap` (⇒ o dobro dos percursos) e **um `Vec` por
/// aresta** para guardar até dois donos, que é `~1,5 ×` peças alocações por quadro por imagem.
///
/// ⇒ um mapa só, e os donos num par FIXO: *uma aresta de uma malha sã tem no máximo dois donos, e
/// isso não é um palpite — é o que o [`super::refine_tests`] afirma sobre a saída*.
#[derive(Clone, Copy)]
pub(super) struct Aresta {
    pub(super) donos: [u32; 2],
    pub(super) n: u8,
    pub(super) meio: Option<Meio>,
}

impl Aresta {
    pub(super) const VAZIA: Self = Self {
        donos: [0; 2],
        n: 0,
        meio: None,
    };

    pub(super) fn junta(&mut self, t: u32) {
        if let Some(slot) = self.donos.get_mut(self.n as usize) {
            *slot = t;
            self.n += 1;
        }
    }

    /// Tira `t` da lista; devolve `true` se a aresta ficou sem donos.
    pub(super) fn larga(&mut self, t: u32) -> bool {
        let mut fora = [0u32; 2];
        let mut n = 0u8;
        for i in 0..self.n as usize {
            if self.donos[i] != t
                && let Some(slot) = fora.get_mut(n as usize)
            {
                *slot = self.donos[i];
                n += 1;
            }
        }
        self.donos = fora;
        self.n = n;
        n == 0
    }

    pub(super) fn outro(&self, t: u32) -> Option<u32> {
        self.donos[..self.n as usize]
            .iter()
            .copied()
            .find(|&o| o != t)
    }
}

/// ⭐⭐⭐ **O LIVRO DAS ARESTAS, indexado pela PONTA MENOR** — sem hash e sem árvore.
///
/// ⛔⛔ **Era um `BTreeMap<(u32, u32), Aresta>`, e a sonda de cena cheia disse que ele ERA o custo
/// da avaliação** (2026-09-16, F6-t): montar a obra de uma malha de `2 430` peças faz `~9` percursos
/// da árvore por triângulo, e a avaliação sem partir nada custava `0,88 ms` contra `0,06` do `Fast`.
/// ⭐ O mapa só é PROCURADO (nunca percorrido), logo a ordem dele não decide nada — um índice pela
/// ponta menor dá as mesmas respostas: a lista de cada vértice tem as poucas arestas que saem dele
/// para vértices maiores, e procurar nela é uma varredura de `~3` entradas contíguas.
///
/// ⚠️ A saída é **a mesma ao bit** (impressão digital de `48` casos do produto, antes e depois), e
/// ⛔ um `HashMap` não era alternativa: está proibido na workspace (`clippy.toml`, HR-5).
#[derive(Default)]
pub(super) struct Arestas {
    por_ponta: Vec<Vec<(u32, Aresta)>>,
}

impl Arestas {
    pub(super) fn com_vertices(n: usize) -> Self {
        Self {
            por_ponta: Vec::with_capacity(n),
        }
    }

    pub(super) fn get(&self, e: &(u32, u32)) -> Option<&Aresta> {
        self.por_ponta
            .get(e.0 as usize)?
            .iter()
            .find(|(o, _)| *o == e.1)
            .map(|(_, a)| a)
    }

    pub(super) fn get_mut(&mut self, e: &(u32, u32)) -> Option<&mut Aresta> {
        self.por_ponta
            .get_mut(e.0 as usize)?
            .iter_mut()
            .find(|(o, _)| *o == e.1)
            .map(|(_, a)| a)
    }

    /// A entrada de `e`, criada VAZIA se não existir.
    pub(super) fn entrada(&mut self, e: (u32, u32)) -> &mut Aresta {
        let i = e.0 as usize;
        if self.por_ponta.len() <= i {
            self.por_ponta.resize_with(i + 1, Vec::new);
        }
        let lista = &mut self.por_ponta[i];
        let pos = match lista.iter().position(|(o, _)| *o == e.1) {
            Some(pos) => pos,
            None => {
                lista.push((e.1, Aresta::VAZIA));
                lista.len() - 1
            }
        };
        &mut lista[pos].1
    }

    pub(super) fn remove(&mut self, e: &(u32, u32)) -> Option<Aresta> {
        let lista = self.por_ponta.get_mut(e.0 as usize)?;
        let pos = lista.iter().position(|(o, _)| *o == e.1)?;
        Some(lista.swap_remove(pos).1)
    }
}

/// Um `f64` ordenável — a fila precisa de `Ord` e o desvio é sempre finito e não-negativo.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Peso(pub(super) f64);
impl Eq for Peso {}
impl Ord for Peso {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&o.0)
    }
}
impl PartialOrd for Peso {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}

/// `a > b` na ordem TOTAL das arestas — comprimento primeiro, e a chave canónica a desempatar.
///
/// ⚠️⚠️ **O desempate é load-bearing e não é estética:** sem ele, numa grelha (onde toda diagonal
/// mede o mesmo) a cadeia de vizinhos pode fechar um ciclo e o laço não termina.
pub(super) fn maior(a: (f64, u32, u32), b: (f64, u32, u32)) -> bool {
    a.0 > b.0 || (a.0 == b.0 && (a.1, a.2) > (b.1, b.2))
}
