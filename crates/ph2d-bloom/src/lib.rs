//! ⭐⭐⭐ **O BRILHO** — a luz forte que vaza para fora da própria forma.
//!
//! O ingrediente **`7`** do [`docs/Render3d/01`], a `W7`. Ele lê o quadro em **HDR**, tira o que
//! passa de um limiar, borra isso numa cadeia de resoluções e soma de volta.
//!
//! # ⛔⛔ Porque ele é uma crate PRÓPRIA e não mais um botão da [`ph2d_style`]
//!
//! Aquela é a lei **por PIXEL** que os dois motores partilham. Esta lê os **VIZINHOS** ⇒ é um
//! **passe**, e pô-la lá obrigaria a crate da lei a receber um G-buffer. A recusa está escrita, com
//! este mecanismo, no `docs/Render3d/11` §11 — *para o contorno desenhado, três dias antes de este
//! ficheiro existir*.
//!
//! # ⭐ A GEOMETRIA é portada; o CORTE é NOSSO — e as duas metades foram medidas
//!
//! O oráculo é o **Godot 4.7.2 (MIT**, lido no artefacto — porta aberta, porta-se com atribuição),
//! corrido sem interface. Medido em `15` corridas (`docs/Render3d/12` §3):
//!
//! - ⭐⭐ **A cadeia é um *mip chain*** e o raio a meia altura **DUPLICA por nível**
//!   (`3 · 9 · 20 · 38 · 55` px num quadro de `512`). Isso porta-se, e é o que este ficheiro faz.
//! - ⛔⛔ **O corte dele NÃO é uma lei**: o limiar move o halo `5 %` a `entrada 4,0` (a quantização
//!   de 8 bits) e `27 %` sobre a faixa `0..3` a `entrada 2,0`, a escala **não move nada** nas
//!   quatro células, e com limiar `3` e entrada `2` **ainda brilha**. *Portar isso seria herdar um
//!   painel em que «Threshold» não faz o que o nome diz.*
//!
//! ⇒ o corte abaixo é **declarado**, com o joelho a ser o controlo fino que o report do dono sobre
//! a camada de estilo pedia um nível acima, e com a **degenerescência gateada**: `knee = 0` é o
//! corte duro **exacto**.

/// ⭐⭐⭐ **Os botões do brilho.** `Default` é o **desligado**, e por isso a omissão é a identidade.
///
/// ⚠️ **A identidade é por CONSTRUÇÃO e não por um `if` de cortesia:** com [`Bloom::enabled`] falso
/// o [`apply`] devolve **sem tocar no quadro**, e com ele ligado mas `intensity = 0` a soma é
/// `+ 0.0`, que em `f32` é o mesmo bit para todo valor finito. *As duas metades têm gate.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bloom {
    /// O passe corre?
    pub enabled: bool,
    /// **Acima de quanto é que a luz brilha** — na luminância de PICO, em unidades da cena.
    ///
    /// ⚠️ **Pico e não média:** um vermelho `(4, 0, 0)` estoura tanto quanto um branco `(4, 4, 4)`,
    /// e uma média ponderada di-lo-ia `0,8` — *uma luz saturada deixaria de brilhar por ser
    /// saturada*.
    pub threshold: f32,
    /// **A largura do JOELHO**, nas mesmas unidades. `0` ⇒ corte duro **exacto** (com gate).
    ///
    /// ⭐ É este o controlo fino que falta ao alvo: ali o limiar mal gateia e não há como suavizar a
    /// passagem entre *«não brilha»* e *«brilha»*.
    pub knee: f32,
    /// Quanto do halo volta para a imagem. `0` ⇒ identidade ao bit.
    pub intensity: f32,
    /// **O peso de cada nível da cadeia** — o nível `k` tem o borrão `2^k` vezes mais largo.
    ///
    /// ⚠️ **Os valores de fábrica são os do ORÁCULO** (`0 · 0,8 · 0,4 · 0,1 · 0 · 0 · 0`) e não
    /// gosto meu: são a faixa que cabe num ecrã, medida (`docs/Render3d/12` §3.4).
    pub levels: [f32; Bloom::LEVELS],
}

