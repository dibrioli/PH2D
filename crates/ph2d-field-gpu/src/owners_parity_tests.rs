//! O gate da lei do dono — ver [`super`].

use super::{Mistura, on_cpu, on_device};
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::{hybrid::Registry, owners::Owners};

/// Uma esfera posta em `x`, como um documento de um nó — a forma que o [`Owners`] recebe.
fn bola(x: f32, raio: f32) -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: raio },
            Xform::at(x, 0.0, 0.0),
        )],
        NodeId(0),
    )
    .expect("a esfera posta")
}

/// A margem da marcha — a mesma ordem de grandeza da tolerância de acerto (`2e-4`).
const MARGEM: f32 = 1.0e-3;
/// Quanto mede um pixel em mundo, nesta cena — a largura que a fronteira de cor consome.
const PIXEL: f32 = 4.4e-3;

/// ⭐⭐⭐ **A grelha onde a lei MUDA**, e ela é construída, não varrida.
///
/// ⛔ Uma grelha uniforme sobre a caixa da peça responde `t = 0` em quase todo lado — e *um corpus
/// no ponto NEUTRO de uma lei não testa essa lei*. As duas esferas encontram-se no plano `x = 0`,
/// num anel de raio `√(0,2² − 0,15²)`; é ali que a mistura vive, e é dali que a varredura sai.
fn amostras() -> Vec<([f32; 3], f32)> {
    let r_anel = (0.2f32 * 0.2 - 0.15 * 0.15).sqrt();
    let mut v = Vec::new();
    for j in 0..24 {
        #[allow(clippy::cast_precision_loss)]
        let a = std::f32::consts::TAU * j as f32 / 24.0;
        let (s, c) = a.sin_cos();
        // ⭐ A travessia da fronteira: `x` varre o vale entre as duas, em passos de um pixel.
        for k in -8i32..=8 {
            #[allow(clippy::cast_precision_loss)]
            let x = k as f32 * PIXEL;
            for w in [0.0f32, PIXEL, PIXEL * 4.0] {
                v.push(([x, r_anel * c, r_anel * s], w));
            }
        }
    }
    // ⭐ E os pólos, onde a resposta é inequívoca: cada esfera sozinha, mais a folha distante.
    for (cx, raio) in [(-0.15f32, 0.2f32), (0.15, 0.2), (0.9, 0.1)] {
        for d in [[0.0f32, 0.0, 1.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0]] {
            v.push(([cx + d[0] * raio, d[1] * raio, d[2] * raio], PIXEL));
        }
    }
    v
}

fn cena() -> Owners {
    let reg = Registry::new();
    // ⚠️ **Três folhas e não duas**: com duas, a rede do filtro (*«ninguém contém o ponto»*) e o
    // filtro a sério dão a mesma resposta, e o ramo que a terceira exercita nunca corre.
    let docs = [bola(-0.15, 0.2), bola(0.15, 0.2), bola(0.9, 0.1)];
    Owners::new(&docs, &reg, MARGEM)
}

/// ⭐⭐⭐ **A LEI DO DONO DO DISPOSITIVO É A DA CPU.**
///
/// # A barra, e de onde ela sai
///
/// Os **índices** são um desempate: eles não têm meio termo, e a exigência é `100 %` — mas só onde
/// a pergunta tem resposta, isto é onde os dois campos **não** empatam à precisão de `f32`. Na
/// fronteira exacta dois números iguais dão donos diferentes por um ULP, e exigir o contrário seria
/// pedir ao dispositivo que reproduzisse os bits de um `f64`.
///
/// O **`t`** é uma diferença de dois campos dividida por uma largura de mundo: a barra é `2e-3`
/// absoluto, que é o que a propagação de `f32` sobre uma fita de esfera dá nesta escala
/// (`|∇| ≈ 2`, `d ≈ 0,4`, `ε ≈ 6e-8` ⇒ `~1e-5` por campo, amplificado por `1/(2·width)` com
/// `width = 4,4e-3` ⇒ `~2e-3`).
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn a_lei_do_dono_do_dispositivo_e_a_da_cpu() {
    let owners = cena();
    let am = amostras();
    assert!(
        am.len() > 400,
        "a grelha encolheu para {} — esta lei mede-se onde ela muda",
        am.len()
    );
    let cpu = on_cpu(&owners, &am);
    let Some(gpu) = on_device(&owners, &am) else {
        println!("sem adaptador — o gate da lei do dono fica por exercitar");
        return;
    };

    // ⭐ **A POPULAÇÃO da pergunta**: quantas amostras têm de facto uma mistura viva. Sem este
    // piso, uma lei que respondesse `t = 0` em todo lado passaria com `0,000` de desvio.
    let vivas = cpu.iter().filter(|m| m.t > 1.0e-4).count();
    assert!(
        vivas > 100,
        "só {vivas} amostras têm mistura viva — o corpus está no ponto neutro da lei"
    );

    let mut pior_t = 0.0f32;
    let mut trocas = 0usize;
    let mut decididas = 0usize;
    for (i, ((p, w), (c, g))) in am.iter().zip(cpu.iter().zip(gpu.iter())).enumerate() {
        let d = (c.t - g.t).abs();
        assert!(
            d <= 2.0e-3,
            "amostra {i} em {p:?} (largura {w}): t da CPU {} contra {} do dispositivo",
            c.t,
            g.t
        );
        pior_t = pior_t.max(d);
        // ⚠️ **Só onde o desempate É um desempate**: `t` perto de `0,5` quer dizer que os dois
        // campos valem o mesmo, e ali qualquer ordem é legítima.
        if c.t < 0.49 {
            decididas += 1;
            if c.a != g.a || c.b != g.b {
                trocas += 1;
            }
        }
    }
    assert!(
        decididas > 300,
        "só {decididas} amostras têm dono decidido — o gate não mede o desempate"
    );
    assert_eq!(
        trocas, 0,
        "{trocas} de {decididas} amostras decididas trocaram de dono entre os motores"
    );
    println!("lei do dono · pior |Δt| {pior_t:.3e} · {decididas} decididas · {vivas} com mistura");
}

/// ⭐⭐ **O CONTROLO: a régua responde.** Sem ele, um arnês que devolvesse a resposta da CPU dos
/// dois lados leria `0,000` e passaria para sempre.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn a_regua_da_lei_do_dono_acusa_uma_troca() {
    let owners = cena();
    let am = amostras();
    let Some(gpu) = on_device(&owners, &am) else {
        println!("sem adaptador — o controlo fica por exercitar");
        return;
    };
    // A mesma cena com as folhas por OUTRA ordem: a resposta certa passa a ser outra, e a régua
    // tem de o dizer.
    let reg = Registry::new();
    let trocada = Owners::new(
        &[bola(0.15, 0.2), bola(-0.15, 0.2), bola(0.9, 0.1)],
        &reg,
        MARGEM,
    );
    let cpu = on_cpu(&trocada, &am);
    let diferentes = am
        .iter()
        .zip(cpu.iter().zip(gpu.iter()))
        .filter(|(_, (c, g)): &(_, (&Mistura, &Mistura))| c.a != g.a)
        .count();
    assert!(
        diferentes > 100,
        "trocar a ordem das folhas mudou só {diferentes} donos — a régua não está a olhar"
    );
}
