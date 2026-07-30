//! §6.1 A ATENÇÃO do solver: de que células ele precisa olhar (child de
//! [`super`] — split por RESPONSABILIDADE, não por tamanho: *quais células
//! importam* é uma pergunta diferente de *como o fluido se move*, que é o que
//! sobrou no pai. Pure code motion; o fingerprint da sessão pina a identidade).

use crate::grid::Grid;
use crate::par::{self, Rows};

// ---------------------------------------------------------------------------

/// Rebuild the active mask + fresh bbox from the water map. Pass 1 marks
/// horizontal wet triples inside the previous (padded) bbox; pass 2 grows a
/// vertical "skirt" wherever a vertical triple sums to EXACTLY 1 — and the 2s
/// it writes count in later sums, which is load-bearing: an isolated front
/// gets a full skirt and can run 1 cell/frame, while a train of close stripes
/// starves its own skirt and waits to merge (keeps a wide front from
/// decomposing into permanent horizontal bands).
pub fn rebuild_active_region(g: &mut Grid) {
    let rows = (g.by1 - g.by0 + 1).max(0) as usize;
    let span = (g.bx1 - g.bx0 + 1).max(0) as usize;
    rebuild_active_region_rows(g, Rows::pick(rows, span, par::MIN_CELLS_REBUILD));
}

