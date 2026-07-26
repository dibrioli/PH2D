//! **A tabela do filme, e o PLANO por dab que a lê** — a metade "sem um único `sqrt`" do AA
//! ([`crate::height_film::FilmAa`], plano 26 §9.6).
//!
//! Módulo irmão do [`crate::height_film`] pelo teto de LOC **e** porque é outra ideia: lá mora a
//! *curva* do filme e a média sobre a grade; aqui mora a *tabela* dela e a base deformada em que a
//! grade é avaliada sem re-percorrer a cadeia de silhueta.

use crate::height_film::{FilmAa, film_of};
use std::sync::{Arc, Mutex};

/// **A tabela de `F(t) = film_of(falloff_weight(t))`** — a curva do filme, pré-computada.
///
/// Construída **uma vez por traço** (o falloff e a hardness não mudam no meio de um) e emprestada por
/// referência aos dois kernels: uma alocação por traço, nunca por dab, senão o `hot_path_no_alloc` do
/// depósito acende — e com razão.
///
/// ⚠️ **A tabela cobre `[0, 1]` inteiro, não só a banda**, e isso é decisão: o `film_at_lut` precisa
/// dela também fora da banda (o early-out do single-sample) e nas amostras que o gradiente joga PARA
/// fora dela, e uma tabela de banda com clamp nas pontas mentiria exactamente nos texels de borda que o
/// AA existe para desenhar. A resolução vem do tamanho, não do recorte.
pub struct FilmLut {
    /// `N + 1` amostras de `F` em `t = i/N`, para a interpolação linear ter o vizinho da direita sem
    /// caso especial no último índice.
    table: Vec<f32>,
    /// `N` como `f32`, o fator de índice — guardado para o `at` não converter por chamada.
    n: f32,
}

impl FilmLut {
    /// Amostras da tabela. **16384** é o mesmo tamanho que a tabulação da transferência sRGB do doc 24
    /// escolheu, e pelo mesmo motivo: a resolução em `t` (6,1e-5) fica **abaixo** do erro que a
    /// linearização da métrica já introduz (~4,4e-5 no aro de um raio 100), então a tabela não é o termo
    /// dominante do épsilon — o que a torna um detalhe de implementação em vez de um segundo knob.
    pub const N: usize = 16_384;