impl Bloom {
    /// Quantos níveis a cadeia declara. ⚠️ **Quantos ela CORRE é derivado da resolução** —
    /// ver [`levels_that_fit`].
    pub const LEVELS: usize = 7;

    /// O limiar de fábrica, do oráculo.
    pub const THRESHOLD: f32 = 1.0;
    /// O joelho de fábrica. ⚠️ **Não é zero**: um corte duro é o que faz uma luz a subir devagar
    /// *aparecer de repente*, que é o defeito que o dono já reportou na tinta por curvatura.
    pub const KNEE: f32 = 0.5;
    /// A intensidade de fábrica, do oráculo.
    pub const INTENSITY: f32 = 0.3;
    /// Os pesos de fábrica, do oráculo (§3.2).
    pub const NIVEIS: [f32; Self::LEVELS] = [0.0, 0.8, 0.4, 0.1, 0.0, 0.0, 0.0];

    /// ⚠️ **O passe só corre se houver o que somar** — e as duas condições são separadas de
    /// propósito: um `intensity = 0` com o interruptor ligado é *«ligado e mudo»*, que é um estado
    /// que o artista alcança e que não deve custar um borrão.
    #[must_use]
    pub fn contributes(&self) -> bool {
        self.enabled && self.intensity > 0.0 && self.levels.iter().any(|w| *w > 0.0)
    }
}

impl Default for Bloom {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: Self::THRESHOLD,
            knee: Self::KNEE,
            intensity: Self::INTENSITY,
            levels: Self::NIVEIS,
        }
    }
}

/// ⭐⭐ **QUANTOS NÍVEIS CABEM NUM QUADRO DESTE TAMANHO** — derivado, nunca fixo.
///
/// # ⛔ Porque não são sempre os sete
///
/// Medido no oráculo (`docs/Render3d/12` §3.4): num quadro de `512` px os níveis `5`, `6` e `7`
/// têm o borrão **maior que a moldura**, e a medição do raio deles lê a moldura em vez do nível.
/// *Um nível cujo borrão não cabe no quadro não acrescenta forma: acrescenta uma constante.*
///
/// A cadeia pára quando o lado menor deixa de ter pelo menos [`LADO_MINIMO`] píxeis.
#[must_use]
pub fn levels_that_fit(w: usize, h: usize) -> usize {
    let mut lado = w.min(h);
    let mut n = 0;
    while n < Bloom::LEVELS && lado >= LADO_MINIMO * 2 {
        lado /= 2;
        n += 1;
    }
    n
}

/// O lado mais pequeno que um nível da cadeia pode ter.
///
/// ⚠️ **`4` e não `1`:** abaixo disto o borrão de um nível é a **média do quadro inteiro**, que é
/// uma constante aditiva e não uma forma — e ela apareceria como um véu cinzento que sobe com o
/// brilho da cena. É o mesmo mecanismo que faz a fábrica do alvo acender só três dos sete níveis.
pub const LADO_MINIMO: usize = 4;

/// ⭐⭐⭐ **O CORTE — quanto de um pixel entra na cadeia.**
///
/// A luminância de pico decide, e o resultado é uma **fracção** que multiplica a cor: assim a
/// MATIZ do que brilha é a do pixel, e não a do canal que passou.
///
/// ```text
/// l  = max(r, g, b)
/// s  = clamp(l − t + k, 0, 2k)         a rampa suave, centrada no limiar
/// b  = max(s² / (4k), l − t)           o joelho por baixo, o corte duro por cima
/// w  = max(b, 0) / max(l, ε)           a fracção
/// ```
///
/// ⭐ **Com `k = 0` o primeiro ramo desaparece** (`s ≡ 0`) e sobra `max(l − t, 0)`, que é o corte
/// duro **exacto** — há gate sobre isso, e é ele que torna o joelho um *controlo* em vez de uma
/// aproximação permanente.
#[must_use]
pub fn bright(rgb: [f32; 3], b: &Bloom) -> [f32; 3] {
    let l = rgb[0].max(rgb[1]).max(rgb[2]);
    if l.is_nan() || l <= 0.0 {
        return [0.0; 3];
    }
    let k = b.knee.max(0.0);
    let duro = l - b.threshold;
    let acima = if k > 0.0 {
        let s = (duro + k).clamp(0.0, 2.0 * k);
        (s * s / (4.0 * k)).max(duro)
    } else {
        duro
    };
    if acima.is_nan() || acima <= 0.0 {
        return [0.0; 3];
    }
    let w = acima / l;
    [rgb[0] * w, rgb[1] * w, rgb[2] * w]
}

