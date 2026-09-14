//! Os gates do [`super`].

use super::*;

/// ⭐ **O `π` da luz-objecto é o MESMO da lâmpada de estúdio** — uma luz branca de força `1` a UMA
/// unidade entrega o que a lâmpada do rig entregava.
///
/// ⛔ Sem isto, a escala de intensidade do modelador seria uma segunda escala, e o artista teria de
/// aprender duas.
#[test]
fn a_light_of_one_at_one_unit_is_the_lamp_the_rig_had() {
    let l = ph2d_field_ecs::FieldLight::default();
    let do_rig = crate::render_light::lamps(&ph2d_light::LightRig::default());
    assert_eq!(do_rig.len(), 1, "o rig de omissão tem UMA lâmpada acesa");
    for c in 0..3 {
        assert!(
            (radiance_at_one(l)[c] - do_rig[0].radiance[c]).abs() < 1.0e-6,
            "canal {c}: {:?} contra {:?}",
            radiance_at_one(l),
            do_rig[0].radiance
        );
    }
}

/// ⚠️ **Força e cor negativas não escurecem a peça** — elas são coadas, porque a subtracção de luz
/// não existe e o que ela produziria é um pixel `NaN` a jusante.
#[test]
fn a_negative_light_is_no_light_never_a_dark_one() {
    let mau = ph2d_field_ecs::FieldLight {
        intensity: -5.0,
        color: [-1.0, 0.5, -0.2],
    };
    assert_eq!(radiance_at_one(mau), [0.0; 3]);
}

/// ⭐⭐⭐ **O SÍTIO DA PRIMEIRA LUZ É A DIRECÇÃO DO RIG QUE ELA SUBSTITUI** — e sai da porta, nunca de
/// um literal.
///
/// ⚠️ **A primeira redacção era um `[f32; 3]` const em MUNDO**, e isso é a mesma classe de erro que o
/// sinal de `y` desta casa já pagou: a direcção do rig é de **ECRÃ**, e onde «superior-esquerda» cai
/// no mundo depende de para onde a câmera olha.
#[test]
fn the_first_light_is_born_where_the_rig_lamp_shone_from() {
    let cam = ph2d_field_render::Orbit::default();
    let p = opening_place(&cam);
    let (right, up, toward_eye) = cam.basis();
    let d = [0, 1, 2].map(|i| p[i] - cam.target[i]);
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    // ⚠️ **Normalizada**: o que se compara é a DIRECÇÃO — a distância é a
    // [`opening_distance`], e ela tem lei própria.
    let n = dot(d, d).sqrt();
    assert!(
        (n - opening_distance(&cam)).abs() < 1.0e-5,
        "a luz nasceu a {n} e a lei pede {}",
        opening_distance(&cam)
    );
    let em_vista = [dot(d, right) / n, dot(d, up) / n, dot(d, toward_eye) / n];
    let lampada = crate::render_light::lamps(&ph2d_light::LightRig::default())[0].to_light;
    for c in 0..3 {
        assert!(
            (em_vista[c] - lampada[c]).abs() < 1.0e-5,
            "a luz nasce em {em_vista:?} e a lâmpada do rig vinha de {lampada:?}"
        );
    }
    // ⭐⭐ **E a FORÇA é `r²`** — é ela que faz a peça receber, no centro, o que a lâmpada do rig
    // lhe dava. *Uma luz num sítio novo com a força velha seria uma cena que escurece sozinha.*
    let (_, lampada_nova) = opening_light(&cam);
    let r = opening_distance(&cam);
    assert!(
        (lampada_nova.intensity - r * r).abs() < 1.0e-5,
        "a força é {} e a lei pede {}",
        lampada_nova.intensity,
        r * r
    );
    // ⭐ **E com uma câmera OUTRA, o mundo muda e a vista não** — é isto que um literal não faz.
    let mut outra = ph2d_field_render::Orbit::from_yaw_pitch(1.1, -0.4);
    outra.target = [0.3, -0.2, 0.7];
    let q = opening_place(&outra);
    assert!(
        (0..3).any(|i| (q[i] - p[i]).abs() > 1.0e-3),
        "o sítio não seguiu a câmera: {q:?} contra {p:?}"
    );
}

