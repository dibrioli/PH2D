//! ⭐⭐⭐ **O CAMPO COM ARCOS É O MESMO CAMPO** — o gate que decide a wave de 2026-09-16.
//!
//! A decomposição exacta existe para a marcha avaliar `22` primitivas onde avaliava `94`. Ela só
//! vale se o **campo não mudar**: a distância e — sobretudo — o **SINAL**.
//!
//! ⚠️ **O sinal é a metade que engana.** A distância a um arco é fácil de acertar e de conferir; o
//! enrolamento não. Entre a corda e o arco há uma **meia-lua** onde o polígono das cordas responde
//! *dentro* e a figura verdadeira responde *fora* (ou o contrário), e um erro ali é uma casca fina
//! de sinal trocado colada a cada quina — invisível numa amostragem grosseira e visível como um
//! risco no render. ⇒ a grelha é fina e a barra do sinal é **zero** longe do bordo.
//!
//! ⚠️ **O perfil é construído AQUI, à mão, e não vem do cozedor**: esta crate não depende dele, e
//! fazê-la depender para um teste poria uma aresta nova no grafo por causa de uma fixtura. O
//! caminho inteiro — desenho → cozedura → arcos → fita → placa — tem a sonda dele na
//! `ph2d-app-field3d`.

use ph2d_field::{FillRule, Profile};

/// Um avaliador em lote de um campo 2D — `(xs, ys) → valores`.
type Avaliador<'a> = &'a mut dyn FnMut(&[f32], &[f32]) -> Vec<f32>;

/// Um quadrado de lado `2·half` com as quatro quinas arredondadas a `r`, nas DUAS descrições.
///
/// A quina é um quarto de círculo ⇒ `bulge = tan(90°/4) = tan(22,5°)`. O sinal sai da orientação:
/// com os vértices em sentido anti-horário, o arco de uma quina convexa curva para a **direita** de
/// `a→b`, logo o bulge é negativo na convenção desta casa (positivo = esquerda).
fn quadrado_redondo(half: f64, r: f64, lados: usize) -> (Profile, Profile) {
    quadrado_redondo_com(half, r, lados, lados_do_cozedor(r, 1e-4))
}

/// Quantos segmentos o cozedor põe num quarto de círculo de raio `r` a esta tolerância — a mesma
/// conta do achatamento (`r·(1 − cos(θ/2)) = tol`). ⚠️ É a polilinha que o PRODUTO tem ao lado dos
/// arcos, e é ela que a especialização por região lia antes da cura: um fixture com a polilinha
/// «perfeita» esconderia o defeito, e um com só as cordas exageraria-o.
fn lados_do_cozedor(r: f64, tol: f64) -> usize {
    let passo = 2.0 * (1.0 - tol / r).acos();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let n = (std::f64::consts::FRAC_PI_2 / passo).ceil() as usize;
    n.max(1)
}

