//! As leis da BORDA do Óleo Real: a cobertura e o albedo por célula, o contorno sub-célula (`borda_suave` — só o meio
//! plano é escada), os entalhes e a silhueta macia (partido de `oleo_real.rs`, 29/10).

use super::*;

/// A cor LINEAR da célula `i` sobre o papel branco: as camadas visíveis por cima umas das outras pela cobertura
/// (a lei [`crate::assenta::Ecra::Porto`]; o rebote só precisa do tom da parede). A Transparency NÃO entra aqui (o
/// campo não depende do cartão além da sua chave, `oleo_cache`): o rebote clareia a parede transparente.
pub(super) fn albedo_da_celula(camadas: &[(&Grid, bool)], i: usize) -> [f32; 3] {
    let (mut p, mut pa) = ([0.0f64; 3], 0.0f64);
    for &(lg, visivel) in camadas {
        if !visivel {
            continue;
        }
        let a_s = crate::opacity::alpha_of_mass(f64::from(lg.sett[i]));
        let a_f = crate::opacity::alpha_of_mass(f64::from(lg.susp[i]));
        if a_s + a_f * (1.0 - a_s) <= 0.0 {
            continue;
        }
        let (c, la) = crate::assenta::cor_e_alfa(
            crate::assenta::Ecra::Porto.com_vela(lg.vela[i]),
            lg.sett_rgb[i].map(f64::from),
            lg.susp_rgb[i].map(f64::from),
            a_s,
            a_f,
        );
        for k in 0..3 {
            p[k] = p[k] * (1.0 - la) + c[k] * la;
        }
        pa += la * (1.0 - pa);
    }
    if pa <= 0.0 {
        return [1.0; 3];
    }
    p.map(|v| (srgb255_to_linear(v / pa) * pa + (1.0 - pa)) as f32)
}

/// A cobertura das camadas visíveis da célula `i`, umas por cima das outras (`0` no papel sem pigmento).
pub(super) fn cobertura_da_celula(camadas: &[(&Grid, bool)], i: usize) -> f64 {
    let mut cobre = 0.0;
    for &(lg, visivel) in camadas {
        if visivel {
            let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
            cobre += crate::opacity::alpha_of_mass(m) * (1.0 - cobre);
        }
    }
    cobre
}

/// A marca do nivelamento ([`crate::grid::nivelado`], K7) na janela `(ax0, ay0, w, h)` — a maior das camadas
/// visíveis — e o PESO dela, `marca × cobertura`. `None` quando nenhuma a tem: a forma de antes, ao bit.
pub(super) fn marca_do_nivelado(
    rep: &impl Reparte,
    camadas: &[(&Grid, bool)],
    (ax0, ay0, w, h): (usize, usize, usize, usize),
) -> Option<(Vec<f64>, Vec<f64>)> {
    if !camadas.iter().any(|&(lg, v)| v && !lg.nivelado.is_empty()) {
        return None;
    }
    let s = camadas[0].0.s;
    let (mut mu, mut peso) = (vec![0f64; w * h], vec![0f64; w * h]);
    rep.por_linhas(w, &mut mu, &mut peso, &|y, mu, peso| {
        for x in 0..w {
            let i = (ax0 + x) + (ay0 + y) * s;
            let mut cobre = 0.0;
            for &(lg, visivel) in camadas {
                if visivel {
                    mu[x] = mu[x].max(f64::from(crate::grid::nivelado::em(lg, i)));
                    let m = f64::from(lg.sett[i]) + f64::from(lg.susp[i]);
                    cobre += crate::opacity::alpha_of_mass(m) * (1.0 - cobre);
                }
            }
            peso[x] = mu[x] * cobre;
        }
    });
    Some((mu, peso))
}

