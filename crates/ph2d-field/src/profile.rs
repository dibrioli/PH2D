//! **O perfil 2D** — a figura plana fechada de onde saem `Extrude` e `Revolve` ([ADR-0161]).
//!
//! É a peça que liga o modelador 3D à **caneta que a casa já tem**: o artista desenha no editor
//! vetorial, e o desenho vira sólido. O fluxo do MoI (*desenhar o contorno, depois extrudar ou
//! revolucionar*) nasce daqui.
//!
//! # ⚠️ O perfil é COZIDO, e é isso que ele guarda
//!
//! O que mora aqui é uma **polilinha fechada**, não uma Bézier. Não é preguiça: a distância exata a
//! uma cúbica exige resolver uma quíntica, que não é exprimível na árvore de avaliação — nem o
//! `libfive` o faz. O que se faz é **achatar com tolerância declarada**, e é por isso que a
//! tolerância viaja **dentro** do perfil ([`Profile::tolerance`]): sem ela, "este perfil está bom?"
//! é uma pergunta sem resposta, e re-cozinhar a fonte com outro número passa despercebido.
//!
//! Isto é a lei **fonte ≠ cozido** do editor vetorial ([ADR-0121]/[ADR-0132]) aplicada uma camada
//! acima: a **fonte** continua a ser o path do documento vetorial, com os handles e o raio vivo de
//! quina; o **cozido** é o que este tipo guarda.
//!
//! ⭐ **O arredondamento de quina do perfil vem de graça** — quem coze usa a geometria já cozida do
//! path, então o *corner widget* do editor vetorial já entregou os arcos. O módulo 3D não tem, e
//! não deve ter, uma segunda resposta para "arredondar a quina de um contorno".
//!
//! # Por que a regra de preenchimento é COPIADA e não importada
//!
//! [`FillRule`] repete o tipo homónimo da `ph2d-vec-scene` de propósito. Esta crate é **o
//! documento** e não pode depender do documento de outro módulo: um `ph2d-field` que importasse o
//! modelo vetorial faria um arquivo salvo do modelador depender do schema do editor de vetores, e
//! um passaria a quebrar o outro. A conversão é trabalho de quem coze (`ph2d-field-profile`).
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
//! [ADR-0121]: ../../../docs/architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md
//! [ADR-0132]: ../../../docs/architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md

use serde::{Deserialize, Serialize};

/// Como os contornos de um perfil se combinam.
///
/// Para um perfil de **um** contorno as duas regras coincidem — a distinção só existe quando há
/// ilha ou buraco.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FillRule {
    /// Respeita a orientação de cada contorno (winding). É o default, e é o que faz um contorno
    /// desenhado ao contrário virar buraco.
    #[default]
    NonZero,
    /// Alterna dentro/fora a cada cruzamento: um contorno aninhado é buraco **independente de como
    /// foi orientado**. É a regra robusta para geometria vinda de booleana.
    EvenOdd,
}

/// Por que um perfil foi recusado.
///
/// ⚠️ Como o resto desta crate, nenhuma variante é zelo: um perfil inválido não dá erro na
/// avaliação — ele dá um sólido errado, em silêncio.
// Sem `Eq`: as variantes carregam os `f32` que explicam a recusa.
#[derive(Clone, Debug, PartialEq)]
pub enum ProfileError {
    /// Nenhum contorno. Uma figura vazia não delimita sólido nenhum.
    Empty,
    /// Menos de 3 pontos: não fecha área.
    TooFewPoints { contour: u32, points: u32 },
    /// Coordenada não-finita.
    NonFinite { contour: u32 },
    /// O contorno colapsou numa reta ou num ponto — uma das extensões da caixa dele é zero.
    ///
    /// ⚠️ É este o teste, e **não** a área: uma figura em oito tem área líquida zero e é um perfil
    /// perfeitamente legítimo sob [`FillRule::EvenOdd`]. Recusar por área mataria o caso válido e
    /// deixaria passar o degenerado de verdade.
    Collapsed {
        contour: u32,
        width: f32,
        height: f32,
    },
    /// A tolerância de cozimento não é um número positivo finito.
    BadTolerance { tolerance: f32 },
}