/// O quadrado redondo com as duas densidades escolhidas: `lados` para a referência `sem`, e
/// `lados_com` para a polilinha que viaja AO LADO dos arcos em `com`.
fn quadrado_redondo_com(half: f64, r: f64, lados: usize, lados_com: usize) -> (Profile, Profile) {
    let bulge = -(std::f64::consts::FRAC_PI_8).tan();
    // Os oito vértices da decomposição: entrada e saída de cada quina, em sentido anti-horário.
    let cantos = [
        (half, half, 0.0_f64), // canto +x +y
        (-half, half, 1.0),    // −x +y
        (-half, -half, 2.0),   // −x −y
        (half, -half, 3.0),    // +x −y
    ];
    let mut exacto: Vec<([f32; 2], f32)> = Vec::new();
    let mut denso: Vec<[f32; 2]> = Vec::new();
    let mut poli_com: Vec<[f32; 2]> = Vec::new();
    for (cx, cy, q) in cantos {
        // O centro do arco desta quina, e os ângulos de entrada/saída.
        let (ox, oy) = (cx - r * cx.signum(), cy - r * cy.signum());
        // ângulo inicial do arco desta quina, em múltiplos de 90°
        let a0 = std::f64::consts::FRAC_PI_2 * q;
        let a1 = a0 + std::f64::consts::FRAC_PI_2;
        #[allow(clippy::cast_possible_truncation)]
        let ponto = |a: f64| [(ox + r * a.cos()) as f32, (oy + r * a.sin()) as f32];
        // A recta que CHEGA a esta quina termina no início do arco.
        exacto.push((ponto(a0), bulge as f32));
        denso.push(ponto(a0));
        poli_com.push(ponto(a0));
        for k in 1..lados {
            let a = a0 + (a1 - a0) * f64::from(u32::try_from(k).unwrap_or(1)) / lados as f64;
            denso.push(ponto(a));
        }
        for k in 1..lados_com {
            let a = a0 + (a1 - a0) * f64::from(u32::try_from(k).unwrap_or(1)) / lados_com as f64;
            poli_com.push(ponto(a));
        }
        // O fim do arco é o início da recta seguinte — entra como vértice de recta.
        exacto.push((ponto(a1), 0.0));
        denso.push(ponto(a1));
        poli_com.push(ponto(a1));
    }
    let com = Profile::with_arcs(vec![(poli_com, exacto)], FillRule::NonZero, 1e-4)
        .expect("o quadrado redondo é válido");
    let sem = Profile::new(vec![denso], FillRule::NonZero, 1e-4).expect("a polilinha é válida");
    (com, sem)
}

fn avalia(p: &Profile, xs: &[f32], ys: &[f32]) -> Vec<f32> {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    let tree = crate::profile::sd_profile(p, &Tree::x(), &Tree::y());
    let shape = crate::Engine::from(tree);
    let mut eval = crate::Engine::new_float_slice_eval();
    let tape = shape.ez_float_slice_tape();
    let zs = vec![0.0f32; xs.len()];
    eval.eval(&tape, xs, ys, &zs).expect("avalia").to_vec()
}

#[test]
fn um_arco_da_o_mesmo_campo_que_a_polilinha_que_ele_substitui() {
    // `192` lados por quarto de volta: a polilinha de controlo é MUITO mais fina que a tolerância,
    // para que a discordância medida seja do ARCO e não da referência.
    let (com, sem) = quadrado_redondo(0.5, 0.15, 192);
    assert_eq!(com.arc_count(), 4, "as quatro quinas têm de ser arcos");
    assert_eq!(sem.arc_count(), 0, "o controlo não pode ter arcos");
    assert_eq!(
        com.prim_count(),
        8,
        "a decomposição exacta tem 8 primitivas"
    );
    assert!(
        sem.segment_count() > 700,
        "a referência tem de ser densa (deu {})",
        sem.segment_count()
    );

    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    const N: i32 = 260;
    for i in 0..=N {
        for j in 0..=N {
            #[allow(clippy::cast_possible_truncation)]
            {
                xs.push((-0.8 + 1.6 * f64::from(i) / f64::from(N)) as f32);
                ys.push((-0.8 + 1.6 * f64::from(j) / f64::from(N)) as f32);
            }
        }
    }
    let a = avalia(&com, &xs, &ys);
    let b = avalia(&sem, &xs, &ys);

    // A referência é uma polilinha de 768 lados sobre arcos de raio `0,15`: a flecha dela é
    // `r·(1−cos(π/384)) ≈ 5e-6`. A barra é uma ordem de grandeza acima disso, e NÃO a tolerância
    // declarada do perfil — *a barra sai do erro da RÉGUA, não de um número redondo*.
    const BARRA: f32 = 5e-5;
    let mut pior = 0.0f32;
    let mut trocas = 0usize;
    let mut pior_troca = 0.0f32;
    for ((va, vb), (x, y)) in a.iter().zip(&b).zip(xs.iter().zip(&ys)) {
        pior = pior.max((va - vb).abs());
        if va.signum() != vb.signum() && va.abs() > BARRA && vb.abs() > BARRA {
            trocas += 1;
            if vb.abs() > pior_troca {
                pior_troca = vb.abs();
                let _ = (x, y);
            }
        }
    }
    assert_eq!(
        trocas, 0,
        "⛔ o SINAL trocou em {trocas} pontos longe do bordo (pior |f| = {pior_troca:.6}) — \
         é a meia-lua entre a corda e o arco"
    );
    assert!(
        pior <= BARRA,
        "os dois campos afastam-se {pior:.3e} (barra {BARRA:.3e})"
    );
}

