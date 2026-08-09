//! **O TRANSFORM PONDERADO PELA MÁSCARA** — mover, girar e escalar a parte
//! LIVRE da malha.
//!
//! É a metade do `Transform.js` da referência que porta: o **gizmo** não vem
//! (o gizmo deste repo é o de SPRITE, 2D — ADR-0110/0111), mas a lei que ele
//! dirigia vem, e vem **corrigida**.
//!
//! # A lei, e por que ela não é a da referência
//!
//! A referência aplica a matriz e **lerpa a POSIÇÃO** pelo peso:
//! `p' = (1−w)·p + w·(M·p)`. Para uma **translação** isso é exato — a
//! combinação convexa de duas translações é uma translação. Para uma
//! **ROTAÇÃO** não é: o lerp corta pela **CORDA**, e o vértice cai *dentro* do
//! círculo em vez de sobre ele.
//!
//! Medido (`tests/measure_transform.rs`), num vértice a distância `r` do eixo:
//!
//! | θ | w | r antes | fracionária | lerp |
//! |---|---|---|---|---|
//! | 90° | 0,500 | 1,00000 | **1,00000** | 0,70711 |
//! | 180° | 0,500 | 1,00000 | **1,00000** | **0,00000** |
//!
//! A 180° com meio peso o lerp **colapsa o vértice sobre o eixo**. E a banda de
//! peso intermediário é *exatamente* o que um pincel de máscara macio pinta —
//! ou seja, o defeito mora onde o artista trabalha, não numa borda exótica.
//!
//! ⚠️ **É a mesma doença que a `line/FLIP` nomeou no tween v2** (*"um lerp corta
//! pela CORDA, então todo giro encolhia o traço"*), e a cura é a mesma: aplicar
//! a **fração do MOVIMENTO**, nunca a fração da posição. Cada um dos três tem
//! forma fechada exata:
//!
//! * mover  — `p = pre + w·t`
//! * girar  — `p = pivô + R(eixo, w·θ)·(pre − pivô)`
//! * escalar — `p = pivô + (pre − pivô)·s^w`
//!
//! Com `w = 1` os três reduzem ao transform inteiro; com `w = 0`, à identidade.
//! **A representação apaga o caso especial**: não há "lerp" a corrigir, porque
//! nunca se interpola uma posição.
//!
//! # O `pre` congelado
//!
//! O gesto lê a foto do pen-down e escreve `f(pre, gesto TOTAL)` — nunca um
//! incremento sobre o resultado anterior. É a lei que este módulo (e o relevo do
//! Painter, quatro vezes) já pagou: um transform incremental faria o resultado
//! depender da **taxa de polling**, e girar 90° num tranco daria outra forma que
//! girar 90° em vinte passos.

use crate::mask_ops::free_weight;
use ph2d_mesh::Mesh;

/// O menor fator de escala que um gesto pode pedir — abaixo disto a peça
/// colapsa num ponto e não há gesto que a traga de volta.
///
/// ⚠️ **Não é um teto de recurso, é o piso da inversibilidade**: `s^w` com
/// `s <= 0` não é uma escala, é um espelho (ou uma singularidade), e um arrasto
/// que passe por zero tem de continuar sendo um arrasto — o mesmo raciocínio do
/// clamp de [`ph2d_mesh::Pose::new`].
pub const MIN_SCALE_FACTOR: f32 = 1.0e-3;

/// O que o gesto FAZ com a parte livre.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TransformKind {
    /// Translada no plano da tela.
    #[default]
    Move,
    /// Gira em torno do eixo da VISTA, pelo pivô.
    Rotate,
    /// Escala uniformemente em torno do pivô.
    Scale,
}

impl TransformKind {
    /// O rótulo que o painel mostra. ⚠️ Os nomes saem **daqui**, nunca de uma
    /// tabela paralela no painel — é a mesma regra dos chips de verbo.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Move => "Move",
            Self::Rotate => "Rotate",
            Self::Scale => "Scale",
        }
    }

    /// A ordem em que o painel os pinta.
    pub const ALL: [Self; 3] = [Self::Move, Self::Rotate, Self::Scale];
}

