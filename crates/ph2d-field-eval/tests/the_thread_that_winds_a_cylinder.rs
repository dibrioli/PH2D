//! ⭐⭐⭐ **OS GATES DA ROSCA** (W135) — o cilindro com o filete helicoidal em V.
//!
//! # ⚠️ Por que a SECÇÃO é o primeiro gate, e não o gradiente
//!
//! A W134 pagou a lei mais cara desta família: *um minorante frouxo não deixa a peça conservadora —
//! ele INFLA-A*. Um campo com `‖∇f‖ ≤ 1` e um minorante honesto pode estar a desenhar **outra
//! forma**, e os dois gates de campo ficam verdes. ⇒ o gate nº 1 pergunta pela **superfície**: ela
//! é o triângulo que o painel autora?
//!
//! ⭐⭐ E os dois juntos dão o resto de graça: *uma função 1-Lipschitz que se anula na fronteira é,
//! por teorema, um minorante da distância a ela.* Não é preciso um terceiro gate a medir distâncias.

use ph2d_field::{
    FieldDoc, MAX_THREAD_STARTS, Node, NodeId, NodeKind, Primitive, Xform, thread_depth_ceiling,
    thread_round_limit,
};
use ph2d_field_eval::{Field, ops_thread};

const RAIO: f64 = 0.17;
const ALTURA: f64 = 0.24;

fn campo(pitch: f64, flank: f64, starts: u32, hands: u32, fraccao: f64) -> (Field, f64, f64) {
    #[allow(clippy::cast_possible_truncation)]
    let depth = f64::from(thread_depth_ceiling(
        RAIO as f32,
        pitch as f32,
        flank as f32,
    )) * fraccao;
    let f = Field::from_tree(&ops_thread::sd_thread(
        RAIO, ALTURA, pitch, depth, flank, starts, hands, 0.0, 0.0,
    ));
    (f, depth, RAIO - depth)
}

/// O primeiro cruzamento da superfície ao andar do eixo para fora — passo a passo, nunca bissecção
/// de intervalo largo (a armadilha que a W134 pagou: ela agarra o fio VIZINHO).
fn primeiro_cruzamento(f: &Field, phi: f64, z: f64, r0: f64, r1: f64) -> Option<f64> {
    let n = 3000;
    let (mut ant, mut r_ant) = (f.at(r0 * phi.cos(), r0 * phi.sin(), z), r0);
    for i in 1..=n {
        let r = r0 + (r1 - r0) * (i as f64) / (n as f64);
        let v = f.at(r * phi.cos(), r * phi.sin(), z);
        if ant < 0.0 && v >= 0.0 {
            return Some(r_ant + (r - r_ant) * (-ant) / (v - ant));
        }
        (ant, r_ant) = (v, r);
    }
    None
}

/// ⭐⭐⭐ **A superfície É o triângulo autorado** — o gate que apanha um minorante que engorda a peça.
#[test]
fn the_thread_carries_the_profile_it_promises() {
    for (pitch, flank, starts) in [
        (0.045_f64, 30.0_f64, 1_u32),
        (0.090, 30.0, 1),
        (0.090, 30.0, 4),
        (0.090, 60.0, 1),
        (0.030, 10.0, 1),
    ] {
        let (f, depth, nucleo) = campo(pitch, flank, starts, 1, 0.75);
        let alpha: f64 = flank.to_radians();
        let a = depth * alpha.tan();
        let b = f64::from(starts) * pitch / std::f64::consts::TAU;
        let (mut pior, mut n) = (0.0_f64, 0_u32);
        for iw in 0..25 {
            for iphi in 0..5 {
                let phi = -2.4 + 4.8 * f64::from(iphi) / 4.0;
                let w = -pitch * 0.5 + pitch * f64::from(iw) / 24.0;
                let z = w + b * phi;
                if z.abs() > ALTURA * 0.7 {
                    continue;
                }
                let Some(r) = primeiro_cruzamento(&f, phi, z, nucleo * 0.8, RAIO * 1.4) else {
                    continue;
                };
                // ⚠️ O ORÁCULO sai do painel (núcleo, profundidade, α), nunca da árvore.
                let prometido = nucleo + depth * (1.0 - (w.abs() / a).min(1.0));
                pior = pior.max(((r - prometido) / depth).abs());
                n += 1;
            }
        }
        assert!(n > 60, "amostras a menos ({n}) — a sonda não achou a peça");
        assert!(
            pior < 0.02,
            "o perfil entregue erra {:.2} % da profundidade em (passo {pitch}, flanco {flank}, \
             entradas {starts}) — a superfície NÃO é o triângulo autorado",
            pior * 100.0
        );
    }
}

