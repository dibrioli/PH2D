//! **O que o RELEVO É** — os campos compostos que a luz lê, emprestados e nunca materializados.
//!
//! [LOC split de `impasto_light`, 2026-08-10.] A linha do corte é a de sempre nesta casa: aqui mora
//! *o que existe naquele texel* (a dobra das camadas, o dente do papel, a forma doada, o material);
//! no irmão fica *o que o produtor de CPU FAZ com isso* (`apply_impasto_light`) e como ele decide se
//! há trabalho (`impasto_visible` / `impasto_fields`).
//!
//! ⚠️ **O `height_at` é a porta que os DOIS produtores atravessam** — o laço da CPU e o fold que sobe
//! para a GPU. É por isso que o dente do papel entra AQUI e não num laço de consumidor: um relevo que
//! só existe num dos lados é a divergência que o report de 2026-08-10 mediu.

use super::impasto_light::ReliefLayer;
use super::*;
use crate::layers::ReliefComposite;

/// The relief + coverage the light reads, sampled straight out of the layer store — no composed buffer,
/// no per-frame allocation. See [`PainterTool::impasto_fields`].
///
/// `pub(super)` for exactly one reader: [`super::impasto_gpu`], which MATERIALISES these fields into
/// canvas planes so the GPU can shade from them. It is the same fold, asked the same way — the GPU port
/// re-implements the *optics* and deliberately re-implements none of the *plumbing*.
pub(super) struct ReliefFields<'a> {
    /// Every visible layer that carries relief, **bottom-up** — the order it composites in. The order
    /// is load-bearing now: [`ReliefComposite::Level`] buries what is *under* it, and until it existed
    /// the fold was a commutative sum that could iterate in any order at all.
    pub(super) layers: Vec<ReliefLayer<'a>>,
    /// The open stroke's relief on the active layer (`None` outside a stroke, or while erasing — an
    /// erase mutates the layer in place, so there is nothing separate to add).
    pub(super) live_h: Option<&'a [f32]>,
    /// That stroke's **solid paint** plane — the same `u8` quantity the layer will store at stroke end
    /// (`PaintState::stroke_film`), so the light reads the identical number before and after the commit.
    /// (It used to be the raw f32 paint envelope, in "full precision" — and the precision was of the
    /// wrong quantity: the relief's ingredient, not the pigment's alpha.)
    pub(super) live_c: Option<&'a [u8]>,
    /// The open stroke's MATERIAL — the BRUSH's, as a scalar, because a stroke's material is constant
    /// (it comes off the brush). No plane is needed for it, which is what makes the whole per-pixel
    /// material cost one merge at commit instead of a second buffer per stroke.
    pub(super) live_mat: MaterialBytes,
    /// **A FORMA doada** pelo módulo 3D (`docs/3D/05.2`) — `[nx, ny, nz, peso]` por texel do canvas, ou
    /// `None` num documento sem escultura.
    ///
    /// ⚠️ Ela é um plano PRONTO, exatamente como os três do relevo: a malha é rasterizada uma vez e
    /// chega aqui como números. Um segundo rasterizador dentro do passe de luz seria uma segunda
    /// resposta a *"que forma há neste pixel"* — a mesma cerca que mantém o FOLD do relevo fora do
    /// shader.
    pub(super) form: Option<&'a [f32]>,
    /// **A OCLUSÃO DE FORMA** doada com ela — cavidade × os dois AOs, um escalar por texel do canvas.
    ///
    /// ⚠️ Ela chega PRONTA, pela mesma razão do plano acima: quem a compõe é o shader do barro, por
    /// uma porta única (`mesh.wgsl::form_occlusion`), e uma segunda composição aqui divergiria da
    /// que o artista vê no viewport.
    pub(super) form_occ: Option<&'a [f32]>,
    /// `Material::NEUTRAL`, quantised ONCE — the ground the material fold starts from, and the material
    /// a layer with no entry reads as. It is a constant, and it is read per texel; deriving it in the
    /// loop is the kind of thing that costs half a millisecond and looks like nothing.
    pub(super) neutral: MaterialBytes,
    /// **O SUBSTRATO** — o dente do papel, ou `None` quando o relevo dele está desligado.
    ///
    /// ⚠️ **Ele mora AQUI e não no laço de um consumidor, e é a correção do report do Enio de
    /// 2026-08-10** (*"Paper parece não funcionar para Digital"*). A primeira versão somava a
    /// inclinação do dente dentro do `apply_impasto_light`, o produtor de **CPU** — e um documento que
    /// a **GPU** compõe (o caso normal) nunca passa por aquele laço: ele recebe os planos que o
    /// `impasto_gpu` dobra a partir DESTES campos. Sete gates verdes sobre um papel morto na tela.
    ///
    /// `ReliefFields` é a resposta única a *"que relevo há neste texel?"*, e o doc do próprio tipo já
    /// dizia por quê: o port de GPU *"re-implementa a ÓPTICA e deliberadamente não re-implementa o
    /// plumbing"*. Um dente que só existe num dos dois lados é plumbing duplicado.
    pub(super) substrate: Option<super::substrate_relief::Substrate<'a>>,
    pub(super) width: usize,
    pub(super) height: usize,
}

