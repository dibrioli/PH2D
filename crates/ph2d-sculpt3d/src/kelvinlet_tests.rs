//! Os gates do [`super`] — e **todos afirmam PROPRIEDADES do campo**, não a
//! forma como ele está escrito.
//!
//! ⚠️ **Um kernel de elasticidade é raro entre os gates deste módulo por não
//! precisar de oráculo externo:** ele tem identidades que o definem (o bico
//! segue o dedo · o twist não pergunta o material · o campo distante cancela ·
//! o gradiente na origem É a matriz), e cada uma delas é falsa sob quase toda
//! reescrita errada. É o oposto do porte do SculptGL, onde o oráculo é o JS
//! executando.

use super::*;

const EPS: f32 = 0.5;

fn len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// **O bico segue o dedo, EXATAMENTE** — é o que a normalização compra, e é o
/// que faz o chip `S ↔ L` trocar a forma da vizinhança sem mover o vértice que
/// o artista agarrou.
#[test]
fn the_tip_follows_the_finger_exactly() {
    let f = [0.3, -0.7, 0.2];
    for s in [Scales::Mono, Scales::Bi, Scales::Tri] {
        let u = grab([0.0, 0.0, 0.0], EPS, f, s);
        for k in 0..3 {
            assert!(
                (u[k] - f[k]).abs() < 1e-6,
                "{s:?}: o bico entrega {u:?} onde o dedo pediu {f:?}"
            );
        }
    }
}

/// **O AGARRE É ANISOTRÓPICO — é a wave inteira numa asserção.**
///
/// À mesma distância do centro, o barro **à frente** do puxão anda mais que o
/// barro **ao lado** dele. Nenhuma curva de falloff pode exprimir isto: ela
/// devolve um escalar, e um escalar não tem para onde apontar.
#[test]
fn the_grab_pulls_more_ahead_than_beside() {
    let f = [1.0, 0.0, 0.0];
    let d = EPS;
    let ahead = grab([d, 0.0, 0.0], EPS, f, Scales::Tri);
    let beside = grab([0.0, d, 0.0], EPS, f, Scales::Tri);
    let ratio = len(ahead) / len(beside);
    assert!(
        ratio > 1.2,
        "o campo saiu isotrópico ({ratio:.4}×) — uma curva de falloff faria isto"
    );
    // ⚠️ **O CONTROLE**: a lei do `s-mode` é `gesto · escalar`, e a razão dela é
    // exatamente 1 por construção. Sem esta metade o gate acima poderia estar a
    // medir qualquer coisa.
    let s_ahead = [f[0] * 0.5, 0.0, 0.0];
    let s_beside = [f[0] * 0.5, 0.0, 0.0];
    assert!((len(s_ahead) / len(s_beside) - 1.0).abs() < 1e-9);
}

/// **O TWIST NÃO PERGUNTA DE QUE MATERIAL O BARRO É** — o `b` cancela, e o
/// oráculo é a forma fechada derivada à parte.
#[test]
fn the_twist_has_a_closed_form_without_the_material() {
    let omega = [0.0, 0.0, 1.0];
    for r in [[0.3, 0.1, -0.2], [1.0, -0.4, 0.7], [0.05, 0.0, 0.0]] {
        let f: [[f32; 3]; 3] = [
            [0.0, -omega[2], omega[1]],
            [omega[2], 0.0, -omega[0]],
            [-omega[1], omega[0], 0.0],
        ];
        let got = raw_affine(r, EPS, &f);
        // `[ 1/rε³ + 3ε²/(2 rε⁵) ]·(ω × r)`, sem `b` em lado nenhum.
        let re = r_eps(r, EPS);
        let re3 = re * re * re;
        let re5 = re3 * re * re;
        let k = 1.0 / re3 + 1.5 * EPS * EPS / re5;
        let cross = [
            omega[1] * r[2] - omega[2] * r[1],
            omega[2] * r[0] - omega[0] * r[2],
            omega[0] * r[1] - omega[1] * r[0],
        ];
        for i in 0..3 {
            assert!(
                (got[i] - k * cross[i]).abs() < 1e-6,
                "r={r:?}: {got:?} contra {:?}",
                [k * cross[0], k * cross[1], k * cross[2]]
            );
        }
    }
}