/// ⭐ **A parede nivelada fica viva** (W3 K7, `49_realshader/oraculo/README.md` §K7): a massa (já `× cobertura`)
/// reparte-se pela marca `mu`. A parte por marcar leva o desfoque largo de sempre (`desfoque`); a marcada é a
/// ESPESSURA média normalizada pelo peso `mu × cobertura` — `desfoque(massa·mu) ÷ desfoque(peso) · peso` —, que alisa
/// o miolo do planalto sem o espalhar para fora dele nem puxar para dentro a orla fina: a parede é a da cobertura
/// (1–2 células, o Rebelle ~2 px) em vez das ~10 do desfoque largo.
pub(super) fn forma_viva(
    massa: &[f64],
    (mu, peso): (&[f64], &[f64]),
    desfoque: &dyn Fn(&[f64]) -> Vec<f64>,
) -> Vec<f64> {
    let livre: Vec<f64> = massa.iter().zip(mu).map(|(m, u)| m * (1.0 - u)).collect();
    let presa: Vec<f64> = massa.iter().zip(mu).map(|(m, u)| m * u).collect();
    let (livre, presa, suporte) = (desfoque(&livre), desfoque(&presa), desfoque(peso));
    (0..massa.len())
        .map(|k| {
            let viva = if suporte[k] > 1e-9 {
                presa[k] / suporte[k] * peso[k]
            } else {
                0.0
            };
            livre[k] + viva
        })
        .collect()
}

/// O raio (células) da DIRECÇÃO da orla: o gradiente da silhueta pesado por uma gaussiana desta meia largura.
pub(super) const RAIO_DIRECCAO: isize = 3;
/// Meio comprimento (células) da média AO LONGO da orla: cobre o degrau de uma borda a ~5° (12 células).
pub(super) const MEIO_TANGENTE: isize = 6;

