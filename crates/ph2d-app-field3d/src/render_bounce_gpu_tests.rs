//! ⭐⭐⭐ **O RICOCHETE NO DISPOSITIVO** — que ele CHEGA à imagem, e quanto custa
//! (`docs/Render3d/08` §12).
//!
//! # ⚠️⚠️ Porque a paridade não chega
//!
//! O [`crate::gpu_frame::paint_parity_tests::a_imagem_do_dispositivo_e_a_da_cpu`] afirma que os
//! dois motores **concordam**, e a referência de CPU dele passou a calcular o ricochete. ⇒ se
//! ninguém o calculasse em lado nenhum, ele continuaria a concordar — *preto contra preto*.
//!
//! ⛔⛔ E o report que abriu esta wave (*«não funciona, não clareia»*) foi exactamente isso, um
//! nível acima: um gate de costura VERDE sobre uma chamada que estava atrás de uma bandeira
//! desligada. *Um gate que prova que dois lados concordam não prova que algum deles fez o
//! trabalho.*
//!
//! ⇒ este ficheiro afirma a outra metade: **a imagem do dispositivo é MAIS CLARA do que a mesma
//! imagem sem ricochete**, e a diferença é grande.

use crate::gpu_frame::paint_parity_tests::{FUNDO, H, W, fixtura};
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

/// ⭐⭐⭐ **UM CANTO, e não a peça convexa da paridade** — duas placas em ângulo recto.
///
/// ⛔⛔ **A fixtura da paridade não contém o fenómeno, e isso foi MEDIDO:** ela é a união de três
/// formas convexas no aberto, onde quase todo raio do hemisfério **escapa** — o brilho subia
/// `+0,513` níveis, que é ruído ao lado de qualquer barra honesta. *Um gate que afirma «a luz
/// chega» sobre uma cena onde ela quase não existe mede o limite da própria fixtura.*
///
/// ⇒ aqui cada placa vê a outra a meio hemisfério, que é onde o ricochete vive.
fn canto() -> (FieldDoc, Vec<FieldDoc>, Vec<ph2d_material::Surface>) {
    let folhas = [
        // o chão
        ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.70, 0.05, 0.70],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform::at(0.0, -0.55, 0.0),
        ),
        // a parede da esquerda
        ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.05, 0.70, 0.70],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform::at(-0.55, 0.0, 0.0),
        ),
    ];
    let mut nos: Vec<Node> = folhas.to_vec();
    nos.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(ph2d_field::Blend::Sharp),
            children: vec![NodeId(0), NodeId(1)],
        },
    ));
    let doc = FieldDoc::new(nos, NodeId(2)).expect("o canto");
    let postas = folhas
        .iter()
        .map(|n| FieldDoc::new(vec![n.clone()], NodeId(0)).expect("a folha posta"))
        .collect();
    // ⚠️ **Branco e fosco**: o ricochete é `albedo × luz`, e um material escuro devolveria pouco —
    // o gate mediria o material em vez da lei.
    let mats = vec![ph2d_material::OpenPbr::default().prepare(); 2];
    (doc, postas, mats)
}

/// A média dos canais de cor dos pixels de PEÇA (alfa cheia) — a luminosidade da imagem.
fn brilho(rgba: &[u8]) -> f64 {
    let (mut soma, mut n) = (0.0f64, 0usize);
    for px in rgba.as_chunks::<4>().0 {
        if px[3] == 0 {
            continue;
        }
        soma += f64::from(px[0]) + f64::from(px[1]) + f64::from(px[2]);
        n += 3;
    }
    if n == 0 { 0.0 } else { soma / n as f64 }
}

