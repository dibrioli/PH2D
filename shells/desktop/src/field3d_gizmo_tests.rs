//! Os gates do gizmo 3D.
//!
//! ⚠️ **Nenhum deles abre uma janela**, e é isso que os torna escrevíveis. A lei é pura: entram uma
//! âncora e uma câmera, sai onde as alças ficam e quanto o arrasto vale. O que um smoke ainda tem de
//! dizer é se o *feel* está certo; o que estes dizem é que o gesto **faz o que promete**.

use super::*;
use ph2d_field_render::{Orbit, Screen};

const W: u32 = 800;
const H: u32 = 600;

fn cam() -> Orbit {
    Orbit::default()
}

fn screen(c: &Orbit) -> Screen {
    Screen::new(W, H, c.half_extent)
}

fn anchor() -> Anchor {
    Anchor::global(1, [0.0, 0.0, 0.0])
}

fn handles(c: &Orbit, mode: Mode) -> Vec<Projected> {
    project(anchor(), c, screen(c), mode)
}

fn of(hs: &[Projected], want: Handle) -> Projected {
    hs.iter()
        .find(|h| h.handle == want)
        .unwrap_or_else(|| panic!("o gizmo não projetou {want:?}"))
        .clone()
}

fn arrow_of(hs: &[Projected], n: usize) -> ([f32; 2], [f32; 2]) {
    match of(hs, Handle::Axis(n)).shape {
        Shape::Arrow { from, to } => (from, to),
        other => panic!("um eixo é uma seta, e veio {other:?}"),
    }
}

fn translation(m: Motion) -> [f32; 3] {
    match m {
        Motion::Translate(d) => d,
        other => panic!("esperava uma translação e veio {other:?}"),
    }
}

fn angle_of(m: Motion) -> f32 {
    match m {
        Motion::Rotate { angle, .. } => angle,
        other => panic!("esperava uma rotação e veio {other:?}"),
    }
}

fn factor_of(m: Motion) -> f32 {
    match m {
        Motion::Scale(f) => f,
        other => panic!("esperava uma escala e veio {other:?}"),
    }
}

// ─────────────────────────────── MOVER ───────────────────────────────

/// ⭐ **O gizmo tem tamanho de TELA**, como o do Blender — o braço mede o mesmo em qualquer zoom.
///
/// ⚠️ Um gizmo de tamanho de mundo fica maior do que a janela ao aproximar e desaparece ao afastar,
/// e é a mesma peça que se está a manipular nos dois casos. A prova é sobre o eixo perpendicular à
/// vista: os outros dois encurtam por projeção, e é suposto.
#[test]
fn the_arm_is_the_same_length_on_screen_at_every_zoom() {
    // Vista de frente: o X e o Y ficam no plano da tela e projetam o braço inteiro.
    let mut c = Orbit::from_yaw_pitch(0.0, 0.0);
    for zoom in [0.05f32, 0.8, 3.5] {
        c.half_extent = zoom;
        let hs = handles(&c, Mode::Move);
        for axis in [0usize, 1] {
            let (from, to) = arrow_of(&hs, axis);
            let len = (to[0] - from[0]).hypot(to[1] - from[1]);
            assert!(
                (len - ARM_PX).abs() < 0.5,
                "com half_extent {zoom} o braço {axis} mediu {len} px em vez de {ARM_PX}"
            );
        }
    }
}

/// ⭐ **Arrastar uma seta move o nó ao longo daquele eixo, e mais nada.**
#[test]
fn dragging_an_axis_moves_along_that_axis_only() {
    let c = cam();
    let s = screen(&c);
    let hs = handles(&c, Mode::Move);
    for n in 0..3 {
        let (from, to) = arrow_of(&hs, n);
        // Arrasta 40 px NA DIREÇÃO da seta, na tela.
        let d = [to[0] - from[0], to[1] - from[1]];
        let len = d[0].hypot(d[1]);
        let m = [d[0] / len * 40.0, d[1] / len * 40.0];
        let delta = translation(drag(Handle::Axis(n), anchor(), &c, s, [0.0, 0.0], m));

        for k in 0..3 {
            if k == n {
                assert!(
                    delta[k] > 0.0,
                    "arrastar na direção da seta {n} tem de andar para a frente nela, e deu {delta:?}"
                );
            } else {
                assert!(
                    delta[k].abs() < 1e-6,
                    "o eixo {n} escorregou para o {k}: {delta:?}"
                );
            }
        }
    }
}

