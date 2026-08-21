//! Os gates de **rodar, redimensionar e acumular** — a metade do gizmo que responde ao movimento.
//!
//! ⚠️ É um módulo-filho do arquivo de gates da projeção: `use super::*` traz as fixtures (`cam`,
//! `anchor`, `handles`, `of`, …), que continuam a existir **uma vez**. Duas cópias delas divergiriam
//! na primeira mudança de fixture, e os dois arquivos passariam a medir cenas diferentes com o mesmo
//! nome.

use super::*;

// ─────────────────────────────── RODAR ───────────────────────────────

/// ⭐ **Uma volta completa do cursor é uma volta completa da peça** — e é a afirmação que separa
/// esta lei da que a maioria dos editores usa.
///
/// ⚠️ Medir o ângulo **em pixels** em torno do centro projetado é o atalho comum, e ele mente fora
/// do eixo da vista: a projeção de um círculo é uma elipse, e o ângulo na elipse não é o ângulo no
/// círculo. O gesto ficaria rápido de um lado e lento do outro, e uma volta **não fecharia**. Aqui o
/// cursor é levado ao plano de rotação real.
///
/// O gate soma 36 passos de 10° **em torno da elipse projetada** e exige 2π — num eixo bem fora da
/// vista, onde a elipse é achatada e o atalho erraria de forma grosseira.
#[test]
fn a_full_turn_of_the_cursor_is_a_full_turn_of_the_part() {
    let c = Orbit::from_yaw_pitch(0.72, 0.52);
    let s = screen(&c);
    let a = anchor();
    let arm = ARM_PX / s.px_per_world();

    for n in 0..3 {
        if !of(&handles(&c, Mode::Rotate), Handle::Ring(n)).live {
            continue;
        }
        // Os pontos do cursor percorrem o CÍRCULO DO MUNDO, projetado — que na tela é uma elipse.
        let (u, v) = {
            let axis = [
                f32::from(u8::from(n == 0)),
                f32::from(u8::from(n == 1)),
                f32::from(u8::from(n == 2)),
            ];
            super::basis_of(axis)
        };
        let at = |t: f32| -> [f32; 2] {
            let (si, co) = t.sin_cos();
            let p = [
                a.origin[0] + (u[0] * co + v[0] * si) * arm,
                a.origin[1] + (u[1] * co + v[1] * si) * arm,
                a.origin[2] + (u[2] * co + v[2] * si) * arm,
            ];
            c.project(p, s).expect("a fixture olha a peça").0
        };
        const STEPS: usize = 36;
        let mut total = 0.0f32;
        for k in 0..STEPS {
            let t0 = k as f32 / STEPS as f32 * std::f32::consts::TAU;
            let t1 = (k + 1) as f32 / STEPS as f32 * std::f32::consts::TAU;
            total += angle_of(drag(Handle::Ring(n), a, &c, s, at(t0), at(t1)));
        }
        assert!(
            (total.abs() - std::f32::consts::TAU).abs() < 1e-3,
            "a argola {n} somou {total} numa volta inteira — esperava ±2π"
        );
    }
}

/// **O eixo do pedido é o eixo da argola**, e não um eixo qualquer perto dele.
#[test]
fn a_ring_spins_about_its_own_axis() {
    let c = cam();
    let s = screen(&c);
    for n in 0..3 {
        let Motion::Rotate { axis, .. } =
            drag(Handle::Ring(n), anchor(), &c, s, [40.0, 40.0], [60.0, 70.0])
        else {
            panic!("uma argola pede rotação");
        };
        for k in 0..3 {
            let want = if k == n { 1.0 } else { 0.0 };
            assert!(
                (axis[k].abs() - want).abs() < 1e-6,
                "a argola {n} pediu o eixo {axis:?}"
            );
        }
    }
}

