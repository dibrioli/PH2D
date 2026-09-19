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

    /// ⭐⭐⭐ **QUANTOS NÚMEROS ESTE BRILHO TEM** — e o painel, a arrumação e o gémeo do dispositivo
    /// contam-nos **daqui**, nunca à mão.
    pub const SLOTS: usize = 4 + Self::LEVELS;

    /// ⭐⭐⭐ **A ARRUMAÇÃO, e há UMA** — `[ligado, limiar, joelho, intensidade, nível 0..6]`.
    ///
    /// ⚠️ **Ela nasce antes do gémeo em WGSL de propósito.** O `Param::Bloom(n)` do painel carrega
    /// esta posição, e o dia em que o dispositivo ganhar o passe ele lê o mesmo `n` — *uma segunda
    /// numeração seria a resposta que envelhece no dia em que nascer um botão*, que é a lei que o
    /// [`ph2d_style::wgsl::pack`] já escreve um módulo ao lado.
    ///
    /// ⚠️ **O interruptor viaja como número** (`0`/`1`): a arrumação de um uniforme não tem
    /// booleanos, e ter DUAS travessias — uma para o `bool` e outra para os `f32` — seria a segunda
    /// máquina ao lado de uma que funciona.
    #[must_use]
    pub fn pack(&self) -> [f32; Self::SLOTS] {
        let mut v = [0.0; Self::SLOTS];
        v[0] = f32::from(u8::from(self.enabled));
        v[1] = self.threshold;
        v[2] = self.knee;
        v[3] = self.intensity;
        v[4..].copy_from_slice(&self.levels);
        v
    }

    /// A volta do [`Bloom::pack`]. ⚠️ **Ida-e-volta com gate**, porque uma arrumação com um só
    /// sentido é meia arrumação.
    #[must_use]
    pub fn unpack(v: &[f32; Self::SLOTS]) -> Self {
        let mut levels = [0.0; Self::LEVELS];
        levels.copy_from_slice(&v[4..]);
        Self {
            // ⚠️ `> 0,5` e não `!= 0`: um arrasto entrega o meio do curso, e um `0,49` tem de
            // decidir para um lado. *Um interruptor sem ponto de viragem declarado é um que muda de
            // ideias no último bit.*
            enabled: v[0] > 0.5,
            threshold: v[1],
            knee: v[2],
            intensity: v[3],
            levels,
        }
    }

    /// ⭐⭐ **O SANEAMENTO é na PORTA**, como o do estilo — a partir daqui o número viaja para a
    /// thread que desenha, e ela não tem cerca.
    ///
    /// ⚠️ **O que não é número vira o valor de fábrica e não zero:** um `NaN` no limiar com `0` faria
    /// a cena inteira brilhar, e *uma recusa não pode ser mais destrutiva do que o pedido*.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let so_finito = |v: f32, fabrica: f32| if v.is_finite() { v.max(0.0) } else { fabrica };
        self.threshold = so_finito(self.threshold, Self::THRESHOLD);
        self.knee = so_finito(self.knee, Self::KNEE);
        self.intensity = so_finito(self.intensity, Self::INTENSITY);
        for (k, w) in self.levels.iter_mut().enumerate() {
            *w = so_finito(*w, Self::NIVEIS[k]);
        }
        self
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
    let add = halo(hdr, w, h, b);
    if add.is_empty() {
        return;
    }
    for (px, a) in hdr[..w * h].iter_mut().zip(add) {
        for c in 0..3 {
            px[c] += a[c];
        }
    }
}