/// ⭐ **E um perfil SEM arcos emite a fita de sempre** — a metade que prova que esta wave não tocou
/// no caminho que já existia.
#[test]
fn um_perfil_sem_arcos_da_o_campo_de_sempre() {
    let quadrado = Profile::new(
        vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]],
        FillRule::NonZero,
        1e-4,
    )
    .expect("o quadrado é válido");
    assert_eq!(quadrado.arc_count(), 0);
    let xs = [0.5f32, 0.5, -0.5, 0.25];
    let ys = [0.5f32, 1.5, 0.5, 0.5];
    let esperado = [-0.5f32, 0.5, 0.5, -0.25];
    let v = avalia(&quadrado, &xs, &ys);
    for (i, (got, want)) in v.iter().zip(&esperado).enumerate() {
        assert!(
            (got - want).abs() < 1e-6,
            "ponto {i}: {got} em vez de {want}"
        );
    }
}

/// ⭐⭐⭐ **A ÁRVORE ESPECIALIZADA POR REGIÃO TEM O MESMO CAMPO — E A MESMA NORMAL — QUE A GLOBAL,
/// NUM PERFIL COM ARCOS** (smoke do dono, 2026-09-16: *«arestas ainda visíveis»*).
///
/// # O defeito que este gate existe para apanhar
///
/// A wave do arco pôs os arcos na árvore GLOBAL, e com ela o modo RENDER (placa) ficou liso. Mas o
/// modo MODEL traça na **CPU**, e a CPU especializa a árvore por ladrilho: a folha especializada lia
/// o perfil pelo `ProfileIndex`, construído da **polilinha densa** — só segmentos. A normal é o
/// gradiente dessa árvore ⇒ constante em cada segmento ⇒ **faixas** na luz. Medido na cena `5`, vista
/// de frente, `1920×1080`: **`12 196`** picos de faceta na CPU contra **`0`** na placa.
///
/// # ⛔ Por que o gate irmão estava VERDE
///
/// `the_specialised_tree_agrees_inside_its_region` mede exactamente esta concordância — mas o corpus
/// dele nasce todo por `Profile::new`, **sem um único arco**, e ele mede **valores**, nunca a normal.
/// Antes da wave do arco as duas árvores eram a mesma polilinha, e concordavam por construção.
///
/// ⇒ este gate tem as duas metades que faltavam: um perfil **com** arcos e a **normal** — que é o
/// que a luz mostra (a mesma lição da W54: *a régua da suavidade é a normal, não a silhueta*).
#[test]
fn a_regiao_especializada_tem_o_campo_e_a_normal_da_arvore_global() {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    let (com, _) = quadrado_redondo(0.5, 0.15, 8);
    assert!(com.arc_count() == 4, "o fixture tem de trazer arcos");
    let idx = crate::profile_index::ProfileIndex::build(&com);

    let tape_de = |t: Tree| {
        let shape = crate::Engine::from(t);
        (
            shape.ez_float_slice_tape(),
            crate::Engine::new_float_slice_eval(),
        )
    };
    let (g_tape, mut g_eval) = tape_de(crate::profile::sd_profile(&com, &Tree::x(), &Tree::y()));
    let mut global = |xs: &[f32], ys: &[f32]| {
        let zs = vec![0.0f32; xs.len()];
        g_eval.eval(&g_tape, xs, ys, &zs).expect("avalia").to_vec()
    };

    // Regiões pequenas espalhadas sobre as QUINAS, que é onde vivem os arcos.
    const H: f32 = 2e-3; // passo das diferenças centrais
    let mut pior_valor = 0.0f32;
    let mut pior_normal = 0.0f32;
    let mut perto = 0usize;
    for (qx, qy) in [(1.0f32, 1.0f32), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
        for passo in 0..6 {
            // O centro da região anda ao longo da bissectriz da quina, por dentro e por fora do bordo.
            #[allow(clippy::cast_precision_loss)]
            let t = 0.40 + 0.03 * passo as f32;
            let c = [qx * t, qy * t];
            let half = 0.06f32;
            let (lo, hi) = ([c[0] - half, c[1] - half], [c[0] + half, c[1] + half]);
            let (r_tape, mut r_eval) = tape_de(crate::profile::sd_profile_in_region(
                &com,
                &idx,
                &Tree::x(),
                &Tree::y(),
                lo,
                hi,
                false,
                None,
            ));
            let mut regiao = |xs: &[f32], ys: &[f32]| {
                let zs = vec![0.0f32; xs.len()];
                r_eval.eval(&r_tape, xs, ys, &zs).expect("avalia").to_vec()
            };
            // Uma grelha estritamente DENTRO da região, com folga para as diferenças centrais.
            let (mut xs, mut ys) = (Vec::new(), Vec::new());
            const N: i32 = 24;
            for i in 0..N {
                for j in 0..N {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        let fx = (i as f32 + 0.5) / N as f32;
                        let fy = (j as f32 + 0.5) / N as f32;
                        xs.push(lo[0] + 2.0 * H + fx * (2.0 * half - 4.0 * H));
                        ys.push(lo[1] + 2.0 * H + fy * (2.0 * half - 4.0 * H));
                    }
                }
            }
            let (a, b) = (regiao(&xs, &ys), global(&xs, &ys));
            for k in 0..xs.len() {
                pior_valor = pior_valor.max((a[k] - b[k]).abs());
            }
            // A NORMAL, onde ela importa: perto da superfície.
            let grad = |f: Avaliador<'_>, x: f32, y: f32| {
                let v = f(&[x + H, x - H, x, x], &[y, y, y + H, y - H]);
                let (gx, gy) = (v[0] - v[1], v[2] - v[3]);
                let n = gx.hypot(gy).max(f32::MIN_POSITIVE);
                (gx / n, gy / n)
            };
            for k in 0..xs.len() {
                if b[k].abs() > 0.02 {
                    continue;
                }
                perto += 1;
                let (ax, ay) = grad(&mut regiao, xs[k], ys[k]);
                let (bx, by) = grad(&mut global, xs[k], ys[k]);
                let ang = (ax * bx + ay * by).clamp(-1.0, 1.0).acos().to_degrees();
                pior_normal = pior_normal.max(ang);
            }
        }
    }
    assert!(
        perto > 500,
        "o corpus tem de tocar a superfície (tocou {perto} pontos)"
    );
    // ⚠️ A barra dos VALORES é a do gate irmão (`1e-5`): o corte é conservador, logo o mínimo sobre
    // as primitivas guardadas É o mínimo global — só a aritmética de `f32` os separa.
    assert!(
        pior_valor < 1.0e-5,
        "a árvore da região discorda da global em {pior_valor:e} num perfil com ARCOS"
    );
    // ⚠️ A barra da NORMAL: uma faceta do cozedor a esta tolerância salta `~4°`, e o ruído das
    // diferenças centrais em `f32` com `h = 2e-3` fica muito abaixo de um décimo de grau.
    assert!(
        pior_normal < 0.25,
        "⛔ a normal da árvore da região afasta-se {pior_normal:.3}° da global — são as FACETAS \
         que o modo MODEL mostra como faixas"
    );
}