/// ⚠️ **Uma argola de perfil não é uma alça** — e o número que a esconde é derivado
/// ([`RING_MIN_DOT`]).
///
/// Vista de perfil ela projeta-se numa reta, e o arrasto degenera junto: o plano de rotação fica
/// paralelo ao raio do cursor. A saída é a argola de VISTA, que não pode ficar de perfil consigo
/// mesma.
#[test]
fn a_ring_seen_edge_on_is_not_a_handle_and_the_view_ring_is() {
    // De frente, o X e o Y ficam de perfil (os eixos deles estão no plano da tela).
    let c = Orbit::from_yaw_pitch(0.0, 0.0);
    let hs = handles(&c, Mode::Rotate);
    assert!(!of(&hs, Handle::Ring(0)).live, "o X está de perfil");
    assert!(!of(&hs, Handle::Ring(1)).live, "o Y está de perfil");
    assert!(of(&hs, Handle::Ring(2)).live, "e o Z está de frente");
    assert!(
        of(&hs, Handle::ViewRing).live,
        "a argola de vista é a rede de segurança e não pode desaparecer"
    );
    assert_eq!(
        angle_of(drag(
            Handle::Ring(0),
            anchor(),
            &c,
            screen(&c),
            [10.0, 10.0],
            [200.0, 300.0]
        )),
        0.0,
        "uma argola que não se pode ver não pode rodar"
    );
}

/// ⚠️ **Só a metade da FRENTE de uma argola é desenhada, e ela é um trecho contíguo.**
///
/// O trecho pode dar a volta ao fim do vetor de amostras; cortar no índice zero partiria a argola em
/// duas no meio da tela. O gate mede a continuidade: nenhum salto entre pontos vizinhos pode ser
/// muito maior do que o passo típico.
#[test]
fn the_front_half_of_a_ring_is_one_unbroken_run() {
    let c = Orbit::from_yaw_pitch(0.9, 0.35);
    for h in handles(&c, Mode::Rotate) {
        let Shape::Arc(pts) = h.shape else {
            panic!("uma argola é um arco");
        };
        assert!(
            pts.len() > 4,
            "{:?} saiu com {} pontos",
            h.handle,
            pts.len()
        );
        let steps: Vec<f32> = pts
            .windows(2)
            .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
            .collect();
        let longest = steps.iter().copied().fold(0.0f32, f32::max);
        let typical = steps.iter().sum::<f32>() / steps.len() as f32;
        assert!(
            longest < typical * 4.0,
            "{:?} tem um salto de {longest} px contra um passo típico de {typical} — o arco partiu",
            h.handle
        );
    }
}

// ─────────────────────────────── TAMANHO ───────────────────────────────

/// ⭐ **Tamanho é RAZÃO, e por isso duas metades de um arrasto valem o produto.**
///
/// ⚠️ Somar diferenças em vez de multiplicar razões é o erro clássico, e ele só aparece com o rato
/// depressa: dois passos de ×1,1 dariam ×2,2 em vez de ×1,21. É a mesma lei do zoom de roda.
#[test]
fn size_is_a_ratio_so_two_halves_multiply() {
    let c = cam();
    let s = screen(&c);
    let (o2, _) = c
        .project(anchor().origin, s)
        .expect("a fixture olha a peça");
    let at = |r: f32| [o2[0] + r, o2[1]];

    let whole = factor_of(drag(Handle::Grip, anchor(), &c, s, at(90.0), at(180.0)));
    assert!(
        (whole - 2.0).abs() < 1e-4,
        "dobrar o raio é ×2, e deu {whole}"
    );

    let a = drag(Handle::Grip, anchor(), &c, s, at(90.0), at(135.0));
    let b = drag(Handle::Grip, anchor(), &c, s, at(135.0), at(180.0));
    let merged = factor_of(a.merge(b));
    assert!(
        (merged - whole).abs() < 1e-4,
        "as duas metades deram {merged} e o arrasto inteiro {whole} — a acumulação somou em vez de multiplicar"
    );
}