/// ⭐⭐⭐ **O ricochete CHEGA à imagem que o dispositivo pinta.**
///
/// A régua é o **brilho médio da peça**: a mesma cena pintada pelo dispositivo (que calcula o
/// ricochete) contra a mesma marcha pintada pela CPU com o canal **vazio**.
///
/// ⚠️ **A barra sai da MEDIÇÃO e não de um número escolhido** — ver o `println!`: nesta fixtura a
/// diferença mede-se em níveis de 8 bits, e a barra é metade dela.
///
/// # ⛔⛔ O que ele NÃO sabe dizer, e quem o diz
///
/// Ele vê que a imagem **SUBIU**, não que ela subiu **pela razão certa**. Medido por mutação:
/// apagar o despacho do ambiente (`env_irradiance` a devolver sempre o céu) faz a segunda chamada
/// somar **céu a dobrar** em vez do ricochete — e este gate fica **VERDE**, porque a imagem subiu
/// na mesma.
///
/// ⇒ quem mata essa é a **paridade** (`a_imagem_do_dispositivo_e_a_da_cpu`), que compara o valor
/// com a referência de CPU. *Os dois são um PAR: a paridade diz que o número está certo e este diz
/// que ele não é zero — e nenhum dos dois sozinho afirma a wave.*
#[test]
#[ignore = "precisa de GPU"]
fn o_ricochete_chega_a_imagem_do_dispositivo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, postas, materiais) = canto();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(
            cam.half_extent,
            f32::from(u16::try_from(W.min(H)).expect("a tela cabe")),
        ),
    );
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: Some(&owners),
    };
    // ⚠️ **Uma lâmpada FORTE e perto, de propósito:** o ricochete é a luz que as superfícies
    // devolvem, e sem luz directa não há o que devolver. *Uma fixtura escura mediria zero contra
    // zero e o gate ficaria verde a dizer nada.*
    let luz = [ph2d_field_render::PointLamp {
        world: [0.6, 0.9, 0.9],
        radiance_at_one: [3.0; 3],
    }];
    let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();
    let olhar = ph2d_view_transform::Look::default();

    // ── o lado SEM ricochete: a marcha do dispositivo pintada pela CPU, com o canal vazio ─────
    let (g, sh) = crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, W, H, true)
        .expect("a marcha do dispositivo");
    // ⚠️⚠️ **TODOS os pixels, e não só os de peça** — o fundo também escreve neste canal, e ali a
    // lei é a OPOSTA à das sombras: elas nascem a `1` (*ausência de sombra*) e o ricochete nasce a
    // `0` (*ausência de luz*). *Sem esta asserção sobre o FUNDO, a linha que o zera é uma linha que
    // nenhuma mutação consegue matar.*
    assert_eq!(
        (0..g.hit.len())
            .filter(|i| sh.bounce_at(*i) != [0.0; 3])
            .count(),
        0,
        "o canal do ricochete tinha de vir VAZIO pelo caminho da leitura, em TODO pixel — quem o \
         enche é a passagem do PINTOR, e este gate mede a diferença entre os dois"
    );
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    let sem = ph2d_field_render::shade_render(
        &g,
        &cam,
        &surfaces,
        &ph2d_field_render::Lighting {
            lamps: &sem_ecra,
            points: &luz,
            sky: &crate::render_light::StudioSky,
            shadows: Some(&sh),
        },
        olhar,
        FUNDO,
    );

    // ── o lado COM: o pintor do dispositivo ──────────────────────────────────────────────────
    let com = crate::gpu_frame::paint(
        t, &doc, &reg, &cam, &luz, &surfaces, olhar, FUNDO, None, W, H, true,
    )
    .expect("o pintor do dispositivo");

    let (b_sem, b_com) = (brilho(&sem), brilho(&com.rgba));
    let subiu = b_com - b_sem;
    println!("  brilho da peça · sem ricochete {b_sem:.3} · com {b_com:.3} · subiu {subiu:.3}");
    // ⚠️ A barra é **metade do medido** — `179,421 → 184,356`, **`+4,935`** níveis neste canto —,
    // como a do `docs/Render3d/08` §3.1, e nunca um número escrito antes da medição.
    //
    // ⛔⛔ **A 1.ª redacção deste gate usava a fixtura da PARIDADE e reprovou com `+0,513`:** ela é
    // a união de três formas CONVEXAS no aberto, onde quase todo raio do hemisfério escapa. *Não
    // era o produto — era a fixtura a não conter o fenómeno*, e é a segunda vez nesta wave.
    assert!(
        subiu >= 2.4,
        "a imagem do dispositivo tinha de ficar mais clara com o ricochete, e subiu só {subiu:.3} \
         níveis — ou ele não está a ser calculado, ou não chega ao pintor"
    );
    // ⛔ **E o CONTROLO do outro lado: ele não pode acender o que devia estar escuro.** Um
    // ricochete que somasse céu em vez de superfície levaria a imagem inteira para cima.
    assert!(
        subiu <= 40.0,
        "a imagem subiu {subiu:.3} níveis — isso não é luz devolvida por superfícies, é um \
         ambiente a mais"
    );
}

