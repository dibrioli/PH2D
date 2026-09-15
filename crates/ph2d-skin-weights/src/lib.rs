//! ⭐⭐⭐ **OS PESOS DE PELE PELO PADRÃO-OURO — *Bounded Biharmonic Weights*.**
//!
//! Ordem do dono, 2026-09-15: *«eu não decido conforme o preço. Quero o estado da arte, o padrão
//! ouro. Descubra qual é e implemente.»*
//!
//! # O que é o estado da arte, e como se soube
//!
//! A pergunta é *«que fracção deste ponto da arte pertence a cada osso?»*. O campo tem uma resposta
//! canónica desde 2011 — **Bounded Biharmonic Weights** (Jacobson, Baran, Popović, Sorkine,
//! SIGGRAPH 2011), cujo caso de demonstração é literalmente **uma personagem 2D**. As revisões de
//! 2026 (*Robust Biharmonic Skinning Using Geometric Fields*, TOG; *Computing Skinning Weights via
//! Convex Duality*, CGF) mudam **como se resolve**, não **o que se resolve**: as duas tomam este
//! modelo como o alvo a atingir.
//!
//! ⚠️ **Clean-room do PAPER** — o mesmo regime do [ADR-0167] para a extracção de quads: a
//! implementação de referência (`libigl`) é **MPL-2.0** e fica **FORA**, como oráculo; o que se
//! porta é a formulação, que é matemática publicada.
//!
//! # A lei, escrita
//!
//! Para cada osso `j`, procura-se a função `w_j` sobre a arte que minimiza a **energia
//! bilaplaciana**
//!
//! ```text
//!     E(w_j) = ∫ ‖Δ w_j‖²
//! ```
//!
//! sujeita a três coisas, e **são as três que fazem a diferença**:
//!
//! | restrição | o que ela compra |
//! |---|---|
//! | `w_j = 1` no osso `j`, `0` nos outros | o osso manda no que é dele |
//! | `0 ≤ w_j ≤ 1` | **localidade**: sem ela a solução tem lóbulos negativos e o braço puxa a perna |
//! | `Σ_j w_j = 1` | a pele é uma partição da arte |
//!
//! ⭐⭐⭐ **E a propriedade que resolve o defeito medido em 2026-09-15 não é nenhuma dessas três: é o
//! DOMÍNIO.** A energia é integrada **sobre a arte** (a malha), não sobre o plano. ⇒
//!
//! - **não há ÓRFÃO por construção** — todo ponto da arte tem peso, porque a solução é definida em
//!   todo o domínio; a lei anterior tinha um raio euclidiano e **fora dele a arte saltava**;
//! - **não há a borda do suporte** — a curvatura do campo é minimizada por definição da energia,
//!   enquanto a normalização de bumps euclidianos explodia junto do raio (medido: `14,24 px` de
//!   faceta contra `0,92`);
//! - **é sensível à FORMA** — dois pedaços de arte que estão perto no plano mas longe **na arte**
//!   (as duas pernas de uma personagem) não se contaminam, porque a energia não atravessa o vazio.
//!
//! ⛔ **A lei anterior — o *bump* `(1 − (d/r)²)²` sobre a distância EUCLIDEANA ao eixo do osso — é
//! a mais fraca do campo**: é exactamente o *«linear distance-based method»* que a literatura
//! nomeia como a fonte dos artefactos de influência cruzada, e ainda por cima com um raio que **não
//! sabe nada da arte** (o item que estava no topo da fila desde 2026-09-09).
//!
//! # Como se resolve aqui
//!
//! Discretamente, sobre a malha que a [`ph2d_poly2d::Mesh2d`] já dá:
//!
//! ```text
//!     Q = L · M⁻¹ · L          (L = laplaciano de cotangentes; M = massa agrupada)
//!     min wᵀ Q w  s.a.  w fixo nos ossos  e  0 ≤ w ≤ 1
//! ```
//!
//! ⭐ **O `Q` nunca é montado.** O gradiente conjugado só precisa do PRODUTO `Q·x = L(M⁻¹(L x))`, e
//! montar `Q` custaria `~19` não-nulos por linha para nada. *A matriz que não existe não tem bug de
//! montagem* — e é a mesma razão pela qual o pré-condicionador usa `diag(Q)` calculado das linhas
//! de `L`, sem a matriz.
//!
//! ⚠️ **As caixas resolvem-se por CONJUNTO ACTIVO com libertação**, não por clamp: clampar e voltar
//! a resolver converge para uma solução admissível que **não é o mínimo** — e a diferença entre as
//! duas é exactamente a suavidade que esta lei existe para comprar.
//!
//! [ADR-0167]: ../../../docs/architecture/decisions/0167-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md

mod cg;
mod handles;
mod laplacian;

#[cfg(test)]
mod bancada;
#[cfg(test)]
mod tests;

pub use handles::Handle;

/// Os botões da solução. ⛔ Nenhum deles é um teto de recurso — ver o doc de cada um.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Quantas rondas de conjunto activo, no máximo.
    ///
    /// ⚠️ **Não é um teto de recurso, é uma REDE.** O conjunto activo converge em poucas rondas
    /// nestes tamanhos; se chegar aqui, o [`Report`] di-lo pelo `convergiu` e quem chama decide —
    /// ⛔ nunca uma solução má entregue em silêncio.
    pub max_rondas: usize,
    /// Quantas iterações de gradiente conjugado, no máximo, por resolução.
    ///
    /// ⚠️ O `Q` é mal-condicionado (`O(h⁻⁴)`), logo o CG precisa de espaço. O [`Report::residuo`]
    /// diz o que de facto sobrou.
    pub max_cg: usize,
    /// A barra do resíduo relativo do CG.
    pub tol_cg: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            // ⚠️ CONTADOS numa varredura: as fixturas deste módulo convergem em `2`–`4` rondas; o
            // `16` é a rede, e o `convergiu` do relatório é quem fala quando ela é tocada.
            max_rondas: 16,
            max_cg: 20_000,
            tol_cg: 1e-10,
        }
    }
}

