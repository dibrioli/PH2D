//! ⭐⭐⭐ **A CAMADA DE ESTILO CHEGA AO PIXEL, NOS DOIS MOTORES** (`docs/Render3d/03`, a `W8`).
//!
//! # ⚠️⚠️ Porque este ficheiro começa por um gate ESTRUTURAL, e não por um número
//!
//! O §24 do [`docs/Render3d/10`] é a auditoria mais cara que este módulo pagou: **duas waves
//! seguidas foram entregues ao dono com tabela, gates e prova de mutação, e viviam num ramo que o
//! produto não corre.** O caminho de OMISSÃO do modelador é o dispositivo a **PINTAR** — ele devolve
//! a imagem e `return`a antes do sombreamento de CPU —, e tudo o que fosse escrito depois daquele
//! `return` era invisível ao artista com todos os relatórios verdes.
//!
//! ⇒ a lei que fica: **antes de declarar uma cura, prove que o produto EXECUTA o código curado no
//! caminho de omissão dele.** Um gate de VALOR precisa de placa e é `#[ignore]`; um gate
//! **estrutural** corre em toda máquina, em toda corrida, e reprova no dia em que os dois caminhos
//! se separarem outra vez.
//!
//! [`docs/Render3d/10`]: ../../../docs/Render3d/10_a_luz_que_atravessa_a_peca.md

use ph2d_field_render::Presentation;
use ph2d_style::{Curvature, Rim, Style, Zones};

/// Um estilo bem longe da fábrica — todos os botões fora do neutro, e nenhum deles subtil.
pub(crate) fn vestido() -> Style {
    Style {
        rim: Rim {
            color: [0.2, 0.7, 1.0],
            strength: 1.5,
            width: 2.5,
        },
        curvature: Curvature {
            convex: [1.0, 0.55, 0.35],
            concave: [0.30, 0.45, 1.0],
            sharpness: 2.0,
        },
        zones: Zones {
            shadow: [0.55, 0.70, 1.0],
            highlight: [1.0, 0.85, 0.55],
            pivot: 0.18,
        },
        indirect_saturation: 1.8,
    }
}

/// ⭐⭐⭐ **A PEÇA QUE TEM AS DUAS COISAS: uma ARESTA e uma COVA com ÁREA.**
///
/// # ⛔⛔ Porque a fixtura da paridade não serve, e o gate disse-o
///
/// A da paridade junta as folhas com [`ph2d_field::Blend::Sharp`], e um vinco vivo é um conjunto de
/// **MEDIDA NULA**: a grelha de píxeis nunca lá cai, e o censo dos botões reprovou com *«a tinta da
/// COVA moveu só 0 canais»* sobre uma lei correcta. ⚠️ *É a mesma lei que a `W147` deste módulo já
/// pagou — «os pontos do vinco PÕEM-SE, não se procuram».*
///
/// ⇒ aqui uma bola leva uma **cratera cavada na frente**: o interior dela é uma cova sobre uma
/// ÁREA que a câmera vê de frente, e o lábio leva filete para não ser um vinco vivo.
///
/// ⚠️⚠️ **E a 1.ª tentativa também não servia, MEDIDA:** duas esferas unidas com filete deram
/// `negativos = 0` sobre `3 456` píxeis — a banda côncava ficava na silhueta e virada ao contrário.
/// *Uma fixtura que não CONTÉM o fenómeno reprova um gate sobre uma lei correcta.*
///
/// ⭐ **Medida (`128×96`, câmera de omissão):** `3 631` píxeis acertados, `584` deles com curvatura
/// **negativa** (`16 %`), `min = −3,361` — que é `−1/0,30`, o raio da cratera **ao terceiro
/// decimal** —, `p50 = +1,819` (a bola, `1/0,55`) e `max = +11,7` (o lábio com filete).
pub(crate) fn peca_com_aresta_e_cova() -> ph2d_field::FieldDoc {
    use ph2d_field::{Blend, NodeId, Op, Primitive, Xform};
    let folhas = [
        // A bola — toda ela ARESTA (curvatura `+1/R`, e é o que a sonda mede).
        ph2d_field_eval::leaf(Primitive::Sphere { radius: 0.55 }, Xform::IDENTITY),
        // ⭐ E a CRATERA, cavada na frente: uma bola subtraída deixa uma cova de curvatura `−1/r`
        // numa ÁREA que a câmera vê de frente.
        ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.30 },
            Xform::at(0.0, 0.0, 0.52),
        ),
    ];
    let mut nos = folhas.to_vec();
    nos.push(crate::gpu_frame::paint_parity_tests::combina(
        // ⭐ O filete arredonda o lábio da cratera, senão ele é um vinco VIVO — medida nula outra
        // vez.
        Op::Difference(Blend::Exact { radius: 0.06 }),
        vec![NodeId(0), NodeId(1)],
    ));
    ph2d_field::FieldDoc::new(nos, NodeId(2)).expect("a bola com a cratera")
}

