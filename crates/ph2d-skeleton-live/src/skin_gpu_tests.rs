//! ⭐⭐⭐ **OS GATES DA METADE DA CPU DA W2** — ver o cabeçalho do [`super`].
//!
//! A afirmação que eles sustentam é a que **define** o *vertex shader*:
//!
//! > a pele posa-se com `Σ w_i · (M_i · p)` sobre uma tabela de pesos por OSSO que é do BIND —
//! > sem lei de dobra, sem tendões, sem projecção no eixo.

use super::*;
use ph2d_skeleton::SkinBone;

/// ⭐⭐ **Uma corrente de `n` TENDÕES, e o do meio DOBRA em quatro sub-ossos.**
///
/// ⚠️⚠️ **A 1.ª redacção desta fixtura tinha a corrente toda RECTA — e assim ela não exercitava a
/// quota da dobra, que é o achado inteiro deste módulo.** Com todos os ossos rectos a
/// `weights_from` só normaliza, e uma mutação que passasse os pesos por tendão CRUS para a mistura
/// sobreviveria a tudo. *Uma fixtura no ponto neutro de uma lei não testa essa lei* — a armadilha
/// que este repo já pagou com o corpus no neutro de um knob.
fn pele_com_dobra(n: usize, ang: f64) -> Skin {
    let l = 40.0_f64;
    let mut bones: Vec<SkinBone> = Vec::new();
    for k in 0..n {
        let base = Xform([1.0, 0.0, 0.0, 1.0, k as f64 * l, 0.0]);
        let spec = ph2d_skeleton::bend::BoneSpec {
            length: l,
            strength: 1.5,
            // O tendão do MEIO dobra; os outros são rectos, e é o contraste que dá o controlo.
            segments: if k == n / 2 { 4 } else { 1 },
            curve: ph2d_skeleton::bend::Bend {
                inn: [0.0, 0.25],
                out: [0.0, -0.25],
            },
        };
        let antes = bones.len();
        SkinBone::bent(
            base,
            spec,
            Xform::IDENTITY,
            Xform::IDENTITY,
            u32::try_from(k).expect("poucos tendoes"),
            &mut bones,
        );
        let (s, c) = (ang * (k + 1) as f64).sin_cos();
        let pivo = [k as f64 * l, 0.0];
        for b in &mut bones[antes..] {
            b.pose = Xform([
                c,
                s,
                -s,
                c,
                pivo[0] - (c * pivo[0] - s * pivo[1]),
                pivo[1] - (s * pivo[0] + c * pivo[1]),
            ]);
        }
    }
    Skin::new(bones).expect("a corrente nao e' vazia")
}

/// Uma corrente RECTA — o controlo da de cima.
fn pele(n: usize, ang: f64) -> Skin {
    let l = 40.0_f64;
    let bones: Vec<SkinBone> = (0..n)
        .map(|k| {
            let base = Xform([1.0, 0.0, 0.0, 1.0, k as f64 * l, 0.0]);
            let (s, c) = (ang * (k + 1) as f64).sin_cos();
            let pivo = [k as f64 * l, 0.0];
            // Roda à volta do pivô: T(pivo) · R · T(-pivo).
            let pose = Xform([
                c,
                s,
                -s,
                c,
                pivo[0] - (c * pivo[0] - s * pivo[1]),
                pivo[1] - (s * pivo[0] + c * pivo[1]),
            ]);
            let mut b = SkinBone::new(base, l, 1.5, Xform::IDENTITY, Xform::IDENTITY)
                .expect("o osso de repouso e' invertivel");
            b.pose = pose;
            b.tendon = u32::try_from(k).expect("poucos ossos");
            b
        })
        .collect();
    Skin::new(bones).expect("a corrente nao e' vazia")
}

/// Uma malha de `n × m` pontos sobre a faixa que a corrente cobre.
fn malha(n: usize, m: usize, largura: f64) -> Mesh2d {
    let mut rest = Vec::with_capacity(n * m);
    for j in 0..m {
        for i in 0..n {
            rest.push([
                i as f64 * largura / (n - 1) as f64,
                j as f64 * 30.0 / (m - 1) as f64 - 15.0,
            ]);
        }
    }
    Mesh2d {
        rest,
        tris: Vec::new(),
        size: [1, 1],
    }
}

/// Pesos por TENDÃO iguais aos que um solver daria a uma faixa: cada ponto pertence sobretudo ao
/// osso mais perto dele ao longo do `x`.
fn por_tendao(mesh: &Mesh2d, tendoes: usize, largura: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(mesh.rest.len() * tendoes);
    for p in &mesh.rest {
        let centro = p[0] / largura * (tendoes - 1) as f64;
        let mut soma = 0.0;
        let brutos: Vec<f64> = (0..tendoes)
            .map(|j| {
                let d = (j as f64 - centro).abs();
                let w = (1.0 - d).max(0.0);
                soma += w;
                w
            })
            .collect();
        for w in brutos {
            out.push(if soma > 0.0 { w / soma } else { 0.0 });
        }
    }
    out
}