/// ⭐⭐⭐ **O FILETE ASSENTA NO NÚCLEO** — a cunha dos dois flancos não continua até ao eixo.
///
/// ⛔ **Sem o terceiro semiespaço a cunha GANHA o `min` lá dentro** (o termo `(|w| − a)·cos α` é
/// negativo e empurra-a abaixo de `dr`), e ali `|∇w| = b/ρ` explode: medido `‖∇f‖ = 2,4562` a
/// `ρ = 0,013`. ⚠️ **Nenhuma régua de FORMA o via** — o ponto é fundo dentro da peça.
#[test]
fn the_ridge_sits_on_the_core_and_never_reaches_the_axis() {
    let (f, _, nucleo) = campo(0.09, 60.0, 4, 1, 0.99);
    let mut pior = 0.0_f64;
    for i in 0..40 {
        for j in 0..40 {
            for k in 0..12 {
                let rho = nucleo * 0.02 + nucleo * 0.9 * f64::from(i) / 39.0;
                let phi = -3.1 + 6.2 * f64::from(j) / 39.0;
                let z = -0.1 + 0.2 * f64::from(k) / 11.0;
                let (x, y) = (rho * phi.cos(), rho * phi.sin());
                // ⭐ Dentro do núcleo o campo TEM de ser o do cilindro **cortado pela laje**, ao
                // valor. ⚠️ **A 1.ª redacção esquecia a laje** e acusava `0,001393` num campo
                // correcto: junto ao eixo o `dr` fica mais fundo que `|z| − H/2` e quem ganha o
                // `max` é o comprimento da peça. *Um oráculo a que falta um dos termos do produto
                // acusa o produto de um defeito que é dele.*
                let esperado = (rho - nucleo).max(z.abs() - ALTURA);
                pior = pior.max((f.at(x, y, z) - esperado).abs());
            }
        }
    }
    assert!(
        pior < 1.0e-6,
        "dentro do núcleo o campo afasta-se do cilindro em {pior:.6} — a cunha do filete está a \
         ganhar o `min` onde não devia, e é lá que `b/ρ` explode"
    );
}

/// ⭐⭐ **A COSTURA DO `atan2` NÃO EXISTE** — e a razão é que `starts` é um INTEIRO: em
/// `φ → φ − 2π` o `w` anda `starts` períodos exactos, e o reduzido não se mexe.
///
/// ⚠️ **A régua é o salto ENCOLHER com o passo** — uma descontinuidade não encolhe.
#[test]
fn the_seam_of_the_angle_does_not_crack_the_piece() {
    for starts in [1_u32, 3, 8] {
        let (f, _, _) = campo(0.09, 30.0, starts, 1, 0.8);
        let mut anterior = f64::INFINITY;
        for passo in [1.0e-2_f64, 1.0e-3, 1.0e-4] {
            let mut salto = 0.0_f64;
            for k in 0..24 {
                let z = -0.15 + 0.3 * f64::from(k) / 23.0;
                for rho in [RAIO * 0.85, RAIO, RAIO * 1.1] {
                    // os dois lados do corte do `atan2`: y = ±δ com x < 0
                    let a = f.at(-rho, passo, z);
                    let b = f.at(-rho, -passo, z);
                    salto = salto.max((a - b).abs());
                }
            }
            assert!(
                salto < anterior * 0.5 || salto < 1.0e-9,
                "o salto na costura não encolheu com o passo ({salto:.3e} contra {anterior:.3e}, \
                 entradas {starts}) — isso é uma DESCONTINUIDADE, não erro de amostragem"
            );
            anterior = salto;
        }
    }
}