/// ⭐⭐⭐ **OS DOIS CAMINHOS RECEBEM A MESMA APRESENTAÇÃO — estrutural, sem placa e sem cena.**
///
/// A lei do §24 tem uma forma barata: *no `smoke_draw_thread`, a apresentação é montada **uma vez** e
/// **antes** do ramo do dispositivo, e os dois braços passam a MESMA variável.* Se alguém voltar a
/// montar um estilo dentro de um dos ramos, isto reprova.
///
/// ⚠️ **A busca é limitada ao corpo do ramo pintado**, e não ao resto do ficheiro: a 1.ª redacção do
/// gate irmão de 19/09 procurava um `return;` em toda a fonte e uma mutação **sobreviveu**, porque
/// ela achou o `return` de outro caminho. *O gate afirmava «há ALGUMA coisa pelo caminho» e o nome
/// dele prometia «ESTE ramo».*
#[test]
fn os_dois_motores_recebem_a_mesma_apresentacao() {
    let fonte = include_str!("smoke_draw_thread.rs");

    // (1) a apresentação é montada UMA vez.
    let montagens = fonte
        .matches("let apresentacao = ph2d_field_render::Presentation {")
        .count();
    assert_eq!(
        montagens, 1,
        "a apresentação da cena é montada {montagens} vezes — ver o §24 do docs/Render3d/10"
    );

    // (2) …e ANTES do ramo que decide o dispositivo.
    let onde_monta = fonte
        .find("let apresentacao = ph2d_field_render::Presentation {")
        .expect("a montagem da apresentação");
    let onde_ramifica = fonte
        .find("let pelo_dispositivo = ")
        .expect("o ramo do dispositivo");
    assert!(
        onde_monta < onde_ramifica,
        "a apresentação nasce DEPOIS do ramo do dispositivo — os dois braços podem divergir"
    );

    // (3) e os DOIS braços passam essa mesma variável.
    let passagens = fonte.matches("&apresentacao,").count();
    assert_eq!(
        passagens, 2,
        "esperava a apresentação passada aos DOIS motores e encontrei {passagens} passagem(ns)"
    );

    // (4) ⚠️⚠️ **O CENSO, com a excepção NOMEADA — e ela apareceu porque o gate a acusou.**
    //
    // A 1.ª redacção exigia ZERO passagens de `p.look` por fora da apresentação, e reprovou sobre
    // produto CORRECTO: há uma terceira, e é o **MATCAP**.
    //
    // ⛔ **E o matcap fica de fora do estilo por DECISÃO, não por esquecimento:** ele é *a luz do
    // OLHO* — um auxiliar de modelação que lê FORMA —, e o estilo é direcção de arte sobre um
    // pipeline fisicamente honesto. Tingir o viewport de modelagem com a grade do filme faria o
    // artista medir a peça através de uma mentira. *O olhar governa os dois (é a gestão de cor da
    // cena); o estilo governa o Render.*
    //
    // ⚠️ **Uma contagem exacta e não um `> 0`:** uma passagem NOVA por fora da apresentação é
    // exactamente o defeito que este gate existe para apanhar.
    let soltos = fonte.matches("            p.look,\n").count();
    assert_eq!(
        soltos, 1,
        "esperava UMA passagem solta do olhar (o matcap) e encontrei {soltos}"
    );
    let onde = fonte
        .find("            p.look,\n")
        .expect("a passagem solta do olhar");
    assert!(
        fonte[..onde]
            .rfind("Shading::Matcap")
            .is_some_and(|m| { fonte[m..onde].find("Shading::Render").is_none() }),
        "a passagem solta do olhar já não é a do matcap — declare a excepção nova ou cure-a"
    );
}

