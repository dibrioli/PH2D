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

// ─────────────────────── A LEI DO ÂNGULO DESDOBRADO (2026-09-20) ───────────────────────

use crate::MisturaDoAngulo;

/// Três ossos em linha com **um ângulo POR OSSO** — a `cadeia3` dá o mesmo aos dois primeiros, e
/// uma cadeia assim não distingue um desdobramento que salta um elo.
fn cadeia3_angulos(j1: f64, j2: f64, fim: f64, t: [f64; 3]) -> Skin {
    let rot = |a: f64, o: [f64; 2]| {
        let (c, s) = (a.cos(), a.sin());
        Xform([
            c,
            s,
            -s,
            c,
            c.mul_add(o[0], -(s * o[1])),
            s.mul_add(o[0], c * o[1]),
        ])
    };
    let seg = |a: f64, l: f64, ang: f64| {
        SkinBone::new(
            Xform([1.0, 0.0, 0.0, 1.0, a, 0.0]),
            l,
            1.0,
            rot(ang, [a, 0.0]),
            Xform::IDENTITY,
        )
        .expect("osso")
    };
    Skin::new(vec![
        seg(0.0, j1, t[0]),
        seg(j1, j2 - j1, t[1]),
        seg(j2, fim - j2, t[2]),
    ])
    .expect("tres ossos")
}

/// ⭐⭐⭐ **A OMISSÃO É BYTE-IDÊNTICA — a lei nova shipa DESLIGADA.**
///
/// ⚠️ **As três metades, e nenhuma basta:** a pele que a [`Skin::new`] devolve escolhe o CÍRCULO ·
/// o [`Skin::blend`] dela é **bit a bit** o [`Skin::blend_com`] com o círculo · e a lei
/// **desdobrada existe e dá OUTRA coisa** nesta mesma fixtura. *Sem a terceira, uma implementação
/// que ignorasse o parâmetro passaria as duas primeiras.*
#[test]
fn a_omissao_e_o_circulo_e_e_byte_identica() {
    let k = cadeia3(2.0, 5.0, 7.0, 2.4, 4.8);
    assert_eq!(k.mistura(), MisturaDoAngulo::Circulo);
    let w = [0.5, 0.3, 0.2];
    let mut mexeu = false;
    for x in [[1.0, 0.7], [3.0, -0.4], [6.5, 0.2], [0.0, 0.0]] {
        let a = k.blend(x, &w);
        let b = k.blend_com(x, &w, MisturaDoAngulo::Circulo);
        assert_eq!(
            a.to_bits_pair(),
            b.to_bits_pair(),
            "em {x:?} a omissão não é o círculo"
        );
        let d = k.blend_com(x, &w, MisturaDoAngulo::Desdobrado);
        mexeu |= (d[0] - a[0]).hypot(d[1] - a[1]) > 1e-9;
    }
    assert!(
        mexeu,
        "a lei desdobrada devolveu o MESMO que a de círculo em toda a fixtura — \
         ou ela não está ligada, ou esta fixtura não a distingue"
    );
}

/// Um par de `f64` comparado ao BIT — `0.0 == -0.0` e `NaN != NaN` não servem a uma promessa de
/// identidade.
trait ParDeBits {
    fn to_bits_pair(self) -> (u64, u64);
}
impl ParDeBits for [f64; 2] {
    fn to_bits_pair(self) -> (u64, u64) {
        (self[0].to_bits(), self[1].to_bits())
    }
}

