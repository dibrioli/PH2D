//! ⭐ (W14) **A grelha FINA da vizinhança** — os `n` mais perto por ANÉIS, sem varrer a multidão
//! inteira (plano 30 §22.3). A lista sai a MESMA da varrida das 3 × 3 células do alcance sem perda
//! (gate `os_vizinhos_por_aneis_sao_os_da_varrida`): os candidatos, o alcance de cada par e a ordem
//! total são os dela; só se pára de procurar quando nenhum por visitar pode entrar nem empatar.

use crate::v2::V2;

/// Quantas células finas cabem no alcance sem perda (o lado da célula grossa).
const FINAS_POR_ALCANCE: f64 = 8.0;

/// As células em listas contíguas sobre a caixa da fotografia (cada lista pela ordem dos índices).
pub(crate) struct GrelhaFina {
    lo: V2,
    lado: f64,
    nx: i64,
    ny: i64,
    inicio: Vec<u32>,
    quem: Vec<u32>,
}

impl GrelhaFina {
    /// `None` se não há uma grelha a fazer (ninguém, um alcance nulo ou uma posição não finita).
    pub(crate) fn new(pos: &[V2], alcance: f64) -> Option<Self> {
        if pos.is_empty() || !(alcance.is_finite() && alcance > 0.0) {
            return None;
        }
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in pos {
            if !(p[0].is_finite() && p[1].is_finite()) {
                return None;
            }
            for k in 0..2 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
        }
        // Uma multidão espalhada por um mundo grande não enche a memória de células vazias.
        let tecto = 4 * pos.len() as i64 + 64;
        let mut lado = alcance / FINAS_POR_ALCANCE;
        let dims = |lado: f64| {
            (
                ((hi[0] - lo[0]) / lado).floor() as i64 + 1,
                ((hi[1] - lo[1]) / lado).floor() as i64 + 1,
            )
        };
        let (mut nx, mut ny) = dims(lado);
        while nx.saturating_mul(ny) > tecto {
            lado *= 2.0;
            (nx, ny) = dims(lado);
        }
        let mut g = Self {
            lo,
            lado,
            nx,
            ny,
            inicio: vec![0; (nx * ny + 1) as usize],
            quem: vec![0; pos.len()],
        };
        let cel: Vec<usize> = pos.iter().map(|&p| g.indice(g.celula(p))).collect();
        for &c in &cel {
            g.inicio[c + 1] += 1;
        }
        for c in 0..(nx * ny) as usize {
            g.inicio[c + 1] += g.inicio[c];
        }
        let mut cursor = g.inicio.clone();
        for (i, &c) in cel.iter().enumerate() {
            g.quem[cursor[c] as usize] = i as u32;
            cursor[c] += 1;
        }
        Some(g)
    }

    fn celula(&self, p: V2) -> (i64, i64) {
        (
            (((p[0] - self.lo[0]) / self.lado).floor() as i64).clamp(0, self.nx - 1),
            (((p[1] - self.lo[1]) / self.lado).floor() as i64).clamp(0, self.ny - 1),
        )
    }

    fn indice(&self, (x, y): (i64, i64)) -> usize {
        (y * self.nx + x) as usize
    }

    fn lista(&self, c: (i64, i64)) -> &[u32] {
        let k = self.indice(c);
        &self.quem[self.inicio[k] as usize..self.inicio[k + 1] as usize]
    }

    /// ⭐ Visita `visita` em cada agente das células do anel `r` à volta de `p` (o anel `0` é a célula
    /// dele), e devolve `(a distância mínima de p a uma célula ainda por visitar, se o anel já cobriu a
    /// grelha inteira)`. ⚠️ A distância é um MINORANTE (menos `10⁻⁶` células): um ponto no bordo da
    /// célula pode cair na vizinha por arredondamento, e um corte por excesso perdia um vizinho.
    pub(crate) fn anel(&self, p: V2, r: i64, mut visita: impl FnMut(u32)) -> (f64, bool) {
        let (cx, cy) = self.celula(p);
        let (x0, x1, y0, y1) = (cx - r, cx + r, cy - r, cy + r);
        let dentro = |x: i64, y: i64| x >= 0 && x < self.nx && y >= 0 && y < self.ny;
        let mut celula = |x: i64, y: i64| {
            if dentro(x, y) {
                self.lista((x, y)).iter().for_each(|&j| visita(j));
            }
        };
        if r == 0 {
            celula(cx, cy);
        } else {
            for x in x0..=x1 {
                celula(x, y0);
                celula(x, y1);
            }
            for y in y0 + 1..y1 {
                celula(x0, y);
                celula(x1, y);
            }
        }
        let tudo = x0 <= 0 && y0 <= 0 && x1 >= self.nx - 1 && y1 >= self.ny - 1;
        // A folga de `p` até ao bordo da PRÓPRIA célula, mais `r` células inteiras.
        let (bx, by) = (
            self.lo[0] + cx as f64 * self.lado,
            self.lo[1] + cy as f64 * self.lado,
        );
        let folga = (p[0] - bx)
            .min(bx + self.lado - p[0])
            .min(p[1] - by)
            .min(by + self.lado - p[1])
            .max(0.0);
        ((r as f64 + folga / self.lado - 1e-6) * self.lado, tudo)
    }
}