/// ⭐⭐⭐ **O PAINEL DE UMA LUZ OFERECE EXACTAMENTE O QUE ELA FAZ** — a lei W34, sobre o objecto novo.
///
/// ⛔ **A rotação e a escala NÃO são oferecidas**, e isso é a metade que interessa: um ponto não tem
/// orientação nem tamanho, e três sliders de ângulo que não movem um pixel são o controlo morto que
/// aquela lei proíbe por escrito.
#[test]
fn the_panel_of_a_light_offers_no_angle_and_no_size() {
    let mut world = bevy_ecs::world::World::new();
    let luz = ph2d_field_ecs::add_light(
        &mut world,
        [1.0, 2.0, 3.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    let linhas = ph2d_field_ecs::params_of(&world, luz);
    let chaves: Vec<ph2d_field::Param> = linhas.iter().map(|(p, _)| *p).collect();
    assert_eq!(
        chaves,
        vec![
            ph2d_field::Param::Pos(0),
            ph2d_field::Param::Pos(1),
            ph2d_field::Param::Pos(2),
            ph2d_field::Param::Light(0),
            ph2d_field::Param::Light(1),
            ph2d_field::Param::Light(2),
            ph2d_field::Param::Light(3),
        ],
        "as linhas de uma luz mudaram"
    );
    // ⭐ **A POSIÇÃO é a da entidade**, e não uma cópia: o que o painel mostra é a pose.
    assert!((linhas[0].1.value - 1.0).abs() < 1.0e-6);
    assert!((linhas[2].1.value - 3.0).abs() < 1.0e-6);
    // ⭐ **E o gesto de a mover é o MESMO** — a porta que move uma forma move uma lâmpada.
    ph2d_field_ecs::set_param(&mut world, luz, ph2d_field::Param::Pos(0), -4.0).expect("mover");
    assert_eq!(of_the_world(&mut world)[0].world[0], -4.0);
}

/// ⭐⭐⭐ **UMA LUZ-OBJECTO ACENDE A PEÇA, E DO LADO EM QUE ELA ESTÁ.**
///
/// # ⚠️ As duas metades, e porque nenhuma chega sozinha
///
/// *«Ficou mais claro»* é satisfeito por qualquer lâmpada em qualquer sítio — inclusive por uma que
/// ignorasse a posição. ⇒ a segunda metade move a luz para o **outro lado** e exige que o lado
/// aceso troque. *Sem ela, a conversão mundo→vista podia estar errada e o gate ficava verde.*
#[test]
fn a_light_object_lights_the_side_it_is_on() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    let (w, h) = (128_u32, 128);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let so = [ph2d_material::OpenPbr::default().prepare()];
    let surface = ph2d_field_render::Surfaces {
        all: &so,
        owners: None,
    };
    let (right, up, toward_eye) = cam.basis();
    // Uma luz a uma unidade, na direcção de vista pedida.
    let em = |v: [f32; 3]| -> Vec<ph2d_field_render::PointLamp> {
        vec![ph2d_field_render::PointLamp {
            world: [0, 1, 2]
                .map(|i| cam.target[i] + v[0] * right[i] + v[1] * up[i] + v[2] * toward_eye[i]),
            radiance_at_one: radiance_at_one(ph2d_field_ecs::FieldLight::default()),
        }]
    };
    let pinta = |points: &[ph2d_field_render::PointLamp]| {
        shade_render(
            &g,
            &cam,
            &surface,
            &Lighting {
                lamps: &[],
                points,
                sky: &crate::render_light::StudioSky,
            },
            crate::shading::OPENING_LOOK,
            [0, 0, 0, 0],
        )
    };
    // A média do verde na METADE esquerda e na direita da peça.
    let lados = |px: &[u8]| -> (f64, f64) {
        let c = px.as_chunks::<4>().0;
        let (mut e, mut ne, mut d, mut nd) = (0.0_f64, 0_usize, 0.0_f64, 0_usize);
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = y * w as usize + x;
                if !g.hit[i] {
                    continue;
                }
                if x < w as usize / 2 {
                    e += f64::from(c[i][1]);
                    ne += 1;
                } else {
                    d += f64::from(c[i][1]);
                    nd += 1;
                }
            }
        }
        (e / ne as f64, d / nd as f64)
    };

    let sem = pinta(&[]);
    let esquerda = em([-0.7, 0.3, 0.65]);
    let direita = em([0.7, 0.3, 0.65]);
    let (se, sd) = lados(&sem);
    let (ee, ed) = lados(&pinta(&esquerda));
    let (de, dd) = lados(&pinta(&direita));
    println!("sem: {se:.1}/{sd:.1} · esquerda: {ee:.1}/{ed:.1} · direita: {de:.1}/{dd:.1}");

    // (a) Ela ACENDE.
    assert!(
        ee > se + 5.0 && ed > sd + 1.0,
        "a luz não acendeu a peça: sem {se:.1}/{sd:.1}, com {ee:.1}/{ed:.1}"
    );
    // (b) E do LADO em que está — nos dois sentidos.
    assert!(
        ee > ed,
        "uma luz à esquerda acendeu mais o lado direito ({ee:.1} contra {ed:.1})"
    );
    assert!(
        dd > de,
        "uma luz à direita acendeu mais o lado esquerdo ({dd:.1} contra {de:.1})"
    );
}