/// ⭐ **O número do arrasto é o que a tela mostra**: mover o rato o comprimento projetado do braço
/// anda exatamente um braço no mundo.
///
/// ⚠️ É a afirmação que separa "move na direção certa" de "move a quantidade certa". Um fator errado
/// aqui passa despercebido num gate de direção e é o que se sente como *"a peça foge da mão"*.
#[test]
fn one_arm_of_mouse_is_one_arm_of_world() {
    let c = cam();
    let s = screen(&c);
    let arm_world = ARM_PX / s.px_per_world();
    let (from, to) = arrow_of(&handles(&c, Mode::Move), 0);
    let m = [to[0] - from[0], to[1] - from[1]];
    let delta = translation(drag(Handle::Axis(0), anchor(), &c, s, [0.0, 0.0], m));
    assert!(
        (delta[0] - arm_world).abs() < arm_world * 1e-3,
        "o braço mede {arm_world} de mundo e o arrasto andou {}",
        delta[0]
    );
}

/// ⚠️ **Uma seta apontada ao observador não é uma alça** — e o gate mede as duas metades: ela não é
/// pintada, e arrastá-la não faz nada.
///
/// Sem isto, um pixel de rato valeria um salto arbitrário: a conta divide pelo comprimento
/// projetado, que ali tende a zero. O sintoma seria a peça a desaparecer da janela num toque.
#[test]
fn an_axis_that_points_at_the_camera_is_not_a_handle() {
    // De frente: o eixo Z aponta ao observador e projeta-se em nada.
    let c = Orbit::from_yaw_pitch(0.0, 0.0);
    let hs = handles(&c, Mode::Move);

    assert!(
        !of(&hs, Handle::Axis(2)).live,
        "o eixo Z está de frente para a câmera e continua pintado"
    );
    assert_eq!(
        translation(drag(
            Handle::Axis(2),
            anchor(),
            &c,
            screen(&c),
            [0.0, 0.0],
            [500.0, 500.0]
        )),
        [0.0; 3],
        "uma alça que não se pode ver não pode arrastar"
    );
    // ⭐ E o gesto não fica sem saída: o quadrado do plano perpendicular a ela está de FRENTE, que é
    // exatamente o que aquele enquadramento pede.
    assert!(of(&hs, Handle::Plane(2)).live);
}

/// **O centro é do disco de vista**, e não de um eixo à sorte.
///
/// ⚠️ A folga central existe para isto: sem ela as três hastes disputariam o mesmo pixel e quem
/// ganhasse dependia da ordem da lista, não da geometria.
#[test]
fn the_centre_belongs_to_the_view_disc() {
    let c = cam();
    let hs = handles(&c, Mode::Move);
    let (o2, _) = c.project(anchor().origin, screen(&c));
    assert_eq!(pick(&hs, o2), Some(Handle::View));
    // E um pixel logo ao lado do centro também: a folga tem raio, não é um ponto.
    assert_eq!(
        pick(&hs, [o2[0] + INNER_PX * 0.5, o2[1]]),
        Some(Handle::View)
    );
}

/// **Apontar o meio de uma haste escolhe aquela seta**, e o vazio não escolhe nada.
#[test]
fn pointing_at_a_shaft_picks_that_axis_and_empty_space_picks_nothing() {
    let c = cam();
    let hs = handles(&c, Mode::Move);
    for n in 0..3 {
        if !of(&hs, Handle::Axis(n)).live {
            continue;
        }
        let (from, to) = arrow_of(&hs, n);
        let mid = [(from[0] + to[0]) * 0.5, (from[1] + to[1]) * 0.5];
        assert_eq!(
            pick(&hs, mid),
            Some(Handle::Axis(n)),
            "o meio da haste {n} tem de ser dela"
        );
    }
    let (o2, _) = c.project(anchor().origin, screen(&c));
    assert_eq!(
        pick(&hs, [o2[0] + ARM_PX * 3.0, o2[1] + ARM_PX * 3.0]),
        None,
        "longe do gizmo não é de ninguém — senão o clique de selecionar viraria um arrasto"
    );
}

