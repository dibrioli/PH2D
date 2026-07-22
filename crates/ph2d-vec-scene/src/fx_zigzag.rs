//! **Zig Zag / Roughen** — o segundo efeito da pilha (ADR-0132), e a prova da promessa dela.
//!
//! O caminho é reamostrado em cristas igualmente espaçadas **por comprimento de arco** e cada
//! amostra é deslocada pela NORMAL, alternando o lado. É o *Zig Zag* do Illustrator e o path
//! operator homónimo do After Effects.
//!
//! # Espaçamento por ARCO, não por segmento — e isto não é preciosismo
//!
//! O Illustrator conta *"cristas por segmento"*, e por isso a mesma forma desenhada com mais
//! âncoras ganha mais cristas: o efeito passa a descrever **como o caminho foi autorado**, não
//! a forma que se vê. É a mesma doença que o `ph2d-vec-blend` pagou caro para curar (*"duas
//! formas que se veem iguais têm de blendar igual"*), e o motor de arco que o Trim trouxe já a
//! resolve. Aqui as cristas são contadas **no caminho inteiro**: picar uma aresta em vinte
//! pedaços não muda um pixel.
//!
//! # Zig Zag e Roughen são o MESMO motor
//!
//! O Illustrator os vende separados; a diferença é só de onde vem o deslocamento — alternado e
//! regular (*Zig Zag*) ou pseudo-aleatório (*Roughen*). Um `bool` decide, e o gerador é o
//! `splitmix64` que o Painter já usa para o jitter dos dabs: determinístico a partir de uma
//! `seed`, então o mesmo documento desenha igual em qualquer máquina (HR-5 — nada de
//! transcendental nem de `rand`).
//!
//! # ⚠️ As âncoras de ENTRADA entram no conjunto de amostras (Enio, 2026-07-18)
//!
//! A 1ª versão **descartava** a entrada e reamostrava em `2·ridges` posições. Sobre um caminho
//! liso isso é exato; sobre um caminho que **já tem uma onda**, é amostrar mais grosso do que a
//! onda que lá está — **aliasing**. Medido: um zigzag de 16 cristas seguido de outro de 4
//! deixava **7 âncoras e comprimento 310 num círculo de 339** — o 2º efeito não *compunha* mal
//! com o 1º, **apagava-o e encolhia a forma abaixo do original**.
//!
//! O Illustrator e o After Effects não têm este problema porque contam cristas **por segmento**
//! — e é exatamente por isso que empilhar lá produz o fractal auto-semelhante que se espera.
//! Nós contamos por arco (§ acima), e o preço dessa escolha era este.
//!
//! A cura é a **UNIÃO**: amostra-se na grade uniforme das cristas **e** nas posições de arco das
//! âncoras que chegaram. Um pico do 1º zigzag *é* uma âncora de entrada, logo sobrevive
//! exatamente — a onda fina passa a cavalgar a grossa. É a mesma regra que o `ph2d-vec-blend`
//! usa para parear duas formas sem arredondar as quinas de nenhuma.
//!
//! Consequência aceite: um caminho com mais âncoras produz mais vértices (as extras caem **sobre
//! a curva deslocada**, aproximando-a melhor). As **cristas** — o que se vê — continuam a ser
//! função só da geometria, e é isso que o gate afirma.

use crate::arc_path::ArcPath;
use crate::{VecVertex, VertexKind};

/// Abaixo desta amplitude (em unidades de mundo) o efeito é o ponto neutro.
const EPS: f64 = 1e-12;

/// O menor número de cristas que produz geometria. Com menos de uma o efeito não tem o que
/// alternar, e o caminho volta intacto.
const MIN_RIDGES: f64 = 1.0;

/// Teto de amostras por contorno — a guarda contra um `ridges` absurdo vindo de um save
/// corrompido virar uma alocação de gigabytes. **Não é o teto do artista**: o slider do painel
/// para muito antes, e este número só existe para que o pior caso seja feio em vez de fatal.
const MAX_SAMPLES: usize = 4096;