impl ReliefFields<'_> {
    /// Height at a canvas pixel, clamped to the canvas (so the central difference can read across the
    /// dirty rect's edge exactly as a full recompose would) — and passed through the **glass ceiling**, so
    /// the LIVE stroke over an already-full pile shows the same paint the commit is about to store (no pop
    /// at pen-up), and stacked layers top out exactly as stacked strokes do.
    ///
    /// **This is the ONLY place the ceiling is applied**, and that is the design: it is a property of what
    /// the paint *looks like*, not of what the paint *is*. The buffer keeps the true relief, so the sculpt's
    /// plane fits and ball offsets reason about the surface the artist actually built
    /// ([`super::impasto_ceiling::soft_ceiling`], which also records why the hard clamp that used to live here was
    /// not a ceiling but an eraser).
    ///
    /// The fold walks the layers bottom-up, each one scaled by its own depth and joined to the pile
    /// under it by its own composite mode.
    #[inline]
    pub(super) fn height_at(&self, x: i64, y: i64) -> f32 {
        let i = self.index(x, y);
        let mut h = 0.0f32;
        for l in &self.layers {
            let mut own = l.height.map_or(0.0, |f| f[i]);
            if l.active {
                own += self.live_h.map_or(0.0, |s| s[i]);
            }
            own *= l.depth;
            match l.composite {
                ReliefComposite::Add => h += own,
                // Bury, in proportion to this layer's own paint: solid paint IS the surface, bare paint
                // shows the pile below untouched. Anything else would make an empty region of a `Level`
                // layer flatten the whole painting.
                ReliefComposite::Level => {
                    let c = self.layer_cover_at(l, i);
                    h = h * (1.0 - c) + own * c;
                }
            }
        }
        let h = super::impasto_ceiling::soft_ceiling(h);
        // ⚠️ **O dente do papel SOMA à altura da tinta, e entra DEPOIS do teto de vidro.** Depois,
        // porque o teto é uma propriedade de como a TINTA se empilha (`impasto_ceiling`) e um papel não
        // é tinta — comprimi-lo pelo peso de uma pilha que não é dele seria o dente sumir justamente
        // onde há mais o que iluminar. E soma, porque as duas são alturas da MESMA superfície (o papel
        // embaixo, a tinta em cima) e a normal de uma soma de alturas é a soma dos gradientes: não há
        // segunda passada de luz a compor, logo não há duas respostas para *"de onde vem a luz?"*.
        //
        // ⚠️ **É uma APROXIMAÇÃO nomeada, e a doc 19 §1.2 já a nomeava:** tinta espessa *preenche* o
        // dente em vez de somar a ele (`lerp(h_papel, h_tinta, f(carga))`), e o `f` daquela lei quer
        // medição própria. A soma é o que a wave shipa; trocá-la é wave com smoke próprio.
        //
        // ⚠️ **E o dente é amostrado nas coordenadas CLAMPADAS ao canvas, como o resto do fold** — a
        // metade que o gate e2e dos dois produtores pegou. O papel é um padrão contínuo, então
        // amostrá-lo em `x = −1` devolve um número perfeitamente válido; só que o produtor de GPU não
        // pode fazer isso, porque ele lê um PLANO que acaba na borda do canvas e clampa. Medido: a
        // divergência era **1149 bytes, começando em (0,0)**, um anel de um texel em volta da tela.
        // *Um fold cujo domínio é o canvas tem de ser lido no canvas, mesmo quando a fonte sabe
        // responder fora dele.*
        match self.substrate.as_ref() {
            Some(s) => {
                let cx = x.clamp(0, self.width as i64 - 1);
                let cy = y.clamp(0, self.height as i64 - 1);
                h + s.tooth_loads(cx, cy)
            }
            None => h,
        }
    }

    /// **A PRESENÇA do papel** — `1` quando há substrato, `0` quando não há.
    ///
    /// ⚠️ **É a única exceção honesta à regra *"relevo sob cobertura zero não acende"*.** A regra existe
    /// porque relevo de TINTA sem tinta é relevo de nada; um papel, ao contrário, está lá — a cobertura
    /// dele é 1 por definição (a doc 19 §1.3 antecipou exatamente isto). Sem ela o dente acenderia só
    /// onde já houvesse tinta, e o Digital — que não tem plano de `covers` nenhum — não veria nada.
    ///
    /// ⚠️ **Escalar e não plano**, e é o que a torna barata dos dois lados: a presença de um papel é
    /// uniforme na tela, então ela viaja para a GPU num BIT do uniform (o idioma do `has_form`) em vez
    /// de uma textura inteira dizendo `1`.
    #[inline]
    pub(super) fn paper_body(&self) -> f32 {
        f32::from(u8::from(self.substrate.is_some()))
    }

    /// One layer's own paint coverage at `i` (`0..1`) — including the open stroke when it is the active
    /// one. This is the `Level` fold's weight; it is NOT [`Self::cover_at`], which is the whole
    /// canvas's.
    #[inline]
    fn layer_cover_at(&self, l: &ReliefLayer<'_>, i: usize) -> f32 {
        let mut c = l.cover.map_or(0.0, |cv| f32::from(cv[i]) / 255.0);
        if l.active {
            c = c.max(self.live_c.map_or(0.0, |s| f32::from(s[i]) / 255.0));
        }
        c.clamp(0.0, 1.0)
    }

    /// The MATERIAL at a canvas pixel — what the paint there IS.
    ///
    /// Folded bottom-up with **`over`**, weighted by each layer's own coverage: the paint on top is the
    /// paint you see, so an opaque layer's material replaces what is under it and a translucent one
    /// mixes. (Deliberately NOT `max` like [`Self::cover_at`] — coverage is a *presence* and material is
    /// an *identity*, and folding an identity by max would mean "the glossiest layer wins", which is not
    /// a thing paint does.)
    ///
    /// The fold starts at NEUTRAL, not at zero, and that is load-bearing: zero is `roughness = 0`, which
    /// is a MIRROR, so bare paper would be the shiniest thing on the canvas and every stroke's
    /// translucent rim would fade toward glass.
    #[inline]
    pub(super) fn material_at(&self, x: i64, y: i64) -> MaterialBytes {
        let i = self.index(x, y);
        // Hoisted, and it matters: this runs once per TEXEL. Quantising the neutral material inside the
        // loop cost 0.5 ms/move at 2048² all by itself — the fold runs every channel over up to four
        // layers, and `to_bytes` is a clamp and a multiply per channel each time it is asked.
        let neutral = self.neutral;
        let mut m = neutral.map(f32::from);
        for l in &self.layers {
            let a = self.layer_cover_at(l, i);
            if a <= 0.0 {
                continue;
            }
            let src = match l.mat {
                Some(mt) => mt[i],
                // A layer with relief but no material entry is a document from before materials existed:
                // it reads as the neutral material, which IS the pass as it shaded then.
                None => neutral,
            };
            if a >= 1.0 {
                // Opaque paint: `over` is a COPY, not a blend — and taking that path skips the whole
                // per-channel lerp, which is 7 multiply-adds since the Wax filter widened the plane.
                //
                // **Measured: it buys nothing** (3.99 ms/move against 3.97 — noise). Kept anyway,
                // because it is byte-exact and because the measurement is the interesting part: the
                // fold is **bandwidth-bound on the 7-byte plane, not ALU-bound**, so shaving arithmetic
                // off it is shaving the wrong thing. Anyone reaching for a faster material fold should
                // reach for fewer BYTES, or for the GPU — which is the pass's next stop anyway.
                m = src.map(f32::from);
                continue;
            }
            for (c, v) in m.iter_mut().enumerate() {
                *v += (f32::from(src[c]) - *v) * a;
            }
        }
        // The live stroke is the topmost paint on the active layer: its material is the BRUSH's, and it
        // is what the artist is looking at while they turn the knob.
        if let Some(lc) = self.live_c {
            let a = f32::from(lc[i]) / 255.0;
            if a >= 1.0 {
                m = self.live_mat.map(f32::from);
            } else if a > 0.0 {
                for (c, v) in m.iter_mut().enumerate() {
                    *v += (f32::from(self.live_mat[c]) - *v) * a;
                }
            }
        }
        m.map(|v| (v + 0.5) as u8)
    }

    /// Paint coverage at a canvas pixel (`0..1`) — the MAX over the layers, not the sum: it is a
    /// presence, not a quantity (two layers of paint over one pixel do not make it 200% paint).
    ///
    /// Deliberately NOT scaled by the layers' Impasto depth: this is what the light uses to decide it is
    /// looking at *paint* rather than paper ([`paint_body`]), and muting a layer's relief does not make
    /// its pigment any less present on the canvas.
    #[inline]
    pub(super) fn cover_at(&self, x: i64, y: i64) -> f32 {
        let i = self.index(x, y);
        let mut c = self.live_c.map_or(0.0, |l| f32::from(l[i]) / 255.0);
        for l in &self.layers {
            if let Some(cv) = l.cover {
                c = c.max(f32::from(cv[i]) / 255.0);
            }
        }
        c.clamp(0.0, 1.0)
    }

    /// Há doação neste documento? A pergunta que decide se o plano de forma é materializado.
    #[inline]
    pub(super) fn has_form(&self) -> bool {
        self.form.is_some()
    }

    /// A forma doada neste pixel — `[nx, ny, nz, peso]`, ou [`NO_FORM`] onde não há escultura.
    ///
    /// ⚠️ **O guard de peso é load-bearing, e é uma armadilha de verdade:** o G-buffer escreve
    /// `[0, 0, 0, 0]` fora da silhueta, e um `z` ZERO não é "nenhuma forma" — é uma normal DEITADA, que
    /// somada à inclinação da tinta daria um vetor quase horizontal e uma faixa preta em volta de toda
    /// escultura. O neutro de *"não há forma aqui"* é `[0, 0, 1]`, e é ele que faz a soma do
    /// [`Rig::shade_over`] reduzir à expressão da tinta.
    #[inline]
    pub(super) fn form_at(&self, x: i64, y: i64) -> [f32; 4] {
        let Some(f) = self.form else {
            return super::impasto_shade::NO_FORM;
        };
        let i = self.index(x, y) * 4;
        if f[i + 3] <= 0.0 {
            return super::impasto_shade::NO_FORM;
        }
        [f[i], f[i + 1], f[i + 2], f[i + 3]]
    }

    /// A oclusão de forma neste pixel — `1.0` (nada oclui) onde não há doação.
    ///
    /// ⚠️ **O neutro é uma CONSTANTE, e é isso que separa esta porta da irmã.** O `form_at` tem de
    /// inventar `[0, 0, 1]` porque o zero do buffer é uma normal deitada; aqui o zero do buffer
    /// seria PRETO, mas o alvo é limpo em BRANCO do lado de quem doa (`render_gbuffer`), então fora
    /// da silhueta o número já é o neutro e não há caso especial a escrever.
    #[inline]
    pub(super) fn form_occlusion_at(&self, x: i64, y: i64) -> f32 {
        self.form_occ
            .map_or(1.0, |o| o[self.index(x, y)].clamp(0.0, 1.0))
    }

    #[inline]
    fn index(&self, x: i64, y: i64) -> usize {
        let cx = x.clamp(0, self.width as i64 - 1) as usize;
        let cy = y.clamp(0, self.height as i64 - 1) as usize;
        cy * self.width + cx
    }
}