/// ⭐⭐⭐ **A LEI DESDOBRADA É A MÉDIA LINEAR DOS ÂNGULOS DA CADEIA, em forma fechada.**
///
/// Numa cadeia de três ossos a `0`, `r₁` e `r₂`, com pesos `w`, a rotação que a lei aplica tem de
/// ser exactamente `Σ wᵢ θ̃ᵢ` — e aqui `r₁`/`r₂` são pequenos, logo o desdobramento é a identidade
/// e o número calcula-se à mão sem ambiguidade nenhuma.
#[test]
fn a_lei_desdobrada_e_a_media_linear_dos_angulos() {
    let (r1, r2) = (0.4_f64, 0.9_f64);
    let k = cadeia3(2.0, 5.0, 7.0, r1, r2);
    let w = [0.5, 0.3, 0.2];
    // A cadeia3 põe o ângulo `r1` nos DOIS primeiros ossos e `r2` no terceiro.
    let alvo = 0.5f64.mul_add(r1, 0.3 * r1) + 0.2 * r2;
    let c = k.centro_de_rotacao(&w).expect("ha' pares");
    let o = k.blend_com(c, &w, MisturaDoAngulo::Desdobrado);
    let u = k.blend_com([c[0] + 1.0, c[1]], &w, MisturaDoAngulo::Desdobrado);
    let medido = (u[1] - o[1]).atan2(u[0] - o[0]);
    assert!(
        (medido - alvo).abs() < 1e-12,
        "a lei aplicou {medido} e a média linear dos ângulos é {alvo}"
    );
    // ⚠️ O CONTROLO: a média em CÍRCULO dá OUTRO número na mesma fixtura, senão este gate não
    // distingue as duas leis.
    let oc = k.blend_com(c, &w, MisturaDoAngulo::Circulo);
    let uc = k.blend_com([c[0] + 1.0, c[1]], &w, MisturaDoAngulo::Circulo);
    let circ = (uc[1] - oc[1]).atan2(uc[0] - oc[0]);
    assert!(
        (circ - alvo).abs() > 1e-6,
        "a média em círculo deu {circ}, que é a linear — a fixtura não separa as duas leis"
    );
}

/// ⭐⭐⭐ **UM OSSO DE PESO ZERO NO MEIO DA CADEIA NÃO PARTE O DESDOBRAMENTO.**
///
/// ⛔⛔⛔ **Esta é a lei inteira da implementação, e ela cabe na ORDEM de duas linhas:** desdobrar é
/// uma propriedade da **CADEIA** e não do ponto. O `continue` do peso zero tem de vir **DEPOIS** de
/// a referência avançar; posto antes, o ângulo do osso seguinte passa a depender de **quais** ossos
/// aquele ponto por acaso reclama.
///
/// A fixtura tem os três ossos a `0`, `0,7π` e `1,4π`: **o do meio é a PONTE** que leva o terceiro
/// à volta certa. Saltá-lo faz o desdobramento medir `1,4π` (que vem enrolado como `−0,6π`) contra
/// `0` em vez de contra `0,7π`, e ele aterra **uma volta inteira** para o outro lado.
///
/// ⚠️⚠️ **Dois pontos VIZINHOS com pesos diferentes leriam então rotações a `2π` de distância, e a
/// arte RASGAVA na fronteira entre eles** — é isso que este gate impede, e é a 2.ª metade
/// (a continuidade) que o afirma.
///
/// ⛔⛔ **A 1.ª redacção deste gate SOBREVIVEU à mutação**, e a causa foi a fixtura: a [`cadeia3`]
/// dá o **mesmo** ângulo aos dois primeiros ossos, logo saltar o do meio não movia a referência e
/// as duas ordens davam o mesmo número. *Uma cadeia em que o elo saltado não muda nada não testa
/// um desdobramento que salta elos.*
#[test]
fn um_osso_de_peso_zero_nao_parte_a_cadeia_do_desdobramento() {
    use std::f64::consts::PI;
    let k = cadeia3_angulos(2.0, 5.0, 7.0, [0.0, 0.7 * PI, 1.4 * PI]);
    // ⚠️ O CONTROLO da fixtura: o 3.º osso chega enrolado, senão não há volta a escolher.
    let cru = k.bones()[2].angulo_da_pose();
    assert!(
        cru < 0.0,
        "o 3.º osso devia chegar ENROLADO (o atan2 vive em (−π, π]) e leu {cru}"
    );
    let c = k.centro_de_rotacao(&[0.5, 0.0, 0.5]).expect("ha' par");
    let angulo = |w: &[f64; 3]| {
        let o = k.blend_com(c, w, MisturaDoAngulo::Desdobrado);
        let u = k.blend_com([c[0] + 1.0, c[1]], w, MisturaDoAngulo::Desdobrado);
        (u[1] - o[1]).atan2(u[0] - o[0])
    };
    // Com a cadeia honrada: 0,5·0 + 0,5·1,4π = 0,7π ⇒ enrolado, 0,7π.
    let esperado = 0.7 * PI;
    let medido = angulo(&[0.5, 0.0, 0.5]);
    assert!(
        (medido - esperado).abs() < 1e-12,
        "com o osso do meio a peso ZERO a lei aplicou {medido} e a cadeia manda {esperado}"
    );
    // ⚠️ E a CONTINUIDADE, que é o que o defeito quebrava: dar um fio de peso ao osso do meio não
    // pode saltar a resposta.
    let e = 1e-6;
    let vizinho = angulo(&[0.5 - e * 0.5, e, 0.5 - e * 0.5]);
    let mut salto = vizinho - medido;
    while salto > PI {
        salto -= 2.0 * PI;
    }
    while salto < -PI {
        salto += 2.0 * PI;
    }
    assert!(
        salto.abs() < 1e-4,
        "um fio de peso no osso do meio saltou a rotação em {salto} rad — a cadeia partiu-se"
    );
}