/// A distância com sinal ANALÍTICA do quadrado redondo — o oráculo dos gates de sinal.
///
/// É a do *round box* 2D: `q = |p| − (h − r)`, `d = ‖máx(q, 0)‖ + mín(máx(qₓ, q_y), 0) − r`. Ela não
/// passa por perfil nenhum, por árvore nenhuma e por índice nenhum — é por isso que serve de régua
/// às três.
fn oraculo(half: f32, r: f32, x: f32, y: f32) -> f32 {
    let (qx, qy) = (x.abs() - (half - r), y.abs() - (half - r));
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r
}

/// ⭐⭐⭐ **UM PONTO EXACTAMENTE SOBRE UMA CORDA TEM O SINAL CERTO — nos TRÊS leitores.**
///
/// ⛔⛔ **O defeito que este gate apanhou foi da 1.ª versão da árvore GLOBAL**, e a grelha do gate
/// irmão nunca o veria: um ponto a `0,0437` de profundidade dentro da peça, **em cima da corda** de
/// uma quina, lia `+0,0437` — fora. O enrolamento pelo raio punha-o de um lado da corda e a meia-lua
/// do outro. *Uma grelha nunca cai num conjunto de medida nula* — os pontos PÕEM-SE (a lição da
/// casa, dita pelo vinco da W147).
///
/// Por isso ele põe pontos: sobre cada corda (dentro do segmento, onde a ambiguidade existe) e
/// sobre o prolongamento dela (onde não existe), e compara a árvore global, a especializada e o
/// índice com o [`oraculo`] — que não passa por nenhum dos três.
#[test]
fn um_ponto_sobre_a_corda_tem_o_sinal_certo_nos_tres_leitores() {
    const HALF: f32 = 0.5;
    const R: f32 = 0.15;
    let (base, _) = quadrado_redondo(f64::from(HALF), f64::from(R), 8);
    // ⚠️ As QUATRO combinações, porque a regra do empate tem dois ramos (arco à esquerda / à
    // direita) e a meia-lua tem duas leis (`NonZero` soma `∓1`, `EvenOdd` só conta): a figura
    // espelhada em `x` percorre-se no sentido HORÁRIO e os arcos dela curvam para a ESQUERDA.
    for (com, rotulo) in [
        (base.clone(), "anti-horário · NonZero"),
        (espelhado(&base, FillRule::NonZero), "horário · NonZero"),
        (
            com_regra(&base, FillRule::EvenOdd),
            "anti-horário · EvenOdd",
        ),
        (espelhado(&base, FillRule::EvenOdd), "horário · EvenOdd"),
    ] {
        um_ponto_sobre_a_corda(&com, rotulo, HALF, R);
    }
}

