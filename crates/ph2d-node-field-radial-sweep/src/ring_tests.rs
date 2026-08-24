//! Os gates do [`super::INNER_RADIUS`] — o anel (doc 89, folha 10).

use super::*;

/// Um sector cheio (360°), para isolar a lei RADIAL da angular.
fn full() -> Sector {
    sector(0.0, 360.0, 1.0)
}

/// A máscara radial a uma distância `r` do centro, num sector cheio.
fn at(r: f32, radius: f32, inner: f32, soft: f32) -> f32 {
    sweep_mask(r, 0.0, radius, inner, soft, 1.0, &full())
}

/// **O ANEL É VAZIO POR DENTRO, CHEIO NA BANDA, VAZIO POR FORA.**
#[test]
fn the_ring_is_empty_inside_full_in_the_band_and_empty_outside() {
    let (radius, inner) = (10.0, 4.0);
    assert_eq!(at(0.0, radius, inner, 0.0), 0.0, "o centro é o buraco");
    assert_eq!(at(3.9, radius, inner, 0.0), 0.0, "ainda no buraco");
    assert_eq!(at(4.1, radius, inner, 0.0), 1.0, "já na banda");
    assert_eq!(at(9.9, radius, inner, 0.0), 1.0, "ainda na banda");
    assert_eq!(at(10.1, radius, inner, 0.0), 0.0, "fora");
}

/// **SEM BURACO, O DISCO DE SEMPRE — AO BIT.**
///
/// ⚠️ Esta é a metade que sustenta *"o default é literal"*, e ela mede a FUNÇÃO e não a cena:
/// [`inner_rise`] devolve `1.0` **exacto** para todo `r` e todo `soft` quando o buraco é zero,
/// e `min(rad, 1.0)` é `rad` para qualquer `rad` que a rampa externa produza (a imagem dela é
/// `[0,1]`). Nenhum caminho novo é tomado no default.
#[test]
fn no_hole_means_the_ramp_is_exactly_one_everywhere() {
    for r in [0.0f32, 0.001, 1.0, 7.5, 1e6] {
        for soft in [0.0f32, 0.001, 0.5, 1.0] {
            let v = inner_rise(r, 0.0, soft);
            assert_eq!(v, 1.0, "inner_rise({r}, 0, {soft}) tem de ser 1,0 exacto");
        }
    }
    // E a consequência: com `inner = 0` a máscara é a rampa externa, número a número.
    for r in [0.0f32, 2.0, 5.0, 9.0, 9.9, 10.0, 12.0] {
        assert_eq!(
            at(r, 10.0, 0.0, 0.3),
            edge_ramp(r, 10.0, 0.3 * 10.0),
            "r = {r}"
        );
    }
}

/// **A BANDA MACIA DO BURACO COME PARA DENTRO DO ANEL** — não para fora.
///
/// ⚠️ O oráculo é escolhido para separar as duas leis: a cura ingénua
/// (`1 − edge_ramp(r, inner, soft)`) põe a rampa em `[inner − soft, inner]` e daria um valor
/// **positivo** em `r = inner − soft/2`, dentro do buraco. A nossa dá zero ali e meio em
/// `inner + soft/2`. As duas concordam nos extremos e desenham anéis diferentes.
#[test]
fn the_soft_edge_of_the_hole_eats_into_the_ring_not_out_of_it() {
    let (radius, inner, soft) = (20.0, 8.0, 0.25);
    let s = soft * inner; // 2,0 — a banda macia é [8, 10]
    assert_eq!(
        at(inner - s * 0.5, radius, inner, soft),
        0.0,
        "meio passo ANTES do raio interno ainda é buraco"
    );
    let mid = at(inner + s * 0.5, radius, inner, soft);
    assert!(
        (mid - 0.5).abs() < 1e-5,
        "meio passo DEPOIS tem de estar a meio: {mid}"
    );
    assert_eq!(
        at(inner + s + 0.1, radius, inner, soft),
        1.0,
        "cheio depois"
    );
}

/// **UM BURACO MAIOR QUE O RAIO EXTERNO ESVAZIA O CAMPO** — uma resposta, não um erro.
///
/// ⚠️ É o que justifica a faixa do slider ir até ao mesmo teto do `radius`: a região
/// impossível é alcançável e comporta-se, em vez de precisar de um clamp que esconderia
/// metade dos anéis legítimos.
#[test]
fn a_hole_wider_than_the_disc_is_an_empty_field() {
    for r in [0.0f32, 3.0, 7.0, 9.9] {
        assert_eq!(at(r, 10.0, 12.0, 0.0), 0.0, "r = {r}");
    }
}

