//! **O que o RELEVO É** — os campos compostos que a luz lê, emprestados e nunca materializados.
//!
//! [LOC split de `impasto_light`, 2026-08-10.] A linha do corte é a de sempre nesta casa: aqui mora
//! *o que existe naquele texel* (a dobra das camadas, o dente do papel, o material);
//! no irmão fica *o que o produtor de CPU FAZ com isso* (`apply_impasto_light`) e como ele decide se
//! há trabalho (`impasto_visible` / `impasto_fields`).
//!
//! ⚠️ **O `height_at` é a porta que os DOIS produtores atravessam** — o laço da CPU e o fold que sobe
//! para a GPU. É por isso que o dente do papel entra AQUI e não num laço de consumidor: um relevo que
//! só existe num dos lados é a divergência que o report de 2026-08-10 mediu.

use super::impasto_light::ReliefLayer;
use super::*;

/// The relief + coverage the light reads, sampled straight out of the layer store — no composed buffer,
/// no per-frame allocation. See [`PainterTool::impasto_fields`].
///
/// `pub(super)` for exactly one reader: [`super::impasto_gpu`], which MATERIALISES these fields into
/// canvas planes so the GPU can shade from them. It is the same fold, asked the same way — the GPU port
/// re-implements the *optics* and deliberately re-implements none of the *plumbing*.
pub(super) struct ReliefFields<'a> {
    /// ⭐ A dobra MATERIALIZADA através dos ajustes que agem no relevo — `(altura, corpo)` por píxel,
    /// antes do tecto. `None` = a dobra por píxel de sempre.
    pub(super) folded: Option<(Vec<f32>, Vec<f32>)>,
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
    /// §BUG-papel (smoke do dono 27/10): os píxeis RGBA8 de cada camada raster VISÍVEL com a opacidade dela —
    /// a TINTA que cobre o papel. O dente vê-se pela parte que a tinta deixa passar ([`Self::papel_a_vista`]).
    /// Vazio sem substrato. O `bool` diz se a camada recebeu tinta do Wet (`PainterTool::tintas_molhadas`).
    pub(super) tinta: Vec<(&'a [u8], f32, bool)>,
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
    /// The fold walks the layers bottom-up, each one joined to the pile under it by
    /// [`crate::layers::fold_relief_step`] — the ONE fold.
    #[inline]
    pub(super) fn height_at(&self, x: i64, y: i64) -> f32 {
        let i = self.index(x, y);
        let h = match &self.folded {
            Some((f, _)) => f[i],
            None => self.fold_at(i),
        };
        let cx = x.clamp(0, self.width as i64 - 1);
        let cy = y.clamp(0, self.height as i64 - 1);
        self.com_o_papel(super::impasto_ceiling::soft_ceiling(h), i, cx, cy)
    }

    /// A altura da tinta já com o tecto, mais o dente do papel que a tinta deixa ver em `i` = `(cx, cy)` (clampados).
    #[inline]
    fn com_o_papel(&self, h: f32, i: usize, cx: i64, cy: i64) -> f32 {
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
                // A tinta tapa o dente mas fica à altura média dele: sem o degrau que a fazia parecer afundada (Bug #71).
                // Só a tinta com CORPO o enche (doc 19 §1.2; dono 08/11: no Digital o relevo do papel tem de se ver na
                // tinta): a opaca tapa-o na proporção da altura dela sobre a do dente — sem corpo (Digital, aguada)
                // a tinta segue o papel; mais alta que o dente (o óleo grosso, §BUG-papel), enterra-o como antes.
                // A tinta do Wet (que já traz o papel desenhado) tapa-o pela opacidade, como sempre (§BUG-papel).
                let enche = (h / (CORPO_QUE_ENCHE * s.amplitude_em_cargas())).clamp(0.0, 1.0);
                let (molhada, outra) = self.papel_a_vista(i);
                let v = molhada * (1.0 - enche * (1.0 - outra));
                h + s.dente_na_tela(cx, cy) * v + s.meio_do_dente() * (1.0 - v)
            }
            None => h,
        }
    }

    /// **O papel que a tinta deixa ver** em `i` (`0..1`): `Π(1 − α·opacidade)` das camadas por cima dele. A tinta
    /// OPACA enterra o dente (o Rebelle, o ArtRage: tinta grossa não mostra o grão); a fina ou transparente — a
    /// opacidade do pincel, a da camada, a aguada — deixa-o ver na proporção. Antes o dente SOMAVA à tinta em
    /// toda a parte e o grão aparecia pela tinta grossa «como se fosse transparente» (`oraculo/README.md`
    /// §BUG-papel).
    ///
    /// Devolve `(sob a tinta do Wet, sob a outra)` — as duas leis de [`Self::height_at`].
    #[inline]
    fn papel_a_vista(&self, i: usize) -> (f32, f32) {
        let (mut molhada, mut outra) = (1.0f32, 1.0f32);
        for &(px, op, wet) in &self.tinta {
            let deixa = 1.0 - f32::from(px[i * 4 + 3]) / 255.0 * op;
            if wet {
                molhada *= deixa;
            } else {
                outra *= deixa;
            }
        }
        (molhada, outra)
    }

    /// **A PRESENÇA do papel** — `1` quando há substrato, `0` quando não há.
    ///
    /// ⚠️ **É a única exceção honesta à regra *"relevo sob cobertura zero não acende"*.** A regra existe
    /// porque relevo de TINTA sem tinta é relevo de nada; um papel, ao contrário, está lá — a cobertura
    /// dele é 1 por definição (a doc 19 §1.3 antecipou exatamente isto). Sem ela o dente acenderia só
    /// onde já houvesse tinta, e o Digital — que não tem plano de `covers` nenhum — não veria nada.
    ///
    /// ⚠️ **Escalar e não plano**, e é o que a torna barata dos dois lados: a presença de um papel é
    /// uniforme na tela, então ela viaja para a GPU num BIT do uniform em vez de uma textura inteira
    /// dizendo `1`.
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
            // A layer with relief but no material entry is a document from before materials existed:
            // it reads as the neutral material, which IS the pass as it shaded then.
            sobre(&mut m, l.mat.map_or(neutral, |mt| mt[i]), a);
        }
        // The live stroke is the topmost paint on the active layer: its material is the BRUSH's, and it
        // is what the artist is looking at while they turn the knob.
        if let Some(lc) = self.live_c {
            let a = f32::from(lc[i]) / 255.0;
            if a > 0.0 {
                sobre(&mut m, self.live_mat, a);
            }
        }
        m.map(|v| (v + 0.5) as u8)
    }

    /// ⭐ **B15.6 — a dobra de UMA LINHA, a porta das duas pistas** (a luz da CPU em faixas e os planos da placa): a
    /// altura ([`Self::height_at`]), o corpo f32 ([`Self::cover_at`]) e o material ([`Self::material_at`]) de
    /// `x0..x0 + alt.len()` na linha `y`, com o índice e o corpo de cada camada lidos UMA vez por texel (os três
    /// amostradores liam-nos quatro vezes). Ao bit com eles (`a_dobra_da_linha_e_a_dos_tres_amostradores_ao_bit`).
    pub(super) fn dobra_da_linha(
        &self,
        y: i64,
        x0: i64,
        alt: &mut [f32],
        cob: &mut [f32],
        mat: &mut [MaterialBytes],
    ) {
        debug_assert!(alt.len() == cob.len() && cob.len() == mat.len());
        let cy = y.clamp(0, self.height as i64 - 1);
        let linha = cy as usize * self.width;
        for (k, ((a, c), mt)) in alt
            .iter_mut()
            .zip(cob.iter_mut())
            .zip(mat.iter_mut())
            .enumerate()
        {
            let cx = (x0 + k as i64).clamp(0, self.width as i64 - 1);
            let i = linha + cx as usize;
            let live = self.live_c.map_or(0.0, |l| f32::from(l[i]) / 255.0);
            let (mut cover_max, mut corpo) = (0.0f32, live);
            let mut m = self.neutral.map(f32::from);
            for l in &self.layers {
                if let Some(cv) = l.cover {
                    corpo = corpo.max(f32::from(cv[i]) / 255.0);
                }
                let la = self.layer_cover_at(l, i);
                cover_max = cover_max.max(la);
                if la > 0.0 {
                    sobre(&mut m, l.mat.map_or(self.neutral, |v| v[i]), la);
                }
            }
            if live > 0.0 {
                sobre(&mut m, self.live_mat, live);
            }
            *mt = m.map(|v| (v + 0.5) as u8);
            let h = match &self.folded {
                Some((f, cf)) => {
                    corpo = cf[i];
                    f[i]
                }
                None => {
                    let mut h = crate::layers::RELIEF_FOLD_SEED;
                    for l in &self.layers {
                        let cover = self.layer_cover_at(l, i);
                        h = crate::layers::fold_relief_step(
                            h,
                            self.own_at(l, i),
                            l.depth,
                            l.composite,
                            cover,
                            cover_max,
                        );
                    }
                    h
                }
            };
            *c = corpo.clamp(0.0, 1.0);
            *a = self.com_o_papel(super::impasto_ceiling::soft_ceiling(h), i, cx, cy);
        }
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
        if let Some((_, corpo)) = &self.folded {
            return corpo[i].clamp(0.0, 1.0);
        }
        let mut c = self.live_c.map_or(0.0, |l| f32::from(l[i]) / 255.0);
        for l in &self.layers {
            if let Some(cv) = l.cover {
                c = c.max(f32::from(cv[i]) / 255.0);
            }
        }
        c.clamp(0.0, 1.0)
    }

    #[inline]
    fn index(&self, x: i64, y: i64) -> usize {
        let cx = x.clamp(0, self.width as i64 - 1) as usize;
        let cy = y.clamp(0, self.height as i64 - 1) as usize;
        cy * self.width + cx
    }
}

