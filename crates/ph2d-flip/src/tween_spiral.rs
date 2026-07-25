//! **A espiral logarítmica** — o movimento rígido do traço entre A e B (Tween v2, `04 §2`).
//!
//! O lerp de coordenadas **ENCOLHE**: um braço que gira 180° passa pelo segmento que liga
//! as duas poses, e no meio do caminho ele é uma linha de comprimento zero. É o defeito
//! clássico do inbetween por interpolação de posição, e a resposta clássica (BetweenIT,
//! Whited et al., Disney EG 2010 — matemática completa na patente EP2315179A2) é
//! interpolar a **TRANSFORMAÇÃO de similaridade**, não a posição:
//!
//! ```text
//! S(a) = σ·R(θ)·a + c          a similaridade que leva A em B (Umeyama, forma fechada)
//! F    = (I − σR)⁻¹·c          o ponto FIXO de S
//! P(t) = F + σᵗ·R(θt)·(a − F)  a espiral: gira e escala em torno de F
//! ```
//!
//! Três coisas que valem mais que a fórmula:
//!
//! 1. **A representação apaga o caso especial.** Quando a similaridade é uma translação
//!    pura (`σ=1`, `θ=0`) o ponto fixo NÃO EXISTE — a matriz `I − σR` é singular. Em vez de
//!    um ramo `if` no chamador, isso vira a variante [`StrokeMotion::Translate`], e a
//!    fórmula do chamador (`advance(a,u) + u·resíduo`) vale para as duas sem ramo nenhum.
//! 2. **O resíduo tem uma porta só.** `resid = b − S(a)` é *o que a similaridade não
//!    explica*, e `S(a)` é exatamente `advance(a, 1.0)` — nada de uma segunda cópia da
//!    similaridade para calcular a diferença.
//! 3. **A conta é em `f64`, a saída em `f32`.** `F` fica LONGE quando o movimento é quase
//!    uma translação (`F ≈ c/det`), e `(p − F)` seguido de `+F` é cancelamento catastrófico.
//!    Em `f64` sobram dígitos de sobra; em `f32` não sobrariam.
//!
//! Orçamento HR-5 (regra 9 do plano — *"1 sincos POR STROKE, nunca por vértice"*): o ajuste
//! gasta **um `atan2`** por traço; a avaliação, **um `sincos` + um `powf`** por traço e por
//! inbetween. Os pontos só somam e multiplicam. Todos pela crate `libm` (porte puro-Rust do
//! MUSL, **igual em toda plataforma** — é o `sin` da `std`, que chama a libm do SISTEMA, que
//! diverge entre OSes).

use ph2d_core::Vec2;

/// Abaixo deste `det(I − σR)` a similaridade é uma **translação pura** (ou perto demais
/// disso para ser distinguível), e o movimento vira [`StrokeMotion::Translate`].
///
/// `det = 1 − 2σcos θ + σ²` — zero exatamente quando `σ=1` e `θ=0`.
///
/// **MEDIDO** por `the_spiral_ruler`, e o número tem uma razão só, que é a certa: é onde as
/// **duas fontes de erro se cruzam**. Descer a espiral joga `F ≈ c/det` para longe e o
/// `(p − F) … + F` come dígitos; subir o piso troca o arco pela corda e paga a *sagitta*
/// (`d·θ/8`). A régua põe as duas colunas lado a lado (unidades de documento):
///
/// ```text
///   θ        det       erro da espiral f32   arco − corda
///   1e-1     1.0e-2    4.43e-5               6.25e-1
///   1e-2     1.0e-4    3.78e-5               6.25e-3
///   1e-3     1.0e-6    7.29e-5               6.25e-5   ← aqui elas se encontram
/// ```
///
/// Em `θ = 1e-3` a espiral em `f32` já erra `7,3e-5` e a corda erra `6,3e-5`: abaixo desse
/// ponto **a corda é pelo menos tão exata quanto a espiral**, e é de graça. Acima, o arco
/// vale muito mais (`6,25e-1` contra `4,4e-5` uma década acima). O piso mora no encontro.
///
/// ⚠️ A 1ª versão deste comentário dizia que o erro *"explode duas ordens de grandeza por
/// década abaixo"* — **falso, e escrito antes de medir**: as linhas de baixo da tabela já
/// caem no ramo `Translate`, então o que elas mostram é o erro da CORDA, que encolhe.
const DET_MIN: f64 = 1.0e-6;

/// Escala abaixo da qual B colapsou num ponto: `σᵗ` levaria todo `t > 0` para o mesmo
/// lugar (um salto no primeiro inbetween). Cai na translação, que degrada suavemente.
const SCALE_MIN: f64 = 1.0e-3;

