//! ⭐⭐⭐ **AS ALÇAS DE VÉRTICE NO CANVAS** (W133) — a lei, a precedência e a **costura** do gesto.
//!
//! Enio, 2026-09-07: *«para esse tipo de objeto e todos os outros que dependem de posição de vertex,
//! os vertex devem aparecer no canvas em tempo real e o usuário então poderá movê-los através do
//! gizmo no próprio canvas»*.
//!
//! | gate | o defeito que ele apanha |
//! |---|---|
//! | `a_vertex_handle_lands_where_the_point_is` | a alça pousar ao lado do ponto que ela move |
//! | `a_vertex_wins_the_pixel_it_is_painted_on` | um braço invisível do gizmo roubar o ponto pintado |
//! | `dragging_a_vertex_stays_in_the_shapes_own_plane` | a alça andar num plano do MUNDO num nó rodado |
//! | `the_frame_selector_does_not_move_the_vertex_plane` | o seletor Global/Local mexer no plano do contorno |
//! | `a_vertex_drag_writes_the_two_rows_of_that_vertex` | a **costura**: o arrasto chegar ao documento |
//! | `dragging_a_vertex_leaves_the_other_points_alone` | escrever no vértice errado (o `first_row` fora) |

use crate::gizmo::{self, Anchor, Frame, Handle, Motion, Shape, Target};
use ph2d_ecs::SimWorld;
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_render::{Orbit, Screen};

const AREA_W: u32 = 800;
const AREA_H: u32 = 600;

fn screen() -> Screen {
    Screen::new(AREA_W, AREA_H, Orbit::default().half_extent)
}

/// O polígono côncavo de referência — o mesmo dos gates da forma.
fn pontos() -> Vec<[f32; 2]> {
    vec![
        [-0.38, -0.22],
        [0.36, -0.12],
        [0.14, 0.38],
        [0.02, 0.04],
        [-0.22, 0.28],
    ]
}

fn poligono() -> Primitive {
    Primitive::Polygon {
        profile: ph2d_field::polygon_profile(pontos()).expect("o contorno"),
        half_height: 0.12,
        round: 0.0,
        chamfer: 0.0,
    }
}

/// Uma âncora nos eixos do mundo, com os locais iguais — o caso sem rotação nem escala.
fn ancora() -> Anchor {
    Anchor {
        entity: 7,
        origin: [0.0; 3],
        axes: Frame::Global.axes([0.0, 0.0, 0.0, 1.0]),
        local: Frame::Local.axes([0.0, 0.0, 0.0, 1.0]),
    }
}

/// ⭐ **A alça pousa onde o ponto está** — a projeção do vértice, e não uma segunda conta.
///
/// ⚠️ É a mesma lei que o gizmo já tinha para a âncora (*a projeção é a MESMA do traçador*): uma
/// alça que agarra ao lado da superfície que ela diz mover é o defeito que ninguém chama de defeito
/// de projeção.
#[test]
fn a_vertex_handle_lands_where_the_point_is() {
    let cam = Orbit::default();
    let a = ancora();
    let alcas = gizmo::project_vertices(a, &pontos(), &cam, screen());
    assert_eq!(alcas.len(), pontos().len(), "uma alça por ponto, na ordem");
    for (i, v) in pontos().iter().enumerate() {
        assert_eq!(
            alcas[i].handle,
            Handle::Vertex(i),
            "a alça {i} tem de carregar o índice {i} — a lista é indexada pela ORDEM"
        );
        let mundo = [
            a.origin[0] + a.local[0][0] * v[0] + a.local[1][0] * v[1],
            a.origin[1] + a.local[0][1] * v[0] + a.local[1][1] * v[1],
            a.origin[2] + a.local[0][2] * v[0] + a.local[1][2] * v[1],
        ];
        let esperado = cam.project(mundo, screen()).expect("o ponto projecta").0;
        let Shape::Point { center } = alcas[i].shape else {
            panic!("um vértice é um ponto");
        };
        let d = (center[0] - esperado[0]).hypot(center[1] - esperado[1]);
        assert!(
            d < 1.0e-3,
            "a alça {i} pousou a {d:.4} px do ponto que ela move"
        );
    }
}