/// ⭐ §BUG-bordas (smoke do dono 27/10, `oraculo/README.md` §BUG-bordas): **a borda é contínua a sub-célula.** A
/// silhueta (cobertura ≥ ½) é uma escada de células; o contorno LISO dela é a média da silhueta (bilinear) AO
/// LONGO da orla — a tangente vem do gradiente da silhueta pesado numa gaussiana — e essa média é a fracção da
/// célula do lado da tinta (`0,5` no contorno), que se inverte na distância do centro da célula ao contorno. A
/// escada vira um contorno que só CORTA tinta (o papel sem pigmento continua papel, o gate
/// `o_relevo_nao_sai_do_pigmento`). Fora da orla e na borda alinhada aos eixos é `1` ao bit; vazio sem orla.
pub(super) fn borda_suave(rep: &impl Reparte, cob: &[f64], w: usize, h: usize) -> Vec<f32> {
    let em = |x: isize, y: isize| {
        cob[x.clamp(0, w as isize - 1) as usize + y.clamp(0, h as isize - 1) as usize * w]
    };
    pub(super) const LADO: usize = (2 * RAIO_DIRECCAO + 1) as usize;
    let mut gauss = [0.0f64; LADO * LADO];
    for (k, g) in gauss.iter_mut().enumerate() {
        let (dx, dy) = (
            (k % LADO) as f64 - RAIO_DIRECCAO as f64,
            (k / LADO) as f64 - RAIO_DIRECCAO as f64,
        );
        *g = libm::exp(-(dx * dx + dy * dy) / (RAIO_DIRECCAO * RAIO_DIRECCAO) as f64);
    }
    let mut v = vec![1f32; w * h];
    por_linha(rep, w, &mut v, &|y, l| {
        for (x, q) in l.iter_mut().enumerate() {
            let (xi, yi) = (x as isize, y as isize);
            let propria = cob[x + y * w];
            // A silhueta DESTA célula: as que cobrem pelo menos metade do que ela cobre. Numa borda que a massa já
            // gradua (a cobertura a descer devagar por células, ou a variar ao longo dela) as vizinhas estão todas
            // dentro e nada muda; só o DEGRAU (metade ou mais a cair numa célula) tem escada. A orla: a célula da
            // tinta (≥ ½) com uma vizinha das 8 fora da silhueta dela.
            if propria < 0.5 {
                continue;
            }
            let lim = 0.5 * propria;
            let (x0, x1, y0, y1) = (
                x.saturating_sub(1),
                (x + 1).min(w - 1),
                y.saturating_sub(1),
                (y + 1).min(h - 1),
            );
            if !(y0..=y1).any(|yy| cob[x0 + yy * w..=x1 + yy * w].iter().any(|&c| c < lim)) {
                continue;
            }
            let m = |x: isize, y: isize| if em(x, y) >= lim { 1.0 } else { 0.0 };
            let bil = |x: f64, y: f64| {
                let (fx, fy) = (exato::floor(x), exato::floor(y));
                let (tx, ty) = (x - fx, y - fy);
                let (ix, iy) = (fx as isize, fy as isize);
                let l0 = m(ix, iy) + (m(ix + 1, iy) - m(ix, iy)) * tx;
                let l1 = m(ix, iy + 1) + (m(ix + 1, iy + 1) - m(ix, iy + 1)) * tx;
                l0 + (l1 - l0) * ty
            };
            let (mut gx, mut gy, mut fora_g, mut soma_g) = (0.0, 0.0, 0.0, 0.0);
            for (k, g) in gauss.iter().enumerate() {
                let (dx, dy) = (
                    (k % LADO) as isize - RAIO_DIRECCAO,
                    (k / LADO) as isize - RAIO_DIRECCAO,
                );
                let dentro = m(xi + dx, yi + dy);
                let p = g * dentro;
                gx += p * dx as f64;
                gy += p * dy as f64;
                fora_g += g * (1.0 - dentro);
                soma_g += g;
            }
            let n = exato::sqrt(gx * gx + gy * gy);
            // §BUG-bordas-W3b: o contorno liso é o de uma BORDA (meio plano); um furo ou uma fenda de uma célula no meio
            // da tinta tem pouco «fora» na janela e não é borda — cortá-lo pela inversão da recta abria um entalhe à
            // volta do furo (a tinta escura em volta dos pontinhos).
            if n < 1e-9 || fora_g < orla_fora_min() * soma_g {
                continue;
            }
            // J6: a escada de um contorno liso é um MEIO PLANO amostrado; o grão de 1 px da tinta (a orla rasgada
            // da faca, o miolo granulado) não é — alisá-lo derretia o grão em manchas.
            if !e_meio_plano(&gauss, &m, (xi, yi), (gx / n, gy / n), soma_g) {
                continue;
            }
            let (tx, ty) = (-gy / n, gx / n);
            let (mut soma, mut peso) = (0.0, 0.0);
            for s in -MEIO_TANGENTE..=MEIO_TANGENTE {
                let p = (MEIO_TANGENTE + 1 - s.abs()) as f64;
                soma += p * bil(x as f64 + s as f64 * tx, y as f64 + s as f64 * ty);
                peso += p;
            }
            // A média ao longo cresce `≈ 1 − ½(1 − d/0,85)²` com a distância `d` do centro ao contorno (medido sobre
            // rectas de declive 0,05–1): invertida, a fracção da célula dentro dele.
            let fora = (1.0 - soma / peso).clamp(0.0, 0.5);
            *q = (1.0 - exato::sqrt(2.0 * fora)) as f32;
        }
    });
    if v.iter().any(|&b| b < 1.0) {
        v
    } else {
        Vec::new()
    }
}

/// O erro máximo (fracção do peso da gaussiana da direcção) do melhor MEIO PLANO perpendicular ao gradiente contra a
/// silhueta da janela para a célula ser a escada de um contorno liso no [`borda_suave`]. A recta amostrada dá `0`
/// (a silhueta de uma recta É o meio plano nos centros), o canto e a curva pouco; o grão de 1 px muito. Medido (J6,
/// régua do Rebelle /454): 0,05 · 0,10 · 0,20 = Knife Stain 278 · 282 · 296, Knife Oily 334 · 336 · —; a 0,20 o grão
/// fixo derrete (RMS 0,094) e voltam 4 pontinhos à orla do Dabber.
pub(crate) const ESCADA_ERRO_MAX: f64 = 0.10;