/// O gesto **TOTAL**, medido do pen-down até agora, em coordenadas LOCAIS da
/// malha.
///
/// ⚠️ Total e não incremental — ver o cabeçalho do módulo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesture {
    /// O deslocamento inteiro do dedo, já descido à escala da peça.
    Move { delta: [f32; 3] },
    /// O ângulo varrido, em torno de um eixo unitário.
    Rotate { axis: [f32; 3], radians: f32 },
    /// A razão de escala pedida (1,0 = nada).
    Scale { factor: f32 },
}

impl Gesture {
    /// O gesto NULO do tipo dado — o que um pen-down produz antes de a mão
    /// andar, e o que devolve a malha ao `pre`.
    #[must_use]
    pub fn neutral(kind: TransformKind) -> Self {
        match kind {
            TransformKind::Move => Self::Move { delta: [0.0; 3] },
            TransformKind::Rotate => Self::Rotate {
                axis: [0.0, 1.0, 0.0],
                radians: 0.0,
            },
            TransformKind::Scale => Self::Scale { factor: 1.0 },
        }
    }
}

/// A sessão de um gesto: a foto do pen-down e quem se move.
///
/// ⚠️ Os três vetores são **paralelos e do tamanho de quem SE MOVE**, não da
/// malha: um vértice totalmente protegido não entra na lista, então ele não é
/// lido, não é escrito e não aparece na janela do undo. É a mesma escolha da
/// referência (`getUnmaskedVertices` filtra `mask > 0`), e aqui ela também
/// **impede deriva numérica** em quem o artista mandou não se mexer.
pub struct MaskTransform {
    moving: Vec<u32>,
    pre: Vec<[f32; 3]>,
    weight: Vec<f32>,
    pivot: [f32; 3],
    /// O rascunho do refresh, reusado entre eventos — o mesmo desenho do
    /// [`crate::SculptStroke`], e pela mesma razão: um `RegionScratch` local
    /// realocaria a vizinhança inteira a cada movimento do dedo.
    region: ph2d_mesh::RegionScratch,
}

impl MaskTransform {
    /// Congela a foto. `None` quando **nada** pode se mover (a malha inteira
    /// está protegida) — e a recusa é do chamador reportar.
    #[must_use]
    pub fn begin(mesh: &Mesh) -> Option<Self> {
        let n = mesh.vert_count();
        let masks = mesh.masks();
        let mut moving = Vec::new();
        let mut pre = Vec::new();
        let mut weight = Vec::new();
        // O centroide PONDERADO: o centro do que de fato vai se mover, que é
        // onde o ZBrush põe o gizmo. Um centroide simples giraria a parte livre
        // em torno do meio da peça INTEIRA, incluindo o que está pregado.
        let mut acc = [0.0f64; 3];
        let mut wsum = 0.0f64;
        for i in 0..n {
            let w = masks.map_or(1.0, |m| free_weight(m[i]));
            if w <= 0.0 {
                continue;
            }
            let p = mesh.positions()[i];
            moving.push(i as u32);
            pre.push(p);
            weight.push(w);
            let wd = f64::from(w);
            for k in 0..3 {
                acc[k] += f64::from(p[k]) * wd;
            }
            wsum += wd;
        }
        if moving.is_empty() || wsum <= 0.0 {
            return None;
        }
        let pivot = [
            (acc[0] / wsum) as f32,
            (acc[1] / wsum) as f32,
            (acc[2] / wsum) as f32,
        ];
        Some(Self {
            moving,
            pre,
            weight,
            pivot,
            region: ph2d_mesh::RegionScratch::default(),
        })
    }

    /// O centro do que se move, em coordenadas LOCAIS.
    #[must_use]
    pub fn pivot(&self) -> [f32; 3] {
        self.pivot
    }

    /// Quantos vértices este gesto pode mover.
    #[must_use]
    pub fn moving_count(&self) -> usize {
        self.moving.len()
    }

    /// Os índices que se movem — **a janela do undo**, e o `moved` que o
    /// [`Mesh::refresh_region`] quer.
    #[must_use]
    pub fn moving(&self) -> &[u32] {
        &self.moving
    }

    /// As posições de ANTES, na ordem de [`Self::moving`] — o outro lado da
    /// entrada de undo.
    #[must_use]
    pub fn base_positions(&self) -> &[[f32; 3]] {
        &self.pre
    }