/// **O movimento rígido de um traço** entre dois quadros-chave.
///
/// ⚠️ A 1ª versão chamava o caso degenerado de `Lerp` e o fazia devolver o **próprio
/// ponto** — "o rígido aqui é a identidade". Era **mentira**, e o gate do órfão a pegou:
/// numa translação pura o rígido É a translação, e quem pergunta *"para onde este traço
/// foi?"* de fora do par (um órfão que precisa viajar junto) recebia "para lugar nenhum".
/// Dentro do par a mentira não aparecia, porque o resíduo cobria a translação inteira.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum StrokeMotion {
    /// Sem rotação nem escala apreciáveis: o movimento rígido é uma translação.
    Translate(Vec2),
    /// Gira e escala em torno do ponto fixo.
    Spiral {
        /// O ponto fixo da similaridade (o "centro" do movimento).
        fixed: Vec2,
        /// Ângulo TOTAL da rotação A→B (radianos, CCW).
        angle: f32,
        /// Razão de escala total A→B.
        scale: f32,
    },
}

impl StrokeMotion {
    /// **Ajusta a similaridade que leva `a` em `b`** (Umeyama 2D em forma fechada).
    ///
    /// Os dois arrays já estão em correspondência ponto a ponto (o padding e o auto-flip
    /// do tween rodam antes) — este é o mesmo pareamento que a interpolação vai usar, e
    /// não uma segunda opinião sobre quem é quem.
    pub(crate) fn fit(a: &[Vec2], b: &[Vec2]) -> Self {
        let n = a.len().min(b.len());
        if n == 0 {
            return Self::Translate(Vec2::ZERO);
        }
        if n < 2 {
            // Um ponto não define rotação nem escala — mas define para onde ele foi.
            return Self::Translate(b[0] - a[0]);
        }
        let inv = 1.0 / n as f64;
        let (mut ca, mut cb) = ((0.0f64, 0.0f64), (0.0f64, 0.0f64));
        for i in 0..n {
            ca.0 += f64::from(a[i].x);
            ca.1 += f64::from(a[i].y);
            cb.0 += f64::from(b[i].x);
            cb.1 += f64::from(b[i].y);
        }
        let (ca, cb) = ((ca.0 * inv, ca.1 * inv), (cb.0 * inv, cb.1 * inv));
        // A translação dos CENTRÓIDES é o rígido de todo caso degenerado abaixo — e é o
        // que um órfão vizinho monta para viajar junto.
        let shift = Vec2::new((cb.0 - ca.0) as f32, (cb.1 - ca.1) as f32);

        // Σ a'·b' e Σ a'×b' — a parte "complexa" do ajuste: o vetor `(dot, cross)` tem
        // ARGUMENTO θ e MÓDULO σ·Σ|a'|².
        let (mut dot, mut cross, mut norm_a) = (0.0f64, 0.0f64, 0.0f64);
        for i in 0..n {
            let (ax, ay) = (f64::from(a[i].x) - ca.0, f64::from(a[i].y) - ca.1);
            let (bx, by) = (f64::from(b[i].x) - cb.0, f64::from(b[i].y) - cb.1);
            dot += ax * bx + ay * by;
            cross += ax * by - ay * bx;
            norm_a += ax * ax + ay * ay;
        }
        if norm_a <= 0.0 {
            return Self::Translate(shift); // A é um ponto: não há forma para girar
        }
        let theta = libm::atan2(cross, dot);
        let scale = (dot * dot + cross * cross).sqrt() / norm_a;
        if scale < SCALE_MIN {
            return Self::Translate(shift); // B colapsou
        }
        let (sin, cos) = libm::sincos(theta);
        let (m00, m01) = (scale * cos, -scale * sin); // M = σ·R(θ)
        let (m10, m11) = (scale * sin, scale * cos);
        // c = b̄ − M·ā
        let cx = cb.0 - (m00 * ca.0 + m01 * ca.1);
        let cy = cb.1 - (m10 * ca.0 + m11 * ca.1);
        // (I − M)·F = c. det = (1−σcos)² + (σsin)² = 1 − 2σcos θ + σ².
        let det = 1.0 - 2.0 * scale * cos + scale * scale;
        if det < DET_MIN {
            return Self::Translate(shift); // o ponto fixo está no infinito
        }
        // (I − M)⁻¹ = 1/det · [[1−σcos, σsin·(−1)·(−1)] …] — a inversa de
        // [[1−m00, −m01], [−m10, 1−m11]] com m01 = −m10.
        let (i00, i01) = (1.0 - m00, -m01);
        let (i10, i11) = (-m10, 1.0 - m11);
        let fx = (i11 * cx - i01 * cy) / det;
        let fy = (-i10 * cx + i00 * cy) / det;
        Self::Spiral {
            fixed: Vec2::new(fx as f32, fy as f32),
            angle: theta as f32,
            scale: scale as f32,
        }
    }

