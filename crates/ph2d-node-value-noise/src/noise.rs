//! **O CATÁLOGO de kernels** de `value.noise` — o que o campo DESENHA — mais a
//! costura com a lei fractal ([`ph2d_fbm`]).
//!
//! ## Kernel e LEI são duas perguntas, e este repo já gastou a palavra `type`
//!
//! O `motion.noise` chama de **Type** a *retificação por oitava* (fBm ·
//! Turbulence · Ridged) — a **LEI**, que mora na folha [`ph2d_fbm`]. O que a
//! folha 15 do doc 89 pede aqui é a outra pergunta: **que ruído de base** a lei
//! soma (Value · Perlin · Cellular). São ortogonais e toda referência que tem as
//! duas as separa — Cavalry (`Type` + `Octaves/Lacunarity/Gain`), VFX Graph
//! (`Noise Type` + os knobs de fractal), Houdini (`Noise Type` + `Turbulence`).
//! Chamar as duas de `type` faria a mesma palavra selecionar coisas diferentes
//! em dois nós irmãos, então o param daqui é **`kernel`** e o rótulo **Pattern**.
//!
//! ## Por que o ruído de base fica LOCAL e a lei vem de uma folha
//!
//! É a decisão que o próprio [`ph2d_fbm`] documenta e que esta wave honra ao pé
//! da letra: a **lei** tinha cópias divergentes e virou folha; o **ruído de
//! base** é espelhado de propósito por crate-nó (o `hash2` daqui é o mesmo do
//! `motion.wiggle`, do `force.curl` e do `force.wind` — cinco espelhos, e o
//! vocabulário partilhado é o *comportamento*, não um símbolo). Uma crate-nó
//! não pode depender de outra crate-nó (drop-crate, ADR-0075), então a escolha
//! real é *espelhar* ou *criar uma folha de catálogo* — e uma folha cujo único
//! consumidor é este nó seria arquitetura à frente do uso.
//!
//! **Determinístico e livre de transcendentais** (HR-5): inteiros, `floor`,
//! polinómios, e `sqrt` — que **não** é transcendental (o IEEE-754 exige-o
//! correctamente arredondado, é instrução de hardware, e o repo já o usa no
//! caminho determinista do `blob_ball` e da física).

use ph2d_fbm::{NoiseType, Spec};

/// Hard ceiling on octaves — an untrusted `f32` param drives the fBm loop count.
/// 8 is past the point of visible return (each octave halves the feature size);
/// the WGSL loop guards on the same bound.
pub(crate) const MAX_OCTAVES: u32 = 8;

/// **Que ruído de base a lei soma** — o que o artista vê mudar de DESENHO.
///
/// O índice é o que o param `f32` carrega (o `Enum` do painel guarda o índice);
/// um índice que não existe cai no [`Kernel::Value`], **que é o que este nó
/// sempre shipou** — nunca num pânico e nunca noutro desenho.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Kernel {
    /// Hash na grade + fade + bilinear. Manchas macias; o mais barato, e o
    /// **default byte-idêntico** ao mundo anterior a esta wave.
    #[default]
    Value,
    /// Gradiente (Perlin 2002). O campo é exactamente zero em cada ponto da
    /// grade, então os extremos caem ENTRE eles: **sem padrão de grade**, fluxo
    /// mais orgânico. É o mesmo kernel que o `motion.noise` usa — o que esta
    /// wave acrescenta é ele estar disponível no domínio de **VALOR**, onde
    /// nenhuma composição o alcançava (o `motion.noise` escreve um canal de
    /// transform, nunca um socket de valor).
    Perlin,
    /// Worley/celular: a distância ao ponto-semente mais próximo. **Outro
    /// desenho, não outra intensidade** — células, bolhas, rachaduras.
    Cellular,
}

impl Kernel {
    #[must_use]
    pub(crate) fn from_index(i: f32) -> Self {
        match i.round() as i32 {
            1 => Self::Perlin,
            2 => Self::Cellular,
            _ => Self::Value,
        }
    }
}