#[cfg(test)]
thread_local! {
    /// (Testes) o [`ESCADA_ERRO_MAX`] de uma variante medida no MESMO processo; `Some(∞)` desliga a lei.
    pub(crate) static ESCADA_ERRO_DA_MEDIDA: core::cell::Cell<Option<f64>> = const { core::cell::Cell::new(None) };
}

#[inline]
pub(super) fn escada_erro_max() -> f64 {
    #[cfg(test)]
    if let Some(v) = ESCADA_ERRO_DA_MEDIDA.with(core::cell::Cell::get) {
        return v;
    }
    ESCADA_ERRO_MAX
}

/// A silhueta `m` na janela da gaussiana à volta de `(xi, yi)` é um meio plano de normal `(nx, ny)` a menos de
/// [`ESCADA_ERRO_MAX`] do peso `soma_g`: as células ordenadas pela projecção na normal, o limiar que menos erra.
pub(super) fn e_meio_plano(
    gauss: &[f64],
    m: &dyn Fn(isize, isize) -> f64,
    (xi, yi): (isize, isize),
    (nx, ny): (f64, f64),
    soma_g: f64,
) -> bool {
    pub(super) const LADO: usize = (2 * RAIO_DIRECCAO + 1) as usize;
    let mut v = [(0.0f64, 0.0f64, 0.0f64); LADO * LADO];
    for (k, (e, &g)) in v.iter_mut().zip(gauss).enumerate() {
        let (dx, dy) = (
            (k % LADO) as isize - RAIO_DIRECCAO,
            (k / LADO) as isize - RAIO_DIRECCAO,
        );
        *e = (dx as f64 * nx + dy as f64 * ny, g, m(xi + dx, yi + dy));
    }
    v.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
    // Os k primeiros dentro: erra o papel entre eles e a tinta depois deles.
    let mut erro: f64 = v.iter().map(|&(_, g, d)| g * d).sum();
    let mut min = erro;
    for &(_, g, d) in &v {
        erro += g * (1.0 - d) - g * d;
        min = min.min(erro);
    }
    min <= escada_erro_max() * soma_g
}

/// O entalhe mais raso que se enche (fracção da cobertura): `oraculo/README.md` §BUG-bordas-W3b.
pub(crate) const ENTALHE_MIN: f64 = 0.10;

#[cfg(test)]
thread_local! {
    /// (Testes) o [`ENTALHE_MIN`] de uma variante medida no MESMO processo; `Some(∞)` desliga a lei.
    pub(crate) static ENTALHE_DA_MEDIDA: core::cell::Cell<Option<f64>> = const { core::cell::Cell::new(None) };
}

/// A fracção mínima de «fora» (pesada pela gaussiana da direcção) para a célula ser BORDA no [`borda_suave`]: a
/// recta pela célula dá ~0,45, o canto interior ~0,25, o furo de uma célula ~0,03 (`oraculo/README.md`
/// §BUG-bordas-W3b).
pub(crate) const ORLA_FORA_MIN: f64 = 0.20;

#[cfg(test)]
thread_local! {
    /// (Testes) o [`ORLA_FORA_MIN`] de uma variante medida no MESMO processo.
    pub(crate) static ORLA_FORA_DA_MEDIDA: core::cell::Cell<Option<f64>> = const { core::cell::Cell::new(None) };
}

#[inline]
pub(super) fn orla_fora_min() -> f64 {
    #[cfg(test)]
    if let Some(v) = ORLA_FORA_DA_MEDIDA.with(core::cell::Cell::get) {
        return v;
    }
    ORLA_FORA_MIN
}

#[inline]
pub(super) fn entalhe_min() -> f64 {
    #[cfg(test)]
    if let Some(v) = ENTALHE_DA_MEDIDA.with(core::cell::Cell::get) {
        return v;
    }
    ENTALHE_MIN
}

