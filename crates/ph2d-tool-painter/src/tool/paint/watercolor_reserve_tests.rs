//! Os gates da LEI da reserva ([`super`], doc 41) — a disputa, o campo e as cercas dele, medidos
//! sem ferramenta e sem composite. Os gates do PRODUTO (a costura no traço em U, pela porta do
//! pincel) vivem em `tests/watercolor_selfseam.rs`.

use super::*;

const W: usize = 160;
const H: usize = 96;

/// Carimba UM dab redondo de reserva `v` nos dois planos, como o depósito faz.
fn dab(level: &mut [u8], prox: &mut [u8], (cx, cy): (f32, f32), r: f32, v: f32) {
    for y in 0..H {
        for x in 0..W {
            let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
            let dn = (dx * dx + dy * dy).sqrt() / r;
            if dn < 1.0 {
                splat_level(&mut level[y * W + x], &mut prox[y * W + x], dn, v);
            }
        }
    }
}

fn style(reserve_r: u16) -> WetStrokeStyle {
    WetStrokeStyle {
        reserve_r,
        ..WetStrokeStyle::capture(&ph2d_painter_brush::BrushSpec::default(), 0.0)
    }
}

fn field(
    level: &[u8],
    prox: &[u8],
    r: u16,
    win: ((usize, usize), (usize, usize)),
) -> ReserveFields {
    ReserveFields::build((level, prox), (W, W * H, win.0, win.1), &[], &style(r)).unwrap()
}

/// Numa passada só o nível é o `v` do próprio dab, do centro à beira — o peso da disputa cancela
/// (o mesmo dab ganha no numerador e no denominador). É o que mantém a borda externa como era.
#[test]
fn a_single_pass_reads_the_dabs_own_reserve_from_core_to_rim() {
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    dab(&mut l, &mut p, (80.0, 48.0), 30.0, 0.6);
    let want = (0.6f32 * 255.0).round() as i32;
    let mut touched = 0;
    for i in 0..W * H {
        if p[i] > 0 {
            touched += 1;
            assert!(
                (i32::from(l[i]) - want).abs() <= 2,
                "px {i}: nível {} contra {want} (prox {})",
                l[i],
                p[i]
            );
        }
    }
    assert!(
        touched > 2000,
        "piso de população: o dab tocou {touched} px"
    );
}

/// **O AFILAMENTO DA BEIRA (`T`) — a anatomia da borda externa que o dono aprovou.**
///
/// ⛔⛔ Ele é a metade da lei que NENHUM outro gate desta crate podia ver, e uma prova de mutação
/// disse-o: os três irmãos que lêem o campo amostram onde `prox = 255` (ali `T = 1` **por
/// construção**) e o quarto afirma sobre o plano do NÍVEL. ⇒ apagar o `taper[..]` do composto
/// **sobrevivia a todos eles**, e o que essa ausência devolve é a borda dura que esta wave existe
/// para curar.
///
/// ⭐ Num dab SÓ o nível é constante, logo `Σ L·q / Σ q = v` **exactamente** e o campo degenera em
/// `T(p) · v` — é isso que torna a medição limpa: o que sobra é o afilamento nu, sem a disputa.
#[test]
fn the_rim_tapers_over_the_outer_shell_and_dies_at_the_edge() {
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    let (cx, cy, r, v) = (80.0f32, 48.0f32, 30.0f32, 0.6f32);
    dab(&mut l, &mut p, (cx, cy), r, v);
    let f = field(&l, &p, 3, ((0, 0), (W, H)));
    let at = |dn: f32| f.sample(3, cx + dn * r, cy);

    // (1) Dentro da casca o campo é o `v` NU — o afilamento não toca no planalto.
    for dn in [0.0f32, 0.30, 0.60, 0.80] {
        assert!(
            (at(dn) - v).abs() <= 2e-2,
            "dn {dn}: o planalto lê {} contra {v}",
            at(dn)
        );
    }
    // (2) A casca AFILA — e é aqui que a mutação sangra: sem o `T` isto continua a ler `v`.
    let (meio, fora) = (at(0.92), at(0.98));
    assert!(
        meio < v * 0.65,
        "dn 0,92 devia estar a meio do afilamento e lê {meio} (v = {v})"
    );
    assert!(
        fora < v * 0.25,
        "dn 0,98 devia estar quase morto e lê {fora} (v = {v})"
    );
    // (3) E ele é MONÓTONO ao longo da casca — um degrau, ou um `max` com um valor qualquer,
    // passaria em (2) e reprovaria aqui.
    let mut prev = f32::INFINITY;
    for k in 0..=20 {
        let dn = 0.85 + 0.0075 * k as f32;
        let aqui = at(dn);
        assert!(
            aqui <= prev + 1e-3,
            "dn {dn}: o afilamento SUBIU ({prev} → {aqui})"
        );
        prev = aqui;
    }
    // (4) CONTROLO — a largura da casca é a `RESERVE_RIM_RAMP` e não um número qualquer: logo
    // ANTES dela o campo ainda é o planalto. Sem esta metade, um afilamento que comesse metade do
    // dab passaria em (1)..(3).
    let antes = 1.0 - RESERVE_RIM_RAMP - 0.02;
    assert!(
        at(antes) >= v - 2e-2,
        "a casca começou cedo demais: dn {antes} lê {}",
        at(antes)
    );
}

