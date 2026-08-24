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

/// ⭐⭐ **O NÍVEL DE RESOLUÇÃO de omissão** de uma forma ainda ligada ao desenho (W55).
///
/// É o joelho que a tabela do `ph2d_field_profile::TOLERANCE_RATIO` mediu: a silhueta erra 0,009 %
/// da peça e o salto de normal fica em 2,14°, o que apagou 98 % dos pixels em degrau. *O default é
/// o número certo; o knob existe para a peça que é grande ou vista de perto.*
pub const DEFAULT_PROFILE_RESOLUTION: u32 = 1;

/// ⭐⭐ **O TETO do nível de resolução — e o recurso dele é o TRAÇADO ASSENTE** (W55).
///
/// Cada nível divide a tolerância de cozimento por si (`ph2d_field_profile::tolerance_ratio_for`).
/// Numa curva suave a contagem de arestas anda com `tol^-1/2`, e o custo do traçado é **linear** nas
/// arestas — então o preço de um nível cresce com a **raiz** dele, e não com ele:
///
/// | nível | tolerância | arestas | traçado assente | *idem*, calmo |
/// |---:|---:|---:|---:|---:|
/// | **1** (omissão) | `1e-4` | 168 | 184,1 ms | *139 ms* |
/// | 2 | `5e-5` | 236 | 241,4 ms | *183 ms* |
/// | 4 | `2,5e-5` | 332 | 336,0 ms | *254 ms* |
/// | 8 | `1,25e-5` | 472 | 450,3 ms | *341 ms* |
/// | **16** (teto) | `6,3e-6` | 664 | 648,7 ms | *491 ms* |
/// | 32 | `3,1e-6` | 940 | 900,5 ms | *682 ms* |
///
/// (sonda `field3d_profile::tests::the_table_that_chose_the_resolution_ceiling`: círculo de raio 0,5
/// extrudado, 640×480, mediana de 7.)
///
/// ⚠️ **A coluna «calmo» é DERIVADA, e a razão está medida ao lado.** A corrida saiu com `load ≈ 4,7`
/// — abaixo do inutilizável, acima do ideal —, e a linha do nível 1 é a **mesma configuração** que a
/// W54 mediu a `load < 3` em **139,3 ms**. As duas leituras do mesmo trabalho dão **184,1** e
/// **139,3**: ⭐ *32 % de diferença só de carga, sem uma linha de código mudar* — que é exactamente
/// por que a lei do `CLAUDE.md` §5 existe. A coluna calma é a medida escalada por esse fator (0,757),
/// e o teto escolhido sobre a coluna **medida** é, por isso, conservador.
///
/// ⭐ **O teto é 16 porque é onde o assentar deixa de parecer instantâneo.** Meio segundo depois de
/// cada gesto é o limite em que o artista ainda lê a espera como *"está a afinar"* em vez de *"o app
/// prendeu"* — e este knob **arrasta-se**, então cada passo do arrasto paga aquilo. O nível 32 não
/// compra nada que se veja (o salto de normal já está em 0,54° no 16) e paga **39 %** a mais.
///
/// ⚠️ **O custo é linear nas arestas** — 0,95 a 1,10 ms/aresta ao longo da tabela inteira —, então
/// não há joelho onde se esconder: o teto é uma escolha de produto sobre uma reta, e diz de que
/// recurso é.
///
/// ⚠️ **É um limite de RECURSO e não de validade**: um perfil de 940 arestas é perfeitamente
/// correcto, e o documento aceita-o por outra porta (`Profile::new` com a tolerância à mão). O que
/// este número fecha é a **faixa do controle**, que é onde um teto pertence.
pub const MAX_PROFILE_RESOLUTION: u32 = 16;

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
