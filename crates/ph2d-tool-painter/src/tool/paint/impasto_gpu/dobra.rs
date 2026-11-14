//! **A dobra do relevo de uma janela, por faixas** — para o destino que o chamador der (o do app: os buffers de
//! envio mapeados da luz); filho de [`super`] (LOC cap).

use super::super::relief_fields::ReliefFields;
use super::ImpastoLamp;
use crate::tool::PainterTool;
use ph2d_painter_brush::material::MaterialBytes;
use rayon::prelude::*;

/// O corpo em byte e o material em dois texels de uma linha da dobra (B15.6: só aqui o corpo vira byte).
fn planos_da_linha(
    crow: &mut [u8],
    m0row: &mut [u8],
    m1row: &mut [u8],
    cob: &[f32],
    mats: &[MaterialBytes],
) {
    // Byte a byte e não `copy_from_slice`: este não se alinhava (um `memcpy` de 4 bytes da libc por canal, 18 % do
    // Ctrl+Z 4K, B15.6-cont). Os mesmos bytes.
    let texels = crow.iter_mut().zip(cob.iter()).zip(mats.iter());
    let planos = m0row
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(m1row.as_chunks_mut::<4>().0.iter_mut());
    for (((c, cb), m), (m0, m1)) in texels.zip(planos) {
        // Quantised the way the canvas already stores coverage, so the shader's unorm decode lands on the CPU's
        // `f32::from(u8) / 255.0` and not merely near it.
        *c = (cb * 255.0 + 0.5) as u8;
        (m0[0], m0[1], m0[2], m0[3]) = (m[0], m[1], m[2], m[3]);
        (m1[0], m1[1], m1[2], m1[3]) = (m[4], m[5], m[6], 255);
    }
}

impl PainterTool {
    /// **A dobra sem os planos** — o que a GPU precisa saber ANTES de reservar a memória de envio: a janela
    /// (já recortada à tela), se há plano do papel, as lâmpadas e a tabela. `None` nos mesmos sítios de
    /// [`Self::impasto_gpu_planes`]. A dobra em si é [`ImpastoDobra::dobrar_faixas`], para os destinos que o chamador
    /// der (o do app: as linhas dos buffers de envio mapeados, sem `Vec` de tela inteira nem `write_texture`).
    #[must_use]
    pub fn impasto_dobra(&self, region: (u32, u32, u32, u32)) -> Option<ImpastoDobra<'_>> {
        if !self.impasto_visible() {
            return None;
        }
        let (w, h) = self.source_size;
        if (w as usize) * (h as usize) == 0 {
            return None;
        }
        let (rx, ry, rw, rh) = region;
        let rx = rx.min(w);
        let ry = ry.min(h);
        let rw = rw.min(w - rx);
        let rh = rh.min(h - ry);
        if (rw as usize) * (rh as usize) == 0 {
            return None;
        }
        // B15.5: com Gaussiano/Nitidez a dobra é a da janela dilatada pela soma dos halos, não a da tela.
        let fields = self.impasto_fields_para(super::super::Region {
            x: rx,
            y: ry,
            w: rw,
            h: rh,
        })?;
        let rig = super::super::impasto_shade::Rig::new(&self.paint.impasto_rig)?;
        let realce_linear = rig.realce_linear();
        let lamps = rig.export_lamps();
        let lut = ph2d_painter_brush::material::SpecLut::get();
        Some(ImpastoDobra {
            width: w,
            height: h,
            region: (rx, ry, rw, rh),
            paper_body: fields.paper_body(),
            realce_linear,
            lamps,
            spec_lut: lut.table(),
            lut_width: ph2d_painter_brush::material::SPEC_LUT as u32,
            rough_levels: ph2d_painter_brush::material::ROUGH_LEVELS as u32,
            fields,
        })
    }
}

/// Uma linha da janela já dobrada, nos bytes dos planos ([`ImpastoPlanes`](super::ImpastoPlanes) diz o que cada um é); `papel` é `Some`
/// exatamente quando [`ImpastoDobra::com_papel`].
pub struct LinhaDobrada<'s> {
    pub relief: &'s [f32],
    pub cover: &'s [u8],
    pub mat0: &'s [u8],
    pub mat1: &'s [u8],
    pub papel: Option<&'s [f32]>,
}

