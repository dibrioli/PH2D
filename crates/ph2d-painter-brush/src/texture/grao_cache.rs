//! ⭐ **A memória do grão Tiled** (Bug #84): com o mapeamento Tiled o [`super::sample`] é função SÓ do píxel da tela
//! e de invariantes do dab (as definições, a base `u`/`v`/`jitter`, a área ligada) — nunca do centro nem do raio. Um
//! traço pousa ~90 dabs sobre cada píxel (o véu do Wet Wash Mix: 38 640 visitas por dab), e o Clouds é um fbm com
//! warp a cada visita. A memória guarda o valor da PRIMEIRA amostra de cada píxel e devolve-o às seguintes: o MESMO
//! `f32`, ao bit, porque é a mesma função pura sobre as mesmas entradas.
//!
//! Só nasce quando a mesma chave chega em dois dabs seguidos (um pincel cujo grão muda a cada dab nunca a aloca) e
//! aloca por blocos de 64×64 à medida que o traço os visita. Escrita concorrente das faixas é benigna: quem escreve
//! escreve o mesmo valor.

use super::{TexDabBasis, TextureKind, TextureMapping, TextureSettings};
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::sync::{Arc, Mutex, OnceLock};

const LADO: u32 = 64;
const VAZIO: u32 = u32::MAX; // um NaN: o `sample` devolve sempre um valor em `0..1`
/// Píxeis da tela acima dos quais a memória não nasce (o recurso: 4 bytes por píxel visitado).
const MAX_PX: u64 = 8192 * 8192;

#[derive(Clone, Copy, PartialEq)]
struct Chave {
    s: TextureSettings,
    u: [f32; 2],
    v: [f32; 2],
    jitter: [f32; 2],
    area: bool,
    w: u32,
    h: u32,
}

pub(crate) struct GraoCache {
    chave: Chave,
    bx: u32,
    blocos: Box<[OnceLock<Box<[AtomicU32]>>]>,
}

struct Estado {
    vista: Option<Chave>,
    atual: Option<Arc<GraoCache>>,
}

static ESTADO: Mutex<Estado> = Mutex::new(Estado {
    vista: None,
    atual: None,
});

/// Se a amostra deste grão é função só do píxel (a premissa da memória).
fn so_do_pixel(s: &TextureSettings) -> bool {
    s.is_active()
        && matches!(s.mapping, TextureMapping::Tiled)
        && !s.flow
        && !s.kind.samples_in_pixels()
        && s.kind != TextureKind::Image
}

/// A memória para o dab de base `b` numa tela `w`×`h`, ou `None` (grão que depende do dab, tela grande demais, ou a
/// chave ainda não se repetiu).
pub(crate) fn para_o_dab(
    s: &TextureSettings,
    b: &TexDabBasis,
    w: u32,
    h: u32,
) -> Option<Arc<GraoCache>> {
    if !so_do_pixel(s) || u64::from(w) * u64::from(h) > MAX_PX || w == 0 || h == 0 {
        return None;
    }
    let chave = Chave {
        s: *s,
        u: b.u,
        v: b.v,
        jitter: b.jitter,
        area: super::pela_area::ligada(),
        w,
        h,
    };
    let mut e = ESTADO
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(c) = e.atual.as_ref().filter(|c| c.chave == chave) {
        return Some(Arc::clone(c));
    }
    if e.vista != Some(chave) {
        e.vista = Some(chave);
        return None;
    }
    let bx = w.div_ceil(LADO);
    let n = (bx * h.div_ceil(LADO)) as usize;
    let c = Arc::new(GraoCache {
        chave,
        bx,
        blocos: (0..n).map(|_| OnceLock::new()).collect(),
    });
    e.atual = Some(Arc::clone(&c));
    Some(c)
}

impl GraoCache {
    /// O grão no píxel `(px, py)`: o guardado, ou `amostra()` guardada.
    #[inline]
    pub(crate) fn em(&self, px: i64, py: i64, amostra: impl FnOnce() -> f32) -> f32 {
        let (Ok(x), Ok(y)) = (u32::try_from(px), u32::try_from(py)) else {
            return amostra();
        };
        if x >= self.chave.w || y >= self.chave.h {
            return amostra();
        }
        let bloco = self.blocos[((y / LADO) * self.bx + x / LADO) as usize]
            .get_or_init(|| (0..LADO * LADO).map(|_| AtomicU32::new(VAZIO)).collect());
        let cel = &bloco[((y % LADO) * LADO + x % LADO) as usize];
        let guardado = cel.load(Relaxed);
        if guardado != VAZIO {
            return f32::from_bits(guardado);
        }
        let v = amostra();
        cel.store(v.to_bits(), Relaxed);
        v
    }
}
