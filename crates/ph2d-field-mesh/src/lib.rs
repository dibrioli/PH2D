#![forbid(unsafe_code)]
//! `ph2d-field-mesh` — **a ponte da escultura**: uma malha vira um campo de distância *amostrado*,
//! e o campo amostrado entra na booleana do modelo implícito ([ADR-0161], plano W5).
//!
//! # ⚠️ Por que uma malha NÃO pode virar uma árvore de avaliação
//!
//! O resto do módulo compila o documento numa `fidget::context::Tree`, e essa álgebra é **fechada**:
//! `Input(Var)` · `Const` · `Binary` · `Unary` · remapeamentos. **Não há operação de consulta a
//! dados.** Um campo em voxels não é exprimível ali, e o caminho que restaria — um termo de árvore
//! por **triângulo** — custa ~10 nós por triângulo: uma escultura de 50 mil triângulos daria meio
//! milhão de nós, avaliados em cada um dos milhões de pontos que o traçado gasta por quadro.
//!
//! ⭐ **A saída é um avaliador HÍBRIDO**, e o número que a autoriza está medido
//! (`measure_sculpt_to_field_bridge`): **uma amostra trilinear custa 1,39× uma avaliação da árvore
//! com JIT** (7,6 ms contra 5,5 ms por milhão de pontos). Misturar uma folha amostrada numa árvore
//! analítica custa aproximadamente o mesmo que uma folha analítica a mais — e a booleana continua a
//! ser `min`/`max`, que é a razão de este módulo existir.
//!
//! | caminho | veredito |
//! |---|---|
//! | a malha vira **expressão** (um termo por triângulo) | ⛔ meio milhão de nós para uma escultura média |
//! | a booleana acontece na **malha** | ⛔ é exatamente o que falha, e a tese do módulo é que não falha |
//! | ⭐ folha **amostrada** dentro da árvore de operações | ✅ 1,39× por ponto, e o `min`/`max` é o mesmo |
//!
//! # ⚠️ O que o `ph2d-sdf` entrega, e o que FALTAVA
//!
//! O voxelizador e o *flood fill* já existem ali, escritos e medidos pela linha da escultura. Mas o
//! campo que eles produzem é uma **banda estreita**: a distância só é escrita nas células que caem
//! dentro da caixa de algum triângulo, e todo o resto fica em `±INFINITY`, que o amostrador de lá
//! doma para `±diagonal da grade`.
//!
//! ⛔ **Aquilo está certo para os consumidores dele** — oclusão, espessura e remesh só perguntam
//! **junto** da superfície — e é **fatal** para uma marcha de esfera: um passo do tamanho da
//! diagonal salta a peça inteira e o raio sai pelo outro lado sem a ver. Medido: o erro contra a
//! esfera analítica era de **89 células**.
//!
//! ⭐ **A cura é uma propagação de chanfro** ([`SampledField::from_mesh`]): duas varreduras sobre a
//! grade estendem a banda para o volume todo. O resultado é um **majorante** da distância euclidiana
//! (um caminho pela grade nunca é mais curto que a reta), e por isso ele é dividido por
//! [`CHAMFER_SAFETY`], que a medição escreveu — uma marcha só é correta contra um **minorante**.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

use ph2d_mesh::{Aabb, Mesh};
use ph2d_sdf::VoxelField;

