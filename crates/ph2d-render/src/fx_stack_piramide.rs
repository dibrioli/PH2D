//! **A PIRÂMIDE do borrão** (ciclo 20 W3b do Motion, doc 129 §5.5 A8): um σ acima do que o kernel
//! percorre (`MAX_HALF / 3`) corre numa resolução dividida por `f = 2^m` — a imagem reduzida por
//! médias 2×2, o borrão lá em baixo, e o fecho amostra de volta por bilinear.
//!
//! Antes o kernel PARAVA em `MAX_HALF` e a régua do nó era o raio dessa saturação na janela medida
//! (`0,6` a 534 px de cena; numa janela 4K a mesma régua saturava em ~0,27). Agora o tecto é o da
//! placa. ⚠️ A variância que a redução (uma caixa de `f`: `(f² − 1)/12`) e a ampliação (uma tenda
//! de meia-largura `f`: `f²/6`) somam é DESCONTADA do σ de baixo — o total é a gaussiana pedida.
//! Gate: `the_pyramid_blur_of_a_step_is_the_normal_cdf` (o oráculo é a lei: o borrão gaussiano de
//! um degrau é `Φ(x/σ)`).

use crate::fx_stack::FxStackPass;
use crate::fx_stack::MAX_HALF;
use crate::fx_stack_plan::kernel_half;
use crate::fx_stack_res::{Globals, Tex, dispatch, write_at};
use ph2d_gpu::GpuContext;

/// O σ (px) acima do qual o borrão desce na pirâmide — o que o kernel percorre com suporte de 3σ.
#[allow(clippy::cast_precision_loss)]
pub(crate) const SIGMA_MAX: f32 = MAX_HALF as f32 / 3.0;

/// O tecto de reduções: a 12, uma imagem de 8K (7 680 px) já é 4 texels mais a folga — reduzir mais
/// não muda nada.
pub(crate) const MAX_NIVEIS: u32 = 12;

/// **O maior σ (px) que o borrão honra** — `SIGMA_MAX · 2^MAX_NIVEIS` = 131 072 px.
#[allow(clippy::cast_precision_loss)]
pub const SIGMA_ALCANCAVEL: f32 = SIGMA_MAX * (1u32 << MAX_NIVEIS) as f32;

