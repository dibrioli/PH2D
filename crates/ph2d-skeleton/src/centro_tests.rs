//! Os gates da lei do centro de rotação. ⚠️ A lei é **pura** — uma pele e um vector de pesos —, logo
//! mede-se sem mundo nenhum e sem geometria de arte.

use crate::{Skin, SkinBone, Xform};

/// Dois ossos em linha, com a junta em `x = j`.
fn cadeia(j: f64, comprimento: f64, rot: f64) -> Skin {
    let (c, s) = (rot.cos(), rot.sin());
    Skin::new(vec![
        SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            j,
            1.0,
            Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            Xform::IDENTITY,
        )
        .expect("o 1.o osso"),
        SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, j, 0.0]),
            comprimento - j,
            1.0,
            Xform([c, s, -s, c, j, 0.0]),
            Xform::IDENTITY,
        )
        .expect("o 2.o osso"),
    ])
    .expect("dois ossos")
}

/// ⭐⭐⭐ **O CENTRO É A JUNTA QUE OS DOIS OSSOS PARTILHAM** — e não um ponto estimado.
///
/// ⚠️ **A fixtura põe a junta FORA do meio de propósito** (`x = 2` numa barra de `7`): com ela ao
/// meio, o centróide da forma calha nela e um estimador que não estima nada passaria. *Foi
/// exactamente isso que aconteceu na 1.ª medição desta wave.*
#[test]
fn o_centro_e_a_junta_partilhada() {
    let k = cadeia(2.0, 7.0, 0.9);
    for (w, esperado) in [
        ([0.5, 0.5], Some([2.0, 0.0])),
        ([0.9, 0.1], Some([2.0, 0.0])),
        ([0.01, 0.99], Some([2.0, 0.0])),
    ] {
        let c = k.centro_de_rotacao(&w).expect("ha' par");
        let alvo = esperado.expect("ha' alvo");
        assert!(
            (c[0] - alvo[0]).abs() < 1e-12 && (c[1] - alvo[1]).abs() < 1e-12,
            "com pesos {w:?} o centro saiu {c:?} e nao a junta {alvo:?}"
        );
    }
}

