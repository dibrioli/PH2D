#![forbid(unsafe_code)]
//! **A PELE** — *Linear Blend Skinning* de um caminho vectorial sobre um esqueleto
//! (estudo 42 item 5, [doc 47](../../../docs/Vector%20Module/47_o_desenho_ganha_ossos.md)).
//!
//! ```text
//! p' = Σ ŵ_j · (M_j · p)
//! ```
//!
//! Um ponto é movido pela mudança de referencial de **cada** osso, misturada pelos pesos dele. É a
//! *smooth skin* do Maya, o *Armature modifier* do Blender, o `Skin`/`Tendon` do Rive e a
//! *region binding* do Moho — e o LBS é a lei que toda a indústria assenta, porque a mistura de
//! afins ainda é um afim: **uma Bézier deformada continua a ser uma Bézier, exacta e editável.**
//! (É essa a diferença para a `ph2d-vec-envelope`, cujo mapa não é afim e por isso paga
//! `sample + fit`.)
//!
//! # As três metades de um vértice skinam-se em SEPARADO
//!
//! Âncora, alça de entrada e alça de saída são **três pontos**, cada um com os pesos da posição
//! **dele**. É o `CubicWeight` do Rive, e é o que a `ph2d-vec-scene` prometia desde a ADR-0108
//! (*"os três skinados independentemente na Fase 1"*). Pesar o vértice inteiro pela âncora faria
//! uma alça que atravessa uma junta rodar com o osso errado.
//!
//! # ⭐ O repouso é a IDENTIDADE, e cai da álgebra
//!
//! `M_j = S⁻¹ ∘ B_j ∘ rest_j⁻¹`, e `rest_j` **é** `S ∘ B_j` no instante em que se ligou ⇒ parado no
//! repouso todo `M_j` é a identidade, e `Σ ŵ_j · p = p` **seja qual for o peso**. A lei da casa
//! (*todo motor novo é no-op no ponto neutro*) não precisa aqui de uma guarda escrita à mão.

use ph2d_vec_scene::{VecPath, Xform};

/// **Um osso, do ponto de vista da pele** — tudo já no espaço LOCAL da forma.
///
/// ⚠️ Ele não sabe o que é uma entidade, um `Transform` ou uma hierarquia: a shell resolve a
/// cinemática (que é a propagação de `Transform` que a casa já corre) e entrega o resultado. É o
/// que mantém esta crate uma folha testável sem mundo ECS nenhum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkinBone {
    /// A origem do eixo de repouso, em espaço da forma.
    pub rest_a: [f64; 2],
    /// A ponta do eixo de repouso, em espaço da forma.
    pub rest_b: [f64; 2],
    /// O raio de influência, em unidades da forma. `0` ⇒ este osso só ganha um ponto pelo
    /// desempate do órfão (ver [`Skin::point`]).
    pub radius: f64,
    /// `M_j` — leva um ponto do repouso para onde ESTE osso o quer agora.
    pub pose: Xform,
}

impl SkinBone {
    /// **A porta única da composição** — `S⁻¹ ∘ B ∘ rest⁻¹`, na ordem certa.
    ///
    /// ⚠️ **A ordem é o defeito clássico** e por isso ela vive numa função só: `Xform::then(outer)`
    /// aplica `self` PRIMEIRO, então a leitura da esquerda para a direita é o inverso da fórmula.
    /// Escrita em dois sítios, ela diverge no primeiro que alguém refactorar.
    ///
    /// `None` quando o repouso é singular (uma forma ou um osso escalados a zero no bind) — o
    /// chamador salta o osso, e os outros renormalizam sozinhos.
    #[must_use]
    pub fn new(
        rest: Xform,
        length: f64,
        strength: f64,
        bone_world: Xform,
        shape_world_inv: Xform,
    ) -> Option<Self> {
        let rest_inv = rest.inverse()?;
        let rest_a = rest.apply([0.0, 0.0]);
        let rest_b = rest.apply([length, 0.0]);
        // ⭐ **O raio mede-se no eixo JÁ no espaço da forma**, e não como `strength · length`: assim
        // ele carrega a escala do bind de graça, e um esqueleto ligado a uma forma escalada 3×
        // alcança 3× mais longe — que é o que o artista vê.
        let span = (rest_b[0] - rest_a[0]).hypot(rest_b[1] - rest_a[1]);
        Some(Self {
            rest_a,
            rest_b,
            radius: (span * strength).max(0.0),
            pose: rest_inv.then(&bone_world).then(&shape_world_inv),
        })
    }
}

/// **A pele de UMA forma** — os ossos a que ela está presa, já resolvidos para este quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct Skin {
    bones: Vec<SkinBone>,
}

impl Skin {
    /// `None` sem osso nenhum — uma pele vazia não é a identidade, é a **ausência** de pele, e o
    /// chamador tem de deixar a forma em paz em vez de a passar por um mapa que não existe.
    #[must_use]
    pub fn new(bones: Vec<SkinBone>) -> Option<Self> {
        (!bones.is_empty()).then_some(Self { bones })
    }

