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

/// Teto de cópias POR EIXO. O número é medido — ver a tabela na tabela de parâmetros
/// (`PathEffect::params`).
const MAX_PER_AXIS: usize = 128;

/// Teto do PRODUTO dos dois eixos — o teto de UM eixo não é o teto de uma grelha, porque o custo
/// é o produto: `128 × 128` seriam 16 384 cópias.
///
/// **1024, e o número é MEDIDO** (CLAUDE.md §0). Silhueta de 24 âncoras, custo de `cooked()`:
///
/// | grelha  | 4×4  | 8×8  |16×16 |32×32 |
/// |---------|------|------|------|------|
/// | ms/cook |0,009 |0,037 |0,170 |0,660 |
///
/// Linear no número de cópias, como seria de esperar. 1024 custa 0,66 ms — já se sente num frame
/// de 60 fps partilhado com o resto do editor, e é aí que o corte fica. ⚠️ O recurso por medir
/// continua a ser o RENDER de 1024 contornos, não o cozimento; quem quiser subir mede ESSE.
const MAX_TOTAL: usize = 1024;

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
    /// Cópias ao longo de **x**, contando o original. `1` = este eixo não copia.
    ///
    /// A contagem **é** o interruptor do eixo: `1` desliga-o. Um toggle separado seria uma
    /// segunda resposta a *"este eixo está ligado?"*, e duas respostas divergem.
    pub copies_x: f64,
    /// Deslocamento por cópia em x, em **percentagem da LARGURA** da forma. `100` encaixa sem
    /// folga, `50` sobrepõe metade, `200` deixa um vão de uma forma.
    pub move_x: f64,
    /// Cópias ao longo de **y**. Com os dois eixos acima de 1, o resultado é uma **grelha**.
    pub copies_y: f64,
    /// Deslocamento por cópia em y, em percentagem da **ALTURA**.
    pub move_y: f64,
    /// **Spin** — cada cópia roda sobre o centro DELA PRÓPRIA, em graus por cópia. As cópias
    /// ficam onde estão e viram: uma fileira continua fileira e ganha um leque.
    pub spin: f64,
    /// **Orbit** — cada cópia roda em torno do centro do ORIGINAL, em graus por cópia. Combinada
    /// com deslocamento, é a espiral; é o *Object Offset* do Array do Blender.
    ///
    /// ⚠️ As duas rotações existem porque **fazem coisas diferentes e as duas servem** (Enio,
    /// 2026-07-18). A 1ª versão tinha só a órbita, a 2ª substituiu-a pelo spin — e substituir era
    /// o erro: sobre uma cópia deslocada, girar sobre si mesma e girar em torno da origem são
    /// duas ferramentas, não duas opiniões sobre a mesma.
    pub orbit: f64,
}

impl Default for RepeatSpec {
    fn default() -> Self {
        Self {
            copies_x: 1.0,
            move_x: 0.0,
            copies_y: 1.0,
            move_y: 0.0,
            spin: 0.0,
            orbit: 0.0,
        }
    }
}