/// A regra de domínio do `max` FICA: re-entintar restaura, e uma passada pálida não clareia o
/// MIOLO de uma escura (o planalto do depósito disputa com peso cheio).
#[test]
fn the_max_rule_survives_in_the_core() {
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    dab(&mut l, &mut p, (60.0, 48.0), 30.0, 0.9); // escura
    dab(&mut l, &mut p, (84.0, 48.0), 30.0, 0.3); // pálida por cima, 0,8 r ao lado
    // Todo o planalto da escura (dn ≤ 0,62) guarda 0,9 AO BIT, mesmo onde a pálida está mais perto.
    let dark = (0.9f32 * 255.0).round() as u8;
    for x in 42..=78 {
        let dn = ((x as f32 + 0.5 - 60.0).abs()) / 30.0;
        if dn <= 0.61 {
            assert_eq!(l[48 * W + x], dark, "x={x} dn={dn:.2}");
        }
    }
    // E o inverso: a fresca por cima de um rasto pálido restaura-o no miolo dela.
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    dab(&mut l, &mut p, (60.0, 48.0), 30.0, 0.2);
    dab(&mut l, &mut p, (60.0, 48.0), 30.0, 1.0);
    assert_eq!(l[48 * W + 60], 255);
}

/// **O gate da costura, na lei nua.** Onde a pálida cobre a beira da escura o nível desce CONTÍNUO
/// ao longo da cauda do feather da escura. A barra é a do oráculo transposta: a transição 10–90 %
/// não é mais estreita do que `0,15·r` só da disputa (o alisamento soma por cima) — e o degrau por
/// pixel nunca passa da inclinação da cauda do feather. CONTROLO: a lei antiga (`max` de
/// `v·rampa`) reprova as duas metades, e está reproduzida aqui para o provar.
#[test]
fn a_pale_pass_over_a_dark_edge_yields_along_the_dark_feather() {
    let r = 40.0f32;
    let (v1, v2) = (0.9f32, 0.35f32);
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    dab(&mut l, &mut p, (50.0, 48.0), r, v1);
    dab(&mut l, &mut p, (50.0 + 1.2 * r, 48.0), r, v2);
    let row: Vec<f32> = (50..=98)
        .map(|x| f32::from(l[48 * W + x]) / 255.0)
        .collect();
    let max_step = row
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max);
    let width = |row: &[f32]| {
        let (hi, lo) = (v2 + 0.9 * (v1 - v2), v2 + 0.1 * (v1 - v2));
        let a = row.iter().position(|&v| v < hi).unwrap();
        let b = row.iter().position(|&v| v < lo).unwrap();
        (b - a) as f32
    };
    // A cauda do feather normalizado cai 1/(0,38·r) por px; o nível escuro escala-a.
    let tail_slope = v1 / (0.38 * r);
    assert!(
        max_step <= tail_slope * 1.15 + 1.0 / 255.0,
        "degrau {max_step:.4}/px contra a cauda do feather {tail_slope:.4}/px"
    );
    assert!(
        width(&row) >= 0.15 * r,
        "10-90 % = {} px, r = {r}",
        width(&row)
    );

    // CONTROLO — a lei de antes: max(v·rampa), rampa nos últimos 15 % do raio.
    let old: Vec<f32> = (50..=98)
        .map(|x| {
            let dn1 = (x as f32 + 0.5 - 50.0).abs() / r;
            let dn2 = (x as f32 + 0.5 - (50.0 + 1.2 * r)).abs() / r;
            let ramp = |dn: f32| ((1.0 - dn) / RESERVE_RIM_RAMP).clamp(0.0, 1.0);
            (v1 * ramp(dn1)).max(v2 * ramp(dn2))
        })
        .collect();
    let old_step = old
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max);
    assert!(
        old_step > tail_slope * 2.0 && width(&old) < 0.15 * r,
        "o controlo tem de conter o fenómeno: degrau {old_step:.4}, largura {}",
        width(&old)
    );
}

