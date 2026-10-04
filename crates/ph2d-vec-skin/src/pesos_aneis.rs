//! ⭐⭐ **O ÍNDICE DOS ANÉIS do domínio** (A3, 2026-10-04) — as duas perguntas da cerca de cobertura
//! da grelha (o ponto está DENTRO? a fronteira TOCA a célula?) varriam TODOS os anéis por pergunta:
//! num *Repeater* `39 × 39` (`1 024` anéis, `131 072` pontos) a malha custava `743 ms` em release
//! (`pesos_preco_tests`). O índice só muda QUAIS arestas se perguntam — a conta de cada uma é a
//! mesma ⇒ a resposta é a mesma AO BIT (gate `o_indice_dos_aneis_responde_como_a_varredura`).

/// Arestas por faixa horizontal (o raio do par-ímpar) e por célula (a caixa da aresta).
pub(super) struct IndiceDosAneis {
    origem: [f64; 2],
    lado: [f64; 2],
    faixas: Vec<Vec<(u32, u32)>>,
    celulas: Vec<Vec<(u32, u32)>>,
}

/// Faixas e colunas do índice.
const DIM: usize = 64;

/// A aresta `a→b` corta o raio horizontal de `p` para `+x`? — a conta do par-ímpar.
pub(super) fn cruza_o_raio(a: [f64; 2], b: [f64; 2], p: [f64; 2]) -> bool {
    (a[1] > p[1]) != (b[1] > p[1]) && (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] > p[0]
}

impl IndiceDosAneis {
    pub(super) fn novo(aneis: &[Vec<[f64; 2]>]) -> Self {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for p in aneis.iter().flatten() {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "dimensão pequena")]
        let lado = [0, 1].map(|k| ((hi[k] - lo[k]) / DIM as f64).max(1e-12));
        let mut ix = Self {
            origem: lo,
            lado,
            faixas: vec![Vec::new(); DIM],
            celulas: vec![Vec::new(); DIM * DIM],
        };
        for (r, anel) in aneis.iter().enumerate() {
            let n = anel.len();
            for i in 0..n {
                let (a, b) = (anel[i], anel[(i + 1) % n]);
                let (c0, c1) = (
                    ix.celula([a[0].min(b[0]), a[1].min(b[1])]),
                    ix.celula([a[0].max(b[0]), a[1].max(b[1])]),
                );
                #[expect(clippy::cast_possible_truncation, reason = "índices de anel e de ponto")]
                let id = (r as u32, i as u32);
                for y in c0[1]..=c1[1] {
                    ix.faixas[y].push(id);
                    for x in c0[0]..=c1[0] {
                        ix.celulas[y * DIM + x].push(id);
                    }
                }
            }
        }
        ix
    }

    fn celula(&self, p: [f64; 2]) -> [usize; 2] {
        [0, 1].map(|k| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "presa ao intervalo da grelha"
            )]
            let c = ((p[k] - self.origem[k]) / self.lado[k]).floor().max(0.0) as usize;
            c.min(DIM - 1)
        })
    }

    fn aresta(aneis: &[Vec<[f64; 2]>], (r, i): (u32, u32)) -> ([f64; 2], [f64; 2]) {
        let anel = &aneis[r as usize];
        (anel[i as usize], anel[(i as usize + 1) % anel.len()])
    }

    /// O [`super::dentro`] pelas arestas da faixa de `p`.
    pub(super) fn dentro(&self, aneis: &[Vec<[f64; 2]>], p: [f64; 2]) -> bool {
        let cruz = self.faixas[self.celula(p)[1]]
            .iter()
            .filter(|&&e| {
                let (a, b) = Self::aresta(aneis, e);
                cruza_o_raio(a, b, p)
            })
            .count();
        cruz % 2 == 1
    }

    /// Alguma aresta toca o rectângulo `[x0,y0]..[x1,y1]`? — pelas células que ele cobre.
    pub(super) fn toca(&self, aneis: &[Vec<[f64; 2]>], x0: f64, y0: f64, x1: f64, y1: f64) -> bool {
        let (c0, c1) = (self.celula([x0, y0]), self.celula([x1, y1]));
        (c0[1]..=c1[1]).any(|y| {
            (c0[0]..=c1[0]).any(|x| {
                self.celulas[y * DIM + x].iter().any(|&e| {
                    let (a, b) = Self::aresta(aneis, e);
                    super::cruza_a_celula(a, b, x0, y0, x1, y1)
                })
            })
        })
    }
}