impl ReliefFields<'_> {
    /// A dobra POR PÍXEL de sempre (sem ajuste que aja no relevo), antes do tecto.
    #[inline]
    fn fold_at(&self, i: usize) -> f32 {
        let mut h = crate::layers::RELIEF_FOLD_SEED;
        let cover_max = self
            .layers
            .iter()
            .map(|l| self.layer_cover_at(l, i))
            .fold(0.0f32, f32::max);
        for l in &self.layers {
            let cover = self.layer_cover_at(l, i);
            h = crate::layers::fold_relief_step(
                h,
                self.own_at(l, i),
                l.depth,
                l.composite,
                cover,
                cover_max,
            );
        }
        h
    }

    /// A altura própria da camada no píxel, com o traço aberto na activa.
    #[inline]
    fn own_at(&self, l: &ReliefLayer<'_>, i: usize) -> f32 {
        let mut own = l.height.map_or(0.0, |f| f[i]);
        if l.active {
            own += self.live_h.map_or(0.0, |s| s[i]);
        }
        own
    }
}

/// ⭐ A tela 2D vista pela porta da dobra através dos ajustes ([`crate::layers::fold_relief_through`]).
pub(super) struct Amostras<'f, 'a> {
    pub(super) f: &'f ReliefFields<'a>,
    pub(super) mascaras: std::collections::BTreeMap<crate::layers::LayerId, Vec<f32>>,
}