/// Uma figura plana fechada, já achatada em polilinhas.
///
/// Os campos são privados e a única porta é [`Profile::new`]: um `Profile` que exista está válido.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// Contornos **fechados**. O segmento de fecho (último → primeiro) é implícito: o primeiro
    /// ponto **não** se repete no fim. Repeti-lo produziria uma aresta de comprimento zero, e uma
    /// aresta de comprimento zero é uma divisão por zero na distância ponto-segmento.
    contours: Vec<Vec<[f32; 2]>>,
    fill: FillRule,
    tolerance: f32,
}

impl Profile {
    /// Constrói e **valida**.
    ///
    /// Pontos consecutivos repetidos são **removidos** (inclusive o fecho, se quem chamou repetiu o
    /// primeiro ponto no fim) — é limpeza de entrada, não uma decisão de forma: um ponto repetido
    /// não muda a figura e só existe para quebrar a distância ponto-segmento.
    ///
    /// # Errors
    /// Ver [`ProfileError`].
    pub fn new(
        contours: Vec<Vec<[f32; 2]>>,
        fill: FillRule,
        tolerance: f32,
    ) -> Result<Self, ProfileError> {
        if !tolerance.is_finite() || tolerance <= 0.0 {
            return Err(ProfileError::BadTolerance { tolerance });
        }
        if contours.is_empty() {
            return Err(ProfileError::Empty);
        }
        let mut cleaned: Vec<Vec<[f32; 2]>> = Vec::with_capacity(contours.len());
        for (i, raw) in contours.into_iter().enumerate() {
            let idx = i as u32;
            if raw.iter().any(|p| !p[0].is_finite() || !p[1].is_finite()) {
                return Err(ProfileError::NonFinite { contour: idx });
            }
            let c = dedup_closed(&raw);
            let n = c.len() as u32;
            if n < 3 {
                return Err(ProfileError::TooFewPoints {
                    contour: idx,
                    points: n,
                });
            }
            let (min, max) = contour_bounds(&c);
            let (w, h) = (max[0] - min[0], max[1] - min[1]);
            if w <= 0.0 || h <= 0.0 {
                return Err(ProfileError::Collapsed {
                    contour: idx,
                    width: w,
                    height: h,
                });
            }
            cleaned.push(c);
        }
        Ok(Self {
            contours: cleaned,
            fill,
            tolerance,
        })
    }

    #[must_use]
    pub fn contours(&self) -> &[Vec<[f32; 2]>] {
        &self.contours
    }

    #[must_use]
    pub fn fill(&self) -> FillRule {
        self.fill
    }

    /// A tolerância com que este perfil foi achatado a partir da fonte — **o erro máximo entre esta
    /// polilinha e a curva que a originou**, em unidades do documento.
    #[must_use]
    pub fn tolerance(&self) -> f32 {
        self.tolerance
    }

    /// Quantas arestas o perfil tem ao todo.
    ///
    /// ⚠️ **É o número que manda no custo**: cada aresta vira **~26 nós** na árvore de avaliação
    /// (medido, `docs/3DModeling/04_resultados_perfis.md` §3), e o traçado avalia a árvore inteira
    /// por pixel. Quem mexer na tolerância mexe aqui.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.contours.iter().map(Vec::len).sum()
    }

    /// A caixa envolvente `(min, max)` de todos os contornos.
    #[must_use]
    pub fn bounds(&self) -> ([f32; 2], [f32; 2]) {
        let mut min = [f32::INFINITY; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        for c in &self.contours {
            let (a, b) = contour_bounds(c);
            for k in 0..2 {
                min[k] = min[k].min(a[k]);
                max[k] = max[k].max(b[k]);
            }
        }
        (min, max)
    }
}

/// Remove pontos consecutivos idênticos, **tratando a lista como fechada** (o último é vizinho do
/// primeiro).
fn dedup_closed(pts: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut out: Vec<[f32; 2]> = Vec::with_capacity(pts.len());
    for &p in pts {
        if out.last() != Some(&p) {
            out.push(p);
        }
    }
    // O fecho: se o último coincide com o primeiro, ele é a aresta de comprimento zero.
    while out.len() > 1 && out.last() == out.first() {
        out.pop();
    }
    out
}

fn contour_bounds(c: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    let mut min = [f32::INFINITY; 2];
    let mut max = [f32::NEG_INFINITY; 2];
    for p in c {
        for k in 0..2 {
            min[k] = min[k].min(p[k]);
            max[k] = max[k].max(p[k]);
        }
    }
    (min, max)
}
