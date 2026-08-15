//! **A LEI DE KERNEL que um MODO escolhe** — os gates que perguntam *qual
//! referência este verbo está seguindo*, e não *o que este verbo faz*.
//!
//! Irmão do `verb_tests.rs`, cortado por ASSUNTO: lá cada gate mede o verbo
//! (*o Fill sobe, o Scrape desce, o Pinch aperta*); aqui cada gate mede a
//! **escolha** — a [`crate::KernelLaw`] que o [`crate::RefMode`] manda, com o
//! outro modo servindo de CONTROLE. Sem o controle, os dois lados poderiam ser
//! o mesmo e o chip não escolheria nada.
//!
//! ⚠️ **Eles são o que torna o chip `S` uma afirmação em vez de um rótulo.**
//! Antes desta wave o app dizia s-mode e rodava, em três verbos, uma lei que
//! nenhuma das duas referências tem — a mesma doença que a W0 curou nos
//! DEFAULTS, aqui na LEI. Os números vivem no atlas
//! (`tests/measure_reference_divergence.rs`): `1,717e-3` no Flatten e
//! `5,776e-4` no Pinch, contra um piso de `5,96e-8`.
//!
//! Este módulo é FILHO de `tests`, então `use super::*` alcança as fixtures
//! compartilhadas — duplicá-las seria como os dois arquivos passam a medir
//! malhas diferentes sem ninguém notar.

use super::*;