/// ⭐⭐⭐ **O PASSE: soma o halo ao quadro, no sítio.**
///
/// `hdr` é **linear de CENA** — antes da exposição e do tonemapper. ⚠️ *Ler depois deles é o que o
/// `docs/Render3d/01` §2 chama de «sem o `1`, o bloom mente»*: o tonemapper comprime justamente o
/// que havia para colher.
///
/// ⚠️ **Devolve cedo quando não há o que somar** — ver [`Bloom::contributes`] —, e é isso que faz o
/// caminho de omissão custar **zero**.
pub fn apply(hdr: &mut [[f32; 3]], w: usize, h: usize, b: &Bloom) {
    if !b.contributes() || w == 0 || h == 0 || hdr.len() < w * h {
        return;
    }
    let n = levels_that_fit(w, h);
    if n == 0 {
        return;
    }

    // (1) o corte, na resolução cheia.
    let mut nivel: Vec<[f32; 3]> = hdr[..w * h].iter().map(|c| bright(*c, b)).collect();
    let (mut lw, mut lh) = (w, h);

    // (2) desce a cadeia, guardando cada degrau — ⚠️ o peso é aplicado na SUBIDA, para que um nível
    // com peso zero continue a alimentar o seguinte. *Um nível mudo não é um nível ausente.*
    let mut degraus: Vec<(Vec<[f32; 3]>, usize, usize)> = Vec::with_capacity(n);
    for _ in 0..n {
        let (dw, dh) = (lw / 2, lh / 2);
        nivel = downsample(&nivel, lw, lh);
        lw = dw;
        lh = dh;
        // ⛔⛔ **REDUZIR NÃO É BORRAR, e o gate apanhou-o na 1.ª corrida:** com só a média de `2×2`
        // e a subida bilinear, os níveis `0` e `1` liam meia-largura **`1` px** (o oráculo lê `3` e
        // `9`) e um quadrado de `8` px **desaparecia** na cadeia em vez de se espalhar. O borrão é o
        // tento, **na resolução do próprio nível** — é isso que o faz dobrar em píxeis de ecrã.
        nivel = tento(&nivel, lw, lh);
        degraus.push((nivel.clone(), lw, lh));
    }

    // (3) a soma dos níveis, cada um reamostrado de volta ao tamanho do quadro.
    let mut halo = vec![[0.0f32; 3]; w * h];
    for (k, (buf, bw, bh)) in degraus.iter().enumerate() {
        let peso = b.levels[k];
        if peso.is_nan() || peso <= 0.0 {
            continue;
        }
        acumula_ampliado(&mut halo, w, h, buf, *bw, *bh, peso);
    }

    // (4) de volta à imagem.
    for (px, add) in hdr[..w * h].iter_mut().zip(halo) {
        for c in 0..3 {
            px[c] += add[c] * b.intensity;
        }
    }
}