/// **Que número o celular devolve** — a metade da família Worley que a distância
/// ao vizinho mais próximo não dá.
///
/// Só é lido com [`Kernel::Cellular`]; o `ParamGate` esconde-o nos outros dois
/// (um controlo que não faz nada é pior que um controlo que falta).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum CellFeature {
    /// `F1` — a distância ao ponto mais próximo. Baixo no miolo de cada célula e
    /// alto longe dele: **bolhas**.
    #[default]
    Cells,
    /// `F2 − F1` — zero exactamente na fronteira entre duas células e a crescer
    /// para dentro: **rachaduras**. É a outra metade do que se usa Worley para
    /// fazer, e sai de duas linhas do mesmo laço.
    Cracks,
}

impl CellFeature {
    #[must_use]
    pub(crate) fn from_index(i: f32) -> Self {
        match i.round() as i32 {
            1 => Self::Cracks,
            _ => Self::Cells,
        }
    }
}

/// A well-mixed integer hash of a lattice cell. Byte-identical to the mix
/// `motion.wiggle` ships (and to its WGSL twin `vn_hash2`) — extraído do
/// [`hash2`] para o celular poder usar os BITS em vez do escalar, o que é
/// **pure code motion** (o [`hash2`] passou a chamá-lo, sem mudar uma operação).
fn hash_bits(ix: i32, iy: i32) -> u32 {
    let mut h = (ix as u32)
        .wrapping_mul(0x27d4_eb2d)
        .wrapping_add((iy as u32).wrapping_mul(0x1656_67b1));
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x2971_75f9);
    h ^= h >> 15;
    h
}

