//! ⭐⭐⭐ **A BANCADA DA 2.ª MISSÃO DO PINCEL DE PLANO** — a memória do plano, o cursor na
//! superfície, o passo e a atenuação do traço, e a PLANURA (`SPEC_pincel_de_plano.md` §6 e §14,
//! gates G-12 e G-15 a G-19).
//!
//! > *«Com resultado inferior ao blender para produzir superfícies planas»* — o dono, 2026-09-16.
//!
//! ⚠️ **As fixturas de traço arrastado usadas aqui são todas de pincel NOSSO** (`copia_*`,
//! `ablacao_*`, `ctl_*`): a nota de proveniência do R-pré (5.ª passagem) põe as `fab_*` fora dos
//! gates, porque foram geradas com o perfil de fábrica do alvo carregado.
//!
//! ⚠️ **O traço arrastado do alvo não é determinístico ao bit** (`1,49e-8`, §14.1) e o nosso pousa
//! os dabs na nossa régua de píxeis: a paridade PONTO a ponto não é a afirmação (medido: até
//! `1,9e-2`, a posição sub-píxel de cada dab). **A afirmação é a PLANURA**, e ela bate:
//!
//! | fixtura (pincel nosso) | a nossa | a do alvo |
//! |---|---|---|
//! | valores de fábrica, 4 passagens | `0,092` | `0,092` |
//! | valores de fábrica, 8 passagens | `0,028` | `0,029` |
//! | sem a firmeza da normal, 8 | `1,188` | `1,380` |
//! | os nossos valores de ANTES, 8 | `1,134` | `1,114` |

use ph2d_mesh::{Mesh, Ray};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

use super::oraculo_do_pincel_de_plano::{Fixtura, comparar, correr, grelha, ler, olho, pincel};

const FAB: &str = "fabrica_e_traco";
const SUP: &str = "cursor_na_superficie";

/// A barra de paridade da porta por script — a do G-1 (`8,4` ULP de `f32`, espec §12).
const BARRA: f32 = 1e-6;
/// ⚠️ **O limiar de uma CONTAGEM de movidos, nomeado** (errata Q9 do R-pré): com «qualquer
/// mudança» as bossas contam `424` contra `417` do alvo, e a diferença são vértices que andaram
/// um ULP; com `1e-7` as duas contagens concordam.
const LIMIAR: f32 = 1e-7;

/// O pincel de uma fixtura da 2.ª missão — o raio vem em `raio_efectivo`.
fn pincel_fab(f: &mut Fixtura) -> Brush {
    if !f.cab.contains_key("raio_objeto") {
        let r = f
            .chave("raio_efectivo")
            .split_whitespace()
            .next()
            .expect("raio")
            .to_string();
        f.cab.insert("raio_objeto".into(), r);
    }
    if !f.cab.contains_key("modificador_carregado") {
        f.cab.insert("modificador_carregado".into(), "nao".into());
    }
    pincel(f)
}

/// Onde a vertical pelo ponto pedido corta a superfície VIVA — o que o raio do rato faz (errata
/// Q8: o bloco `c` destas fixturas traz os pontos PEDIDOS em `z = 0`).
fn na_superficie(m: &Mesh, x: f32, y: f32, olho: [f32; 3]) -> Option<[f32; 3]> {
    let origem = [x - olho[0] * 10.0, y - olho[1] * 10.0, -olho[2] * 10.0];
    m.raycast(&Ray::new(origem, olho)).map(|h| h.point)
}

/// A porta por script, com o cursor NA superfície.
fn correr_na_superficie(f: &mut Fixtura) -> Vec<[f32; 3]> {
    let mut m = grelha(f).expect("grelha");
    let b = pincel_fab(f);
    let e = olho(f);
    let mut s = SculptStroke::default();
    s.begin(&m);
    for c in f.cursores.clone() {
        let centro =
            na_superficie(&m, c[0], c[1], e).expect("o ponto pedido nao corta a superficie");
        s.dab(
            &mut m,
            &b,
            &Dab::at(centro, b.radius, e),
            Symmetry::default(),
        );
    }
    m.positions().to_vec()
}