/// A dobra do relevo de uma janela, pronta a escrever onde o chamador mandar ([`PainterTool::impasto_dobra`]).
pub struct ImpastoDobra<'a> {
    pub width: u32,
    pub height: u32,
    /// `(x, y, w, h)`, já recortada à tela — o que [`ImpastoPlanes::region`](super::ImpastoPlanes::region) diria.
    pub region: (u32, u32, u32, u32),
    pub paper_body: f32,
    /// B31.2 só no Wet Paint: o realce linear entre entradas da tabela (`false` = o truncado do main).
    pub realce_linear: bool,
    pub lamps: Vec<ImpastoLamp>,
    pub spec_lut: &'static [f32],
    pub lut_width: u32,
    pub rough_levels: u32,
    fields: ReliefFields<'a>,
}

impl ImpastoDobra<'_> {
    /// Há plano do papel (o substrato está ligado)?
    #[must_use]
    pub fn com_papel(&self) -> bool {
        self.fields.substrate.is_some()
    }

    /// Dobra a janela por FAIXAS de linhas, em paralelo: para cada faixa, `abre(linhas)` dá o destino dela (no fio
    /// que a dobra — não precisa de ser `Send`) e `escreve(destino, j, linha)` recebe cada linha `j` da faixa já
    /// dobrada num rascunho do fio (memória normal, quente na cache). O destino do app são as vistas mapeadas da faixa
    /// nos buffers de envio, que podem ser write-combining: só se escreve, em sequência.
    ///
    /// The samplers are the light's OWN, so a texel inside the window carries the identical bytes a full fold
    /// would have given it. **In parallel** (ADR-0109's exception): every texel is a pure function of `(x, y)` and
    /// the frozen `fields`, so rows are disjoint and the result is byte-identical by construction (4096²: 201,5 →
    /// 14,55 ms). B15.6: the row is the SAME fold as the CPU light's bands ([`ReliefFields::dobra_da_linha`]); só
    /// aqui o corpo vira byte. `linhas` são relativas à janela (`0..h`).
    pub fn dobrar_faixas<E>(
        &self,
        abre: impl Fn(std::ops::Range<u32>) -> E + Sync,
        escreve: impl Fn(&mut E, usize, LinhaDobrada<'_>) + Sync,
    ) {
        let (rx, ry, rw, rh) = self.region;
        let rww = rw as usize;
        let com_papel = self.com_papel();
        let fields = &self.fields;
        let init = || Rascunho {
            relief: vec![0f32; rww],
            cover: vec![0u8; rww],
            mat0: vec![0u8; rww * 4],
            mat1: vec![0u8; rww * 4],
            papel: vec![0f32; if com_papel { rww } else { 0 }],
            cob: vec![0f32; rww],
            mats: vec![[0u8; 7]; rww],
        };
        (0..rh.div_ceil(LINHAS_POR_FAIXA))
            .into_par_iter()
            .for_each_init(init, |r, faixa| {
                let y0 = faixa * LINHAS_POR_FAIXA;
                let linhas = y0..(y0 + LINHAS_POR_FAIXA).min(rh);
                let mut destino = abre(linhas.clone());
                for (j, row) in linhas.enumerate() {
                    let (y, x0) = (i64::from(ry + row), i64::from(rx));
                    if com_papel {
                        let pap = Some(&mut r.papel[..]);
                        fields.dobra_da_linha_com_papel(
                            y,
                            x0,
                            &mut r.relief,
                            &mut r.cob,
                            &mut r.mats,
                            pap,
                        );
                    } else {
                        fields.dobra_da_linha(y, x0, &mut r.relief, &mut r.cob, &mut r.mats);
                    }
                    planos_da_linha(&mut r.cover, &mut r.mat0, &mut r.mat1, &r.cob, &r.mats);
                    escreve(
                        &mut destino,
                        j,
                        LinhaDobrada {
                            relief: &r.relief,
                            cover: &r.cover,
                            mat0: &r.mat0,
                            mat1: &r.mat1,
                            papel: com_papel.then_some(&r.papel[..]),
                        },
                    );
                }
            });
    }
}

/// Linhas por tarefa do rayon: 4K = 135 faixas (o rayon equilibra), e uma vista mapeada por faixa e plano.
const LINHAS_POR_FAIXA: u32 = 16;

/// O rascunho de uma linha, um por fio do rayon.
struct Rascunho {
    relief: Vec<f32>,
    cover: Vec<u8>,
    mat0: Vec<u8>,
    mat1: Vec<u8>,
    papel: Vec<f32>,
    cob: Vec<f32>,
    mats: Vec<MaterialBytes>,
}
