//! Time scopes: remapping the clock of an upstream subtree (plan §1.5, M2.N1).
//!
//! A [`TimeMap`] is a pure function `t -> t'`. The cook engine
//! ([`crate::cook::Cook::cook_scoped`]) applies one when it descends into the
//! **inputs** of a remapper node, so the node's whole upstream subtree cooks at
//! `t'` while the node itself — and everything downstream — stays on the outer
//! clock. This is the Houdini `timeshift` / Nuke `TimeOffset` reading: the wire
//! carries "this subtree, sampled at another time", not a delayed value.
//!
//! **Why in the substrate and not in a node:** a node only sees its own inputs
//! (`EvalCtx`, FBP black box, ADR-0031); nothing inside a node can change the
//! playhead its *upstream* was pulled at. The remap has to happen in the puller.
//! Keeping it here also earns the memo for free: a `Loop` that returns to the
//! same `t'` hits the cache instead of recomputing the subtree.
//!
//! **Determinism (HR-5):** every mode is `+ - * floor/rem_euclid` on `f64`.
//! No transcendentals, no rounding-mode surprises — a remapped replay is
//! bit-identical across platforms, like the unremapped one.
//!
//! **Isolation:** this module is a leaf (depends on nothing in the crate) and
//! the cook's scoped entry points are additive siblings of the plain ones, so a
//! parallel line that never remaps time compiles and behaves exactly as before.

/// How a [`TimeMap`] rewrites the clock of the subtree above it.
///
/// The vocabulary is the reference catalogue's (`_NODES.md` — timeRemap:
/// `scale` / `loop` / `pingpong` / `freeze` / `reverse`), which is in turn the
/// motion-design standard (After Effects Time Remap, Cavalry Time Offset).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TimeMode {
    /// `t' = t·scale + offset`. Slow motion (`scale < 1`), fast-forward, or a
    /// plain time shift. The identity when `scale = 1, offset = 0`.
    #[default]
    Scale,
    /// `t' = offset + (t·scale) mod duration`. A cycling window: the subtree
    /// replays its first `duration` seconds forever. The memo makes the second
    /// lap free.
    Loop,
    /// A triangle wave of period `2·duration`: forward, then backward. Reads as
    /// a physical back-and-forth rather than the hard cut of `Loop`.
    PingPong,
    /// `t' = offset`, constant. Freezes the subtree on one moment (a still,
    /// while downstream physics keeps running).
    Freeze,
    /// `t' = offset - t·scale`. Runs the subtree backwards.
    Reverse,
    /// **A curva AUTORADA** — `t' = offset + duration · curva(frac(t·scale / duration))`,
    /// o *Time Remapping* do AE, que é uma propriedade **desenhada** e não um
    /// `scale + offset`. Os cinco modos acima são fechados: compô-los dá mapas
    /// afins por pedaços (o `Loop`/`PingPong` dobram), e **nenhuma composição tem
    /// CURVATURA** — que é exactamente o que um ease é.
    ///
    /// ⚠️ **A janela REPETE, e isso não é um detalhe de conveniência — é a razão
    /// de este modo ser o SUPERSET dos dois cíclicos.** Com a tabela da identidade
    /// ele é `Loop` **exacto** (a rampa é o que a tabela representa sem erro); com
    /// uma tabela triangular ele é `PingPong` **à resolução da tabela** (a quina em
    /// `u = 0,5` cai entre duas amostras — dois gates medem cada igualdade e dizem
    /// qual é qual). A primeira versão **clampava**
    /// `u` em `[0,1]`, e o defeito foi apanhado no smoke: a sub-árvore andava durante
    /// `duration` segundos e **congelava para sempre** depois. Isso é a mesma classe
    /// que o `fade` do `motion.oscillator`, construído e removido no mesmo dia — um
    /// controle cuja unidade é *"segundos desde um zero que ninguém vê"*, que na tela
    /// lê como *"travou tudo"*. Um relógio autorado **não expira**.
    ///
    /// ⚠️ **O substrato fica AGNÓSTICO de curva** (o precedente do `LutSpec`): o
    /// que viaja aqui é uma **tabela de `f32`** que a crate do NÓ preenche a partir
    /// da forma autorada; nada em `ph2d-nodegraph` sabe o que um ponto de controle
    /// é. Uma curva com `duration` não-positiva degrada para [`TimeMode::Scale`],
    /// como o `Loop` já faz.
    Curve,
}

