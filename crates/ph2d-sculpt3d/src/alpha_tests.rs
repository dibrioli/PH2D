//! **O KERNEL do alpha** — o que cada padrão promete, medido.
//!
//! ⚠️ A pergunta que estes gates existem para responder **não** é *"o padrão
//! parece bonito?"* — isso é do smoke, e o oráculo é o olho. É *"ele é um
//! STENCIL?"*: ele cobre a faixa inteira (`0` em algum lugar, `1` em outro), ele
//! é função apenas da posição e da escala, e ele não carrega uma trama que
//! ninguém autorou.
//!
//! Um padrão que ficasse, digamos, entre 0,4 e 0,6 compilaria, pintaria um chip,
//! passaria por qualquer teste de "mudou alguma coisa" — e seria **um redutor de
//! força com nome de textura**. É esse o modo de falha aqui.

use super::*;

/// Uma nuvem de pontos determinística e sem estrutura de grade.
///
/// ⚠️ **Amostrar numa grade regular seria o pior fixture possível para este
/// módulo**: os padrões são construídos SOBRE uma grade, então uma amostragem
/// alinhada mediria a fase dela em vez do padrão. O passo irracional garante que
/// as amostras varrem a célula inteira.
fn cloud(n: usize) -> Vec<[f32; 3]> {
    (0..n)
        .map(|i| {
            let t = i as f32;
            [
                t * 0.073_137_1 - 3.0,
                t * 0.041_231_5 + 1.7,
                t * 0.098_765_4 - 0.9,
            ]
        })
        .collect()
}

const SCALE: f32 = 0.25;

/// **Todo padrão é um STENCIL: ele fica em `[0, 1]` e cobre quase a faixa toda.**
#[test]
fn every_pattern_is_a_stencil_and_not_a_dimmer() {
    let pts = cloud(4000);
    for a in Alpha::ALL {
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for &p in &pts {
            let w = a.weight_at(p, SCALE);
            assert!(
                (0.0..=1.0).contains(&w),
                "{} saiu da faixa unitária em {p:?}: {w}",
                a.label()
            );
            lo = lo.min(w);
            hi = hi.max(w);
        }
        assert!(
            lo < 0.05,
            "{} nunca chega a zero (mínimo {lo}) — ele não recorta nada",
            a.label()
        );
        assert!(
            hi > 0.95,
            "{} nunca chega a um (máximo {hi}) — ele só enfraquece o pincel",
            a.label()
        );
    }
}

/// **O padrão é função APENAS da posição e da escala** — nenhum estado
/// escondido, nenhuma dependência da ordem em que se pergunta.
///
/// ⚠️ É o que o motor assume ao ler o alpha na posição congelada: se a mesma
/// posição desse dois valores, o envelope voltaria a lavar o padrão.
#[test]
fn the_same_point_always_weighs_the_same() {
    let pts = cloud(500);
    for a in Alpha::ALL {
        let first: Vec<f32> = pts.iter().map(|&p| a.weight_at(p, SCALE)).collect();
        // De trás para a frente, e intercalando os padrões: se houvesse cache ou
        // estado, a ordem o denunciaria.
        for (i, &p) in pts.iter().enumerate().rev() {
            for other in Alpha::ALL {
                let _ = other.weight_at([p[1], p[2], p[0]], SCALE * 1.3);
            }
            assert_eq!(
                a.weight_at(p, SCALE),
                first[i],
                "{} mudou de opinião sobre {p:?}",
                a.label()
            );
        }
    }
}

/// **Dobrar a escala dobra a feature, AO BIT.**
///
/// Só `p / scale` entra na fórmula, então `w(2p, 2s)` e `w(p, s)` são a mesma
/// expressão — e com potências de dois a divisão é exata, o que transforma uma
/// afirmação sobre "invariância" numa igualdade de bits em vez de uma tolerância
/// escolhida a dedo.
///
/// ⚠️ É esta propriedade que faz a pista de `Alpha Scale` significar *tamanho da
/// feature*: sem ela, o knob mudaria o padrão além de mudar o tamanho dele, e
/// procurar a escala certa envolveria re-procurar o padrão que já se tinha.
#[test]
fn doubling_the_scale_doubles_the_feature_bit_for_bit() {
    let pts = cloud(1000);
    for a in Alpha::ALL {
        for &p in &pts {
            let twice = [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0];
            assert_eq!(
                a.weight_at(twice, SCALE * 2.0),
                a.weight_at(p, SCALE),
                "{} não é auto-similar em {p:?}",
                a.label()
            );
        }
    }
}