    /// **Onde o ponto `p` de A está no fator `u`, pela parte RÍGIDA do movimento.**
    ///
    /// Numa `Translate` isto é `p + d·u`; numa `Spiral`, o arco. Nos dois casos é a
    /// resposta COMPLETA a *"para onde este traço leva um ponto que anda com ele?"* — o que
    /// um órfão vizinho precisa saber (ver o ⚠️ do enum).
    ///
    /// `u` fora de `[0,1]` é extrapolação legítima: a espiral **continua girando**, que é
    /// o overshoot que um animador de fato quer (antecipação/rebote), em vez de esticar
    /// uma reta para fora do movimento.
    pub(crate) fn advance(&self, p: Vec2, u: f32) -> Vec2 {
        match *self {
            Self::Translate(d) => p + d * u,
            Self::Spiral {
                fixed,
                angle,
                scale,
            } => {
                let (sin, cos) = libm::sincosf(angle * u);
                let k = libm::powf(scale, u);
                let d = p - fixed;
                fixed + Vec2::new(k * (cos * d.x - sin * d.y), k * (sin * d.x + cos * d.y))
            }
        }
    }

    /// **Onde o ponto pareado `a ↔ b` está no fator `u`** — rígido + resíduo CO-ROTACIONADO,
    /// a porta única do ponto INTERPOLADO (o `advance` responde outra pergunta: onde vai um
    /// passageiro que não tem par).
    ///
    /// ⚠️ **Na `Translate` a conta é escrita como `a + (b − a)·u`, e isso é load-bearing:**
    /// é algebricamente a mesma expressão (`a + d·u + (b − a − d)·u`), mas com **uma**
    /// multiplicação em vez de duas — e é o que faz uma translação pura sair **byte-idêntica**
    /// ao tween do v1 em vez de a um ulp dele. A forma de duas parcelas errava o último bit
    /// (medido: `36.175` × `36.174995`), e um gate de bits é muito mais difícil de fraudar
    /// que um de épsilon.
    ///
    /// ⚠️ **Na `Spiral` o resíduo é CO-ROTACIONADO, não somado no referencial do MUNDO** — é
    /// a cura do §9 do plano (*"rotação grande ainda torce o lerp do resíduo"*). O resíduo
    /// `r = b − S(a)` é *o que a similaridade global não explica*: uma feature (uma corcova,
    /// um cotovelo) **presa ao corpo**. E o corpo GIRA. No meio do caminho ele já rodou `θ·u`,
    /// mas `r` foi medido em B, com o corpo a `θ` inteiro. Somar `r·u` no referencial fixo (o
    /// v2 anterior) deixa a feature apontando para a atitude ERRADA, e a forma TORCE quanto
    /// MAIOR o giro — medido: 160°+corcova, o meio ACHATAVA a corcova (erro intrínseco 0,59
    /// contra 0,17 co-rotacionado). A cura carrega `r` do referencial de B para o referencial
    /// PARCIAL em `u`: rodado `θ(u−1)`, escalado `σ^(u−1)`.
    ///
    /// Fecha os dois extremos por construção: em `u=1` o mapa é a identidade (`R(0)`, `σ⁰`) ⇒
    /// `r` inteiro ⇒ fecha em B; em `u=0` o fator externo `·u` zera ⇒ abre em A.
    ///
    /// **Resíduo zero (similaridade PURA) ⇒ termo zero ⇒ byte-idêntico ao espiral de antes**
    /// — é o que preserva todos os gates existentes (giro-de-braço, escala geométrica, pivô,
    /// extremos: todos com `r = 0`). E `θ=0 ∧ σ=1` já cai em `Translate`, onde não há giro
    /// para descasar. É subsunção, não um modo novo.
    ///
    /// ⚠️ **Sozinho, um giro global não modela ARTICULAÇÃO** (partes girando quantidades
    /// diferentes) — mas ainda AJUDA (medido: 90°+cotovelo 80°, erro 0,35 → 0,10), porque o
    /// giro dominante é o global. O resíduo que sobra (a rotação diferencial) é a fronteira da
    /// reconstrução por-aresta do Sederberg, nomeada no §9 do plano, não construída aqui.
    pub(crate) fn point_at(&self, a: Vec2, b: Vec2, u: f32) -> Vec2 {
        match *self {
            Self::Translate(_) => a + (b - a) * u,
            Self::Spiral { angle, scale, .. } => {
                let r = self.residual(a, b);
                // `r` do referencial de B (rodado θ, escalado σ) para o de `u` (rodado θu,
                // escalado σᵘ): rotaciona por θ(u−1) e escala por σ^(u−1).
                let (sin, cos) = libm::sincosf(angle * (u - 1.0));
                let k = libm::powf(scale, u - 1.0);
                let carried = Vec2::new(k * (cos * r.x - sin * r.y), k * (sin * r.x + cos * r.y));
                self.advance(a, u) + carried * u
            }
        }
    }

    /// **O resíduo do ponto** — o que a similaridade NÃO explica (`b − S(a)`).
    ///
    /// Uma porta só: `S(a)` é `advance(a, 1.0)`, a MESMA função que a interpolação usa.
    /// Uma segunda cópia da similaridade aqui divergiria da primeira no dia em que alguém
    /// mexesse numa delas — e o sintoma seria o inbetween não fechar em B.
    pub(crate) fn residual(&self, a: Vec2, b: Vec2) -> Vec2 {
        b - self.advance(a, 1.0)
    }
}

#[cfg(test)]
#[path = "tween_spiral_tests.rs"]
mod tests;
