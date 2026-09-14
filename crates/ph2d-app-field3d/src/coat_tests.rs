//! ⏱️⭐ **O VERNIZ** — a sonda que mede quais dos cinco números dele movem o pixel, e quanto.
//!
//! # ⛔⛔ Porque ela existe antes de uma linha de produto
//!
//! O `crate::materials::surface_of` escreve hoje `7` dos `15` números do OpenPBR (`docs/Render3d/05`
//! §20), e **cinco** dos que ficam são o verniz: `coat_weight`, `coat_color`, `coat_roughness`,
//! `coat_ior` e `coat_darkening`. A lei honra-os todos — o `prepare` deriva `coat_alpha`, `coat_f0`,
//! o escurecimento da base e a atenuação — e nenhum controlo lhes chega.
//!
//! ⚠️ **Mas «a lei honra-o» não é «ele move o quadro»**, e a diferença decide quem ganha linha: um
//! número cujo efeito não se vê é um controlo morto com aparência de vivo, exactamente como a cor da
//! emissão abaixo de zero (§20.3). ⇒ esta sonda mede **um a um**, no rig do produto.
//!
//! # ⚠️ A régua é o MÁXIMO por pixel, e não a média
//!
//! Um verniz é um **realce local**: ele acende uma mancha pequena e deixa o resto da peça onde
//! estava. A média da peça mal se move — foi a régua da §20, e ali estava certa porque a emissão é
//! **global**. *Uma régua emprestada da wave anterior mede a grandeza da wave anterior.*

use crate::render_light::{StudioSky, lamps};

/// O par `(máximo |Δ| por pixel, quantos pixels mudam pelo menos `2` bytes)` entre dois quadros.
///
/// ⚠️ **`2` bytes e não `1`**: um byte é o arredondamento da própria quantização, e contá-lo daria
/// população a uma diferença que ninguém vê.
fn diferenca(a: &[u8], b: &[u8], hit: &[bool]) -> (u8, usize) {
    let (mut pior, mut visiveis) = (0u8, 0usize);
    for (i, (pa, pb)) in a
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0.iter())
        .enumerate()
    {
        if !hit[i] {
            continue;
        }
        let d = (0..3).map(|c| pa[c].abs_diff(pb[c])).max().unwrap_or(0);
        pior = pior.max(d);
        visiveis += usize::from(d >= 2);
    }
    (pior, visiveis)
}

