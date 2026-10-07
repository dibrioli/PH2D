//! **O que o gesto CERCA** (decisão do dono, 2026-10-06: *«então vamos corrigir»* — os buracos de um
//! rabisco que volta para trás, nos quatro meios). A regra não-zero de [`super::solid::fill_coverage`]
//! anula dois contornos de sentidos opostos: onde o rabisco volta para trás fica um buraco CERCADO pela
//! tinta (medido: `3 022`–`4 139` píxeis num rabisco, `diag_o_solid_com_papel`). A mancha é «a região
//! cercada pelo gesto», então o que ela cerca é dela.
//!
//! ⚠️ **Laço a laço.** O buraco do rabisco nasce DENTRO de um laço que se cruza; o de uma forma
//! `Remove` nasce ENTRE laços (um contorno de sentido oposto, `solid_shapes`) e é de propósito
//! (`a_remove_shape_punches_a_hole_in_the_solid`). Então cada laço classifica só o que ELE cerca, e
//! esse vazio enche. (No produto o rabisco nunca se mistura com formas — `solid_fill_loops` usa umas
//! OU o caminho —, e entre cópias de simetria espelhadas somar o winding das outras faria a cópia
//! furar o vazio cheio da vizinha: a mesma anulação, de volta.)
//!
//! «Cercado» é uma propriedade da caixa INTEIRA do laço, nunca de uma janela — por isso a
//! classificação corre sobre a caixa toda (a mesma em qualquer janela que a aguada recomponha) e
//! guarda-se enquanto os laços não mudam. Na caixa, com um píxel de folga (a borda é de FORA):
//!
//! - **barreira** é o píxel cujo centro tem winding ≠ 0 ou por onde passa um segmento do laço — um
//!   cruzamento do rabisco é barreira, então um buraco que toca o lado de fora só num PONTO continua
//!   cercado;
//! - **de fora** é o vazio ligado à borda por vazios (4-vizinhança);
//! - o resto do vazio está **cercado**: ele e a sua orla (a barreira encostada a ele e, pelo lado, a
//!   nenhum de fora) enchem. Tudo o mais é a cobertura não-zero, ao byte — um gesto sem buracos não muda.

use std::sync::Arc;

const LIVRE: u8 = 0;
const BARREIRA: u8 = 1;
const FORA: u8 = 2;

/// Uma caixa em píxeis inteiros da tela.
#[derive(Clone, Copy)]
struct Caixa {
    x0: i64,
    y0: i64,
    w: usize,
    h: usize,
}