/// ⭐⭐⭐ **AS DUAS MÃOS FAZEM UM CRUZAMENTO, e uma não** — e a prova é uma SIMETRIA exacta.
///
/// Sob `y → −y` temos `φ → −φ`, logo `w₁ = z − bφ` e `w₂ = z + bφ` **trocam**. Com as duas mãos o
/// conjunto `{w₁, w₂}` é invariante ⇒ o campo é espelhado ao bit; com uma mão não é.
#[test]
fn the_two_hands_are_a_mirror_and_one_hand_is_not() {
    let ler = |hands: u32| {
        let (f, _, _) = campo(0.09, 45.0, 6, hands, 0.7);
        let (mut pior, mut n) = (0.0_f64, 0_u32);
        for i in 0..14 {
            for j in 0..14 {
                for k in 0..6 {
                    let x = -0.2 + 0.4 * f64::from(i) / 13.0;
                    let y = 0.02 + 0.18 * f64::from(j) / 13.0;
                    let z = -0.12 + 0.24 * f64::from(k) / 5.0;
                    let d = (f.at(x, y, z) - f.at(x, -y, z)).abs();
                    if d.is_finite() {
                        pior = pior.max(d);
                        n += 1;
                    }
                }
            }
        }
        assert!(n > 500);
        pior
    };
    let duas = ler(2);
    let uma = ler(1);
    assert!(
        duas < 1.0e-9,
        "com as DUAS mãos o campo tinha de ser espelhado ao bit, e erra {duas:.3e}"
    );
    assert!(
        uma > 1.0e-3,
        "com UMA mão o campo NÃO pode ser espelhado ({uma:.3e}) — se for, a segunda mão não está a \
         fazer nada e o knob é morto"
    );
}

/// ⭐⭐ **UMA ENTRADA ESCRITA COM FRACÇÃO ATERRA NUMA ENTRADA** — a lição da W128 paga pela
/// REPRESENTAÇÃO: um `b` fraccionário rachava a peça de alto a baixo na costura do `atan2`.
#[test]
fn a_start_written_with_a_fraction_lands_on_a_start() {
    let mut p = Primitive::Thread {
        radius: 0.5,
        half_height: 0.4,
        pitch: 0.12,
        depth: thread_depth_ceiling(0.5, 0.12, 30.0) * 0.7,
        flank: 30.0,
        starts: 2,
        hands: 1,
        round: 0.0,
        chamfer: 0.0,
    };
    // ⚠️ **O ZERO não entra aqui, e é a faixa que o diz** — a [`Span::Count`] começa em `1`, e a
    // guarda da porta recusa um valor que a faixa não oferece. *A 1.ª redacção deste gate esperava
    // que o zero fosse COAGIDO para `1`, e a porta devolveu `Err`: a régua estava errada, não o
    // produto.* A linha abaixo é essa lei escrita como afirmação.
    assert!(
        ph2d_field::set_dim(&mut p, 0, 5, 0.0).is_err(),
        "a faixa das entradas não oferece o zero, logo a porta tem de o RECUSAR em vez de o coagir"
    );
    for (escrito, esperado) in [(2.6_f32, 3_u32), (2.4, 2), (9999.0, MAX_THREAD_STARTS)] {
        ph2d_field::set_dim(&mut p, 0, 5, escrito).expect("a escrita");
        let Primitive::Thread { starts, .. } = p else {
            unreachable!()
        };
        assert_eq!(
            starts, esperado,
            "escrever {escrito} nas entradas devia aterrar em {esperado}"
        );
    }
}

/// ⭐⭐ **O TECTO DA PROFUNDIDADE É ONDE AS VOLTAS SE TOCAM** — `2·depth·tan α = pitch`, que é a
/// cerca do MODELO (acima dela o triângulo invade a volta vizinha e o campo, que mede só a mais
/// próxima, passa a dizer «fora» dentro da peça).
#[test]
fn the_depth_ceiling_is_where_the_turns_touch() {
    for (pitch, flank) in [(0.12_f32, 30.0_f32), (0.20, 45.0), (0.06, 10.0)] {
        // ⚠️ Um raio GRANDE, para o tecto do núcleo não ser quem manda nesta medição.
        let tecto = thread_depth_ceiling(10.0, pitch, flank);
        let base = 2.0 * tecto * flank.to_radians().tan();
        assert!(
            (base - pitch).abs() < pitch * 1.0e-4,
            "no tecto a base do triângulo devia ser exactamente o passo: {base} contra {pitch}"
        );
    }
    // ⭐ E o OUTRO recurso — o núcleo — manda quando o passo é grosso.
    let tecto = thread_depth_ceiling(1.0, 100.0, 30.0);
    assert!(
        (tecto - (1.0 - ph2d_field::THREAD_CORE_FLOOR)).abs() < 1.0e-5,
        "com um passo enorme quem manda tem de ser o núcleo, e não o período: {tecto}"
    );
}

