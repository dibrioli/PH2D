//! **O AVENTAL do núcleo de três caixas** — a entrada da horizontal, lida do canvas e
//! PREMULTIPLICADA, e a saída desfeita. Cortado do [`super`] pelo tecto de LOC quando a fila 44
//! (item 10) fundiu o avental na horizontal e o desfazer na vertical: aqui moram as três portas que
//! as duas rotas partilham, ao bit.

use super::super::blur::src_coord;

/// **De onde a horizontal lê cada linha** (fila 44, item 10).
///
/// * `Plano` — um avental já materializado (o oráculo, as sondas e a porta de bissecção).
/// * `Tela` — a linha `j` do avental é CONSTRUÍDA num rascunho por trabalhador, pela mesma
///   [`linha_do_avental`] que a [`avental`] corre: o buffer inteiro do avental (`16 B/px`, escrito
///   uma vez e lido outra) deixa de existir, e cada linha continua a ser os mesmos quatro `f32`.
pub(crate) enum FonteH<'a> {
    Plano(&'a [[f32; 4]]),
    Tela(&'a (dyn Fn(usize, &mut [[f32; 4]]) + Sync)),
}

/// **O AVENTAL**: a região mais a margem que as três caixas vão consumir, lida do canvas e já
/// PREMULTIPLICADA. ⭐ Ele é por-pixel puro — *uma linha dele não lê nenhuma outra* —, logo parti-lo
/// em fatias é byte-idêntico (ADR-0171).
///
/// ⚠️ **Porta própria desde 2026-09-22, e não por estética:** a sonda que parte o relógio do borrão
/// entre as passagens (`diag_onde_o_borrao_gasta`) precisava de o cronometrar sozinho, e a
/// alternativa era **replicá-lo** no teste — *uma sonda que replica o passo que mede pode medir
/// outro programa, que é como as sondas desta casa já mentiram meia dúzia de vezes*.
#[allow(clippy::too_many_arguments)]
pub(crate) fn avental(
    buf: &[u8],
    fw: i64,
    fh: i64,
    min_x: i64,
    min_y: i64,
    ap_w: usize,
    ap_h: usize,
    r_total: usize,
    wrap: [bool; 2],
    paralelo: bool,
) -> Vec<[f32; 4]> {
    super::conta::AVENTAIS_INTEIROS.set(super::conta::AVENTAIS_INTEIROS.get() + 1);
    let mut apron = vec![[0f32; 4]; ap_w * ap_h];
    let linha_avental = |j: usize, dest: &mut [[f32; 4]]| {
        linha_do_avental(buf, fw, fh, min_x, min_y, r_total, wrap, j, dest);
    };
    if paralelo {
        use rayon::prelude::*;
        apron
            .par_chunks_mut(ap_w)
            .enumerate()
            .for_each(|(j, dest)| linha_avental(j, dest));
    } else {
        apron
            .chunks_mut(ap_w)
            .enumerate()
            .for_each(|(j, dest)| linha_avental(j, dest));
    }
    apron
}

/// **A linha `j` do avental**, lida do canvas e PREMULTIPLICADA — a porta ÚNICA da conta, com
/// dois chamadores: o avental materializado ([`avental`]) e a horizontal que o constrói linha a
/// linha ([`FonteH::Tela`]). Uma linha dela não lê nenhuma outra.
#[allow(clippy::too_many_arguments)]
pub(crate) fn linha_do_avental(
    buf: &[u8],
    fw: i64,
    fh: i64,
    min_x: i64,
    min_y: i64,
    r_total: usize,
    wrap: [bool; 2],
    j: usize,
    dest: &mut [[f32; 4]],
) {
    let sy = src_coord(min_y + j as i64 - r_total as i64, fh, wrap[1]);
    let s = escala_do_avental(r_total);
    for (i, d) in dest.iter_mut().enumerate() {
        let sx = src_coord(min_x + i as i64 - r_total as i64, fw, wrap[0]);
        let si = ((sy * fw + sx) * 4) as usize;
        // Em INTEIROS (`c·α·s`, `α·255·s`): a soma corrente das caixas fica exacta (B24.1).
        let a = f32::from(buf[si + 3]) * s;
        *d = [
            f32::from(buf[si]) * a,
            f32::from(buf[si + 1]) * a,
            f32::from(buf[si + 2]) * a,
            a * 255.0,
        ];
    }
}

/// **A escala INTEIRA do avental** de um borrão de alcance `r_total` (B24.1): a maior `s` que deixa
/// a soma de uma caixa — `lado ≤ 2⌈r_total/3⌉ + 1` valores `≤ 65 025·s` — abaixo de `2²⁴`, onde o
/// `f32` soma inteiros sem erro. Quanto maior `s`, mais fino o arredondamento de cada caixa.
#[must_use]
pub(crate) fn escala_do_avental(r_total: usize) -> f32 {
    let lado = 2 * r_total.div_ceil(3) + 1;
    #[allow(clippy::cast_precision_loss)]
    let s = ((1usize << 24) / (65_025 * lado)).max(1) as f32;
    s
}

/// **Desfazer a premultiplicação** — a porta ÚNICA, com dois chamadores: a passagem inteira da
/// rota separada e a escrita final da vertical fundida ([`caixa_v3_com`]). Da escala do avental
/// (cor `c·α·s`, alfa `α·255·s`) de volta a recto `0..=255`.
#[must_use]
pub(crate) fn desfaz(r_total: usize) -> impl Fn(&mut [f32; 4]) + Sync + Send {
    let s = escala_do_avental(r_total);
    move |p: &mut [f32; 4]| {
        let a = p[3] / (255.0 * s);
        let inv = if a > 1e-4 { 1.0 / (a * s) } else { 0.0 };
        *p = [p[0] * inv, p[1] * inv, p[2] * inv, a];
    }
}