/// A mesma figura espelhada em `x` — o sentido inverte-se, e com ele o lado de cada arco.
fn espelhado(p: &Profile, fill: FillRule) -> Profile {
    let poli: Vec<[f32; 2]> = p.contours()[0].iter().map(|q| [-q[0], q[1]]).collect();
    let arcs: Vec<([f32; 2], f32)> = p.arcs()[0]
        .iter()
        .map(|(q, b)| ([-q[0], q[1]], -b))
        .collect();
    Profile::with_arcs(vec![(poli, arcs)], fill, p.tolerance()).expect("o espelho é válido")
}

/// A mesma figura com outra lei de preenchimento.
fn com_regra(p: &Profile, fill: FillRule) -> Profile {
    Profile::with_arcs(
        vec![(p.contours()[0].clone(), p.arcs()[0].clone())],
        fill,
        p.tolerance(),
    )
    .expect("válido")
}

fn um_ponto_sobre_a_corda(com: &Profile, rotulo: &str, half: f32, r: f32) {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    let idx = crate::profile_index::ProfileIndex::build(com);
    let ev = |t: Tree, xs: &[f32], ys: &[f32]| {
        let shape = crate::Engine::from(t);
        let tape = shape.ez_float_slice_tape();
        let mut e = crate::Engine::new_float_slice_eval();
        let zs = vec![0.0f32; xs.len()];
        e.eval(&tape, xs, ys, &zs).expect("avalia").to_vec()
    };

    // Os pontos POSTOS: sobre cada corda de quina, a várias fracções dela.
    let arcos = &com.arcs()[0];
    let n = arcos.len();
    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    for i in 0..n {
        let ((a, bulge), (b, _)) = (arcos[i], arcos[(i + 1) % n]);
        if bulge == 0.0 {
            continue;
        }
        for t in [0.1f32, 0.25, 0.5, 0.75, 0.9] {
            xs.push(a[0] + t * (b[0] - a[0]));
            ys.push(a[1] + t * (b[1] - a[1]));
        }
    }
    assert_eq!(xs.len(), 20, "quatro cordas, cinco pontos em cada");

    let global = ev(
        crate::profile::sd_profile(com, &Tree::x(), &Tree::y()),
        &xs,
        &ys,
    );
    let mut indice = Vec::new();
    idx.sd_batch(&xs, &ys, &mut indice);
    let mut pior = (0.0f32, String::new());
    for k in 0..xs.len() {
        let certo = oraculo(half, r, xs[k], ys[k]);
        // A região: uma caixa pequena à volta do ponto, com ele no MEIO (a âncora fica longe dele).
        let (lo, hi) = ([xs[k] - 0.03, ys[k] - 0.03], [xs[k] + 0.03, ys[k] + 0.03]);
        let regiao = ev(
            crate::profile::sd_profile_in_region(
                com,
                &idx,
                &Tree::x(),
                &Tree::y(),
                lo,
                hi,
                false,
                None,
            ),
            &xs[k..=k],
            &ys[k..=k],
        )[0];
        for (nome, v) in [
            ("global", global[k]),
            ("região", regiao),
            ("índice", indice[k]),
        ] {
            assert!(
                v.signum() == certo.signum(),
                "⛔ [{rotulo}] ({}, {}) está a {certo:+.5} (oráculo) e o leitor `{nome}` diz \
                 {v:+.5} — o SINAL sobre a corda",
                xs[k],
                ys[k]
            );
            let erro = (v - certo).abs();
            if erro > pior.0 {
                pior = (erro, format!("{nome} em ({}, {})", xs[k], ys[k]));
            }
        }
    }
    // ⚠️ A barra do VALOR: os três avaliam a MESMA geometria em `f32`, e o oráculo é exacto.
    assert!(
        pior.0 < 2e-6,
        "[{rotulo}] o pior valor afasta-se {:e} do oráculo — {}",
        pior.0,
        pior.1
    );
}

