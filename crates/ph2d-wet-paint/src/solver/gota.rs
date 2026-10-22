//! **A lei da GOTA** (filho de [`super`], LOC cap) — o instrumento das sondas `sonda_gota`; o produto não a usa.

use super::js_round_i64;

/// **A lei da GOTA** (W1e, F1) — quando a água pode DESCER para papel seco.
///
/// O port deixa a orla de baixo correr para o seco em 3 quadros de cada 4 (o fluxo desses quadros é a
/// velocidade suavizada, sem freio — «é por isso que os escorridos correm»), e a cada coluna ao mesmo
/// ritmo: a orla inteira abre em DEDOS (o pente, 7–10 por traço de 150 px). Uma gota num plano inclinado
/// não se solta assim: a conta de baixo só vence o papel seco quando a altura dela passa a crítica, que
/// cai com a inclinação (`h_c ∝ 1/|g|`, o depinning). Aqui, nos DOIS construtores do fluxo, a componente
/// que DESCE do fluxo de uma célula SECA leva o factor `min(1, 2·hill(h/h_c))` da altura `h` da conta
/// logo acima dela, `hill(q) = q⁸/(1+q⁸)` (1 em `h = h_c`; com `q⁴` a conta do Wet Wash a meia inclinação
/// — 17,7 contra a crítica 40 — ainda fugia 7 % e abria o pente); tudo o que corre sobre papel molhado não
/// muda, e com a tela direita não há lei. Nada é zero: abaixo da crítica a orla escorre devagar como
/// uma conta presa, e a transição suave faz o primeiro ponto a encher ganhar. `drip_head = 0` (o port)
/// não toca em nada.
///
/// | `G` (Gravity) · `H` | tela direita | meia (anel 4) | máxima (anel 8) | poça de 4 passadas, máx. |
/// |---|---|---|---|---|
/// | 0,005 · 0 (o port) | 0 fios | 0 · orla 0,76 | 1 fio de 21 px | 4 fios |
/// | 0,005 · 0,1 | 0, ao bit | 0 · orla 0,94 | 0 | 4 fios |
/// | **0,005 · 0,2 (o produto)** | **0 fios, ao bit** | **0 · orla 1,30** | **0** | **5 fios (corre)** |
/// | 0,01 · 0 | 0 | 1 · 20 px | 6–7 dedos (o pente) | — |
/// | 0,01 · 0,2 | 0 | 0 | 4 dedos | — |
/// | Rebelle | 0 | 0 · orla 1,6 | 1 fio fino de 250–300 px | — |
///
/// (Bancada do doc 06 §2, Wet Round, água 1 · 0,7 · 0,4 · 0,2, 30 s — `sonda_gota` +
/// `ferramentas/fios.py`, 05 §4b.) O fio único e longo do máximo NÃO sai desta lei: precisa de um
/// mecanismo não local (a conta a equalizar ao longo da orla, para o primeiro fio drenar os outros).
///
/// ⛔ **O PRODUTO NÃO A USA (W1f, 10/10):** ela cortava o escorrimento em RECTÂNGULOS (pontas de fio
/// quadradas, orla de baixo recta — o smoke do dono). A frente quase nunca passa pela altura crítica:
/// avança por um degrau binário (a célula «seca» destrava com `film > 1e-3`), e a linha nova enche-se
/// de LADO sem trava (a lei só trava a componente que desce) — uma fila inteira de cada vez. ~40
/// variantes medidas e recusadas no 05 §4b (fonte da conta, papel, limiar, pino, coluna, fluxo inteiro,
/// lei na gravidade, transição suave). Fica aqui como instrumento das sondas (`sonda_gota`).
pub const ALTURA_DA_GOTA: f64 = 0.2;

#[derive(Clone, Copy)]
pub(crate) struct Gota {
    gnx: f64,
    gny: f64,
    /// A célula FINA de cima (a fonte), em passos inteiros contra `ĝ`.
    dx: i64,
    dy: i64,
    h_c: f64,
}

impl Gota {
    /// `drip_head` = `h_c · |g|` (o produto liga-o); `None` sem gravidade ou com a lei desligada.
    pub(crate) fn new(drip_head: f64, gx: f64, gy: f64) -> Option<Self> {
        let g = (gx * gx + gy * gy).sqrt();
        (drip_head > 0.0 && g > 0.0).then(|| Gota {
            gnx: gx / g,
            gny: gy / g,
            dx: js_round_i64(gx / g),
            dy: js_round_i64(gy / g),
            h_c: drip_head / g,
        })
    }

    /// O fluxo `(ex, ey)` da célula `fi`, com a lei aplicada.
    ///
    /// ⚠️ **A advecção é por RECOLHA** — quem move a água para o seco é a célula SECA, que a puxa ao
    /// longo do próprio fluxo. Por isso a lei vale numa célula seca (sem filme nem marca de humidade),
    /// sobre a componente que DESCE, com a altura da conta logo ACIMA dela (a fonte). Medidos e
    /// recusados (05 §4b): a altura da própria célula seca (zero) — prendia a água toda, uma barreira;
    /// o fluxo INTEIRO da célula seca (margens presas) — 8–9 dedos em vez de 6 e riscos brancos no
    /// topo do traço.
    #[inline]
    pub(crate) fn aplica(
        &self,
        ex: f64,
        ey: f64,
        fi: usize,
        fine_s: usize,
        film: &[f32],
        wet: &[u8],
    ) -> (f64, f64) {
        if film[fi] > 1e-3 || wet[fi] != 0 {
            return (ex, ey);
        }
        let d = ex * self.gnx + ey * self.gny;
        if d <= 0.0 {
            return (ex, ey);
        }
        let j = fi as i64 - self.dx - self.dy * fine_s as i64;
        if j < 0 || j as usize >= film.len() {
            return (ex, ey);
        }
        let q8 = (f64::from(film[j as usize]) / self.h_c).powi(8);
        let k = (2.0 * q8 / (1.0 + q8)).min(1.0);
        (ex - d * self.gnx * (1.0 - k), ey - d * self.gny * (1.0 - k))
    }
}
