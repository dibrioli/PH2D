//! **O lote publicado para quem tiver um DISPOSITIVO** — a metade do carimbo que mora no tool.
//!
//! # Por que o tool não tem device, e não deve ter
//!
//! O molde é o do `denoise_ml_with_progress` do editor de áudio: a crate pesada nunca alcança o
//! tool, e a ponte é do **shell**. Aqui isso vale duas vezes, porque a contenção corta nos dois
//! sentidos — nada de `wgpu` entra no `ph2d-tool-painter`, e **nada do Painter alcança o kernel**.
//! O que atravessa a fronteira é dado simples: uma região de bytes, uma TABELA e uma lista de
//! discos. É o que torna estruturalmente impossível o device ter opinião sobre a lei do falloff
//! ([[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]; ver
//! `docs/Painter/33_plano_gpu_do_carimbo.md` §2).
//!
//! # O predicado ESTREITA, e cada cláusula tem mecanismo
//!
//! A rota em banda ([`super::stamp_banded`]) já exclui Shape, Grain, imagem e o cap de Accumulate.
//! [`eligible`] tira mais três, e nenhuma por simetria — cada uma é uma lei que o kernel **não**
//! transcreve. Todo caso fora da lista cai na rota em banda, que é testada e continua sendo o
//! caminho de tudo o que não é o quente: **o modo de falha de um caso novo é lento, nunca errado.**

use ph2d_painter_brush::{BrushBlend, BrushSpec, Dab, DirtyRect, dab_write_bounds};

/// Nós da tabela do perfil — **medido, não escolhido** (gate `the_device_paints_what_the_cpu_paints`,
/// varredura com todo o resto igual): 1 024 REPROVA por dois níveis · 16 384 → 71 bytes · 65 536 →
/// 18 · 262 144 → 8, de 122 880. O joelho é aqui, e 256 KB cabem no L2.
pub(super) const LUT_NODES: usize = 65_536;

/// Um disco de pigmento já resolvido — dado simples, sem o preenchimento de alinhamento que só um
/// buffer de GPU precisa (esse mora na crate do device, que é quem sabe o que WGSL alinha).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceDab {
    pub center: [f32; 2],
    pub radius: f32,
    pub coverage: f32,
    pub color: [f32; 3],
    /// As duas LINHAS do mapa linear do footprint (flatten & rotate), avaliadas nos vetores da base.
    /// Um deform de dab É linear — a premissa é gate, não comentário.
    pub m0: [f32; 2],
    pub m1: [f32; 2],
}

/// O lote publicado: a região JÁ EXTRAÍDA (RGBA8 contíguo, `w · h · 4`), a tabela e os discos.
#[derive(Debug)]
pub struct DeviceStampJob<'a> {
    pub base: &'a [u8],
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub lut: &'a [f32],
    pub dabs: &'a [DeviceDab],
    pub preserve_alpha: bool,
}