/// ⚠️ **Agarrar em cima do centro não manda a peça para o infinito.** O piso é do RAIO INICIAL, e é
/// por isso que ele existe: uma razão com denominador zero seria um salto num pixel.
#[test]
fn grabbing_at_the_centre_does_not_blow_the_part_up() {
    let c = cam();
    let s = screen(&c);
    let (o2, _) = c
        .project(anchor().origin, s)
        .expect("a fixture olha a peça");
    let f = factor_of(drag(
        Handle::Grip,
        anchor(),
        &c,
        s,
        o2,
        [o2[0] + 300.0, o2[1]],
    ));
    assert!((f - 1.0).abs() < f32::EPSILON, "esperava inerte e deu {f}");
}

/// ⛔ **O modo de tamanho tem UMA alça, e é uma decisão medida** (ADR-0161 §6: escala não-uniforme
/// destrói a propriedade de distância).
///
/// ⚠️ O gate existe para o dia em que alguém "completar" o gizmo com três caixas por eixo, como as
/// do Blender. Elas seriam três controles a prometer o que o modelo não entrega — e o artista
/// concluiria que o app tem um bug que ele não tem.
#[test]
fn the_size_mode_offers_one_handle_because_the_model_is_uniform() {
    let c = cam();
    let hs = handles(&c, Mode::Scale);
    assert_eq!(hs.len(), 1, "as alças de tamanho são: {hs:?}");
    assert_eq!(hs[0].handle, Handle::Grip);
    assert!(hs[0].live, "um punho de TELA não tem como degenerar");
}

// ─────────────────────────────── OS MODOS ───────────────────────────────

/// **Cada modo oferece o seu conjunto de alças, e nenhuma do outro.**
///
/// ⚠️ Sem isto, uma alça sobrevivente do modo anterior ficaria apontável sem ser pintada — o pior
/// dos dois mundos, porque o clique faz uma coisa que nada na tela anunciou.
#[test]
fn each_mode_offers_its_own_handles_and_none_of_the_others() {
    let c = cam();
    let kinds =
        |mode: Mode| -> Vec<Handle> { handles(&c, mode).into_iter().map(|h| h.handle).collect() };

    let mv = kinds(Mode::Move);
    assert!(mv.contains(&Handle::View) && mv.contains(&Handle::Axis(0)));
    assert!(
        !mv.iter()
            .any(|h| matches!(h, Handle::Ring(_) | Handle::ViewRing | Handle::Grip))
    );

    let rot = kinds(Mode::Rotate);
    assert!(rot.contains(&Handle::ViewRing) && rot.contains(&Handle::Ring(1)));
    assert!(!rot.iter().any(|h| matches!(
        h,
        Handle::Axis(_) | Handle::Plane(_) | Handle::View | Handle::Grip
    )));

    assert_eq!(kinds(Mode::Scale), vec![Handle::Grip]);
}

/// **Todo verbo tem um rótulo que traduz** — nenhum vaza a chave crua na tela.
///
/// ⚠️ O `tr` da casa devolve a própria chave quando não conhece uma (de propósito: o identificador
/// feio é o alarme), então "traduziu" mede-se por *"o que voltou é diferente da chave"*.
#[test]
fn every_mode_has_a_translation() {
    for m in Mode::ALL {
        assert_ne!(
            ph2d_i18n::tr(m.key()),
            m.key(),
            "o verbo {m:?} não está na tabela de i18n"
        );
    }
}

/// ⚠️ **Verbos diferentes não se somam.** Não pode acontecer num arrasto (a alça fixa o verbo), e
/// inventar uma soma entre um giro e uma escala seria pior do que o segundo ganhar.
#[test]
fn merging_two_different_verbs_keeps_the_second() {
    let t = Motion::Translate([1.0, 0.0, 0.0]);
    let r = Motion::Rotate {
        axis: [0.0, 1.0, 0.0],
        angle: 0.5,
    };
    assert_eq!(t.merge(r), r);
    assert_eq!(r.merge(t), t);
}

// ─────────────────────────────── OS EIXOS ───────────────────────────────