impl Caixa {
    /// A caixa dos pontos, com um píxel de folga em volta.
    fn de<'a>(pontos: impl Iterator<Item = &'a [f32; 2]>) -> Option<Self> {
        let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
        for p in pontos {
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        if !(lo[0] <= hi[0] && lo[0].is_finite() && hi[0].is_finite() && hi[1].is_finite()) {
            return None;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (x0, y0) = (lo[0].floor() as i64 - 1, lo[1].floor() as i64 - 1);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Some(Self {
            x0,
            y0,
            w: (hi[0].ceil() as i64 + 2 - x0) as usize,
            h: (hi[1].ceil() as i64 + 2 - y0) as usize,
        })
    }

    /// O índice do píxel `(x, y)` da tela, se ele cai na caixa.
    fn indice(&self, x: i64, y: i64) -> Option<usize> {
        let (i, j) = (x - self.x0, y - self.y0);
        #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
        (i >= 0 && j >= 0 && i < self.w as i64 && j < self.h as i64)
            .then(|| j as usize * self.w + i as usize)
    }
}

/// O que UM laço cerca, só à volta dos buracos: o efeito em cada píxel da caixa deles (`0` nada,
/// [`BURACO`], [`ORLA`]).
struct Furo {
    caixa: Caixa,
    efeito: Vec<u8>,
}

const BURACO: u8 = 1;
const ORLA: u8 = 2;

/// O que os laços cercam: o de cada laço que cerca alguma coisa.
pub(crate) struct Cercado {
    furos: Vec<Furo>,
}

impl Cercado {
    /// A classificação dos `loops`, ou `None` se nenhum cerca vazio nenhum.
    pub(crate) fn de(loops: &[Vec<[f32; 2]>]) -> Option<Arc<Self>> {
        type Memo = Option<(Vec<Vec<[f32; 2]>>, Option<Arc<Cercado>>)>;
        thread_local! {
            static ULTIMO: std::cell::RefCell<Memo> = const { std::cell::RefCell::new(None) };
        }
        ULTIMO.with(|u| {
            if let Some((l, c)) = u.borrow().as_ref()
                && l.as_slice() == loops
            {
                return c.clone();
            }
            let c = Self::classifica(loops).map(Arc::new);
            *u.borrow_mut() = Some((loops.to_vec(), c.clone()));
            c
        })
    }

    fn classifica(loops: &[Vec<[f32; 2]>]) -> Option<Self> {
        let validos: Vec<&[[f32; 2]]> = loops
            .iter()
            .filter(|l| l.len() >= 3)
            .map(Vec::as_slice)
            .collect();
        let furos: Vec<Furo> = validos.iter().filter_map(|l| furo(l)).collect();
        (!furos.is_empty()).then_some(Self { furos })
    }

    /// Enche, na janela `out` (`w × h`, origem `origin` em píxeis inteiros da tela), o vazio cercado e
    /// a sua orla.
    pub(crate) fn enche(&self, out: &mut [u8], w: usize, h: usize, origin: [f32; 2]) {
        #[allow(clippy::cast_possible_truncation)]
        let (ox, oy) = (origin[0].round() as i64, origin[1].round() as i64);
        for f in &self.furos {
            #[allow(clippy::cast_possible_wrap)]
            let (gx0, gx1) = (
                ox.max(f.caixa.x0),
                (ox + w as i64).min(f.caixa.x0 + f.caixa.w as i64),
            );
            #[allow(clippy::cast_possible_wrap)]
            let (gy0, gy1) = (
                oy.max(f.caixa.y0),
                (oy + h as i64).min(f.caixa.y0 + f.caixa.h as i64),
            );
            for gy in gy0..gy1 {
                for gx in gx0..gx1 {
                    let k = f.caixa.indice(gx, gy).expect("na interseção");
                    if f.efeito[k] != 0 {
                        #[allow(clippy::cast_sign_loss)]
                        let o = ((gy - oy) as usize) * w + (gx - ox) as usize;
                        out[o] = 255;
                    }
                }
            }
        }
    }
}

/// O que o laço `l` cerca, ou `None`. A classificação corre sobre INTERVALOS de linha (as barreiras
/// de uma linha são poucos intervalos): só à volta dos buracos é que se desce ao píxel.
fn furo(l: &[[f32; 2]]) -> Option<Furo> {
    let caixa = Caixa::de(l.iter())?;
    let (w, h) = (caixa.w, caixa.h);
    // 1) A barreira de cada linha: o winding ≠ 0 nos centros e os píxeis por onde passa o caminho.
    let mut barreira = intervalos_de_winding(l, caixa);
    #[allow(clippy::cast_precision_loss)]
    let (ox, oy) = (caixa.x0 as f32, caixa.y0 as f32);
    for i in 0..l.len() {
        let (a, b) = (l[i], l[(i + 1) % l.len()]);
        percorre([a[0] - ox, a[1] - oy], [b[0] - ox, b[1] - oy], w, h, |k| {
            #[allow(clippy::cast_possible_wrap)]
            let x = (k % w) as i64;
            barreira[k / w].push((x, x));
        });
    }
    // 2) O vazio de cada linha é o complemento; de fora é o vazio ligado à borda (4-vizinhança).
    #[allow(clippy::cast_possible_wrap)]
    let ultimo = w as i64 - 1;
    let livres: Vec<Vec<(i64, i64)>> = barreira
        .iter_mut()
        .map(|b| {
            b.sort_unstable();
            let mut v = Vec::new();
            let mut x = 0i64;
            for &(de, ate) in b.iter() {
                if de > x {
                    v.push((x, de - 1));
                }
                x = x.max(ate + 1);
            }
            if x <= ultimo {
                v.push((x, ultimo));
            }
            v
        })
        .collect();
    let mut fora: Vec<Vec<bool>> = livres.iter().map(|r| vec![false; r.len()]).collect();
    let mut pilha: Vec<(usize, usize)> = Vec::new();
    // A folga de 1 px põe a 1.ª e a última coluna livres em TODA linha, ligadas na vertical à
    // primeira e à última linha: semeá-las basta (semear os lados é equivalente — mutação provada).
    for y in [0, h - 1] {
        for (k, f) in fora[y].iter_mut().enumerate() {
            if !*f {
                *f = true;
                pilha.push((y, k));
            }
        }
    }
    while let Some((y, k)) = pilha.pop() {
        let (de, ate) = livres[y][k];
        for ny in [y.wrapping_sub(1), y + 1] {
            if ny >= h {
                continue;
            }
            let r = &livres[ny];
            let mut j = r.partition_point(|&(_, b)| b < de);
            while j < r.len() && r[j].0 <= ate {
                if !fora[ny][j] {
                    fora[ny][j] = true;
                    pilha.push((ny, j));
                }
                j += 1;
            }
        }
    }
    // 3) Sobrou vazio? A caixa dos buracos, com dois píxeis em volta (a orla e os vizinhos dela).
    let (mut x0, mut y0, mut x1, mut y1) = (i64::MAX, usize::MAX, i64::MIN, 0usize);
    for (y, r) in livres.iter().enumerate() {
        for (k, &(de, ate)) in r.iter().enumerate() {
            if !fora[y][k] {
                x0 = x0.min(de);
                x1 = x1.max(ate);
                y0 = y0.min(y);
                y1 = y1.max(y);
            }
        }
    }
    if x0 > x1 {
        return None;
    }
    let (gx0, gx1) = ((x0 - 2).max(0), (x1 + 2).min(ultimo));
    #[allow(clippy::cast_possible_wrap)]
    let (gy0, gy1) = (y0.saturating_sub(2), (y1 + 2).min(h - 1));
    #[allow(clippy::cast_sign_loss)]
    let gw = (gx1 - gx0 + 1) as usize;
    let gh = gy1 - gy0 + 1;
    let mut estado = vec![BARREIRA; gw * gh];
    for y in gy0..=gy1 {
        for (k, &(de, ate)) in livres[y].iter().enumerate() {
            let (a, b) = (de.max(gx0), ate.min(gx1));
            if a <= b {
                #[allow(clippy::cast_sign_loss)]
                estado[(y - gy0) * gw + (a - gx0) as usize..=(y - gy0) * gw + (b - gx0) as usize]
                    .fill(if fora[y][k] { FORA } else { LIVRE });
            }
        }
    }
    // 4) O efeito: o buraco, e a orla (barreira encostada a ele e a nenhum de fora).
    #[allow(clippy::cast_possible_wrap)]
    let em = |x: i64, y: i64| -> u8 {
        if x < 0 || y < 0 || x >= gw as i64 || y >= gh as i64 {
            return FORA;
        }
        #[allow(clippy::cast_sign_loss)]
        {
            estado[y as usize * gw + x as usize]
        }
    };
    let mut efeito = vec![0u8; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            #[allow(clippy::cast_possible_wrap)]
            let (xi, yi) = (x as i64, y as i64);
            efeito[y * gw + x] = match estado[y * gw + x] {
                LIVRE => BURACO,
                BARREIRA => {
                    let viz =
                        || (-1i64..=1).flat_map(move |dy| (-1i64..=1).map(move |dx| (dx, dy)));
                    let encosta = viz().any(|(dx, dy)| em(xi + dx, yi + dy) == LIVRE);
                    // Encostar ao lado de fora PELO LADO é o contorno do gesto (a cobertura dele
                    // fica); só em diagonal é o píxel do cruzamento por onde o buraco tocava o lado
                    // de fora — e enchê-lo é o que o fecha.
                    let lado = [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)];
                    if encosta && !lado.iter().any(|&(dx, dy)| em(xi + dx, yi + dy) == FORA) {
                        ORLA
                    } else {
                        0
                    }
                }
                _ => 0,
            };
        }
    }
    #[allow(clippy::cast_possible_wrap)]
    Some(Furo {
        caixa: Caixa {
            x0: caixa.x0 + gx0,
            y0: caixa.y0 + gy0 as i64,
            w: gw,
            h: gh,
        },
        efeito,
    })
}