/// **Um ponto não-finito não pesa nada** — a mesma peneira do
/// [`crate::Falloff::weight`], e pelo mesmo motivo.
#[test]
fn a_non_finite_point_weighs_nothing() {
    let bad = [
        [f32::NAN, 0.0, 0.0],
        [0.0, f32::INFINITY, 0.0],
        [0.0, 0.0, f32::NEG_INFINITY],
        [f32::NAN, f32::NAN, f32::NAN],
    ];
    for a in Alpha::ALL {
        for &p in &bad {
            assert_eq!(
                a.weight_at(p, SCALE),
                0.0,
                "{} deixou {p:?} passar",
                a.label()
            );
        }
        // E uma escala absurda é grampeada, não propagada.
        assert!(a.weight_at([0.3, 0.2, 0.1], f32::NAN).is_finite());
        assert!(a.weight_at([0.3, 0.2, 0.1], 0.0).is_finite());
        assert!(a.weight_at([0.3, 0.2, 0.1], -5.0).is_finite());
    }
}

/// A fração média da superfície que um padrão cobre.
fn coverage(a: Alpha, pts: &[[f32; 3]]) -> f32 {
    let sum: f32 = pts.iter().map(|&p| a.weight_at(p, SCALE)).sum();
    sum / pts.len() as f32
}

/// **Os padrões ESPARSOS cobrem menos que os densos.**
///
/// ⚠️ Não é decoração de catálogo: a cobertura é a força aparente que o pincel
/// perde ao armar o alpha, e é ela que decide se o artista precisa subir a
/// força. Uma trinca cobre pouco **de propósito** (uma trinca é uma linha), e um
/// dia em que Cracks cobrisse tanto quanto Scales seria o dia em que as duas
/// larguras de sulco tivessem convergido — dois chips desenhando a mesma coisa.
#[test]
fn the_sparse_patterns_cover_less_than_the_dense_ones() {
    let pts = cloud(4000);
    let (cracks, pores) = (coverage(Alpha::Cracks, &pts), coverage(Alpha::Pores, &pts));
    let (scales, ridges) = (coverage(Alpha::Scales, &pts), coverage(Alpha::Ridges, &pts));
    for (name, sparse) in [("Cracks", cracks), ("Pores", pores)] {
        for (dname, dense) in [("Scales", scales), ("Ridges", ridges)] {
            assert!(
                sparse < dense * 0.6,
                "{name} cobre {sparse:.3} contra {dname} {dense:.3} — \
                 os dois desenham a mesma densidade"
            );
        }
    }
}

/// **O padrão não repete com o período da grade que o gera.**
///
/// A correlação entre `w(p)` e `w(p + 1 célula)` tem de ser baixa. Se ela fosse
/// alta, o artista veria a trama da grade de hash impressa na escultura — uma
/// regularidade que ninguém autorou, e a assinatura clássica de um ruído mal
/// construído.
#[test]
fn the_pattern_does_not_repeat_with_the_lattice() {
    let pts = cloud(3000);
    for a in Alpha::ALL {
        // Um passo de exatamente uma célula, no eixo x.
        let cell = SCALE / a.frequency();
        let xs: Vec<f32> = pts.iter().map(|&p| a.weight_at(p, SCALE)).collect();
        let ys: Vec<f32> = pts
            .iter()
            .map(|&p| a.weight_at([p[0] + cell, p[1], p[2]], SCALE))
            .collect();
        assert!(
            correlation(&xs, &ys).abs() < 0.35,
            "{} repete com o período da grade (correlação {:.3})",
            a.label(),
            correlation(&xs, &ys)
        );
    }
}

/// Correlação de Pearson. `0` = as duas amostras não sabem nada uma da outra.
fn correlation(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len() as f32;
    let (ma, mb) = (a.iter().sum::<f32>() / n, b.iter().sum::<f32>() / n);
    let mut cov = 0.0;
    let (mut va, mut vb) = (0.0f32, 0.0f32);
    for (&x, &y) in a.iter().zip(b) {
        let (dx, dy) = (x - ma, y - mb);
        cov += dx * dy;
        va += dx * dx;
        vb += dy * dy;
    }
    let denom = (va * vb).sqrt();
    if denom <= 0.0 { 0.0 } else { cov / denom }
}