/// Quantos vértices andaram mais do que [`LIMIAR`].
fn movidos(repouso: &[[f32; 3]], pos: &[[f32; 3]]) -> usize {
    repouso
        .iter()
        .zip(pos)
        .filter(|(r, p)| (0..3).any(|k| (r[k] - p[k]).abs() > LIMIAR))
        .count()
}

fn maior_distancia(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0, f32::max))
        .fold(0.0, f32::max)
}

/// ⭐ **G-15 — um dab com o cursor NA superfície reproduz o oráculo**, com os valores de fábrica do
/// perfil *aparar* e a memória da normal presa. ⚠️ A memória é semeada pelo primeiro dab, o que
/// não move nada (errata Q6: semeada no primeiro que MOVE, a reprodução erra `3,0e-03`).
#[test]
fn um_dab_com_o_cursor_na_superficie_reproduz_o_oraculo() {
    let mut f = ler(FAB, "passo_script_1dab_na_superficie");
    assert!(
        (f.num("firmeza_da_normal") - 1.0).abs() < f32::EPSILON,
        "a fixtura mudou de sujeito"
    );
    let nossa = correr_na_superficie(&mut f);
    let d = maior_distancia(&nossa, &f.saida);
    assert!(d <= BARRA, "max|Δ| {d:.3e} passa a barra {BARRA:.0e}");
    assert_eq!(movidos(&f.repouso, &nossa), movidos(&f.repouso, &f.saida));
    assert!(
        movidos(&f.repouso, &nossa) > 0,
        "o dab nao moveu nada: a fixtura nao contem o fenomeno"
    );
}

/// ⭐ **G-19 — com o cursor NA superfície, uma superfície já plana não se mexe** — e as outras
/// nove células da §2.2 reproduzem o oráculo na mesma porta. ⚠️ As células homónimas de `lei/`,
/// com o cursor fora do relevo, movem `274` e `271` vértices: elas testam a LEI, esta o PRODUTO.
#[test]
fn um_plano_ja_plano_nao_se_mexe_com_o_cursor_na_superficie() {
    let celulas = [
        "sup_rampa",
        "sup_degrau",
        "sup_sulco",
        "sup_sulco_raio02",
        "sup_sulco_raio06",
        "sup_area025",
        "sup_area10",
        "sup_area20",
        "sup_normal025",
        "sup_normal10",
        "sup_bossas",
    ];
    for nome in celulas {
        let mut f = ler(SUP, nome);
        let nossa = correr_na_superficie(&mut f);
        let d = maior_distancia(&nossa, &f.saida);
        assert!(
            d <= BARRA,
            "{nome}: max|Δ| {d:.3e} passa a barra {BARRA:.0e}"
        );
        if matches!(nome, "sup_rampa" | "sup_degrau") {
            // ⚠️ **A barra e não uma contagem:** num plano exacto a distância ao plano ajustado é
            // ruído de arredondamento, e o nosso anda até `1,8e-7` onde o do alvo anda `3e-8` —
            // os dois dentro da barra do G-19, e uma contagem com limiar `1e-7` leria ruído.
            let anda = maior_distancia(&nossa, &f.repouso);
            println!("{nome}: o maior passo e' {anda:.3e}");
            assert!(
                anda <= BARRA,
                "{nome}: uma superficie ja' plana mexeu-se {anda:.3e} com o cursor em cima dela"
            );
        } else {
            let (n, dele) = (movidos(&f.repouso, &nossa), movidos(&f.repouso, &f.saida));
            assert_eq!(n, dele, "{nome}: {n} movidos contra {dele}");
            assert!(n > 0, "{nome}: a celula viva nao moveu nada");
        }
    }
    assert_eq!(
        celulas.len(),
        11,
        "a populacao de `cursor_na_superficie` mudou"
    );
}