/// Duas amostras mais próximas do que esta fração do caminho são **o mesmo sítio**, e a segunda
/// é descartada. Sem isto, uma âncora que calha em cima de um ponto da grade produziria um
/// segmento de comprimento ~0 — invisível, mas suficiente para envenenar tangentes a jusante.
const MERGE_FRACTION: f64 = 1e-6;

/// **Os parâmetros do Zig Zag.** Neutro em `amplitude == 0`.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ZigZagSpec {
    /// Quanto cada crista se afasta do caminho, em **PERCENTAGEM da forma**: `100` = a média
    /// entre a largura e a altura dela ([`crate::effect::FxCtx`]).
    ///
    /// ⚠️ A 1ª versão pôs isto em unidades de MUNDO e o doc argumentava que era o certo. Era o
    /// contrário: as formas da cena têm ~2-3 unidades, então o slider era inútil (Enio,
    /// 2026-07-18). Percentagem faz o mesmo número desenhar a mesma coisa em qualquer escala.
    pub amplitude: f64,
    /// Quantas cristas ao longo do caminho INTEIRO.
    pub ridges: f64,
    /// `true` = ondas suaves (as âncoras ganham alças ao longo da tangente); `false` = picos
    /// em quina, o *Corner* do Illustrator.
    pub smooth: bool,
    /// `Some(seed)` = **Roughen**: o deslocamento vira pseudo-aleatório em vez de alternado.
    /// `None` = Zig Zag regular.
    pub rough_seed: Option<u64>,
}

impl Default for ZigZagSpec {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            ridges: 8.0,
            smooth: false,
            rough_seed: None,
        }
    }
}

impl ZigZagSpec {
    /// Sem amplitude não há onda — e o neutro tem de ser um no-op byte-idêntico, senão a pilha
    /// não pode saltá-lo e o `Cow::Borrowed` morre (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.amplitude.abs() <= EPS || self.ridges < MIN_RIDGES
    }
}

/// `splitmix64` — o mesmo gerador do jitter do Painter. Determinístico e sem transcendental.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Um número em `[-1, 1]` a partir do gerador.
fn signed_unit(state: &mut u64) -> f64 {
    // 53 bits de mantissa: a divisão é exata e o resultado cobre `[0, 1)` uniformemente.
    #[allow(clippy::cast_precision_loss)]
    let u = (splitmix64(state) >> 11) as f64 / (1u64 << 53) as f64;
    u.mul_add(2.0, -1.0)
}

/// **A onda triangular na posição de arco `s`**, em `[-1, 1]`.
///
/// A alternância tem de ser função de `s` e **não do índice da amostra**: assim que a lista de
/// amostras deixou de ser uniforme (a união com as âncoras de entrada), `k % 2` passou a
/// atribuir picos a posições arbitrárias. `+1` em `s = 0`, `-1` a meia crista, e como `ridges`
/// é inteiro a onda fecha exatamente em `s = total` — um contorno fechado não ganha emenda.
///
/// Álgebra pura, sem transcendental (HR-5).
fn ridge_wave(s: f64, total: f64, ridges: usize) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let u = s / total * (ridges as f64) * 2.0;
    let m = u.rem_euclid(2.0);
    if m <= 1.0 {
        2.0f64.mul_add(-m, 1.0)
    } else {
        2.0f64.mul_add(m, -3.0)
    }
}

