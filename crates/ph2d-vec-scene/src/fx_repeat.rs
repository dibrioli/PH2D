//! **Repeater** — o terceiro efeito da pilha (ADR-0132), e o primeiro que MULTIPLICA contornos.
//!
//! N cópias da forma, cada uma com a transformação aplicada mais uma vez. É o *Repeater* do
//! After Effects e o *Transform* do Illustrator, e é o staple de motion graphics: fileiras,
//! catavento, escadas, espirais.
//!
//! # O que o distingue dos dois primeiros efeitos
//!
//! O Trim e o Zig Zag são **por-contorno** — cada contorno entra, um contorno sai, e o
//! `apply_per_contour` trata os buracos de um compound independentemente. O Repeater não cabe
//! nessa porta: ele toma a forma INTEIRA (contorno primário **mais** os buracos) e emite `n`
//! cópias dela. Um buraco copiado sozinho deixaria de ser buraco.
//!
//! Então este é o primeiro efeito a construir a saída diretamente, e é por isso que
//! `PathEffect::apply` ramifica por variante em vez de encaminhar tudo para uma função só.
//!
//! # A transformação é CUMULATIVA, e é isso que dá a espiral
//!
//! A cópia `k` leva `M^k`, não `k·M`: a matriz é composta consigo mesma. Com só translação as
//! duas coincidem (uma fileira); com rotação **e** translação a composição espirala, enquanto o
//! múltiplo desenharia um arco de raio fixo. A espiral é o comportamento do AE, e é o que o
//! artista espera quando mexe nos dois botões ao mesmo tempo.
//!
//! # As distâncias são RELATIVAS e POR EIXO — o *Relative Offset* do Blender
//!
//! `Move X = 100` desloca cada cópia por exatamente uma **LARGURA** da forma; `Move Y = 100`,
//! por uma **ALTURA**. Então `100` cola as cópias sem folga, `50` sobrepõe metade, `200` deixa
//! um vão de uma forma. É o *Relative Offset Factor* do modificador **Array** do Blender, e a
//! propriedade que o torna útil é essa: um número redondo produz um encaixe exato.
//!
//! ⚠️ **A 1ª versão dividia os dois eixos pela MÉDIA das dimensões** (`ref_size`), herdada do
//! `Size` do Zig Zag. Numa forma quadrada ninguém nota; numa forma alta, `Move X = 100` desloca
//! por `(w+h)/2` e as cópias ficam com folga ou sobrepostas — nunca encaixadas. Era o que
//! impedia as *"combinações interessantes"* que o Enio pediu (2026-07-18).
//!
//! A média continua certa **para o Zig Zag**: uma amplitude é isotrópica, uma onda não tem eixo.
//! Aqui a grandeza é *quanto mede a forma NAQUELA direção*, que é outra pergunta.
//!
//! # E por isso este efeito mede a sua ENTRADA, não o caminho autorado
//!
//! O [`FxCtx`] é medido UMA vez no caminho autorado, para que um botão signifique o mesmo
//! independentemente da ordem da pilha. O Repeater diverge dessa lei **de propósito**: ladrilhar
//! é uma operação sobre *a coisa que está a ser ladrilhada*, e essa é a entrada.
//!
//! É o que faz a GRELHA cair de graça, exatamente como no Blender — onde uma grelha é **dois
//! modificadores Array empilhados**:
//!
//! ```text
//! Repeater(Move X = 100, Copies = 5)   →  uma fileira de 5
//! Repeater(Move Y = 100, Copies = 3)   →  três fileiras: uma grelha 5×3
//! ```
//!
//! Com o `ctx` autorado, o 2º Repeater mediria a forma ORIGINAL e as fileiras sobrepor-se-iam
//! quando o passo anterior não fosse em Y. Medindo a entrada, cada um ladrilha o que recebeu.

use crate::{Contour, VecPath, VecVertex};

/// Abaixo disto uma distância é zero.
const EPS: f64 = 1e-12;

/// Menos de duas cópias não é repetição — é a própria forma.
const MIN_COPIES: f64 = 1.0;

/// Meia-volta em graus, para converter sem constante mágica.
const HALF_TURN_DEG: f64 = 180.0;

/// **A medida da forma que chega**: `(tamanho por eixo, centro)` da caixa de controle.
///
/// Um eixo DEGENERADO (uma reta horizontal não tem altura) cairia num deslocamento sempre nulo,
/// e o botão daquele eixo ficaria morto — a mesma lacuna que o Blender tapa com o *Constant
/// Offset*, que aqui não cabe no orçamento de parâmetros. Então um eixo sem extensão empresta a
/// do outro: o controle continua vivo e o número continua a significar *"uma forma de
/// distância"*.
fn measure(path: &VecPath) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    let mut seen = false;
    for v in path.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            seen = true;
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
    }
    if !seen {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut size = [hi[0] - lo[0], hi[1] - lo[1]];
    let other = [size[1], size[0]];
    for k in 0..2 {
        if size[k] <= EPS {
            size[k] = other[k];
        }
    }
    (size, [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5])
}