/// **A ESCALA É O LIMITE QUE A LEI GERAL NÃO CONSEGUE TOMAR** — a
/// singularidade removível que quase me fez shipar o `ν` errado.
#[test]
fn the_scale_field_is_the_limit_the_general_law_cannot_take() {
    // O ganho do bico da lei GERAL é exatamente zero no material que shipa: a
    // rota geral computaria `0/0`.
    assert!(
        affine_gain(Affine::Scale, B).abs() < 1e-6,
        "o ganho geral da escala tinha de ser zero em ν = {POISSON}"
    );
    // ⚠️ **E o limite EXISTE** — a rota fatorada concorda com a lei geral
    // avaliada a um fio de distância da singularidade. É isto que separa *"o
    // modo morreu"* de *"a minha parametrização morreu"*.
    let b = b_of(0.4999);
    for r in [[0.2, 0.1, 0.0], [0.6, -0.3, 0.4], [1.4, 0.0, 0.9]] {
        let got = scale(r, EPS, 0.3, Scales::Mono);
        // A lei geral com `F = 0,3·I`, re-derivada aqui a partir da fórmula do
        // paper — oráculo independente da rota que ele julga.
        let re = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2] + EPS * EPS).sqrt();
        let (re3, re5) = (re * re * re, re * re * re * re * re);
        let c = (1.0 - b) / re3 + 1.5 * EPS * EPS / re5;
        let rr = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];
        let gain = (2.5 - 5.0 * b) / (EPS * EPS * EPS);
        for i in 0..3 {
            let general =
                (c * r[i] + 3.0 * b / re5 * rr * r[i] - b / re3 * 4.0 * r[i]) * 0.3 / gain;
            assert!(
                (got[i] - general).abs() < 2e-3,
                "r={r:?}[{i}]: fatorada {} contra geral {general}",
                got[i]
            );
        }
    }
    // ⚠️ O CONTROLE: as outras duas famílias NÃO são singulares lá, e é isso
    // que torna o zero acima um fato sobre a escala e não sobre o modelo.
    assert!(affine_gain(Affine::Twist, B) > 1.0);
    assert!(affine_gain(Affine::Pinch, B) > 1.0);
}

/// **NEM A ESCALA PERGUNTA DE QUE MATERIAL O BARRO É** — o irmão do gate do
/// twist, e a razão de os dois partilharem o [`super::radial`].
#[test]
fn the_scale_field_does_not_ask_what_the_material_is() {
    let r = [0.35, -0.2, 0.5];
    let got = scale(r, EPS, 1.0, Scales::Tri);
    // `K(r)·r` normalizado por `5/2`, sem um `b` em lado nenhum.
    let mut acc = [0.0f32; 3];
    let mut norm = 0.0f32;
    for &(w, m) in Scales::Tri.taps() {
        let e = EPS * m;
        let re = r_eps(r, e);
        let (re3, re5) = (re * re * re, re * re * re * re * re);
        let k = w * (1.0 / re3 + 1.5 * e * e / re5);
        for i in 0..3 {
            acc[i] += k * r[i];
        }
        norm += w * 2.5 / (e * e * e);
    }
    for i in 0..3 {
        assert!((got[i] - acc[i] / norm).abs() < 1e-6, "{got:?}");
    }
}