/// ⭐ **`Local` roda com o objeto; `Global` não.**
#[test]
fn local_axes_follow_the_node_and_global_ones_do_not() {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    // Um quarto de volta em torno de Z: o X do objeto passa a apontar para o +Y do mundo.
    let quarter = [0.0, 0.0, s, s];

    let g = Frame::Global.axes(quarter);
    assert_eq!(g[0], [1.0, 0.0, 0.0], "o global não conhece o objeto");

    let l = Frame::Local.axes(quarter);
    assert!(
        (l[0][0]).abs() < 1e-6 && (l[0][1] - 1.0).abs() < 1e-6,
        "o X local devia apontar para o +Y do mundo e aponta para {:?}",
        l[0]
    );
    // E continuam ortonormais — senão o arrasto num eixo escorregaria para outro.
    for a in l {
        assert!((ph2d_field::xform::dot(a, a) - 1.0).abs() < 1e-5);
    }
    for (i, j) in [(0, 1), (1, 2), (0, 2)] {
        assert!(ph2d_field::xform::dot(l[i], l[j]).abs() < 1e-5);
    }
}

/// ⭐ **A lei do gizmo lê os eixos da ÂNCORA, e nunca os do mundo por baixo do pano.**
///
/// ⚠️ É o gate que impede a escolha de referencial de existir só no painel. Com uma âncora de eixos
/// rodados, arrastar a seta 0 tem de mover **naquela direção** — se sair no `+X` do mundo, algum
/// sítio da lei ficou a ler `WORLD_AXES` e o seletor é decorativo.
#[test]
fn the_law_moves_along_the_anchors_axes_not_the_worlds() {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let rotated = Anchor {
        entity: 1,
        origin: [0.0; 3],
        axes: Frame::Local.axes([0.0, 0.0, s, s]),
    };
    let c = cam();
    let sc = screen(&c);
    let hs = project(rotated, &c, sc, Mode::Move);
    let Shape::Arrow { from, to } = of(&hs, Handle::Axis(0)).shape else {
        panic!("um eixo é uma seta");
    };
    let d = [to[0] - from[0], to[1] - from[1]];
    let len = d[0].hypot(d[1]);
    let delta = translation(drag(
        Handle::Axis(0),
        rotated,
        &c,
        sc,
        [0.0, 0.0],
        [d[0] / len * 40.0, d[1] / len * 40.0],
    ));
    assert!(
        delta[1] > 0.0 && delta[0].abs() < 1e-5 && delta[2].abs() < 1e-5,
        "a seta 0 de um objeto rodado tem de andar no +Y do mundo, e andou {delta:?}"
    );
}

/// **Todo referencial tem um rótulo que traduz.**
#[test]
fn every_axis_frame_has_a_translation() {
    for f in Frame::ALL {
        assert_ne!(ph2d_i18n::tr(f.key()), f.key(), "o eixo {f:?} não traduz");
    }
}

// ─────────────────────────── PRESO À GRELHA ───────────────────────────

/// ⭐ **`since` é a inversa exacta de `merge`** — a identidade que faz o total-desde-a-pegada
/// funcionar.
///
/// ⚠️ Sem ela o arrasto aplicaria a mais ou a menos: o que se manda ao mundo é
/// `total.since(applied)`, e o que o mundo passa a ter é `applied.merge(isso)`. Se as duas não
/// forem inversas, cada evento de ponteiro deixa um resíduo — e o resíduo acumula, então o defeito
/// cresce com a duração do gesto.
#[test]
fn since_is_the_exact_inverse_of_merge() {
    let cases = [
        (
            Motion::Translate([0.7, -0.2, 0.35]),
            Motion::Translate([0.3, 0.1, -0.05]),
        ),
        (
            Motion::Rotate {
                axis: [0.0, 1.0, 0.0],
                angle: 1.1,
            },
            Motion::Rotate {
                axis: [0.0, 1.0, 0.0],
                angle: 0.4,
            },
        ),
        (Motion::Scale(2.5), Motion::Scale(1.25)),
    ];
    for (total, applied) in cases {
        let back = applied.merge(total.since(applied));
        match (total, back) {
            (Motion::Translate(a), Motion::Translate(b)) => {
                for k in 0..3 {
                    assert!((a[k] - b[k]).abs() < 1e-5, "{a:?} != {b:?}");
                }
            }
            (Motion::Rotate { angle: a, .. }, Motion::Rotate { angle: b, .. }) => {
                assert!((a - b).abs() < 1e-5, "{a} != {b}");
            }
            (Motion::Scale(a), Motion::Scale(b)) => assert!((a - b).abs() < 1e-5, "{a} != {b}"),
            other => panic!("as variantes trocaram: {other:?}"),
        }
    }
}