/// ⭐ **Um quadrado de plano move NO plano dele e nunca ao longo da normal.**
#[test]
fn a_plane_handle_never_moves_along_its_normal() {
    let c = cam();
    let s = screen(&c);
    let hs = handles(&c, Mode::Move);
    for n in 0..3 {
        if !of(&hs, Handle::Plane(n)).live {
            continue;
        }
        let delta = translation(drag(
            Handle::Plane(n),
            anchor(),
            &c,
            s,
            [10.0, 10.0],
            [90.0, 130.0],
        ));
        assert!(
            delta[n].abs() < 1e-4,
            "o plano perpendicular a {n} andou {} ao longo da própria normal",
            delta[n]
        );
        let in_plane = delta[(n + 1) % 3].abs() + delta[(n + 2) % 3].abs();
        assert!(in_plane > 1e-3, "e ele tem de andar de facto: {delta:?}");
    }
}

/// **O disco de vista nunca degenera** — é a rede de segurança de todo enquadramento.
#[test]
fn the_view_handle_works_from_every_angle() {
    for (yaw, pitch) in [(0.0, 0.0), (0.72, 0.52), (2.1, -1.4), (0.0, 1.5)] {
        let c = Orbit::from_yaw_pitch(yaw, pitch);
        assert!(of(&handles(&c, Mode::Move), Handle::View).live);
        let d = translation(drag(
            Handle::View,
            anchor(),
            &c,
            screen(&c),
            [0.0, 0.0],
            [30.0, -20.0],
        ));
        let n = d[0].abs() + d[1].abs() + d[2].abs();
        assert!(
            n > 1e-3 && n.is_finite(),
            "de ({yaw}, {pitch}) o disco de vista não moveu nada: {d:?}"
        );
    }
}

/// ⚠️ **O quadrilátero é testado por produto vetorial**, e não por caixa alinhada: um quadrado do
/// mundo projeta-se como um losango, e uma caixa reclamaria pixels do vizinho — exatamente nos
/// cantos onde as três alças de plano se tocam.
#[test]
fn a_projected_plane_is_a_rhombus_not_a_box() {
    let c = cam();
    let hs = handles(&c, Mode::Move);
    let h = of(&hs, Handle::Plane(2));
    let Shape::Quad(q) = h.shape else {
        panic!("um plano é um quadrilátero");
    };
    assert!(h.live);
    let mid = [(q[0][0] + q[2][0]) * 0.5, (q[0][1] + q[2][1]) * 0.5];
    assert_eq!(pick(&hs, mid), Some(Handle::Plane(2)));

    // E um canto da caixa envolvente que está FORA do losango não é dele. Se não houver nenhum, o
    // losango era um retângulo alinhado e o gate não teria nada a dizer — então isso é reprovação.
    let xs = q.iter().map(|p| p[0]);
    let ys = q.iter().map(|p| p[1]);
    let (x0, x1) = (
        xs.clone().fold(f32::INFINITY, f32::min),
        xs.fold(f32::NEG_INFINITY, f32::max),
    );
    let (y0, y1) = (
        ys.clone().fold(f32::INFINITY, f32::min),
        ys.fold(f32::NEG_INFINITY, f32::max),
    );
    let outside = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
        .into_iter()
        .filter(|c| pick(&hs, *c) != Some(Handle::Plane(2)))
        .count();
    assert!(
        outside > 0,
        "nenhum canto da caixa envolvente ficou de fora — o teste de losango não está a ser exercido"
    );
}

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
            c.project(p, s).0
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
    let (o2, _) = c.project(anchor().origin, s);
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
    let (o2, _) = c.project(anchor().origin, s);
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