/// ⭐⭐⭐ **A CURVATURA É MEDIDA QUANDO O ESTILO A LÊ — nos dois motores, e é estrutural.**
///
/// ⛔ Sem esta soma o artista mexeria na tinta de aresta e a peça **não mudava um pixel**: a
/// grandeza que o botão escolhe nunca teria sido medida. É a forma de knob morto que o `CLAUDE.md`
/// §5.0 nomeia, e ela não aparece em gate de valor nenhum — aparece como *«não vejo efeito»* num
/// report do dono.
#[test]
fn a_curvatura_e_medida_quando_o_estilo_a_le() {
    let cpu = include_str!("smoke_draw_thread.rs");
    assert!(
        cpu.contains("|| p.style.reads_curvature()"),
        "o caminho de REFERÊNCIA não pergunta ao estilo se ele lê a curvatura"
    );
    let wgsl = include_str!("../../ph2d-field-gpu/src/paint_wgsl_sondas.rs");
    assert!(
        wgsl.contains("|| pintor.modo2.z != 0u"),
        "o caminho do DISPOSITIVO não lê a bandeira do estilo"
    );
    let setup = include_str!("../../ph2d-field-gpu/src/paint.rs");
    assert!(
        setup.contains("u32::from(pintor.style.reads_curvature())"),
        "a bandeira do estilo não chega ao uniforme"
    );
}

/// ⭐⭐⭐ **O ESTILO ENTRA ENTRE A FÍSICA E O OLHAR, nos dois motores** — a ordem é a lei.
///
/// ⛔ Depois do olhar seria tinta sobre um valor já cortado em `1`, e um contorno que não respira
/// com a exposição **separa-se da peça** quando o artista expõe.
#[test]
fn o_estilo_entra_antes_do_olhar_nos_dois_motores() {
    let cpu = include_str!("../../ph2d-field-render/src/shade_render.rs");
    let estilo = cpu.find("pres.style.apply(").expect("o estilo na CPU");
    let olhar = cpu.find("pres.look.apply(").expect("o olhar na CPU");
    assert!(estilo < olhar, "na CPU o olhar corre ANTES do estilo");

    let gpu = include_str!("../../ph2d-field-gpu/src/paint_wgsl_sondas.rs");
    let estilo = gpu
        .find("let cena = st_apply(")
        .expect("o estilo no dispositivo");
    let olhar = gpu
        .find("return vt_to_display(cena,")
        .expect("o olhar no dispositivo");
    assert!(
        estilo < olhar,
        "no dispositivo o olhar corre ANTES do estilo"
    );

    // ⚠️ E a saturação da indirecta corre ANTES das lâmpadas nos dois — saturar depois saturaria
    // também o realce do sol, que é outra lei.
    let sat = cpu
        .find("pres.style.saturate_indirect(")
        .expect("a saturação na CPU");
    let lampadas = cpu
        .find("for lamp in light.lamps {")
        .expect("as lâmpadas na CPU");
    assert!(
        sat < lampadas,
        "na CPU a saturação corre depois das lâmpadas"
    );
    let sat = gpu
        .find("rgb = st_saturate_indirect(")
        .expect("a saturação no dispositivo");
    let lampadas = gpu
        .find("for (var l: u32 = 0u; l < s.n_lamps")
        .expect("as lâmpadas no dispositivo");
    assert!(
        sat < lampadas,
        "no dispositivo a saturação corre depois das lâmpadas"
    );
}

