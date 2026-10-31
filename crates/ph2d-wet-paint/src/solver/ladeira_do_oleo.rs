//! ⭐ **A LADEIRA DO ÓLEO para a água** (Bug 2 do smoke do dono 30/10: «a água sobe a ladeira do óleo»). O nivelamento
//! corria só pela altura da ÁGUA (`film`) e o capilar pelo dente do papel; a altura do óleo assentado não entrava em
//! passe nenhum e a água espalhava-se por cima dele como por papel liso. A superfície que a água nivela é água + óleo: a
//! do pé da parede corre para longe dela, a que está em cima do óleo desce. Só a tinta ASSENTADA com o Real Oil tem
//! altura (`sett × espessura × Depth·Real Oil`, a massa do campo de `oleo_real`); sem o campo do óleo na grade, `None`
//! e o fluxo de sempre, ao bit.
//!
//! Smoke do dono 30/10: com o Real Oil na mão, a ladeira é a do
//! óleo de ANTES do traço ([`Grid::oleo_antes`], fotografada no início do traço) — o óleo que se pinta corre como
//! sempre e a água do traço novo respeita o relevo dos anteriores. Sonda: `queda_na_ladeira_tests`.

use crate::grid::Grid;

/// A altura de água (`film`) que uma unidade de massa de óleo assentado ergue. MEDIDO (`diag_a_agua_sobe_a_ladeira_do_oleo`):
/// o miolo de um Dabber tem `~3000` de massa e a água de um Wet Flat `film` `1–4`; a parede tem de ganhar da água.
pub(crate) const FILME_POR_MASSA: f64 = 1.0 / 1000.0;

/// A altura do óleo assentado na célula fina `i`, em unidades de `film`.
fn altura(g: &Grid, i: usize) -> f32 {
    let m = f64::from(g.sett[i]);
    if m <= 0.0 {
        return 0.0;
    }
    let k = crate::grid::oleo_celula::ganho(&g.oleo, i);
    if k <= 0.0 {
        return 0.0;
    }
    (m * crate::grid::espessura::factor(g, i) * k * FILME_POR_MASSA) as f32
}

/// A altura do óleo nas linhas FINAS `r0..=r1` (as que o passe de fluxo amostra), em unidades de `film`.
pub(crate) struct LadeiraDoOleo {
    r0: usize,
    s: usize,
    h: Vec<f32>,
    /// A variante da [`sonda`] lida uma vez por passe, e o `film` de referência.
    sonda: (u8, f64),
}

/// **SONDA da queda pela diluição** (J11, `queda_na_ladeira_tests::diag_a_queda_pela_diluicao`): variantes de lei
/// trocadas em EXECUÇÃO (nunca `const` recompilada), com `w = min(1, film ÷ FREF)²` (lubrificação: a água fina
/// corre devagar). `0` = a lei de sempre, ao bit · `1` = o tecto do nivelamento × `w` · `2` = a ladeira × `w` ·
/// `3` = a de sempre, contada · `4` = a ladeira × `film ÷ FREF` (linear). Os contadores só correm com variante ≠ 0.
/// Não é a lei do produto — o dono escolhe pela prancha `docs/Painter/Pinceis/pranchas/J11_queda_diluicao.png`.
#[doc(hidden)]
pub mod sonda {
    use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering::Relaxed};

    /// A variante viva.
    pub static VARIANTE: AtomicU8 = AtomicU8::new(0);
    /// O `film` de referência (`f32` em bits).
    pub static FREF: AtomicU32 = AtomicU32::new(0x3F80_0000);
    /// Termos de célula com ladeira ≠ 0.
    pub static N_LADEIRA: AtomicU64 = AtomicU64::new(0);
    /// … desses, os que bateram no tecto do nivelamento.
    pub static N_TECTO: AtomicU64 = AtomicU64::new(0);

    pub(super) fn agora() -> (u8, f64) {
        (
            VARIANTE.load(Relaxed),
            f64::from(f32::from_bits(FREF.load(Relaxed))).max(1e-6),
        )
    }

    pub(super) fn conta(com_ladeira: bool, no_tecto: bool) {
        if com_ladeira {
            N_LADEIRA.fetch_add(1, Relaxed);
            if no_tecto {
                N_TECTO.fetch_add(1, Relaxed);
            }
        }
    }
}