/// ⭐⭐ **E o campo INTEIRO da árvore global bate o oráculo** — não só a polilinha que ele substitui.
#[test]
fn a_arvore_global_com_arcos_bate_o_oraculo_analitico() {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    let (com, _) = quadrado_redondo(0.5, 0.15, 8);
    let tree = crate::profile::sd_profile(&com, &Tree::x(), &Tree::y());
    let shape = crate::Engine::from(tree);
    let tape = shape.ez_float_slice_tape();
    let mut e = crate::Engine::new_float_slice_eval();
    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    for i in 0..=200 {
        for j in 0..=200 {
            #[allow(clippy::cast_precision_loss)]
            {
                xs.push(-0.8 + 1.6 * i as f32 / 200.0);
                ys.push(-0.8 + 1.6 * j as f32 / 200.0);
            }
        }
    }
    let zs = vec![0.0f32; xs.len()];
    let v = e.eval(&tape, &xs, &ys, &zs).expect("avalia").to_vec();
    let mut pior = 0.0f32;
    for k in 0..xs.len() {
        pior = pior.max((v[k] - oraculo(0.5, 0.15, xs[k], ys[k])).abs());
    }
    assert!(
        pior < 2e-6,
        "a árvore com arcos afasta-se {pior:e} do oráculo analítico"
    );
}