impl RepeatSpec {
    /// Uma cópia em cada eixo é a forma. O neutro tem de ser no-op byte-idêntico, senão a pilha
    /// não pode saltá-lo e o `Cow::Borrowed` morre (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.count_x() * self.count_y() <= 1
    }

    /// A contagem inteira do eixo x, saneada.
    #[must_use]
    pub fn count_x(&self) -> usize {
        Self::count(self.copies_x)
    }

    /// A contagem inteira do eixo y, saneada.
    #[must_use]
    pub fn count_y(&self) -> usize {
        Self::count(self.copies_y)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn count(v: f64) -> usize {
        (v.floor().max(MIN_COPIES) as usize).min(MAX_PER_AXIS)
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
    /// `self ∘ rhs` — aplica `rhs` primeiro.
    fn then(self, rhs: Self) -> Self {
        Self {
            a: rhs.a.mul_add(self.a, rhs.c * self.b),
            b: rhs.b.mul_add(self.a, rhs.d * self.b),
            c: rhs.a.mul_add(self.c, rhs.c * self.d),
            d: rhs.b.mul_add(self.c, rhs.d * self.d),
            tx: rhs.tx.mul_add(self.a, rhs.ty.mul_add(self.b, self.tx)),
            ty: rhs.tx.mul_add(self.c, rhs.ty.mul_add(self.d, self.ty)),
        }
    }

    fn apply(self, p: [f64; 2]) -> [f64; 2] {
        [
            p[0].mul_add(self.a, p[1].mul_add(self.b, self.tx)),
            p[0].mul_add(self.c, p[1].mul_add(self.d, self.ty)),
        ]
    }
}

/// **A transformação da cópia `(ix, iy)`** da grelha.
///
/// Três coisas, nesta ordem, e cada botão faz exatamente uma:
///
/// 1. **spin** — roda a forma sobre o centro dela própria;
/// 2. **move** — desloca `ix` larguras em x e `iy` alturas em y;
/// 3. **orbit** — roda o conjunto em torno do centro do ORIGINAL.
///
/// O índice das rotações é `ix + iy`, a distância em cópias até ao original: numa fileira é uma
/// rampa linear, numa grelha é um gradiente diagonal. O índice linear (`iy·nx + ix`) daria um
/// salto no fim de cada linha, que se vê.
fn transform_for(
    ix: usize,
    iy: usize,
    spec: &RepeatSpec,
    size: [f64; 2],
    center: [f64; 2],
) -> Affine {
    #[allow(clippy::cast_precision_loss)]
    let (fx, fy) = (ix as f64, iy as f64);
    let step = fx + fy;
    let deg = |d: f64| d * step / HALF_TURN_DEG * core::f64::consts::PI;
    // `sin`/`cos` uma vez por CÓPIA, nunca por ponto.
    let (ss, sc) = deg(spec.spin).sin_cos();
    let (os, oc) = deg(spec.orbit).sin_cos();
    // POR EIXO: x pela largura, y pela altura. É o que faz `100` encaixar exatamente.
    let (dx, dy) = (
        spec.move_x / 100.0 * size[0] * fx,
        spec.move_y / 100.0 * size[1] * fy,
    );
    let (ox, oy) = (center[0], center[1]);

    // spin em torno do centro, depois translação: `p ↦ R_s(p − c) + c + d`.
    let a1 = Affine {
        a: sc,
        b: -ss,
        c: ss,
        d: sc,
        tx: dx + ox - (sc * ox - ss * oy),
        ty: dy + oy - ss.mul_add(ox, sc * oy),
    };
    // e a órbita, também em torno do centro do ORIGINAL: `q ↦ R_o(q − c) + c`.
    let a2 = Affine {
        a: oc,
        b: -os,
        c: os,
        d: oc,
        tx: ox - (oc * ox - os * oy),
        ty: oy - os.mul_add(ox, oc * oy),
    };
    a2.then(a1)
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
    let (nx, ny) = (spec.count_x(), spec.count_y());
    let (size, center) = measure(path);
    // Um passo que não move NADA (sem distância e sem ângulo) empilharia as cópias exatamente em
    // cima da forma: invisível, e caro. É o neutro pela outra porta.
    let step = transform_for(1, 0, spec, size, center);
    let step_y = transform_for(0, 1, spec, size, center);
    let inert = |m: &Affine| {
        m.tx.abs() <= EPS && m.ty.abs() <= EPS && (m.a - 1.0).abs() <= EPS && m.b.abs() <= EPS
    };
    if inert(&step) && inert(&step_y) {
        return out;
    }

    // Os contornos ORIGINAIS, capturados antes de a saída crescer — copiar de uma lista que se
    // está a estender daria cópias das cópias.
    let source: Vec<(Vec<VecVertex>, bool)> = (0..path.contour_count())
        .filter_map(|k| path.contour(k).map(|(v, cl)| (v.to_vec(), cl)))
        .collect();

    let mut made = 1usize; // o original conta para o teto do produto
    for iy in 0..ny {
        for ix in 0..nx {
            if ix == 0 && iy == 0 {
                continue; // o original já está em `out`
            }
            if made >= MAX_TOTAL {
                return out;
            }
            made += 1;
            let m = transform_for(ix, iy, spec, size, center);
            for (verts, closed) in &source {
                out.subpaths.push(Contour {
                    verts: verts.iter().map(|v| map_vert(v, m)).collect(),
                    closed: *closed,
                });
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "fx_repeat_tests.rs"]
mod tests;