/// ⏱️ **SONDA — qual dos cinco números do verniz move o quadro, e quanto.**
///
/// ⚠️ **Não é um gate — não há barra aqui.** Ela imprime uma tabela e o `/proc/loadavg` ao lado.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_which_of_the_coat_numbers_move_the_picture() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640, 360);
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
    let lamps = lamps(&ph2d_light::LightRig::default());
    let light = Lighting {
        lamps: &lamps,
        sky: &StudioSky,
    };
    let olhar = crate::shading::OPENING_LOOK;
    // ⚠️ **A BASE É FOSCA** (`specular_roughness 0,6`): é sobre uma superfície baça que um verniz
    // aparece como o que ele é — um segundo realce, nítido, por cima de um primeiro que não é.
    // Numa base já polida os dois lóbulos sobrepõem-se e a medição leria metade do efeito.
    let base = ph2d_material::OpenPbr {
        specular_roughness: 0.6,
        ..ph2d_material::OpenPbr::default()
    };
    let pinta = |m: ph2d_material::OpenPbr| {
        let so = [m.prepare()];
        let surface = ph2d_field_render::Surfaces {
            all: &so,
            owners: None,
        };
        shade_render(&g, &cam, &surface, &light, olhar, BG)
    };

    println!(
        "carga: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("esfera {w}x{h} · base fosca (roughness 0,6) · olhar do produto");

    let sem = pinta(base);
    println!("\n peso do verniz ·  pior Δ ·  pixels que mudam ≥2");
    for peso in [0.0_f32, 0.1, 0.25, 0.5, 0.75, 1.0] {
        let (pior, n) = diferenca(
            &pinta(ph2d_material::OpenPbr {
                coat_weight: peso,
                ..base
            }),
            &sem,
            &g.hit,
        );
        println!("  {peso:12.2} · {pior:7} · {n:22}");
    }

    // ⚠️ **Os outros quatro medem-se com o verniz LIGADO**, contra o verniz no ponto de omissão:
    // com peso zero todos eles são inertes por construção, e medi-los ali responderia à pergunta
    // errada (*«um knob desligado faz alguma coisa?»*).
    let cheio = ph2d_material::OpenPbr {
        coat_weight: 1.0,
        ..base
    };
    let referencia = pinta(cheio);
    println!("\n número do verniz · valor ·  pior Δ ·  pixels que mudam ≥2");
    let linha = |nome: &str, valor: String, m: ph2d_material::OpenPbr| {
        let (pior, n) = diferenca(&pinta(m), &referencia, &g.hit);
        println!("  {nome:>16} · {valor:>5} · {pior:7} · {n:22}");
    };
    for r in [0.1_f32, 0.25, 0.5, 1.0] {
        linha(
            "coat_roughness",
            format!("{r:.2}"),
            ph2d_material::OpenPbr {
                coat_roughness: r,
                ..cheio
            },
        );
    }
    for i in [1.0_f32, 1.2, 2.0, 2.5] {
        linha(
            "coat_ior",
            format!("{i:.2}"),
            ph2d_material::OpenPbr {
                coat_ior: i,
                ..cheio
            },
        );
    }
    for c in [[1.0_f32, 0.6, 0.2], [0.2, 0.4, 1.0]] {
        linha(
            "coat_color",
            format!("{:.1}", c[1]),
            ph2d_material::OpenPbr {
                coat_color: c,
                ..cheio
            },
        );
    }
    for d in [0.0_f32, 0.5] {
        linha(
            "coat_darkening",
            format!("{d:.2}"),
            ph2d_material::OpenPbr {
                coat_darkening: d,
                ..cheio
            },
        );
    }

    // ⭐⭐⭐ **A PONTA DO SLIDER DO IOR sai daqui**, e não de um número escolhido: ela é onde o
    // número **deixa de ser observável**. ⚠️ A régua é o Δ contra o ponto ANTERIOR — a distância ao
    // default cresce para sempre e nunca diria onde a curva assenta.
    println!("\n coat_ior · pior Δ contra o ponto anterior · pixels que mudam ≥2");
    let mut anterior: Option<Vec<u8>> = None;
    for ior in [
        1.0_f32, 1.1, 1.2, 1.4, 1.6, 2.0, 2.5, 3.0, 4.0, 6.0, 10.0, 20.0,
    ] {
        let agora = pinta(ph2d_material::OpenPbr {
            coat_ior: ior,
            ..cheio
        });
        if let Some(antes) = &anterior {
            let (pior, n) = diferenca(&agora, antes, &g.hit);
            println!("  {ior:8.2} · {pior:28} · {n:22}");
        }
        anterior = Some(agora);
    }
}

/// Um material com o verniz ligado e os cinco números fora do ponto de omissão.
fn envernizado() -> ph2d_field_ecs::FieldMaterial {
    ph2d_field_ecs::FieldMaterial {
        roughness: 0.6,
        coat: 1.0,
        coat_roughness: 0.2,
        coat_color: [0.9, 0.7, 0.4],
        coat_ior: 2.0,
        coat_darkening: 0.5,
        ..ph2d_field_ecs::FieldMaterial::default()
    }
}

/// A radiância que um material devolve sob a lâmpada principal, num ponto de frente.
fn devolve(m: ph2d_field_ecs::FieldMaterial) -> [f32; 3] {
    let s = crate::materials::surface_of(m);
    let n = [0.0_f32, 0.3, 0.953_939_2];
    let v = [0.0_f32, 0.0, 1.0];
    let para_a_luz = [0.4_f32, 0.6, 0.692_820_3];
    s.direct(n, v, para_a_luz, [3.0; 3])
}

