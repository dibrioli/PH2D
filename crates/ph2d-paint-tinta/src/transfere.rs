//! Eq. (4): pigmento CONSERVADO entre pincel, filme, aberto e fechado — sobre somas (F4_2 §2, E1).

use crate::bandas::Bandas;
use crate::lei::{Optica, camada, cobertura, optica};
use crate::soma::Soma;

#[inline]
pub(crate) fn move_um(de: &mut f32, para: &mut f32, f: f32) -> f32 {
    let (antes_de, antes_para) = (*de, *para);
    *de = antes_de - antes_de * f;
    let movido = antes_de - *de;
    *para = antes_para + movido;
    movido
}

/// Move a fracção `f ∈ [0,1]` da fatia `de` para `para` (a MESMA fracção de `v`, de `Σv·Lab` e de
/// `Σv·t`, `Σv·m`: a cor e o meio de `de` não mudam); devolve o que moveu. O que chega é `antes − depois` lido de `de`
/// (não `x·f`), para que o arredondamento não crie nem apague pigmento além de 1 ulp.
pub fn transfere(de: &mut Soma, para: &mut Soma, f: f32) -> Soma {
    let f = f.clamp(0.0, 1.0);
    let (antes_de, antes_para) = (de.v, para.v);
    let v = move_um(&mut de.v, &mut para.v, f);
    let lab = [0, 1, 2].map(|i| move_um(&mut de.lab[i], &mut para.lab[i], f));
    let t = move_um(&mut de.t, &mut para.t, f);
    let m = move_um(&mut de.m, &mut para.m, f);
    debug_assert!(de.v >= 0.0 && para.v >= 0.0, "volume negativo");
    debug_assert!(
        ((de.v + para.v) - (antes_de + antes_para)).abs()
            <= 4.0 * f32::EPSILON * (antes_de + antes_para),
        "transferência criou/perdeu pigmento"
    );
    Soma { v, lab, t, m }
}

/// Pick-up (F2 eq. 4): o pincel recolhe `k_u · o · A_c` do pigmento ABERTO da tela.
pub fn pick_up(
    tela: &mut Soma,
    pincel: &mut Soma,
    k_u: f32,
    abertura: f32,
    area_contacto: f32,
) -> Soma {
    transfere(tela, pincel, k_u * abertura * area_contacto)
}

/// A óptica de uma célula: filme e aberto entram JUNTOS (assentar não muda o que se vê).
#[must_use]
pub fn optica_celula(filme: &Soma, aberto: &Soma, m: f32, molhado: bool) -> Optica<3> {
    optica(&filme.junta(*aberto), m, molhado)
}

/// Fecho: o aberto cura (SECO, meio `m`) e passa a ser fundo, `R_g ← R(óptica do aberto, R_g)`, aberto
/// `← 0`. O que se via antes do fecho (aberto seco sobre `R_g`, filme vazio) é bit a bit o novo `R_g`.
pub fn fecha(rg: &mut Bandas<3>, aberto: &mut Soma, m: f32) {
    let o = optica_celula(&Soma::default(), aberto, m, false);
    *rg = camada(&o, rg);
    *aberto = Soma::default();
}

/// Ecrã da célula para o compositor RGBA (F4_2 §3.2, «exacto sobre branco»): `R_w = camada(o, R_g)` e a
/// cobertura média `C` da camada — ver [`decompoe`]. Devolve `(cor sRGB [0,1], α)`.
#[must_use]
pub fn ecra_sobre_branco(o: &Optica<3>, rg: &Bandas<3>) -> (Bandas<3>, f32) {
    let c = cobertura(o);
    decompoe(&camada(o, rg), ((c.0[0] + c.0[1]) + c.0[2]) / 3.0)
}

/// A reflectância `R_w` (linear) sobre branco e a cobertura média `C` → `(cor, α)` para um compositor que faz o
/// `over` nos valores sRGB CODIFICADOS (a tela do PH2D): `e = sRGB(R_w)`, `α = max(max_b(1 − e), C)`,
/// `cor = 1 − (1 − e)/α`. Sobre branco `cor·α + (1 − α) = e` (até ao arredondamento) e `cor ∈ [0,1]` porque
/// `α ≥ 1 − e`. Em luz linear a mesma conta dava, a `α ≈ 0,98`, um vermelho de `40` → `~5` (E9 §7).
#[must_use]
pub fn decompoe(rw: &Bandas<3>, media_c: f32) -> (Bandas<3>, f32) {
    // `R = 1` é o papel nu: `1` exacto (o `1,055 − 0,055` do sRGB em `f32` não o devolve).
    let e = rw.0.map(|r| {
        if r >= 1.0 {
            1.0
        } else {
            crate::oklab::linear_para_srgb(r)
        }
    });
    let falta = e.map(|r| 1.0 - r);
    let alfa = falta[0]
        .max(falta[1])
        .max(falta[2])
        .max(media_c)
        .clamp(0.0, 1.0);
    if alfa <= 0.0 {
        return (Bandas(e), 0.0);
    }
    (
        Bandas(falta.map(|f| (1.0 - f / alfa).clamp(0.0, 1.0))),
        alfa,
    )
}
