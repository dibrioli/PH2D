//! O lado ESPECTRAL da tinta (E10b, «Pigment mix»): por célula `[filme, aberto]` de [`Espectro`] (112 B), num vector
//! LATERAL que só nasce quando um pincel com `p > 0` pousa ([`garante`]); com `p = 0` em toda a tela não existe e a
//! cor é a de [`super::srgb_0_255`] AO BIT. Ainda SEM campo em `Grid` (o `grid.rs` e as portas de transferência
//! são da ligação seguinte): aqui vivem as operações por célula que essa ligação chama. Doc:
//! `docs/Painter/Arquitetura/E10b_pigment_mix.md`.

use super::{CelulaTinta, Grid};
use ph2d_paint_tinta::oklab::oklab_para_srgb;
use ph2d_paint_tinta::{
    Bandas, Espectro, Optica, Soma, Tinta, cor_pigmento, optica_de_rinf, s_de_transparencia,
    transfere_pigmento,
};

/// O lado espectral de uma célula: `[filme, aberto]`, pesado pelos MESMOS volumes de [`CelulaTinta`].
pub type LadoEspectral = [Espectro; 2];

/// O lado espectral de uma célula que já tinha tinta: as cores dela com `p = 0` (a única subida às bandas).
#[must_use]
pub fn migra(c: &CelulaTinta) -> LadoEspectral {
    [Espectro::da_soma(&c.filme), Espectro::da_soma(&c.aberto)]
}

/// O campo lateral; se ainda não existia, nasce migrado da tinta que a grade tem.
pub fn garante<'a>(
    lado: &'a mut Vec<LadoEspectral>,
    tinta: &[CelulaTinta],
) -> &'a mut [LadoEspectral] {
    if lado.is_empty() {
        *lado = tinta.iter().map(migra).collect();
    }
    lado
}

/// Move a fracção `f` da fatia `de` para `para` na tinta E no lado espectral (vazio = só a tinta, como hoje).
pub fn transfere(
    de: (&mut Soma, Option<&mut Espectro>),
    para: (&mut Soma, Option<&mut Espectro>),
    f: f32,
) {
    match (de.1, para.1) {
        (Some(ed), Some(ep)) => {
            let _ = transfere_pigmento((de.0, ed), (para.0, ep), f);
        }
        (None, None) => {
            let _ = ph2d_paint_tinta::transfere(de.0, para.0, f);
        }
        _ => unreachable!("o lado espectral existe para a grade inteira ou para nenhuma célula"),
    }
}

/// A cor sRGB `0..255` da fatia com o lado espectral (o irmão de [`super::srgb_0_255`]).
#[must_use]
pub fn srgb_0_255(s: &Soma, e: Option<&Espectro>) -> Option<[f32; 3]> {
    let c = match e {
        Some(e) => cor_pigmento(s, e)?,
        None => s.cor()?,
    };
    Some(oklab_para_srgb(c.lab).map(|x| (x * 255.0).clamp(0.0, 255.0)))
}

// ---- E10c: a ligação ao motor (o campo `Grid::pigmento`, as portas por índice) ----

/// Índice da fatia no [`LadoEspectral`].
pub const FILME: usize = 0;
/// Índice da fatia no [`LadoEspectral`].
pub const ABERTO: usize = 1;

/// O campo lateral existe (a grade INTEIRA o tem).
#[inline]
#[must_use]
pub fn existe(g: &Grid) -> bool {
    !g.pigmento.is_empty()
}

/// O `Espectro` de `massa` (unidades de `susp`) da tinta sRGB `0..255` — o par de [`super::soma_da_massa`].
#[must_use]
pub fn de_massa(cor_srgb_0_255: [f32; 3], transparencia: f32, massa: f32, p: f32) -> Espectro {
    let t = Tinta::de_srgb(cor_srgb_0_255.map(|c| c / 255.0), transparencia);
    Espectro::de(t, (massa / super::M_UM).max(0.0), p)
}

/// Antes de um depósito com Pigment mix `p`: `p > 0` aloca o campo (migrado da tinta de ANTES do depósito).
#[inline]
pub fn garante_para(g: &mut Grid, p: f32) {
    if p > 0.0 && g.pigmento.is_empty() && g.tinta.len() == g.cells {
        let _ = garante(&mut g.pigmento, &g.tinta);
    }
}

/// Move a fracção `f` do `Espectro` `de = (célula, fatia)` para `para` (`None` = sai da grade). Nada sem o campo.
#[inline]
pub fn move_em(g: &mut Grid, de: (usize, usize), para: Option<(usize, usize)>, f: f32) {
    if g.pigmento.is_empty() || f <= 0.0 || Some(de) == para {
        return;
    }
    let mut a = g.pigmento[de.0][de.1];
    match para {
        Some((j, k)) => {
            let _ = Espectro::transfere(&mut a, &mut g.pigmento[j][k], f);
        }
        None => {
            let _ = Espectro::transfere(&mut a, &mut Espectro::default(), f);
        }
    }
    g.pigmento[de.0][de.1] = a;
}

/// Pousa `e` na fatia `(i, k)`: soma-o, ou SUBSTITUI (a fatia que a soma também substitui). Nada sem o campo.
#[inline]
pub fn pousa(g: &mut Grid, (i, k): (usize, usize), e: Espectro, substitui: bool) {
    if let Some(l) = g.pigmento.get_mut(i) {
        l[k] = if substitui { e } else { l[k].junta(e) };
    }
}