/// Os intervalos (inclusivos, em `x` da caixa) de cada linha onde o winding do laço no centro do
/// píxel é ≠ 0.
fn intervalos_de_winding(l: &[[f32; 2]], caixa: Caixa) -> Vec<Vec<(i64, i64)>> {
    cruzamentos(&[l], caixa)
        .into_iter()
        .map(|linha| {
            let mut v = Vec::new();
            let mut soma = 0;
            for par in linha.windows(2) {
                soma += par[0].1;
                if soma != 0 {
                    let (de, ate) = intervalo_entre(par[0].0, par[1].0, caixa.w);
                    if de <= ate {
                        v.push((de, ate));
                    }
                }
            }
            v
        })
        .collect()
}

/// Os píxeis com o centro em `(xa, xb]` (`x + 0,5 > xa` e `≤ xb`), presos à caixa.
fn intervalo_entre(xa: f32, xb: f32, w: usize) -> (i64, i64) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    (
        ((xa - 0.5).floor() as i64 + 1).max(0),
        ((xb - 0.5).floor() as i64).min(w as i64 - 1),
    )
}

/// Os cruzamentos de cada linha da `caixa` (em `y + 0,5`) com as arestas dos `laços`, ordenados por
/// `x`, com o sentido da aresta.
fn cruzamentos(lacos: &[&[[f32; 2]]], caixa: Caixa) -> Vec<Vec<(f32, i32)>> {
    let h = caixa.h;
    #[allow(clippy::cast_precision_loss)]
    let (ox, oy) = (caixa.x0 as f32, caixa.y0 as f32);
    let mut cruza: Vec<Vec<(f32, i32)>> = vec![Vec::new(); h];
    for l in lacos {
        for i in 0..l.len() {
            let (a, b) = (l[i], l[(i + 1) % l.len()]);
            let (a, b) = ([a[0] - ox, a[1] - oy], [b[0] - ox, b[1] - oy]);
            if a[1] == b[1] {
                continue;
            }
            let (dir, p, q) = if a[1] < b[1] { (1, a, b) } else { (-1, b, a) };
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (y_de, y_ate) = (
                (p[1] - 0.5).ceil().max(0.0) as usize,
                ((q[1] - 0.5).ceil().max(0.0) as usize).min(h),
            );
            for (y, linha) in cruza.iter_mut().enumerate().take(y_ate).skip(y_de) {
                #[allow(clippy::cast_precision_loss)]
                let yc = y as f32 + 0.5;
                linha.push((p[0] + (q[0] - p[0]) * (yc - p[1]) / (q[1] - p[1]), dir));
            }
        }
    }
    for linha in &mut cruza {
        linha.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
    cruza
}

/// Os píxeis (índices `y·w + x` na caixa) por onde passa o segmento `a → b` — o percurso de
/// Amanatides–Woo, as duas pontas incluídas.
fn percorre(a: [f32; 2], b: [f32; 2], w: usize, h: usize, mut marca: impl FnMut(usize)) {
    #[allow(clippy::cast_possible_truncation)]
    let (mut x, mut y) = (a[0].floor() as i64, a[1].floor() as i64);
    #[allow(clippy::cast_possible_truncation)]
    let (xf, yf) = (b[0].floor() as i64, b[1].floor() as i64);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let (sx, sy) = (
        if dx >= 0.0 { 1 } else { -1 },
        if dy >= 0.0 { 1 } else { -1 },
    );
    #[allow(clippy::cast_precision_loss)]
    let proxima = |p: f32, c: i64, s: i64| {
        if s > 0 {
            (c + 1) as f32 - p
        } else {
            p - c as f32
        }
    };
    let passo = |d: f32| {
        if d == 0.0 {
            f32::INFINITY
        } else {
            1.0 / d.abs()
        }
    };
    let (ddx, ddy) = (passo(dx), passo(dy));
    let (mut tx, mut ty) = (
        if dx == 0.0 {
            f32::INFINITY
        } else {
            proxima(a[0], x, sx) * ddx
        },
        if dy == 0.0 {
            f32::INFINITY
        } else {
            proxima(a[1], y, sy) * ddy
        },
    );
    let mut passos = (xf - x).abs() + (yf - y).abs() + 1;
    loop {
        #[allow(clippy::cast_possible_wrap)]
        if x >= 0 && y >= 0 && x < w as i64 && y < h as i64 {
            #[allow(clippy::cast_sign_loss)]
            marca(y as usize * w + x as usize);
        }
        passos -= 1;
        if passos <= 0 {
            break;
        }
        if tx < ty {
            x += sx;
            tx += ddx;
        } else {
            y += sy;
            ty += ddy;
        }
    }
}

#[cfg(test)]
#[path = "solid_cercado_tests.rs"]
mod tests;