/// Quantas células o maior lado da caixa recebe, quando ninguém escolhe.
///
/// ⚠️ **É uma escolha de PRODUTO com a tabela ao lado**, e não um teto de recurso. Medido
/// (`measure_sampling_cost`, malha de 25 mil triângulos, já com a folga da [`PAD_CELLS`]):
///
/// | resolução | células | memória | construir | célula |
/// |---:|---:|---:|---:|---:|
/// | 64 | 614 125 | 2,3 MB | 77 ms | 0,0188 |
/// | 96 | 1 601 613 | 6,1 MB | 163 ms | 0,0125 |
/// | **128** | **3 285 748** | **12,5 MB** | **308 ms** | **0,0094** |
/// | 192 | 9 618 228 | 36,7 MB | 839 ms | 0,0063 |
/// | 256 | 21 177 204 | 80,8 MB | 1 847 ms | 0,0047 |
///
/// ⭐ **O número de triângulos quase não conta** (57,9 ms contra 76,7 ms entre mil e 25 mil): quem
/// manda é o **cubo da resolução**, porque a propagação de chanfro visita a grade inteira duas
/// vezes. E o custo é pago **uma vez**, na importação, não por quadro.
///
/// 128 põe a célula a ~1/128 do maior lado da peça — abaixo do que o traçado resolve num viewport
/// comum — por 12,5 MB e um terço de segundo. ⛔ 256 sextuplica a memória e o relógio por um detalhe
/// que só uma exportação em Max veria: é o valor a passar à mão quando a peça o justificar, não o
/// default.
pub const DEFAULT_RESOLUTION: u32 = 128;

/// Quantas células de campo honesto existem para lá da malha.
///
/// ⚠️ **Fora da caixa a resposta é a distância à CAIXA**, que é um minorante honesto e portanto
/// seguro para a marcha — e **errado para um filete**: um arredondamento entre a escultura e uma
/// forma analítica precisa da distância à ESCULTURA nos pontos onde o filete acontece, e esses ficam
/// até um raio para fora dela. Oito células cobrem qualquer raio que faça sentido nesta resolução;
/// acima disso o que falta é resolução, não folga.
///
/// O preço é a caixa maior à mesma célula: `(res + 16)³ / res³`, que a 128 é **1,42×** de memória.
const PAD_CELLS: u32 = 8;

/// Por quanto a distância de chanfro é dividida para virar um **minorante**.
///
/// ⭐ **Uma marcha de esfera só é correta contra um minorante da distância**: superestimar faz o raio
/// atravessar a peça. A propagação por chanfro anda **pela grade**, e um caminho pela grade nunca é
/// mais curto que a reta — logo ela **super**estima, e o número que a corrige tem de ser medido, não
/// suposto.
///
/// Medido (`measure_chamfer_overshoot`, esfera analítica como oráculo):
///
/// | resolução | célula | maior razão `chanfro / verdadeiro` |
/// |---:|---:|---:|
/// | 48 | 0,02500 | 1,1151 |
/// | 96 | 0,01250 | **1,1174** |
/// | 192 | 0,00625 | 1,1000 |
///
/// ⚠️ **A medição não é o pior caso, é o pior caso VISTO** — e aqui há um limite que não depende da
/// fixture: a métrica de chanfro com pesos `(1, √2, √3)` erra no máximo ~14 % contra a euclidiana em
/// 3D, por construção da própria métrica (o pior ângulo fica entre a face e a diagonal). O valor
/// abaixo cobre esse limite com 3 % de folga sobre o pior medido.
///
/// ⛔ **A primeira volta desta wave escreveu 1,05 aqui, por palpite**, e a sonda mediu 1,10 — o
/// campo teria superestimado em 5 % e a marcha atravessaria a peça. *Um número de segurança que não
/// veio de uma tabela é o defeito que ele diz prevenir.*
///
/// ⛔ Não vale subir isto "por segurança": cada 1 % a mais é 1 % de passo perdido em **cada** passo
/// de **cada** raio. E ⛔ não vale baixá-lo para o valor medido: uma peça com outra orientação de
/// feição anda no ângulo que a esfera não visitou.
///
/// ⚠️ **A divisão é UNIFORME, e não só sobre a parte propagada.** Ela encolhe também os valores
/// exatos da banda — em **5 % de um valor que ali vale no máximo uma célula**, ou seja, muito abaixo
/// do erro que a própria grade tem. Em troca, o campo fica **monótono e sem costura** na fronteira
/// da banda; deflacionar só a propagação criaria um degrau justamente onde a marcha desacelera.
pub const CHAMFER_SAFETY: f32 = 1.15;