/// Arrasta a esfera por `pull`, entregue em `steps` eventos de ponteiro.
///
/// ⚠️ **A rampa é sobre o puxão TOTAL** (`pull · k/steps`), não um passo por
/// evento: o Grab é `Grip::Hold`, e a mão que ele modela é uma posição ABSOLUTA
/// do dedo, nunca um incremento.
fn dragged(mode: crate::RefMode, pull: [f32; 3], steps: usize) -> ph2d_mesh::Mesh {
    let mut mesh = sphere();
    let b = Brush {
        verb: Verb::Move,
        mode,
        radius: 0.5,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 1..=steps {
        let t = k as f32 / steps as f32;
        let p = [pull[0] * t, pull[1] * t, pull[2] * t];
        s.dab(
            &mut mesh,
            &b,
            &Dab::pulling([0.0, 0.0, 1.0], b.radius, [0.0, 0.0, -1.0], p),
            Symmetry::default(),
        );
    }
    mesh
}

/// O índice do vértice de repouso mais próximo de `p`.
fn nearest(rest: &ph2d_mesh::Mesh, p: [f32; 3]) -> usize {
    (0..rest.vert_count())
        .min_by(|&x, &y| {
            let d = |k: usize| {
                let q = rest.positions()[k];
                (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)
            };
            d(x).total_cmp(&d(y))
        })
        .expect("a esfera tem vértices")
}

/// **O PESO É UM FATO SOBRE O `pre` CONGELADO — o mesmo puxão entregue em 1 e em
/// 12 eventos tem de dar a MESMA malha.**
///
/// ⚠️ **Isto não é uma tolerância, é uma IDENTIDADE**, e vale a pena saber por
/// quê: o Grab é [`crate::Grip::Hold`], que recomputa `accum = w` do zero a cada
/// dab (`unit_accum = false` sem campo) e escreve o alvo a partir das posições
/// CONGELADAS no pen-down. Logo o dab final — o único que importa — escreve
/// `base + f(pull)` nos dois casos, bit a bit. Qualquer termo de `w` que leia
/// estado VIVO quebra isto, porque o estado vivo é justamente a deformação que
/// os dabs anteriores produziram: realimentação.
///
/// ⚠️ **Era exatamente o que o `b-mode` fazia**, e o número é o mecanismo: com
/// a normal VIVA no `FrontFace::Continuous` a divergência mede **0,013280 ·
/// 0,535897 · 0,867680** nos puxões 0,2 / 0,6 / 0,9 — no maior, quase o
/// tamanho do próprio gesto. Com a normal do `pre`, **0,000000** nos três.
///
/// ⚠️ **O `s-mode` é o CONTROLE e não é redundante:** ele não tem termo de
/// facing nenhum, então já era invariante — se ele estivesse vermelho o defeito
/// seria de outra coisa (do aplicador, do `Hold`, da fixture), e a leitura deste
/// gate mudaria de dono.
#[test]
fn the_weight_is_a_fact_about_the_frozen_surface() {
    let rest = sphere();
    for pull in [0.2f32, 0.6, 0.9] {
        let p = [pull, 0.0, 0.0];
        let mut worst = [0.0f32; 3];
        for (i, mode) in [crate::RefMode::S, crate::RefMode::B, crate::RefMode::L]
            .into_iter()
            .enumerate()
        {
            let one = dragged(mode, p, 1);
            let many = dragged(mode, p, 12);
            worst[i] = (0..rest.vert_count())
                .map(|v| {
                    let (a, b) = (one.positions()[v], many.positions()[v]);
                    (0..3).map(|k| (a[k] - b[k]).abs()).fold(0.0f32, f32::max)
                })
                .fold(0.0f32, f32::max);
        }
        // ⚠️ **O anti-vácuo:** um gesto que não move barro é invariante por
        // acidente, e os três `worst` sairiam zero sem nada ser provado.
        let moved = {
            let m = dragged(crate::RefMode::B, p, 12);
            let t = nearest(&rest, [0.0, 0.0, 1.0]);
            let (a, b) = (rest.positions()[t], m.positions()[t]);
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        };
        assert!(moved > pull * 0.5, "a fixture não contém o gesto ({moved})");
        for (i, name) in ["s", "b", "l"].iter().enumerate() {
            assert!(
                worst[i] < 1e-5,
                "puxão {pull}: o {name}-mode divergiu {} entre 1 e 12 eventos — \
                 algum termo do peso está a ler estado VIVO",
                worst[i]
            );
        }
    }
}

/// **O BARRO AGARRADO NUNCA LARGA O DEDO — em NENHUM evento do arrasto.**
///
/// ⚠️ **Afirmação distinta da do gate acima, e o par é deliberado:** aquele
/// julga a LEI (o resultado não pode depender de quantos eventos o rato mandou);
/// este julga o que o ARTISTA VÊ *durante* o gesto. O defeito reportado — *"os
/// modos B e L do grab estão bizarros"* — era visível como o barro a escorregar
/// de volta no meio do arrasto, e uma lei correta medida só no FIM não teria
/// nada a dizer sobre isso.
///
/// ⚠️ **O bico é onde o `b-mode` se auto-destruía:** ali a normal congelada é
/// `+Z`, o olho é `−Z`, e `max(−(n·eye), 0)` vale exatamente **1** — o peso
/// cheio. Com a normal VIVA, o próprio deslocamento gira a superfície para fora
/// do olho e o fator desaba: medido, **0,9956 → 0,1418** do puxão no 9º de 12
/// eventos, com salto de volta no último. Hoje, `1,0000` em todos.
#[test]
fn the_grabbed_clay_never_lets_go_of_the_finger() {
    let rest = sphere();
    let tip = nearest(&rest, [0.0, 0.0, 1.0]);
    let pull = [0.6f32, 0.0, 0.0];
    const STEPS: usize = 12;

    for mode in [crate::RefMode::S, crate::RefMode::B, crate::RefMode::L] {
        let mut mesh = sphere();
        let b = Brush {
            verb: Verb::Move,
            mode,
            radius: 0.5,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        let mut worst = f32::MAX;
        for k in 1..=STEPS {
            let t = k as f32 / STEPS as f32;
            let want = [pull[0] * t, pull[1] * t, pull[2] * t];
            s.dab(
                &mut mesh,
                &b,
                &Dab::pulling([0.0, 0.0, 1.0], b.radius, [0.0, 0.0, -1.0], want),
                Symmetry::default(),
            );
            let (a, q) = (rest.positions()[tip], mesh.positions()[tip]);
            let got =
                ((q[0] - a[0]).powi(2) + (q[1] - a[1]).powi(2) + (q[2] - a[2]).powi(2)).sqrt();
            let asked = (want[0] * want[0] + want[1] * want[1] + want[2] * want[2]).sqrt();
            worst = worst.min(got / asked);
        }
        assert!(
            worst > 0.98,
            "{mode:?}: no pior evento do arrasto o bico entregou {worst:.4} do \
             que o dedo pediu — o barro escorregou sob a mão"
        );
    }
}

/// **EM `S`, O FLATTEN É O SCRAPE — ao vértice.**
///
/// Não é uma aproximação nem um piso escolhido: o `Flatten.js:11` nasce com
/// `_negative = true`, o `:57` faz `comp = -1` e o `:64` faz
/// `if (distToPlane * comp > 0.0) continue` — ou seja, o *Flatten* da
/// referência **é** o nosso `Scrape`, e o outro lado dela é o nosso `Fill`.
/// Quem quiser os dois lados escolhe o modo `B`, que é a leitura do `plane.cc`.
///
/// ⚠️ **Este gate é o que torna o chip `S` uma afirmação em vez de um rótulo.**
/// Antes desta wave o app dizia s-mode e rodava um Flatten que nenhuma das duas
/// referências tem — a mesma doença que a W0 curou nos DEFAULTS, aqui na LEI.
#[test]
fn in_s_mode_the_flatten_is_the_scrape_vertex_by_vertex() {
    let mut bumpy = sphere();
    let poke = Brush {
        verb: Verb::Draw,
        radius: 0.15,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&bumpy);
    s.dab(
        &mut bumpy,
        &poke,
        &dab_at([0.0, 0.0, 1.0], poke.radius),
        Symmetry::default(),
    );

    let c = [0.1, 0.0, 1.0];
    let run = |verb: Verb, mode: crate::RefMode| {
        let mut mesh = bumpy.clone();
        let b = Brush {
            verb,
            radius: 0.5,
            strength: 1.0,
            falloff: Falloff::Constant,
            mode,
            ..Brush::default()
        };
        let mut st = SculptStroke::default();
        st.begin(&mesh);
        st.dab(&mut mesh, &b, &dab_at(c, b.radius), Symmetry::default());
        snapshot(&mesh)
    };
    let flat_s = run(Verb::Flatten, crate::RefMode::S);
    let scrape = run(Verb::Scrape, crate::RefMode::S);
    let flat_b = run(Verb::Flatten, crate::RefMode::B);

    let gap = |a: &[[f32; 3]], b: &[[f32; 3]]| {
        a.iter()
            .zip(b)
            .map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max)
    };
    assert_eq!(
        gap(&flat_s, &scrape),
        0.0,
        "em `S` o Flatten tem de ser o Scrape AO VÉRTICE"
    );
    // ⚠️ **O CONTROLE, e sem ele o gate passaria com os dois modos em `S`:** em
    // `B` o mesmo dab tem de divergir do Scrape, senão o chip não escolhe nada.
    let b_gap = gap(&flat_b, &scrape);
    assert!(
        b_gap > 1e-4,
        "o modo `B` tem de morder o outro lado também ({b_gap})"
    );
}

/// **EM `S`, O PINCH PUXA EM 3D — e o deslocamento aponta para o CENTRO.**
///
/// O `Pinch.js:52-58` soma `(centro − v) · f · deform` cru, sem projetar. O
/// oráculo aqui não é *"tem componente normal"* (isso é fraco: qualquer erro de
/// sinal também teria) — é **a DIREÇÃO**: cada vértice deslocado tem de andar ao
/// longo da reta que o liga ao centro do dab, ao cosseno.
///
/// ⚠️ **E o gate mede o par, não o absoluto:** o mesmo traço em `B` tem de
/// deixar a direção do traço QUIETA. Sem essa metade ele passaria com os dois
/// modos em `S`, e o chip não escolheria nada.
///
/// ⚠️ **Ele passou a dirigir um TRAÇO e não um dab, e não foi conveniência:** o
/// `B` do Pinch é o `pinch.cc`, cuja lei precisa da direção do gesto — sem ela a
/// referência **recusa o dab** (`pinch.cc:188-195`) e nós também. Um dab solto
/// aqui media o `B` no degenerado dele e afirmava a lei errada sobre um conjunto
/// vazio.
#[test]
fn in_s_mode_the_pinch_pulls_straight_at_the_centre() {
    let c = [0.0, 0.0, 1.0];
    let run = |mode: crate::RefMode| {
        let mut mesh = sphere();
        let b = Brush {
            verb: Verb::Pinch,
            radius: 0.5,
            strength: 1.0,
            mode,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        // Dois dabs ao longo de `+x`: o segundo é o que tem direção, e é ele que
        // o oráculo lê. O primeiro existe para o `SculptStroke` a poder derivar.
        s.dab(
            &mut mesh,
            &b,
            &dab_at([-0.06, 0.0, 1.0], b.radius),
            Symmetry::default(),
        );
        let base = snapshot(&mesh);
        s.dab(&mut mesh, &b, &dab_at(c, b.radius), Symmetry::default());
        // O pior cosseno contra a direção `v -> centro`, e o pior |normal|/|d|.
        let (mut worst_cos, mut worst_normal, mut moved) = (1.0f32, 0.0f32, 0usize);
        for (p, q) in base.iter().zip(mesh.positions()) {
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if len < 1e-5 {
                continue;
            }
            moved += 1;
            let to_c = [c[0] - p[0], c[1] - p[1], c[2] - p[2]];
            let cl = (to_c[0] * to_c[0] + to_c[1] * to_c[1] + to_c[2] * to_c[2]).sqrt();
            let cos = (d[0] * to_c[0] + d[1] * to_c[1] + d[2] * to_c[2]) / (len * cl);
            worst_cos = worst_cos.min(cos);
            // A normal da área no polo é ~+Z.
            worst_normal = worst_normal.max(d[2].abs() / len);
        }
        (worst_cos, worst_normal, moved)
    };

    let (cos_s, _normal_s, moved) = run(crate::RefMode::S);
    assert!(
        cos_s > 0.9999,
        "em `S` todo vértice anda RETO para o centro (pior cosseno {cos_s})"
    );
    // O CONTROLE: em `B` a lei do `pinch.cc` remove a componente ao longo do
    // traço, e é isso que faz o mesmo dab andar para OUTRO lugar.
    let (cos_b, _normal_b, moved_b) = run(crate::RefMode::B);
    // ⚠️ **O `assert_eq!(moved, moved_b)` que estava aqui era EXATO e passou a
    // ser FALSO, e o que mudou foi a lei.** Ele dizia *"a pegada não depende do
    // modo"*, verdade enquanto as duas leis eram projeções que nunca zeram o
    // vetor; o `AcrossStroke` zera **por construção** o vértice que cai sobre a
    // LINHA do traço, porque ali o deslocamento ao centro é inteiramente a
    // componente que a lei remove. É o *"pinched towards a line"* do comentário
    // da referência, medido em **61 contra 60** vértices — um.
    //
    // ⇒ A metade que ele de facto protegia — *um conjunto vazio faz os `worst_*`
    // devolverem a identidade e o gate passa sem medir nada* — fica, e é o que
    // as duas linhas abaixo afirmam. Uma propriedade que deixou de ser verdade
    // não se afrouxa com um limiar: ela sai, e a que sobra é dita inteira.
    assert!(moved > 0, "a fixture não contém o fenômeno em `S`");
    assert!(moved_b > 0, "a fixture não contém o fenômeno em `B`");
    assert!(
        cos_b < 0.9999,
        "em `B` o puxão NÃO é reto para o centro (cosseno {cos_b})"
    );
}

/// **O PINCH EM `B` APERTA ATRAVÉS DO TRAÇO E DEIXA A LINHA QUIETA** — o
/// `pinch.cc:39-60`, e a lei que fecha o report do Enio de 2026-08-15 (*"Pinch em
/// B e S bons mas idênticos ou quase idênticos"*).
///
/// ⚠️ **O gate afirmava o CONTRÁRIO até esta wave, e o comentário dele carregava
/// o erro que a nota do [`crate::LateralPull::Tangential`] agora nomeia:** ele
/// dizia que o `pinch.cc` projeta *"a tangente ao longo do TRAÇO mais a
/// NORMAL"* e que *"nenhum dos dois projeta como nós"*. As duas frases saíram do
/// COMENTÁRIO do Blender (*"the X vector (aligned to the stroke)"*), que é falso
/// no próprio Blender — o código monta `X = cross(area_no, grab_delta)`, que é
/// **perpendicular** ao traço. Lida a fonte em vez do comentário: o `crease.cc`
/// projeta **exatamente** como nós, e o `pinch.cc` remove a componente **ao
/// longo** do traço.
///
/// ⚠️ **E o *"does not secretly flatten"* do nome antigo era uma afirmação sobre
/// a lei antiga.** O `pinch.cc` **GUARDA** a componente normal (`z_disp`) de
/// propósito, então o Pinch em `B` passa a ter uma — é a mudança de
/// comportamento desta wave, e ela vem da referência. Quem quer o aperto puro no
/// plano tem o `S` ao lado no mesmo verbo.
#[test]
fn the_b_pinch_squeezes_across_the_stroke_and_leaves_the_line_alone() {
    let mut mesh = sphere();
    let c = [0.0, 0.0, 1.0];
    let b = Brush {
        verb: Verb::Pinch,
        radius: 0.5,
        strength: 1.0,
        mode: crate::RefMode::B,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    // ⚠️ **DOIS dabs ao longo de `+x`, e o primeiro não é decoração:** a lei do
    // `B` precisa da direção do gesto, e o [`crate::Dab::path`] nasce em zero —
    // a referência **recusa** o primeiro dab de cada passe (`pinch.cc:188-195`)
    // e nós recusamos junto. Um dab solto media esta lei no degenerado dela.
    s.dab(
        &mut mesh,
        &b,
        &dab_at([-0.06, 0.0, 1.0], b.radius),
        Symmetry::default(),
    );
    let base = snapshot(&mesh);
    s.dab(&mut mesh, &b, &dab_at(c, b.radius), Symmetry::default());

    // ⚠️ **O oráculo é a razão ATRAVÉS ÷ AO LONGO, e ela é adimensional** — um
    // aperto que some é indistinguível de um aperto certo se o gate só medir
    // magnitude, e um traço em `+x` faz de `y` o eixo *através* e de `x` o eixo
    // *ao longo*. A lei do `pinch.cc` deixa o `x` quieto e é isso que se afirma.
    let (mut across, mut along) = (0.0f32, 0.0f32);
    for (p, q) in base.iter().zip(mesh.positions()) {
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-5 {
            continue;
        }
        across = across.max(d[1].abs());
        along = along.max(d[0].abs());
    }
    let lateral = across;
    // ⚠️ **O piso é DERIVADO do ganho, e não escolhido** — ele é anti-vácuo, e
    // um literal aqui envelhece com a constante. O maior deslocamento possível
    // num dab é `ganho · força · raio` (peso 1 na distância cheia, que o
    // falloff nunca entrega junto); medido, o produto pousa em 28 % disso.
    //
    // ⚠️ **Ele era `0.02` e o número mudou porque o PRODUTO mudou:** o
    // [`crate::PINCH_GAIN`] não existia, o alvo era a tangente inteira atenuada
    // pelo peso, e o verbo era **20× mais forte que a referência**. O piso
    // antigo estava calibrado nesse erro.
    let floor = crate::PINCH_GAIN * b.strength * b.radius * 0.2;
    assert!(
        lateral > floor,
        "o Pinch não apertou nada ({lateral} contra o piso {floor})"
    );
    assert!(
        along < across * 0.05,
        "o Pinch em `B` moveu {along:.6} AO LONGO do traço contra {across:.6} \
         através dele — a componente que o `pinch.cc` remove voltou, e com ela \
         o aperto radial que torna o `B` indistinguível do `S`"
    );
}

/// **SEM DIREÇÃO NÃO HÁ APERTO NO `B`** — a recusa que o `pinch.cc:188-195` faz
/// (*"delay the first daub because grab delta is not setup"*, e `return` com
/// `grab_delta` zero), e a metade que impede alguém de "consertar" o degenerado
/// inventando um eixo.
///
/// ⚠️ **O `S` é o CONTROLE, e ele não é um espantalho:** o MESMO dab solto, no
/// mesmo lugar, aperta em `S` — então o que este gate mede é a LEI, não uma
/// fixture que não toca a malha.
#[test]
fn without_a_stroke_direction_the_b_pinch_refuses_and_the_s_pinch_does_not() {
    let squeeze = |mode: crate::RefMode| {
        let mut mesh = sphere();
        let base = snapshot(&mesh);
        let b = Brush {
            verb: Verb::Pinch,
            radius: 0.5,
            strength: 1.0,
            mode,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        // UM dab: o `Dab::path` nasce em zero e ninguém o preencheu.
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.0, 0.0, 1.0], b.radius),
            Symmetry::default(),
        );
        let mut worst = 0.0f32;
        for (p, q) in base.iter().zip(mesh.positions()) {
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            worst = worst.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
        }
        worst
    };
    let s_only = squeeze(crate::RefMode::S);
    assert!(
        s_only > 1e-4,
        "o CONTROLE não aperta ({s_only}) — a fixture não contém o fenômeno"
    );
    assert_eq!(
        squeeze(crate::RefMode::B),
        0.0,
        "o `B` apertou sem saber para que lado o traço vai — um eixo inventado \
         é uma direção que o artista não desenhou"
    );
}

/// **O EIXO DO TRAÇO É ORTOGONALIZADO CONTRA A NORMAL** — o `cross` duas vezes
/// do `pinch.cc:199-200`, e não uma normalização do gesto cru.
///
/// ⚠️ **A diferença só aparece num gesto que MERGULHA**, e um traço sobre uma
/// esfera mergulha um pouco por curvatura. Sem a ortogonalização o eixo carrega
/// parte da normal, e a lei passaria a remover parte do `z_disp` que o
/// `pinch.cc` **guarda** de propósito — um erro que nenhum traço raso denuncia.
#[test]
fn the_stroke_axis_is_orthogonalised_against_the_normal() {
    let n = [0.0, 0.0, 1.0];
    let flat = super::super::target::stroke_axis(n, [1.0, 0.0, 0.0]).expect("traço tangencial");
    // O MESMO gesto com meio metro de mergulho: o eixo tem de ser o mesmo.
    let diving = super::super::target::stroke_axis(n, [1.0, 0.0, -0.5]).expect("traço a mergulhar");
    for k in 0..3 {
        assert!(
            (flat[k] - diving[k]).abs() < 1e-6,
            "o eixo do traço mudou por o gesto mergulhar: {flat:?} contra {diving:?}"
        );
    }
    // E os dois degenerados devolvem a MESMA resposta, que é a ausência dela.
    assert_eq!(super::super::target::stroke_axis(n, [0.0; 3]), None);
    assert_eq!(super::super::target::stroke_axis(n, [0.0, 0.0, 2.0]), None);
}