    /// Tabela para este pincel. `F` é avaliada com as MESMAS funções do produto (`falloff_weight` e
    /// `film_of`), então a tabela não pode discordar da curva: ela **é** a curva, amostrada.
    #[must_use]
    pub fn new(spec: &crate::BrushSpec) -> Self {
        let n = Self::N;
        #[expect(
            clippy::cast_precision_loss,
            reason = "N = 16384 é exato em f32; o índice é um contador pequeno"
        )]
        let inv = 1.0 / (n as f32);
        let mut table = Vec::with_capacity(n + 1);
        for i in 0..=n {
            #[expect(clippy::cast_precision_loss, reason = "i <= 16384, exato em f32")]
            let t = (i as f32) * inv;
            table.push(film_of(spec.falloff_weight(t)));
        }
        #[expect(clippy::cast_precision_loss, reason = "N = 16384 é exato em f32")]
        let nf = n as f32;
        Self { table, n: nf }
    }

    /// **`raio × minor` mínimo para a LUT ser oferecida** — medido, não escolhido.
    ///
    /// O erro da expansão é o resto de 3ª ordem, logo escala com a **CURVATURA** da silhueta, e a
    /// curvatura é governada pelo **menor raio local** ([`crate::FootprintDeform::minor_fraction`]).
    /// Abaixo deste produto o pior texel passa de meio nível de u8 e o chamador fica no caminho exato.
    ///
    /// ⚠️ **E a coincidência que não é coincidência:** é a partir daqui que o AA custa caro (68,7 ms a
    /// raio 100 contra ~9 a raio 20) — os dois escalam com o tamanho da pegada, então a LUT rende
    /// exactamente onde o custo está e é recusada exactamente onde erraria.
    pub const MIN_EFFECTIVE_RADIUS: f32 = 40.0;

    /// **A porta única da admissibilidade** — o chamador pergunta aqui em vez de re-derivar a regra.
    ///
    /// Três cláusulas, e a terceira é do CHAMADOR porque é por-texel:
    ///
    /// 1. a família de falloff **SUAVE** — `Constant` se exclui por DOIS motivos ao mesmo tempo (é
    ///    errático, porque um degrau interage com a grade de texels, **e é mais LENTO**, 0,46×, porque a
    ///    curva dele é a constante 1 e não há raiz a economizar); `Custom` porque a `for_dab` toma o dab
    ///    inteiro como banda para ele e a tabela seria indexada por uma curva do documento;
    /// 2. **`raio × minor ≥ `[`Self::MIN_EFFECTIVE_RADIUS`];
    /// 3. ⚠️ **o texel não pode STRADDLEAR a fronteira calota↔banda da cápsula** — ali o `B` correto muda
    ///    no meio da grade 3×3 e nenhuma base única serve (medido: 0,77 nível a raio 40, contra 0,06 nas
    ///    outras regiões). É uma faixa de ~2 linhas de texel por calota, então o caminho exato ali custa
    ///    quase nada — e é por isso que a cláusula é do chamador, que é quem tem a projeção em mãos.
    #[must_use]
    pub fn admissible(spec: &crate::BrushSpec, radius: f32) -> bool {
        let smooth = matches!(
            spec.falloff,
            crate::Falloff::Smooth
                | crate::Falloff::Smoother
                | crate::Falloff::Sphere
                | crate::Falloff::Sharp
                | crate::Falloff::Pow4
                | crate::Falloff::Root
                | crate::Falloff::Linear
                | crate::Falloff::InvSquare
        );
        // Hardness ≥ 1 torna QUALQUER falloff um degrau (`falloff_weight` devolve 1 ou 0), então ela
        // recai no caso `Constant` e sai pela mesma porta.
        let hard = spec.hardness >= 1.0;
        let minor = spec.dab_footprint([1.0, 0.0]).minor_fraction();
        smooth && !hard && radius * minor >= Self::MIN_EFFECTIVE_RADIUS
    }

    /// `F(t)` por interpolação linear. Fora de `[0, 1]` clampa — que é o que a curva faz de qualquer
    /// forma (`falloff_weight` clampa o remap, `weight` devolve 0 em `t >= 1`).
    #[inline]
    #[must_use]
    pub fn at(&self, t: f32) -> f32 {
        let x = (t * self.n).clamp(0.0, self.n);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "x está clampado em [0, N] logo acima"
        )]
        let i = x as usize;
        let f = x - {
            #[expect(clippy::cast_precision_loss, reason = "i <= 16384, exato em f32")]
            let fi = i as f32;
            fi
        };
        // `i + 1` existe: a tabela tem N+1 entradas e `i <= N`. Em `i == N` o `f` é 0, então o vizinho
        // não é lido — mas o índice tem de ser válido, e é por isso que a tabela leva a entrada extra.
        let a = self.table[i];
        let b = self.table[(i + 1).min(self.table.len() - 1)];
        a + (b - a) * f
    }
}

/// **O plano por DAB: a tabela mais a base deformada `B`** — a porta ÚNICA que os dois kernels
/// (pigmento e altura) atravessam, para nenhum dos dois escolher uma regra que o outro não conhece.
///
/// `B` leva um offset de TEXEL ao espaço deformado, e é **linear em cada região** ([`FilmAa::film_at_lut`]
/// tem a derivação). O plano guarda as duas que existem:
///
/// * **`disc`** — `B = A/radius`: o disco do pigmento, e as **CALOTAS** da cápsula (uma calota *é* um
///   disco centrado no extremo);
/// * **`band`** — `B = A·P/radius` com `P = I − uuᵀ`: a **BANDA** da cápsula varrida, onde o termo de
///   2ª ordem é exatamente zero.
///
/// A escolha entre elas é por TEXEL e sai da projeção no eixo — por isso ela mora aqui, e não no
/// chamador: duas cópias da mesma pergunta divergem, e esta em particular divergiria só na fronteira,
/// onde ninguém olha.
pub struct FilmLutPlan<'a> {
    lut: &'a FilmLut,
    /// `B·(1,0)` e `B·(0,1)` para o disco / as calotas.
    disc: [[f32; 2]; 2],
    /// A banda da cápsula, quando o dab varre (`None` no pigmento e no 1º dab de um traço).
    band: Option<CapsuleBand>,
}