/// ⭐⭐ **O OLHO DA HIERARQUIA APAGA A LUZ** — e de graça, porque é o mesmo componente que esconde uma
/// forma. *Uma luz que só se desliga apagando-a é uma luz que ninguém experimenta.*
#[test]
fn the_hierarchy_eye_switches_a_light_off() {
    let mut world = bevy_ecs::world::World::new();
    let luz = ph2d_field_ecs::add_light(
        &mut world,
        [1.0, 1.0, 1.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    assert_eq!(of_the_world(&mut world).len(), 1);
    world
        .entity_mut(luz)
        .insert(ph2d_ecs::Visibility { hidden: true });
    assert!(
        of_the_world(&mut world).is_empty(),
        "a luz escondida continuou a acender"
    );
    // ⭐ E o controlo: sem o `hidden`, ela volta.
    world
        .entity_mut(luz)
        .insert(ph2d_ecs::Visibility { hidden: false });
    assert_eq!(of_the_world(&mut world).len(), 1);
}

/// ⚠️ **A ordem das luzes é ESTÁVEL** — a soma de `f32` não é associativa, e sem isto o mesmo
/// documento daria duas imagens conforme a ordem em que o ECS calhou de arrumar os arquétipos.
#[test]
fn the_lights_come_out_in_a_stable_order() {
    let mut world = bevy_ecs::world::World::new();
    let a = ph2d_field_ecs::add_light(
        &mut world,
        [1.0, 0.0, 0.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    let b = ph2d_field_ecs::add_light(
        &mut world,
        [2.0, 0.0, 0.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    let _ = ph2d_field_ecs::add_light(
        &mut world,
        [3.0, 0.0, 0.0],
        ph2d_field_ecs::FieldLight::default(),
    );
    let antes: Vec<f32> = of_the_world(&mut world)
        .iter()
        .map(|l| l.world[0])
        .collect();
    assert_eq!(antes.len(), 3);
    // ⚠️ **A perturbação é a que importa:** inserir um componente MUDA o arquétipo da entidade, e é
    // isso que reordena uma varredura de ECS. *Sem esta metade o gate só media que três é três.*
    world.entity_mut(b).insert(ph2d_ecs::Name::new("Zzz"));
    world
        .entity_mut(a)
        .insert(ph2d_ecs::Visibility { hidden: false });
    let depois: Vec<f32> = of_the_world(&mut world)
        .iter()
        .map(|l| l.world[0])
        .collect();
    assert_eq!(
        antes, depois,
        "a ordem mudou ao tocar nas luzes — a soma de `f32` não é associativa, logo isto são duas \
         imagens do mesmo documento"
    );
    // ⛔ **E ela NÃO é a ordem de criação**, que é o que se leria por engano: os bits de uma
    // entidade não crescem com o `spawn`. O que a lei promete é ser a **mesma** todas as vezes.
    assert_ne!(
        antes,
        vec![1.0, 2.0, 3.0],
        "a ordem calhou de ser a de criação: este assert existe para o leitor seguinte não a \
         confundir com a lei — se ela passar a sê-lo, o comentário acima é que está errado"
    );
}

/// ⏱️ **SONDA — o que a troca CUSTA À IMAGEM.** A lâmpada ancorada no ecrã saiu do modo Render e
/// entrou uma luz-objecto no sítio dela; esta sonda mede quanto a peça se mexeu.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_the_swap_costs_the_picture() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    let (w, h) = (640_u32, 360);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let so = [ph2d_material::OpenPbr::default().prepare()];
    let surface = ph2d_field_render::Surfaces {
        all: &so,
        owners: None,
    };
    let pinta = |lamps: &[ph2d_field_render::Lamp], points: &[ph2d_field_render::PointLamp]| {
        shade_render(
            &g,
            &cam,
            &surface,
            &Lighting {
                lamps,
                points,
                sky: &crate::render_light::StudioSky,
            },
            crate::shading::OPENING_LOOK,
            [0, 0, 0, 0],
        )
    };
    let stats = |px: &[u8]| -> (f64, u32) {
        let (mut s, mut n, mut b) = (0.0_f64, 0_usize, 0_u32);
        for (i, p) in px.as_chunks::<4>().0.iter().enumerate() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            s += f64::from(p[1]);
            b += u32::from(p[0] == 255 && p[1] == 255 && p[2] == 255);
        }
        (s / n as f64, b)
    };
    let delta = |a: &[u8], b: &[u8]| -> (f64, u32) {
        let (ca, cb) = (a.as_chunks::<4>().0, b.as_chunks::<4>().0);
        let (mut s, mut mx, mut n) = (0.0_f64, 0_u32, 0_usize);
        for i in 0..ca.len() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            for c in 0..3 {
                let d = u32::from(ca[i][c].abs_diff(cb[i][c]));
                s += f64::from(d);
                mx = mx.max(d);
            }
        }
        (s / (3 * n) as f64, mx)
    };

    let antes = pinta(
        &crate::render_light::lamps(&ph2d_light::LightRig::default()),
        &[],
    );
    let (onde, lampada) = opening_light(&cam);
    let uma = vec![ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: radiance_at_one(lampada),
    }];
    let depois = pinta(&[], &uma);
    let so_ceu = pinta(&[], &[]);
    println!("esfera {w}x{h}, olhar do produto\n");
    println!("                                  · média · branco · |Δ| médio · máx");
    let (m0, b0) = stats(&antes);
    println!("a lâmpada de ECRÃ (o de antes)    · {m0:5.1} · {b0:6} ·        — ·   —");
    for (nome, px) in [
        ("a luz de ABERTURA          ", &depois),
        ("só o céu (cena sem luz)     ", &so_ceu),
    ] {
        let (m, b) = stats(px);
        let (dm, dx) = delta(px, &antes);
        println!("{nome}      · {m:5.1} · {b:6} · {dm:8.2} · {dx:3}");
    }
    println!("\nA FORÇA, com a luz no sítio de abertura");
    for f in [0.5_f32, 1.0, 2.0, 4.0] {
        let l = vec![ph2d_field_render::PointLamp {
            world: opening_place(&cam),
            radiance_at_one: radiance_at_one(ph2d_field_ecs::FieldLight {
                intensity: f,
                color: [1.0; 3],
            }),
        }];
        let (m, b) = stats(&pinta(&[], &l));
        println!("  força {f:.1} · média {m:5.1} · branco chapado {b:6}");
    }
    println!("\nO PAR (distância, força) que a queda `1/r²` deixa EQUIVALENTE a 1 unidade");
    for r in [1.0_f32, 1.5, 2.0, 3.0, 5.0] {
        let d = [0, 1, 2].map(|i| cam.target[i] + (opening_place(&cam)[i] - cam.target[i]) * r);
        let l = vec![ph2d_field_render::PointLamp {
            world: d,
            radiance_at_one: radiance_at_one(ph2d_field_ecs::FieldLight {
                intensity: r * r,
                color: [1.0; 3],
            }),
        }];
        let px = pinta(&[], &l);
        let (m, b) = stats(&px);
        let (dm, dx) = delta(&px, &antes);
        println!(
            "  a {r:.1} unidades, força {:5.1} · média {m:5.1} · branco {b:5} · |Δ| do de antes \
             {dm:6.2} · máx {dx:3}",
            r * r
        );
    }
    println!("\nA DISTÂNCIA, com força 1 (a queda `1/r²`)");
    for r in [0.8_f32, 1.0, 1.5, 3.0] {
        let d = [0, 1, 2].map(|i| cam.target[i] + (opening_place(&cam)[i] - cam.target[i]) * r);
        let l = vec![ph2d_field_render::PointLamp {
            world: d,
            radiance_at_one: radiance_at_one(ph2d_field_ecs::FieldLight::default()),
        }];
        let (m, b) = stats(&pinta(&[], &l));
        println!("  a {r:.1} unidades · média {m:5.1} · branco chapado {b:6}");
    }
}

/// ⭐⭐ **A LUZ CAI COM `1/r²`** — a lei que faz aproximá-la da peça ser um gesto, e a única do
/// renderizador que a régua do LADO não apanha (mudar a queda não troca o lado aceso).
///
/// # ⚠️⚠️ A primeira redacção deste gate era uma TAUTOLOGIA, e uma mutação provou-o
///
/// Ela chamava `Surface::direct` com uma radiância que **o próprio teste dividia por `r²`**, e
/// afirmava que o resultado caía por quatro. ⇒ apagar a divisão do PRODUTO deixava-a verde: o que
/// ela media era a aritmética escrita nela própria. *Um gate que re-implementa a lei não mede o
/// produto — mede-se a si mesmo.*
///
/// ⇒ hoje ela corre pelo [`ph2d_field_render::shade_render`], e o que compara são duas IMAGENS.
///
/// ⚠️ **A distância é grande de propósito** (`8` e `16` unidades sobre uma peça de raio `0,6`): a uma
/// unidade a geometria muda entre as duas medições — os ângulos de incidência não são os mesmos — e
/// a razão deixaria de ser `4` por uma razão que não é a queda.
#[test]
fn a_light_falls_off_with_the_square_of_the_distance() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    let (w, h) = (96_u32, 96);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let so = [ph2d_material::OpenPbr::default().prepare()];
    let surface = ph2d_field_render::Surfaces {
        all: &so,
        owners: None,
    };
    let (right, up, toward_eye) = cam.basis();
    // ⚠️ **Em LINEAR de cena, sem olhar nenhum**: a vista comprime, e comprimir uma razão não a
    // preserva. `Look::default()` com exposição `0` é a identidade abaixo do branco.
    let media = |points: &[ph2d_field_render::PointLamp]| -> f64 {
        let px = shade_render(
            &g,
            &cam,
            &surface,
            &Lighting {
                lamps: &[],
                points,
                sky: &Escuro,
            },
            ph2d_view_transform::Look::default(),
            [0, 0, 0, 0],
        );
        let c = px.as_chunks::<4>().0;
        let (mut s, mut n) = (0.0_f64, 0_usize);
        for (i, p) in c.iter().enumerate() {
            if g.hit[i] {
                // ⚠️⚠️ **De volta a LINEAR.** A saída é sRGB8, e a curva é `~^(1/2,2)`: uma razão de
                // `4` em linear lê-se **`1,88`** em bytes. *A primeira redacção deste controlo
                // comparou bytes e acusou o produto de dividir por `1,876` — o número é
                // exactamente `4^(1/2,2)`, e o defeito era da régua.*
                s += f64::from(ph2d_color::srgb::srgb_to_linear_byte(p[1]));
                n += 1;
            }
        }
        s / n as f64
    };
    let em = |r: f32| -> Vec<ph2d_field_render::PointLamp> {
        let d = [-0.5_f32, 0.6, 0.62];
        vec![ph2d_field_render::PointLamp {
            world: [0, 1, 2].map(|i| {
                cam.target[i] + r * (d[0] * right[i] + d[1] * up[i] + d[2] * toward_eye[i])
            }),
            // ⚠️ **A força compensa a distância**, senão a imagem longe é preta e a razão mede o
            // chão do byte em vez da queda.
            radiance_at_one: radiance_at_one(ph2d_field_ecs::FieldLight {
                intensity: r * r,
                color: [1.0; 3],
            }),
        }]
    };
    // Com a força a compensar, as duas imagens têm de ser a MESMA: `r²/r² = 1`.
    let perto = media(&em(8.0));
    let longe = media(&em(16.0));
    println!("a 8 unidades: {perto:.3} · a 16 (com força 4×): {longe:.3}");
    assert!(perto > 0.05, "a fixtura ficou preta: {perto}");
    assert!(
        (perto - longe).abs() / perto < 0.02,
        "com a força a compensar a distância a imagem mudou {:.1} % — a queda deixou de ser 1/r²",
        100.0 * (perto - longe).abs() / perto
    );
    // ⭐ **E o CONTROLO: sem compensar, ela cai por quatro.** Sem esta metade o gate passaria sobre
    // um renderizador que ignorasse a distância por completo.
    let crua = |r: f32| -> f64 {
        media(&[ph2d_field_render::PointLamp {
            world: em(r)[0].world,
            radiance_at_one: radiance_at_one(ph2d_field_ecs::FieldLight {
                intensity: 64.0,
                color: [1.0; 3],
            }),
        }])
    };
    let razao = crua(8.0) / crua(16.0);
    println!("sem compensar, a razão 8→16 é {razao:.3}");
    assert!(
        (razao - 4.0).abs() < 0.15,
        "dobrar a distância dividiu a luz por {razao:.3} e a lei pede 4"
    );
    // ⛔⛔ **E uma luz DENTRO DA PEÇA não apaga a imagem** — ver
    // [`ph2d_field_render::POINT_LAMP_MIN_DISTANCE`]. Sem o piso, `1/0` entra como `inf`, sai como
    // `NaN`, e o olhar transforma-o em **PRETO**: *um pixel `NaN` e um pixel legitimamente preto
    // leem-se iguais, que é o pior modo de falha que existe.*
    //
    // ⚠️ **A asserção é sobre a IMAGEM e não sobre a constante.** A primeira redacção afirmava
    // `MIN_DISTANCE > 0.0` — o clippy chamou-lhe *«esta asserção tem valor constante»*, e tinha
    // razão: ela não mede o produto.
    // ⚠️⚠️ **EXACTAMENTE SOBRE UM PONTO DO G-BUFFER**, e as duas redacções anteriores falharam por
    // não chegarem lá: no **centro** da esfera a luz está por detrás de toda a superfície (`N·L < 0`)
    // e o preto é a resposta certa; **encostada** a `0,02` o divisor é pequeno mas finito, e uma
    // mutação que punha o piso a `0` **sobreviveu**. *Só a distância exactamente zero divide por
    // zero, então é ela que o gate tem de produzir* — e o G-buffer entrega o ponto.
    let alvo = (0..g.hit.len())
        .find(|i| g.hit[*i])
        .expect("um pixel de peça");
    let em_cima = shade_render(
        &g,
        &cam,
        &surface,
        &Lighting {
            lamps: &[],
            points: &[ph2d_field_render::PointLamp {
                world: g.point[alvo],
                radiance_at_one: radiance_at_one(ph2d_field_ecs::FieldLight::default()),
            }],
            sky: &Escuro,
        },
        ph2d_view_transform::Look::default(),
        [0, 0, 0, 0],
    );
    let px = em_cima.as_chunks::<4>().0[alvo];
    assert!(
        px[1] > 200,
        "o pixel debaixo da luz saiu {px:?} — sem o piso, `1/0` entra como `inf`, sai como `NaN`, e \
         o olhar transforma-o em PRETO. *Um pixel `NaN` e um pixel legitimamente preto leem-se \
         iguais, que é o pior modo de falha que existe.*"
    );
}

/// Um céu APAGADO — o gate da queda mede a LUZ, e o céu somaria um termo que não cai com `r`.
struct Escuro;
impl ph2d_material::Environment for Escuro {
    fn radiance(&self, _d: [f32; 3], _a: f32) -> [f32; 3] {
        [0.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [0.0; 3]
    }
}

/// ⭐⭐⭐ **UMA LUZ É UMA LINHA DA HIERARQUIA** — e o gate mede-o pela **mesma query** que a casa usa,
/// não por uma lista de componentes escrita à mão.
///
/// # ⛔⛔ O defeito que ele fecha
///
/// A primeira redacção do `add_light` dava `Name` e `FieldPose` e mais nada. A luz existia, iluminava
/// a peça e **não tinha linha nenhuma**: não se podia escolher, renomear, esconder nem apagar. *Uma
/// luz que não aparece na Hierarquia não é um objecto 3D — é uma variável global com uma posição*, e
/// era exactamente o oposto da ordem do dono.
///
/// ⚠️ **A query é `With<Transform>, Without<ChildOf>`** (`ph2d_ecs::HierarchyWalkState`), e este gate
/// escreve-a por extenso de propósito: se ela mudar do outro lado, o que se lê aqui é o gate a ficar
/// verde sobre uma luz invisível. *A cerca contra isso é o piso de população logo abaixo.*
#[test]
fn a_light_is_a_row_of_the_hierarchy() {
    use bevy_ecs::prelude::{With, Without};
    let mut world = bevy_ecs::world::World::new();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.3 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a peça");
    ph2d_field_ecs::spawn_doc(&mut world, &doc, "Model");
    let luz = ph2d_field_ecs::add_light(
        &mut world,
        [1.0, 1.0, 1.0],
        ph2d_field_ecs::FieldLight::default(),
    );

    let mut q = world.query_filtered::<bevy_ecs::entity::Entity, (
        With<ph2d_ecs::Transform>,
        Without<bevy_ecs::hierarchy::ChildOf>,
    )>();
    let raizes: Vec<bevy_ecs::entity::Entity> = q.iter(&world).collect();
    // ⚠️ **PISO DE POPULAÇÃO**: a peça tem de estar lá também, senão um dia em que a query deixe de
    // casar seja o que for este gate leria «zero raízes» e passaria por vacuidade.
    assert_eq!(raizes.len(), 2, "raízes: {raizes:?}");
    assert!(raizes.contains(&luz), "a luz não é uma raiz da Hierarquia");
    // ⭐ **E ela tem NOME** — é a linha que o artista lê.
    assert_eq!(
        world.get::<ph2d_ecs::Name>(luz).map(|n| n.0.clone()),
        Some("Light".to_string())
    );
    // ⭐ **A segunda não se chama «Light» outra vez** — duas linhas iguais são duas linhas que o
    // artista não consegue distinguir.
    let luz2 =
        ph2d_field_ecs::add_light(&mut world, [2.0; 3], ph2d_field_ecs::FieldLight::default());
    assert_eq!(
        world.get::<ph2d_ecs::Name>(luz2).map(|n| n.0.clone()),
        Some("Light 2".to_string())
    );
}
