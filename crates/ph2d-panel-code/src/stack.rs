//! ⭐ **A pilha da aba de scripts** (auditoria de layout 11/11): o código em cima, depois o quadro do
//! depurador, a Bancada e a faixa de baixo. O código tem a prioridade, como num IDE: cada cartão diz a
//! altura que quer, o mínimo a que encolhe e o cabeçalho ([`Want`]); quando a aba não chega, três
//! degraus, cada um pela ordem Bancada → depurador → faixa — (1) até ao mínimo, (2) até ao cabeçalho
//! (o título dobrado, a barra com Continue, só as abas), (3) os cartões saem (Bancada, faixa e por
//! fim o depurador, cuja barra é a última a ir) até o código guardar [`CODE_LEAST_ROWS`] linhas; sem
//! espaço nem para isso o código fica com a aba toda. Em TODA a altura os rects de [`tab`] não se
//! cruzam nem saem da aba (a janela de 900×560 punha a Bancada por cima do depurador e o código a 0).

use ph2d_editor_core::zones::Rect;

/// Quantas linhas o código guarda enquanto os cartões ainda têm de onde ceder.
pub const CODE_LEAST_ROWS: usize = 4;

/// A altura que um cartão quer, o mínimo a que encolhe e o cabeçalho a que cai por último
/// (`h == 0` = o cartão não está).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Want {
    pub h: f32,
    pub least: f32,
    pub head: f32,
    /// O artista PEDIU o cartão (a faixa aberta, ou um ⊗/⚠ clicado): não cai abaixo do `least` e sai
    /// por último — antes dele o corpo do depurador e, sem espaço, a barra dele.
    pub asked: bool,
}

/// O mínimo do cartão do código em [`CODE_LEAST_ROWS`] linhas INTEIRAS à vista (com as métricas `m`
/// da vista): o recuo do poço ([`crate::cards::code_well`]), a busca (`find_h`, 0 = fechada), o
/// respiro de cima, as linhas e a barra de estado.
#[must_use]
pub fn code_least(find_h: f32, m: &crate::metrics::Metrics) -> f32 {
    crate::cards::well_pad() + find_h + m.top_pad + m.line_h * CODE_LEAST_ROWS as f32 + m.status_h
}

/// O que cada cartão presente custa: a sua altura e um `gap`.
fn cost(hs: &[f32], gap: f32) -> f32 {
    hs.iter().filter(|h| **h > 0.0).map(|h| h + gap).sum()
}

/// Os degraus 1 e 2 sobre os cartões `on`; devolve as alturas e o que ainda sobra de `room`
/// (negativo = não coube). O 1.º encolhe o que falta (o corpo rola); o 2.º cai INTEIRO ao cabeçalho
/// (um corpo meio cortado, como a linha do erro a meio, não serve; o resto vai ao código).
fn squeeze<const N: usize>(
    room: f32,
    gap: f32,
    wants: &[Want; N],
    on: [bool; N],
) -> ([f32; N], f32) {
    let mut hs: [f32; N] = std::array::from_fn(|i| if on[i] { wants[i].h.max(0.0) } else { 0.0 });
    let mut over = cost(&hs, gap) - room;
    for whole in [false, true] {
        for (h, w) in hs.iter_mut().zip(wants) {
            if over <= 0.0 {
                break;
            }
            let floor = if whole { w.head } else { w.least };
            let give = (*h - floor.min(*h)).max(0.0);
            let cut = if whole { give } else { over.min(give) };
            *h -= cut;
            over -= cut;
            if cut > 0.0 && *h <= 0.0 {
                over -= gap;
            }
        }
    }
    (hs, -over)
}

/// As alturas de `wants` (pela ordem em que encolhem) numa aba de altura `avail` que deixa
/// `code_least` ao código; `drop` = a ordem em que saem no 3.º degrau. Cada cartão presente custa
/// também um `gap`; a soma nunca passa de `avail`.
#[must_use]
pub fn fit<const N: usize>(
    avail: f32,
    code_least: f32,
    gap: f32,
    wants: [Want; N],
    drop: [usize; N],
) -> [f32; N] {
    let room = (avail - code_least).max(0.0);
    let mut on = wants.map(|w| w.h > 0.0);
    for i in drop {
        let (hs, left) = squeeze(room, gap, &wants, on);
        if left >= 0.0 {
            return hs;
        }
        on[i] = false;
    }
    [0.0; N]
}

/// Os rects da aba: o código em cima e, por baixo, cada cartão presente depois de um `gap`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub code: Rect,
    pub debug: Option<Rect>,
    pub bench: Option<Rect>,
    /// A faixa de baixo (`h == 0` = saiu: não pinta nem recebe cliques).
    pub band: Rect,
}

/// A pilha da aba `area`: as alturas de [`fit`] para a Bancada, o depurador e a faixa (saem a
/// Bancada, a faixa e por fim o depurador; a faixa PEDIDA, [`Want::asked`], sai por último), postas
/// por baixo do código.
#[must_use]
pub fn tab(area: Rect, code_least: f32, gap: f32, [bench, debug, band]: [Want; 3]) -> Placed {
    let order = if band.asked { [0, 1, 2] } else { [0, 2, 1] };
    let [b, d, f] = fit(area.h, code_least, gap, [bench, debug, band], order);
    let code = Rect::new(
        area.x,
        area.y,
        area.w,
        (area.h - cost(&[b, d, f], gap)).max(0.0),
    );
    let mut y = code.y + code.h;
    let mut next = |h: f32| {
        (h > 0.0).then(|| {
            let r = Rect::new(area.x, y + gap, area.w, h);
            y += gap + h;
            r
        })
    };
    let (debug, bench) = (next(d), next(b));
    let band = next(f).unwrap_or(Rect::new(area.x, area.y + area.h, area.w, 0.0));
    Placed {
        code,
        debug,
        bench,
        band,
    }
}