/// ⭐⭐⭐ **O PIXEL PINTADO É O PIXEL AGARRADO** — o ponto ganha ao braço que passa por cima dele.
///
/// ⚠️ **Ele pergunta à lista DO PRODUTO** ([`crate::input::handles`]), e não a uma montada
/// aqui. ⛔⛔ **A primeira redacção montava a lista dentro do teste, e uma mutação que trocava a
/// ordem no produto SOBREVIVEU** — o gate media a ordem que ele próprio tinha escrito. *Um gate que
/// copia a fórmula fica verde sobre uma lei que ninguém shipa.*
///
/// ⚠️ **É a ordem da lista que decide as duas coisas de uma vez**: ela é percorrida de frente para
/// trás a apontar e ao contrário a pintar, logo à cabeça = por cima **e** primeiro a agarrar.
#[test]
fn a_vertex_wins_the_pixel_it_is_painted_on() {
    crate::smoke::set_armed_by_panel(true);
    crate::smoke::with_smoke(|s| {
        s.vp_mut().area = Some(ph2d_editor::zones::Rect {
            x: 0.0,
            y: 0.0,
            w: f32::from(u16::try_from(AREA_W).expect("cabe")),
            h: f32::from(u16::try_from(AREA_H).expect("cabe")),
        });
        s.vp_mut().cam = Orbit::default();
        s.gizmo_mode = gizmo::Mode::Move;
        s.gizmo = Some(ancora());
        s.vertices = Some(gizmo::Vertices {
            entity: 7,
            first_row: 1,
            points: pontos(),
        });

        let lista = crate::input::handles(s);
        // ⚠️ **O controlo é a MESMA lista sem os pontos** — se ali ninguém reclama o pixel, este
        // gate não está a medir disputa nenhuma e ficaria verde por vácuo.
        let so_gizmo = gizmo::project(ancora(), &s.vp().cam, screen(), s.gizmo_mode);
        let mut disputados = 0;
        for (i, alca) in gizmo::project_vertices(ancora(), &pontos(), &s.vp().cam, screen())
            .iter()
            .enumerate()
        {
            let Shape::Point { center } = alca.shape else {
                panic!("um vértice é um ponto");
            };
            if gizmo::pick(&so_gizmo, center).is_some() {
                disputados += 1;
            }
            assert_eq!(
                gizmo::pick(&lista, center),
                Some(Handle::Vertex(i)),
                "o pixel do vértice {i} foi para outra alça — o que se vê tem de ser o que se agarra"
            );
        }
        assert!(
            disputados > 0,
            "nenhum dos {} pontos cai sobre uma alça do gizmo — este gate não mediu disputa nenhuma",
            pontos().len()
        );
    })
    .expect("o módulo está armado");
}

/// ⭐⭐⭐ **UM VÉRTICE ANDA NO PLANO DA FORMA**, mesmo com o nó rodado.
///
/// ⚠️ **A régua é a componente FORA do plano, e ela tem de ser zero.** Um vértice tem duas
/// coordenadas: se o arrasto o tirasse do plano do contorno, o deslocamento projectado de volta
/// perderia parte do movimento e a alça andaria menos do que a mão.
#[test]
fn dragging_a_vertex_stays_in_the_shapes_own_plane() {
    let cam = Orbit::default();
    // Um quarto de volta em X: o plano do contorno deixa de ser o `z = 0` do mundo.
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let rodado = Anchor {
        entity: 7,
        origin: [0.0; 3],
        axes: Frame::Global.axes([s, 0.0, 0.0, s]),
        local: Frame::Local.axes([s, 0.0, 0.0, s]),
    };
    let d = match gizmo::drag(
        Handle::Vertex(0),
        rodado,
        &cam,
        screen(),
        [400.0, 300.0],
        [460.0, 340.0],
    ) {
        Motion::Translate(d) => d,
        outro => panic!("mover um vértice é uma translação, e veio {outro:?}"),
    };
    let normal = rodado.local[2];
    let fora = d[0] * normal[0] + d[1] * normal[1] + d[2] * normal[2];
    assert!(
        fora.abs() < 1.0e-5,
        "o arrasto tirou o vértice do plano do contorno em {fora:.6} — ele só tem duas coordenadas"
    );
    let anda = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    assert!(
        anda > 1.0e-3,
        "o arrasto não moveu nada ({anda:.6}) — o gate acima ficaria verde sobre uma alça morta"
    );
}

/// ⭐⭐ **O SELETOR Global/Local não mexe no plano do vértice.**
///
/// ⚠️ **É o defeito que a ponte quase teve:** `Anchor::axes` obedece ao seletor, e ler esse campo
/// aqui faria a alça andar num plano do mundo enquanto o número do painel mexia no plano da forma.
/// *A alça e a linha do painel são a mesma grandeza, ou uma delas mente.*
#[test]
fn the_frame_selector_does_not_move_the_vertex_plane() {
    let cam = Orbit::default();
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let rot = [s, 0.0, 0.0, s];
    let local = Frame::Local.axes(rot);
    let (de, para) = ([400.0, 300.0], [470.0, 250.0]);
    let como = |frame: Frame| {
        gizmo::drag(
            Handle::Vertex(2),
            Anchor {
                entity: 7,
                origin: [0.0; 3],
                axes: frame.axes(rot),
                local,
            },
            &cam,
            screen(),
            de,
            para,
        )
    };
    assert_eq!(
        como(Frame::Global),
        como(Frame::Local),
        "trocar o referencial do GIZMO mudou para onde um VÉRTICE anda — ele não tem essa escolha"
    );
}

