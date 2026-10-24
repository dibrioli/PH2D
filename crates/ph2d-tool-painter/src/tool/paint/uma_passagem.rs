//! **O nível de UMA passagem do traço** — o que uma passagem DESTE pincel deposita, medido até o traço percorrer um
//! diâmetro de dab, e daí FIXO: a sobreposição do traço consigo (os cantos, a volta, esfregar) nunca o sobe. Dois
//! leitores, a mesma lei: a fronteira e a molhada da orla ([`super::traco_plano::PlanoDoTraco`]) e o alvo da mancha
//! do Solid ([`super::solid_deposit`]).
//!
//! ⭐ **Com grão, o nível é o do grão MÉDIO** (bug #58, 6.ª volta): o máximo da cobertura no 1.º diâmetro lia o pico
//! do grão que calhava no começo (Noise grosso Tiled: o patamar do Wet a 0,77–1,05 do miolo conforme o sítio).
//! Agora, em cada centro de dab do 1.º diâmetro, a cobertura `v` divide-se pelo grão `g` que os dabs lhe aplicaram
//! (`grain_at`, a mesma porta do carimbo) e multiplica-se pelo grão médio `ḡ` da textura: `v·ḡ/g` (o Flow como
//! taxa: `v = g·tecto·S`, exacto) ou `1 − (1 − v)^(ḡ/g)` (o Accumulate que empilha, Beer–Lambert). Sem grão, ou
//! num mapa que o carimbo sorteia (Random) ou recorta (Stencil), o máximo de sempre.

use ph2d_painter_brush::Dab;
use ph2d_painter_brush::texture::TextureMapping;

/// Os pontos da média do grão (sequência R2, sem aliasing com o período do grão).
const AMOSTRAS_DO_GRAO: usize = 4096;
/// O lado (px) do quadrado onde se mede a média de um grão preso à tela.
const LADO_DA_MEDIA_TILED: f32 = 4096.0;
/// Abaixo deste grão o texel não se divide (o `u8` da cobertura amplificado).
const GRAO_MINIMO: f32 = 0.2;

#[derive(Clone, Debug, Default)]
pub(crate) struct UmaPassagem<T = f32> {
    /// O nível (a cobertura: `0..1` no plano, `0..255` no `stroke_mask`).
    nivel: T,
    /// O nível ficou fixo (não sobe mais).
    fixo: bool,
    /// O comprimento (px) do caminho dos centros dos dabs até agora, e o último centro.
    percurso: f32,
    ultimo_centro: Option<[f32; 2]>,
    /// O maior diâmetro de dab até agora (px).
    diametro: f32,
    /// Os dabs do 1.º diâmetro (só enquanto aberto): onde se lê o nível e o grão que cada texel levou.
    dabs: Vec<Dab>,
    /// O grão médio `ḡ` da textura (medido uma vez por traço).
    grao_medio: Option<f32>,
}

impl<T: Copy + PartialOrd> UmaPassagem<T> {
    /// O nível de uma passagem agora (`0` antes da primeira observação).
    #[must_use]
    pub(super) fn nivel(&self) -> T {
        self.nivel
    }

    /// O nível ainda pode subir ([`Self::observa`] lê alguma coisa)?
    #[must_use]
    pub(super) fn aberto(&self) -> bool {
        !self.fixo
    }

    /// Soma ao percurso os centros de `dabs`, guarda o maior diâmetro e, enquanto aberto, os dabs.
    pub(super) fn percorre(&mut self, dabs: &[Dab]) {
        for d in dabs {
            if let Some(u) = self.ultimo_centro {
                self.percurso += (d.center[0] - u[0]).hypot(d.center[1] - u[1]);
            }
            self.ultimo_centro = Some(d.center);
            self.diametro = self.diametro.max(2.0 * d.radius_px);
        }
        if !self.fixo {
            self.dabs.extend_from_slice(dabs);
        }
    }

    /// Sobe o nível para `v` enquanto não está fixo; `true` se subiu.
    pub(super) fn observa(&mut self, v: T) -> bool {
        if self.fixo || v <= self.nivel {
            return false;
        }
        self.nivel = v;
        true
    }

    /// Um diâmetro percorrido: o eixo já recebeu a passagem inteira, e o nível fica.
    pub(super) fn fixa_se_percorreu(&mut self) {
        self.fixo |= self.diametro > 0.0 && self.percurso >= self.diametro;
        if self.fixo {
            self.dabs = Vec::new();
        }
    }
}

impl crate::tool::PainterTool {
    /// O grão do pincel entra na leitura do nível? (Tiled e View: o grão de cada texel é reproduzível.)
    fn grao_que_o_nivel_le(&self) -> bool {
        let b = &self.paint.brush;
        b.grain_samples(self.paint.texture_image.is_some())
            && matches!(
                b.texture.mapping,
                TextureMapping::Tiled | TextureMapping::ViewPlane
            )
    }

    /// O grão que o dab `d` aplica ao texel `(x, y)` — a base dele como o carimbo a tira (o sorteio não entra).
    fn grao_do_dab(&self, d: &Dab, x: i64, y: i64) -> f32 {
        let b = &self.paint.brush;
        let dims = [self.source_size.0 as f32, self.source_size.1 as f32];
        let base = ph2d_painter_brush::texture::dab_basis(
            &b.texture,
            &mut 0u64,
            dims,
            b.dab_footprint_of(d),
        );
        let imagem = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        ph2d_painter_brush::dab::grain_at(b, &base, imagem.as_ref(), x, y, d.center, d.radius_px)
    }