/// **Uma malha, como campo de distância com sinal** — pronta para entrar numa booleana.
///
/// ⚠️ **Não é serializável, e isso é a arquitetura e não uma falta.** Uma grade de 128³ pesa 16 MB;
/// pô-la dentro do documento faria cada `cook` (que corre por quadro) copiar isso, e faria um
/// projeto guardado carregar a grade em vez de a **regenerar** da malha, que é a fonte. O documento
/// guarda um NOME; quem resolve nome → campo é um registo à parte.
pub struct SampledField {
    dims: [usize; 3],
    step: f32,
    origin: [f32; 3],
    /// Distância com sinal em cada amostra — **densa**, já estendida para fora da banda.
    dist: Vec<f32>,
    /// A caixa que as amostras cobrem.
    box_: Aabb,
}

impl SampledField {
    /// Voxeliza `mesh`, estende a banda para o volume todo, e devolve o campo. `None` se a malha for
    /// vazia — *um campo vazio leria como espaço cheio, que é o oposto do que uma malha vazia é*.
    #[must_use]
    pub fn from_mesh(mesh: &Mesh, resolution: u32) -> Option<Self> {
        if mesh.faces().is_empty() || mesh.positions().is_empty() {
            return None;
        }
        // ⭐ **A caixa cresce PAD_CELLS células além da malha, e a resolução cresce com ela** — o
        // passo fica o mesmo, e o que se ganha é campo honesto à volta da peça.
        //
        // ⚠️ Sem isso a grade encosta na malha com a folga de 1,51 células que o `VoxelField`
        // reserva, e **fora dela a resposta é a distância à CAIXA**, não à peça. Isso é seguro para
        // a marcha (é um minorante) e é **errado para um filete**: um raio de arredondamento maior
        // que a folga seria calculado contra a caixa. Oito células cobrem qualquer filete que faça
        // sentido nesta resolução; acima disso o que falta é resolução, não folga.
        let b = mesh.bounds();
        let ext = [
            b.max[0] - b.min[0],
            b.max[1] - b.min[1],
            b.max[2] - b.min[2],
        ];
        let longest = ext[0].max(ext[1]).max(ext[2]).max(f32::MIN_POSITIVE);
        let step = longest / resolution.max(1) as f32;
        let pad = step * PAD_CELLS as f32;
        let padded = Aabb {
            min: [b.min[0] - pad, b.min[1] - pad, b.min[2] - pad],
            max: [b.max[0] + pad, b.max[1] + pad, b.max[2] + pad],
        };
        let mut grid = VoxelField::for_bounds(padded, resolution + 2 * PAD_CELLS);
        grid.voxelize(mesh);
        grid.flood_fill();

        let dims = grid.dims();
        let step = grid.step();
        let origin = grid.origin();
        let box_ = Aabb {
            min: origin,
            max: [
                ((dims[0] - 1) as f32).mul_add(step, origin[0]),
                ((dims[1] - 1) as f32).mul_add(step, origin[1]),
                ((dims[2] - 1) as f32).mul_add(step, origin[2]),
            ],
        };
        let dist = extend_band(grid.distances(), dims, step);
        Some(Self {
            dims,
            step,
            origin,
            dist,
            box_,
        })
    }

    /// A caixa que a grade cobre.
    #[must_use]
    pub fn bounds(&self) -> Aabb {
        self.box_
    }

    /// A aresta da célula — o **erro** desta representação, e o número que decide se ela serve para a
    /// peça em mãos.
    #[must_use]
    pub fn cell(&self) -> f32 {
        self.step
    }