/// `Σ coefₖ · espectroₖ` em `f64` (o par do `combina` do gather linear).
#[must_use]
pub fn combina(termos: &[(Espectro, f64)]) -> Espectro {
    let mut acc = [0.0f64; 14];
    for (e, c) in termos {
        for (a, x) in acc
            .iter_mut()
            .zip(e.ln_r.iter().chain(&e.delta).chain([&e.p]))
        {
            *a += f64::from(*x) * c;
        }
    }
    Espectro {
        ln_r: core::array::from_fn(|i| acc[i] as f32),
        delta: core::array::from_fn(|k| acc[10 + k] as f32),
        p: acc[13].max(0.0) as f32,
    }
}

/// `a + (b − a)·d` por componente (o arrasto, par do `fatia` de `deposito_tinta::arrasta`).
#[must_use]
pub fn lerp(a: Espectro, b: Espectro, d: f32) -> Espectro {
    let l = |x: f32, y: f32| x + (y - x) * d;
    Espectro {
        ln_r: core::array::from_fn(|i| l(a.ln_r[i], b.ln_r[i])),
        delta: core::array::from_fn(|k| l(a.delta[k], b.delta[k])),
        p: l(a.p, b.p).max(0.0),
    }
}

/// A óptica da célula (filme e aberto juntos, como [`ph2d_paint_tinta::optica_celula`]) pela cor do Pigment mix;
/// sem o lado é a de hoje AO BIT.
#[must_use]
pub fn optica_celula(
    filme: &Soma,
    aberto: &Soma,
    lado: Option<&LadoEspectral>,
    m: f32,
    molhado: bool,
) -> Optica<3> {
    let Some(l) = lado else {
        return ph2d_paint_tinta::optica_celula(filme, aberto, m, molhado);
    };
    let s = filme.junta(*aberto);
    match cor_pigmento(&s, &l[FILME].junta(l[ABERTO])) {
        None => Optica {
            k: Bandas::default(),
            s: Bandas::default(),
        },
        Some(c) => optica_de_rinf(
            Bandas(c.linear()),
            s_de_transparencia(c.transparencia),
            s.v,
            m,
            molhado,
        ),
    }
}

/// O espelho sRGB `0..255` das duas fatias da célula `i` (`None` = sem pigmento na fatia).
#[must_use]
pub fn espelho(g: &Grid, i: usize) -> [Option<[f32; 3]>; 2] {
    let c = super::em(g, i);
    let l = g.pigmento.get(i);
    [
        srgb_0_255(&c.filme, l.map(|l| &l[FILME])),
        srgb_0_255(&c.aberto, l.map(|l| &l[ABERTO])),
    ]
}

/// Devolve a janela de um recorte, DEPOIS de [`super::restaura_janela`]: `linhas` vazio com o campo vivo ⇒ a janela
/// nasce migrada da tinta já restaurada; o campo que nasce aqui nasce migrado fora da janela.
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[LadoEspectral]) {
    if g.pigmento.is_empty() {
        if linhas.is_empty() || g.tinta.len() != g.cells {
            return;
        }
        let _ = garante(&mut g.pigmento, &g.tinta);
    }
    let (n, s) = (g.cells, g.s);
    let neutro = [Espectro::default(); 2];
    super::super::altura::restaura_campo(&mut g.pigmento, n, s, janela, linhas, neutro);
    if linhas.is_empty() {
        let (x0, y0, x1, y1) = janela;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let i = y as usize * s + x as usize;
                g.pigmento[i] = g.tinta.get(i).map_or(neutro, migra);
            }
        }
    }
}

/// Σ de cada componente do `Espectro` sobre a grade (f64): a régua da conservação.
#[must_use]
pub fn soma_total(g: &Grid) -> [f64; 14] {
    let mut acc = [0.0f64; 14];
    for l in &g.pigmento {
        for e in l {
            for (a, x) in acc
                .iter_mut()
                .zip(e.ln_r.iter().chain(&e.delta).chain([&e.p]))
            {
                *a += f64::from(*x);
            }
        }
    }
    acc
}

#[cfg(test)]
#[path = "pigmento_tests.rs"]
mod motor_tests;

#[cfg(test)]
mod tests {
    use super::super::{soma_da_massa, srgb_0_255 as srgb_hoje};
    use super::*;

    /// Com `p = 0` o lado espectral (nascido por migração ou pousado) dá a cor de hoje AO BIT, também depois de mover.
    #[test]
    fn tinta_pigmento_p0_igual_ao_bit_no_motor() {
        let cores = [
            [40.0, 90.0, 220.0],
            [245.0, 205.0, 25.0],
            [0.0; 3],
            [255.0; 3],
            [200.0, 30.0, 60.0],
        ];
        let mut tinta: Vec<CelulaTinta> = cores
            .iter()
            .map(|&c| CelulaTinta {
                filme: soma_da_massa(c, 0.3, 0.0, 600.0),
                aberto: soma_da_massa(c, 0.1, 1.0, 250.0),
                ..CelulaTinta::NEUTRA
            })
            .collect();
        let mut lado = Vec::new();
        let _ = garante(&mut lado, &tinta);
        for i in 0..cores.len() - 1 {
            let (a, b) = tinta.split_at_mut(i + 1);
            let (la, lb) = lado.split_at_mut(i + 1);
            transfere(
                (&mut a[i].filme, Some(&mut la[i][0])),
                (&mut b[0].filme, Some(&mut lb[0][0])),
                0.4,
            );
        }
        for (c, l) in tinta.iter().zip(&lado) {
            for (s, e) in [(&c.filme, &l[0]), (&c.aberto, &l[1])] {
                let (x, y) = (srgb_0_255(s, Some(e)), srgb_hoje(s));
                assert_eq!(
                    x.map(|v| v.map(f32::to_bits)),
                    y.map(|v| v.map(f32::to_bits))
                );
            }
        }
        assert_eq!(core::mem::size_of::<LadoEspectral>(), 112);
    }
}