/// Uma cena com um polígono só, e a entidade dele.
fn cena() -> (SimWorld, u64) {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(poligono()))],
        NodeId(0),
    )
    .expect("a peça");
    let mut sim = SimWorld::new();
    let root = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça");
    (sim, root.to_bits())
}

/// Os pontos que o documento tem agora.
fn pontos_de(sim: &SimWorld, bits: u64) -> Vec<[f32; 2]> {
    let e = bevy_ecs::entity::Entity::from_bits(bits);
    let no = sim
        .world()
        .get::<ph2d_field_ecs::FieldNode>(e)
        .expect("um nó");
    let ph2d_field::NodeShape::Leaf(p) = &no.shape else {
        panic!("uma folha");
    };
    ph2d_field::vertex_rows(p).expect("tem vértices").points
}

/// ⭐⭐⭐ **A COSTURA: o arrasto chega ao DOCUMENTO, nas duas coordenadas daquele vértice.**
///
/// ⚠️ **É este o gate que a `DIRETIVA_IMPLEMENTACAO` §1 exige**, e não os quatro de cima. Eles
/// perguntam *«a lei está certa?»*; este pergunta *«arrastar o ponto move o ponto?»* — e a resposta
/// atravessa a ponte, a tabela de linhas daquela forma e a porta que valida.
///
/// ⭐ **E ele mede a conta, não só o efeito**: um deslocamento de mundo de `(0,07 · −0,05 · 0)` sobre
/// um nó sem rotação nem escala tem de aparecer **tal e qual** nas duas coordenadas locais.
#[test]
fn a_vertex_drag_writes_the_two_rows_of_that_vertex() {
    let (mut sim, bits) = cena();
    let antes = pontos_de(&sim, bits);
    let d = [0.07_f32, -0.05, 0.0];
    crate::scene::apply_motion_for_test(
        &mut sim,
        bits,
        &[bevy_ecs::entity::Entity::from_bits(bits)],
        Target::Vertex(2),
        Motion::Translate(d),
    );
    let depois = pontos_de(&sim, bits);
    let esperado = [antes[2][0] + d[0], antes[2][1] + d[1]];
    assert!(
        (depois[2][0] - esperado[0]).abs() < 1.0e-5 && (depois[2][1] - esperado[1]).abs() < 1.0e-5,
        "o vértice 2 devia ir de {:?} para {esperado:?} e foi para {:?} — o arrasto não chegou ao \
         documento, ou chegou noutra unidade",
        antes[2],
        depois[2]
    );
}

/// ⭐⭐ **E só aquele vértice se mexe** — o `first_row` da tabela daquela forma está certo.
///
/// ⚠️ **Sem esta metade, um `first_row` errado passaria despercebido**: o gate acima mede o vértice
/// que ele próprio nomeia, e um desvio de duas linhas escreveria no vizinho com a mesma aritmética.
/// *É a diferença entre «o número mudou» e «o número CERTO mudou».*
#[test]
fn dragging_a_vertex_leaves_the_other_points_alone() {
    let (mut sim, bits) = cena();
    let antes = pontos_de(&sim, bits);
    crate::scene::apply_motion_for_test(
        &mut sim,
        bits,
        &[bevy_ecs::entity::Entity::from_bits(bits)],
        Target::Vertex(0),
        Motion::Translate([0.09, 0.04, 0.0]),
    );
    let depois = pontos_de(&sim, bits);
    assert_eq!(
        antes.len(),
        depois.len(),
        "a contagem de vértices mudou ao arrastar um deles — a linha da CONTAGEM foi escrita"
    );
    for i in 1..antes.len() {
        assert_eq!(
            antes[i], depois[i],
            "o vértice {i} mexeu-se e ninguém lhe tocou — o `first_row` daquela forma está fora"
        );
    }
    // ⛔⛔ **E o vértice 0 tem de ir para o sítio EXACTO, e não só «mexer-se».**
    //
    // A 1.ª redacção pedia `antes[0] != depois[0]`, e uma mutação que punha o `first_row` do
    // polígono em `0` **SOBREVIVEU**: ali o par de linhas passa a ser *(contagem, v1.x)*, então o
    // `x` do vértice 0 muda à mesma e nenhum vizinho se mexe. *A diferença entre «o número mudou» e
    // «o número CERTO mudou» é a única coisa que separa este gate de estar verde sobre a escrita
    // deslocada.*
    let esperado = [antes[0][0] + 0.09, antes[0][1] + 0.04];
    assert!(
        (depois[0][0] - esperado[0]).abs() < 1.0e-5 && (depois[0][1] - esperado[1]).abs() < 1.0e-5,
        "o vértice 0 devia ir de {:?} para {esperado:?} e foi para {:?} — as duas linhas escritas \
         não são as daquele vértice",
        antes[0],
        depois[0]
    );
}

