//! ⭐ **O rebote no gabarito** (W5c, a lei do plano §2.4): cada corpo OPACO que recebe luz devolve ao
//! plano, igual em toda direção, `L_r(x) = min(a, A_MAX) · luz_em(x⁺)` — o albedo vezes a luz que a
//! câmera mostra junto a `x` (o exterior mais próximo, F3).
//!
//! A fronteira de cada refletor parte-se em ELEMENTOS de comprimento `≤ h`, de radiância constante.
//! Os extremos deles são ângulos-evento ([`crate::linha`]): em cada intervalo o raio acerta UM
//! elemento, logo a linha de um ponto — a luz que não depende do rebote mais os pesos (ângulo ×
//! transmissão) de cada elemento — é EXATA para a discretização. O rebote `k` é então um produto
//! matriz × vetor sobre o `k − 1`; o `h` converge-se num gate.

use std::f64::consts::TAU;

use ph2d_light2d_ecs::CenaDeLuz;

use crate::{Linha, Objeto, Pontual, geometria, linha, montar, opaco, rgb};

/// ⭐ O teto do albedo do rebote. O recurso é a CONVERGÊNCIA: numa sala fechada o resíduo depois de
/// `n` rebotes é `≤ A_MAX^n`; com `a = 1` a série diverge (a luz cresceria quadro a quadro até estourar
/// o f16 da placa). O número é o teto físico dos materiais (neve fresca ≈ 0,8–0,9).
pub const ALBEDO_MAX: f32 = 0.9;

/// O afastamento de `x⁺` da fronteira (para FORA), em metros.
const FORA: f64 = 1e-9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpcoesDoRebote {
    /// O comprimento máximo de um elemento da fronteira, em metros.
    pub passo_m: f64,
    /// Quantos rebotes (0 = nenhum: a [`crate::fluencia`]).
    pub rebotes: u32,
}

pub(crate) struct Refletor {
    pub(crate) objeto: usize,
    pub(crate) vertices: Vec<[f64; 2]>,
    /// Por aresta: `(quantos elementos, o id do primeiro)`.
    arestas: Vec<(usize, usize)>,
}

/// Os elementos refletores de uma cena.
pub(crate) struct Elementos {
    pub(crate) refletores: Vec<Refletor>,
    /// Os extremos INTERIORES às arestas (os vértices já são eventos).
    pub(crate) pontos: Vec<[f64; 2]>,
}

impl Elementos {
    pub(crate) fn vazio() -> Self {
        Self {
            refletores: Vec::new(),
            pontos: Vec::new(),
        }
    }

    /// O id do elemento da aresta `aresta` do refletor `r` no parâmetro `u ∈ [0, 1)`.
    pub(crate) fn elemento(&self, r: usize, aresta: usize, u: f64) -> usize {
        let (n, primeiro) = self.refletores[r].arestas[aresta];
        primeiro + ((u * n as f64) as usize).min(n - 1)
    }
}

/// Um elemento: o seu albedo (já com o teto) e o ponto `x⁺` onde se mede a luz que ele devolve.
struct Elemento {
    albedo: [f64; 3],
    fora: [f64; 2],
}

fn reflete(o: &Objeto) -> bool {
    o.emissao.is_none() && opaco(o) && o.albedo.iter().any(|&a| a > 0.0)
}

fn partir(objetos: &[Objeto], passo: f64) -> (Elementos, Vec<Elemento>) {
    let teto = f64::from(ALBEDO_MAX);
    let mut els = Elementos::vazio();
    let mut todos = Vec::new();
    for (k, o) in objetos.iter().enumerate().filter(|(_, o)| reflete(o)) {
        let geometria::Fronteira::Poligono(v) = &o.fronteira else {
            continue;
        };
        let albedo = o.albedo.map(|a| a.min(teto));
        let normais = geometria::normais_para_fora(v);
        let mut arestas = Vec::with_capacity(v.len());
        for (i, &a) in v.iter().enumerate() {
            let b = v[(i + 1) % v.len()];
            let e = [b[0] - a[0], b[1] - a[1]];
            let n = ((e[0].hypot(e[1]) / passo).ceil() as usize).max(1);
            arestas.push((n, todos.len()));
            let ponto = |s: f64| [a[0] + s * e[0], a[1] + s * e[1]];
            for j in 0..n {
                if j > 0 {
                    els.pontos.push(ponto(j as f64 / n as f64));
                }
                let m = ponto((j as f64 + 0.5) / n as f64);
                todos.push(Elemento {
                    albedo,
                    fora: [m[0] + FORA * normais[i][0], m[1] + FORA * normais[i][1]],
                });
            }
        }
        els.refletores.push(Refletor {
            objeto: k,
            vertices: v.clone(),
            arestas,
        });
    }
    (els, todos)
}