/// ⭐ **A memória do plano reproduz o oráculo** (as `12` fixturas de `firmeza/`), e **G-12**: a
/// firmeza do centro é MUDA numa superfície simétrica e VIVA num degrau — as duas metades, senão
/// um knob que mexesse em tudo passaria.
#[test]
fn a_memoria_do_plano_reproduz_o_oraculo_e_o_centro_e_mudo_so_na_simetria() {
    let mut saidas = std::collections::BTreeMap::new();
    for nome in [
        "firmeza_normal_00",
        "firmeza_normal_025",
        "firmeza_normal_05",
        "firmeza_normal_10",
        "firmeza_centro_00",
        "firmeza_centro_025",
        "firmeza_centro_05",
        "firmeza_centro_10",
        "firmeza_centro_rampa_00",
        "firmeza_centro_rampa_10",
        "firmeza_centro_bossas_00",
        "firmeza_centro_bossas_10",
    ] {
        let f = ler("firmeza", nome);
        let nossa = correr(&f);
        let (d, _, _) = comparar(&f, &nossa);
        assert!(
            d <= BARRA,
            "{nome}: max|Δ| {d:.3e} passa a barra {BARRA:.0e}"
        );
        assert_eq!(
            movidos(&f.repouso, &nossa),
            movidos(&f.repouso, &f.saida),
            "{nome}"
        );
        saidas.insert(nome, nossa);
    }
    let muda = maior_distancia(
        &saidas["firmeza_centro_bossas_00"],
        &saidas["firmeza_centro_bossas_10"],
    );
    assert!(
        muda <= BARRA,
        "G-12: nas bossas simetricas o knob do centro devia ser mudo ({muda:.3e})"
    );
    let viva = maior_distancia(&saidas["firmeza_centro_00"], &saidas["firmeza_centro_10"]);
    assert!(
        viva >= 1e-2,
        "G-12: no degrau o knob do centro devia estar vivo ({viva:.3e})"
    );
    let normal = maior_distancia(&saidas["firmeza_normal_00"], &saidas["firmeza_normal_10"]);
    assert!(
        normal >= 1e-2,
        "a firmeza da normal ficou muda no degrau ({normal:.3e})"
    );
}

/// ⭐ **G-16 — o passo do traço é 7 % do diâmetro**: saltos de `6`, `13`, `14` e `21` px dão `0`,
/// `1`, `2` e `3` dabs efectivos, pelo `walk` da casa (o primeiro dab do traço é o do pen-down).
///
/// ⛔ **A célula de `7` px é uma DIVERGÊNCIA DECLARADA** (errata Q4): o alvo dá `1` dab num salto
/// de exactamente um passo, e a casa recusa-o de propósito — o `<=` do [`ph2d_sculpt3d::walk`] tem
/// gate e a razão escrita (o dab fica para o evento seguinte, no mesmo sítio). A diferença só
/// existe se a caneta levantar nesse evento.
#[test]
fn o_passo_do_traco_e_sete_por_cento_do_diametro() {
    let plano = Brush {
        verb: Verb::Plane,
        ..Brush::default()
    };
    let passo = ph2d_sculpt3d::passo_do_traco(&plano, 50.0);
    assert_eq!(passo, 7.0, "a 50 px o passo e' EXACTAMENTE 7");
    for (salto, alvo, nosso) in [
        (6.0f32, 0u32, 0u32),
        (7.0, 1, 0),
        (13.0, 1, 1),
        (14.0, 2, 2),
        (21.0, 3, 3),
    ] {
        let n = ph2d_sculpt3d::walk([0.0, 0.0], [salto, 0.0], passo).map_or(0, |w| w.len());
        assert_eq!(
            n, nosso,
            "salto de {salto} px: {n} dabs (o alvo da' {alvo})"
        );
    }
}