/// **O CAMPO DISTANTE CANCELA POR CONSTRUÇÃO** — as duas somas do paper, e não
/// uma tabela de pesos decorada.
#[test]
fn the_far_field_cancels_by_construction() {
    for s in [Scales::Bi, Scales::Tri] {
        let sum: f32 = s.taps().iter().map(|&(w, _)| w).sum();
        assert!(
            sum.abs() < 1e-6,
            "{s:?}: Σw = {sum} (o termo 1/r sobrevive)"
        );
    }
    let sq: f32 = Scales::Tri.taps().iter().map(|&(w, m)| w * m * m).sum();
    assert!(sq.abs() < 1e-6, "Tri: Σwε² = {sq} (o termo 1/r³ sobrevive)");
    // ⚠️ **O CONTROLE que dá sentido ao de cima:** o `Bi` NÃO satisfaz a segunda
    // soma, e é exatamente por isso que ele decai a `1/r³` e não a `1/r⁵`.
    let bi_sq: f32 = Scales::Bi.taps().iter().map(|&(w, m)| w * m * m).sum();
    assert!(bi_sq.abs() > 1.0);
}

/// **O RESÍDUO NA BORDA DA PEGADA, medido** — o número que escolheu a família
/// [`Scales::Tri`] E o [`crate::KELVINLET_REACH`], e o que diz por que um
/// Kelvinlet cru não cabe num pincel que tem raio.
#[test]
fn the_rim_residual_is_what_chose_the_scale_family() {
    let f = [1.0, 0.0, 0.0];
    // O pior caso: à FRENTE do puxão, onde o termo radial soma em vez de faltar.
    // `ε = 1`, e a borda da pegada fica em `r = KELVINLET_REACH`.
    let rim = |s: Scales| len(grab([crate::KELVINLET_REACH, 0.0, 0.0], 1.0, f, s));
    let (mono, bi, tri) = (rim(Scales::Mono), rim(Scales::Bi), rim(Scales::Tri));
    // ⚠️ **É por isto que o campo cru não pode ser o pincel:** quase um TERÇO do
    // deslocamento do bico sobreviveria até ao corte da pegada, e ali ele vira
    // um degrau.
    //
    // ⚠️ **E este gate ESTAVA VERDE sobre o report de 2026-08-14, com a própria
    // mensagem já falsa.** Ele dizia *"o Tri é o que torna a borda do CURSOR
    // honesta"*, frase verdadeira enquanto `ε = raio/3` — aí a pegada era
    // `raio` e o corte CAÍA no anel do cursor. A §7.11 pôs `ε = raio` e a pegada
    // em `3·raio`, e o MESMO `0,0347` mudou-se para 3× fora do cursor, numa
    // costura 11× mais longa; o gate continuou verde porque ele mede o resíduo
    // e o **CERTIFICA como aceitável** — um veredito calibrado para uma
    // colocação que deixou de existir. Quem fecha o degrau é a
    // [`crate::kelvinlet::rim_landing`], e quem o pina é o
    // `the_elastic_field_lands_at_the_rim_instead_of_being_cut`: este aqui
    // segue medindo só qual FAMÍLIA decai mais depressa.
    assert!(
        mono > 0.30,
        "um Kelvinlet cru tinha de sobrar quase um terço na borda: {mono:.4}"
    );
    assert!(
        (0.07..0.09).contains(&bi),
        "o Bi mata o termo 1/r e sobra o 1/r³: {bi:.4}"
    );
    assert!(
        tri < 0.036,
        "o Tri é o que torna o resíduo da borda PEQUENO: {tri:.4}"
    );
    // ⚠️ **A ORDEM é a propriedade**, e não os três números: cada soma que
    // cancela tem de comprar uma ordem de decaimento.
    assert!(tri < bi && bi < mono);
}