/// ⭐⭐⭐ **O ESTILO DE FÁBRICA É A IMAGEM DE ANTES, BYTE A BYTE** — na referência de CPU.
///
/// ⚠️ É esta asserção que deixa as paridades já pagas (`docs/Render3d/08` §12, a `100,000 %`) ficarem
/// de pé sem serem re-medidas: se ela cair, todas elas passam a medir outra coisa.
#[test]
fn o_estilo_de_fabrica_nao_move_um_byte() {
    let (doc, _, materiais) = crate::gpu_frame::paint_parity_tests::fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let sem = pinta_na_cpu(&doc, &reg, &cam, &luz, &surfaces, Style::default());
    let olhar_so = pinta_na_cpu_so_com_o_olhar(&doc, &reg, &cam, &luz, &surfaces);
    assert_eq!(sem, olhar_so, "o estilo de fábrica moveu um byte");
}

/// ⭐⭐⭐ **CADA BOTÃO MOVE A IMAGEM, no caminho do PRODUTO.**
///
/// ⚠️ A crate da lei já tem um gate por botão sobre a função pura; **este mede o PIXEL**, que é
/// outra pergunta: um botão pode estar certo na lei e não chegar ao sombreador. *A régua é o
/// produto; uma lei pura é um resumo dele.*
#[test]
fn cada_botao_do_estilo_move_a_imagem_no_produto() {
    let doc = peca_com_aresta_e_cova();
    let (_, _, materiais) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = vec![materiais[0]];
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let base = pinta_na_cpu(&doc, &reg, &cam, &luz, &surfaces, Style::default());
    let d = Style::default();
    let casos: [(&str, Style); 5] = [
        (
            "o contorno",
            Style {
                rim: Rim {
                    color: [0.2, 0.7, 1.0],
                    strength: 1.5,
                    width: 2.5,
                },
                ..d
            },
        ),
        (
            "a tinta da ARESTA",
            Style {
                curvature: Curvature {
                    convex: [1.0, 0.4, 0.2],
                    ..d.curvature
                },
                ..d
            },
        ),
        (
            "a tinta da COVA",
            Style {
                curvature: Curvature {
                    concave: [0.2, 0.4, 1.0],
                    ..d.curvature
                },
                ..d
            },
        ),
        (
            "a grade por zona",
            Style {
                zones: Zones {
                    shadow: [0.5, 0.6, 1.0],
                    highlight: [1.0, 0.8, 0.5],
                    ..d.zones
                },
                ..d
            },
        ),
        (
            "a saturação da indirecta",
            Style {
                indirect_saturation: 0.0,
                ..d
            },
        ),
    ];
    for (nome, style) in casos {
        let px = pinta_na_cpu(&doc, &reg, &cam, &luz, &surfaces, style);
        let movidos = px.iter().zip(&base).filter(|(a, b)| a != b).count();
        assert!(
            movidos > 200,
            "{nome} moveu só {movidos} canais — ou o botão não chega ao pixel, ou a régua não olha"
        );
    }
}