/// ⭐ **G-17 — cada dab do traço arrastado vale `(1 + a)/2` de um dab por script.** Três metades:
/// a lei (`0,570`), o oráculo (o par de fixturas lê `0,563`, e a barra `0,02` é NOSSA, dita — o
/// resíduo é a posição sub-píxel), e o PRODUTO (o mesmo dab com e sem arrasto, razão exacta).
#[test]
fn a_atenuacao_do_traco_e_a_da_lei() {
    let base = Brush {
        verb: Verb::Plane,
        falloff: ph2d_sculpt3d::Falloff::Smooth,
        traco_arrastado: true,
        ..Brush::default()
    };
    let lei = base.factor_do_traco();
    assert!((lei - 0.570).abs() < 5e-4, "a lei da' {lei}");

    let amplitude = |f: &Fixtura| maior_distancia(&f.repouso, &f.saida);
    let script = ler(FAB, "passo_script_1dab_na_superficie");
    for (nome, dabs) in [
        ("passo_salto_7px", 1.0f32),
        ("passo_salto_13px", 1.0),
        ("passo_salto_14px", 2.0),
    ] {
        let razao = amplitude(&ler(FAB, nome)) / amplitude(&script) / dabs;
        assert!(
            (razao - lei).abs() <= 0.02 * dabs,
            "{nome}: o oraculo le' {razao} contra a lei {lei}"
        );
    }

    let mut f = ler(FAB, "passo_script_1dab_na_superficie");
    let mut b = pincel_fab(&mut f);
    let mut uma_vez = |arrastado: bool| {
        b.traco_arrastado = arrastado;
        let mut m = grelha(&f).expect("grelha");
        let e = olho(&f);
        let mut s = SculptStroke::default();
        s.begin(&m);
        for c in &f.cursores {
            let centro = na_superficie(&m, c[0], c[1], e).expect("superficie");
            s.dab(
                &mut m,
                &b,
                &Dab::at(centro, b.radius, e),
                Symmetry::default(),
            );
        }
        maior_distancia(m.positions(), &f.repouso)
    };
    let (por_script, arrastado) = (uma_vez(false), uma_vez(true));
    assert!(
        (arrastado / por_script - lei).abs() < 1e-4,
        "o produto: arrastado {arrastado} / por script {por_script} nao e' o factor {lei}"
    );
}

/// A régua da §14.5: o RMS do resíduo ao plano de mínimos quadrados dos vértices cuja posição de
/// REPOUSO tem `|x| ≤ 0,5` e `|y| ≤ 0,1` (errata Q5: escolhidos no repouso, medidos na saída).
fn planura(repouso: &[[f32; 3]], pos: &[[f32; 3]]) -> f64 {
    let sel: Vec<[f64; 3]> = repouso
        .iter()
        .zip(pos)
        .filter(|(r, _)| r[0].abs() <= 0.5 && r[1].abs() <= 0.1)
        .map(|(_, p)| [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])])
        .collect();
    assert_eq!(sel.len(), 441, "a faixa da regua mudou de populacao");
    let mut m = [[0.0f64; 3]; 3];
    let mut v = [0.0f64; 3];
    for p in &sel {
        let q = [p[0], p[1], 1.0];
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] += q[i] * q[j];
            }
            v[i] += q[i] * p[2];
        }
    }
    let s = cramer(m, v);
    #[allow(clippy::cast_precision_loss)]
    let n = sel.len() as f64;
    (sel.iter()
        .map(|p| (p[2] - (s[0] * p[0] + s[1] * p[1] + s[2])).powi(2))
        .sum::<f64>()
        / n)
        .sqrt()
}

fn cramer(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    let det = |a: [[f64; 3]; 3]| {
        a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
    };
    let d = det(m);
    let mut s = [0.0; 3];
    for (k, sk) in s.iter_mut().enumerate() {
        let mut a = m;
        for i in 0..3 {
            a[i][k] = v[i];
        }
        *sk = det(a) / d;
    }
    s
}

/// **O traço ARRASTADO** como a app o faz: eventos de rato de `1` px a `200` px por unidade, o
/// passo do [`ph2d_sculpt3d::passo_do_traco`] pelo `walk` da casa, e cada dab onde o rato corta a
/// superfície viva. ⭐ **Uma passagem é UMA travessia** — medido: lida como ida-e-volta a planura
/// de 4 passagens seria `0,028` contra os `0,092` do alvo; lida como travessia é `0,092`.
fn arrastar(f: &Fixtura, pincel: &Brush, passagens: usize) -> Vec<[f32; 3]> {
    const PX: f32 = 200.0;
    let mut m = grelha(f).expect("grelha");
    let olho = olho(f);
    let passo = ph2d_sculpt3d::passo_do_traco(pincel, pincel.radius * PX);
    let mut s = SculptStroke::default();
    s.begin(&m);
    let dab = |m: &mut Mesh, s: &mut SculptStroke, x_px: f32| {
        if let Some(c) = na_superficie(m, x_px / PX, 0.0, olho) {
            s.dab(
                m,
                pincel,
                &Dab::at(c, pincel.radius, olho),
                Symmetry::default(),
            );
        }
    };
    let mut x = -100.0f32;
    dab(&mut m, &mut s, x);
    let mut ancora = [x, 0.0];
    for t in 0..passagens {
        let dir = if t % 2 == 0 { 1.0 } else { -1.0 };
        for _ in 0..200 {
            x += dir;
            if let Some(passos) = ph2d_sculpt3d::walk(ancora, [x, 0.0], passo) {
                for q in passos {
                    dab(&mut m, &mut s, q[0]);
                }
                ancora = passos.anchor();
            }
        }
    }
    m.positions().to_vec()
}