/// O alisamento **não atravessa papel seco**: dois washes separados por UMA coluna seca ficam
/// cada um com o nível dele ao bit, qualquer que seja o raio.
#[test]
fn the_field_never_crosses_dry_paper() {
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    for y in 20..70 {
        for x in 30..70 {
            (l[y * W + x], p[y * W + x]) = (230, 255);
        }
        for x in 71..110 {
            (l[y * W + x], p[y * W + x]) = (60, 255);
        }
    }
    let f = field(&l, &p, 24, ((0, 0), (W, H)));
    for y in 20..70 {
        assert!((f.sample(24, 69.0, y as f32) - 230.0 / 255.0).abs() < 1e-6);
        assert!((f.sample(24, 71.0, y as f32) - 60.0 / 255.0).abs() < 1e-6);
    }
    // CONTROLO: tapada a coluna seca, os dois níveis misturam-se na fronteira.
    for y in 20..70 {
        (l[y * W + 70], p[y * W + 70]) = (60, 255);
    }
    let f = field(&l, &p, 24, ((0, 0), (W, H)));
    assert!(
        f.sample(24, 69.0, 45.0) < 200.0 / 255.0,
        "ligados, difundem"
    );
}

/// O campo é função do MAPA, não da janela: a mesma amostra lida por duas janelas diferentes (as
/// duas com o suporte `R` inteiro à volta) é igual AO BIT — somas inteiras, uma divisão.
/// **A PREMISSA do `reserve_reach` na janela do composite: uma margem abaixo de `R` lê OUTRO campo.**
///
/// ⭐ O campo é `Σ L·q / Σ q`, uma **RAZÃO** — logo onde `L` é localmente constante truncar a caixa
/// não muda nada (numerador e denominador perdem os mesmos pesos), e a única região onde a margem
/// importa é a COSTURA, onde `L` varia dentro de `R`. Medido sobre duas passadas de reservas
/// diferentes, `R = 24`, com a costura em `x ≈ 78`:
///
/// | margem até à costura | pior `\|Δ\|` no campo |
/// |---|---|
/// | 38 (≥ `R`) | `0,000094` |
/// | 18 | `0,028832` |
/// | 8  | `0,138007` |
///
/// ⚠️ **A primeira linha é o CONTROLO**: com margem acima de `R` as duas janelas concordam — e o
/// irmão [`the_field_is_a_function_of_the_map_not_of_the_window`] exige-o **AO BIT**. Sem ela, um
/// campo que ignorasse a janela por completo passaria nas outras duas.
#[test]
fn a_caixa_truncada_le_outro_campo() {
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    dab(&mut l, &mut p, (60.0, 48.0), 30.0, 0.85);
    dab(&mut l, &mut p, (96.0, 50.0), 30.0, 0.3);
    let r = 24u16;
    let full = field(&l, &p, r, ((0, 0), (W, H)));
    let pior = |ox: usize| -> f32 {
        let part = field(&l, &p, r, ((ox, 0), (W - ox, H)));
        ((ox + 1)..(W - 2))
            .map(|x| (full.sample(r, x as f32, 48.0) - part.sample(r, (x - ox) as f32, 48.0)).abs())
            .fold(0.0, f32::max)
    };
    let (largo, meio, apertado) = (pior(40), pior(60), pior(70));
    // CONTROLO — margem 38 ≥ R = 24: as duas janelas leem o MESMO campo.
    assert!(
        largo < 1e-3,
        "controlo: com margem acima de R as janelas divergiram {largo}"
    );
    // A margem deficiente muda o campo, e MAIS quanto mais apertada.
    assert!(
        meio > 1e-2,
        "margem 18 < R = 24 devia divergir e leu {meio}"
    );
    assert!(
        apertado > 5e-2,
        "margem 8 < R = 24 devia divergir mais e leu {apertado}"
    );
    assert!(
        apertado > meio && meio > largo,
        "o erro devia crescer ao apertar a margem: {largo} · {meio} · {apertado}"
    );
}

/// **A FIAÇÃO: a janela do composite reserva o raio do campo.**
///
/// ⛔⛔ Ela **não é alcançável por pixel**, e isso está MEDIDO e não suposto: apagar este termo
/// deixa o [`the_reserve_field_keeps_incremental_equal_to_full`] **verde** nos dois raios da
/// fixtura — o erro do campo mora a `22 px` de qualquer dab daquele quadro, onde a lavagem já não
/// pesa, e o quadro seguinte reescreve por cima. A PREMISSA está no irmão
/// [`a_caixa_truncada_le_outro_campo`]; o que sobra afirmar é que o `pad` da janela **consulta** o
/// raio do campo, e isso lê-se no ficheiro que o calcula.
///
/// ⚠️ *Um gate de texto é o mais fraco desta crate — e é o mais forte que esta metade admite.*
/// Ele falha ALTO se alguém reformatar a linha, que é o modo de falha barato.
#[test]
fn a_janela_do_composite_reserva_o_raio_do_campo() {
    let src = include_str!("watercolor_render/window.rs");
    assert!(
        src.contains("reserve_radius("),
        "a janela deixou de derivar o raio do campo da reserva"
    );
    // ⚠️ A metade que conta: o raio não pode ficar numa variável MORTA — ele entra no alcance.
    assert!(
        src.contains("reach.max(self.paint.wet_styles.reserve_reach(cur))"),
        "o raio do campo é derivado e NÃO entra no alcance da janela"
    );
}