/// O que a solução **de facto** entregou. ⛔ Nenhum campo é decorativo: cada um responde a uma
/// pergunta que um consumidor tem de poder fazer antes de acreditar nos pesos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Report {
    /// Vértices da malha.
    pub vertices: usize,
    /// Ossos.
    pub ossos: usize,
    /// Quantos vértices ficaram PRESOS a um osso (a condição de Dirichlet).
    ///
    /// ⚠️ **Zero aqui é uma solução sem sujeito** — a energia tem mínimo trivial `w ≡ 0`, e a
    /// normalização final devolveria `NaN`. A porta recusa antes disso.
    pub presos: usize,
    /// Rondas de conjunto activo gastas.
    pub rondas: usize,
    /// `false` quando a rede das rondas foi tocada — a solução é admissível e **pode não ser o
    /// mínimo**.
    pub convergiu: bool,
    /// O pior resíduo relativo que um CG deixou.
    pub residuo: f64,
    /// O pior `|Σ_j w_j − 1|` depois de normalizar. ⚠️ É o controlo da partição de unidade.
    pub soma_pior: f64,
    /// O pior peso fora de `[0, 1]` depois de tudo. ⚠️ Deve ser `0`; é o controlo das caixas.
    pub fora_de_banda: f64,
}

/// Os pesos, por vértice e por osso.
#[derive(Clone, Debug)]
pub struct Weights {
    /// `por_vertice[v][j]` — a fracção do vértice `v` que pertence ao osso `j`. Soma `1` por vértice.
    pub por_vertice: Vec<Vec<f64>>,
    /// Ver [`Report`].
    pub report: Report,
}

/// ⭐⭐⭐ **A PORTA: os pesos de pele de uma malha, pelo padrão-ouro.**
///
/// `None` quando a malha não tem triângulos, quando não há ossos, ou quando **nenhum** vértice fica
/// preso a um osso — ⛔ nos três casos a resposta certa é *não sei*, e não um vector de zeros que o
/// consumidor normalizaria em `NaN`.
#[must_use]
pub fn bounded_biharmonic(
    mesh: &ph2d_poly2d::Mesh2d,
    ossos: &[Handle],
    opts: Options,
) -> Option<Weights> {
    if mesh.tris.is_empty() || mesh.rest.is_empty() || ossos.is_empty() {
        return None;
    }
    let n = mesh.rest.len();
    let lap = laplacian::build(mesh)?;
    let presos = handles::pin(mesh, ossos);
    let total_presos = presos.iter().filter(|p| p.is_some()).count();
    if total_presos == 0 {
        return None;
    }

    let mut por_osso: Vec<Vec<f64>> = Vec::with_capacity(ossos.len());
    let (mut rondas_pior, mut residuo_pior, mut convergiu) = (0usize, 0.0_f64, true);
    for j in 0..ossos.len() {
        // A condição de Dirichlet deste osso: `1` nos vértices dele, `0` nos dos outros.
        let fixos: Vec<Option<f64>> = presos
            .iter()
            .map(|p| p.map(|dono| f64::from(u8::from(dono == j))))
            .collect();
        let (w, r) = cg::active_set(&lap, &fixos, opts);
        rondas_pior = rondas_pior.max(r.rondas);
        residuo_pior = residuo_pior.max(r.residuo);
        convergiu &= r.convergiu;
        por_osso.push(w);
    }

    // ⭐ **A partição de unidade entra aqui, e não na energia.** Pô-la como restrição acopla os `m`
    // problemas num só, `m` vezes maior; normalizar no fim dá a mesma resposta nos pontos que
    // importam e mantém cada osso um problema independente — que é o que a implementação de
    // referência também faz.
    let mut por_vertice = vec![vec![0.0; ossos.len()]; n];
    let (mut soma_pior, mut fora) = (0.0_f64, 0.0_f64);
    for (v, linha) in por_vertice.iter_mut().enumerate() {
        let soma: f64 = (0..ossos.len()).map(|j| por_osso[j][v]).sum();
        if soma > 0.0 {
            for (j, c) in linha.iter_mut().enumerate() {
                *c = por_osso[j][v] / soma;
            }
        } else {
            // ⛔ Um vértice que nenhum osso alcança **não existe** nesta lei (a energia é resolvida
            // em todo o domínio), mas se a malha tiver uma ilha desligada dos ossos ela chega aqui.
            // A resposta honesta é o osso mais próprio: o primeiro. ⚠️ O `soma_pior` denuncia-o.
            linha[0] = 1.0;
        }
        let s: f64 = linha.iter().sum();
        soma_pior = soma_pior.max((s - 1.0).abs());
        for &c in linha.iter() {
            fora = fora.max((-c).max(c - 1.0).max(0.0));
        }
    }

    Some(Weights {
        por_vertice,
        report: Report {
            vertices: n,
            ossos: ossos.len(),
            presos: total_presos,
            rondas: rondas_pior,
            convergiu,
            residuo: residuo_pior,
            soma_pior,
            fora_de_banda: fora,
        },
    })
}