/// ⭐⭐ **UM OSSO SOZINHO NÃO TEM CENTRO — e não precisa.**
///
/// ⚠️ **As duas metades:** `None` quando não há par (a 1.ª), e a lei devolver **o mesmo ponto que a
/// mistura linear** nesse caso (a 2.ª). *É ela que prova que a ausência de centro não é um buraco:
/// uma transformação rígida leva `p` ao mesmo sítio qualquer que seja o centro.*
#[test]
fn um_osso_sozinho_nao_tem_centro_e_nao_precisa() {
    let k = cadeia(2.0, 7.0, 0.9);
    assert_eq!(k.centro_de_rotacao(&[1.0, 0.0]), None);
    assert_eq!(k.centro_de_rotacao(&[0.0, 1.0]), None);
    assert_eq!(k.centro_de_rotacao(&[0.0, 0.0]), None);
    for w in [[1.0, 0.0], [0.0, 1.0]] {
        for p in [[1.0, 0.5], [5.0, -2.0], [0.0, 0.0]] {
            let (a, b) = (k.blend(p, &w), k.blend_linear(p, &w));
            assert!(
                (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
                "com um osso sozinho ({w:?}) a lei rigida deu {a:?} e a linear {b:?}"
            );
        }
    }
}

/// ⭐⭐⭐ **EM REPOUSO A LEI É A IDENTIDADE, AO BIT.**
#[test]
fn em_repouso_a_lei_e_a_identidade() {
    let k = cadeia(2.0, 7.0, 0.0);
    for w in [[1.0, 0.0], [0.5, 0.5], [0.2, 0.8], [0.0, 1.0]] {
        for p in [[1.0, 0.5], [3.0, -0.4], [6.9, 0.2]] {
            let q = k.blend(p, &w);
            assert!(
                (q[0] - p[0]).abs() < 1e-12 && (q[1] - p[1]).abs() < 1e-12,
                "em repouso o ponto {p:?} com pesos {w:?} foi para {q:?}"
            );
        }
    }
}

/// ⭐⭐⭐ **A LEI NÃO ENCOLHE A ARTE — a mistura linear encolhe.**
///
/// A régua é a distância à junta: sob uma rotação rígida ela é **invariante**, e é exactamente isso
/// que a mistura linear perde (ela dá a CORDA do arco, `cos(θ/2)` da distância).
///
/// ⚠️ **O CONTROLO vem primeiro:** sem a mistura linear a encolher de facto, a linha de baixo é
/// trivial.
#[test]
fn a_lei_nao_encolhe_a_arte_e_a_mistura_linear_encolhe() {
    for graus in [60.0_f64, 90.0, 120.0] {
        let k = cadeia(2.0, 7.0, graus.to_radians());
        let c = k.centro_de_rotacao(&[0.5, 0.5]).expect("ha' par");
        let p = [2.0, 1.0]; // a uma unidade da junta, do lado de fora
        let raio = (p[0] - c[0]).hypot(p[1] - c[1]);
        let dist = |q: [f64; 2]| {
            let b = k.blend_linear(c, &[0.5, 0.5]);
            (q[0] - b[0]).hypot(q[1] - b[1])
        };
        let (rig, lin) = (
            dist(k.blend(p, &[0.5, 0.5])),
            dist(k.blend_linear(p, &[0.5, 0.5])),
        );
        let esperado = (graus.to_radians() / 2.0).cos();
        eprintln!(
            "[centro] {graus:>3.0}° · raio {raio:.4} · rigida {rig:.4} · linear {lin:.4} \
             (a corda prevê {:.4})",
            raio * esperado
        );
        // ⭐ O CONTROLO: a mistura linear encolhe, e encolhe o que a CORDA manda.
        assert!(
            (lin - raio * esperado).abs() < 1e-9,
            "a mistura linear nao deu a corda ({lin} contra {})",
            raio * esperado
        );
        // E a lei rígida preserva o raio.
        assert!(
            (rig - raio).abs() < 1e-9,
            "a lei rigida encolheu o raio de {raio} para {rig}"
        );
    }
}

/// ⭐⭐ **A LEI É CONTÍNUA NO PESO** — nenhum peso faz o ponto saltar.
///
/// ⚠️ Ela atravessa o sítio onde o centro **deixa de existir** (`w → (1,0)`), que é onde uma lei
/// escrita com um `if` saltaria.
#[test]
fn a_lei_e_continua_no_peso() {
    let k = cadeia(2.0, 7.0, 1.4);
    let p = [2.0, 1.0];
    const N: usize = 2000;
    let mut ant = k.blend(p, &[1.0, 0.0]);
    let mut pior = 0.0_f64;
    for i in 1..=N {
        #[expect(clippy::cast_precision_loss, reason = "i <= N")]
        let t = i as f64 / N as f64;
        let q = k.blend(p, &[1.0 - t, t]);
        pior = pior.max((q[0] - ant[0]).hypot(q[1] - ant[1]));
        ant = q;
    }
    let total = {
        let (a, b) = (k.blend(p, &[1.0, 0.0]), k.blend(p, &[0.0, 1.0]));
        (a[0] - b[0]).hypot(a[1] - b[1])
    };
    #[expect(clippy::cast_precision_loss, reason = "N")]
    let passo_medio = total / N as f64;
    eprintln!("[centro] pior passo {pior:.6} · passo medio {passo_medio:.6}");
    assert!(
        pior < passo_medio * 4.0,
        "o ponto saltou ao varrer o peso: pior passo {pior} contra {passo_medio} de media"
    );
}

/// Três ossos em linha (juntas em `j1` e `j2`), com os DOIS primeiros girados — logo a 2.ª junta
/// **move-se**.
fn cadeia3(j1: f64, j2: f64, fim: f64, r1: f64, r2: f64) -> Skin {
    let rot = |t: f64, o: [f64; 2]| {
        let (c, s) = (t.cos(), t.sin());
        // roda em torno da ORIGEM do mundo, para a junta de facto andar
        Xform([
            c,
            s,
            -s,
            c,
            c.mul_add(o[0], -(s * o[1])),
            s.mul_add(o[0], c * o[1]),
        ])
    };
    Skin::new(vec![
        SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            j1,
            1.0,
            rot(r1, [0.0, 0.0]),
            Xform::IDENTITY,
        )
        .expect("1"),
        SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, j1, 0.0]),
            j2 - j1,
            1.0,
            rot(r1, [j1, 0.0]),
            Xform::IDENTITY,
        )
        .expect("2"),
        SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, j2, 0.0]),
            fim - j2,
            1.0,
            rot(r2, [j2, 0.0]),
            Xform::IDENTITY,
        )
        .expect("3"),
    ])
    .expect("tres ossos")
}

