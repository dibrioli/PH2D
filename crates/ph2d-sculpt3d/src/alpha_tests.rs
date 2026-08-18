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

/// O frame de FÁBRICA — o que o pincel default carrega.
///
/// ⚠️ Os seis isotrópicos o ignoram por construção (e há gate para isso); os três
/// direcionais são medidos no eixo em que o artista os encontra.
fn frame() -> AlphaFrame {
    crate::Brush::default().alpha_frame()
}

/// **Todo padrão é um STENCIL: ele fica em `[0, 1]` e cobre quase a faixa toda.**
#[test]
fn every_pattern_is_a_stencil_and_not_a_dimmer() {
    let f = frame();
    let pts = cloud(4000);
    for a in Alpha::ALL {
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for &p in &pts {
            let w = a.weight_at(p, SCALE, &f);
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
    let f = frame();
    let pts = cloud(500);
    for a in Alpha::ALL {
        let first: Vec<f32> = pts.iter().map(|&p| a.weight_at(p, SCALE, &f)).collect();
        // De trás para a frente, e intercalando os padrões: se houvesse cache ou
        // estado, a ordem o denunciaria.
        for (i, &p) in pts.iter().enumerate().rev() {
            for other in Alpha::ALL {
                let _ = other.weight_at([p[1], p[2], p[0]], SCALE * 1.3, &f);
            }
            assert_eq!(
                a.weight_at(p, SCALE, &f),
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
    let f = frame();
    let pts = cloud(1000);
    for a in Alpha::ALL {
        for &p in &pts {
            let twice = [p[0] * 2.0, p[1] * 2.0, p[2] * 2.0];
            assert_eq!(
                a.weight_at(twice, SCALE * 2.0, &f),
                a.weight_at(p, SCALE, &f),
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
    let f = frame();
    let bad = [
        [f32::NAN, 0.0, 0.0],
        [0.0, f32::INFINITY, 0.0],
        [0.0, 0.0, f32::NEG_INFINITY],
        [f32::NAN, f32::NAN, f32::NAN],
    ];
    for a in Alpha::ALL {
        for &p in &bad {
            assert_eq!(
                a.weight_at(p, SCALE, &f),
                0.0,
                "{} deixou {p:?} passar",
                a.label()
            );
        }
        // E uma escala absurda é grampeada, não propagada.
        assert!(a.weight_at([0.3, 0.2, 0.1], f32::NAN, &f).is_finite());
        assert!(a.weight_at([0.3, 0.2, 0.1], 0.0, &f).is_finite());
        assert!(a.weight_at([0.3, 0.2, 0.1], -5.0, &f).is_finite());
    }
}

/// A fração média da superfície que um padrão cobre.
fn coverage(a: Alpha, pts: &[[f32; 3]]) -> f32 {
    let f = frame();
    let sum: f32 = pts.iter().map(|&p| a.weight_at(p, SCALE, &f)).sum();
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

/// **Um padrão ISOTRÓPICO não repete com o período da grade que o gera.**
///
/// A correlação entre `w(p)` e `w(p + 1 célula)` tem de ser baixa. Se ela fosse
/// alta, o artista veria a trama da grade de hash impressa na escultura — uma
/// regularidade que ninguém autorou, e a assinatura clássica de um ruído mal
/// construído.
///
/// ⚠️ **A premissa é sobre RUÍDO, e ela não alcança a família direcional** — um
/// estrato REPETE por construção, e uma trama também: a regularidade deles é
/// AUTORADA, que é a palavra que separa as duas coisas. Este gate mediu 0,956 no
/// Strata no dia em que ele nasceu, e o número estava certo sobre um padrão
/// correto. O que os três direcionais prometem em vez disto é
/// [`turning_the_axis_turns_the_pattern`].
///
/// ⚠️ **A pergunta é feita à PORTA** ([`Alpha::is_directional`]), nunca a uma
/// lista de nomes aqui — e o **controle positivo** logo abaixo é o que impede
/// este gate de virar vácuo: um `is_directional` que respondesse `true` para
/// todos o deixaria verde sem medir nada.
#[test]
fn an_isotropic_pattern_does_not_repeat_with_the_lattice() {
    let f = frame();
    let pts = cloud(3000);
    let (mut iso, mut dir) = (0usize, 0usize);
    for a in Alpha::ALL {
        if a.is_directional() {
            dir += 1;
            continue;
        }
        iso += 1;
        // Um passo de exatamente uma célula, no eixo x.
        let cell = SCALE / a.frequency();
        let xs: Vec<f32> = pts.iter().map(|&p| a.weight_at(p, SCALE, &f)).collect();
        let ys: Vec<f32> = pts
            .iter()
            .map(|&p| a.weight_at([p[0] + cell, p[1], p[2]], SCALE, &f))
            .collect();
        assert!(
            correlation(&xs, &ys).abs() < 0.35,
            "{} repete com o período da grade (correlação {:.3})",
            a.label(),
            correlation(&xs, &ys)
        );
    }
    assert!(
        iso > 0 && dir > 0,
        "a família se colapsou ({iso} isotrópicos, {dir} direcionais) — \
         este gate está medindo o vácuo"
    );
}

/// **GIRAR O EIXO GIRA O PADRÃO** — o que os três direcionais compartilham, e a
/// única coisa que os separa dos seis isotrópicos.
///
/// O oráculo é uma IDENTIDADE, não um limiar escolhido: o frame de `az + 90°` é o
/// de `az` girado de 90° em torno de Z (`n' = Rn`, `t' = Rt`, `b' = n' × t' =
/// Rb`), então projetar `Rp` no frame girado devolve **as mesmas coordenadas**
/// que projetar `p` no original. ⇒ o padrão visto no ponto girado, com o eixo
/// girado, tem de ser o MESMO padrão.
///
/// ⚠️ **Correlação e não igualdade ao bit**, e o motivo é o rotor: ele ACUMULA
/// noventa passos de um grau, então `rotate_by_degrees(az + 90)` não é o
/// perpendicular exato de `rotate_by_degrees(az)`. A rotação do PONTO é exata
/// (`(x, y) → (−y, x)`); o que carrega o erro é o frame, e sobre uma nuvem ele
/// aparece como uma correlação um pouco abaixo de 1, nunca como um padrão
/// diferente.
///
/// ⚠️ **A metade ISOTRÓPICA é o controle, e ela é BIT-EXATA — mas afirmando
/// OUTRA coisa.** A primeira versão deste gate exigiu a rotação dos NOVE e
/// reprovou o Noise com correlação 0,006, sobre um padrão perfeitamente correto:
/// *isotrópico* aqui quer dizer **não lê o frame**, e não *invariante por
/// rotação* — a grade do ruído é alinhada ao OBJETO, então girar o ponto muda o
/// valor, como tem de mudar. O que os seis prometem é o oposto do que os três
/// prometem: **girar o eixo não move um bit**, com o ponto parado. Duas
/// afirmações, uma por família, e é o par que impede um `is_directional` mentiroso
/// de deixar o gate verde nos dois sentidos.
#[test]
fn turning_the_axis_turns_the_pattern() {
    let pts = cloud(3000);
    let a90 = crate::Brush {
        alpha_az_deg: 90,
        ..crate::Brush::default()
    }
    .alpha_frame();
    let a180 = crate::Brush {
        alpha_az_deg: 180,
        ..crate::Brush::default()
    }
    .alpha_frame();
    // `(x, y) → (−y, x)`: os 90° exatos, sem passar por rotor nenhum.
    let turn = |p: [f32; 3]| [-p[1], p[0], p[2]];

    let (mut iso, mut dir) = (0usize, 0usize);
    for a in Alpha::ALL {
        let here: Vec<f32> = pts.iter().map(|&p| a.weight_at(p, SCALE, &a90)).collect();
        if a.is_directional() {
            dir += 1;
            let there: Vec<f32> = pts
                .iter()
                .map(|&p| a.weight_at(turn(p), SCALE, &a180))
                .collect();
            assert!(
                correlation(&here, &there) > 0.95,
                "{} não girou com o eixo (correlação {:.3})",
                a.label(),
                correlation(&here, &there)
            );
        } else {
            iso += 1;
            let same: Vec<f32> = pts.iter().map(|&p| a.weight_at(p, SCALE, &a180)).collect();
            assert_eq!(
                here,
                same,
                "{} é isotrópico e mesmo assim mudou com o eixo",
                a.label()
            );
        }
    }
    assert!(
        iso > 0 && dir > 0,
        "a família se colapsou ({iso} isotrópicos, {dir} direcionais) — \
         este gate está medindo o vácuo"
    );
}

/// **O STRATA EMPILHA AO LONGO DO EIXO** — a promessa que o artista lê no nome.
///
/// Andar ao longo do eixo atravessa as camadas (o valor varia muito); andar
/// DENTRO de uma camada, perpendicular ao eixo, quase não muda nada. É a
/// diferença entre *camada* e *listra em qualquer direção*, e ela é o que faz o
/// controle de eixo significar alguma coisa.
///
/// ⚠️ **A razão é o oráculo, não os dois números soltos:** uma escala diferente
/// mudaria os dois juntos, e é a RAZÃO entre eles que descreve a forma.
#[test]
fn the_strata_stack_along_the_axis() {
    let f = frame();
    let axis = f.axis();
    // A perpendicular ao eixo, no plano XY — a direção "dentro da camada".
    let across = [-axis[1], axis[0], 0.0];
    let step = SCALE / 4.0;
    let (mut along_var, mut across_var) = (0.0f32, 0.0f32);
    for &p in &cloud(2000) {
        let w = Alpha::Strata.weight_at(p, SCALE, &f);
        let a = Alpha::Strata.weight_at(
            [
                p[0] + axis[0] * step,
                p[1] + axis[1] * step,
                p[2] + axis[2] * step,
            ],
            SCALE,
            &f,
        );
        let c = Alpha::Strata.weight_at(
            [
                p[0] + across[0] * step,
                p[1] + across[1] * step,
                p[2] + across[2] * step,
            ],
            SCALE,
            &f,
        );
        along_var += (a - w).abs();
        across_var += (c - w).abs();
    }
    assert!(
        along_var > across_var * 4.0,
        "as camadas não empilham ao longo do eixo: \
         variação ao longo {along_var:.1} contra {across_var:.1} atravessada"
    );
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
    let f = frame();
    let plain = crate::Brush::default();
    assert!(plain.alpha.is_none(), "o default tem de ser SEM alpha");
    for p in cloud(200) {
        assert_eq!(plain.alpha_weight(p, &f), 1.0, "peso ≠ 1 em {p:?}");
    }
    // Inclusive onde o padrão armado devolveria zero.
    assert_eq!(plain.alpha_weight([f32::NAN, 0.0, 0.0], &f), 1.0);

    // E o inverso: armado, ele deixa de ser constante.
    let armed = crate::Brush {
        alpha: Some(Alpha::Pores),
        ..crate::Brush::default()
    };
    let ws: Vec<f32> = cloud(500)
        .into_iter()
        .map(|p| armed.alpha_weight(p, &f))
        .collect();
    let lo = ws.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = ws.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        hi - lo > 0.9,
        "armado, o alpha ainda é quase constante ({lo}..{hi})"
    );
}

/// **O DESLOCAMENTO MOVE O CARIMBO, e um `[0, 0]` é byte-idêntico.**
///
/// ⚠️ **As duas metades num gate só, porque uma sem a outra mente.** Só a
/// primeira deixaria passar um deslocamento que também mexe no caso neutro (toda
/// arte já feita mudaria de lugar); só a segunda deixaria passar um controle que
/// não faz nada. É o par presença/ausência que este painel já exige de toda row.
#[test]
fn the_offset_moves_the_stamp_and_zero_is_byte_identical() {
    use crate::Brush;
    let (w, h) = (32usize, 32usize);
    // Bandas DIAGONAIS: um padrão constante num dos eixos não distinguiria um
    // deslocamento em X de um em Y, e o gate ficaria verde com os dois trocados.
    let rgba: Vec<u8> = (0..w * h)
        .flat_map(|i| {
            let (x, y) = ((i % w) as f32, (i / w) as f32);
            let v = (((x * 0.4 + y * 0.17).sin() * 0.5 + 0.5) * 255.0) as u8;
            [v, v, v, 255]
        })
        .collect();
    let img = AlphaImage::from_rgba(w as u32, h as u32, &rgba).expect("a fixture é uma imagem");

    // O eixo encarando a vista é o que `set_alpha_image` semeia — sem ele o
    // carimbo é projetado de lado e a fatia degenera, que é outro defeito.
    let mut brush = Brush {
        alpha_scale: 0.5,
        alpha_elev_deg: MAX_AXIS_ELEV_DEG,
        ..Brush::default()
    };
    brush.alpha = Some(Alpha::Image(std::sync::Arc::new(img)));

    let probe = |b: &Brush| -> Vec<f32> {
        let f = b.alpha_frame();
        (0..16)
            .map(|i| {
                let t = 1.6f32.mul_add(i as f32 / 15.0, -0.8);
                b.alpha_weight([t, 0.25, 0.0], &f)
            })
            .collect()
    };

    let neutral = probe(&brush);

    // ⚠️ **`[0, 0]` é o mundo de antes AO BIT** — `x - 0.0` é `x` em IEEE-754.
    let mut zeroed = brush.clone();
    zeroed.alpha_offset = [0.0, 0.0];
    assert_eq!(
        probe(&zeroed),
        neutral,
        "um deslocamento nulo mudou o padrão: toda arte já feita anda de lugar"
    );

    // E um deslocamento de MEIO ladrilho move o carimbo de verdade.
    let mut moved = brush.clone();
    moved.alpha_offset = [brush.alpha_scale * 0.5, 0.0];
    let after = probe(&moved);
    let worst = neutral
        .iter()
        .zip(&after)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst > 0.05,
        "o deslocamento não move o carimbo (pior diferença {worst:.4}) — o \
         controle desenha e não faz nada"
    );

    // ⚠️ **E ele NÃO alcança um procedural**, porque um campo homogêneo não tem
    // posição. A neutralidade é do `alpha_frame`, não da row escondida: uma row
    // escondida com valor autorado agiria em silêncio e sem como desfazer.
    let mut field = moved.clone();
    field.alpha = Some(Alpha::Strata);
    let mut field_neutral = field.clone();
    field_neutral.alpha_offset = [0.0, 0.0];
    assert_eq!(
        probe(&field),
        probe(&field_neutral),
        "um deslocamento autorado com uma imagem vazou para um padrão procedural"
    );
}

/// **A PISTA DA SEMENTE É DE GRAÇA QUANDO ELA É ZERO** — `hash4(x, y, z, 0)` é
/// `hash3(x, y, z)` **AO BIT**.
///
/// ⚠️ **Não é promessa, é aritmética:** `0u32.wrapping_mul(k)` é `0` e `a ^ 0`
/// é `a`, então o termo novo desaparece da soma antes da avalanche. É isso que
/// deixa o [`crate::FilterKind::Random`] reusar o hash desta crate **sem mover
/// um pixel** de nenhum alpha procedural — e é a razão de o [`hash3`] delegar
/// em vez de os dois viverem lado a lado, onde divergiriam ao primeiro ajuste.
#[test]
fn the_fourth_lane_is_free_when_the_seed_is_zero() {
    for &(x, y, z) in &[
        (0, 0, 0),
        (1, -7, 913),
        (i32::MIN, i32::MAX, -1),
        (12345, 67890, -424242),
    ] {
        assert_eq!(
            hash4(x, y, z, 0),
            hash3(x, y, z),
            "a pista da semente cobrou em ({x}, {y}, {z})"
        );
    }
}

/// **E A SEMENTE MUDA O NÚMERO** — o controle sem o qual o gate acima é
/// satisfeito por um `hash4` que ignora o 4º argumento.
#[test]
fn a_seed_that_is_not_zero_changes_the_hash() {
    let base = hash4(3, 5, 8, 0);
    for w in [1, 2, -1, 7777] {
        assert_ne!(hash4(3, 5, 8, w), base, "a semente {w} nao mudou nada");
    }
}