/// ⭐⭐ **O TENTO `(1,2,1)` separável, na resolução do nível.**
///
/// ⚠️ **É ele, e não a redução, que faz o halo ter LARGURA** — ver o comentário no [`apply`], que é
/// onde o gate `o_raio_do_halo_dobra_por_nivel` reprovou a 1.ª redacção desta crate.
fn tento(src: &[[f32; 3]], w: usize, h: usize) -> Vec<[f32; 3]> {
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let em = |x: isize, lim: usize| -> usize {
        #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
        let v = x.clamp(0, lim as isize - 1) as usize;
        v
    };
    let mut mid = vec![[0.0f32; 3]; w * h];
    for y in 0..h {
        for x in 0..w {
            #[allow(clippy::cast_possible_wrap)]
            let xi = x as isize;
            let (a, b, c) = (
                src[y * w + em(xi - 1, w)],
                src[y * w + x],
                src[y * w + em(xi + 1, w)],
            );
            for k in 0..3 {
                mid[y * w + x][k] = a[k].mul_add(0.25, b[k].mul_add(0.5, c[k] * 0.25));
            }
        }
    }
    let mut out = vec![[0.0f32; 3]; w * h];
    for y in 0..h {
        #[allow(clippy::cast_possible_wrap)]
        let yi = y as isize;
        for x in 0..w {
            let (a, b, c) = (
                mid[em(yi - 1, h) * w + x],
                mid[y * w + x],
                mid[em(yi + 1, h) * w + x],
            );
            for k in 0..3 {
                out[y * w + x][k] = a[k].mul_add(0.25, b[k].mul_add(0.5, c[k] * 0.25));
            }
        }
    }
    out
}

/// Metade da resolução, pela média de `2×2`.
///
/// ⚠️ **A média de quatro é o que faz o raio DOBRAR por nível** (§3.4) — um subamostrador que
/// escolhesse um dos quatro daria uma cadeia de aliasing, não de borrão.
fn downsample(src: &[[f32; 3]], w: usize, h: usize) -> Vec<[f32; 3]> {
    let (dw, dh) = (w / 2, h / 2);
    let mut out = vec![[0.0f32; 3]; dw * dh];
    for y in 0..dh {
        for x in 0..dw {
            let mut acc = [0.0f32; 3];
            for (dy, dx) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                let s = src[(y * 2 + dy) * w + (x * 2 + dx)];
                for c in 0..3 {
                    acc[c] += s[c];
                }
            }
            out[y * dw + x] = [acc[0] * 0.25, acc[1] * 0.25, acc[2] * 0.25];
        }
    }
    out
}

/// Soma `src` ampliado ao tamanho de `dst`, com peso, por interpolação bilinear.
fn acumula_ampliado(
    dst: &mut [[f32; 3]],
    w: usize,
    h: usize,
    src: &[[f32; 3]],
    sw: usize,
    sh: usize,
    peso: f32,
) {
    if sw == 0 || sh == 0 {
        return;
    }
    #[allow(clippy::cast_precision_loss)]
    let (fx, fy) = (sw as f32 / w as f32, sh as f32 / h as f32);
    for y in 0..h {
        for x in 0..w {
            #[allow(clippy::cast_precision_loss)]
            let (u, v) = ((x as f32 + 0.5) * fx - 0.5, (y as f32 + 0.5) * fy - 0.5);
            let s = amostra_bilinear(src, sw, sh, u, v);
            let d = &mut dst[y * w + x];
            for c in 0..3 {
                d[c] += s[c] * peso;
            }
        }
    }
}

fn amostra_bilinear(src: &[[f32; 3]], sw: usize, sh: usize, u: f32, v: f32) -> [f32; 3] {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let (x0, y0) = (u.floor(), v.floor());
    let (tx, ty) = (u - x0, v - y0);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let ix = |f: f32, lim: usize| -> usize { (f.max(0.0) as usize).min(lim - 1) };
    let (xa, xb) = (ix(x0, sw), ix(x0 + 1.0, sw));
    let (ya, yb) = (ix(y0, sh), ix(y0 + 1.0, sh));
    let mut out = [0.0f32; 3];
    for c in 0..3 {
        let a = src[ya * sw + xa][c] * (1.0 - tx) + src[ya * sw + xb][c] * tx;
        let b = src[yb * sw + xa][c] * (1.0 - tx) + src[yb * sw + xb][c] * tx;
        out[c] = a * (1.0 - ty) + b * ty;
    }
    out
}

#[cfg(test)]
#[path = "lei_tests.rs"]
mod lei_tests;