/// [`rebuild_active_region`] com a rota de caminhada FORÇADA — a porta dos
/// gates de identidade (ADR-0145). O produto chama sempre o
/// [`rebuild_active_region`].
///
/// ⚠️ **Três das quatro sub-passadas são row-disjuntas; a SAIA não é**, e a
/// linha de corte é o mecanismo, não a conveniência:
///
/// * a **limpeza** escreve `active` na própria linha e não lê nada;
/// * o **scan da extensão viva** lê `film`/`susp`/`vel` (nenhum tocado aqui) e
///   escreve UM par de escalares por linha (`live_lo[y]`/`live_hi[y]`);
/// * o **passe 1** lê o trio `film[i−1..=i+1]`, que é **HORIZONTAL** (a mesma
///   linha), escreve `active[i−1..=i+1]` (idem) e reduz a bbox por `min`/`max`
///   — associativos e comutativos, logo o `reduce` paralelo devolve o número
///   exato do `fold` serial;
/// * a **saia** escreve `active[i±s]` e o comentário dela diz por quê:
///   *"scanned top-to-down so earlier 2s shape later sums"* — cross-row, com
///   ordem load-bearing. Fica **serial**, e é isso que a mantém correta.
pub fn rebuild_active_region_rows(g: &mut Grid, mode: Rows) {
    let s = g.s;
    let w = g.w as i32;
    let h = g.h as i32;
    if !g.has_fluid || g.bx1 < g.bx0 {
        g.empty_bbox();
        return;
    }
    // Clear the mask over the previous bbox padded ±2. Extend the clear to
    // the pad ring when the box touches the sheet edge, so skirt 2s written
    // on the pad cannot go stale (the [1..W] clamp plus pad writes would
    // otherwise leak permanent actives on row 0 / row H+1).
    let px0 = (g.bx0 - 2).max(1);
    let px1 = (g.bx1 + 2).min(w);
    let py0 = (g.by0 - 2).max(1);
    let py1 = (g.by1 + 2).min(h);
    let cx0 = if px0 == 1 { 0 } else { px0 };
    let cx1 = if px1 == w { w + 1 } else { px1 };
    let cy0 = if py0 == 1 { 0 } else { py0 };
    let cy1 = if py1 == h { h + 1 } else { py1 };
    // Só a JANELA da faixa viva precisa ser limpa: `active ⊆ faixa` por
    // construção (invariante publicado por esta mesma função), então uma
    // célula fora dela já vale 0 e a limpeza seria escrever 0 sobre 0.
    {
        let spans_on = g.spans_enabled;
        let Grid {
            row_lo,
            row_hi,
            active,
            ..
        } = g;
        let (row_lo, row_hi): (&[i32], &[i32]) = (row_lo, row_hi);
        let band = cy0 as usize * s..(cy1 as usize + 1) * s;
        par::walk_rows(mode, &mut active[band], s, |r, row| {
            let y = cy0 + r as i32;
            let (wl, wh) = crate::grid::span_window_of(row_lo, row_hi, spans_on, s, y);
            let (l, hgh) = (wl.max(cx0), wh.min(cx1));
            if l > hgh {
                return;
            }
            row[l as usize..=hgh as usize].fill(0);
        });
    }
    // A extensão VIVA desta passada — "há algo aqui que o solver ainda tem de
    // terminar". São TRÊS coisas, e cada uma responde a um passe diferente:
    //
    //   `film > 0`  a água (o `advect`, e o próprio passe 1 abaixo)
    //   `susp > 0`  o pigmento em suspensão — o `drying_pass` é gateado em
    //               `film/susp`, NÃO na máscara ("paint dries everywhere"),
    //               então uma faixa que só cobrisse o ativo pararia de secar
    //               pigmento num pixel que a máscara já largou
    //   `vel != 0`  a VELOCIDADE sobrevivente: o ramo inativo do `advect`
    //               zera `vel`, e essa escrita É observável (o fingerprint da
    //               sessão inclui `vel_x`/`vel_y`). Uma célula que secou e
    //               ficou com velocidade tem de continuar visível até o
    //               advect zerá-la — aí ela sai sozinha na próxima passada.
    //               ⚠️ Isto foi a rede de debug que achou, na PRIMEIRA
    //               execução da suíte: sem este termo, o rastro de um drip
    //               que se afasta mais de 5 células fica com velocidade
    //               fóssil e o estado deriva do motor original.
    //
    // O anel de dreno fica FORA do termo de velocidade: o `apply_boundaries`
    // reescreve os dois componentes lá em todo frame, logo após o advect, e
    // nada lê `vel` entre um e outro — a zeragem do advect é natimorta ali.
    // ⚠️ Varre TODA linha cuja janela é não-vazia, não só as da bbox: a faixa
    // é o que lembra onde há velocidade fóssil, e a bbox de um traço NOVO não
    // tem por que cobrir o rastro de um antigo. O laço de linhas é O(altura)
    // e sai na hora nas vazias; o custo real é a janela, que é a faixa.
    g.clear_live();
    {
        let spans_on = g.spans_enabled;
        // ⚠️ A velocidade mora na grade de FLUXO: a pergunta *"esta célula secou
        // mas ainda carrega velocidade?"* é feita à célula de fluxo que a
        // contém. Em `rf = 1` o índice é o próprio `i`.
        let geom = g.flow;
        let Grid {
            row_lo,
            row_hi,
            film,
            susp,
            vel_x,
            vel_y,
            live_lo,
            live_hi,
            ..
        } = g;
        let (row_lo, row_hi): (&[i32], &[i32]) = (row_lo, row_hi);
        let (film, susp): (&[f32], &[f32]) = (film, susp);
        let (velx, vely): (&[f32], &[f32]) = (vel_x, vel_y);
        par::walk_row_scalars2(mode, live_lo, live_hi, |ry, out_lo, out_hi| {
            let y = ry as i32;
            let (wl, wh) = crate::grid::span_window_of(row_lo, row_hi, spans_on, s, y);
            let (l, r) = (wl.max(1), wh.min(w));
            if l > r {
                return;
            }
            let vel_rows = y >= 2 && y < h;
            let base = ry * s;
            let mut lo = i32::MAX;
            let mut hi = i32::MIN;
            let mut i = l as usize + base;
            let fy = crate::flow::fine_to_flow(y, geom.rf);
            for x in l..=r {
                let vi = if geom.is_identity() {
                    i
                } else {
                    geom.idx(crate::flow::fine_to_flow(x, geom.rf), fy)
                };
                let live = film[i] > 0.0
                    || susp[i] > 0.0
                    || (vel_rows && x >= 2 && x < w && (velx[vi] != 0.0 || vely[vi] != 0.0));
                if live {
                    if x < lo {
                        lo = x;
                    }
                    hi = x;
                }
                i += 1;
            }
            if lo <= hi {
                *out_lo = lo;
                *out_hi = hi;
            }
        });
    }

    // Pass 1 — wet cells (one row/col inside the brushable area; the drain
    // ring only ever activates via the skirt).
    let sx0 = px0.max(2);
    let sx1 = px1.min(w - 1);
    let sy0 = py0.max(3);
    let sy1 = py1.min(h - 2);
    // O elemento NEUTRO da redução da bbox. Ele é neutro de verdade e não por
    // convenção: `x >= sx0 >= 2` e `y >= sy0 >= 3`, então `max(0, ...)` e
    // `min(w+1, ...)` nunca escolhem o lado da identidade sobre um valor real.
    let ident = (w + 1, 0, h + 1, 0, false);
    let (fx0, fx1, fy0, fy1, fired) = if sy1 < sy0 {
        ident
    } else {
        let spans_on = g.spans_enabled;
        let Grid {
            row_lo,
            row_hi,
            film,
            active,
            ..
        } = g;
        let (row_lo, row_hi): (&[i32], &[i32]) = (row_lo, row_hi);
        let film: &[f32] = film;
        let band = sy0 as usize * s..(sy1 as usize + 1) * s;
        par::walk_rows_reduce(
            mode,
            &mut active[band],
            s,
            ident,
            |r, row| {
                let y = sy0 + r as i32;
                // A janela cobre `film ⊕ 2`, então o trio horizontal e as
                // escritas `active[i±1]` cabem nela: fora daqui o trio é
                // 0 + 0 + 0. ⚠️ E é por o trio ser HORIZONTAL que esta linha
                // não precisa de nenhuma outra: ela lê e escreve só a si.
                let (wl, wh) = crate::grid::span_window_of(row_lo, row_hi, spans_on, s, y);
                let (rx0, rx1) = (wl.max(sx0), wh.min(sx1));
                if rx0 > rx1 {
                    return ident;
                }
                let mut acc = ident;
                let base = y as usize * s;
                let mut i = rx0 as usize + base;
                for x in rx0..=rx1 {
                    if film[i - 1] as f64 + film[i] as f64 + film[i + 1] as f64 > 0.0 {
                        let k = x as usize;
                        row[k - 1] = 1;
                        row[k] = 1;
                        row[k + 1] = 1;
                        acc.4 = true;
                        if x < acc.0 {
                            acc.0 = x;
                        }
                        if x > acc.1 {
                            acc.1 = x;
                        }
                        if y < acc.2 {
                            acc.2 = y;
                        }
                        if y > acc.3 {
                            acc.3 = y;
                        }
                    }
                    i += 1;
                }
                acc
            },
            |a, b| {
                (
                    a.0.min(b.0),
                    a.1.max(b.1),
                    a.2.min(b.2),
                    a.3.max(b.3),
                    a.4 || b.4,
                )
            },
        )
    };
    if !fired {
        g.empty_bbox();
        return;
    }

    // Pass 2 — the skirt, scanned top-to-down so earlier 2s shape later sums.
    let kx0 = (fx0 - 2).max(1);
    let kx1 = (fx1 + 2).min(w);
    let ky0 = (fy0 - 2).max(1);
    let ky1 = (fy1 + 2).min(h);
    let mut nx0 = w + 1;
    let mut nx1 = 0;
    let mut ny0 = h + 1;
    let mut ny1 = 0;
    let mut any_fire = false;
    for y in ky0..=ky1 {
        // O trio VERTICAL só pode somar 1 onde alguma das três células é
        // ativa, e as ativas desta passada saíram do passe 1 — dentro da
        // janela, que carrega a margem de ±2 nas linhas vizinhas também.
        let (wl, wh) = g.span_window(y);
        let (rx0, rx1) = (wl.max(kx0), wh.min(kx1));
        if rx0 > rx1 {
            continue;
        }
        let mut i = rx0 as usize + y as usize * s;
        for x in rx0..=rx1 {
            if g.active[i - s] as u32 + g.active[i] as u32 + g.active[i + s] as u32 == 1 {
                g.active[i - s] = 2;
                g.active[i] = 2;
                if g.active[i + s] == 0 {
                    g.active[i + s] = 2;
                }
                any_fire = true;
                if x < nx0 {
                    nx0 = x;
                }
                if x > nx1 {
                    nx1 = x;
                }
                if y < ny0 {
                    ny0 = y;
                }
                if y > ny1 {
                    ny1 = y; // the fire row itself
                }
            }
            i += 1;
        }
    }
    if !any_fire {
        // defensive: keep the pass-1 extent
        nx0 = fx0;
        nx1 = fx1;
        ny0 = fy0;
        ny1 = fy1;
    }
    g.bx0 = (nx0 - 5).max(1);
    g.bx1 = (nx1 + 5).min(w);
    g.by0 = (ny0 - 5).max(1);
    g.by1 = (ny1 + 5).min(h);
    g.has_fluid = true;
    // Publica a faixa por-linha com o MESMO pad ±5 que a bbox acabou de levar
    // — é o análogo por-linha do casco, e é ele que mantém `active ⊆ faixa`.
    g.publish_spans_from_live();
}
