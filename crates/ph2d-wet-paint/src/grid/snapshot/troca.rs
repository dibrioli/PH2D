//! O undo/redo do motor numa passada: o registo que sai da pilha TROCA os seus tiles com a grade nos planos densos que
//! o restauro não redimensiona — o tile passa a ser o do inverso, sem alocar nem largar a memória dele; o resto copia
//! como o [`Registo::inverso`] + [`Registo::restaura_com`], que ficam como oráculo (`ANTIGO`; ADR-0204).

use super::*;
use crate::sonda_fases::marca;

/// B18.11: a linha do tile que já é igual ao bit à da grade não se troca (trocar iguais não muda nenhum dos dois
/// lados) — só se lê. `PH2D_TROCA_POUPA=0` (lida uma vez) troca tudo, como antes.
trait PoupaIgual: Sized {
    fn iguais(a: &[Self], b: &[Self]) -> bool;
}

impl PoupaIgual for f32 {
    #[inline]
    fn iguais(a: &[Self], b: &[Self]) -> bool {
        a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
    }
}

impl PoupaIgual for [f32; 3] {
    #[inline]
    fn iguais(a: &[Self], b: &[Self]) -> bool {
        a.as_flattened()
            .iter()
            .zip(b.as_flattened())
            .all(|(x, y)| x.to_bits() == y.to_bits())
    }
}

impl PoupaIgual for u8 {
    #[inline]
    fn iguais(a: &[Self], b: &[Self]) -> bool {
        a == b
    }
}

/// As células compostas trocam sempre (sem igualdade ao bit barata).
impl PoupaIgual for tinta::CelulaTinta {
    fn iguais(_: &[Self], _: &[Self]) -> bool {
        false
    }
}

impl PoupaIgual for tinta::pigmento::LadoEspectral {
    fn iguais(_: &[Self], _: &[Self]) -> bool {
        false
    }
}

fn poupa() -> bool {
    #[cfg(test)]
    if TROCA_CEGA.with(std::cell::Cell::get) {
        return false;
    }
    static P: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *P.get_or_init(|| std::env::var("PH2D_TROCA_POUPA").map_or(true, |v| v != "0"))
}