/// **Os parâmetros do Repeater.** Neutro em `copies <= 1`.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RepeatSpec {
    /// Quantas cópias ao todo, contando o original. `1` = neutro.
    pub copies: f64,
    /// Deslocamento por cópia no eixo x, em **percentagem da forma** ([`FxCtx::ref_size`]).
    pub move_x: f64,
    /// Idem em y.
    pub move_y: f64,
    /// Rotação por cópia, em **graus**, em torno do centro da forma. Cumulativa.
    pub rotate: f64,
}

impl Default for RepeatSpec {
    fn default() -> Self {
        Self {
            copies: 1.0,
            move_x: 0.0,
            move_y: 0.0,
            rotate: 0.0,
        }
    }
}

impl RepeatSpec {
    /// Uma cópia é a forma. O neutro tem de ser no-op byte-idêntico, senão a pilha não pode
    /// saltá-lo e o `Cow::Borrowed` morre (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.copies < MIN_COPIES + 1.0
    }
}

/// Um afim 2D em linha-maior: `[a, b, tx; c, d, ty]`.
#[derive(Copy, Clone, Debug)]
struct Affine {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}

impl Affine {
    fn apply(self, p: [f64; 2]) -> [f64; 2] {
        [
            p[0].mul_add(self.a, p[1].mul_add(self.b, self.tx)),
            p[0].mul_add(self.c, p[1].mul_add(self.d, self.ty)),
        ]
    }
}

/// **A transformação da cópia `k`**: rodada `k·θ` em torno do centro DELA, deslocada `k·d`.
///
/// ⚠️ Não é `M^k`. A 1ª versão compunha a matriz consigo mesma, com a rotação ancorada no centro
/// do ORIGINAL — matematicamente uma espiral, e visualmente as cópias **orbitam e espalham-se**
/// em vez de ladrilhar. A folha de contacto mostrou-o à primeira. Aqui mover é mover e girar é
/// girar cada cópia, e os dois não se contaminam: uma fileira continua fileira quando se
/// acrescenta rotação, e ganha-se um leque.
fn transform_for(k: usize, spec: &RepeatSpec, size: [f64; 2], center: [f64; 2]) -> Affine {
    // `sin`/`cos` uma vez por CÓPIA (≤128), nunca por ponto.
    #[allow(clippy::cast_precision_loss)]
    let kf = k as f64;
    let rad = spec.rotate * kf / HALF_TURN_DEG * core::f64::consts::PI;
    let (s, c) = rad.sin_cos();
    // POR EIXO: x pela largura, y pela altura. É o que faz `100` encaixar exatamente.
    let (dx, dy) = (
        spec.move_x / 100.0 * size[0] * kf,
        spec.move_y / 100.0 * size[1] * kf,
    );
    let (ox, oy) = (center[0], center[1]);
    // T(centro) ∘ R(θ) ∘ T(−centro), com a translação somada no fim.
    Affine {
        a: c,
        b: -s,
        c: s,
        d: c,
        tx: dx + ox - (c * ox - s * oy),
        ty: dy + oy - s.mul_add(ox, c * oy),
    }
}

fn map_vert(v: &VecVertex, m: Affine) -> VecVertex {
    VecVertex {
        anchor: m.apply(v.anchor),
        in_handle: m.apply(v.in_handle),
        out_handle: m.apply(v.out_handle),
        kind: v.kind,
        // O afim é rotação + translação (sem escala), então um comprimento LOCAL sobrevive
        // intacto. Um parâmetro de escala teria de multiplicá-lo aqui — a mesma conversão que
        // o raio do gradiente radial já faz.
        corner_radius: v.corner_radius,
    }
}

/// **Aplica o Repeater à forma inteira.** O original fica onde está; as cópias entram como
/// contornos adicionais.
///
/// A `fill_rule` do caminho é preservada — duas cópias sobrepostas de uma forma com buraco
/// continuam a vazar pela regra que o artista escolheu.
#[must_use]
pub fn repeat_path(path: &VecPath, spec: &RepeatSpec) -> VecPath {
    let mut out = path.clone();
    if spec.is_neutral() {
        return out;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let copies = spec.copies.floor().max(MIN_COPIES) as usize;
    let (size, center) = measure(path);
    let step = transform_for(1, spec, size, center);
    // Um passo que não move NADA (sem distância e sem ângulo) empilharia `n` cópias exatamente
    // em cima da forma: invisível, e caro. É o neutro pela outra porta.
    if step.tx.abs() <= EPS
        && step.ty.abs() <= EPS
        && (step.a - 1.0).abs() <= EPS
        && step.b.abs() <= EPS
    {
        return out;
    }

    // Os contornos ORIGINAIS, capturados antes de a saída crescer — copiar de uma lista que se
    // está a estender daria cópias das cópias.
    let source: Vec<(Vec<VecVertex>, bool)> = (0..path.contour_count())
        .filter_map(|k| path.contour(k).map(|(v, cl)| (v.to_vec(), cl)))
        .collect();

    for k in 1..copies {
        let m = transform_for(k, spec, size, center);
        for (verts, closed) in &source {
            out.subpaths.push(Contour {
                verts: verts.iter().map(|v| map_vert(v, m)).collect(),
                closed: *closed,
            });
        }
    }
    out
}

#[cfg(test)]
#[path = "fx_repeat_tests.rs"]
mod tests;