impl Amostras<'_, '_> {
    fn camada(&self, id: crate::layers::LayerId) -> Option<&ReliefLayer<'_>> {
        self.f.layers.iter().find(|l| l.id == id)
    }
}

impl crate::layers::ReliefSamples for Amostras<'_, '_> {
    fn len(&self) -> usize {
        self.f.width * self.f.height
    }
    fn has(&self, id: crate::layers::LayerId) -> bool {
        self.camada(id).is_some()
    }
    fn at(&self, id: crate::layers::LayerId, i: usize) -> (f32, f32) {
        self.camada(id).map_or((0.0, 0.0), |l| {
            (self.f.own_at(l, i), self.f.layer_cover_at(l, i))
        })
    }
    fn mask(&self, id: crate::layers::LayerId) -> Option<Vec<f32>> {
        self.mascaras.get(&id).cloned()
    }
}

/// Um passo do `over` do material com o peso `a > 0`: a tinta opaca COPIA (sem o lerp de 7 canais — mede-se
/// igual, a dobra é limitada pela banda do plano de 7 bytes, não pela aritmética); a translúcida mistura.
#[inline]
fn sobre(m: &mut [f32; 7], src: MaterialBytes, a: f32) {
    if a >= 1.0 {
        *m = src.map(f32::from);
        return;
    }
    for (c, v) in m.iter_mut().enumerate() {
        *v += (f32::from(src[c]) - *v) * a;
    }
}

/// A espessura de tinta que ENCHE o dente do papel, em fracções da altura dele ([`ReliefFields::height_at`]): a tinta tão
/// alta como o dente enche-o. Sonda 08/11: a pilha do Dabber do Wet não chega a este plano (o corpo dela vive na grade do
/// Wet), por isso a tinta do Wet esconde o papel pela marca da camada, não por aqui.
const CORPO_QUE_ENCHE: f32 = 1.0;