/// A planura de UM arrasto com os valores de uma fixtura, e a do alvo, em fracção do repouso.
fn planura_do_arrasto(nome: &str) -> (f64, f64) {
    let mut f = ler(FAB, nome);
    let mut b = pincel_fab(&mut f);
    b.traco_arrastado = true;
    let passagens: usize = f
        .chave("passagens")
        .split_whitespace()
        .next()
        .and_then(|p| p.parse().ok())
        .expect("passagens");
    let repouso = planura(&f.repouso, &f.repouso);
    let nossa = planura(&f.repouso, &arrastar(&f, &b, passagens)) / repouso;
    (nossa, planura(&f.repouso, &f.saida) / repouso)
}

/// ⭐⭐ **G-18 — a firmeza da normal é a ALAVANCA do aparar**, as duas metades que o dono sente:
/// com os valores de fábrica, 8 passagens levam o relevo a `≤ 0,05` da rugosidade inicial; sem a
/// firmeza da normal, o mesmo traço deixa-o `≥ 1,0` (mais rugoso que em repouso). E a terceira: os
/// valores que o pincel tinha ANTES desta wave também não achatam — é o relato do dono.
///
/// As barras saem do vale MEDIDO no alvo (`0,029` · `1,380` · `1,114`), com folga dos dois lados.
#[test]
fn a_firmeza_da_normal_e_a_alavanca_do_aparar() {
    let (fabrica, dele) = planura_do_arrasto("copia_aparar_continuo_8");
    println!("fabrica, 8 passagens: nossa {fabrica:.3}  alvo {dele:.3}");
    assert!(
        fabrica <= 0.05,
        "os valores de fabrica deixam o relevo a {fabrica:.3} em 8 passagens"
    );
    let (sem_firmeza, dele) = planura_do_arrasto("ablacao_sem_firmeza_da_normal_8");
    println!("sem firmeza da normal: nossa {sem_firmeza:.3}  alvo {dele:.3}");
    assert!(
        sem_firmeza >= 1.0,
        "sem a firmeza da normal o relevo ficou a {sem_firmeza:.3}"
    );
    let (antes, dele) = planura_do_arrasto("ctl_valores_nossos_traco_arrastado_8");
    println!("os valores de antes: nossa {antes:.3}  alvo {dele:.3}");
    assert!(
        antes >= 1.0,
        "os valores de antes achataram ({antes:.3}) — o relato do dono sumiu"
    );
}

/// ⭐ **Os valores de FÁBRICA do nosso pincel de plano são os do perfil *aparar*** (espec §14.2) —
/// pelas portas por que o painel os lê, e comparados com a fixtura de pincel nosso que os escreve.
#[test]
fn o_pincel_de_plano_nasce_com_os_valores_do_aparar() {
    let mut f = ler(FAB, "copia_aparar_continuo_8");
    let alvo = pincel_fab(&mut f);
    let v = Verb::Plane;
    let fabrica = Brush::default();
    let pares = [
        ("forca", v.default_strength(), alvo.strength),
        ("dureza", v.default_hardness(), alvo.hardness),
        (
            "extensao do centro",
            fabrica.area_radius_frac,
            alvo.area_radius_frac,
        ),
        (
            "extensao da normal",
            fabrica.normal_radius_frac,
            alvo.normal_radius_frac,
        ),
        (
            "firmeza da normal",
            fabrica.plano_firmeza_normal,
            alvo.plano_firmeza_normal,
        ),
        (
            "firmeza do centro",
            fabrica.plano_firmeza_centro,
            alvo.plano_firmeza_centro,
        ),
        ("altura", fabrica.plano_altura, alvo.plano_altura),
        (
            "profundidade",
            fabrica.plano_profundidade,
            alvo.plano_profundidade,
        ),
    ];
    for (nome, nosso, dele) in pares {
        assert!(
            (nosso - dele).abs() < 1e-6,
            "{nome}: nasce {nosso}, o aparar tem {dele}"
        );
    }
    assert_eq!(v.default_accumulate(), alvo.accumulate, "acumular");
    assert_eq!(
        fabrica.plano_inversao, alvo.plano_inversao,
        "o que o Ctrl faz"
    );
    assert_eq!(
        v.default_falloff(ph2d_sculpt3d::RefMode::birth_for(v)),
        alvo.falloff,
        "curva"
    );
}

