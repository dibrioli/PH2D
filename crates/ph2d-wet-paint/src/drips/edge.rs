//! A ORLA das gotas: a linha de contacto ([`pin_downhill`]), o que é orla de baixo ([`is_front`]) e a
//! caminhada da orla de uma bacia ([`walk_edge`]).

use super::*;

/// **A linha de contacto** — chamada pelo advect de Jacobi entre o fluxo materializado (`scratch.uv`) e a
/// saída: com a tela inclinada uma célula ativa SECA e POR FORA da mancha (seca a jusante) não puxa água de
/// lado nenhum — a mancha guarda a forma do traço e a água desce DENTRO dela (pelos buracos da franja,
/// que têm água por baixo) até à orla de baixo. ⛔ Fechar também os buracos de dentro prendia a água: a
/// orla deixava de escurecer (baixo/cima 1,0) e as gotas não enchiam (medido). ⛔ Cortar só a componente de DESCIDA (a 1.ª
/// forma) deixava a água entrar de LADO no papel seco por baixo de uma encosta: bases planas e degraus
/// (report do dono 08/10, foto do S).
/// Como a saída e o transporte leem o MESMO `uv`, a massa continua exata (o gather não muda de forma).
/// Linhas disjuntas (cada uma escreve só o seu `uv`): a rota de linhas do advect, `mode`.
pub fn pin_downhill(g: &mut Grid, grav: [f64; 2], notch_w: i32, mode: Rows) {
    let Some((nx, ny, _)) = unit(grav) else {
        return;
    };
    if g.by0 > g.by1 {
        return;
    }
    let (g_w, g_h) = (g.w as i32, g.h as i32);
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        active,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let (film, active): (&[f32], &[u8]) = (film, active);
    let folha = Folha {
        film,
        s,
        w: g_w,
        h: g_h,
    };
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows(mode, &mut scratch.uv[band], s, |ri, row| {
        let y = by0 + ri as i32;
        let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        if lo > hi {
            return;
        }
        let base = y as usize * s;
        for x in lo as usize..=hi as usize {
            if active[base + x] != 0 && folha.por_fora(x as i32, y, nx, ny, notch_w) {
                row[x] = [0.0, 0.0];
            }
        }
    });
}

/// A folha vista pela orla: a película e o tamanho — o predicado partilhado não pede o `Grid` inteiro (a
/// linha de contacto escreve o `scratch` dele enquanto pergunta).
#[derive(Clone, Copy)]
pub(super) struct Folha<'a> {
    film: &'a [f32],
    s: usize,
    w: i32,
    h: i32,
}

impl<'a> Folha<'a> {
    pub(super) fn of(g: &'a Grid) -> Self {
        Self {
            film: &g.film,
            s: g.s,
            w: g.w as i32,
            h: g.h as i32,
        }
    }

    #[inline]
    fn molhada(&self, x: i32, y: i32) -> bool {
        x >= 1
            && y >= 1
            && x <= self.w
            && y <= self.h
            && self.film[x as usize + y as usize * self.s] as f64 > DRY_FILM
    }

    /// **Papel seco POR FORA da mancha** — o predicado ÚNICO da linha de contacto e da orla de baixo
    /// (auditoria L1-4: as duas contas discordavam por uma célula, e a água passava por baixo de uma orla).
    /// `(x, y)` seca e as [`FRONT_DRY`] células a jusante secas (fora da folha conta como seco: a água cai
    /// dela), e não uma BRECHA estreita da franja (água dos dois lados na transversal, a até `notch_w`): a
    /// tensão superficial fecha-a. Medido na W1c: na orla recta sem as brechas a encher a faixa escura
    /// abre 110 cunhas claras (0–10 com elas); na encosta do S elas não mudam a borda (37 contra 40 colunas
    /// com um claro na linha da orla, patamares 1 contra 2).
    pub(super) fn por_fora(&self, x: i32, y: i32, nx: f64, ny: f64, notch_w: i32) -> bool {
        if self.molhada(x, y) {
            return false;
        }
        let (mut cx, mut cy) = (x, y);
        let seco = (0..FRONT_DRY).all(|_| {
            (cx, cy) = downhill(cx, cy, nx, ny);
            !self.molhada(cx, cy)
        });
        let lado = |sinal: f64| {
            (1..=notch_w).any(|k| {
                let t = sinal * f64::from(k);
                self.molhada(
                    (f64::from(x) - ny * t).round() as i32,
                    (f64::from(y) + nx * t).round() as i32,
                )
            })
        };
        seco && !(lado(-1.0) && lado(1.0))
    }
}

/// A célula a jusante de `(x, y)` (um passo ao longo da gravidade, arredondado).
#[inline]
pub(super) fn downhill(x: i32, y: i32, nx: f64, ny: f64) -> (i32, i32) {
    (x + nx.round() as i32, y + ny.round() as i32)
}

