//! ⭐ **O FILME do traço** ([`crate::tuning::Knob::StrokeFilm`], J4 aguarela — `docs/Painter/Pinceis/exames/aguarela.md`
//! «Motor J4»): o pincel de lavagem deixa UMA camada de tinta por onde passa, como o Rebelle — cada célula fica com o
//! MÁXIMO que um dab do traço lhe trouxe, nunca a SOMA dos dabs que a cruzaram.
//!
//! Medido (sonda `familias::aguarela::sonda`): somar os dabs faz o miolo do traço — o recto de 200 px do Mop é a soma
//! de 44 discos a 8 células, e cada célula lê o tapete das cerdas no desvio ao centro do dab (a mesma linha do tapete
//! ao longo do traço inteiro): listras 0,065 e um padrão de período 8 a ±20 % logo no depósito; o perfil através é a
//! CORDA do disco (o domo: ombros 0,77 do centro); as voltas da espiral e o início do traço somam mais. O máximo é
//! plano por construção: a corda, a fase do tapete e a sobreposição deixam de contar.
//!
//! A espessura do filme é a do miolo da soma num traço LARGO (`1 ÷ CapsuleSpacing` dabs, 40), a TODO o tamanho: o traço
//! fino da soma pesava `2r ÷ 1 célula` × o peso do fino (`1 ÷ 0,035` ≈ 29 dabs) e o filme tirado dele cobria a 5 px 0,62
//! do de 40 (o Rebelle 0,98) — o peso do fino desfaz-se (`agua_do_dab`) e o passo deixa de contar.
//! `0` = a soma de sempre, ao bit (o campo nem existe).

/// O máximo por célula do traço vivo (em unidades de depósito × dabs), da grade inteira; preguiçoso.
#[derive(Clone, Default)]
pub(crate) struct Filme {
    campo: Vec<f32>,
    /// O passo de uma linha da grade do campo.
    s: usize,
    /// As células escritas neste traço (`x1 < x0` = nenhuma): o que [`Self::limpa`] zera.
    caixa: (i32, i32, i32, i32),
    /// O centro do dab anterior e a direcção do traço (a [`Forma`] ao longo).
    ultimo: Option<(f64, f64)>,
    tangente: Option<(f64, f64)>,
}

impl Filme {
    /// O campo do tamanho da grade (`cells`), a zeros se ainda não existia.
    pub(crate) fn garante(&mut self, cells: usize, s: usize) {
        if self.campo.len() != cells || self.s != s {
            self.campo = vec![0.0; cells];
            self.s = s;
            self.caixa = (0, 0, -1, -1);
        }
    }

    /// Um traço novo: zera o que o anterior escreveu.
    pub(crate) fn limpa(&mut self) {
        let (x0, y0, x1, y1) = self.caixa;
        if x1 >= x0 && !self.campo.is_empty() {
            for y in y0..=y1 {
                let o = y as usize * self.s;
                self.campo[o + x0 as usize..=o + x1 as usize].fill(0.0);
            }
        }
        self.caixa = (0, 0, -1, -1);
        self.ultimo = None;
        self.tangente = None;
    }

    /// Um dab em `(x, y)`: a direcção do traço até ele (a do anterior se ele não andou; `None` no 1.º).
    pub(crate) fn passo(&mut self, x: f64, y: f64) -> Option<(f64, f64)> {
        if let Some((ax, ay)) = self.ultimo {
            let d = (x - ax).hypot(y - ay);
            if d > 1e-6 {
                self.tangente = Some(((x - ax) / d, (y - ay) / d));
            }
        }
        self.ultimo = Some((x, y));
        self.tangente
    }

    /// As linhas `y0..=y1` do campo (stride `s`), uma fatia cada — o depósito por linhas parte-as como parte o `wet`.
    pub(crate) fn linhas(
        &mut self,
        (x0, y0, x1, y1): (i32, i32, i32, i32),
    ) -> std::slice::ChunksMut<'_, f32> {
        let s = self.s;
        self.caixa = if self.caixa.2 < self.caixa.0 {
            (x0, y0, x1, y1)
        } else {
            (
                self.caixa.0.min(x0),
                self.caixa.1.min(y0),
                self.caixa.2.max(x1),
                self.caixa.3.max(y1),
            )
        };
        self.campo[y0 as usize * s..(y1 as usize + 1) * s].chunks_mut(s)
    }
}