/// ⭐⭐⭐ **O PAR PESA PELO PRODUTO DOS PESOS — não pela soma.**
///
/// ⛔⛔ **Nasceu de uma mutação SOBREVIVENTE**: com DOIS ossos há um par só, e o peso dele
/// **cancela-se na normalização** — trocar `wᵢ·wⱼ` por `wᵢ+wⱼ` não mudava um bit. *Uma fixtura com
/// um par só não testa como os pares se pesam.*
///
/// ⚠️⚠️ **E a 1.ª redacção deste gate voltou a cair na armadilha da SIMETRIA:** com
/// `w = (0,45 · 0,10 · 0,45)` as duas leis dão **`3,5000`**, porque a fixtura é simétrica. *É a
/// terceira vez nesta jornada que uma fixtura simétrica aprova uma lei que não distingue nada.*
/// ⇒ os pesos são `(0,80 · 0,15 · 0,05)`, e as duas leis separam-se por `0,445`.
#[test]
fn o_par_pesa_pelo_produto_dos_pesos() {
    let k = cadeia3(2.0, 5.0, 7.0, 0.0, 0.0);
    let w = [0.80, 0.15, 0.05];
    let c = k.centro_de_rotacao(&w).expect("ha' pares");
    // ⚠️ **As duas contas estão escritas à mão de propósito** — uma cópia da lei julgaria a lei.
    // As juntas: `(1,2)` em `x = 2` · `(2,3)` em `x = 5` · `(1,3)` no meio das pontas mais
    // próximas de 1 e 3, que são `x = 2` e `x = 5` ⇒ `x = 3,5`.
    let (p12, p23, p13) = (0.80_f64 * 0.15, 0.15_f64 * 0.05, 0.80_f64 * 0.05);
    let produto = p13.mul_add(3.5, p12.mul_add(2.0, p23 * 5.0)) / (p12 + p23 + p13);
    let (s12, s23, s13) = (0.80_f64 + 0.15, 0.15_f64 + 0.05, 0.80_f64 + 0.05);
    let soma = s13.mul_add(3.5, s12.mul_add(2.0, s23 * 5.0)) / (s12 + s23 + s13);
    eprintln!(
        "[centro] x do centro = {:.4} · produto prevê {produto:.4} · soma preveria {soma:.4}",
        c[0]
    );
    assert!(
        (c[0] - produto).abs() < 1e-9,
        "o centro saiu {:.6} e o produto prevê {produto:.6}",
        c[0]
    );
    // ⭐ O CONTROLO: as duas leis TÊM de discordar nesta fixtura, senão o gate é vácuo.
    assert!(
        (produto - soma).abs() > 0.4,
        "as duas leis concordam nesta fixtura ({produto} contra {soma}) — ela nao as distingue"
    );
}

