//! **O arranjo planar** de N formas — as FACES que o Shape Builder pinta.
//!
//! Quando N formas se sobrepõem, o plano fica dividido em regiões atômicas: cada ponto
//! está dentro de um subconjunto das formas, e dois pontos estão na mesma **face** quando
//! têm o mesmo subconjunto **e** dá para caminhar de um ao outro sem cruzar borda. É
//! sobre essas faces que o cursor do Shape Builder arrasta.
//!
//! ## Por que aqui não há um DCEL
//!
//! O reflexo é construir a subdivisão planar à mão: achar todas as interseções, quebrar as
//! curvas, montar meia-arestas, caminhar as faces. É o que a literatura manda (e o que o
//! manual de features sugere, via Clipper2).
//!
//! **Não é preciso**, e a razão é que uma face tem uma definição *conjuntista* que o motor
//! booleano que já temos responde direto:
//!
//! ```text
//! região(M) = (∩ das formas em M)  −  (∪ das formas fora de M)
//! ```
//!
//! e as componentes **desconexas** dessa região saem **separadas de graça** — o
//! [`crate::apply_many`] já devolve um `VecPath` por grupo de contenção. A pertinência `M`
//! de um ponto sai de um `contains_point` por forma, que custa nada.
//!
//! Duas consequências que valem mais que a economia de código:
//!
//! 1. **O realce e o resultado saem do MESMO motor.** A face que pisca sob o cursor é
//!    computada pela mesma booleana que produz a forma final. Não existe o bug em que o
//!    editor mostra uma região e entrega outra — a família de bug que este repo já pagou
//!    três vezes (`feedback_derived_coordinate_seed_must_match_sample`).
//! 2. **Nada de robustez numérica nova.** O `linesweeper` já é o motor endurecido do
//!    módulo; um DCEL escrito à mão traria seus próprios casos degenerados (três curvas no
//!    mesmo ponto, tangência, aresta de comprimento zero) para dentro de casa.
//!
//! **O escape hatch, se um dia o número pedir:** o `linesweeper` expõe `Topology` +
//! `WindingNumber` como trait — dá para fazer UMA varredura e extrair o contorno de
//! qualquer conjunto de faces por predicado sobre o winding. É estritamente mais eficiente
//! e estritamente mais código. Só troque depois de **medir** (o roteador de conectores
//! custou 1–6 µs e a otimização que eu ia fazer era desnecessária).

use ph2d_vec_scene::{VecPath, contains_point};

/// Teto de formas num arranjo. A pertinência é um bitmask de 32 bits, então o limite duro
/// seria 32; **16** é o limite de bom senso — o número de regiões cresce como `2^N`, e um
/// Shape Builder com 16 formas já é uma tela cheia.
pub const MAX_BUILD_SHAPES: usize = 16;

/// Quais formas cobrem um ponto: bit `i` ligado = a forma `i` o contém.
///
/// Uma pertinência **vazia** (`0`) é o lado de FORA de tudo — não é face, e não é
/// pintável.
pub type Membership = u32;

/// Uma FACE do arranjo: a pertinência, mais **qual componente conexa** dela.
///
/// A componente importa: três formas em fila podem deixar a região "só dentro da do meio"
/// partida em duas ilhas, e elas são faces DIFERENTES — o usuário espera pintar uma sem a
/// outra. Um predicado sobre a pertinência sozinho pegaria as duas.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FaceId {
    pub membership: Membership,
    pub component: usize,
}

/// O arranjo de um conjunto de formas, em coordenadas de **MUNDO**.
///
/// As formas entram já assadas (`bake_xform`): a booleana precisa de um frame único, e as
/// entidades de origem têm `Transform` diferentes — é a mesma regra que a booleana normal
/// já segue.
pub struct Arrangement {
    /// As formas de origem, em mundo, na ordem de z (fundo → topo).
    sources: Vec<VecPath>,
    /// Memo por pertinência: a geometria já calculada de cada região visitada.
    ///
    /// `Vec` e não `HashMap` de propósito (ADR-0022 proíbe iteração de `HashMap` nas
    /// crates de simulação, e a ordem determinística aqui é de graça): o número de classes
    /// VISITADAS num gesto é punhado, e a busca linear num punhado é mais rápida que o
    /// hash.
    memo: Vec<(Membership, Vec<VecPath>)>,
}

