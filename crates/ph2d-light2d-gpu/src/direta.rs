//! **C3 — a luz DIRETA das lâmpadas**, lado CPU: quais emissores da cena vão pelo caminho direto
//! (`c3_direta.wgsl`) e como sobem à placa.
//!
//! As cascatas custam o mesmo com 1 ou 10 000 emissores; o caminho direto custa POR LÂMPADA. ⇒ as
//! [`LAMPADAS_DIRETAS_MAX`] lâmpadas mais fortes (as `Luz`: discos e pontos) vão por ele — borda
//! reta e penumbra da geometria em qualquer nitidez, de qualquer tamanho —, e o resto fica nas
//! cascatas como antes. Os sprites que brilham (polígonos) ficam sempre nas cascatas.

use bytemuck::{Pod, Zeroable};
use ph2d_light2d_ecs::{CenaDeLuz, Emissor};

use crate::config::Grade;

/// O teto de lâmpadas pelo caminho direto (as mais fortes). Os recursos: o TEMPO DE PLACA e o CÓDIGO
/// da visibilidade nas sondas (8 × u8 num `Rg32Uint`). Medido na W5a (`medida_luz2d_w5a`, 1920×1080,
/// RTX 5060 Ti, `smoke`, `load` ~1–2, o padrão com a captura fina de ½ px e o preenchimento), nível
/// Baixo: 1,73 ms sem lâmpada · 4,54 com 6 · 6,10 com 8 (as duas últimas, junto à parede, custam 0,78
/// cada) ⇒ `1,73 + 0,546·N ≤ 5,56` (o orçamento do governador, ⅓ do quadro a 60 Hz) dá N ≤ 7,0. Acima
/// disto as mais fracas ficam nas cascatas (e o Inspector avisa delas). Era 6 na W4b (0,686 ms por
/// lâmpada medidos sob carga; a mesma cena hoje dá 0,546 na W4b e na W5a) e 8 antes da grade fina.
pub const LAMPADAS_DIRETAS_MAX: usize = 7;

/// Uma lâmpada como o `c3_direta.wgsl` a lê (32 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct Lampada {
    /// Em texels finos.
    pub centro: [f32; 2],
    /// Em texels finos; `0` = ponto.
    pub raio: f32,
    pub inv_alcance: f32,
    /// rgb: a radiância do disco, ou a intensidade do ponto.
    pub radiancia: [f32; 4],
}

/// O buffer das lâmpadas: um cabeçalho (`n`) e o teto de itens.
pub(crate) const BYTES_DO_BUFFER: u64 =
    16 + (LAMPADAS_DIRETAS_MAX * std::mem::size_of::<Lampada>()) as u64;

/// A força de uma lâmpada para a escolha das diretas: a fluência a 1 m no canal mais forte.
fn forca(e: &Emissor) -> Option<f32> {
    let max = |c: &[f32; 3]| c.iter().copied().fold(0.0_f32, f32::max);
    match e {
        Emissor::Disco {
            raio, radiancia, ..
        } => Some(max(radiancia) * 2.0 * raio),
        Emissor::Ponto { intensidade, .. } => Some(max(intensidade)),
        Emissor::Poligono { .. } => None,
    }
}

/// ⭐ **Quais emissores vão pelo caminho direto**: os discos e pontos (as `Luz`), os
/// [`LAMPADAS_DIRETAS_MAX`] mais fortes; empate pela ordem da cena (a da coleta, estável).
#[must_use]
pub fn lampadas_diretas(cena: &CenaDeLuz) -> Vec<usize> {
    let mut v: Vec<(f32, usize)> = cena
        .emissores
        .iter()
        .enumerate()
        .filter_map(|(i, e)| forca(e).map(|f| (f, i)))
        .collect();
    v.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    v.truncate(LAMPADAS_DIRETAS_MAX);
    let mut idx: Vec<usize> = v.into_iter().map(|(_, i)| i).collect();
    idx.sort_unstable();
    idx
}

fn inverso_do_alcance(alcance_m: f32) -> f32 {
    if alcance_m.is_infinite() {
        0.0
    } else {
        1.0 / alcance_m
    }
}

/// O conteúdo do buffer para `diretas` na grade `g`: o cabeçalho e as lâmpadas.
pub(crate) fn bytes_na_grade(cena: &CenaDeLuz, diretas: &[usize], g: &Grade) -> Vec<u8> {
    let itens: Vec<Lampada> = diretas
        .iter()
        .filter_map(|&i| {
            let (centro, raio, cor, alcance) = match cena.emissores.get(i)? {
                Emissor::Disco {
                    centro,
                    raio,
                    radiancia,
                    alcance_m,
                } => (*centro, *raio, *radiancia, *alcance_m),
                Emissor::Ponto {
                    centro,
                    intensidade,
                    alcance_m,
                } => (*centro, 0.0, *intensidade, *alcance_m),
                Emissor::Poligono { .. } => return None,
            };
            let c = g.em_texels(centro);
            #[allow(clippy::cast_possible_truncation)]
            Some(Lampada {
                centro: [c[0] as f32, c[1] as f32],
                raio: (f64::from(raio) / g.texel_m) as f32,
                inv_alcance: inverso_do_alcance(alcance),
                radiancia: [cor[0], cor[1], cor[2], 0.0],
            })
        })
        .collect();
    let n = u32::try_from(itens.len()).unwrap_or(0);
    let mut out = Vec::with_capacity(BYTES_DO_BUFFER as usize);
    out.extend_from_slice(bytemuck::bytes_of(&[n, 0, 0, 0]));
    out.extend_from_slice(bytemuck::cast_slice(&itens));
    out
}