/// A metade "banda" do plano: a base própria dela mais o eixo e a corda de que o teste de fronteira é
/// feito.
struct CapsuleBand {
    basis: [[f32; 2]; 2],
    u: [f32; 2],
    back: f32,
}

impl<'a> FilmLutPlan<'a> {
    /// Assa a base para este dab. `sweep` é o eixo da cápsula (`(u, back)`, o mesmo par que o
    /// `sweep_axis` da altura devolve); `None` = disco.
    #[must_use]
    pub fn new(
        lut: &'a FilmLut,
        fp: crate::FootprintDeform,
        radius: f32,
        sweep: Option<([f32; 2], f32)>,
    ) -> Self {
        let inv = 1.0 / radius;
        let disc = [fp.apply([inv, 0.0]), fp.apply([0.0, inv])];
        let band = sweep.map(|(u, back)| {
            // `P·o = o − (o·u)u`, aplicado às duas colunas antes do afim do footprint.
            let proj = |o: [f32; 2]| {
                let s = o[0] * u[0] + o[1] * u[1];
                fp.apply([o[0] - s * u[0], o[1] - s * u[1]])
            };
            CapsuleBand {
                basis: [proj([inv, 0.0]), proj([0.0, inv])],
                u,
                back,
            }
        });
        Self { lut, disc, band }
    }

    /// O filme neste texel pela LUT — ou **`None`** quando o texel STRADDLEA a fronteira calota↔banda,
    /// onde o `B` correto muda no meio da grade 3×3 e nenhuma base única serve. O chamador cai no
    /// caminho exato ali; medido, é uma faixa de ~2 linhas de texel por calota (33 contra 342 do
    /// interior), então recusá-la custa quase nada e ACEITÁ-la custaria 0,77 nível de u8.
    ///
    /// `w` é o ponto já deformado (`A·(r/radius)`, cujo módulo é o `t`) e `d` é o offset CRU ao centro
    /// do dab — é ele, e não o resíduo, que responde de que lado da fronteira o texel está.
    #[must_use]
    pub fn film_at(&self, aa: &FilmAa, t: f32, w: [f32; 2], d: [f32; 2]) -> Option<f32> {
        let basis = match &self.band {
            None => self.disc,
            Some(b) => {
                let proj = d[0] * b.u[0] + d[1] * b.u[1];
                // A grade alcança [`crate::height_film::AA_REACH_PX`] texels e `u` é unitário, então
                // uma sub-amostra move a projeção por até isso: mais perto que isso de `0` ou de
                // `back`, o texel tem amostras nas DUAS regiões.
                let reach = crate::height_film::AA_REACH_PX;
                if proj.abs() < reach || (proj - b.back).abs() < reach {
                    #[cfg(test)]
                    LUT_STRADDLES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return None;
                }
                if proj > 0.0 && proj < b.back {
                    b.basis
                } else {
                    self.disc
                }
            }
        };
        #[cfg(test)]
        LUT_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Some(aa.film_at_lut(self.lut, t, w, basis[0], basis[1]))
    }
}