/// ⭐⭐⭐ **OS CINCO NÚMEROS DO VERNIZ CHEGAM À LEI — e cada um MOVE a resposta.**
///
/// # ⚠️ Porque a régua é a RADIÂNCIA, e não a `Surface`
///
/// A [`ph2d_material::Surface`] guarda o `OpenPbr` inteiro lá dentro, logo duas superfícies com
/// params diferentes são **sempre** diferentes por `PartialEq` — um gate escrito assim passaria com
/// a tradução a deitar o número fora, porque ele continuaria a viajar no campo cru. ⇒ *compara-se o
/// que a lei RESPONDE, não o que ela guarda.*
///
/// **Mutações que devem sangrar:** apagar qualquer uma das cinco linhas do
/// [`crate::materials::surface_of`].
#[test]
fn every_coat_number_reaches_the_law_and_moves_the_answer() {
    let base = envernizado();
    let referencia = devolve(base);
    let muda = |nome: &str, m: ph2d_field_ecs::FieldMaterial| {
        let agora = devolve(m);
        let d = (0..3)
            .map(|k| (agora[k] - referencia[k]).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            d > 1.0e-3,
            "mexer no `{nome}` não mudou a radiância ({referencia:?} → {agora:?}) — ou ele não \
             atravessa o `surface_of`, ou a lei deixou de o ler"
        );
    };
    muda("coat", ph2d_field_ecs::FieldMaterial { coat: 0.3, ..base });
    muda(
        "coat_roughness",
        ph2d_field_ecs::FieldMaterial {
            coat_roughness: 0.9,
            ..base
        },
    );
    muda(
        "coat_color",
        ph2d_field_ecs::FieldMaterial {
            coat_color: [0.2, 0.4, 1.0],
            ..base
        },
    );
    muda(
        "coat_ior",
        ph2d_field_ecs::FieldMaterial {
            coat_ior: 1.1,
            ..base
        },
    );
    muda(
        "coat_darkening",
        ph2d_field_ecs::FieldMaterial {
            coat_darkening: 1.0,
            ..base
        },
    );
}

/// ⭐⭐⭐ **OS QUATRO NÚMEROS DO VERNIZ FICAM TRAVADOS ENQUANTO NÃO HOUVER VERNIZ** — visíveis e
/// inactivos, nunca escondidos.
///
/// # ⛔⛔ Esta era «só existem», e a ordem do dono corrigiu-a (14/09)
///
/// Enio: *«os slideres que só aparecem sob uma condição específica não devem desaparecer, mas apenas
/// serem inativados, mas sempre visíveis»*. ⚠️ **A lei já estava escrita nesta casa** — o
/// [`ph2d_field::Span::Locked`] diz, por extenso, que *«é diferente de "não aparece": o valor
/// continua a ser um facto que o artista precisa de ler, e esconder a linha faria o painel saltar de
/// tamanho a cada travessia»*. O que eu apliquei foi a W34 (*o painel oferece exactamente o que o
/// gesto faz*), que proíbe **pintar um controlo** que não pode ser honrado e **não** manda apagar a
/// linha.
///
/// ⚠️ **E a prova de que a lei do efeito é a certa está no gate acima**: com o peso a zero, mexer em
/// qualquer dos quatro **não move um bit** — o `prepare` mistura-os todos por `coat_weight`. *A régua
/// da travagem e a régua do efeito são a mesma pergunta feita de dois lados.*
///
/// **Mutações que devem sangrar:** apagar o braço `13..=18` do `inerte` · trocá-lo por `20..=22`.
#[test]
fn the_coat_numbers_are_locked_while_the_coat_is_off() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = super::colour_row_tests::a_ball();
    let vivas = |sim: &mut ph2d_ecs::SimWorld| -> Vec<(&'static str, bool)> {
        super::colour_row_tests::rows_of(sim, folha)
            .iter()
            .filter(|r| matches!(r.param, ph2d_field::Param::Material(_)))
            .map(|r| (r.key, r.live))
            .collect()
    };
    let do_verniz = [
        "field.dim.coat_color",
        "field.dim.coat_roughness",
        "field.dim.coat_ior",
        "field.dim.coat_darkening",
    ];
    let apagado = vivas(&mut sim);
    // ⭐ **As `15` linhas estão lá, e as do verniz estão TRAVADAS.**
    assert_eq!(
        apagado.len(),
        15,
        "a secção do material tem de ter as 15 linhas em qualquer estado: {apagado:?}"
    );
    for k in do_verniz {
        assert_eq!(
            apagado.iter().find(|(c, _)| *c == k).map(|(_, v)| *v),
            Some(false),
            "a linha `{k}` tinha de estar VISÍVEL e TRAVADA com o verniz a zero: {apagado:?}"
        );
    }
    // ⛔ **E as outras continuam vivas** — trava o que morreu, e mais nada.
    assert!(
        apagado
            .iter()
            .filter(|(c, _)| !do_verniz.contains(c) && *c != "field.dim.emission_color")
            .all(|(_, v)| *v),
        "travar o verniz travou linha alheia: {apagado:?}"
    );

    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(12), 1.0)
        .expect("o verniz");
    let aceso = vivas(&mut sim);
    for k in do_verniz {
        assert_eq!(
            aceso.iter().find(|(c, _)| *c == k).map(|(_, v)| *v),
            Some(true),
            "a linha `{k}` continua travada com o verniz aceso: {aceso:?}"
        );
    }
    // ⛔ **E o brilho continua travado** — os dois pesos são independentes, e um `||` a mais no
    // `inerte` destravaria os dois de uma vez.
    assert_eq!(
        aceso
            .iter()
            .find(|(c, _)| *c == "field.dim.emission_color")
            .map(|(_, v)| *v),
        Some(false),
        "acender o verniz destravou a cor do BRILHO: os dois pesos deixaram de ser independentes"
    );
}