/// Quantas reduções à metade o borrão de `sigma_px` pede (`0` = o caminho de sempre).
#[must_use]
pub(crate) fn niveis(sigma_px: f32) -> u32 {
    if sigma_px <= SIGMA_MAX {
        return 0;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let m = (sigma_px / SIGMA_MAX).log2().ceil() as u32;
    m.clamp(1, MAX_NIVEIS)
}

/// O σ do borrão na resolução de baixo (`f = 2^niveis`), descontadas a caixa e a tenda.
#[must_use]
pub(crate) fn sigma_baixo(sigma_px: f32, f: u32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let f = f as f32;
    let resto = sigma_px * sigma_px - (f * f - 1.0) / 12.0 - f * f / 6.0;
    resto.max(0.0).sqrt() / f
}

/// As dimensões do nível `k` (`f = 2^k`): a imagem reduzida, com UMA célula de cada lado — o
/// borrado continua FORA da imagem, e a volta lê-o lá (a meia célula da borda não tem outro vizinho).
#[must_use]
pub(crate) fn dims_do_nivel(dims: [u32; 2], k: u32) -> [u32; 2] {
    let f = 1u32 << k;
    [dims[0].div_ceil(f) + 2, dims[1].div_ceil(f) + 2]
}

/// O alcance (px) do borrão de `sigma_px` — o suporte de 3σ, mais a tenda da ampliação.
#[must_use]
pub(crate) fn alcance(sigma_px: f32) -> u32 {
    let m = niveis(sigma_px);
    if m == 0 {
        return crate::fx_stack_plan::kernel_half(sigma_px);
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let tres = (3.0 * sigma_px).ceil() as u32;
    tres + (1 << m)
}

/// Os três passes da pirâmide. Concatenado DEPOIS do `FX_STACK_MID_WGSL` (consome o `dst`, o
/// `blur_v_at` e o `compose_blur`).
pub(crate) const PIRAMIDE_WGSL: &str = r"
// `aux = (largura de entrada, altura de entrada, 0, é o 1.º nível)`: a média 2×2 do nível de cima.
// O texel `j` do nível `k` cobre os pixels `[(j − 1)·2^k, j·2^k)` — uma célula de folga antes da
// imagem —, logo lê os texels `2j − 2, 2j − 1` da imagem (1.º nível) ou `2j − 1, 2j` do nível de
// cima (que também tem a folga).
// O 1.º nível lê a IMAGEM e passa-a pela `source_of` (o alfa invertido dos de dentro); os seguintes
// já leem a fonte. Fora da entrada vale `source_of(transparente)`, a mesma borda do borrão.
fn tap_entrada(x: i32, y: i32) -> vec4<f32> {
    if (x < 0 || y < 0 || x >= g.aux.x || y >= g.aux.y) { return source_of(vec4<f32>(0.0)); }
    let v = textureLoad(t0, vec2<i32>(x, y), 0);
    return select(v, source_of(v), g.aux.w == 1);
}

@compute @workgroup_size(8, 8, 1)
fn cs_halve(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    let p = 2 * vec2<i32>(i32(id.x), i32(id.y)) - select(1, 2, g.aux.w == 1);
    let s = tap_entrada(p.x, p.y) + tap_entrada(p.x + 1, p.y)
        + tap_entrada(p.x, p.y + 1) + tap_entrada(p.x + 1, p.y + 1);
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), s * 0.25);
}

// O passe vertical lá em baixo: só o borrado, sem o fecho (o fecho é na resolução da imagem).
@compute @workgroup_size(8, 8, 1)
fn cs_blur_v_only(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), blur_v_at(id));
}

// `aux = (largura de baixo, altura de baixo, f, 0)`: o borrado de baixo amostrado por bilinear no
// centro do pixel (`(p + ½)/f − ½`), e o MESMO fecho do passe vertical de sempre.
fn tap_baixo(x: i32, y: i32) -> vec4<f32> {
    if (x < 0 || y < 0 || x >= g.aux.x || y >= g.aux.y) { return source_of(vec4<f32>(0.0)); }
    return textureLoad(t1, vec2<i32>(x, y), 0);
}

@compute @workgroup_size(8, 8, 1)
fn cs_op_up(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= g.dims.x || id.y >= g.dims.y) { return; }
    // ⚠️ `+ ½` e não `− ½`: a grelha de baixo começa uma célula ANTES da imagem. Sem a folga, a
    // meia célula da borda lia «fora» como transparente quando o borrado continua lá — medido: a
    // 1.ª linha e a 1.ª coluna saíam a metade (81 contra 128 níveis, σ = 100); presa à borda, o
    // declive de Φ dava 2,7 níveis a σ = 300.
    let fim = vec2<f32>(f32(g.aux.x - 1), f32(g.aux.y - 1));
    let u = clamp((vec2<f32>(f32(id.x), f32(id.y)) + 0.5) / f32(g.aux.z) + 0.5, vec2<f32>(0.0), fim);
    let b0 = floor(u);
    let fr = u - b0;
    let x = i32(b0.x);
    let y = i32(b0.y);
    let top = mix(tap_baixo(x, y), tap_baixo(x + 1, y), fr.x);
    let bot = mix(tap_baixo(x, y + 1), tap_baixo(x + 1, y + 1), fr.x);
    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), compose_blur(id, mix(top, bot, fr.y)));
}
";