/// **O GRADIENTE NA ORIGEM É A MATRIZ** — a normalização das três famílias
/// afins, por diferença finita e sem tocar na aritmética que ela julga.
#[test]
fn the_affine_tip_is_the_matrix_it_was_asked_for() {
    let h = 1e-3_f32;
    let axis = [0.0, 0.0, 1.0];
    // Twist: `u(h·x) ≈ ω × (h·x)`.
    let u = twist([h, 0.0, 0.0], EPS, [0.0, 0.0, 2.0], Scales::Tri);
    assert!((u[1] / h - 2.0).abs() < 5e-3, "twist: {:?}", u[1] / h);
    // Scale: `u(h·x) ≈ s·h·x`.
    let u = scale([h, 0.0, 0.0], EPS, 0.5, Scales::Tri);
    assert!((u[0] / h - 0.5).abs() < 5e-3, "scale: {:?}", u[0] / h);
    // Pinch: `+s` ao longo da normal, `−s/2` no plano perpendicular.
    let along = pinch([0.0, 0.0, h], EPS, axis, 0.4, Scales::Tri);
    assert!((along[2] / h - 0.4).abs() < 5e-3, "pinch axial: {along:?}");
    let across = pinch([h, 0.0, 0.0], EPS, axis, 0.4, Scales::Tri);
    assert!(
        (across[0] / h + 0.2).abs() < 5e-3,
        "pinch lateral: {across:?}"
    );
}

/// **O CAMPO É LISO ONDE O DE 1848 É SINGULAR** — a regularização inteira, num
/// gate: aproximar-se do centro converge para o gesto em vez de divergir.
#[test]
fn the_regularised_field_is_finite_at_the_centre() {
    let f = [1.0, 0.0, 0.0];
    let mut prev = f32::INFINITY;
    for k in [1.0, 0.5, 0.25, 0.125, 0.0625, 0.0] {
        let u = grab([k * EPS, 0.0, 0.0], EPS, f, Scales::Tri);
        let d = len(sub(u, f));
        assert!(u.iter().all(|c| c.is_finite()), "r = {k}·ε deu {u:?}");
        assert!(
            d <= prev + 1e-6,
            "aproximar-se do centro afastou-se do gesto"
        );
        prev = d;
    }
    assert!(prev < 1e-6);
}

/// **A LINEARIDADE NO GESTO** — dobrar o puxão dobra o campo inteiro, em toda
/// parte. É o que permite ao chamador multiplicar o gesto pela força do pincel
/// *antes* de chamar, em vez de a força ter de atravessar o kernel.
#[test]
fn the_field_is_linear_in_the_gesture() {
    let r = [0.31, -0.12, 0.44];
    let a = grab(r, EPS, [1.0, 0.0, 0.0], Scales::Tri);
    let b = grab(r, EPS, [0.0, 1.0, 0.0], Scales::Tri);
    let both = grab(r, EPS, [1.0, 1.0, 0.0], Scales::Tri);
    for i in 0..3 {
        assert!((both[i] - a[i] - b[i]).abs() < 1e-6);
    }
    let twice = grab(r, EPS, [2.0, 0.0, 0.0], Scales::Tri);
    for i in 0..3 {
        assert!((twice[i] - 2.0 * a[i]).abs() < 1e-6);
    }
}

/// **O CAMPO NÃO TEM ORIENTAÇÃO PRÓPRIA** — girar o gesto e o ponto juntos gira
/// a resposta. Sem isto o pincel teria um eixo preferido escondido, e o artista
/// veria a mesma pincelada dar resultados diferentes conforme a câmera.
#[test]
fn the_field_turns_with_the_world() {
    let r = [0.4, 0.15, -0.2];
    let f = [0.6, -0.3, 0.1];
    // Rotação de 90° em torno de Z: `(x, y, z) -> (−y, x, z)`.
    let rot = |v: [f32; 3]| [-v[1], v[0], v[2]];
    let direct = rot(grab(r, EPS, f, Scales::Tri));
    let turned = grab(rot(r), EPS, rot(f), Scales::Tri);
    for i in 0..3 {
        assert!(
            (direct[i] - turned[i]).abs() < 1e-6,
            "{direct:?} contra {turned:?}"
        );
    }
}