    /// Quantas amostras a grade tem.
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.dist.len()
    }

    /// ⭐ **A distância com sinal em `p`** — e ela vale **em toda a parte**, não só dentro da caixa.
    ///
    /// Dentro: interpolação trilinear da grade estendida.
    /// Fora: a distância **à caixa**, que é um minorante honesto da distância à malha, porque a
    /// malha está dentro dela. É isso que deixa a marcha aproximar-se sem saltar por cima.
    #[must_use]
    pub fn at(&self, p: [f32; 3]) -> f32 {
        let outside = distance_to_box(self.box_, p);
        if outside > 0.0 {
            return outside;
        }
        let inv = 1.0 / self.step;
        let mut i0 = [0usize; 3];
        let mut frac = [0.0f32; 3];
        for a in 0..3 {
            let g = (p[a] - self.origin[a]) * inv;
            // Inclui o `NaN`: a comparação negada apanha-o sem um ramo próprio.
            if !(g >= 0.0 && g <= (self.dims[a] - 1) as f32) {
                return outside;
            }
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let i = (g.floor() as usize).min(self.dims[a] - 2);
            i0[a] = i;
            frac[a] = g - i as f32;
        }
        let rx = self.dims[0];
        let rxy = rx * self.dims[1];
        let base = i0[0] + i0[1] * rx + i0[2] * rxy;
        let c = |dx: usize, dy: usize, dz: usize| self.dist[base + dx + dy * rx + dz * rxy];
        let lerp = |a: f32, b: f32, t: f32| (b - a).mul_add(t, a);
        let (fx, fy, fz) = (frac[0], frac[1], frac[2]);
        let x00 = lerp(c(0, 0, 0), c(1, 0, 0), fx);
        let x10 = lerp(c(0, 1, 0), c(1, 1, 0), fx);
        let x01 = lerp(c(0, 0, 1), c(1, 0, 1), fx);
        let x11 = lerp(c(0, 1, 1), c(1, 1, 1), fx);
        lerp(lerp(x00, x10, fy), lerp(x01, x11, fy), fz)
    }
}

/// Os 13 vizinhos já visitados numa varredura crescente, com o comprimento do salto **em células**.
///
/// ⚠️ **O peso é o comprimento EUCLIDIANO do salto**, e não 1 por passo: um salto diagonal cobre
/// `√3` células, e cobrá-lo como 1 daria uma "distância" que anda mais depressa que a luz — um
/// majorante deixaria de ser majorante e a marcha atravessaria a peça.
const FORWARD: [([isize; 3], f32); 13] = [
    ([-1, 0, 0], 1.0),
    ([0, -1, 0], 1.0),
    ([0, 0, -1], 1.0),
    ([-1, -1, 0], std::f32::consts::SQRT_2),
    ([1, -1, 0], std::f32::consts::SQRT_2),
    ([-1, 0, -1], std::f32::consts::SQRT_2),
    ([1, 0, -1], std::f32::consts::SQRT_2),
    ([0, -1, -1], std::f32::consts::SQRT_2),
    ([0, 1, -1], std::f32::consts::SQRT_2),
    ([-1, -1, -1], 1.732_050_8),
    ([1, -1, -1], 1.732_050_8),
    ([-1, 1, -1], 1.732_050_8),
    ([1, 1, -1], 1.732_050_8),
];