/// ⭐ §BUG-bordas-W3b (smoke do dono 27/10, foto `smoke_27_10/7.png`): **a cobertura parcial é ÁREA, não espessura
/// — a tinta não tem entalhes de uma célula.** As alturas (desfocadas) vão × a cobertura CRUA da célula; uma célula
/// pintada mais rala que as vizinhas (o ruído do pouso na orla rasgada) virava um poço de uma célula com paredes de
/// toda a espessura: a tinta dela sem luz nem céu (quase preta) sobre o papel na sombra cheia — os pontinhos escuros
/// soltos; e o contorno liso da [`borda_suave`] cortava fundo a célula SALIENTE de uma orla rasgada, com as vizinhas
/// dos lados dentro da tinta — o mesmo entalhe. A célula COM pigmento cuja cobertura (depois do contorno) fica
/// `≥ ENTALHE_MIN` abaixo da MEDIANA das 3×3 cobre (altura e alfa) até essa mediana: a mediana enche o entalhe e
/// deixa a borda (recta, canto, graduada: a mediana de um campo monótono é ele próprio) onde está. O papel sem
/// pigmento continua papel. Devolve o factor da célula sobre a cobertura crua (o `borda` onde nada se enche).
pub(super) fn sem_entalhes(
    rep: &impl Reparte,
    cob: &[f64],
    borda: Vec<f32>,
    w: usize,
    h: usize,
) -> Vec<f32> {
    let min = entalhe_min();
    // A cobertura depois do contorno sub-célula: é ela que tem (ou não) entalhes.
    let depois = |k: usize| cob[k] * borda.get(k).map_or(1.0, |&b| f64::from(b));
    let mut f = vec![1f32; w * h];
    por_linha(rep, w, &mut f, &|y, l| {
        for (x, q) in l.iter_mut().enumerate() {
            let k = x + y * w;
            let c = depois(k);
            if cob[k] <= 0.0 || c >= 1.0 - min {
                continue;
            }
            let mut v = [0f64; 9];
            for (n, e) in v.iter_mut().enumerate() {
                let (xx, yy) = (
                    (x + n % 3).saturating_sub(1).min(w - 1),
                    (y + n / 3).saturating_sub(1).min(h - 1),
                );
                *e = depois(xx + yy * w);
            }
            v.sort_unstable_by(f64::total_cmp);
            let mediana = v[4];
            if mediana - c >= min {
                *q = (mediana / cob[k]) as f32;
            }
        }
    });
    if !f.iter().any(|&q| q != 1.0) {
        return borda;
    }
    if borda.is_empty() {
        return f;
    }
    f.iter()
        .zip(&borda)
        .map(|(&a, &b)| if a != 1.0 { a } else { b })
        .collect()
}

/// Média móvel separável de raio `r` (caixa `2r+1`), presa às bordas; cada passagem por linhas de `rep`.
pub(super) fn desfoca(rep: &impl Reparte, v: &[f64], w: usize, h: usize, r: usize) -> Vec<f64> {
    let n = (2 * r + 1) as f64;
    let mut tmp = vec![0f64; w * h];
    por_linha(rep, w, &mut tmp, &|y, linha| {
        for (x, q) in linha.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in 0..=2 * r {
                s += v[(x + k).saturating_sub(r).min(w - 1) + y * w];
            }
            *q = s / n;
        }
    });
    let mut out = vec![0f64; w * h];
    por_linha(rep, w, &mut out, &|y, linha| {
        for (x, q) in linha.iter_mut().enumerate() {
            let mut s = 0.0;
            for k in 0..=2 * r {
                s += tmp[x + (y + k).saturating_sub(r).min(h - 1) * w];
            }
            *q = s / n;
        }
    });
    out
}