/// O que um dab ACRESCENTA a uma célula do filme: `alvo` acima do máximo guardado (e o máximo sobe), ou nada.
#[inline]
#[must_use]
pub(crate) fn acrescenta(guardado: &mut f32, alvo: f64) -> f64 {
    let m = f64::from(*guardado);
    if alvo <= m {
        return 0.0;
    }
    *guardado = alvo as f32;
    alvo - m
}

/// A espessura do filme em dabs: `1 ÷ capsula` (o miolo de um traço largo), vezes `agua_do_dab` (`1 ÷` o peso do traço
/// fino, que o depósito já traz na opacidade) — a mesma a todo o tamanho, e o passo dos dabs não conta.
#[inline]
#[must_use]
pub(crate) fn espessura(capsula: f64, agua_do_dab: f64) -> f64 {
    agua_do_dab / capsula.max(1e-3)
}

/// O raio (**px** de tela, [`crate::sim::Params::px_por_celula`]) até ao qual o filme é CHATO: o Rebelle `Liner`/`Wash Round` a 40 px pousa um perfil plano através
/// do traço (o miolo ±1 % da borda a 1 px dela), a 200 px um domo (`Liner` 0,89 a 5 % da largura, 0,94 a 20 %, 1 no meio).
pub(crate) const DOMO_R0: f64 = 20.0;

/// ⭐ **A forma do filme largo** (J6): o alvo de cada dab cai `1 − lado·u⊥²` através do traço
/// ([`crate::tuning::Knob::StrokeFilmDome`]; ⛔ medido: `u³` não muda a orla — 0,842 contra 0,831 de `u²` a 2,5 % da
/// largura, o Rebelle 0,909: a orla clara é a da água, não o expoente) e `1 − ponta·u∥²` ao longo dele
/// ([`crate::tuning::Knob::StrokeFilmEnds`]): pelo MÁXIMO do filme só o 1.º e o último dab mostram a queda ao longo — as
/// tampas esbatem, o corpo não (o dab seguinte cobre-o no seu centro). Os dois `× (1 − R0/r)⁺`: crescem com o raio.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Forma {
    lado: f64,
    ponta: f64,
    r: f64,
    /// A direcção do traço neste dab (`None` no 1.º: a queda ao longo vale em todas as direcções).
    tangente: Option<(f64, f64)>,
}

impl Forma {
    #[must_use]
    /// `r` em células, `px` os px de uma célula: o domo mede o raio em px ([`DOMO_R0`]).
    pub(crate) fn nova(lado: f64, ponta: f64, r: f64, px: f64, tangente: Option<(f64, f64)>) -> Self {
        let r_px = r * px;
        let g = if r_px <= DOMO_R0 { 0.0 } else { 1.0 - DOMO_R0 / r_px };
        Self {
            lado: lado * g,
            ponta: ponta * g,
            r,
            tangente,
        }
    }

    /// O filme chato de sempre (o alvo não se mexe: ao bit).
    #[must_use]
    pub(crate) fn chata(&self) -> bool {
        self.lado <= 0.0 && self.ponta <= 0.0
    }