/// ⭐ **A série do rebote resolvida** sobre uma cena: depois de [`Rebote::resolver`], a fluência e a
/// luz recebida em qualquer ponto com os rebotes pedidos.
pub struct Rebote {
    objetos: Vec<Objeto>,
    pontuais: Vec<Pontual>,
    ceu: [f64; 3],
    elementos: Elementos,
    radiancia: Vec<[f64; 3]>,
    residuo: f64,
}

impl Rebote {
    /// Resolve `op.rebotes` rebotes com elementos de no máximo `op.passo_m`.
    ///
    /// # Panics
    /// Com `passo_m` não finito ou `≤ 0`.
    #[must_use]
    pub fn resolver(cena: &CenaDeLuz, op: OpcoesDoRebote) -> Self {
        assert!(op.passo_m.is_finite() && op.passo_m > 0.0, "passo_m > 0");
        let (objetos, pontuais) = montar(cena);
        let ceu = rgb(cena.ceu);
        let (elementos, todos) = partir(&objetos, op.passo_m);
        // As linhas de cada elemento (a matriz), uma vez: cada rebote é só o produto.
        let linhas = em_paralelo(&todos, |e| {
            linha(&objetos, &pontuais, ceu, &elementos, e.fora)
        });
        let mut radiancia = vec![[0.0; 3]; todos.len()];
        let mut residuo = 0.0;
        for _ in 0..op.rebotes {
            let nova: Vec<[f64; 3]> = todos
                .iter()
                .zip(&linhas)
                .map(|(e, l)| {
                    let f = aplicar(l, &radiancia);
                    [0, 1, 2].map(|c| e.albedo[c] * f[c] / TAU)
                })
                .collect();
            residuo = nova
                .iter()
                .zip(&radiancia)
                .flat_map(|(a, b)| (0..3).map(move |c| (a[c] - b[c]).abs()))
                .fold(0.0, f64::max);
            radiancia = nova;
        }
        Self {
            objetos,
            pontuais,
            ceu,
            elementos,
            radiancia,
            residuo,
        }
    }

    /// A fluência em `ponto` com os rebotes resolvidos.
    #[must_use]
    pub fn fluencia(&self, ponto: [f64; 2]) -> [f64; 3] {
        let l = linha(
            &self.objetos,
            &self.pontuais,
            self.ceu,
            &self.elementos,
            ponto,
        );
        aplicar(&l, &self.radiancia)
    }

    /// A luz que uma superfície de frente recebe em `ponto`: `F / 2π` (plano §2.3).
    #[must_use]
    pub fn luz_em(&self, ponto: [f64; 2]) -> [f64; 3] {
        self.fluencia(ponto).map(|f| f / TAU)
    }

    /// A maior mudança de radiância de um elemento no ÚLTIMO rebote (0 sem rebotes).
    #[must_use]
    pub fn residuo(&self) -> f64 {
        self.residuo
    }

    /// Quantos elementos a fronteira dos refletores tem.
    #[must_use]
    pub fn n_elementos(&self) -> usize {
        self.radiancia.len()
    }
}

/// `direto + Σ peso · L(elemento)`.
fn aplicar(l: &Linha, radiancia: &[[f64; 3]]) -> [f64; 3] {
    let mut f = l.direto;
    for &(e, w) in &l.pesos {
        for c in 0..3 {
            f[c] += w * radiancia[e][c];
        }
    }
    f
}

/// `f` em cada item, em blocos contíguos por fio — a ordem (e o bit) não dependem dos fios.
fn em_paralelo<T: Sync, R: Send>(itens: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let fios = std::thread::available_parallelism().map_or(1, usize::from);
    let bloco = itens.len().div_ceil(fios).max(1);
    std::thread::scope(|s| {
        let tarefas: Vec<_> = itens
            .chunks(bloco)
            .map(|c| s.spawn(|| c.iter().map(&f).collect::<Vec<R>>()))
            .collect();
        tarefas
            .into_iter()
            .flat_map(|t| t.join().expect("um fio do rebote"))
            .collect()
    })
}