impl Arrangement {
    /// Monta o arranjo. `sources` em MUNDO, ordem de z (fundo → topo). Mais que
    /// [`MAX_BUILD_SHAPES`] formas são **ignoradas** (as do topo entram).
    #[must_use]
    pub fn new(mut sources: Vec<VecPath>) -> Self {
        sources.truncate(MAX_BUILD_SHAPES);
        Self {
            sources,
            memo: Vec::new(),
        }
    }

    /// As formas de origem (mundo).
    #[must_use]
    pub fn sources(&self) -> &[VecPath] {
        &self.sources
    }

    /// Quantas formas o arranjo tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// A pertinência de um ponto de MUNDO: quais formas o contêm.
    ///
    /// Cada forma responde pela **sua** `fill_rule` (um compound com buraco diz "fora"
    /// dentro do buraco, que é o certo), e pela geometria **cozida** — o `contains_point`
    /// já cozinha, então uma quina arredondada é apalpada pela borda que se vê.
    #[must_use]
    pub fn membership_at(&self, p: [f64; 2]) -> Membership {
        let mut m: Membership = 0;
        for (i, s) in self.sources.iter().enumerate() {
            if contains_point(s, p) {
                m |= 1 << i;
            }
        }
        m
    }

    /// A geometria da região de pertinência `m` — **uma entrada por componente conexa**.
    ///
    /// Memoizada: a pertinência sob o cursor só muda quando ele cruza uma borda, então um
    /// arrasto inteiro costuma tocar um punhado de classes.
    pub fn region(&mut self, m: Membership) -> &[VecPath] {
        if let Some(i) = self.memo.iter().position(|(k, _)| *k == m) {
            return &self.memo[i].1;
        }
        let region = self.compute_region(m);
        self.memo.push((m, region));
        &self.memo.last().expect("acabou de ser inserida").1
    }

    /// A face sob o ponto, ou `None` se ele está FORA de todas as formas.
    ///
    /// A componente é achada apalpando as componentes da região com o mesmo
    /// `contains_point` — a região veio da booleana, e o ponto tem de cair dentro de
    /// exatamente uma delas.
    pub fn face_at(&mut self, p: [f64; 2]) -> Option<FaceId> {
        let membership = self.membership_at(p);
        if membership == 0 {
            return None; // fora de tudo: não é face
        }
        let component = self
            .region(membership)
            .iter()
            .position(|c| contains_point(c, p))?;
        Some(FaceId {
            membership,
            component,
        })
    }

    /// A geometria de uma face (mundo). `None` se a face não existe mais.
    pub fn face_path(&mut self, f: FaceId) -> Option<&VecPath> {
        self.region(f.membership).get(f.component)
    }

    /// `região(M) = (∩ das formas em M) − (∪ das formas fora de M)`.
    ///
    /// Os casos degenerados de `apply_many` (que exige ≥ 2 paths) são tratados aqui, e é
    /// onde mora quase todo o cuidado:
    /// - `|M| == 1` e nada fora: a região É a forma.
    /// - `|M| == 0`: o lado de fora — não é região (vazio).
    fn compute_region(&self, m: Membership) -> Vec<VecPath> {
        let inside: Vec<&VecPath> = self.bits(m).map(|i| &self.sources[i]).collect();
        let outside: Vec<&VecPath> = self.bits(!m).map(|i| &self.sources[i]).collect();
        if inside.is_empty() {
            return Vec::new();
        }
        // (1) A interseção de tudo que cobre o ponto.
        let core: Vec<VecPath> = if inside.len() == 1 {
            vec![inside[0].clone()]
        } else {
            crate::apply_many(&inside, crate::BoolOp::Intersect)
        };
        if outside.is_empty() {
            return core;
        }
        // (2) Menos tudo que NÃO o cobre. O `Subtract` do `apply_many` é
        // `base − (c1 ∪ c2 ∪ …)` (o acumulador dobra), então uma chamada basta por
        // componente do núcleo.
        let mut out = Vec::new();
        for c in &core {
            let mut args: Vec<&VecPath> = Vec::with_capacity(1 + outside.len());
            args.push(c);
            args.extend(outside.iter().copied());
            out.extend(crate::apply_many(&args, crate::BoolOp::Subtract));
        }
        out
    }

    /// Os índices de forma ligados em `m` (limitados ao nº de formas).
    fn bits(&self, m: Membership) -> impl Iterator<Item = usize> + '_ {
        (0..self.sources.len()).filter(move |i| m & (1 << i) != 0)
    }
}

#[cfg(test)]
#[path = "arrangement_tests.rs"]
mod arrangement_tests;
