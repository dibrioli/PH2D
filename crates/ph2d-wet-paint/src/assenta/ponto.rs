//! O ecrã da [`Ecra::Cobre`] partido para o Newton da secagem (31/10, `docs/Painter/49_realshader/SECAGEM_diagnostico.md`):
//! o que os três canais partilham calcula-se uma vez ([`Prep`]), a cor PARADA do avaliador leva a sua metade da
//! diferença de pigmento feita ([`Parada`]), e a derivada lê UM canal — tudo ao bit (`assenta_bit_tests`).
use super::{Ecra, cor};
use crate::colorops::{
    Perfil, a_mesma_tinta_filtrada, diferenca_de_pigmento, entre_perfis, filtrada_entre_perfis,
    velatura_canal, velatura_k,
};
use crate::colorops::{ks_of_srgb255, reflectance_of_ks, srgb255_of_linear};

/// O `d` do tapar extra de [`Ecra::Cobre`]: a diferença de pigmento, `0` sob o Glaze se é a mesma tinta filtrada.
#[inline]
pub(super) fn d_de(vela: f64, sc: [f64; 3], uc: [f64; 3]) -> f64 {
    if vela > 0.0 && a_mesma_tinta_filtrada(sc, uc) {
        0.0
    } else {
        diferenca_de_pigmento(sc, uc)
    }
}

/// A cor que fica PARADA no avaliador do [`resolve`] (o filme ao assentar, o assentado ao re-molhar), com a sua
/// metade de [`d_de`] feita uma vez — ao bit (os produtos do ângulo comutam).
#[derive(Clone, Copy)]
pub(super) struct Parada {
    l: Perfil,
    lod: Option<Perfil>,
}

impl Parada {
    pub(super) fn de(vela: f64, c: [f64; 3]) -> Self {
        Parada {
            l: Perfil::de(c),
            lod: (vela > 0.0).then(|| Perfil::de_od(c)),
        }
    }

    #[inline]
    pub(super) fn d(&self, vela: f64, var: [f64; 3]) -> f64 {
        if vela > 0.0
            && self
                .lod
                .as_ref()
                .is_some_and(|lod| filtrada_entre_perfis(&Perfil::de_od(var), lod))
        {
            return 0.0;
        }
        entre_perfis(&Perfil::de(var), &self.l)
    }
}

/// [`cor_e_alfa`] partido: ou já feito (as leis sem o tapar, as células sem as duas camadas), ou o [`Ponto`] cujos
/// canais se avaliam um a um (o [`resolve`] lê UM canal na derivada).
pub(super) enum Prep {
    Feito(([f64; 3], f64)),
    Ponto(Ponto),
}

/// O que os três canais da [`Ecra::Cobre`] partilham.
pub(super) struct Ponto {
    a_t: f64,
    a_e: f64,
    la: f64,
    la_w: f64,
    t_s: [f64; 3],
    /// O assentado PURO (sem a diluição em branco de `t_s`): a [`mistura_opaca`] faz-se entre as tintas.
    sc: [f64; 3],
    c_eff: [f64; 3],
    uc: [f64; 3],
    vela: f64,
    k_vel: Option<f64>,
    /// A tinta OPACA ([`Ecra::com_opaca`]): `(c, opaca)` — a concentração da tinta de cima na mistura SUBTRACTIVA
    /// ([`mistura_opaca`]) e a fracção com que o ponto vai para ela (`opaca × d`).
    opaca: Option<(f64, f64)>,
}

/// MOLHADA: a COBERTURA do ponto inteiro (a tinta de cima e a seca debaixo da água) vale a densidade
/// `p = 1 + (molhada − 1)·a_f`, com a cor da camada fixa — o papel molhado escurece tudo por igual e só até à
/// cor pura (o Rebelle: o cruzamento (131,124,201) → (194,188,224), seno do ângulo dos perfis 0,05; e o
/// amarelo molhado acumulado é a própria tinta, (245,205,25)). ⛔ Escalar a densidade da COR passava da
/// cor pura: o amarelo molhado ficava mais escuro que a tinta (0,90 do azul seco nas camadas).
#[inline]
fn la_molhada(la: f64, p: f64) -> f64 {
    if p == 1.0 {
        la
    } else {
        1.0 - (1.0 - la).powf(p)
    }
}