    /// O factor do alvo na célula a `(dx, dy)` do centro do dab.
    #[must_use]
    pub(crate) fn fator(&self, dx: f64, dy: f64) -> f64 {
        let r2 = (self.r * self.r).max(1e-9);
        let d2 = dx * dx + dy * dy;
        let (ao_longo, atraves) = match self.tangente {
            Some((tx, ty)) => {
                let p = dx * tx + dy * ty;
                (p * p, (d2 - p * p).max(0.0))
            }
            None => (d2, d2),
        };
        let lado = 1.0 - self.lado * (atraves / r2).min(1.0);
        let ponta = 1.0 - self.ponta * (ao_longo / r2).min(1.0);
        lado.max(0.0) * ponta.max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{DOMO_R0, Filme, Forma, acrescenta, espessura};

    /// ⭐ A forma: chata sem os knobs e até ao raio chato; acima dele a borda (através) e a tampa (ao longo) caem, e
    /// crescem com o raio; ao longo a tampa cai mais que o lado.
    #[test]
    fn a_forma_so_cresce_acima_do_raio_chato() {
        assert!(Forma::nova(0.0, 0.0, 100.0, 1.0, None).chata());
        assert!(Forma::nova(0.5, 0.5, DOMO_R0, 1.0, None).chata());
        let borda = |r: f64| 1.0 - Forma::nova(0.3, 0.0, r, 1.0, Some((1.0, 0.0))).fator(0.0, r);
        assert!(borda(40.0) > 0.0 && borda(40.0) < borda(100.0) && borda(100.0) < 0.3);
        let f = Forma::nova(0.3, 0.6, 100.0, 1.0, Some((1.0, 0.0)));
        assert!(
            f.fator(90.0, 0.0) < f.fator(0.0, 90.0),
            "a tampa cai mais que o lado"
        );
    }

    /// ⭐ O filme guarda o MÁXIMO: um dab mais fraco por cima não acrescenta nada, um mais forte só a diferença.
    #[test]
    fn o_filme_acrescenta_so_o_que_passa_do_maximo() {
        let mut m = 0.0f32;
        assert_eq!(acrescenta(&mut m, 4.0), 4.0);
        assert_eq!(acrescenta(&mut m, 3.0), 0.0);
        assert_eq!(acrescenta(&mut m, 6.0), 2.0);
        assert_eq!(m, 6.0);
    }

    /// ⭐ Um traço novo começa sem filme: o anterior é zerado onde escreveu (e só lá).
    #[test]
    fn o_traco_novo_comeca_sem_filme() {
        let mut f = Filme::default();
        f.garante(10 * 8, 10);
        for l in f.linhas((2, 3, 5, 4)) {
            l[2..=5].fill(7.0);
        }
        f.limpa();
        assert!(f.campo.iter().all(|&v| v == 0.0));
        assert!(f.caixa.2 < f.caixa.0);
    }

    /// A espessura é a do miolo de um traço largo (40 dabs a 0,025) e desfaz o peso do fino.
    #[test]
    fn a_espessura_e_a_do_miolo_largo_a_todo_o_tamanho() {
        assert_eq!(espessura(0.025, 1.0), 40.0);
        assert_eq!(espessura(0.025, 0.25), 10.0);
    }
}

/// ⭐ O domo fala em PX ([`DOMO_R0`], calibrado no Rebelle): o mesmo pincel (raio 100 px) a razão 1 (100 células) e 2
/// (50 células) dá o MESMO perfil em px — através e ao longo, em 21 pontos do centro à borda; tolerância `1e-12`
/// (medido: `0`, a conta é a mesma em px). Com o domo em células a razão 2 dava `1 − 20/50` contra `1 − 20/100`.
#[cfg(test)]
#[test]
fn o_domo_do_filme_e_o_mesmo_em_px_a_toda_a_razao() {
    let (rpx, lado, ponta) = (100.0, 0.4, 0.6);
    let a = Forma::nova(lado, ponta, rpx, 1.0, Some((1.0, 0.0)));
    let b = Forma::nova(lado, ponta, rpx / 2.0, 2.0, Some((1.0, 0.0)));
    for k in 0..=20 {
        let s = rpx * f64::from(k) / 20.0;
        for (dx, dy) in [(0.0, s), (s, 0.0)] {
            let (fa, fb) = (a.fator(dx, dy), b.fator(dx / 2.0, dy / 2.0));
            assert!((fa - fb).abs() <= 1e-12, "a {s} px: razão 1 {fa} × razão 2 {fb}");
        }
    }
    assert!(a.fator(0.0, rpx) < 0.9, "controlo: a 100 px o filme é um domo");
}