/// ⭐ **Estende a banda estreita para o volume inteiro**, por propagação de chanfro.
///
/// Duas varreduras — uma crescente e uma decrescente — bastam: cada célula recebe o menor
/// `distância(vizinho) + salto`, e o par de sentidos cobre todos os caminhos monótonos da grade.
///
/// ⚠️ **O SINAL não se propaga, ele já está resolvido.** O *flood fill* do `ph2d-sdf` marcou o
/// interior com `−INFINITY` e o exterior com `+INFINITY`; o que se propaga é só a **magnitude**.
///
/// ⛔ **Havia aqui uma barreira que impedia a propagação de atravessar a superfície, e a prova de
/// mutação mostrou-a INERTE** — tirá-la não pôs um único gate a vermelho. E o motivo é aritmética,
/// não sorte: `|d|` é **1-Lipschitz em toda a parte, inclusive através do nível zero**, então um
/// caminho que atravessa a parede continua a dar um majorante válido. A barreira só **removia
/// caminhos**, e portanto só piorava o limite — numa parede fina ela deixava o meio da parede sem
/// nenhum vizinho do próprio lado com valor útil. *Uma precaução que a medição diz não fazer nada
/// não é grátis: ela é a próxima pessoa a acreditar que ela faz.*
fn extend_band(src: &[f32], dims: [usize; 3], step: f32) -> Vec<f32> {
    let (rx, ry, rz) = (dims[0], dims[1], dims[2]);
    let rxy = rx * ry;
    // A magnitude por resolver, e o lado a que cada célula pertence.
    let mut mag: Vec<f32> = src.iter().map(|d| d.abs()).collect();
    let inside: Vec<bool> = src.iter().map(|d| d.is_sign_negative()).collect();

    let mut sweep = |order: &[([isize; 3], f32)], reverse: bool| {
        let idx = |i: usize, j: usize, k: usize| i + j * rx + k * rxy;
        for kk in 0..rz {
            for jj in 0..ry {
                for ii in 0..rx {
                    let (i, j, k) = if reverse {
                        (rx - 1 - ii, ry - 1 - jj, rz - 1 - kk)
                    } else {
                        (ii, jj, kk)
                    };
                    let n = idx(i, j, k);
                    let mut best = mag[n];
                    if best == 0.0 {
                        continue;
                    }
                    for (off, w) in order {
                        let (mut o0, mut o1, mut o2) = (off[0], off[1], off[2]);
                        if reverse {
                            o0 = -o0;
                            o1 = -o1;
                            o2 = -o2;
                        }
                        let (Some(a), Some(b), Some(c)) = (
                            i.checked_add_signed(o0),
                            j.checked_add_signed(o1),
                            k.checked_add_signed(o2),
                        ) else {
                            continue;
                        };
                        if a >= rx || b >= ry || c >= rz {
                            continue;
                        }
                        let m = idx(a, b, c);
                        let cand = w.mul_add(step, mag[m]);
                        if cand < best {
                            best = cand;
                        }
                    }
                    mag[n] = best;
                }
            }
        }
    };
    sweep(&FORWARD, false);
    sweep(&FORWARD, true);

    mag.iter()
        .zip(&inside)
        .map(|(m, inside)| {
            // ⚠️ O chanfro é um MAJORANTE — dividir é o que o torna utilizável por uma marcha.
            let d = if m.is_finite() {
                m / CHAMFER_SAFETY
            } else {
                *m
            };
            if *inside { -d } else { d }
        })
        .collect()
}

/// A distância de `p` à caixa: `0` dentro, positiva fora.
fn distance_to_box(b: Aabb, p: [f32; 3]) -> f32 {
    let mut s = 0.0f32;
    for ((lo, hi), c) in b.min.iter().zip(&b.max).zip(&p) {
        let d = (lo - c).max(c - hi).max(0.0);
        s = d.mul_add(d, s);
    }
    s.sqrt()
}

#[cfg(test)]
mod tests;

/// ⭐ **A ponte fecha aqui**: um campo amostrado é uma folha que o avaliador híbrido sabe consumir.
///
/// ⚠️ **A implementação é de UMA linha e é o ponto inteiro da crate.** O `ph2d-field-eval` declara o
/// trait e não sabe o que é uma malha; esta crate sabe as duas coisas e não é conhecida por nenhuma
/// das duas. Apagar a escultura apaga esta crate, e mais nada.
impl ph2d_field_eval::hybrid::Sampled for SampledField {
    fn at(&self, p: [f32; 3]) -> f32 {
        SampledField::at(self, p)
    }
}