/// Um traço por script do pincel de fábrica, com o cursor na superfície, ao longo de `y` em `x0`.
fn traco_em_y(m: &mut Mesh, s: &mut SculptStroke, x0: f32, sim: Symmetry) {
    let b = Brush {
        verb: Verb::Plane,
        radius: 0.25,
        ..Brush::default()
    };
    let olho = [0.0, 0.0, -1.0];
    for k in 0..8u8 {
        let y = -0.2 + 0.05 * f32::from(k);
        let c = na_superficie(m, x0, y, olho).expect("superficie");
        s.dab(m, &b, &Dab::at(c, b.radius, olho), sim);
    }
}

/// ⭐ **A memória do plano é UMA POR PASSE DE SIMETRIA**: com o espelho em `X`, a firmeza da normal
/// em `1` e uma superfície simétrica, a saída é simétrica. ⛔ Com uma memória partilhada, a cópia
/// espelhada prenderia a normal da cópia directa, e a peça sairia torta.
#[test]
fn a_memoria_do_plano_e_uma_por_passe_de_simetria() {
    let f = ler(FAB, "copia_aparar_continuo_8");
    let mut m = grelha(&f).expect("grelha");
    let mut s = SculptStroke::default();
    s.begin(&m);
    traco_em_y(&mut m, &mut s, 0.3, Symmetry::MIRROR_X);
    let lado = 97usize;
    assert_eq!(f.repouso.len(), lado * lado, "a grelha mudou de forma");
    let mut pior = 0.0f32;
    for j in 0..lado {
        for k in 0..lado {
            let (i, par) = (j * lado + k, j * lado + (lado - 1 - k));
            let d = |v: usize| {
                let (p, r) = (m.positions()[v], f.repouso[v]);
                [p[0] - r[0], p[1] - r[1], p[2] - r[2]]
            };
            let (a, b) = (d(i), d(par));
            pior = pior
                .max((a[0] + b[0]).abs())
                .max((a[1] - b[1]).abs())
                .max((a[2] - b[2]).abs());
        }
    }
    assert!(
        movidos(&f.repouso, m.positions()) > 0,
        "o traco nao moveu nada: o gate mediria o vazio"
    );
    assert!(
        pior <= 1e-5,
        "a saida espelhada desvia {pior:.3e}: a memoria atravessou os passes"
    );
}

/// ⭐ **Cada traço novo ESQUECE a memória** (espec §14.5: levantar a caneta esquece a normal
/// fixada). O traço B depois do A, com `begin` no meio, é o traço B num traço novo.
#[test]
fn cada_traco_novo_esquece_a_memoria_do_plano() {
    let f = ler(FAB, "copia_aparar_continuo_8");
    let mut m = grelha(&f).expect("grelha");
    let mut s = SculptStroke::default();
    s.begin(&m);
    traco_em_y(&mut m, &mut s, -0.3, Symmetry::default());
    let depois_de_a = m.clone();
    s.begin(&m);
    traco_em_y(&mut m, &mut s, 0.3, Symmetry::default());

    let mut fresca = depois_de_a;
    let mut nova = SculptStroke::default();
    nova.begin(&fresca);
    traco_em_y(&mut fresca, &mut nova, 0.3, Symmetry::default());
    assert!(
        maior_distancia(m.positions(), f.repouso.as_slice()) > 1e-4,
        "o traco B nao moveu nada"
    );
    assert_eq!(
        m.positions(),
        fresca.positions(),
        "o traco B herdou a memoria do A"
    );
}