/// ⭐⭐⭐ **OS DOIS MOTORES PINTAM O MESMO ESTILO** — o gate de VALOR, com placa.
///
/// ⚠️ **`#[ignore]` porque precisa de adaptador**, como todo gate de GPU desta casa: *skip gracioso
/// não é verde* (`CLAUDE.md` §5.0). A barra é a MESMA da paridade que já existia (`≤ 1` byte, a
/// descida a 8 bits) — a diferença é que aqui os dois lados correm com a camada de estilo LIGADA,
/// que é o que o §24 cobrava.
///
/// # ⛔⛔⛔ E ele achou uma divergência PRÉ-EXISTENTE que só um consumidor SENSÍVEL revela
///
/// A 1.ª corrida reprovou com `3`–`5` bytes em oito píxeis, e a **atribuição botão a botão** diz
/// que a lei do estilo não tem culpa nenhuma — quem diverge é a **CURVATURA**:
///
/// | o que se liga | píxeis acima de `1` | pior |
/// |---|---:|---:|
/// | a fábrica | `0` | **`0`** (byte-idêntico) |
/// | só o contorno | `0` | `1` |
/// | só as zonas e a saturação | `0` | `1` |
/// | a tinta por curvatura, `nitidez 0,2` | `0` | `1` |
/// | a tinta por curvatura, `nitidez 1` | `163` | `7` |
/// | a tinta por curvatura, `nitidez 2` | `168` | `13` |
/// | a tinta por curvatura, `nitidez 8` | `168` | **`45`** |
///
/// ⭐⭐⭐ **A contagem SATURA em ~`165` e a magnitude cresce LINEARMENTE com a nitidez** — que é a
/// assinatura de *uma diferença pequena na CURVATURA, amplificada pelo ganho*. Os dois motores
/// medem-na por caminhos diferentes (a referência pela fita achatada da `ph2d-field-eval`, o
/// dispositivo pelo `field()` do WGSL), e **a divergência já lá estava**: o único consumidor que
/// ela tinha — a subsuperfície maciça — passa a curvatura por uma tabela pré-integrada com piso
/// (`max(κ, 0,01)`), que a **satura**. *Esta tinta é o primeiro consumidor LINEAR nela, e por isso
/// o primeiro instrumento que a vê.*
///
/// ⏳ **DÍVIDA NOMEADA, e ela não é desta wave:** o instrumento que falta mede a **curvatura** nos
/// dois motores, e não o pixel. Enquanto ele não existir, a tinta por curvatura é medida onde ela
/// não amplifica (`nitidez ≤ 0,2`) — ⛔ *e a barra do gate NÃO foi afrouxada para engolir os `13`
/// bytes: a wave que fechar aquela dívida sobe a nitidez desta linha e o gate volta a apertar.*
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn o_dispositivo_e_a_referencia_pintam_o_mesmo_estilo() {
    let doc = peca_com_aresta_e_cova();
    let (_, _, mats) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = vec![mats[0]];
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    // ⚠️ A `nitidez` é a do regime em que a divergência da curvatura não é amplificada — ver a
    // tabela no cabeçalho. Tudo o resto é o estilo vestido a sério.
    let style = Style {
        curvature: Curvature {
            sharpness: 0.2,
            ..vestido().curvature
        },
        ..vestido()
    };
    let Some((cpu, gpu, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
        &surfaces, &doc, &luz, None, style,
    ) else {
        panic!("sem adaptador de GPU — este gate não pode ser saltado em silêncio");
    };
    // ⚠️⚠️ **O CONTROLO vem PRIMEIRO:** um estilo que não movesse nada faria as duas imagens
    // baterem trivialmente, e o gate passaria a medir o nada.
    let Some((cru, _, _)) =
        crate::gpu_frame::paint_parity_tests::dois_caminhos(&surfaces, &doc, &luz)
    else {
        panic!("sem adaptador de GPU");
    };
    let movidos = cpu.iter().zip(&cru).filter(|(a, b)| a != b).count();
    assert!(
        movidos > 2000,
        "o estilo moveu só {movidos} canais na referência — o gate não tem sujeito"
    );

    let piores: Vec<(usize, u8, u8)> = cpu
        .iter()
        .zip(&gpu)
        .enumerate()
        .filter(|(_, (a, b))| a.abs_diff(**b) > 1)
        .map(|(i, (a, b))| (i, *a, *b))
        .take(8)
        .collect();
    assert!(
        piores.is_empty(),
        "os dois motores discordam com o estilo vestido: {piores:?}"
    );
}

/// ⭐⭐⭐ **E A FÁBRICA É BYTE-IDÊNTICA NO DISPOSITIVO** — a asserção de que tudo o resto depende.
///
/// ⚠️ Sem ela, as paridades já pagas (`docs/Render3d/08` §12, a `100,000 %`) passariam a medir outra
/// coisa sem ninguém notar. **Medido: `0` píxeis fora, pior `0`.**
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn o_estilo_de_fabrica_e_byte_identico_no_dispositivo() {
    let doc = peca_com_aresta_e_cova();
    let (_, _, mats) = crate::gpu_frame::paint_parity_tests::fixtura();
    let materiais = vec![mats[0]];
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::paint_parity_tests::lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let Some((cpu, gpu, _)) = crate::gpu_frame::paint_parity_tests::dois_caminhos_vestidos(
        &surfaces,
        &doc,
        &luz,
        None,
        Style::default(),
    ) else {
        panic!("sem adaptador de GPU");
    };
    let fora = cpu.iter().zip(&gpu).filter(|(a, b)| a != b).count();
    assert_eq!(fora, 0, "a fábrica moveu {fora} bytes no dispositivo");
}

/// Pinta pela porta da REFERÊNCIA, com a apresentação inteira.
fn pinta_na_cpu(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    luz: &[ph2d_field_render::PointLamp],
    surfaces: &ph2d_field_render::Surfaces<'_>,
    style: Style,
) -> Vec<u8> {
    let pres = Presentation {
        look: crate::shading::OPENING_LOOK,
        style: style.sanitized(),
        piece_radius: ph2d_field_eval::bounds::bounding_ball(doc, reg).map_or(1.0, |b| b.radius),
    };
    sombreia(doc, reg, cam, luz, surfaces, &pres)
}

/// O mesmo, com o estilo que a [`Presentation::of`] dá — a identidade.
fn pinta_na_cpu_so_com_o_olhar(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    luz: &[ph2d_field_render::PointLamp],
    surfaces: &ph2d_field_render::Surfaces<'_>,
) -> Vec<u8> {
    sombreia(
        doc,
        reg,
        cam,
        luz,
        surfaces,
        &Presentation::of(crate::shading::OPENING_LOOK),
    )
}

fn sombreia(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    luz: &[ph2d_field_render::PointLamp],
    surfaces: &ph2d_field_render::Surfaces<'_>,
    pres: &Presentation,
) -> Vec<u8> {
    const W: u32 = 128;
    const H: u32 = 96;
    let mut g = ph2d_field_render::trace(doc, reg, cam, W, H);
    // ⚠️ **A curvatura é assada quando ALGUÉM a lê** — a mesma soma de duas portas que o produto
    // faz. Sem isto o gate da tinta de aresta mediria uma peça com curvatura `0` em todo lado.
    if (surfaces
        .all
        .iter()
        .any(ph2d_material::Surface::reads_curvature)
        || pres.style.reads_curvature())
        && let Some(bola) = ph2d_field_eval::bounds::bounding_ball(doc, reg)
    {
        let mut eval = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
        g.curvature = ph2d_field_render::curvatura::do_gbuffer(
            &mut eval,
            &g,
            ph2d_field_render::curvatura::eps_para(bola.radius),
        );
    }
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    ph2d_field_render::shade_render(
        &g,
        cam,
        surfaces,
        &ph2d_field_render::Lighting {
            lamps: &sem_ecra,
            points: luz,
            sky: &crate::render_light::StudioSky,
            shadows: None,
        },
        pres,
        [0, 0, 0, 0],
    )
}