/// ⭐⭐⭐ **O HALO SOZINHO** — o que o brilho ACRESCENTA, sem o somar.
///
/// # ⚠️ Porque a porta devolve o halo em vez de só o somar
///
/// Quem compõe pode não ter o quadro em cena-linear para somar: no modelador o **FUNDO** é uma cor
/// que o artista dá em BYTES e que nunca passou pelo olhar, logo não há como o converter de volta.
/// ⛔ E tratar peça e fundo por regras diferentes está **refutado por medição** neste módulo
/// (`docs/Render3d/10` §11.3): *um `if` por pixel desenha a fronteira entre os dois ramos*, e a
/// cura que o fez pintou um fio serrilhado na silhueta.
///
/// ⇒ o halo sai daqui em **cena-linear** — lido do HDR, que é o que o torna honesto — e quem compõe
/// aplica-lhe o olhar e soma-o **com a mesma regra em todo pixel**.
///
/// Devolve **vazio** quando não há o que somar, e é isso que faz a omissão custar zero.
#[must_use]
pub fn halo(hdr: &[[f32; 3]], w: usize, h: usize, b: &Bloom) -> Vec<[f32; 3]> {
    if !b.contributes() || w == 0 || h == 0 || hdr.len() < w * h {
        return Vec::new();
    }
    let n = levels_that_fit(w, h);
    if n == 0 {
        return Vec::new();
    }

    // (1) o corte, na resolução cheia.
    let corte: Vec<[f32; 3]> = hdr[..w * h].iter().map(|c| bright(*c, b)).collect();

    // (2) DESCE a cadeia com o filtro de 13 taps.
    let mut mips: Vec<(Vec<[f32; 3]>, usize, usize)> = Vec::with_capacity(n);
    let (mut lw, mut lh) = (w, h);
    let mut actual = corte;
    for _ in 0..n {
        let (dw, dh) = (lw / 2, lh / 2);
        actual = desce13(&actual, lw, lh);
        lw = dw;
        lh = dh;
        mips.push((actual.clone(), lw, lh));
    }

    // (3) SOBE um degrau de cada vez, somando. ⭐ É esta a metade que dá a qualidade: cada nível é
    // re-borrado por TODAS as tendas mais finas por onde passa, e é isso que dissolve os cantos.
    #[allow(clippy::cast_precision_loss)]
    let aspecto = w as f32 / h.max(1) as f32;
    let (raio_u, raio_v) = (RAIO_DA_TENDA, RAIO_DA_TENDA * aspecto);
    let (topo, tw, th) = mips[n - 1].clone();
    let mut acc: Vec<[f32; 3]> = topo
        .iter()
        .map(|c| c.map(|x| x * peso_do_nivel(b, n - 1)))
        .collect();
    let (mut aw, mut ah) = (tw, th);
    for k in (0..n - 1).rev() {
        let (mip, mw, mh) = &mips[k];
        let mut subido = sobe_tenda(&acc, aw, ah, *mw, *mh, raio_u, raio_v);
        let peso = peso_do_nivel(b, k);
        for (d, s) in subido.iter_mut().zip(mip) {
            for c in 0..3 {
                d[c] += s[c] * peso;
            }
        }
        acc = subido;
        aw = *mw;
        ah = *mh;
    }

    // (4) e o último degrau, de volta ao quadro — a intensidade é do halo, não da soma.
    let mut halo = sobe_tenda(&acc, aw, ah, w, h, raio_u, raio_v);
    for px in &mut halo {
        for canal in px {
            *canal *= b.intensity;
        }
    }
    halo
}

/// ⭐ **O peso do nível `k`**, com o tecto da tabela — um índice fora dela vale `1`, que é o neutro
/// da cadeia da referência (ela não tem pesos por nível).
fn peso_do_nivel(b: &Bloom, k: usize) -> f32 {
    b.levels
        .get(k)
        .copied()
        .map_or(1.0, |w| if w.is_nan() || w < 0.0 { 0.0 } else { w })
}

/// ⭐⭐⭐ **O RAIO DA TENDA, em unidades de UV** — o mesmo número do gémeo que já shipa
/// (`ph2d_render::motion_fx::BASE_FILTER_RADIUS`).
///
/// ⚠️ **Em UV e não em píxeis, e isso é a lei:** a tenda corre em TODOS os degraus da cadeia, e um
/// raio em píxeis faria o halo encolher com a resolução. *É por ser fracção do quadro que o halo de
/// uma janela pequena e o de uma grande são o mesmo halo.*
pub const RAIO_DA_TENDA: f32 = 0.006;