#[inline]
pub(super) fn inside(g: &Grid, x: i32, y: i32) -> bool {
    x >= 1 && y >= 1 && x <= g.w as i32 && y <= g.h as i32
}

/// É orla de baixo: molhada, e a célula logo a jusante é papel seco POR FORA da mancha
/// ([`Folha::por_fora`], o MESMO predicado que a prende) — não um buraco da franja das cerdas DENTRO do
/// traço (uma gota nascida num buraco cai na água do próprio traço e morre, e a bacia dela calava a orla
/// verdadeira: medido, o funil 10/10 da água 1,0 sumia).
#[inline]
pub(super) fn is_front(g: &Grid, x: i32, y: i32, nx: f64, ny: f64, notch_w: i32) -> bool {
    let f = Folha::of(g);
    let (cx, cy) = downhill(x, y, nx, ny);
    f.molhada(x, y) && f.por_fora(cx, cy, nx, ny, notch_w)
}

/// A célula de orla da coluna transversal `t` de `o`, a mais perto (ao longo da gravidade) de `k0`:
/// `(índice, ao longo)`.
fn orla_na_coluna(
    g: &Grid,
    o: (i32, i32),
    (nx, ny): (f64, f64),
    notch_w: i32,
    t: i32,
    k0: i32,
) -> Option<(usize, i32)> {
    let (bx, by) = (
        f64::from(o.0) - ny * f64::from(t),
        f64::from(o.1) + nx * f64::from(t),
    );
    (0..=2 * EDGE_SEARCH).find_map(|d| {
        let k = k0 + if d % 2 == 0 { d / 2 } else { -(d + 1) / 2 };
        let (x, y) = (
            (bx + nx * f64::from(k)).round() as i32,
            (by + ny * f64::from(k)).round() as i32,
        );
        (inside(g, x, y) && is_front(g, x, y, nx, ny, notch_w))
            .then(|| (x as usize + y as usize * g.s, k))
    })
}

/// **A orla da bacia** de `(ox, oy)`: caminha na transversal para os dois lados até `reach`, e em cada
/// coluna toma a célula de orla mais perto (ao longo da gravidade) da da coluna anterior; mais de
/// [`EDGE_GAP`] colunas seguidas sem orla é papel seco — a bacia acaba ali. Chama `visita` em cada célula
/// de orla (índice, transversal, ao longo) e devolve `[t mín, t máx, k mín, k máx]`.
///
/// ⚠️ Custa `O(reach · EDGE_SEARCH)` por gota — a orla e não a ÁREA (a varredura em caixa que ela
/// substituiu custava 15 ms por passo a 4096² com cem gotas; doc 05 §4b).
pub(super) fn walk_edge(
    g: &Grid,
    o: (i32, i32),
    nx: f64,
    ny: f64,
    notch_w: i32,
    reach: f64,
    mut visita: impl FnMut(usize, i32, i32),
) -> [i32; 4] {
    let r = reach.floor() as i32;
    let mut out = [0, 0, 0, 0];
    let mut coluna = |t: i32, k0: i32| -> Option<i32> {
        let (i, k) = orla_na_coluna(g, o, (nx, ny), notch_w, t, k0)?;
        visita(i, t, k);
        Some(k)
    };
    let k_origem = coluna(0, 0).unwrap_or(0);
    out[2] = k_origem;
    out[3] = k_origem;
    for lado in [-1i32, 1] {
        let (mut k0, mut falhas) = (k_origem, 0);
        for passo in 1..=r {
            let t = lado * passo;
            match coluna(t, k0) {
                Some(k) => {
                    k0 = k;
                    falhas = 0;
                    out[0] = out[0].min(t);
                    out[1] = out[1].max(t);
                    out[2] = out[2].min(k);
                    out[3] = out[3].max(k);
                }
                None => {
                    falhas += 1;
                    if falhas > EDGE_GAP {
                        break;
                    }
                }
            }
        }
    }
    out
}

/// A água ATRÁS da célula de orla `i`: a película somada contra a gravidade até ao papel seco (no máximo
/// `max` células) — o que a orla daquela coluna tem para dar.
fn agua_atras(g: &Grid, i: usize, nx: f64, ny: f64, max: i32) -> f64 {
    let (x0, y0) = ((i % g.s) as f64, (i / g.s) as f64);
    let mut soma = 0.0;
    for k in 0..max {
        let (x, y) = (
            (x0 - nx * f64::from(k)).round() as i32,
            (y0 - ny * f64::from(k)).round() as i32,
        );
        if !inside(g, x, y) {
            break;
        }
        let f = f64::from(g.film[x as usize + y as usize * g.s]);
        if f <= DRY_FILM {
            break;
        }
        soma += f;
    }
    soma
}