    /// ⭐ **O grão médio `ḡ`** da textura: num grão preso à tela (Tiled) a média num quadrado de
    /// [`LADO_DA_MEDIA_TILED`] px; num grão que anda com o dab (View) a média sob a ponta do dab `d`, pesada pelo
    /// falloff.
    fn grao_medio(&self, d: &Dab) -> f32 {
        let b = &self.paint.brush;
        let tiled = b.texture.mapping == TextureMapping::Tiled;
        let r = d.radius_px.max(0.5);
        // R2 (Roberts): o par de irracionais da razão plástica.
        let (a1, a2) = (0.754_877_7_f32, 0.569_840_3_f32);
        let (mut soma, mut peso) = (0.0f64, 0.0f64);
        for k in 0..AMOSTRAS_DO_GRAO {
            let (u, v) = ((0.5 + a1 * k as f32).fract(), (0.5 + a2 * k as f32).fract());
            let (x, y, w) = if tiled {
                (u * LADO_DA_MEDIA_TILED, v * LADO_DA_MEDIA_TILED, 1.0)
            } else {
                let (dx, dy) = ((2.0 * u - 1.0) * r, (2.0 * v - 1.0) * r);
                let t = dx.hypot(dy) / r;
                if t >= 1.0 {
                    continue;
                }
                (d.center[0] + dx, d.center[1] + dy, b.falloff_weight(t))
            };
            let g = self.grao_do_dab(d, x.floor() as i64, y.floor() as i64);
            soma += f64::from(w * g);
            peso += f64::from(w);
        }
        if peso > 0.0 {
            (soma / peso) as f32
        } else {
            1.0
        }
    }

    /// ⭐ **O nível de uma passagem com o grão no seu valor MÉDIO** (`None` sem grão que se leia, ou sem texel que
    /// se divida): o máximo, nos centros dos dabs do 1.º diâmetro, da cobertura `v` (de `cobertura`, pelo índice do
    /// texel) levada do grão `g` que os dabs lhe aplicaram ao grão médio `ḡ`.
    pub(super) fn nivel_no_grao_medio<T: Copy + PartialOrd>(
        &self,
        p: &mut UmaPassagem<T>,
        cobertura: impl Fn(usize) -> f32,
    ) -> Option<f32> {
        if !self.grao_que_o_nivel_le() || p.dabs.is_empty() {
            return None;
        }
        let medio = match p.grao_medio {
            Some(g) => g,
            None => *p.grao_medio.insert(self.grao_medio(&p.dabs[0])),
        };
        let b = &self.paint.brush;
        // O Flow como taxa até o tecto (`cover_add`): `v = g·tecto·S` é linear no grão; o Accumulate empilha.
        let linear = self.cobertura_tapada(b);
        let (w, h) = (self.source_size.0 as i64, self.source_size.1 as i64);
        let mut melhor: Option<f32> = None;
        for c in &p.dabs {
            let (x, y) = (c.center[0].floor() as i64, c.center[1].floor() as i64);
            if x < 0 || y < 0 || x >= w || y >= h {
                continue;
            }
            let v = cobertura((y * w + x) as usize);
            if v <= 0.0 {
                continue;
            }
            // O grão do texel: a média do de cada dab que o alcança, pesada pelo falloff (Tiled: o mesmo em todos).
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let (mut soma, mut peso) = (0.0f32, 0.0f32);
            for d in &p.dabs {
                let t = (px - d.center[0]).hypot(py - d.center[1]) / d.radius_px.max(0.5);
                if t < 1.0 {
                    let k = b.falloff_weight(t);
                    soma += k * self.grao_do_dab(d, x, y);
                    peso += k;
                }
            }
            if peso <= 0.0 || soma / peso < GRAO_MINIMO {
                continue;
            }
            let razao = medio / (soma / peso);
            let n = if linear {
                v * razao
            } else {
                1.0 - (1.0 - v.min(1.0)).powf(razao)
            };
            melhor = Some(melhor.map_or(n, |m: f32| m.max(n)));
        }
        melhor.map(|n| n.clamp(0.0, 1.0))
    }

    /// Depois de um lote do traço com Solid: o lote anda o percurso, e enquanto o nível está aberto ele sobe ao
    /// nível do `stroke_mask` (a cobertura do traço sob o tecto) que o lote escreveu.
    pub(super) fn observa_a_passagem_do_solid(&mut self, dabs: &[Dab]) {
        let w = self.source_size.0 as usize;
        let mut p = std::mem::take(&mut self.paint.solid_passagem);
        p.percorre(dabs);
        if p.aberto() && self.paint.stroke_mask.len() == w * self.source_size.1 as usize {
            let mask = &self.paint.stroke_mask;
            let nivel = match self.nivel_no_grao_medio(&mut p, |i| f32::from(mask[i]) / 255.0) {
                Some(n) => Some((n * 255.0 + 0.5) as u8),
                None => self.tiled_chord_region(dabs).map(|r| {
                    let (x0, y0) = (r.x as usize, r.y as usize);
                    (y0..y0 + r.h as usize)
                        .flat_map(|y| mask[y * w + x0..y * w + x0 + r.w as usize].iter())
                        .copied()
                        .max()
                        .unwrap_or(0)
                }),
            };
            if let Some(n) = nivel {
                p.observa(n);
            }
        }
        p.fixa_se_percorreu();
        self.paint.solid_passagem = p;
    }

    /// O nível de uma passagem do traço com Solid (`0..255`; `None` antes de o traço o medir).
    #[must_use]
    pub(super) fn passagem_do_solid(&self) -> Option<u32> {
        let n = self.paint.solid_passagem.nivel();
        (n > 0).then_some(u32::from(n))
    }
}