/// Quantas amostras a tabela do [`TimeMode::Curve`] carrega.
///
/// ⚠️ **MEDIDO, não escolhido** (`measure_curve_lut`): o erro de uma tabela lida
/// linearmente é um erro de **TEMPO**, e ele escala com a JANELA — o recurso é o
/// `duration`, que o slider do nó capa em **10 s**. Pior caso medido sobre uma
/// curva com um *hold* no meio (a mais dura das duas fixtures):
///
/// | amostras | erro a 2 s | a 10 s | pior Δ de velocidade |
/// |---|---|---|---|
/// | 16 | 0,842 quadro | 4,21 | 0,443 |
/// | 32 | 0,215 | 1,08 | 0,226 |
/// | 64 | 0,054 | 0,270 | 0,113 |
/// | **128** | **0,0135** | **0,0675** | 0,056 |
/// | 256 | 0,0034 | 0,017 | 0,027 |
///
/// **128** põe o pior caso da janela MÁXIMA em **0,0675 de um quadro** a 60 fps —
/// sete vezes abaixo de meio quadro, que é o limiar em que uma amostra pode
/// sequer existir. O preço é **512 bytes** num `TimeMap`, que é `Copy` e entra na
/// chave do memo; a alternativa exacta (os pontos de controle) poria a LEI da
/// curva dentro do substrato, que é a segunda resposta que o `LutSpec` recusou.
pub const TIME_CURVE_SAMPLES: usize = 128;

/// A tabela da IDENTIDADE (`curva(u) = u`) — o que um `TimeMode::Curve` sem forma
/// autorada carrega, pela lei do `value.curve`: *uma curva não-desenhada é a
/// identidade*.
#[must_use]
pub const fn identity_curve_lut() -> [f32; TIME_CURVE_SAMPLES] {
    let mut out = [0.0f32; TIME_CURVE_SAMPLES];
    let last = (TIME_CURVE_SAMPLES - 1) as f32;
    let mut i = 0;
    while i < TIME_CURVE_SAMPLES {
        out[i] = i as f32 / last;
        i += 1;
    }
    out
}

/// Lê a tabela em `u ∈ [0,1]` com interpolação LINEAR — a mesma lei nos dois
/// lados, e a razão de o erro acima ser o que é.
#[must_use]
pub fn sample_curve_lut(lut: &[f32; TIME_CURVE_SAMPLES], u: f32) -> f32 {
    let last = (TIME_CURVE_SAMPLES - 1) as f32;
    let x = (u.clamp(0.0, 1.0) * last).min(last);
    let i = x.floor() as usize;
    let f = x - i as f32;
    let a = lut[i];
    let b = lut[(i + 1).min(TIME_CURVE_SAMPLES - 1)];
    a + (b - a) * f
}

impl TimeMode {
    /// Decode the `f32` an enum-valued param stores (its option index), by the
    /// declaration order above. Out-of-range → [`TimeMode::Scale`], the
    /// identity: an unknown mode must not invent a time warp.
    #[must_use]
    pub fn from_index(i: f32) -> Self {
        match i.round() as i32 {
            1 => TimeMode::Loop,
            2 => TimeMode::PingPong,
            3 => TimeMode::Freeze,
            4 => TimeMode::Reverse,
            5 => TimeMode::Curve,
            _ => TimeMode::Scale,
        }
    }

    /// The stable discriminant used when hashing a scope (see
    /// [`TimeMap::hash_into`]).
    #[must_use]
    pub fn index(self) -> u8 {
        match self {
            TimeMode::Scale => 0,
            TimeMode::Loop => 1,
            TimeMode::PingPong => 2,
            TimeMode::Freeze => 3,
            TimeMode::Reverse => 4,
            TimeMode::Curve => 5,
        }
    }
}