/// ⭐⭐⭐ **O FILTRO DE 13 TAPS QUE DESCE** — o da referência (Call of Duty / Jimenez, SIGGRAPH 2014),
/// que é o que o [`ph2d_render`] já corre no dispositivo para o brilho do Motion.
///
/// ⛔⛔ **A 1.ª redacção desta crate usava uma média de `2×2` e subia CADA nível directamente ao
/// quadro cheio**, e o dono viu o resultado numa foto: *«bloom bizarro de baixa qualidade»*, com
/// blocos à volta das peças. A causa é aritmética — um bilinear a partir de um nível de `15` px de
/// largura para `1900` desenha a GRELHA desse nível —, e a cura é a da referência: descer com um
/// filtro largo e subir **um degrau de cada vez**.
///
/// ⚠️⚠️ **E o achado maior é que esta lei já existia neste repositório** (`ph2d-render/src/shaders/
/// bloom.wgsl`, doc 67 do Motion). *Antes de construir, MEÇA se a composição já o exprime*
/// (`CLAUDE.md` §5.0) — e eu construí um motor pior ao lado de um melhor.
fn desce13(src: &[[f32; 3]], sw: usize, sh: usize) -> Vec<[f32; 3]> {
    let (dw, dh) = (sw / 2, sh / 2);
    if dw == 0 || dh == 0 {
        return Vec::new();
    }
    #[allow(clippy::cast_precision_loss)]
    let (tx, ty) = (1.0 / sw as f32, 1.0 / sh as f32);
    let mut out = vec![[0.0f32; 3]; dw * dh];
    for j in 0..dh {
        for i in 0..dw {
            #[allow(clippy::cast_precision_loss)]
            let (u, v) = ((i as f32 + 0.5) / dw as f32, (j as f32 + 0.5) / dh as f32);
            let s = |du: f32, dv: f32| amostra_uv(src, sw, sh, u + du, v + dv);
            let (a, c, g, i2) = (
                s(-2.0 * tx, 2.0 * ty),
                s(2.0 * tx, 2.0 * ty),
                s(-2.0 * tx, -2.0 * ty),
                s(2.0 * tx, -2.0 * ty),
            );
            let (b2, d, f, h2) = (
                s(0.0, 2.0 * ty),
                s(-2.0 * tx, 0.0),
                s(2.0 * tx, 0.0),
                s(0.0, -2.0 * ty),
            );
            let e = s(0.0, 0.0);
            let (j2, k2, l2, m2) = (s(-tx, ty), s(tx, ty), s(-tx, -ty), s(tx, -ty));
            let px = &mut out[j * dw + i];
            for ch in 0..3 {
                px[ch] = e[ch].mul_add(
                    0.125,
                    (a[ch] + c[ch] + g[ch] + i2[ch]).mul_add(
                        0.031_25,
                        (b2[ch] + d[ch] + f[ch] + h2[ch])
                            .mul_add(0.0625, (j2[ch] + k2[ch] + l2[ch] + m2[ch]) * 0.125),
                    ),
                );
            }
        }
    }
    out
}

/// ⭐⭐⭐ **A TENDA DE 9 TAPS QUE SOBE UM DEGRAU** — `[1 2 1; 2 4 2; 1 2 1] / 16`, a da referência.
///
/// ⚠️ **Ela sobe UM degrau**, e é isso que a separa da 1.ª redacção desta crate: subir do nível `k`
/// direito ao quadro é um bilinear de `2^k` para `1`, e o que ele desenha é a grelha do nível `k`.
fn sobe_tenda(
    src: &[[f32; 3]],
    sw: usize,
    sh: usize,
    dw: usize,
    dh: usize,
    raio_u: f32,
    raio_v: f32,
) -> Vec<[f32; 3]> {
    let mut out = vec![[0.0f32; 3]; dw * dh];
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return out;
    }
    for j in 0..dh {
        for i in 0..dw {
            #[allow(clippy::cast_precision_loss)]
            let (u, v) = ((i as f32 + 0.5) / dw as f32, (j as f32 + 0.5) / dh as f32);
            let s = |du: f32, dv: f32| amostra_uv(src, sw, sh, u + du, v + dv);
            let e = s(0.0, 0.0);
            let (b2, d, f, h2) = (
                s(0.0, raio_v),
                s(-raio_u, 0.0),
                s(raio_u, 0.0),
                s(0.0, -raio_v),
            );
            let (a, c, g, i2) = (
                s(-raio_u, raio_v),
                s(raio_u, raio_v),
                s(-raio_u, -raio_v),
                s(raio_u, -raio_v),
            );
            let px = &mut out[j * dw + i];
            for ch in 0..3 {
                px[ch] = e[ch].mul_add(
                    4.0,
                    (b2[ch] + d[ch] + f[ch] + h2[ch]).mul_add(2.0, a[ch] + c[ch] + g[ch] + i2[ch]),
                ) / 16.0;
            }
        }
    }
    out
}

/// Amostra bilinear em coordenadas de **UV**, com a borda presa — é o `textureSample` do gémeo.
fn amostra_uv(src: &[[f32; 3]], sw: usize, sh: usize, u: f32, v: f32) -> [f32; 3] {
    #[allow(clippy::cast_precision_loss)]
    let (x, y) = (u.mul_add(sw as f32, -0.5), v.mul_add(sh as f32, -0.5));
    amostra_bilinear(src, sw, sh, x, y)
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