/// ⏱️ **O que o ricochete custa NO DISPOSITIVO** — a medição que o `CLAUDE.md §0.0` exige antes de
/// qualquer tecto.
///
/// ⚠️ **A A/B faz-se pela fonte e não por um knob:** pôr `ao_rays: 0` no `PaintSetup` do
/// [`crate::gpu_frame`] desliga só a passagem do ricochete (a oclusão continua a correr na marcha).
/// *Um knob de produto criado para uma medição é um controlo que fica.*
///
/// # ⏱️ Medido 2026-09-17 (`1920×1080`, `48` direcções, `load 2,5`–`5,0`, mínimo de 7)
///
/// | | min | mediana |
/// |---|---:|---:|
/// | marcha + sombra + oclusão + pintura | `6,14 ms` | `6,62` |
/// | **e mais o ricochete** | **`9,86`** | `10,44` |
///
/// ⇒ **`+3,72 ms`**, `1,61×` o quadro assente — e ele inteiro continua **abaixo** de um quadro de
/// `16,7 ms`, noutra thread.
///
/// ⭐⭐⭐ **Na CPU a MESMA resposta custava `~1,15 s`** (`docs/Render3d/08` §8.2: `19,06 ms` por
/// direcção × `48`). *O caminho mais lento definia o tecto do mais rápido, no módulo cuja razão de
/// existir é o mais rápido* — o `CLAUDE.md §0.0` à letra, e a razão é **`309×`**.
#[test]
#[ignore = "medição — precisa de GPU e de máquina calma"]
fn mede_o_que_o_ricochete_custa_no_dispositivo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    const LW: u32 = 1920;
    const LH: u32 = 1080;
    let (doc, _, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let luz = [ph2d_field_render::PointLamp {
        world: [0.6, 0.9, 0.9],
        radiance_at_one: [3.0; 3],
    }];
    let olhar = ph2d_view_transform::Look::default();
    println!(
        "\n  {LW}×{LH} · {} direcções · load {}",
        ph2d_field_render::OCCLUSION_PASSES,
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let mut v: Vec<f64> = Vec::new();
    for _ in 0..7 {
        let t0 = std::time::Instant::now();
        let p = crate::gpu_frame::paint(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, FUNDO, None, LW, LH, true,
        )
        .expect("o pintor");
        std::hint::black_box(p.rgba.len());
        v.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    v.sort_by(f64::total_cmp);
    println!(
        "  marcha + pintura: min {:6.2} ms · mediana {:6.2} ms",
        v[0],
        v[v.len() / 2]
    );
}

/// ⭐⭐⭐ **O QUADRO DE MOVIMENTO NÃO PAGA O RICOCHETE** — e é a metade que protege o report que o
/// dono já fez uma vez (*«mover os objetos ficou muito lento»*).
///
/// Com a bandeira do quadro assente em baixo (`antialias = false`) a passagem do ricochete não é
/// compilada nem despachada, o canal fica vazio, e a imagem é a que a CPU pinta **com o canal
/// vazio** — ou seja, a de sempre.
///
/// ⚠️⚠️ **O gate irmão (`o_ricochete_chega_a_imagem_do_dispositivo`) é o CONTROLO deste:** ali a
/// mesma comparação tem de DIFERIR. *Sem o par, uma implementação que nunca calculasse o ricochete
/// passaria neste e ninguém daria por isso.*
#[test]
#[ignore = "precisa de GPU"]
fn o_quadro_de_movimento_nao_paga_o_ricochete() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, postas, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(
            cam.half_extent,
            f32::from(u16::try_from(W.min(H)).expect("a tela cabe")),
        ),
    );
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: Some(&owners),
    };
    let luz = [ph2d_field_render::PointLamp {
        world: [0.6, 0.9, 0.9],
        radiance_at_one: [3.0; 3],
    }];
    let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();
    let olhar = ph2d_view_transform::Look::default();

    // ⚠️ **`false` nos DOIS lados** — é a bandeira do quadro de movimento, e ela também desliga a
    // re-amostragem da borda: comparar um lado com ela e outro sem mediria o anti-serrilhado.
    let (g, sh) = crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, W, H, false)
        .expect("a marcha do dispositivo");
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    let cpu = ph2d_field_render::shade_render(
        &g,
        &cam,
        &surfaces,
        &ph2d_field_render::Lighting {
            lamps: &sem_ecra,
            points: &luz,
            sky: &crate::render_light::StudioSky,
            shadows: Some(&sh),
        },
        olhar,
        FUNDO,
    );
    let gpu = crate::gpu_frame::paint(
        t, &doc, &reg, &cam, &luz, &surfaces, olhar, FUNDO, None, W, H, false,
    )
    .expect("o pintor do dispositivo");

    let pior = cpu
        .iter()
        .zip(gpu.rgba.iter())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    println!("  quadro de movimento · pior desvio {pior} nível(is)");
    // A barra é a mesma do gate de paridade: `1` nível é arredondamento entre dois motores.
    assert!(
        pior <= 2,
        "o quadro de movimento diverge da CPU sem ricochete por {pior} níveis — ele está a pagar \
         o ricochete, e é essa a regressão que o dono já reprovou uma vez"
    );
}