/// A clock rewrite applied to one node's upstream subtree.
///
/// `duration` is only read by [`TimeMode::Loop`] / [`TimeMode::PingPong`]; a
/// non-positive one degrades to [`TimeMode::Scale`] rather than dividing by
/// zero (a zero-length loop has no meaning, and `NaN` would poison every
/// downstream fingerprint).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimeMap {
    pub mode: TimeMode,
    pub scale: f64,
    pub offset: f64,
    pub duration: f64,
    /// A tabela do [`TimeMode::Curve`] — ver [`TIME_CURVE_SAMPLES`]. Lida **só**
    /// naquele modo; nos outros ela viaja e ninguém a consulta.
    pub curve: [f32; TIME_CURVE_SAMPLES],
}

impl Default for TimeMap {
    fn default() -> Self {
        Self {
            mode: TimeMode::Scale,
            scale: 1.0,
            offset: 0.0,
            duration: 1.0,
            curve: identity_curve_lut(),
        }
    }
}

impl TimeMap {
    /// Is this the identity map (`t' = t`)? An identity scope is dropped by the
    /// cook, so a remapper left at its defaults costs nothing — no second cache
    /// lane, no missed memo against unscoped consumers of the same subtree.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.mode == TimeMode::Scale && self.scale == 1.0 && self.offset == 0.0
    }

    /// `t -> t'`. Total: never panics, never returns `NaN` for finite input.
    #[must_use]
    pub fn apply(&self, t: f64) -> f64 {
        let scaled = t * self.scale;
        let looping = self.duration > 0.0 && self.duration.is_finite();
        match self.mode {
            TimeMode::Scale => scaled + self.offset,
            TimeMode::Freeze => self.offset,
            TimeMode::Reverse => self.offset - scaled,
            // A curva mapeia a JANELA em si mesma: com a tabela da identidade isto
            // reduz a `offset + t·scale` (o `Scale`) a menos do arredondamento da
            // ida-e-volta pela divisão — a byte-identidade do mundo que já shipava
            // é garantida pelo MODO, que continua a ser `Scale`.
            TimeMode::Curve if looping => {
                // A janela REPETE (`rem_euclid`, a mesma volta do `Loop` logo abaixo) —
                // ver a nota do variant: clampar aqui fazia a sub-árvore expirar.
                let u = (scaled / self.duration).rem_euclid(1.0);
                self.offset + self.duration * f64::from(sample_curve_lut(&self.curve, u as f32))
            }
            TimeMode::Loop if looping => self.offset + scaled.rem_euclid(self.duration),
            TimeMode::PingPong if looping => {
                // Triangle wave: fold the second half of each 2·duration period
                // back on itself, so the subtree plays forward then backward
                // with no discontinuity at the turn.
                let u = scaled.rem_euclid(2.0 * self.duration);
                let folded = if u > self.duration {
                    2.0 * self.duration - u
                } else {
                    u
                };
                self.offset + folded
            }
            // Degenerate duration: behave as the plain scale, never divide by 0.
            TimeMode::Loop | TimeMode::PingPong | TimeMode::Curve => scaled + self.offset,
        }
    }

    /// Fold this map into an FNV-1a accumulator, so the cook can key its memo
    /// per *scope chain* (`ScopeKey`). Bit patterns, not values: two maps that
    /// differ only in `-0.0` vs `0.0` are distinguishable, and a `NaN` param
    /// hashes consistently rather than comparing unequal to itself.
    pub fn hash_into(&self, hash: &mut u64) {
        let mut mix = |bytes: &[u8]| {
            for b in bytes {
                *hash ^= u64::from(*b);
                *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        };
        mix(&[self.mode.index()]);
        mix(&self.scale.to_bits().to_le_bytes());
        mix(&self.offset.to_bits().to_le_bytes());
        mix(&self.duration.to_bits().to_le_bytes());
        // ⚠️ **A tabela entra na chave SÓ no modo que a lê.** Misturá-la sempre
        // partiria a pista de memo de dois mapas `Scale` que diferem num número
        // que ninguém consulta — um miss, não um erro, mas um miss de graça; e
        // NÃO misturá-la no modo `Curve` serviria a sub-árvore de uma curva
        // ANTERIOR, que é o erro.
        if self.mode == TimeMode::Curve {
            for v in &self.curve {
                mix(&v.to_bits().to_le_bytes());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A curvatura do mapa num intervalo — a segunda diferença central. **Um mapa
    /// AFIM tem zero, exactamente**; é isso que separa uma curva de qualquer
    /// composição dos cinco modos fechados.
    fn curvature(m: &TimeMap, a: f64, b: f64) -> f64 {
        let n = 64;
        let h = (b - a) / n as f64;
        let mut worst = 0.0f64;
        for k in 1..n {
            let t = a + k as f64 * h;
            let d2 = m.apply(t + h) - 2.0 * m.apply(t) + m.apply(t - h);
            worst = worst.max(d2.abs());
        }
        worst
    }

    /// A tabela de um ease com tangentes chatas, amostrada como o nó a preenche.
    fn ease_lut() -> [f32; TIME_CURVE_SAMPLES] {
        let mut out = [0.0f32; TIME_CURVE_SAMPLES];
        let last = (TIME_CURVE_SAMPLES - 1) as f32;
        for (k, o) in out.iter_mut().enumerate() {
            let u = k as f32 / last;
            *o = u * u * (3.0 - 2.0 * u); // smoothstep, o `Interp::Smooth`
        }
        out
    }

    /// **Uma CURVA dobra o relógio, e nenhuma composição dos cinco modos dobra.**
    ///
    /// ⚠️ Este é o gate que fecha a linha 45 da folha 06, e ele traz o CONTROLE
    /// dentro de si: os cinco modos são fechados e compô-los dá mapas **afins por
    /// pedaços** (o `Loop`/`PingPong` dobram, o `Reverse` inverte) — a segunda
    /// diferença deles é **o ULP da aritmética** longe de uma dobra, por mais que
    /// se encadeiem. É a curvatura que não é exprimível, não o deslocamento.
    ///
    /// ⚠️ **A barra é um FOSSO, não um zero** — a primeira versão pedia `== 0.0` e
    /// reprovou sobre produto CORRETO: `Scale` mede **4,44e-16** (o arredondamento
    /// de `t·scale`, não curvatura). Medido, a curva dobra **1,1e-4** contra
    /// **≤ 4,5e-16** de todo mapa afim — oito ordens de fosso, e um gate que pede
    /// zero de um `f64` é um gate que código certo não consegue satisfazer.
    #[test]
    fn a_curve_bends_the_clock_and_no_chain_of_the_closed_modes_can() {
        let curved = TimeMap {
            mode: TimeMode::Curve,
            scale: 1.0,
            offset: 0.0,
            duration: 2.0,
            curve: ease_lut(),
        };
        let bend = curvature(&curved, 0.2, 1.8);
        assert!(
            bend > 1e-6,
            "a curva tem de dobrar o relogio, e mede {bend}"
        );

        // CONTROLE: os cinco fechados, sozinhos e ENCADEADOS, num intervalo sem dobra.
        for (tag, m) in [
            ("Scale", map(TimeMode::Scale, 2.0, 0.3, 2.0)),
            ("Reverse", map(TimeMode::Reverse, 1.5, 1.0, 2.0)),
            ("Loop", map(TimeMode::Loop, 1.0, 0.0, 8.0)),
            ("PingPong", map(TimeMode::PingPong, 1.0, 0.0, 8.0)),
            ("Freeze", map(TimeMode::Freeze, 1.0, 0.7, 2.0)),
        ] {
            let c = curvature(&m, 0.2, 1.8);
            assert!(
                c < 1e-12 && c * 1e6 < bend,
                "{tag} e' afim no intervalo: mede {c} contra {bend} da curva"
            );
        }
        // E a COMPOSIÇÃO de dois deles continua afim — encadear não cria curvatura.
        let a = map(TimeMode::Scale, 2.0, 0.3, 2.0);
        let b = map(TimeMode::Reverse, 1.5, 1.0, 2.0);
        let n = 64;
        let (lo, hi) = (0.2f64, 1.8f64);
        let h = (hi - lo) / n as f64;
        let chained = |t: f64| b.apply(a.apply(t));
        let mut worst = 0.0f64;
        for k in 1..n {
            let t = lo + k as f64 * h;
            worst = worst.max((chained(t + h) - 2.0 * chained(t) + chained(t - h)).abs());
        }
        assert!(
            worst < 1e-12 && worst * 1e6 < bend,
            "dois mapas afins compostos medem {worst} contra {bend} da curva"
        );
    }

    /// A tabela de um relógio com uma PAUSA desenhada no meio — a forma da cena
    /// `=58`, e a mais dura das fixtures (um patamar tem derivada zero de um lado
    /// e não-zero do outro).
    fn paused_lut() -> [f32; TIME_CURVE_SAMPLES] {
        let mut out = [0.0f32; TIME_CURVE_SAMPLES];
        let last = (TIME_CURVE_SAMPLES - 1) as f32;
        for (k, o) in out.iter_mut().enumerate() {
            let u = k as f32 / last;
            *o = if u < 0.40 {
                u * (0.42 / 0.40)
            } else if u < 0.60 {
                0.42
            } else {
                0.42 + (u - 0.60) * (0.58 / 0.40)
            };
        }
        out
    }

    /// **Uma curva não desenhada é EXACTAMENTE o `Loop`** — e é isso que faz deste
    /// modo o superset dos dois cíclicos, não uma sexta opção ao lado deles.
    ///
    /// ⚠️ Este gate **substituiu** um que pedia igualdade com o `Scale`, e a troca é
    /// a correcção que o smoke do Enio forçou: a lei antiga clampava a janela, então
    /// a curva neutra era um `Scale` que **parava** no fim dela. A lei do
    /// `value.curve` (*uma curva não-desenhada é a identidade*) continua de pé — o
    /// que mudou é que a identidade é lida DENTRO de uma janela que repete, e a
    /// identidade repetida chama-se `Loop`. A varredura cobre **cinco** janelas de
    /// propósito: uma só provaria a forma e não a repetição.
    #[test]
    fn an_undrawn_curve_is_exactly_the_loop_it_generalises() {
        let curved = TimeMap {
            mode: TimeMode::Curve,
            scale: 1.0,
            offset: 0.25,
            duration: 4.0,
            ..TimeMap::default()
        };
        let looped = map(TimeMode::Loop, 1.0, 0.25, 4.0);
        let mut worst = 0.0f64;
        for k in 0..=400 {
            let t = k as f64 * 0.05; // 0 .. 20 s = cinco janelas
            worst = worst.max((curved.apply(t) - looped.apply(t)).abs());
        }
        assert!(
            worst < 1e-6,
            "a curva neutra tem de SER o Loop em cinco janelas, e diverge {worst}"
        );
    }

    /// **Uma curva triangular é o `PingPong`, à resolução da tabela.**
    ///
    /// ⚠️ A barra **não** é zero, e o motivo é geométrico e não numérico: a quina do
    /// triângulo mora em `u = 0,5`, e com 128 amostras `0,5 = 63,5/127` cai **entre**
    /// duas delas — a tabela corta o bico. O erro é o da célula
    /// (`duration/(N−1) ≈ 0,047 s` aqui), e pedir zero seria pedir que uma tabela
    /// representasse exactamente uma função que ela não pode.
    #[test]
    fn a_triangle_curve_is_the_pingpong_it_generalises_to_table_resolution() {
        let mut tri = [0.0f32; TIME_CURVE_SAMPLES];
        let last = (TIME_CURVE_SAMPLES - 1) as f32;
        for (k, o) in tri.iter_mut().enumerate() {
            let u = k as f32 / last;
            *o = (1.0 - (2.0 * u - 1.0).abs()) * 0.5;
        }
        // Uma volta do PingPong dura `2·duration`; a mesma volta como curva é UMA janela.
        let curved = TimeMap {
            mode: TimeMode::Curve,
            scale: 1.0,
            offset: 0.0,
            duration: 6.0,
            curve: tri,
        };
        let pinged = map(TimeMode::PingPong, 1.0, 0.0, 3.0);
        let mut worst = 0.0f64;
        for k in 0..=600 {
            let t = k as f64 * 0.05; // 0 .. 30 s = cinco voltas
            worst = worst.max((curved.apply(t) - pinged.apply(t)).abs());
        }
        let cell = 6.0 / (TIME_CURVE_SAMPLES - 1) as f64;
        assert!(
            worst < cell,
            "a curva triangular tem de ser o PingPong a menos da celula ({cell}), e diverge {worst}"
        );
        // CONTROLE: a divergência é a QUINA, não um desalinhamento — longe dela o
        // erro cai para o ULP da tabela.
        let far = (curved.apply(1.0) - pinged.apply(1.0)).abs();
        assert!(
            far < 1e-6,
            "longe da quina os dois tem de coincidir, e medem {far} de diferenca"
        );
    }

    /// **NENHUM relógio autorado EXPIRA** — o gate de classe, irmão do
    /// `no_control_of_this_oscillator_expires_with_the_clock`.
    ///
    /// ⚠️ Este é o defeito que o smoke do Enio apanhou e que a suíte não apanhava: a
    /// cena `=58` andava durante `duration` segundos e **congelava para sempre**, e o
    /// gate que existia media o movimento DENTRO da janela — verde sobre produto
    /// morto. A cura é a lei (`rem_euclid` no lugar do `clamp`); este gate é a prova
    /// de que ela vale, e traz o **CONTROLE NEGATIVO dentro de si**: a lei antiga,
    /// escrita à mão aqui, morre onde a nova vive. Sem essa metade, um `clamp` que
    /// voltasse passaria despercebido de novo.
    #[test]
    fn no_authored_clock_expires_the_curve_still_moves_far_past_its_window() {
        let m = TimeMap {
            mode: TimeMode::Curve,
            scale: 1.0,
            offset: 0.0,
            duration: 6.0,
            curve: paused_lut(),
        };
        // A 37ª janela — bem longe de qualquer coisa que um autor tenha desenhado.
        let base = 36.0 * m.duration;
        let step = 0.05;
        let mut moved = 0.0f64;
        for k in 0..(m.duration / step) as i32 {
            let t = base + f64::from(k) * step;
            moved = moved.max((m.apply(t + step) - m.apply(t)).abs());
        }
        assert!(
            moved > 0.02,
            "na 37a janela o relogio tem de andar, e o maior passo mede {moved}"
        );

        // E a PAUSA desenhada continua lá — repetir não é ignorar a forma.
        let held = (m.apply(base + 0.50 * m.duration) - m.apply(base + 0.45 * m.duration)).abs();
        assert!(
            held < 1e-3,
            "a pausa desenhada tem de valer na 37a janela tambem, e o relogio anda {held}"
        );

        // CONTROLE NEGATIVO — a lei ANTIGA (clamp) no mesmo instante: morta.
        let clamped = |t: f64| {
            let u = (t / m.duration).clamp(0.0, 1.0);
            m.duration * f64::from(sample_curve_lut(&m.curve, u as f32))
        };
        let old = (clamped(base + step) - clamped(base)).abs();
        assert!(
            old < 1e-9,
            "o controle negativo tem de estar CONGELADO na 37a janela, e anda {old}"
        );
    }

    /// **Uma janela degenerada degrada para `Scale`** — o guard que o `Loop` já
    /// tinha, pelo mesmo motivo: nunca dividir por zero.
    #[test]
    fn a_curve_with_no_window_degrades_to_the_plain_scale() {
        for duration in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            let m = TimeMap {
                mode: TimeMode::Curve,
                scale: 2.0,
                offset: 0.5,
                duration,
                curve: ease_lut(),
            };
            assert_eq!(
                m.apply(1.0),
                2.5,
                "duration {duration} tem de cair no Scale"
            );
        }
    }

    /// **A tabela entra na chave do memo SÓ no modo que a lê.**
    ///
    /// As duas metades falham por motivos opostos: não misturá-la no `Curve`
    /// serviria a sub-árvore de uma curva ANTERIOR (um erro), e misturá-la sempre
    /// partiria a pista de dois mapas `Scale` que diferem num número que ninguém
    /// consulta (um miss de graça).
    #[test]
    fn the_table_keys_the_memo_only_where_it_is_read() {
        let key = |m: &TimeMap| {
            let mut h = 0xcbf2_9ce4_8422_2325u64;
            m.hash_into(&mut h);
            h
        };
        let a = TimeMap {
            mode: TimeMode::Curve,
            curve: identity_curve_lut(),
            ..TimeMap::default()
        };
        let b = TimeMap {
            curve: ease_lut(),
            ..a
        };
        assert_ne!(key(&a), key(&b), "duas curvas distintas sao duas pistas");

        let c = TimeMap {
            mode: TimeMode::Scale,
            curve: identity_curve_lut(),
            ..TimeMap::default()
        };
        let d = TimeMap {
            curve: ease_lut(),
            ..c
        };
        assert_eq!(key(&c), key(&d), "fora do Curve a tabela nao parte a pista");
    }

    fn map(mode: TimeMode, scale: f64, offset: f64, duration: f64) -> TimeMap {
        TimeMap {
            curve: identity_curve_lut(),
            mode,
            scale,
            offset,
            duration,
        }
    }

    #[test]
    fn scale_shifts_and_stretches_and_the_default_is_the_identity() {
        assert!(TimeMap::default().is_identity());
        assert_eq!(TimeMap::default().apply(3.25), 3.25);
        // Half speed, shifted a second later.
        let m = map(TimeMode::Scale, 0.5, 1.0, 1.0);
        assert!(!m.is_identity());
        assert_eq!(m.apply(4.0), 3.0);
    }

    #[test]
    fn loop_cycles_and_pingpong_folds() {
        let l = map(TimeMode::Loop, 1.0, 0.0, 2.0);
        assert_eq!(l.apply(0.5), 0.5);
        assert_eq!(l.apply(2.5), 0.5, "the second lap repeats the first");
        assert_eq!(l.apply(-0.5), 1.5, "negative time wraps forward, not back");

        let p = map(TimeMode::PingPong, 1.0, 0.0, 2.0);
        assert_eq!(p.apply(0.5), 0.5);
        assert_eq!(p.apply(2.5), 1.5, "past the turn it runs backwards");
        assert_eq!(p.apply(3.5), 0.5);
        assert_eq!(p.apply(4.0), 0.0, "and lands back at the start");
        // Continuous at the turning point (no jump the eye would catch).
        assert_eq!(p.apply(2.0), 2.0);
    }

    #[test]
    fn freeze_is_constant_and_reverse_runs_backwards() {
        let f = map(TimeMode::Freeze, 1.0, 1.5, 1.0);
        assert_eq!(f.apply(0.0), 1.5);
        assert_eq!(f.apply(99.0), 1.5);

        let r = map(TimeMode::Reverse, 1.0, 10.0, 1.0);
        assert_eq!(r.apply(0.0), 10.0);
        assert_eq!(r.apply(4.0), 6.0);
    }

    /// A zero (or infinite) duration must not divide by zero into `NaN`: a
    /// `NaN` playhead poisons every downstream fingerprint (`NaN != NaN`, so
    /// nothing ever hits the memo again) — a silent, unbounded recompute.
    #[test]
    fn a_degenerate_duration_degrades_to_scale_instead_of_nan() {
        for bad in [0.0, -2.0, f64::INFINITY, f64::NAN] {
            for mode in [TimeMode::Loop, TimeMode::PingPong] {
                let t = map(mode, 1.0, 0.0, bad).apply(3.0);
                assert_eq!(t, 3.0, "mode {mode:?} with duration {bad} fell through");
            }
        }
    }

    #[test]
    fn from_index_round_trips_and_rejects_out_of_range() {
        for mode in [
            TimeMode::Scale,
            TimeMode::Loop,
            TimeMode::PingPong,
            TimeMode::Freeze,
            TimeMode::Reverse,
        ] {
            assert_eq!(TimeMode::from_index(f32::from(mode.index())), mode);
        }
        // Junk decodes to the identity mode, never to a random warp.
        assert_eq!(TimeMode::from_index(-1.0), TimeMode::Scale);
        assert_eq!(TimeMode::from_index(99.0), TimeMode::Scale);
    }

    #[test]
    fn the_scope_hash_separates_maps_that_differ_anywhere() {
        let h = |m: TimeMap| {
            let mut acc = 0xcbf2_9ce4_8422_2325u64;
            m.hash_into(&mut acc);
            acc
        };
        let base = map(TimeMode::Loop, 1.0, 0.0, 2.0);
        assert_eq!(h(base), h(base), "stable");
        assert_ne!(h(base), h(map(TimeMode::PingPong, 1.0, 0.0, 2.0)));
        assert_ne!(h(base), h(map(TimeMode::Loop, 2.0, 0.0, 2.0)));
        assert_ne!(h(base), h(map(TimeMode::Loop, 1.0, 1.0, 2.0)));
        assert_ne!(h(base), h(map(TimeMode::Loop, 1.0, 0.0, 3.0)));
    }
}