/// ⭐ **Preso à grelha, o total pousa EXATAMENTE num degrau.**
///
/// ⚠️ E é o TOTAL que se prende, não cada incremento: prender incrementos e somá-los acumula o erro
/// de cada arredondamento, e o gesto acaba fora da grelha depois de uns segundos — com a ficha a
/// mostrar um número redondo que a peça não tem.
#[test]
fn a_snapped_total_lands_exactly_on_a_step() {
    let step = 0.05f32;
    let Motion::Translate(d) = Motion::Translate([0.237, -0.081, 0.0]).snapped(step) else {
        panic!("continua translação");
    };
    for v in d {
        let k = (v / step).round();
        assert!((v - k * step).abs() < 1e-6, "{v} não é múltiplo de {step}");
    }
    assert!(
        (d[0] - 0.25).abs() < 1e-6,
        "0,237 arredonda a 0,25 e deu {}",
        d[0]
    );

    // O ângulo prende a 15°, que é o maior passo que ainda contém 30, 45, 60 e 90.
    let a = angle_of(
        Motion::Rotate {
            axis: [0.0, 0.0, 1.0],
            angle: 0.80,
        }
        .snapped(step),
    );
    assert!(
        (a.to_degrees() - 45.0).abs() < 1e-3,
        "0,80 rad (45,8°) devia prender a 45° e deu {}°",
        a.to_degrees()
    );

    // O fator prende ao que a ficha consegue exprimir.
    assert!((factor_of(Motion::Scale(1.47).snapped(step)) - 1.5).abs() < 1e-6);
    // ⛔ E nunca a zero ou a negativo: uma escala nula faria o campo deixar de ser uma distância.
    assert!(factor_of(Motion::Scale(0.01).snapped(step)) > 0.0);
}

/// ⭐ **O passo da grelha é DERIVADO do enquadramento**, e a condição que o fixa é concreta: dois
/// degraus vizinhos têm de estar mais afastados do que a tolerância do ponteiro.
///
/// ⚠️ Um passo fixo em unidades de mundo é inútil nos dois extremos — aproximado, dois pontos da
/// grelha ficam a meia tela; afastado, ficam dentro do mesmo pixel.
#[test]
fn the_grid_step_is_derived_from_the_framing() {
    let mut last = 0.0f32;
    for zoom in [0.02f32, 0.1, 0.8, 3.5] {
        let s = Screen::new(W, H, zoom);
        let step = snap_step(s);
        assert!(
            step * s.px_per_world() >= GRAB_PX,
            "com half_extent {zoom} o passo {step} mede {} px — abaixo da tolerância do ponteiro",
            step * s.px_per_world()
        );
        // E é um degrau REDONDO da escada 1-2-5.
        let m = step / 10f32.powf(step.log10().floor());
        assert!(
            [1.0f32, 2.0, 5.0].iter().any(|k| (m - k).abs() < 1e-3),
            "com half_extent {zoom} o passo {step} não é 1, 2 nem 5 vezes uma potência de dez"
        );
        assert!(step >= last, "aproximar não pode ENGROSSAR a grelha");
        last = step;
    }
}