/// Um canal do ponto sobre o branco `v` re-expresso como cor da camada de cobertura `la`.
#[inline]
fn molha_canal(v: f64, la: f64) -> f64 {
    ((v - 255.0 * (1.0 - la)) / la).clamp(0.0, 255.0)
}

impl Ponto {
    /// O canal `k` da cor da camada.
    #[inline]
    fn canal(&self, k: usize) -> f64 {
        let r = self.na_caixa(k);
        let r = match self.opaca {
            // A mistura é das TINTAS e cobre `la`: misturar o assentado já diluído em branco e re-expressá-lo pela
            // cobertura cortava o canal que o pigmento de cima absorve — a orla fina ferrugem (dono 31/10, pressão
            // baixa: cor da camada (181,48,0) no amarelo fino sobre o azul fino).
            Some((c, o)) => {
                let m = 255.0 - self.la * (255.0 - mistura_opaca(self.sc[k], self.uc[k], c));
                r + (m - r) * o
            }
            None => r,
        };
        molha_canal(r, self.la)
    }

    /// O canal `k` do ponto sobre o branco, na caixa das tintas.
    #[inline]
    fn na_caixa(&self, k: usize) -> f64 {
        let (t_s, c_eff) = (self.t_s[k], self.c_eff[k]);
        let mut r = t_s * (1.0 - self.a_e) + c_eff * self.a_e;
        if let Some(kv) = self.k_vel {
            r = velatura_canal(r, kv, t_s, c_eff);
        }
        // O GLAZE do pincel: o filme FILTRA o assentado — a fracção `m` do ponto é o produto das duas camadas sobre o
        // branco. Uma vez por camada (o filme e o assentado), nunca por carimbo: é o que não leva ao preto.
        if self.vela > 0.0 {
            let f = 255.0 - self.a_t * (255.0 - self.uc[k]);
            r += self.vela * (t_s * f / 255.0 - r);
        }
        // ⛔ A CAIXA das tintas (Bug #50): o tapar extra pinta-se com a tinta diluída em BRANCO — é o que faz o claro
        // cobrir o escuro como no Rebelle, mas onde o filme é mais ESCURO o branco clareava o ponto além das duas
        // tintas (a orla esbranquiçada). Cada canal pára na mais clara das duas: o Rebelle nunca passa dela (as
        // bordas do óleo sobre o azul seco, `fixtures/rebelle/{oil,express}.png`).
        // ⚠️ Isto prende o ponto SOBRE O BRANCO; a COR que se compõe prende-se em [`na_caixa_das_tintas`] (o render).
        r.min(t_s.max(self.uc[k]))
    }
}

/// ⭐ **A tinta OPACA mistura-se SUBTRACTIVAMENTE com a de baixo** (dono 31/10: «amarelo sobre azul seco fica com
/// bordas brancas»; o coordenador sobre o segmento RGB: «cinzento»). O tapar extra da aguarela pinta a tinta diluída em
/// BRANCO, e a recta entre complementares no RGB passa pelo cinzento; a tinta real mistura pigmento: o canal é a média
/// em K/S (Kubelka–Munk, a mesma porta da mistura do motor, [`crate::colorops::km_weighted_mean_color`]) do assentado
/// visto `t_s` e da tinta de cima `uc` pela concentração `c` — amarelo fino sobre azul → verde, espesso → amarelo,
/// nunca branco nem cinzento.
#[inline]
fn mistura_opaca(t_s: f64, uc: f64, c: f64) -> f64 {
    let ks = (1.0 - c) * ks_of_srgb255(t_s) + c * ks_of_srgb255(uc);
    srgb255_of_linear(reflectance_of_ks(ks))
}

/// A concentração da tinta de cima na [`mistura_opaca`] para a cobertura `a` do que ela tapa: a opaca vai DEPRESSA
/// para a cor dela (`1 − (1 − a)^k`) — em K/S o pigmento de baixo domina até perto de `1` (o azul a 3 % ainda
/// esverdeia o amarelo).
#[inline]
fn concentracao_opaca(a: f64) -> f64 {
    1.0 - (1.0 - a.clamp(0.0, 1.0)).powi(CONCENTRACAO_OPACA_K)
}

/// O expoente de [`concentracao_opaca`] — escolhido, NÃO varrido: com ele a cena do amarelo sobre o azul seco
/// (`borda_branca_tests::a_tinta_opaca_mistura_como_pigmento`) dá o miolo da Gouache Flat (222,204,42) e a orla verde.
const CONCENTRACAO_OPACA_K: i32 = 3;