/// Quantas vezes a estrada rápida foi tomada, e quantas o straddle a recusou.
///
/// ⚠️ **Contadores, não um segundo caminho** — eles OBSERVAM o produto. Existem porque a lição do
/// ADR-0120 é exatamente esta: um caminho rápido que nunca dispara é código morto com todos os gates
/// verdes, e a única defesa é CONTAR quantas vezes ele disparou no laço real.
#[cfg(test)]
static LUT_HITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
/// Irmão do [`LUT_HITS`]: os texels que a fronteira calota↔banda devolveu ao caminho exato.
#[cfg(test)]
static LUT_STRADDLES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Zera os dois contadores e devolve o par `(hits, straddles)` acumulado desde o último reset.
#[cfg(test)]
pub(crate) fn take_lut_counts() -> (usize, usize) {
    use std::sync::atomic::Ordering::Relaxed;
    (LUT_HITS.swap(0, Relaxed), LUT_STRADDLES.swap(0, Relaxed))
}

/// ⚠️ **Os contadores são GLOBAIS e os testes correm em PARALELO** — sem esta trava um gate lê os
/// disparos de outro, e a primeira rodada de mutações desta wave mediu exatamente isso (uma mutação
/// no basis da banda "matou" o gate que conta hits, que ela não toca). Todo gate que RODA a estrada
/// rápida a segura, não só os que a LEEM: quem polui é quem dispara.
#[cfg(test)]
pub(crate) static COUNT_LOCK: Mutex<()> = Mutex::new(());

/// Segura a [`COUNT_LOCK`] ignorando envenenamento — um teste que entrou em pânico segurando-a não
/// pode derrubar os outros gates com uma falha que não é a deles.
#[cfg(test)]
pub(crate) fn lock_counts() -> std::sync::MutexGuard<'static, ()> {
    COUNT_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A chave do memo: **tudo** que [`crate::BrushSpec::falloff_weight`] lê — e são exatamente estes dois
/// campos (o `custom_falloff` fica de fora porque `Custom` não é admissível, então a chave é
/// COMPLETA). Uma chave incompleta serviria a tabela do pincel anterior, em silêncio.
#[derive(Clone, Copy, PartialEq, Eq)]
struct LutKey {
    falloff: crate::Falloff,
    /// Os BITS da hardness, não o `f32`: comparação por bits é reflexiva mesmo em `NaN`, e uma chave
    /// que nunca casa consigo mesma reconstruiria a tabela em todo dab — o custo que o memo existe
    /// para evitar, com todos os gates verdes.
    hardness_bits: u32,
}

/// **UMA entrada, e é o bastante:** o pincel não muda no meio de um traço, então o memo acerta em todo
/// dab depois do primeiro. Um mapa seria um cache de PINCÉIS, que é outra pergunta — e ninguém a fez.
static LUT_MEMO: Mutex<Option<(LutKey, Arc<FilmLut>)>> = Mutex::new(None);

/// **A tabela deste pincel, construída uma vez por TRAÇO** (`None` = inadmissível ⇒ caminho exato).
///
/// ⚠️ **Por dab a LUT seria 9× MAIS CARA que o que ela substitui** — 16 384 avaliações de
/// `falloff_weight` contra ~1 800 amostras de banda a raio 20. O memo não é uma otimização do
/// desenho: ele **é** o desenho, e é o que separa esta wave de uma regressão de perf.
///
/// A admissibilidade é perguntada ANTES do memo porque ela depende do `radius`, que o Jitter Scale
/// move por dab — um dab pequeno demais cai no caminho exato sem tocar a tabela guardada.
#[must_use]
pub fn film_lut_for(spec: &crate::BrushSpec, radius: f32) -> Option<Arc<FilmLut>> {
    if !FilmLut::admissible(spec, radius) {
        return None;
    }
    let key = LutKey {
        falloff: spec.falloff,
        hardness_bits: spec.hardness.to_bits(),
    };
    // Um `Mutex` envenenado é razão para o caminho exato, nunca para um pânico: o filme é uma
    // aparência, e recusar-se a pintar seria pior que pintá-la pela rota lenta.
    let mut slot = LUT_MEMO.lock().ok()?;
    if let Some((k, lut)) = slot.as_ref()
        && *k == key
    {
        return Some(Arc::clone(lut));
    }
    let lut = Arc::new(FilmLut::new(spec));
    *slot = Some((key, Arc::clone(&lut)));
    Some(lut)
}