    /// Escreve `f(pre, gesto)` na malha **e conserta a vizinhança que ele
    /// moveu**. Idempotente: chamar duas vezes com o mesmo gesto dá o mesmo
    /// resultado, porque a entrada é sempre o `pre`.
    ///
    /// ⚠️ **O refresh mora AQUI, como no [`crate::SculptStroke::dab`]:** quem
    /// escreve posições é quem sabe quais escreveu, e um chamador que esquecesse
    /// de o pedir ficaria com normais velhas — que não falham, **escurecem**.
    ///
    /// ⚠️ **E ele é a metade CARA do evento**, medido em
    /// `tests/measure_transform.rs`: a 98k vértices aplicar custa 0,277 ms e o
    /// refresh **2,094** — 88%. Um dab mexe na pegada e a vizinhança a
    /// re-descobrir é local; este gesto mexe em dois terços da malha.
    pub fn apply(&mut self, mesh: &mut Mesh, gesture: &Gesture) {
        self.write(mesh, gesture);
        mesh.refresh_region(&self.moving, &mut self.region);
    }

    /// A escrita crua, sem o refresh — separada só para o [`Self::apply`] caber
    /// numa leitura.
    fn write(&self, mesh: &mut Mesh, gesture: &Gesture) {
        let out = mesh.positions_mut();
        match *gesture {
            Gesture::Move { delta } => {
                for (k, &i) in self.moving.iter().enumerate() {
                    let (p, w) = (self.pre[k], self.weight[k]);
                    out[i as usize] = [
                        p[0] + delta[0] * w,
                        p[1] + delta[1] * w,
                        p[2] + delta[2] * w,
                    ];
                }
            }
            Gesture::Rotate { axis, radians } => {
                let axis = normalize(axis);
                let pivot = self.pivot;
                for (k, &i) in self.moving.iter().enumerate() {
                    let (p, w) = (self.pre[k], self.weight[k]);
                    let d = [p[0] - pivot[0], p[1] - pivot[1], p[2] - pivot[2]];
                    // ⚠️ A FRAÇÃO cai no ÂNGULO, nunca na posição.
                    let r = rodrigues(d, axis, radians * w);
                    out[i as usize] = [pivot[0] + r[0], pivot[1] + r[1], pivot[2] + r[2]];
                }
            }
            Gesture::Scale { factor } => {
                let factor = factor.max(MIN_SCALE_FACTOR);
                let pivot = self.pivot;
                for (k, &i) in self.moving.iter().enumerate() {
                    let (p, w) = (self.pre[k], self.weight[k]);
                    // ⚠️ A FRAÇÃO é um EXPOENTE, não um lerp: `s^0 = 1` e
                    // `s^1 = s`, e a composição de dois gestos é o produto dos
                    // fatores — que é o que uma escala É.
                    let s = factor.powf(w);
                    out[i as usize] = [
                        pivot[0] + (p[0] - pivot[0]) * s,
                        pivot[1] + (p[1] - pivot[1]) * s,
                        pivot[2] + (p[2] - pivot[2]) * s,
                    ];
                }
            }
        }
    }
}

/// Roda `p` em torno de `axis` (unitário) por `radians`.
fn rodrigues(p: [f32; 3], axis: [f32; 3], radians: f32) -> [f32; 3] {
    let (s, c) = radians.sin_cos();
    let d = axis[0] * p[0] + axis[1] * p[1] + axis[2] * p[2];
    let cross = [
        axis[1] * p[2] - axis[2] * p[1],
        axis[2] * p[0] - axis[0] * p[2],
        axis[0] * p[1] - axis[1] * p[0],
    ];
    [
        p[0].mul_add(c, cross[0].mul_add(s, axis[0] * d * (1.0 - c))),
        p[1].mul_add(c, cross[1].mul_add(s, axis[1] * d * (1.0 - c))),
        p[2].mul_add(c, cross[2].mul_add(s, axis[2] * d * (1.0 - c))),
    ]
}

/// Normaliza, caindo no eixo Y se o vetor for degenerado — um eixo de giro
/// nulo faria a rotação devolver `NaN` para a malha inteira.
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = v[2].mul_add(v[2], v[0].mul_add(v[0], v[1] * v[1])).sqrt();
    if len > 0.0 && len.is_finite() {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 1.0, 0.0]
    }
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