/// Média móvel separável de meia largura FRACIONÁRIA `hh` (as pontas `⌊hh⌋ + 1` pesam `hh − ⌊hh⌋`, a `caixa` do
/// Smooth Edges do composite), presa às bordas.
pub(super) fn desfoca_frac(rep: &impl Reparte, v: &[f64], w: usize, h: usize, hh: f64) -> Vec<f64> {
    let k = exato::floor(hh) as isize;
    let f = hh - k as f64;
    let n = 2.0 * k as f64 + 1.0 + 2.0 * f;
    let caixa = |ler: &dyn Fn(isize) -> f64| {
        let mut s = 0.0;
        for d in -k..=k {
            s += ler(d);
        }
        if f > 0.0 {
            s += f * (ler(-k - 1) + ler(k + 1));
        }
        s / n
    };
    let (wi, hi) = (w as isize - 1, h as isize - 1);
    let mut tmp = vec![0f64; w * h];
    por_linha(rep, w, &mut tmp, &|y, linha| {
        for (x, q) in linha.iter_mut().enumerate() {
            *q = caixa(&|d| v[(x as isize + d).clamp(0, wi) as usize + y * w]);
        }
    });
    let mut out = vec![0f64; w * h];
    por_linha(rep, w, &mut out, &|y, linha| {
        for (x, q) in linha.iter_mut().enumerate() {
            *q = caixa(&|d| tmp[x + (y as isize + d).clamp(0, hi) as usize * w]);
        }
    });
    out
}

/// A cobertura a partir da qual a célula é SILHUETA no Smooth Edges: a da mancha do composite (`SS0`).
pub(crate) const MANCHA_DA_SILHUETA: f64 = 0.12;
/// A folga da faixa da orla (a tenda de uns que não soma `1` ao bit).
pub(super) const FOLGA_DA_ORLA: f64 = 1e-6;

/// ⭐ **D1 W3b (smoke do dono 27/10: «Smooth Edges só atua na cor e não no relevo… deve ser para todo o papel»):
/// o Smooth Edges da TELA amacia a SILHUETA antes do campo** (`oraculo/README.md` §Smooth Edges na tela). A
/// cobertura depois do contorno sub-célula ([`borda_suave`], [`sem_entalhes`]) desfoca-se por uma tenda de raio `r`
/// células (duas caixas de `r/2`, a lei do composite); na FAIXA da orla — onde a mancha (`≥ MANCHA_DA_SILHUETA`)
/// desfocada pela mesma tenda não é nem `0` nem `1` — a célula COM pigmento cobre o desfocado. A cobertura e as três
/// alturas vão × este factor, por isso a parede, a normal, os brilhos e a sombra no papel seguem a mesma silhueta que
/// a cor. O miolo e o papel longe da orla ficam ao bit; o papel sem pigmento continua papel. `r = 0` = `borda`.
pub(super) fn amacia_a_silhueta(
    rep: &impl Reparte,
    cob: &[f64],
    borda: Vec<f32>,
    (w, h): (usize, usize),
    r: f64,
) -> Vec<f32> {
    if r <= 0.0 {
        return borda;
    }
    let depois: Vec<f64> = cob
        .iter()
        .enumerate()
        .map(|(k, &c)| c * borda.get(k).map_or(1.0, |&b| f64::from(b)))
        .collect();
    let mancha: Vec<f64> = depois
        .iter()
        .map(|&c| if c >= MANCHA_DA_SILHUETA { 1.0 } else { 0.0 })
        .collect();
    let hh = 0.5 * r;
    let tenda = |v: &[f64]| desfoca_frac(rep, &desfoca_frac(rep, v, w, h, hh), w, h, hh);
    let (bm, bc) = (tenda(&mancha), tenda(&depois));
    let mut f = if borda.is_empty() {
        vec![1f32; w * h]
    } else {
        borda
    };
    for k in 0..w * h {
        if cob[k] > 0.0 && bm[k] > FOLGA_DA_ORLA && bm[k] < 1.0 - FOLGA_DA_ORLA {
            f[k] = (bc[k].min(1.0) / cob[k]) as f32;
        }
    }
    if f.iter().all(|&q| q == 1.0) {
        Vec::new()
    } else {
        f
    }
}