/// ⭐⭐⭐ **A TRANSLAÇÃO É A MISTURA DO CENTRO — e não o centro cru.**
///
/// ⛔⛔ **Também nasceu de uma mutação SOBREVIVENTE**: na fixtura de dois ossos a junta **não se
/// mexe** (os dois ossos rodam à volta dela), logo `blend_linear(junta) == junta` e trocar um pelo
/// outro era um no-op. ⇒ aqui o 1.º osso também roda, e a 2.ª junta **viaja**.
#[test]
fn a_translacao_e_a_mistura_do_centro() {
    let k = cadeia3(2.0, 5.0, 7.0, 0.6, 1.2);
    let w = [0.0, 0.5, 0.5];
    let c = k.centro_de_rotacao(&w).expect("ha' par");
    let base = k.blend_linear(c, &w);
    let viajou = (base[0] - c[0]).hypot(base[1] - c[1]);
    eprintln!("[centro] a junta viajou {viajou:.4} do repouso");
    // ⭐ O CONTROLO: sem a junta a viajar, esta lei e a do centro cru dão o mesmo.
    assert!(
        viajou > 0.5,
        "a junta desta fixtura mal se mexeu ({viajou}) — ela nao distingue as duas leis"
    );
    // O ponto NA junta tem de ir para onde a mistura o leva.
    let q = k.blend(c, &w);
    assert!(
        (q[0] - base[0]).abs() < 1e-9 && (q[1] - base[1]).abs() < 1e-9,
        "o ponto no centro foi para {q:?} e a mistura dele diz {base:?}"
    );
}

/// ⭐⭐⭐ **NUM OSSO QUE DOBRA, A JUNTA É A DO SUB-OSSO — e não a ponta do osso inteiro.**
///
/// ⛔⛔ **Nasceu de uma mutação SOBREVIVENTE**: as fixturas acima só têm ossos RECTOS (`sub = (0,1)`),
/// e ali o eixo do sub-osso **é** o eixo do osso — trocar `lerp(k/n)` por `lerp(0)` não mudava um
/// bit. *Uma fixtura sem osso dobrado não testa a fatia.*
///
/// ⚠️ Os sub-ossos de um osso que dobra partilham `rest_a`/`rest_b`; sem a fatia, dois sub-ossos
/// consecutivos teriam o eixo INTEIRO como junta partilhada, e o centro cairia no princípio do osso
/// em vez de na dobra.
#[test]
fn num_osso_que_dobra_a_junta_e_a_do_sub_osso() {
    let mut ossos = Vec::new();
    SkinBone::bent(
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        crate::bend::BoneSpec {
            length: 8.0,
            strength: 1.0,
            segments: 4,
            curve: crate::bend::Bend {
                inn: [0.0, 1.0],
                out: [0.0, -1.0],
            },
        },
        Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
        Xform::IDENTITY,
        0,
        &mut ossos,
    );
    assert!(
        ossos.len() >= 4,
        "a fixtura devia ter um osso partido em quatro (leu {}) — sem sub-ossos ela nao contem o \
         fenomeno",
        ossos.len()
    );
    let k = Skin::new(ossos).expect("a pele dobrada");
    let n = k.len();
    // Os dois PRIMEIROS sub-ossos: a junta deles é o fim da 1.ª fatia, em `x = 8/4 = 2`.
    let mut w = vec![0.0; n];
    w[0] = 0.5;
    w[1] = 0.5;
    let c = k.centro_de_rotacao(&w).expect("ha' par");
    assert!(
        (c[0] - 2.0).abs() < 1e-9,
        "a junta dos dois primeiros sub-ossos saiu em x = {:.6} e a fatia manda x = 2 — sem a \
         fatia ela cairia em x = 0, o principio do osso inteiro",
        c[0]
    );
    // ⭐ O CONTROLO: um par MAIS À FRENTE tem de dar OUTRA junta, senão a fatia não está a ser lida.
    let mut w2 = vec![0.0; n];
    w2[2] = 0.5;
    w2[3] = 0.5;
    let c2 = k.centro_de_rotacao(&w2).expect("ha' par");
    assert!(
        (c2[0] - 6.0).abs() < 1e-9,
        "a junta do 3.o com o 4.o saiu em x = {:.6} e a fatia manda x = 6",
        c2[0]
    );
}