/// **A JANELA DE 27 CÉLULAS É SUFICIENTE** — medido contra uma busca em 125.
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE.** Trocar a varredura pelas
/// 8 células de um octante devolve a semente ERRADA em boa parte do espaço — os
/// poros ficam cortados de um lado, as escamas assimétricas — e passava em
/// **todos** os outros gates: o resultado continua em `[0,1]`, continua variando,
/// continua determinístico, continua auto-similar e continua descorrelacionado no
/// período da grade. Nenhuma dessas perguntas é *"a semente mais próxima é a mais
/// próxima?"*.
///
/// ⚠️ **O oráculo é uma RE-IMPLEMENTAÇÃO deliberada** (o padrão do `serial_side`
/// do Painter): ele recomputa a posição da semente pela mesma fórmula, mas varre
/// **125** células em vez de 27. Se o produto e a referência concordam, a janela
/// menor é suficiente — e é isso que a prosa do [`worley`] afirma. Se um dia
/// discordarem, o número 27 é que está errado.
#[test]
fn the_nearest_seed_is_inside_the_twenty_seven_cell_window() {
    for p in cloud(2000) {
        let (f1, f2) = worley(p);
        let base = [p[0].floor(), p[1].floor(), p[2].floor()];
        let bi = [base[0] as i32, base[1] as i32, base[2] as i32];
        let (mut b1, mut b2) = (f32::INFINITY, f32::INFINITY);
        for dz in -2..=2 {
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let cell = [bi[0] + dx, bi[1] + dy, bi[2] + dz];
                    let h = hash3(cell[0], cell[1], cell[2]);
                    let d = [
                        (cell[0] as f32 - base[0] + unit(h)) - (p[0] - base[0]),
                        (cell[1] as f32 - base[1] + unit(h.rotate_left(11))) - (p[1] - base[1]),
                        (cell[2] as f32 - base[2] + unit(h.rotate_left(22))) - (p[2] - base[2]),
                    ];
                    let d2 = d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1]));
                    if d2 < b1 {
                        b2 = b1;
                        b1 = d2;
                    } else if d2 < b2 {
                        b2 = d2;
                    }
                }
            }
        }
        assert!(
            (f1 - b1.sqrt()).abs() < 1e-6,
            "a 1ª semente de {p:?} está fora da janela: {f1} contra {}",
            b1.sqrt()
        );
        assert!(
            (f2 - b2.sqrt()).abs() < 1e-6,
            "a 2ª semente de {p:?} está fora da janela: {f2} contra {}",
            b2.sqrt()
        );
    }
}

/// **Os três eixos entram no hash por caminhos DIFERENTES.**
///
/// A prosa do [`hash3`] diz que multiplicadores iguais fariam `(a,b,c)` e
/// `(b,a,c)` colidirem, e o padrão sairia espelhado em torno da diagonal — uma
/// simetria que ninguém autorou. Isto é a afirmação, medida.
#[test]
fn permuting_the_axes_gives_a_different_cell() {
    assert_ne!(hash3(1, 2, 3), hash3(2, 1, 3));
    assert_ne!(hash3(1, 2, 3), hash3(1, 3, 2));
    assert_ne!(hash3(1, 2, 3), hash3(3, 2, 1));
}

/// **Um pincel sem alpha pesa `1.0` EXATO** — é daqui que sai a byte-identidade.
///
/// ⚠️ **A afirmação inteira é sobre o BIT.** `x * 1.0` é `x` ao bit no IEEE-754
/// (para todo finito, e para o infinito), então a multiplicação que o motor
/// acrescentou ao falloff **não pode mover um resultado** enquanto esta função
/// devolver exatamente um. Devolver `0,999_999` compilaria, pintaria igual, e
/// mudaria a última casa de todos os dezesseis verbos — uma regressão que
/// nenhuma tolerância de gate veria.
#[test]
fn a_brush_without_an_alpha_weighs_exactly_one() {
    let plain = crate::Brush::default();
    assert!(plain.alpha.is_none(), "o default tem de ser SEM alpha");
    for p in cloud(200) {
        assert_eq!(plain.alpha_weight(p), 1.0, "peso ≠ 1 em {p:?}");
    }
    // Inclusive onde o padrão armado devolveria zero.
    assert_eq!(plain.alpha_weight([f32::NAN, 0.0, 0.0]), 1.0);

    // E o inverso: armado, ele deixa de ser constante.
    let armed = crate::Brush {
        alpha: Some(Alpha::Pores),
        ..crate::Brush::default()
    };
    let ws: Vec<f32> = cloud(500)
        .into_iter()
        .map(|p| armed.alpha_weight(p))
        .collect();
    let lo = ws.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = ws.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        hi - lo > 0.9,
        "armado, o alpha ainda é quase constante ({lo}..{hi})"
    );
}