/// A hash de célula → um pseudo-aleatório `f32 ∈ [-1, 1)`. Células vizinhas
/// saem descorrelacionadas (sem padrão de grade visível).
fn hash2(ix: i32, iy: i32) -> f32 {
    (hash_bits(ix, iy) as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// Smootherstep fade `6t⁵−15t⁴+10t³` (Perlin) on `t ∈ [0,1]` — C² continuous, so
/// the noise has no lattice-crossing creases.
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Smooth 2D value noise at `(x, y)`, bilinearly interpolating the four lattice
/// corner hashes with a smootherstep fade. Range `[-1, 1]`.
fn value_noise_2d(x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (u, v) = (fade(x - x0), fade(y - y0));
    let n00 = hash2(ix, iy);
    let n10 = hash2(ix + 1, iy);
    let n01 = hash2(ix, iy + 1);
    let n11 = hash2(ix + 1, iy + 1);
    let nx0 = n00 + u * (n10 - n00);
    let nx1 = n01 + u * (n11 - n01);
    nx0 + v * (nx1 - nx0)
}

/// O produto interno do gradiente da quina com o vector-distância `(dx, dy)`.
///
/// Os oito gradientes 2D de 2002 são `(±1, ±2)` e `(±2, ±1)` — a apontar para os
/// meios das arestas de um quadrado, logo **nenhum se alinha com um eixo** (que
/// era a fonte do viés direccional de 1985). Escolhidos por três bits da hash e
/// escritos como `±u ± 2v` (o truque do próprio Perlin).
fn dot_grad(h: u32, dx: f32, dy: f32) -> f32 {
    let g = h & 7;
    let (u, v) = if g < 4 { (dx, dy) } else { (dy, dx) };
    let a = if g & 1 != 0 { -u } else { u };
    let b = if g & 2 != 0 { -2.0 * v } else { 2.0 * v };
    a + b
}

/// Normalização para o gradiente cru cair perto de `[-1, 1]`.
///
/// ⚠️ **O número aqui é MEDIDO, e a medição derrubou o que estava escrito.** O
/// `motion.noise` — que shipa este mesmo kernel — afirma no doc dele que *"o pico
/// empírico desta construção é ~1,49, então `1/1.5` mapeia-o com folga"*. Varrido
/// por CÉLULA (180×180 células × sub-grid 24², a sonda `measure_the_true_perlin_peak`)
/// o pico cru é **1,5111** ⇒ com `1/1.5` a saída chega a **1,0074**: 0,74% FORA.
///
/// | kernel | pico medido |
/// |---|---|
/// | Value | 0,9999 (convexo por construção) |
/// | Perlin | **1,0074** |
/// | Cellular | 1,0000 (clampado por construção) |
///
/// A constante **fica em `1/1.5`** de propósito: apertá-la faria o Perlin deste
/// nó divergir 1,3% do Perlin do `motion.noise`, e *"é o mesmo campo"* vale mais
/// que 0,74% de folga. O que muda é a **afirmação**: o alcance é `[-1, 1]` com
/// 1% de folga, e o gate afirma isso em vez de afirmar o que é falso.
const PERLIN_NORM: f32 = 1.0 / 1.5;

/// A folga que o [`PERLIN_NORM`] deixa por cima de `1.0` — medida, não escolhida.
/// É a **barra do gate**, e nada em produção a lê: uma const `pub(crate)` sem
/// leitor seria só um número à espera de divergir do que a sonda mede.
#[cfg(test)]
pub(crate) const PERLIN_PEAK: f32 = 1.008;

/// Perlin **gradient** noise de uma oitava em `(x, y)`. Alcance `[-1, 1]`.
///
/// ⚠️ O `seed` deste nó entra na COORDENADA (o eixo `y`), não na hash — é a
/// convenção que o nó já tinha, e é ela que mantém `seed` a significar uma coisa
/// só nos três kernels. (No `force.curl` o seed e o tempo partilham o eixo `x` e
/// por isso colidem; aqui o tempo anda em `x` e o seed em `y`, logo não.)
fn perlin_2d(x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (fx, fy) = (x - x0, y - y0);
    let (u, v) = (fade(fx), fade(fy));
    let n00 = dot_grad(hash_bits(ix, iy), fx, fy);
    let n10 = dot_grad(hash_bits(ix + 1, iy), fx - 1.0, fy);
    let n01 = dot_grad(hash_bits(ix, iy + 1), fx, fy - 1.0);
    let n11 = dot_grad(hash_bits(ix + 1, iy + 1), fx - 1.0, fy - 1.0);
    let nx0 = n00 + u * (n10 - n00);
    let nx1 = n01 + u * (n11 - n01);
    (nx0 + v * (nx1 - nx0)) * PERLIN_NORM
}

/// A régua que mapeia o celular CRU para a faixa `[-1, 1]` dos irmãos.
///
/// ⚠️ **O número é MEDIDO** (sonda `measure_the_raw_cellular_distribution`, 360k
/// amostras por linha) e o achado é que **as duas features alcançam 1,0**, então
/// uma régua por-feature seria um segundo número sem trabalho a fazer:
///
/// | feature | jitter | cru min | cru max | média |
/// |---|---|---|---|---|
/// | F1 | 0,0 | 0,0000 | 0,7071 | 0,3834 |
/// | F1 | 0,5 | 0,0006 | 0,9047 | 0,3983 |
/// | F1 | 1,0 | 0,0007 | **1,0000** | 0,4342 |
/// | F2−F1 | 0,0 | 0,0000 | **1,0000** | 0,3156 |
/// | F2−F1 | 1,0 | 0,0000 | **1,0000** | 0,2659 |
///
/// ⚠️ **A MÉDIA baixa (0,27–0,43) é a natureza do Worley, não um defeito de
/// escala:** quase toda a área de uma célula está a meia distância da semente, e
/// o brilho é esparso por construção. Quem dirige um canal UNIPOLAR (tamanho,
/// opacidade) com este campo põe `amplitude 0.5 / offset 0.5` — a cena `=37`
/// fá-lo, e a primeira versão dela, que não fazia, saía com tamanhos NEGATIVOS.
const CELL_SPAN: f32 = 1.0;

/// **Worley/celular** em `(x, y)`. Cada célula da grade tem um ponto-semente; o
/// valor é a distância ao mais próximo (`Cells`) ou a folga até ao segundo
/// (`Cracks`), mapeada para `[-1, 1]` como os irmãos.
///
/// `jitter` posiciona a semente: `0` põe-na no centro exacto (uma grade
/// regular — favos), `1` em qualquer ponto da célula (o Worley clássico). É a
/// semântica `Randomness` do Blender.
///
/// ⚠️ **A busca é 3×3 e a aproximação está NOMEADA:** com `jitter ≤ 1` a semente
/// nunca sai da própria célula, então o vizinho mais próximo de um ponto está
/// sempre no 3×3 — para `F1` é exacto. Para `F2` existe um canto degenerado em
/// que o segundo mais próximo cai no anel seguinte; Blender e VFX Graph fazem o
/// mesmo 3×3 pela mesma razão (o erro vive abaixo do que uma célula desenha).
fn cellular_2d(x: f32, y: f32, feature: CellFeature, jitter: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (fx, fy) = (x - x0, y - y0);
    let j = jitter.clamp(0.0, 1.0);
    // Distâncias ao QUADRADO no laço (a raiz é uma só, no fim) — a ordem é a
    // mesma, então o `min` não muda de vencedor.
    let (mut d1, mut d2) = (f32::INFINITY, f32::INFINITY);
    for oy in -1..=1 {
        for ox in -1..=1 {
            let bits = hash_bits(ix + ox, iy + oy);
            // Dois aleatórios independentes de UMA hash: as metades alta e baixa
            // de um `u32` já avalanchado (16 bits = 1/65535 de resolução, muito
            // além do que um jitter desenha).
            let rx = (bits & 0xffff) as f32 * (1.0 / 65535.0);
            let ry = (bits >> 16) as f32 * (1.0 / 65535.0);
            let px = ox as f32 + 0.5 + (rx - 0.5) * j;
            let py = oy as f32 + 0.5 + (ry - 0.5) * j;
            let (dx, dy) = (px - fx, py - fy);
            let d = dx * dx + dy * dy;
            if d < d1 {
                d2 = d1;
                d1 = d;
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    let f1 = d1.sqrt();
    let raw = match feature {
        CellFeature::Cells => f1,
        CellFeature::Cracks => d2.sqrt() - f1,
    };
    // A mesma convenção dos irmãos: baixo perto da feição, alto longe dela. Quem
    // quer o inverso (cristas claras) põe `amplitude` negativa — a porta já existe.
    (raw / CELL_SPAN).clamp(0.0, 1.0) * 2.0 - 1.0
}

/// **Uma oitava do kernel escolhido** — a porta única que a lei fractal soma.
///
/// Todo estado que um kernel precisa (a feição e o jitter do celular) entra
/// aqui, e não na lei: [`ph2d_fbm`] não tem — e não deve ter — opinião sobre
/// qual ruído está a somar.
pub(crate) fn base(kernel: Kernel, feature: CellFeature, jitter: f32, x: f32, y: f32) -> f32 {
    match kernel {
        Kernel::Value => value_noise_2d(x, y),
        Kernel::Perlin => perlin_2d(x, y),
        Kernel::Cellular => cellular_2d(x, y, feature, jitter),
    }
}

/// A soma fractal deste nó: `octaves` camadas do kernel escolhido, cada uma a
/// `lacunarity×` a frequência e a `roughness×` a amplitude da anterior,
/// **normalizada pela soma dos pesos** (o alcance não cresce quando se
/// acrescenta detalhe).
///
/// ⚠️ **A LEI vem da folha [`ph2d_fbm`] e o CLAMP das oitavas fica aqui.** A
/// folha faz `octaves.max(1)` e **não** tem tecto: um param `f32` não confiável
/// dirigiria o laço. O tecto é deste nó (é ele que tem o gémeo em WGSL com o
/// mesmo `8` cravado), e é por isso que a adopção da folha não pode ser uma
/// chamada nua.
///
/// ⚠️ **A `lacunarity` era a const `2.0` deste arquivo e virou param** (doc 89
/// folha 15). A troca é **byte-idêntica em `2.0`** por aritmética e não por
/// promessa: a folha já a recebia pelo `Spec`, então o que muda é de ONDE o
/// número vem, nunca a multiplicação — e a folha documenta que escalar por uma
/// potência de dois não arredonda. ⚠️ E ela é **provadamente INERTE em uma
/// oitava**: o `px *= lacunarity` do laço acontece DEPOIS da única amostra.
pub(crate) fn fbm_2d(
    x: f32,
    y: f32,
    octaves: u32,
    lacunarity: f32,
    roughness: f32,
    base_fn: impl Fn(f32, f32) -> f32,
) -> f32 {
    let spec = Spec {
        octaves: octaves.clamp(1, MAX_OCTAVES),
        lacunarity,
        roughness,
        ty: NoiseType::Fbm,
    };
    ph2d_fbm::eval(spec, x, y, |px, py, _o| base_fn(px, py))
}

#[cfg(test)]
#[path = "noise_tests.rs"]
mod tests;