impl LadeiraDoOleo {
    /// `None` sem o campo do óleo (nenhum traço com o Real Oil nesta grade) ou sem altura nenhuma nas linhas; `antes` =
    /// a altura fotografada no início do traço ([`Grid::oleo_antes`]) em vez da de agora.
    pub(crate) fn de(g: &Grid, r0: usize, r1: usize, antes: bool) -> Option<Self> {
        if g.oleo.is_empty() || r1 < r0 || (antes && g.oleo_antes.len() != g.cells) {
            return None;
        }
        let s = g.s;
        let r1 = r1.min(g.cells / s - 1);
        let mut h = vec![0f32; (r1 + 1 - r0) * s];
        let mut alguma = false;
        for y in r0..=r1 {
            for x in 0..s {
                let i = y * s + x;
                let v = if antes { g.oleo_antes[i] } else { altura(g, i) };
                if v > 0.0 {
                    h[(y - r0) * s + x] = v;
                    alguma = true;
                }
            }
        }
        alguma.then_some(Self {
            r0,
            s,
            h,
            sonda: sonda::agora(),
        })
    }

    /// O termo de nivelamento de um eixo (`dfilm` = a diferença de `film` entre os vizinhos, `o` = a da ladeira, `f` =
    /// o `film` desta célula), já clampado. Variante 0 da [`sonda`] = a lei de sempre, ao bit.
    #[inline]
    pub(crate) fn termo(
        &self,
        dfilm: f64,
        o: f64,
        f: f64,
        level_k: f64,
        dif: f64,
        clamp: f64,
    ) -> f64 {
        let (v, fref) = self.sonda;
        if v == 0 {
            return ((dfilm + o) * level_k * dif).clamp(-clamp, clamp);
        }
        let r = (f / fref).clamp(0.0, 1.0);
        let w = r * r;
        let (o, clamp) = match v {
            1 => (o, clamp * w),
            2 => (o * w, clamp),
            4 => (o * r, clamp),
            _ => (o, clamp),
        };
        let lx = (dfilm + o) * level_k * dif;
        sonda::conta(o != 0.0, lx.abs() >= clamp);
        lx.clamp(-clamp, clamp)
    }

    /// A altura na célula fina `fi` (`0` fora das linhas).
    #[inline]
    pub(crate) fn em(&self, fi: usize) -> f64 {
        fi.checked_sub(self.r0 * self.s)
            .and_then(|k| self.h.get(k))
            .map_or(0.0, |v| f64::from(*v))
    }
}

/// A ladeira das linhas finas que o passe de fluxo das linhas de FLUXO `by0..=by1` amostra. Sem o Real Oil na mão, a
/// altura de agora; com ele (`oleo_na_mao`), só a do óleo de ANTES do traço — o óleo que se pinta corre como sempre
/// (MEDIDO: com a ladeira também nele, as 10 miniaturas Oil/* mudavam 82–230 níveis; a miniatura é um traço só, a
/// fotografia dela é vazia ⇒ `None`, ao bit).
pub(crate) fn do_passe(g: &Grid, by0: i32, by1: i32, oleo_na_mao: bool) -> Option<LadeiraDoOleo> {
    if g.oleo.is_empty() {
        return None;
    }
    let (gh, rf) = (g.h as i32, g.flow.rf);
    let r0 = crate::flow::flow_probe(by0 - 1, gh, rf).max(0) as usize;
    let r1 = crate::flow::flow_probe(by1 + 1, gh, rf).max(0) as usize;
    LadeiraDoOleo::de(g, r0, r1, oleo_na_mao)
}

/// No início de um traço: com o Real Oil na mão e óleo na grade, fotografa a altura do óleo dos traços anteriores; senão
/// esvazia a fotografia. Custa uma passagem pela grade só quando se pinta óleo sobre óleo.
pub(crate) fn fotografa(g: &mut Grid, oleo_na_mao: bool) {
    if !oleo_na_mao || g.oleo.is_empty() {
        g.oleo_antes = Vec::new();
        return;
    }
    let h: Vec<f32> = (0..g.cells).map(|i| altura(g, i)).collect();
    g.oleo_antes = h;
}
