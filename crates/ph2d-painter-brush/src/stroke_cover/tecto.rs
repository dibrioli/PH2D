//! O TEXEL do tecto do traço (B29.1, decisão 3): `u8` = a lei que shipou, ao bit; [`TexelLargo`] = a cobertura em
//! `u16` + o pré-traço RGBA guardado no 1.º toque, e a tela RECOMPOSTA `pré·(1 − m) + cor·m` (um arredondamento
//! por texel em vez de um por dab — o Flow baixo chega ao alvo). Medição: `docs/Painter/missao_4k/BACKLOG.md` B29.1.

/// Um texel da máscara do traço. `grava` devolve o pré-traço quando o texel sabe recompor.
pub trait Tecto: Copy + Send + Sync {
    fn m(self) -> f32;
    fn grava(&mut self, m_novo: f32, px: [u8; 4]) -> Option<[u8; 4]>;
}

impl Tecto for u8 {
    #[inline(always)]
    fn m(self) -> f32 {
        f32::from(self) / 255.0
    }
    #[inline(always)]
    fn grava(&mut self, m_novo: f32, _px: [u8; 4]) -> Option<[u8; 4]> {
        *self = (m_novo * 255.0 + 0.5) as u8;
        None
    }
}

/// A cobertura do traço em `u16` e o pré-traço (capturado quando o texel sai de `m = 0`): 6 bytes por texel.
/// `Zeroable`: o `Default` é o zero dos bits, e o mapa da tela nasce de `bytemuck::zeroed_vec` (páginas do SO,
/// como o `u8`) — `vec![TexelLargo::default(); n]` escrevia ~50 MB a 4K no pen-down (+8 ms, medido 11/11).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, bytemuck::Zeroable)]
pub struct TexelLargo {
    pub m: u16,
    pub pre: [u8; 4],
}

impl TexelLargo {
    /// A cobertura no degrau `u8` dos leitores que pensam em bytes.
    #[must_use]
    pub fn m8(self) -> u8 {
        ((u32::from(self.m) * 255 + 32767) / 65535) as u8
    }
}

impl Tecto for TexelLargo {
    #[inline(always)]
    fn m(self) -> f32 {
        f32::from(self.m) / 65535.0
    }
    #[inline(always)]
    fn grava(&mut self, m_novo: f32, px: [u8; 4]) -> Option<[u8; 4]> {
        if self.m == 0 {
            self.pre = px;
        }
        self.m = (m_novo.clamp(0.0, 1.0) * 65535.0 + 0.5) as u16;
        Some(self.pre)
    }
}

/// `pré·(1 − m) + cor·m` em PRÉ-MULTIPLICADO (Mix, alfa reto à entrada e à saída): o que a source-over por dab
/// telescopa para, sem os arredondamentos intermédios.
#[must_use]
pub(crate) fn recompoe(pre: [u8; 4], cor: [f32; 3], m: f32) -> [f32; 4] {
    let pa = f32::from(pre[3]) / 255.0;
    let ao = pa * (1.0 - m) + m;
    if ao <= 0.0 {
        return [0.0; 4];
    }
    let c = |k: usize| (f32::from(pre[k]) / 255.0 * pa * (1.0 - m) + cor[k] * m) / ao;
    [c(0), c(1), c(2), ao]
}

/// [`recompoe`] já no byte da tela (o mesmo `encode` do carimbo): para quem escreve o tecto largo fora do dab
/// (a mancha do Solid).
#[must_use]
pub fn recompoe_bytes(pre: [u8; 4], cor: [f32; 3], m: f32) -> [u8; 4] {
    recompoe(pre, cor, m).map(crate::dab::encode)
}