/// ⭐⭐ **A COR DO VERNIZ É A TERCEIRA AMOSTRA, e o IOR NÃO é uma fracção.**
///
/// ⚠️ **As duas metades são a mesma lei vista de dois lados:** a tabela `CORES` do `scene_panel`
/// decide quem é amostra, e o `material_span` decide a faixa de cada número. *Um verniz com o IOR a
/// abrir em `0` daria metade do curso do dedo a sítios que o modelo não admite.*
///
/// **Mutações que devem sangrar:** tirar a entrada `(11, …)` da tabela `CORES` · devolver
/// `SoftFromZero` para o campo `14` no `material_span`.
#[test]
fn the_coat_colour_is_a_swatch_and_the_ior_is_not_a_fraction() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = super::colour_row_tests::a_ball();
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(12), 1.0)
        .expect("o verniz");
    let rows = super::colour_row_tests::rows_of(&mut sim, folha);

    let amostras: Vec<(ph2d_field::Param, [u8; 3])> = rows
        .iter()
        .filter_map(|r| r.swatch.map(|c| (r.param, c)))
        .collect();
    // ⚠️ **QUATRO amostras, sempre** — as quatro cores do material. O que muda com o estado é o
    // `live` de cada uma, e isso tem gate próprio.
    assert_eq!(
        amostras,
        vec![
            (ph2d_field::Param::Material(1), [231, 231, 231]),
            (ph2d_field::Param::Material(7), [255, 255, 255]),
            (ph2d_field::Param::Material(13), [255, 255, 255]),
            (ph2d_field::Param::Material(20), [255, 255, 255]),
        ],
        "a cor do verniz tem de ser uma AMOSTRA, e a de omissão é branca"
    );
    // ⚠️ **Três amostras na mesma forma, três selectores** — a lei do par `(quem, qual)`.
    let ids: Vec<_> = [1u8, 7, 13, 20]
        .iter()
        .map(|k| ph2d_panel_model3d::ids::model3d_color_swatch(folha.to_bits(), *k))
        .collect();
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        4,
        "duas das quatro amostras da mesma forma partilham o id do selector"
    );

    let ior = rows
        .iter()
        .find(|r| r.param == ph2d_field::Param::Material(17))
        .expect("a linha do IOR");
    assert!(
        (ior.lo - 1.0).abs() < 1.0e-6,
        "o slider do IOR começa em {} e tem de começar em 1 — abaixo do vácuo não há material",
        ior.lo
    );
    assert!(
        matches!(ior.bound, ph2d_field::Bound::Hard(t) if (t - 2.5).abs() < 1.0e-6),
        "o tecto do IOR tem de ser DURO e `2,5` (acima do diamante): {:?}",
        ior.bound
    );
    // ⛔ **E os vizinhos continuam fracções** — sem esta metade, um `Range` posto em todos passaria.
    let rug = rows
        .iter()
        .find(|r| r.param == ph2d_field::Param::Material(16))
        .expect("a rugosidade do verniz");
    assert!(
        rug.lo.abs() < 1.0e-6 && matches!(rug.bound, ph2d_field::Bound::Soft(_)),
        "a rugosidade do verniz deixou de ser uma fracção de curso macio: {:?}",
        (rug.lo, rug.bound)
    );
}