/// ⭐⭐⭐ **A COSTURA VALE PARA TODA FORMA DE VÉRTICES, e não só para o polígono** (W133).
///
/// Enio pediu *«esse tipo de objeto **e todos os outros** que dependem de posição de vertex»*, e hoje
/// são **dois** — o triângulo (seis linhas fixas, `first_row = 0`) e o polígono (a contagem à frente,
/// `first_row = 1`). ⚠️ **As duas tabelas põem as coordenadas em sítios diferentes**, e é exactamente
/// aí que uma alça escreve no número errado.
///
/// ⚠️ **A contagem é uma CATRACA com a metade justa:** quando uma terceira forma de vértices nascer,
/// este gate reprova **com o nome dela**, e quem a escrever acrescenta-a aqui. *Uma lista escrita à
/// mão sem censo é a que fica para trás na wave seguinte.*
#[test]
fn every_shape_with_vertices_moves_the_point_the_handle_names() {
    let casos: Vec<(&str, Primitive)> = vec![
        ("polygon", poligono()),
        (
            "triangle",
            Primitive::Triangle {
                a: [-0.34, -0.20],
                b: [-0.10, 0.36],
                c: [0.32, -0.08],
                half_height: 0.13,
                round: 0.0,
                chamfer: 0.0,
            },
        ),
    ];
    for (nome, p) in casos {
        let doc = FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .unwrap_or_else(|e| panic!("«{nome}» não é uma peça válida: {e:?}"));
        let mut sim = SimWorld::new();
        let bits = ph2d_field_ecs::spawn_doc(sim.world_mut(), &doc, "peça").to_bits();
        let antes = pontos_de(&sim, bits);
        let d = [0.05_f32, 0.03, 0.0];
        let alvo = 1_usize;
        crate::scene::apply_motion_for_test(
            &mut sim,
            bits,
            &[bevy_ecs::entity::Entity::from_bits(bits)],
            Target::Vertex(alvo),
            Motion::Translate(d),
        );
        let depois = pontos_de(&sim, bits);
        let esperado = [antes[alvo][0] + d[0], antes[alvo][1] + d[1]];
        assert!(
            (depois[alvo][0] - esperado[0]).abs() < 1.0e-5
                && (depois[alvo][1] - esperado[1]).abs() < 1.0e-5,
            "«{nome}»: o vértice {alvo} devia ir de {:?} para {esperado:?} e foi para {:?}",
            antes[alvo],
            depois[alvo]
        );
        for (i, (a, b)) in antes.iter().zip(depois.iter()).enumerate() {
            if i != alvo {
                assert_eq!(a, b, "«{nome}»: o vértice {i} mexeu-se e ninguém lhe tocou");
            }
        }
    }
}

/// ⭐⭐ **A CATRACA da lista acima** — quantas formas de vértices existem, contadas na PORTA.
///
/// ⚠️ **A metade justa vive noutra crate**, e é ela que impede esta de mentir: o
/// `the_vertex_door_knows_every_shape_with_free_rows` (em `ph2d-field-eval`) percorre
/// [`ph2d_field::PrimitiveKind::ALL`] com um representante de cada e prova que a porta conhece toda
/// forma cuja tabela declara linhas de POSIÇÃO. *Uma catraca sem censo de obsolescência vira
/// licença* — aqui o censo é aquele, e esta linha é só o número que o gate irmão cobre.
///
/// ⇒ Quando a terceira forma de vértices nascer, este gate reprova e quem a escrever acrescenta-a ao
/// `every_shape_with_vertices_moves_the_point_the_handle_names`.
#[test]
fn the_gate_above_covers_every_shape_that_has_vertices() {
    const COBERTAS: usize = 2;
    let vivas = [
        poligono(),
        Primitive::Triangle {
            a: [-0.3, -0.2],
            b: [-0.1, 0.3],
            c: [0.3, -0.1],
            half_height: 0.1,
            round: 0.0,
            chamfer: 0.0,
        },
    ]
    .into_iter()
    .filter(|p| ph2d_field::vertex_rows(p).is_some())
    .count();
    assert_eq!(
        vivas, COBERTAS,
        "o gate irmão cobre {COBERTAS} formas de vértices e a porta responde por outro número — \
         acrescente a forma nova a `every_shape_with_vertices_moves_the_point_the_handle_names`"
    );
}