/// **O KNOB ESTÁ NO PAINEL, e o kernel sabe dele.**
#[test]
fn the_inner_radius_is_reachable_and_uploaded() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == INNER_RADIUS)
        .expect("o Inner Radius tem de estar pintado");
    let outer = PARAM_HINTS
        .iter()
        .find(|h| h.param == "radius")
        .expect("o Radius existe");
    assert_eq!(
        hint.max, outer.max,
        "os dois raios partilham o teto — ver o ⚠️ do hint"
    );
    assert!(
        GPU_KERNEL.params.contains(&INNER_RADIUS),
        "o device tem de receber o raio interno: {:?}",
        GPU_KERNEL.params
    );
    assert!(
        GPU_KERNEL.wgsl_lib.contains("rs_inner_rise"),
        "a rampa interna falta no shader"
    );
}

// ───────────────────────── O VIÉS ANGULAR (folha 10) ─────────────────────────

/// Um sector estreito — a cunha FINA que a célula da conferência nomeia.
fn wedge() -> Sector {
    sector(-20.0, 20.0, 1.0)
}

/// O quanto a borda ANGULAR de uma cunha é macia: varre em ângulo e devolve a
/// largura da faixa em que a máscara não é nem `0` nem `1`.
///
/// ⚠️ **O raio da varredura tem de cair no PLATÔ radial**, e a primeira versão
/// não caía: a máscara é o `min` das duas rampas, então varrer a `r = 3` com
/// `soft = 0.9` e `radius = 8` media a rampa RADIAL (o platô acaba em `0.8`) e a
/// faixa dava 159 mesmo com a borda angular dura. *Uma sonda de uma borda tem de
/// estar longe da outra.*
fn angular_band(soft: f32, bias: f32) -> usize {
    let r = 0.4;
    (0..721)
        .filter(|k| {
            let a = (*k as f32 - 360.0).to_radians() * 0.25;
            let (lx, ly) = (r * a.cos(), r * a.sin());
            let m = sweep_mask(lx, ly, 8.0, 0.0, soft, bias, &wedge());
            m > 1e-4 && m < 1.0 - 1e-4
        })
        .count()
}

/// A mesma medida para a borda RADIAL — a que o viés **não** pode tocar.
fn radial_band(soft: f32, bias: f32) -> usize {
    (0..2000)
        .filter(|k| {
            let r = *k as f32 * 0.01;
            let m = sweep_mask(r, 0.0, 8.0, 0.0, soft, bias, &full());
            m > 1e-4 && m < 1.0 - 1e-4
        })
        .count()
}

/// ⭐ **A CÉLULA, medida:** uma cunha fina com borda radial macia e angular DURA
/// era inexprimível — `soft = 0.9` amaciava as duas.
#[test]
fn the_angular_edge_can_be_hard_while_the_radial_one_stays_soft() {
    let soft = 0.9;
    let (linked, hard_angle) = (angular_band(soft, 1.0), angular_band(soft, 0.0));
    assert!(
        linked > 50,
        "CONTROLE: com o viés no neutro a borda angular tem de ser MACIA ({linked})"
    );
    assert_eq!(hard_angle, 0, "viés 0 tem de dar uma borda angular DURA");
    // E a radial não se mexeu — é a metade que faz disto um viés e não um `soft`.
    assert_eq!(
        radial_band(soft, 1.0),
        radial_band(soft, 0.0),
        "o viés angular tocou a borda RADIAL"
    );
}

/// **O neutro é a identidade ao BIT** — `soft · 1.0 · half` é `soft · half` exacto
/// em IEEE-754, então nenhum campo já autorado se move.
#[test]
fn the_neutral_bias_is_bit_identical() {
    for &soft in &[0.0f32, 0.15, 0.5, 0.9, 1.0] {
        for k in 0..64 {
            let a = (k as f32 / 64.0 - 0.5) * 0.8;
            let (lx, ly) = (3.0 * a.cos(), 3.0 * a.sin());
            let with = sweep_mask(lx, ly, 8.0, 1.0, soft, 1.0, &wedge());
            // O oráculo é a expressão de antes, escrita à mão.
            let want = {
                let sec = wedge();
                let r = (lx * lx + ly * ly).sqrt();
                let rad = edge_ramp(r, 8.0, soft * 8.0).min(inner_rise(r, 1.0, soft * 1.0));
                let pa = pseudo_angle(lx, ly);
                let d = wrap_sym(pa - sec.pa_mid, sec.period);
                rad.min(edge_ramp(d, sec.pa_half, soft * sec.pa_half))
            };
            assert_eq!(with.to_bits(), want.to_bits(), "soft={soft} k={k}");
        }
    }
}

/// E o viés ABRE também: acima de `1` a borda angular fica mais macia que a
/// radial — a outra metade do eixo, que a célula não pedia e a lei dá de graça.
#[test]
fn a_bias_above_one_softens_the_angular_edge_further() {
    let soft = 0.3;
    let (neutral, wider) = (angular_band(soft, 1.0), angular_band(soft, 2.0));
    assert!(
        wider > neutral,
        "viés 2 tinha de alargar a faixa macia: {neutral} -> {wider}"
    );
}