/// Os globals da PIRÂMIDE (`niveis` reduções · H · V · a volta) a partir de `slot`; devolve o slot
/// seguinte. O deslocamento é dividido pelo fator (o A7 tornou-o fraccionário: cabe).
pub(crate) fn escreve_piramide(
    blob: &mut [u8],
    mut slot: usize,
    g: Globals,
    dims: [u32; 2],
    niveis: u32,
) -> usize {
    let f = 1u32 << niveis;
    let mut d = dims;
    for k in 1..=niveis {
        let entrada = d;
        d = crate::fx_stack_piramide::dims_do_nivel(dims, k);
        #[allow(clippy::cast_possible_wrap)]
        let aux = [entrada[0] as i32, entrada[1] as i32, 0, i32::from(k == 1)];
        write_at(
            blob,
            slot,
            &Globals {
                dims: d,
                aux,
                off_x: 0.0,
                off_y: 0.0,
                ..g
            },
        );
        slot += 1;
    }
    #[allow(clippy::cast_precision_loss)]
    let fator = f as f32;
    let baixo = sigma_baixo(g.band, f).max(1e-4);
    let de_baixo = Globals {
        dims: d,
        half: kernel_half(baixo),
        inv_two_sigma2: 1.0 / (2.0 * baixo * baixo),
        aux: [0, 0, 1, 0],
        ..g
    };
    write_at(
        blob,
        slot,
        &Globals {
            off_x: g.off_x / fator,
            off_y: 0.0,
            ..de_baixo
        },
    );
    write_at(
        blob,
        slot + 1,
        &Globals {
            off_x: 0.0,
            off_y: g.off_y / fator,
            ..de_baixo
        },
    );
    #[allow(clippy::cast_possible_wrap)]
    let aux = [d[0] as i32, d[1] as i32, f as i32, 0];
    write_at(blob, slot + 2, &Globals { aux, ..g });
    slot + 3
}

/// A entrada da pirâmide: a imagem, a saída do degrau, as dimensões e os grupos do despacho.
pub(crate) type Rede<'a> = (
    &'a wgpu::TextureView,
    &'a wgpu::TextureView,
    [u32; 2],
    [u32; 2],
);

impl FxStackPass {
    /// O despacho da PIRÂMIDE: as `niveis` reduções (a cadeia de baixo vive em `work[2]`/`work[3]`,
    /// do tamanho da imagem: cabe), o H e o V lá em baixo, e a volta com o fecho. Devolve o slot
    /// seguinte — em lockstep com o [`escreve_piramide`].
    pub(crate) fn despacha_piramide(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        work: &[Tex; 4],
        (input, saida, [w, h], [gx, gy]): Rede<'_>,
        niveis: u32,
        mut slot: u32,
    ) -> u32 {
        let mut a = 2usize;
        let mut d = [w, h];
        for k in 1..=niveis {
            d = dims_do_nivel([w, h], k);
            let (de, para) = if k == 1 {
                (input, 2)
            } else {
                (&work[a].view, 5 - a)
            };
            let bg = self.bind(gpu, &self.bgl_mid, de, de, &work[para].view);
            let (x, y) = (d[0].div_ceil(8), d[1].div_ceil(8));
            dispatch(encoder, &self.pipeline_halve, &bg, slot, x, y);
            slot += 1;
            a = para;
        }
        let b = 5 - a;
        let (x, y) = (d[0].div_ceil(8), d[1].div_ceil(8));
        let (va, vb) = (&work[a].view, &work[b].view);
        let bg_h = self.bind(gpu, &self.bgl_mid, va, va, vb);
        dispatch(encoder, &self.pipeline_h, &bg_h, slot, x, y);
        let bg_v = self.bind(gpu, &self.bgl_mid, vb, vb, va);
        dispatch(encoder, &self.pipeline_v_only, &bg_v, slot + 1, x, y);
        let bg_up = self.bind(gpu, &self.bgl_mid, input, va, saida);
        dispatch(encoder, &self.pipeline_up, &bg_up, slot + 2, gx, gy);
        slot + 3
    }
}