/// ⭐⭐ **MEXER NO PASSO RE-ASSENTA A PROFUNDIDADE** — a porta repõe a invariante, e quem o faz é a
/// coerção GERAL. *Uma escrita que deixa a peça inválida apaga a cena inteira* (a lei da W127).
#[test]
fn narrowing_the_pitch_reseats_the_depth() {
    let mut p = Primitive::Thread {
        radius: 0.5,
        half_height: 0.4,
        pitch: 0.24,
        depth: thread_depth_ceiling(0.5, 0.24, 30.0) * 0.99,
        flank: 30.0,
        starts: 1,
        hands: 1,
        round: 0.0,
        chamfer: 0.0,
    };
    // o passo cai para um quarto — a profundidade tem de o seguir
    ph2d_field::set_dim(&mut p, 0, 2, 0.06).expect("a escrita");
    // ⚠️ A porta da validação é o próprio documento — é ele que a cena constrói por quadro.
    FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p.clone()))],
        NodeId(0),
    )
    .expect("a peça ficou inválida depois de apertar o passo");
    let Primitive::Thread { depth, .. } = p else {
        unreachable!()
    };
    assert!(
        depth <= thread_depth_ceiling(0.5, 0.06, 30.0),
        "a profundidade não seguiu o passo: {depth}"
    );
}

/// ⭐⭐ **A ROSCA OFERECE FILETE PORQUE TEM ARESTAS** — e o tecto dele MORRE quando a terra fecha,
/// que é a segunda metade da lei do [`thread_round_limit`].
#[test]
fn the_thread_offers_a_fillet_and_it_dies_when_the_land_closes() {
    let tecto = thread_depth_ceiling(0.5, 0.16, 30.0);
    let a_meio = thread_round_limit(0.16, tecto * 0.5, 30.0);
    let no_tecto = thread_round_limit(0.16, tecto * 0.999, 30.0);
    assert!(
        a_meio > 0.0,
        "a meia profundidade a rosca tem de oferecer filete, e ofereceu {a_meio}"
    );
    assert!(
        no_tecto < a_meio * 0.02,
        "no tecto da profundidade a terra fecha e o filete não tem onde caber: {no_tecto} contra \
         {a_meio}"
    );
    let p = Primitive::Thread {
        radius: 0.5,
        half_height: 0.4,
        pitch: 0.16,
        depth: tecto * 0.5,
        flank: 30.0,
        starts: 1,
        hands: 1,
        round: 0.0,
        chamfer: 0.0,
    };
    assert_eq!(
        ph2d_field::round_limit(&p),
        Some(a_meio),
        "o painel e a validação têm de ler o MESMO tecto de filete"
    );
}

/// ⭐⭐ **O DIVISOR SEGURA A FAMÍLIA INTEIRA** — `‖∇f‖ ≤ 1` em toda a faixa permitida de entradas e
/// de mãos. ⚠️ Ele é a metade que, com a secção acima, dá o minorante por teorema.
#[test]
fn every_start_count_keeps_the_field_marching() {
    for starts in [1_u32, 2, 8, MAX_THREAD_STARTS] {
        for hands in [1_u32, 2] {
            let (f, _, _) = campo(0.09, 30.0, starts, hands, 0.9);
            let mut pior = 0.0_f64;
            for i in 0..26 {
                for j in 0..26 {
                    for k in 0..26 {
                        let at = |t: usize| -0.29 + 0.58 * (t as f64 + 0.5) / 26.0;
                        let (x, y, z) = (at(i), at(j), at(k));
                        let g = f.gradient_norm(x, y, z, 1.0e-5);
                        if g.is_finite() {
                            pior = pior.max(g);
                        }
                    }
                }
            }
            assert!(
                pior < 1.02,
                "‖∇f‖ = {pior:.4} com {starts} entradas e {hands} mão(s) — o divisor `k` deixou de \
                 segurar a família"
            );
        }
    }
}

/// A peça que o censo constrói tem de caber num documento — o representante do censo é derivado do
/// tecto, e este gate é a rede contra ele deixar de o ser.
#[test]
fn the_thread_of_the_census_is_a_piece_the_document_accepts() {
    let p = Primitive::Thread {
        radius: 0.5,
        half_height: 0.45,
        pitch: 0.16,
        depth: thread_depth_ceiling(0.5, 0.16, 30.0) * 0.75,
        flank: 30.0,
        starts: 1,
        hands: 1,
        round: 0.0,
        chamfer: 0.0,
    };
    FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("o documento tem de aceitar a rosca do censo");
}