/// A ponte que o shell instala. `None` — ou um `None` de volta — devolve o lote à rota em banda.
pub type DeviceStamp = Box<dyn Fn(&DeviceStampJob<'_>) -> Option<Vec<u8>> + Send>;

/// **Este pincel pode ser carimbado pelo device?**
///
/// ⚠️ As três cláusulas, com o mecanismo de cada uma:
///
/// - **`blend` tem de ser `Mix`.** Os outros 23 modos são 23 leis (`blend_rgb`), e traduzir todas
///   seria a segunda resposta que esta crate existe para não ter. O kernel transcreve UMA.
/// - **`pigment_mix` tem de ser zero.** Acima disso o `blend_over_pigment` faz um crossfade para uma
///   mistura RYB subtrativa — outra lei inteira, e ela lê o alfa do destino por texel.
/// - **Smooth Edges tem de estar desligado.** O AA do filme amostra a cadeia de silhueta NOVE vezes
///   por texel contra uma base deformada; ele não é um valor tabelável, é um passe.
///
/// ⚠️ **`deposits_height` NÃO está aqui, de propósito** — o filme (`film_coverage`) é uma função
/// escalar pura da silhueta, então ele entra na TABELA em [`build_lut`], e o depósito do Impasto
/// vem junto de graça. *Quem pode ser tabelado não precisa ser excluído.*
#[must_use]
pub fn eligible(brush: &BrushSpec) -> bool {
    matches!(brush.blend, BrushBlend::Mix)
        && brush.effective_pigment_mix() == 0.0
        && !brush.film_aa_wanted(false)
}

/// A tabela que sobe ao device — **preenchida com as funções que o produto usa**, na ordem que ele
/// usa: `film_coverage(deposits_height, falloff_weight(t))`, que é literalmente o `w` do
/// `stamp_band` com `shape = None` (o `silhouette_at` reduz ao falloff) e sem AA.
#[must_use]
pub fn build_lut(brush: &BrushSpec) -> Vec<f32> {
    let body = brush.deposits_height();
    #[allow(clippy::cast_precision_loss)]
    (0..LUT_NODES)
        .map(|i| {
            let t = i as f32 / (LUT_NODES - 1) as f32;
            ph2d_painter_brush::height::film_coverage(body, brush.falloff_weight(t))
        })
        .collect()
}

/// A tabela do traço, reconstruída só quando a lei que a define muda.
///
/// ⚠️ **A chave é a lei, não o pincel:** o perfil é função de `hardness`, do preset de `falloff` e
/// de o pincel depositar corpo — e nada disso muda dentro de um traço, enquanto `radius_px` e
/// `color` mudam a cada dab. Chavear pelo `BrushSpec` inteiro reconstruiria 65 536 nós por dab.
///
/// ⚠️ **`Falloff::Custom` NUNCA é cacheado**: a curva é editável e não entra na chave, então um
/// cache o congelaria no perfil de quando o artista abriu o card — *um cache que erra por não
/// perceber uma edição é pior que nenhum*.
#[derive(Debug, Default)]
pub(crate) struct LutCache {
    key: Option<(u32, u8, bool)>,
    table: Vec<f32>,
    /// Quantas vezes a tabela foi CONSTRUÍDA. Um `u32` que ninguém no produto lê, e é o único
    /// oráculo honesto do cache: *"a tabela mudou"* não separa **reconstruiu** de **reconstruiu e
    /// deu o mesmo**, e comparar endereços de `Vec` mede o alocador, não a decisão.
    builds: u32,
}

impl LutCache {
    /// A tabela deste pincel, construída se a lei mudou.
    pub(crate) fn get(&mut self, brush: &BrushSpec) -> &[f32] {
        let custom = matches!(brush.falloff, ph2d_painter_brush::Falloff::Custom);
        let key = (
            brush.hardness.to_bits(),
            brush.falloff as u8,
            brush.deposits_height(),
        );
        if custom || self.key != Some(key) {
            self.table = build_lut(brush);
            self.key = (!custom).then_some(key);
            self.builds += 1;
        }
        &self.table
    }

    /// Quantas construções aconteceram — o oráculo do gate do cache.
    #[cfg(test)]
    pub(crate) fn builds(&self) -> u32 {
        self.builds
    }
}

/// Os discos, resolvidos como o `stamp_dabs_per_pixel` os resolve.
///
/// ⚠️ O footprint é avaliado a partir do `brush`, **não** de uma cópia por-dab: o `footprint_deform`
/// lê só `dab_flatten` e `dab_angle_deg`, e o spec por-dab do laço serial sobrescreve apenas
/// `radius_px` e `color` — logo o mapa é o MESMO número. Um `BrushSpec` clonado por dab seria uma
/// cópia grande para chegar ao mesmo lugar.
#[must_use]
pub fn device_dabs(dabs: &[Dab], brush: &BrushSpec) -> Vec<DeviceDab> {
    dabs.iter()
        .map(|d| {
            let fp = brush.dab_footprint(brush.dab_rotor(d));
            let e0 = fp.apply([1.0, 0.0]);
            let e1 = fp.apply([0.0, 1.0]);
            DeviceDab {
                center: d.center,
                radius: d.radius_px,
                coverage: d.coverage,
                color: d.color,
                m0: [e0[0], e1[0]],
                m1: [e0[1], e1[1]],
            }
        })
        .collect()
}

/// A união dos retângulos de escrita declarados — a mesma porta que o lote em banda consulta.
fn bounds(dabs: &[Dab], w: u32, h: u32) -> Option<DirtyRect> {
    dabs.iter()
        .filter_map(|d| dab_write_bounds(d.center, d.radius_px, w, h))
        .fold(None, |acc: Option<DirtyRect>, r| {
            Some(acc.map_or(r, |a| {
                let x = a.x.min(r.x);
                let y = a.y.min(r.y);
                DirtyRect {
                    x,
                    y,
                    w: (a.x + a.w).max(r.x + r.w) - x,
                    h: (a.y + a.h).max(r.y + r.h) - y,
                }
            }))
        })
}

/// **Redundância mínima para o device valer** — visitas de texel por pixel de região.
///
/// ⚠️ **Derivada da tabela, não escolhida.** As duas rotas escalam com grandezas DIFERENTES: a
/// fronteira é ~linear na ÁREA DA REGIÃO (sobe e desce a mesma janela) e o carimbo da CPU é ~linear
/// nas VISITAS (`Σ` pegadas). A razão entre as duas — a redundância — é o que decide, e sem um piso
/// a rota do device **PERDE** exatamente onde ela parece mais atraente. Medido pela porta do artista
/// (`ph2d-paint-gpu/tests/measure_product_stamp.rs`, RTX, 4096², pincel r=155):
///
/// | figura | região | visitas | redundância | CPU | device | ganho |
/// |---|---|---|---|---|---|---|
/// | 300 | 1,36 M | 8,56 M | 6,3× | 7,21 ms | 2,75 ms | **2,62×** |
/// | 600 | 4,05 M | 17,04 M | 4,2× | 15,83 | 10,66 | **1,48×** |
/// | 1200 | 13,77 M | 34,00 M | 2,5× | 38,31 | 53,89 | **0,71×** |
/// | 1900 | 16,78 M | 5,63 M | 0,3× | 18,25 | 82,23 | **0,22×** |
///
/// Ajustando as duas retas: ~3 ns por pixel de região no device (conservador — o custo por pixel
/// PIORA com a região, 1,7 → 3,9 ns/px na varredura) contra ~1 ns por visita na CPU. O ponto de
/// virada cai entre 2,5× e 4,2×, e o piso fica na ponta ALTA de propósito: superestimar o device é o
/// erro seguro, porque ele manda o lote duvidoso para a rota que já shipa.
///
/// ⚠️ E a figura do report do artista tem redundância **9,9×** (17,3 M visitas sobre um bbox de
/// 1440×1216) — bem acima do piso, que é o que faz a wave alcançar o caso que a motivou.
pub(super) const MIN_REDUNDANCY: f32 = 4.0;

/// **Vale subir este lote?** — a porta ÚNICA, perguntada pelo produto para DECIDIR e pelo gate para
/// AFIRMAR (o padrão do `wants_bands`, um nível acima).
///
/// ⚠️ Ela pergunta o TRABALHO à mesma função que a rota em banda usa (`batch_work`): duas contas do
/// que um lote custa divergiriam, e cada rota decidiria por um número diferente.
pub(super) fn wants_device(dabs: &[Dab], w: u32, h: u32) -> bool {
    let Some(r) = bounds(dabs, w, h) else {
        return false;
    };
    let region = (r.w as usize) * (r.h as usize);
    if region == 0 {
        return false;
    }
    #[allow(clippy::cast_precision_loss)]
    let (work, region) = (
        super::stamp_banded::batch_work(dabs, w, h) as f32,
        region as f32,
    );
    work >= MIN_REDUNDANCY * region
}

/// Carimba o lote pelo device, devolvendo a região escrita — ou `None` para o chamador cair na rota
/// em banda.
///
/// ⚠️ **A região é a bbox, extraída para um buffer próprio, e isso foi MEDIDO contra a alternativa**
/// (`measure_boundary::the_wiring_choice_between_a_copy_and_full_width_rows`): linhas de largura
/// cheia dispensam a cópia — a fatia do canvas já é contígua — mas sobem a largura do CANVAS em vez
/// da largura da figura, e custam **7,70 ms contra 2,80**. A cópia dos dois sentidos é **0,18 ms**,
/// 6% da fronteira.
///
/// ⚠️ **O retângulo devolvido é a região INTEIRA, e é o número honesto:** o kernel escreve todo
/// texel dela (o não-tocado recebe a própria base de volta, byte-idêntica), então é exatamente onde
/// bytes foram escritos — que é a pergunta que o `declare_wrote` faz, e não *"onde a imagem
/// mudou"*. A rota da CPU devolve a união exata dos spans, menor; a diferença é o commit do undo
/// varrer uma janela um pouco maior, nunca tinta a menos.
// O irmão em banda leva o mesmo  pela mesma razão: são os argumentos que um carimbo tem.
#[allow(clippy::too_many_arguments)]
pub(super) fn stamp(
    bridge: &DeviceStamp,
    buf: &mut [u8],
    w: u32,
    h: u32,
    dabs: &[DeviceDab],
    lut: &[f32],
    src: &[Dab],
    preserve_alpha: bool,
) -> Option<DirtyRect> {
    let r = bounds(src, w, h)?;
    let (rw, rh) = (r.w as usize, r.h as usize);
    if rw == 0 || rh == 0 {
        return None;
    }
    let stride = (w as usize) * 4;
    let row = rw * 4;
    let x0 = (r.x as usize) * 4;
    let mut base = vec![0u8; rw * rh * 4];
    for (i, dst) in base.chunks_exact_mut(row).enumerate() {
        let s = (r.y as usize + i) * stride + x0;
        dst.copy_from_slice(&buf[s..s + row]);
    }
    let out = bridge(&DeviceStampJob {
        base: &base,
        x: r.x,
        y: r.y,
        w: r.w,
        h: r.h,
        lut,
        dabs,
        preserve_alpha,
    })?;
    if out.len() != base.len() {
        return None; // a ponte falhou; o lote volta para a rota em banda
    }
    for (i, s) in out.chunks_exact(row).enumerate() {
        let d = (r.y as usize + i) * stride + x0;
        buf[d..d + row].copy_from_slice(s);
    }
    Some(r)
}

impl crate::tool::PainterTool {
    /// **Instala a ponte para o dispositivo.** O shell chama isto uma vez, quando tem um device na
    /// mão; `None` a retira e todo lote volta para a CPU.
    ///
    /// ⚠️ **Isto não liga uma feature, entrega uma CAPACIDADE** — o que decide se um lote a usa é o
    /// predicado ([`eligible`]) mais a rota de falloff puro, avaliados por lote. Um pincel com
    /// Grain, um blend que não é o `Mix` ou o Smooth Edges do impasto seguem na CPU com a ponte
    /// instalada.
    pub fn set_device_stamp(&mut self, bridge: Option<DeviceStamp>) {
        self.device_stamp = bridge;
    }

    /// Se há uma ponte instalada — o que o arch-gate do shell pergunta.
    #[must_use]
    pub fn has_device_stamp(&self) -> bool {
        self.device_stamp.is_some()
    }
}

#[cfg(test)]
#[path = "stamp_device_tests.rs"]
mod tests;