#[test]
fn the_field_is_a_function_of_the_map_not_of_the_window() {
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    dab(&mut l, &mut p, (60.0, 48.0), 30.0, 0.85);
    dab(&mut l, &mut p, (96.0, 50.0), 30.0, 0.3);
    let r = 6u16;
    let full = field(&l, &p, r, ((0, 0), (W, H)));
    let (ox, oy) = (37usize, 11usize);
    let part = field(&l, &p, r, ((ox, oy), (W - ox - 9, H - oy - 5)));
    let mut seen = 0;
    for y in (oy + 8)..(H - 14) {
        for x in (ox + 8)..(W - 18) {
            let a = full.sample(r, x as f32, y as f32);
            let b = part.sample(r, (x - ox) as f32, (y - oy) as f32);
            assert_eq!(a.to_bits(), b.to_bits(), "({x},{y})");
            seen += usize::from(a > 0.0);
        }
    }
    assert!(seen > 1500, "piso de população: {seen}");
}

/// Re-carimbar o mesmo dab não mexe um bit (o traço passa ~20 dabs por pixel), e sem mixer o
/// campo nem existe — Charge = 1 fica byte-idêntico por AUSÊNCIA.
#[test]
fn restamping_is_idempotent_and_no_mixer_means_no_field() {
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    dab(&mut l, &mut p, (80.0, 48.0), 30.0, 0.47);
    let (l0, p0) = (l.clone(), p.clone());
    dab(&mut l, &mut p, (80.0, 48.0), 30.0, 0.47);
    assert!(l == l0 && p == p0);
    assert!(ReserveFields::build((&[], &[]), (W, W * H, (0, 0), (W, H)), &[], &style(3)).is_none());
}

/// A cauda a seco É a do depósito: `feather` tem o planalto até `0,62` e cai linear a zero — a
/// disputa não inventa um perfil. Se alguém mexer no `feather`, este gate manda rever a cauda.
#[test]
fn the_dry_claim_is_the_deposits_own_feather() {
    use super::super::watercolor_accum::feather;
    for dn in [0.0f32, 0.3, 0.62, 0.7, 0.81, 0.9, 0.99] {
        let from_feather = (feather(dn) / feather(1.0 - CLAIM_TAIL_DRY)).min(1.0);
        let claim = ((1.0 - dn) / CLAIM_TAIL_DRY).min(1.0);
        let slack = if dn < 0.62 { 0.09 } else { 1e-5 }; // o planalto desce 8 %; a disputa satura-o
        assert!(
            (from_feather - claim).abs() <= slack,
            "dn={dn}: {from_feather} contra {claim}"
        );
    }
}

/// **O Rewet difunde à escala do PINCEL**, e a janela do composite sabe-o: o raio passa do Bleed
/// (`7 px`) quando o pincel é grande, e nunca do tecto declarado.
#[test]
fn rewet_diffuses_at_the_scale_of_the_brush() {
    assert_eq!(
        reserve_radius(96.0, 7, 7, 0.0),
        7,
        "a seco: o joelho, preso ao core_r"
    );
    assert_eq!(reserve_radius(96.0, 7, 7, 1.0), 24, "Rewet cheio: 0,25 r");
    assert_eq!(
        reserve_radius(8.0, 4, 7, 1.0),
        7,
        "pincel pequeno: o Bleed manda"
    );
    assert_eq!(reserve_radius(4096.0, 48, 48, 1.0) as usize, RESERVE_R_MAX);
}

/// O campo lê-se por BILINEAR: a meio de dois pixels vale a média deles, não um dos dois — é o que
/// tira a escada de pixel da leitura em coordenadas deformadas (o Ragged Edge desloca até 6 px).
#[test]
fn the_field_is_read_bilinear_not_nearest() {
    let (mut l, mut p) = (vec![0u8; W * H], vec![0u8; W * H]);
    for y in 0..H {
        for x in 0..W {
            (l[y * W + x], p[y * W + x]) = (if x < 80 { 200 } else { 100 }, 255);
        }
    }
    let f = field(&l, &p, 1, ((0, 0), (W, H)));
    let (a, b) = (f.sample(1, 79.0, 40.0), f.sample(1, 80.0, 40.0));
    let mid = f.sample(1, 79.5, 40.0);
    assert!(a > b + 0.05, "a fixtura tem de ter um degrau: {a} {b}");
    assert!(
        (mid - 0.5 * (a + b)).abs() < 1e-6,
        "meio pixel lê {mid}, entre {a} e {b}"
    );
}