/// **Aplica o Zig Zag a UM contorno.** Devolve `(verts, closed)`.
///
/// O contorno **mantém** o seu `closed`: ondular não abre nem fecha uma forma.
#[must_use]
pub fn zigzag_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &ZigZagSpec,
    ref_size: f64,
) -> (Vec<VecVertex>, bool) {
    let n = verts.len();
    // Sem referência de tamanho não há distância a construir — e é a mesma resposta do neutro.
    if n < 2 || spec.is_neutral() || ref_size <= EPS {
        return (verts.to_vec(), closed);
    }
    // `100` = a média das dimensões da forma. É aqui, e só aqui, que a percentagem vira mundo.
    let amplitude = spec.amplitude / 100.0 * ref_size;
    // O percurso por arco vem da porta única (`crate::arc_path`), partilhada com quem mais
    // precisar de "onde fica o arco `s` neste caminho" — o walker privado que vivia aqui era a
    // primeira das duas cópias que sempre acabam por divergir.
    let Some(ap) = ArcPath::from_contour(verts, closed) else {
        return (verts.to_vec(), closed);
    };
    let total = ap.total();
    if total <= EPS {
        return (verts.to_vec(), closed);
    }

    // Uma crista sobe e a seguinte desce: são DUAS amostras por crista.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ridges = spec.ridges.floor().max(MIN_RIDGES) as usize;
    let count = (ridges * 2).min(MAX_SAMPLES);
    #[allow(clippy::cast_precision_loss)]
    let step = total / count as f64;

    // A GRADE das cristas. Fechado dá a volta (`count` pontos); aberto tem as duas pontas como
    // amostras próprias, daí o `+ 1`.
    let grid_n = if closed { count } else { count + 1 };
    let mut at: Vec<f64> = Vec::with_capacity(grid_n + n + 1);
    for k in 0..grid_n {
        #[allow(clippy::cast_precision_loss)]
        at.push((k as f64 * step).min(total));
    }
    // ...e a UNIÃO com as âncoras que chegaram. É esta linha que faz o efeito COMPOR: um pico
    // do zigzag anterior é uma âncora, logo é amostrado onde está em vez de ser saltado.
    at.extend_from_slice(ap.anchor_arcs());
    if !closed {
        at.push(total);
    }
    at.sort_by(f64::total_cmp);
    let merge = total * MERGE_FRACTION;
    at.dedup_by(|a, b| (*a - *b).abs() <= merge);
    at.truncate(MAX_SAMPLES);
    let m = at.len();
    if m < 2 {
        return (verts.to_vec(), closed);
    }

    // O braço de uma alça é um terço do vão ATÉ o vizinho — na grade uniforme isto é `step/3`
    // (byte-idêntico ao que era), e numa lista com âncoras extras cada alça encolhe para o vão
    // que de facto tem. Numa ponta livre o vão espelha o do outro lado.
    let gap = |k: usize, forward: bool| -> f64 {
        let wrap = at[0] + total - at[m - 1];
        if forward {
            if k + 1 < m {
                at[k + 1] - at[k]
            } else if closed {
                wrap
            } else {
                at[k] - at[k - 1]
            }
        } else if k > 0 {
            at[k] - at[k - 1]
        } else if closed {
            wrap
        } else {
            at[1] - at[0]
        }
    };

    let mut state = spec.rough_seed.unwrap_or(0);
    let mut out = Vec::with_capacity(m);
    for (k, &s) in at.iter().enumerate() {
        let (point, tangent) = ap.frame_at(s);
        // A normal é a tangente rodada um quarto de volta — troca de eixo e sinal, sem
        // transcendental (HR-5). Numa cúspide não há direção: o ponto entra sem deslocamento,
        // em vez de ser DESCARTADO — descartar tirava uma crista da conta em silêncio.
        let normal = [-tangent[1], tangent[0]];
        let lift = if spec.rough_seed.is_some() {
            signed_unit(&mut state)
        } else {
            ridge_wave(s, total, ridges)
        } * amplitude;
        let anchor = [
            normal[0].mul_add(lift, point[0]),
            normal[1].mul_add(lift, point[1]),
        ];
        // Suave: as alças acompanham a TANGENTE do caminho original naquele ponto — é a
        // construção que faz uma sequência de amostras reproduzir uma curva lisa em vez de
        // uma poligonal arredondada a olho.
        let (back, fwd) = if spec.smooth {
            (gap(k, false) / 3.0, gap(k, true) / 3.0)
        } else {
            (0.0, 0.0)
        };
        out.push(VecVertex {
            anchor,
            in_handle: [
                tangent[0].mul_add(-back, anchor[0]),
                tangent[1].mul_add(-back, anchor[1]),
            ],
            out_handle: [
                tangent[0].mul_add(fwd, anchor[0]),
                tangent[1].mul_add(fwd, anchor[1]),
            ],
            kind: if spec.smooth {
                VertexKind::Smooth
            } else {
                VertexKind::Corner
            },
            corner_radius: 0.0,
        });
    }
    (out, closed)
}

#[cfg(test)]
#[path = "fx_zigzag_tests.rs"]
mod tests;