/// **A bacia de uma gota nova, pela ÁGUA**: a meia-largura (células, ao longo da orla) que recolhe `v` de
/// água atrás da orla, a partir da origem e para os dois lados, no máximo `reach`. Um traço largo tem mais
/// água atrás de cada coluna e pinga mais junto (o S do Rebelle a 100 px: um fio a cada ~90 px); um fino
/// recolhe de mais longe (a bancada a 40 px: um fio por traço de 150).
pub(super) fn basin_reach(
    g: &Grid,
    o: (i32, i32),
    (nx, ny): (f64, f64),
    notch_w: i32,
    reach: f64,
    v: f64,
) -> f64 {
    // Para os dois lados AO MESMO TEMPO, até juntar `v`: a bacia de uma poça larga junta-a em poucas
    // colunas (medido a 4096²: somar o alcance inteiro de cada gota nova custava 141 ms no passo em que
    // nasciam 544).
    let max = (4.0 * reach).ceil() as i32;
    let (i0, k_origem) =
        orla_na_coluna(g, o, (nx, ny), notch_w, 0, 0).map_or((None, 0), |(i, k)| (Some(i), k));
    let mut soma = i0.map_or(0.0, |i| agua_atras(g, i, nx, ny, max));
    if soma >= v {
        return 1.0;
    }
    // Por lado: a orla da coluna anterior, as colunas seguidas sem orla e se ele ainda anda.
    let mut lados = [(k_origem, 0, true); 2];
    for passo in 1..=reach.floor() as i32 {
        for (lado, (k0, falhas, anda)) in [-1i32, 1].into_iter().zip(lados.iter_mut()) {
            if !*anda {
                continue;
            }
            match orla_na_coluna(g, o, (nx, ny), notch_w, lado * passo, *k0) {
                Some((i, k)) => {
                    (*k0, *falhas) = (k, 0);
                    soma += agua_atras(g, i, nx, ny, max);
                }
                None => {
                    *falhas += 1;
                    *anda = *falhas <= EDGE_GAP;
                }
            }
        }
        if soma >= v {
            return f64::from(passo);
        }
        if !lados[0].2 && !lados[1].2 {
            break;
        }
    }
    reach
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uma coluna `x = 10` de uma folha 20×40: água nos trechos `molhadas` (de, até), o resto seco; tudo
    /// ativo.
    fn folha(molhadas: &[(usize, usize)]) -> Grid {
        let mut g = Grid::new(20, 40);
        g.spans_enabled = false;
        for y in 1..=40 {
            for x in 1..=20 {
                g.active[x + y * g.s] = 1;
            }
        }
        for &(de, ate) in molhadas {
            for y in de..=ate {
                g.film[10 + y * g.s] = 1.0;
            }
        }
        g.expand_bbox(1, 1, 20, 40);
        g
    }

    /// A linha de contacto PRENDE a célula `(10, y)` (zera o fluxo dela)?
    fn presa(g: &mut Grid, y: usize) -> bool {
        g.scratch.ensure(g.cells);
        g.scratch.uv.fill([1.0, 1.0]);
        pin_downhill(g, [0.0, 0.04], DripLaw::DEFAULT.notch_w, Rows::Serial);
        g.scratch.uv[10 + y * g.s] == [0.0, 0.0]
    }

    /// ⭐ **L1-4 — a orla e a linha de contacto são UM predicado**: uma célula molhada é orla de baixo
    /// (dela nasce a gota) se e só se a seca logo abaixo dela está presa (a água não passa por baixo de
    /// uma orla). Os dois casos da auditoria de 08/10 — uma corrida de 8 secas antes de mais água (a orla
    /// nascia e a água passava por baixo dela) e a orla a 2 células do fundo da folha (presa, nunca
    /// pingava) — e a brecha estreita da franja, que enche (nem presa, nem orla).
    #[test]
    fn a_orla_de_baixo_e_a_celula_presa_logo_abaixo_dela() {
        for (nome, molhadas, e) in [
            ("corrida de 8 secas", &[(5, 10), (19, 22)][..], 10usize),
            ("a 2 células do fundo", &[(30, 38)][..], 38),
            ("seco até ao fundo", &[(5, 20)][..], 20),
            ("um buraco da franja", &[(5, 10), (13, 30)][..], 10),
            ("uma brecha estreita", &[(5, 10)][..], 10),
        ] {
            let mut g = folha(molhadas);
            if nome == "uma brecha estreita" {
                // Água dos dois lados da célula de baixo, a 2 colunas: a brecha enche (não é orla).
                for x in [8, 12] {
                    g.film[x + (e + 1) * g.s] = 1.0;
                }
            }
            let orla = is_front(&g, 10, e as i32, 0.0, 1.0, DripLaw::DEFAULT.notch_w);
            let segura = presa(&mut g, e + 1);
            assert_eq!(
                orla, segura,
                "{nome}: orla de baixo = {orla}, a célula de baixo presa = {segura}"
            );
        }
    }
}