/// ⭐⭐⭐ **A MEIA VOLTA: o guarda da degenerescência da lei de CÍRCULO é CÓDIGO MORTO.**
///
/// ⛔⛔⛔ **O doc do [`Skin::blend`] prometia** que com duas rotações a `180°` exactas e pesos
/// iguais *«a soma é zero e a lei cai na mistura linear»*. **Ela não cai.** Medido: com os ossos a
/// `0` e a `π`, `Σ w cos` é `0` **exacto** e `Σ w sin` é **`+6,123234e-17`** — porque `sin(π)` em
/// `f64` não é zero —, logo a guarda `sx == 0 && sy == 0` **nunca arma** e o `atan2` devolve
/// **`+90°`**: uma rotação inteira tirada do resíduo de um arredondamento.
///
/// ⚠️ *Uma promessa de fallback num doc-comment é o pior sítio para uma guarda morta: quem a lê
/// deixa de procurar o caso.* ⇒ este gate afirma o que **acontece**, e a prosa foi corrigida.
///
/// ⭐ A lei DESDOBRADA chega ao mesmo `+90°`, e a diferença é toda: ali ele é a média linear
/// `0,5·0 + 0,5·π`, **por construção**, e não o sinal de um último bit.
#[test]
fn na_meia_volta_o_guarda_do_circulo_nao_arma() {
    use std::f64::consts::PI;
    let k = cadeia(2.0, 7.0, PI);
    let w = [0.5, 0.5];
    let c = k.centro_de_rotacao(&w).expect("ha' par");
    let base = k.blend_linear(c, &w);
    let angulo = |lei: MisturaDoAngulo| {
        let u = k.blend_com([c[0] + 1.0, c[1]], &w, lei);
        (u[1] - base[1]).atan2(u[0] - base[0])
    };

    // (1) A guarda NÃO arma — o resíduo de `sin(π)` mantém o vector não-nulo.
    let (sx, sy) = (0.5f64.mul_add(1.0, 0.5 * PI.cos()), 0.5 * PI.sin());
    assert_eq!(sx, 0.0, "a soma dos cossenos devia ser zero exacto");
    assert!(
        sy != 0.0 && sy.abs() < 1e-15,
        "a soma dos senos é o resíduo de sin(π) e vale {sy:e} — é ele que mata a guarda"
    );

    // (2) ⇒ a lei de círculo NÃO cai na mistura linear: ela roda 90°.
    let p = [5.0, 1.0];
    let circulo = k.blend_com(p, &w, MisturaDoAngulo::Circulo);
    let linear = k.blend_linear(p, &w);
    assert!(
        (circulo[0] - linear[0]).hypot(circulo[1] - linear[1]) > 1e-3,
        "a média em círculo caiu na mistura linear — a guarda que o doc prometia armou,          e esta fixtura deixou de descrever o defeito que ela documenta"
    );
    assert!(
        (angulo(MisturaDoAngulo::Circulo) - PI * 0.5).abs() < 1e-9,
        "o círculo devia aplicar os +90° que o resíduo produz"
    );

    // (3) A desdobrada chega ao mesmo número, e POR CONSTRUÇÃO — é a média linear `0,5·0 + 0,5·π`.
    assert!(
        (angulo(MisturaDoAngulo::Desdobrado) - PI * 0.5).abs() < 1e-12,
        "a lei desdobrada devia aplicar a média LINEAR dos dois ângulos"
    );

    // (4) E ela é RÍGIDA: a distância ao centro conserva-se, que é o que a mistura linear perde.
    let desd = k.blend_com(p, &w, MisturaDoAngulo::Desdobrado);
    let antes = (p[0] - c[0]).hypot(p[1] - c[1]);
    let depois = (desd[0] - base[0]).hypot(desd[1] - base[1]);
    assert!(
        (depois - antes).abs() < 1e-12,
        "a lei desdobrada encolheu o raio de {antes} para {depois}"
    );
    let dlin = (linear[0] - base[0]).hypot(linear[1] - base[1]);
    assert!(
        (dlin - antes).abs() > 1e-3,
        "a mistura linear devia ENCOLHER o raio — é o controlo desta metade"
    );
}