    /// Quantos ossos esta pele tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bones.len()
    }

    /// Sempre `false` — [`Skin::new`] recusa a pele vazia. Existe para o `clippy::len_without_is_empty`
    /// e para dizer isso em voz alta.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bones.is_empty()
    }

    /// Os ossos, para quem quer medir (as sondas e os gates).
    #[must_use]
    pub fn bones(&self) -> &[SkinBone] {
        &self.bones
    }

    /// **Os pesos NORMALIZADOS de um ponto**, escritos em `w` (que tem de ter `len()` casas).
    ///
    /// Devolve `false` quando o ponto é **órfão** — fora do raio de todo osso. Nesse caso `w` sai
    /// com `1` no osso mais próximo e `0` no resto: o *point binding* do Moho, e a razão está no
    /// [doc 47 §2.4](../../../docs/Vector%20Module/47_o_desenho_ganha_ossos.md) — com suporte
    /// infinito (`1/d²`) um ponto longe de tudo passa a seguir a **média do esqueleto**, e a aba de
    /// um chapéu atrasa-se atrás da cabeça.
    pub fn weights_at(&self, p: [f64; 2], w: &mut [f64]) -> bool {
        debug_assert_eq!(w.len(), self.bones.len());
        let mut soma = 0.0;
        let (mut perto, mut perto_d2) = (0usize, f64::INFINITY);
        for (i, b) in self.bones.iter().enumerate() {
            let d2 = dist2_to_segment(p, b.rest_a, b.rest_b);
            if d2 < perto_d2 {
                (perto, perto_d2) = (i, d2);
            }
            // O bump `(1 − x²)²` com `x = d/r`: `1` no eixo, **`0` E derivada `0`** na borda. É a
            // continuidade C¹ que faz um ponto atravessar a fronteira de influência sem estalo.
            let peso = if b.radius > 0.0 {
                let x2 = d2 / (b.radius * b.radius);
                if x2 < 1.0 {
                    let t = 1.0 - x2;
                    t * t
                } else {
                    0.0
                }
            } else {
                0.0
            };
            w[i] = peso;
            soma += peso;
        }
        if soma > 0.0 {
            for v in w.iter_mut() {
                *v /= soma;
            }
            return true;
        }
        // ⛔ **Nada de podar pesos pequenos.** Uma catraca por baixo (`w < 1/256 ⇒ 0`) devolveria
        // exactamente o estalo que o bump C¹ existe para evitar: o osso saltaria de `1/256` para
        // zero no meio do movimento. O suporte já é finito — não há cauda para cortar.
        for (i, v) in w.iter_mut().enumerate() {
            *v = f64::from(u8::from(i == perto));
        }
        false
    }

    /// Onde este ponto vai parar. `w` é o rascunho de [`Skin::weights_at`], reusado por ponto para
    /// a pele inteira não alocar uma vez por vértice.
    #[must_use]
    pub fn point(&self, p: [f64; 2], w: &mut [f64]) -> [f64; 2] {
        self.weights_at(p, w);
        let mut out = [0.0, 0.0];
        for (b, &peso) in self.bones.iter().zip(w.iter()) {
            if peso == 0.0 {
                continue;
            }
            let q = b.pose.apply(p);
            out[0] += peso * q[0];
            out[1] += peso * q[1];
        }
        out
    }

    /// **Deforma a forma inteira, em lugar** — âncora e as duas alças de todo vértice de todo
    /// contorno.
    ///
    /// ⚠️ **O `corner_radius` viaja INTACTO.** Ele é fonte (o raio que o cozimento resolve), e a
    /// deformação de uma pele é localmente quase-rígida — escalá-lo pediria um factor por VÉRTICE,
    /// que é a mesma conta da caneta do bug #27 (`√|det|`) mas com um afim diferente por ponto.
    /// Fica **nomeado**, não esquecido.
    pub fn apply(&self, path: &mut VecPath) {
        let mut w = vec![0.0; self.bones.len()];
        path.for_each_vert_mut(|v| {
            v.anchor = self.point(v.anchor, &mut w);
            v.in_handle = self.point(v.in_handle, &mut w);
            v.out_handle = self.point(v.out_handle, &mut w);
        });
    }
}

/// Distância AO QUADRADO de `p` ao segmento `a..b` (a raiz nunca é precisa: a lei compara com
/// `r²` e a mistura só usa razões).
#[must_use]
pub fn dist2_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (abx, aby) = (b[0] - a[0], b[1] - a[1]);
    let (apx, apy) = (p[0] - a[0], p[1] - a[1]);
    let len2 = abx * abx + aby * aby;
    // Osso de comprimento zero ⇒ a distância é ao PONTO. (Um osso assim não influencia ninguém
    // pelo raio — só pode ganhar o desempate do órfão —, e aqui isso resolve-se sozinho.)
    let t = if len2 > 0.0 {
        ((apx * abx + apy * aby) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (dx, dy) = (apx - t * abx, apy - t * aby);
    dx * dx + dy * dy
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