#[cfg(test)]
thread_local! {
    /// A troca cega de antes (troca também as linhas iguais), por fio: a régua A/B no MESMO processo.
    pub(crate) static TROCA_CEGA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Linhas de tile poupadas (iguais ao bit), só em teste: o gate de que a poupança corre.
#[cfg(test)]
pub(crate) static POUPADAS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Troca os tiles `ts` de um plano denso com a grade (um job por faixa de tiles). O tile que não tem o plano (vazio)
/// só tira a cópia: o restauro não lhe punha nada.
fn troca_no_pool<'a, T: Clone + Send + PoupaIgual>(
    v: &'a mut [T],
    s: usize,
    ts: Vec<(Ret, &'a mut Vec<T>)>,
    sc: &rayon::Scope<'a>,
) {
    if ts.is_empty() || v.is_empty() {
        return;
    }
    let mut por_faixa: Vec<Vec<(Ret, &'a mut Vec<T>)>> = Vec::new();
    por_faixa.resize_with(v.len().div_ceil(TILE * s), Vec::new);
    for (r, d) in ts {
        por_faixa[r.y0 / TILE].push((r, d));
    }
    let poupa = poupa();
    for ((ty, faixa), tiles) in v.chunks_mut(TILE * s).enumerate().zip(por_faixa) {
        if tiles.is_empty() {
            continue;
        }
        sc.spawn(move |_| {
            let o = ty * TILE * s;
            for (r, d) in tiles {
                let w = r.x1 - r.x0;
                if d.len() == w * (r.y1 - r.y0) {
                    for (i, y) in (r.y0..r.y1).enumerate() {
                        let a = y * s + r.x0 - o;
                        let (g, t) = (&mut faixa[a..a + w], &mut d[i * w..(i + 1) * w]);
                        if poupa && T::iguais(g, t) {
                            #[cfg(test)]
                            POUPADAS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            continue;
                        }
                        g.swap_with_slice(t);
                    }
                } else {
                    d.clear();
                    for y in r.y0..r.y1 {
                        let a = y * s + r.x0 - o;
                        d.extend_from_slice(&faixa[a..a + w]);
                    }
                }
            }
        });
    }
}

/// Os tiles de fluxo (`vel_x`/`vel_y`) trocam com a grade no pool, um job por faixa de tiles em cada plano, com a
/// mesma poupança das linhas iguais (B18.11: 6–7 ms em série a 4K). Plano vazio na grade ⇒ o tile fica vazio (o
/// `tira` de um plano vazio). `PH2D_FLUXO_SERIE=1` (lida uma vez) = o [`troca_ou_copia`] em série, como antes.
fn troca_o_fluxo(
    g: &mut Grid,
    tiles: impl Iterator<Item = (u32, [Vec<f32>; 2])>,
) -> std::collections::BTreeMap<u32, [Vec<f32>; 2]> {
    let (fs, frows) = (g.flow.s, g.flow.rows);
    let mut tiles: Vec<(u32, [Vec<f32>; 2])> = tiles.collect();
    if fluxo_em_serie() {
        for (k, [vx, vy]) in &mut tiles {
            let r = ret_do_tile(*k, fs, frows);
            troca_ou_copia(&mut g.vel_x, fs, r, vx);
            troca_ou_copia(&mut g.vel_y, fs, r, vy);
        }
        return tiles.into_iter().collect();
    }
    let (mut lx, mut ly) = (
        Vec::with_capacity(tiles.len()),
        Vec::with_capacity(tiles.len()),
    );
    for (k, [vx, vy]) in &mut tiles {
        let r = ret_do_tile(*k, fs, frows);
        lx.push((r, vx));
        ly.push((r, vy));
    }
    for (v, l) in [(&g.vel_x, &mut lx), (&g.vel_y, &mut ly)] {
        if v.is_empty() {
            l.drain(..).for_each(|(_, d)| d.clear());
        }
    }
    let (vx, vy) = (&mut g.vel_x, &mut g.vel_y);
    rayon::scope(|sc| {
        troca_no_pool(vx, fs, lx, sc);
        troca_no_pool(vy, fs, ly, sc);
    });
    tiles.into_iter().collect()
}

fn fluxo_em_serie() -> bool {
    #[cfg(test)]
    if FLUXO_EM_SERIE.with(std::cell::Cell::get) {
        return true;
    }
    static S: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *S.get_or_init(|| std::env::var("PH2D_FLUXO_SERIE").is_ok_and(|v| v == "1"))
}

#[cfg(test)]
thread_local! {
    /// O fluxo em série de antes, por fio: a régua A/B no MESMO processo.
    pub(crate) static FLUXO_EM_SERIE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// O mesmo para um plano de fluxo, em série (o caminho de antes, `PH2D_FLUXO_SERIE=1`).
fn troca_ou_copia(v: &mut [f32], s: usize, r: Ret, d: &mut Vec<f32>) {
    let w = r.x1 - r.x0;
    if !v.is_empty() && d.len() == w * (r.y1 - r.y0) {
        for (i, y) in (r.y0..r.y1).enumerate() {
            v[y * s + r.x0..y * s + r.x1].swap_with_slice(&mut d[i * w..(i + 1) * w]);
        }
    } else {
        let novo = tira(v, s, r);
        poe(v, s, r, d);
        *d = novo;
    }
}

macro_rules! planos {
    (densos: $($f:ident : $t:ty $(= $i:literal)?),* ; esparsos: $($e:ident : $te:ty = $ie:literal),* $(,)?) => {
        /// Os planos que trocam: cheios na grade e do comprimento da base (o `ajusta` não lhes mexe).
        #[derive(Clone, Copy)]
        struct Trocam { $($f: bool,)* $($e: bool,)* }

        impl Trocam {
            fn de(g: &Grid, v: &[usize; 9]) -> Self {
                #[cfg(test)]
                let esparsos = !ESPARSOS_COPIAM.with(std::cell::Cell::get);
                #[cfg(not(test))]
                let esparsos = true;
                Trocam {
                    $($f: !g.$f.is_empty() $(&& v[$i] == g.$f.len())?,)*
                    $($e: esparsos && !g.$e.is_empty() && v[$ie] == g.$e.len(),)*
                }
            }
        }

        /// Antes do `ajusta`: os planos que não trocam tiram a cópia do inverso; devolve o do registo, a pôr depois.
        fn copia_os_outros(g: &Grid, r: Ret, t: &mut TileFino, ok: Trocam) -> TileFino {
            let mut pos = TileFino::default();
            $( if !ok.$f { pos.$f = std::mem::replace(&mut t.$f, Plano::tira_tile(&g.$f, g.s, r)); } )*
            // O esparso que troca só copia no tile que não o tem (o restauro não lhe punha nada).
            $( if !ok.$e || t.$e.len() != (r.x1 - r.x0) * (r.y1 - r.y0) {
                pos.$e = std::mem::replace(&mut t.$e, Plano::tira_tile(&g.$e, g.s, r));
            } )*
            pos
        }

        /// Os planos que trocam: cada denso em jobs por faixa de tiles, cada esparso em jobs por fatia de pedaços (as
        /// linhas do tile que têm o plano; as outras já tiraram a cópia).
        fn troca_os_planos(g: &mut Grid, tiles: &mut [(Ret, TileFino)], ok: Trocam) {
            struct Listas<'a> {
                $($f: Vec<(Ret, &'a mut Vec<$t>)>,)*
                $($e: Vec<(usize, &'a mut [$te])>,)*
            }
            let mut l = Listas { $($f: Vec::new(),)* $($e: Vec::new(),)* };
            let s = g.s;
            for (r, t) in tiles.iter_mut() {
                let TileFino { $($f,)* $($e,)* .. } = t;
                $( if ok.$f { l.$f.push((*r, $f)); } )*
                let w = r.x1 - r.x0;
                $( if ok.$e && $e.len() == w * (r.y1 - r.y0) {
                    l.$e.extend($e.chunks_mut(w).enumerate().map(|(i, v)| ((r.y0 + i) * s + r.x0, v)));
                } )*
            }
            rayon::scope(|sc| {
                $( troca_no_pool(&mut g.$f, s, l.$f, sc); )*
                $( g.$e.troca_trechos_poupando(l.$e, <$te as MesmoBit>::mesmo_bit, sc); )*
            });
        }
    };
}

planos!(
    densos: film: f32, susp: f32, susp_rgb: [f32; 3], sett: f32, sett_rgb: [f32; 3], relevo: f32,
    altura: f32 = 0, espessura: f32 = 1, crista: f32 = 2, nivelado: f32 = 3, vela: f32,
    tinta: tinta::CelulaTinta = 7, pigmento: tinta::pigmento::LadoEspectral = 8, wet: u8, active: u8, bloom: u8;
    esparsos: material: crate::material::MaterialCelula = 4, aparencia: aparencia::AparenciaCelula = 5,
    oleo: oleo_celula::OleoCelula = 6,
);

#[cfg(test)]
thread_local! {
    /// Os planos esparsos copiam como antes (a troca só dos densos), por fio: a régua A/B no MESMO processo.
    pub(crate) static ESPARSOS_COPIAM: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

impl Registo {
    /// O `inverso(g, self, extra)` + `restaura_com(g, extra, zera_o_fluxo)`, ao bit, numa passada: os tiles de `self` e
    /// de `extra` passam para o inverso (os registos ficam sem tiles). Devolve o inverso e se o papel mudou.
    pub fn troca(
        &mut self,
        g: &mut Grid,
        extra: Option<&mut Registo>,
        zera_o_fluxo: bool,
    ) -> (Registo, bool) {
        #[cfg(test)]
        if ANTIGO.with(std::cell::Cell::get) {
            let inv = Self::inverso(g, self, extra.as_deref());
            let p = self.restaura_com(g, extra.as_deref(), zera_o_fluxo);
            return (inv, p);
        }
        let mut out = Registo::abre(g);
        // Os tiles na ordem do restauro: os do registo, depois os do `extra` que ele não tem.
        let (ex_finos, ex_fluxo) = extra.map_or_else(Default::default, |e| {
            (std::mem::take(&mut e.finos), std::mem::take(&mut e.fluxo))
        });
        let ex_finos: Vec<_> = ex_finos
            .into_iter()
            .filter(|(k, _)| !self.finos.contains_key(k))
            .collect();
        let ex_fluxo: Vec<_> = ex_fluxo
            .into_iter()
            .filter(|(k, _)| !self.fluxo.contains_key(k))
            .collect();
        let (s, rows) = (g.s, g.rows);
        let (ks, mut tiles): (Vec<u32>, Vec<(Ret, TileFino)>) = std::mem::take(&mut self.finos)
            .into_iter()
            .chain(ex_finos)
            .map(|(k, t)| (k, (ret_do_tile(k, s, rows), t)))
            .unzip();
        let v = self.base.vazios;
        macro_rules! amostra {
            ($f:ident) => {
                tiles.iter().find_map(|(_, t)| t.$f.first().cloned())
            };
        }
        let (a0, a1, a2, a3) = (
            amostra!(altura),
            amostra!(espessura),
            amostra!(crista),
            amostra!(nivelado),
        );
        let (a5, a6, a7) = (amostra!(material), amostra!(aparencia), amostra!(oleo));
        let (a8, a9) = (amostra!(tinta), amostra!(pigmento));
        let ok = Trocam::de(g, &v);
        marca("troca: tiles para a lista");
        #[cfg(any(test, feature = "sonda-fases"))]
        if crate::sonda_fases::censo_ligado() {
            for (r, t) in &tiles {
                t.censo(g, *r);
            }
            marca(crate::sonda_fases::CENSO_FASE);
        }
        let por: Vec<TileFino> = {
            let g = &*g;
            tiles
                .par_iter_mut()
                .map(|(r, t)| copia_os_outros(g, *r, t, ok))
                .collect()
        };
        marca("troca: copia_os_outros");
        ajusta(&mut g.altura, v[0], a0.as_ref());
        ajusta(&mut g.espessura, v[1], a1.as_ref());
        ajusta(&mut g.crista, v[2], a2.as_ref());
        ajusta(&mut g.nivelado, v[3], a3.as_ref());
        ajusta_campo(&mut g.material, v[4], a5.is_some());
        ajusta_campo(&mut g.aparencia, v[5], a6.is_some());
        ajusta_campo(&mut g.oleo, v[6], a7.is_some());
        ajusta(&mut g.tinta, v[7], a8.as_ref());
        ajusta(&mut g.pigmento, v[8], a9.as_ref());
        marca("troca: ajusta");
        troca_os_planos(g, &mut tiles, ok);
        marca("troca: troca_os_planos");
        let ts: Vec<(Ret, &TileFino)> = tiles.iter().map(|(r, _)| *r).zip(&por).collect();
        TileFino::poe_no_pool(g, &ts);
        drop(ts);
        drop(por);
        marca("troca: poe_no_pool + larga");
        super::super::aparencia::reconta_teto(g);
        super::super::oleo_celula::reconta_real(g);
        marca("troca: reconta");
        out.fluxo = troca_o_fluxo(
            g,
            std::mem::take(&mut self.fluxo).into_iter().chain(ex_fluxo),
        );
        marca("troca: fluxo");
        out.finos = ks.into_iter().zip(tiles.into_iter().map(|t| t.1)).collect();
        if zera_o_fluxo {
            g.flow_x.fill(0.0);
            g.flow_y.fill(0.0);
        }
        let b = &self.base;
        g.bx0 = b.bx0;
        g.by0 = b.by0;
        g.bx1 = b.bx1;
        g.by1 = b.by1;
        g.has_fluid = b.has_fluid;
        g.row_lo.copy_from_slice(&b.row_lo);
        g.row_hi.copy_from_slice(&b.row_hi);
        g.fill_tiles_all();
        marca("troca: finos + escalares");
        let paper_changed = g.paper_preset != b.paper_preset || g.paper_sheet != b.paper_sheet;
        g.paper_preset = b.paper_preset;
        g.paper_sheet = b.paper_sheet;
        (out, paper_changed)
    }
}