/// ⭐⭐⭐ **A LEI DO SHADER REPRODUZ A LEI DO PRODUTO** — esta é a W2 inteira, numa asserção.
///
/// ⚠️ **A barra é DERIVADA e não escolhida:** a referência corre em `f64` (é o produto) e a lei da
/// placa em `f32`, logo o desvio é o erro de representação nas coordenadas em jogo. Com o eixo a
/// chegar a `~160` unidades, um `f32` resolve `~1e-5` ali (`160 · 2⁻²³ ≈ 1,9e-5`), e a barra é
/// **`4 ULP`** dessa magnitude.
///
/// ⚠️ **E o CONTROLO está dentro:** uma pose ERRADA (a corrente com outro ângulo) tem de violar a
/// mesma barra. *Sem ele, uma barra folgada passaria sobre um shader que ignorasse os pesos.*
#[test]
fn a_lei_da_placa_reproduz_a_lei_do_produto() {
    const LARGURA: f64 = 160.0;
    let p = pele_com_dobra(4, 0.25);
    let m = malha(13, 5, LARGURA);
    let pt = por_tendao(&m, 4, LARGURA);
    let gpu = PeleGpu::empacota(&p, Xform::IDENTITY, &m, &pt);
    // ⭐ **Quatro TENDÕES, mas SETE ossos** — o do meio partiu-se em quatro sub-ossos. É esse
    // numero que o shader vê, e é ele que a tabela cobre.
    assert_eq!(
        gpu.ossos,
        p.bones().len(),
        "a tabela tem de cobrir os OSSOS resolvidos, nao os tendoes"
    );
    assert!(
        gpu.ossos > 4,
        "a fixtura devia ter um osso que DOBRA (leu {} ossos para 4 tendoes) — sem ele a quota \
         nunca e' exercitada",
        gpu.ossos
    );
    assert_eq!(gpu.rest.len(), m.rest.len());

    let poses = poses_do_quadro(&p);
    let placa = posa_como_a_placa(&gpu, &poses);

    // A referência: a porta do PRODUTO, ponto a ponto.
    let mut w = p.scratch();
    let barra = LARGURA * f64::from(f32::EPSILON) * 4.0;
    let mut pior = 0.0_f64;
    for (v, &q) in m.rest.iter().enumerate() {
        let esperado = p.point_with(q, &pt[v * 4..(v + 1) * 4], &mut w);
        let obtido = placa[v];
        let d = (f64::from(obtido[0]) - esperado[0]).hypot(f64::from(obtido[1]) - esperado[1]);
        pior = pior.max(d);
    }
    println!("  lei da placa x lei do produto: pior desvio {pior:.3e} (barra {barra:.3e})");
    assert!(
        pior <= barra,
        "a lei da placa erra {pior:.3e} contra a barra de {barra:.3e} — ela NAO e' a lei do produto"
    );

    // ⛔ O CONTROLO: com as poses de OUTRA corrente, a mesma barra tem de ser violada.
    let outra = poses_do_quadro(&pele_com_dobra(4, 0.60));
    let errado = posa_como_a_placa(&gpu, &outra);
    let pior_errado = m
        .rest
        .iter()
        .enumerate()
        .map(|(v, &q)| {
            let esperado = p.point_with(q, &pt[v * 4..(v + 1) * 4], &mut w);
            (f64::from(errado[v][0]) - esperado[0]).hypot(f64::from(errado[v][1]) - esperado[1])
        })
        .fold(0.0_f64, f64::max);
    assert!(
        pior_errado > barra * 100.0,
        "com as poses ERRADAS o desvio foi {pior_errado:.3e}, dentro de 100x a barra — esta \
         fixtura nao distingue uma pose da outra e o gate nao afirma nada"
    );
}

/// ⭐⭐ **A TABELA DE PESOS É DO BIND: mover um osso NÃO a muda.**
///
/// É a propriedade que faz o quadro enviar só `N` afins — *se ela fosse falsa, o shader teria de
/// recalcular a quota da dobra por vértice por quadro, e a W2 não existiria.*
#[test]
fn mover_um_osso_nao_muda_a_tabela_de_pesos() {
    const LARGURA: f64 = 160.0;
    let m = malha(9, 3, LARGURA);
    let pt = por_tendao(&m, 3, LARGURA);
    let a = PeleGpu::empacota(&pele_com_dobra(3, 0.10), Xform::IDENTITY, &m, &pt);
    let b = PeleGpu::empacota(&pele_com_dobra(3, 1.20), Xform::IDENTITY, &m, &pt);
    assert_eq!(
        a.pesos, b.pesos,
        "a tabela de pesos mudou com a POSE — ela tem de ser uma grandeza do bind"
    );
    assert_eq!(a.rest, b.rest, "o repouso mudou com a pose");
}

/// ⛔ **Sem peso nenhum o ponto fica INTACTO** — o mesmo caso degenerado que a
/// [`ph2d_skeleton::Skin::blend`] declara, e não a origem.
#[test]
fn um_ponto_que_ninguem_reclama_fica_onde_esta() {
    let m = malha(4, 2, 100.0);
    let zeros = vec![0.0_f64; m.rest.len() * 2];
    let p = pele(2, 0.5);
    let gpu = PeleGpu::empacota(&p, Xform::IDENTITY, &m, &zeros);
    let placa = posa_como_a_placa(&gpu, &poses_do_quadro(&p));
    for (v, &q) in m.rest.iter().enumerate() {
        // ⚠️ **A barra é DERIVADA, como a da paridade acima** — a 1.ª redacção escreveu `1e-6` a
        // olho e reprovou sobre produto CORRECTO: `33,333333333333336` em `f64` lê-se
        // `33,333332` em `f32`, que é `1,3e-6` de distância. *O ponto fica intacto a menos da
        // representação, e essa é a única promessa que se pode fazer.*
        let barra = q[0].abs().max(q[1].abs()).max(1.0) * f64::from(f32::EPSILON) * 4.0;
        assert!(
            (f64::from(placa[v][0]) - q[0]).abs() <= barra
                && (f64::from(placa[v][1]) - q[1]).abs() <= barra,
            "o vertice {v} sem dono saiu de {q:?} para {:?} (barra {barra:.3e})",
            placa[v]
        );
    }
}