/// ⭐⭐⭐ **O CORTE POR REGIÃO GUARDA A PRIMITIVA QUE A FLECHA DECIDE** — o gate que duas mutações
/// SOBREVIVENTES pediram.
///
/// O corte espacial do índice deita fora toda primitiva que não pode ser a mais próxima de nenhum
/// ponto da região — e para um arco ele só conhece a CORDA, mais a desigualdade *«todo ponto do arco
/// está a menos da flecha da corda»*. Esquecer a flecha no **minorante** ou no **majorante** deixava
/// os cinco gates acima verdes (mutações M7 e M8, 2026-09-16): nenhuma região do corpus caía onde a
/// flecha decide. *Um corpus onde uma desigualdade nunca aperta não testa a desigualdade.*
///
/// As duas regiões abaixo foram construídas à mão para que ela aperte, e a conta está ao lado:
///
/// * **cintura** — um arco sobe até `y = 1` e uma recta passa em `y = 1,2`; a caixa está entre os
///   dois. A corda do arco está a `1,08` da caixa e o arco a `0,08`; a recta fecha o `dmax` em
///   `0,12²`. Sem a flecha no minorante, o arco lê `1,08²` e é deitado fora.
/// * **pé** — um arco sobe até `y = 1` sobre uma recta em `y = −0,3`; a caixa encosta à corda
///   (`y = 0`) por dentro. A recta está a `0,2` e é a mais próxima; o arco está a `1,1`. Sem a
///   flecha no majorante, o `far` do arco lê `0,12²` e empurra o `dmax` para baixo da recta.
#[test]
fn o_corte_por_regiao_guarda_a_primitiva_que_a_flecha_decide() {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    // O arco das duas figuras: corda de `x = −2` a `x = 2` em `y = 0`, flecha `1` para CIMA.
    const B: f32 = 0.5; // bulge = 2·flecha/corda = 2·1/4
    let cintura = Profile::with_arcs(
        vec![(
            vec![[-2.0, 0.0], [0.0, 1.0], [2.0, 0.0], [2.0, 1.2], [-2.0, 1.2]],
            // anti-horário: o arco vai para +x e curva para CIMA, que é a ESQUERDA ⇒ bulge > 0
            vec![
                ([-2.0, 0.0], B),
                ([2.0, 0.0], 0.0),
                ([2.0, 1.2], 0.0),
                ([-2.0, 1.2], 0.0),
            ],
        )],
        FillRule::NonZero,
        1e-4,
    )
    .expect("a cintura é válida");
    let pe = Profile::with_arcs(
        vec![(
            vec![
                [-2.0, 0.0],
                [-2.0, -0.3],
                [2.0, -0.3],
                [2.0, 0.0],
                [0.0, 1.0],
            ],
            // anti-horário: o arco volta para −x curvando para CIMA, que é a DIREITA ⇒ bulge < 0
            vec![
                ([-2.0, 0.0], 0.0),
                ([-2.0, -0.3], 0.0),
                ([2.0, -0.3], 0.0),
                ([2.0, 0.0], -B),
            ],
        )],
        FillRule::NonZero,
        1e-4,
    )
    .expect("o pé é válido");

    for (nome, p, lo, hi) in [
        ("cintura", &cintura, [-0.02f32, 1.08f32], [0.02f32, 1.12f32]),
        ("pé", &pe, [-0.02f32, -0.12f32], [0.02f32, -0.08f32]),
    ] {
        let idx = crate::profile_index::ProfileIndex::build(p);
        let ev = |t: Tree, xs: &[f32], ys: &[f32]| {
            let shape = crate::Engine::from(t);
            let tape = shape.ez_float_slice_tape();
            let mut e = crate::Engine::new_float_slice_eval();
            let zs = vec![0.0f32; xs.len()];
            e.eval(&tape, xs, ys, &zs).expect("avalia").to_vec()
        };
        let (mut xs, mut ys) = (Vec::new(), Vec::new());
        for i in 0..9 {
            for j in 0..9 {
                #[allow(clippy::cast_precision_loss)]
                {
                    xs.push(lo[0] + (hi[0] - lo[0]) * (i as f32 + 0.5) / 9.0);
                    ys.push(lo[1] + (hi[1] - lo[1]) * (j as f32 + 0.5) / 9.0);
                }
            }
        }
        let global = ev(
            crate::profile::sd_profile(p, &Tree::x(), &Tree::y()),
            &xs,
            &ys,
        );
        let regiao = ev(
            crate::profile::sd_profile_in_region(
                p,
                &idx,
                &Tree::x(),
                &Tree::y(),
                lo,
                hi,
                false,
                None,
            ),
            &xs,
            &ys,
        );
        let mut culled = Vec::new();
        idx.sd_batch_culled(&xs, &ys, &mut Vec::new(), &mut culled);
        let mut pior = 0.0f32;
        for k in 0..xs.len() {
            pior = pior
                .max((regiao[k] - global[k]).abs())
                .max((culled[k] - global[k]).abs());
        }
        assert!(
            pior < 1e-5,
            "⛔ [{nome}] o corte por região deitou fora a primitiva mais próxima: a distância \
             afasta-se {pior:e} da árvore global — a esfera-marcha ATRAVESSARIA a peça"
        );
    }
}