impl Prep {
    #[inline]
    pub(super) fn cor_e_alfa(&self) -> ([f64; 3], f64) {
        match self {
            Prep::Feito(f) => *f,
            Prep::Ponto(p) => ([0, 1, 2].map(|k| p.canal(k)), p.la_w),
        }
    }

    /// [`sobre_branco`].
    #[inline]
    pub(super) fn sobre_branco(&self) -> [f64; 3] {
        let (c, la) = self.cor_e_alfa();
        c.map(|v| 255.0 * (1.0 - la) + v * la)
    }

    /// Só o canal `k` de [`Prep::sobre_branco`] — a mesma aritmética.
    #[inline]
    pub(super) fn sobre_branco_canal(&self, k: usize) -> f64 {
        let (c, la) = match self {
            Prep::Feito((c, la)) => (c[k], *la),
            Prep::Ponto(p) => (p.canal(k), p.la_w),
        };
        255.0 * (1.0 - la) + c * la
    }
}

/// [`cor_e_alfa`] com o [`tapa`] de `a_f` e o [`d_de`] à mão (pedidos só onde a conta os usa).
#[inline]
pub(super) fn prepara(
    e: Ecra,
    sc: [f64; 3],
    uc: [f64; 3],
    a_s: f64,
    a_f: f64,
    tapa: impl FnOnce() -> f64,
    dif: impl FnOnce() -> f64,
) -> Prep {
    let Ecra::Cobre {
        filme,
        molhada,
        vela,
        oleo,
        opaca,
        ..
    } = e
    else {
        return Prep::Feito((cor(e, sc, uc, a_s, a_f), a_s + a_f * (1.0 - a_s)));
    };
    if a_f <= 0.0 {
        return Prep::Feito((sc, a_s));
    }
    let p = 1.0 + (molhada - 1.0) * a_f;
    let a_t = a_f;
    if a_s <= 0.0 {
        return Prep::Feito((
            uc.map(|v| molha_canal(255.0 - a_t * (255.0 - v), a_t)),
            la_molhada(a_t, p),
        ));
    }
    // O tapar EXTRA é da diferença de pigmento: a mesma tinta sobre si mesma só soma pigmento (sem ele,
    // re-molhar o azul para dentro de um filme azul clareava-o — medido no teste das transferências).
    // Sob o Glaze, a mesma tinta filtrada por si não é OUTRA a tapar ([`d_de`]): com o extra, cada filme fino que a
    // água traz clareava o assentado escuro e o assentar guardava-o — as linhas claras na orla (doc 42 §8).
    let h = tapa();
    let d = dif();
    // Sobre o ÓLEO não há tapar extra (dono 30/10: a aguada é a mesma tinta mais diluída e cobre pela quantidade de
    // pigmento; o Rebelle: sobre o óleo a aguada tem a cobertura que tem no papel, 0,35 × 0,38): o extra era pintado
    // com a tinta diluída em BRANCO — o creme sobre o óleo e no papel de cor.
    let a_e = a_t + (h - a_t).max(0.0) * d * (1.0 - oleo.max(0.0));
    let t_s = sc.map(|v| 255.0 - a_s * (255.0 - v));
    let c_eff = uc.map(|v| 255.0 - (a_t / a_e) * (255.0 - v));
    let la = a_s + a_e * (1.0 - a_s);
    let mut ponto = Ponto {
        sc,
        a_t,
        a_e,
        la,
        la_w: la_molhada(la, p),
        t_s,
        c_eff,
        uc,
        vela,
        k_vel: velatura_k(a_e, filme, 1.0, t_s, c_eff),
        opaca: None,
    };
    // Pela diferença de pigmento `d`, como o tapar extra: a mesma tinta sobre si só soma pigmento (a lei da aguarela).
    if opaca > 0.0 && d > 0.0 {
        // A parte da tinta de cima no PIGMENTO do ponto: sobre o assentado cheio (`a_s = 1`) é `a_e`; sobre o fino,
        // pouco azul debaixo de pouco amarelo é amarelo (a orla fina da pressão baixa).
        let parte = if a_s >= 1.0 {
            a_e
        } else {
            a_e / (a_e + a_s * (1.0 - a_e))
        };
        ponto.opaca = Some((concentracao_opaca(parte), opaca * d));
    }
    Prep::Ponto(ponto)
}
