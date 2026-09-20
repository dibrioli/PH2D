//! **OS GATES DO PIVÔ** do `source.shape` — ordem do dono (2026-09-19): *«crie no nó Shape o
//! offset do Pivot»*.
//!
//! ⚠️ **Irmão do [`super::tests`] pelo tecto de LOC (HR-18) e por PERGUNTA:** ali mede-se a PORTA
//! ÚNICA (a chave que o shell publica e o nó lê) e o catálogo de formas; aqui mede-se **onde a
//! forma se pendura**, que é uma lei sobre a caixa de corte e não sobre a receita de cada espécie.
//!
//! ⛔⛔ **O que este ficheiro existe para impedir é o CONTROLO MORTO** (§5.0): um param que o
//! cartão pinta e que o barro ignora. A única régua que o apanha é medir o PRODUTO com duas
//! posições do knob — *nenhum censo de registo o vê, porque ele está registado.*

use super::{build_shape_path, manifest_default};
use ph2d_node_motion_shape::{ShapeKind, ShapeParams, shape_key};

/// ⭐⭐⭐ **O PIVÔ CHEGA AO BARRO — a pergunta que o §5.0 diz que nenhum instrumento faz.**
///
/// Ordem do dono (2026-09-19): *«crie no nó Shape o offset do Pivot»*. Um param que o cartão
/// pinta e que a forma ignora é o **controlo morto** canónico desta casa, e a única régua que o
/// apanha é medir o PRODUTO com duas posições do knob.
///
/// A lei tem **três** metades, e cada uma mata uma cura barata:
/// 1. `0` é **no-op byte-idêntico** — senão isto mudava toda forma que já shipou;
/// 2. a forma **DESLOCA-SE** pela fracção pedida da própria extensão;
/// 3. e ela **NÃO se deforma** — a extensão fica a mesma. *Sem a 3.ª, um pivô implementado a
///    esticar a caixa (em vez de a transladar) passaria nas duas primeiras.*
#[test]
fn o_pivot_desloca_a_forma_sem_a_deformar() {
    let faixa = |kind: ShapeKind, pivot: [f32; 2]| {
        let p = build_shape_path(&ShapeParams {
            kind,
            size: 1.0,
            pivot,
            ..ShapeParams::read(manifest_default)
        });
        p.verts.iter().fold(
            (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
            |(xl, xh, yl, yh), v| {
                (
                    xl.min(v.anchor[0]),
                    xh.max(v.anchor[0]),
                    yl.min(v.anchor[1]),
                    yh.max(v.anchor[1]),
                )
            },
        )
    };

    // (1) O NEUTRO, e ele é medido contra a forma construída SEM tocar no param.
    let nu = build_shape_path(&ShapeParams {
        kind: ShapeKind::Star,
        ..ShapeParams::read(manifest_default)
    });
    let neutro = build_shape_path(&ShapeParams {
        kind: ShapeKind::Star,
        pivot: [0.0, 0.0],
        ..ShapeParams::read(manifest_default)
    });
    assert_eq!(
        nu.verts.len(),
        neutro.verts.len(),
        "o neutro tem de ser a MESMA forma"
    );
    for (a, b) in nu.verts.iter().zip(&neutro.verts) {
        assert_eq!(a.anchor, b.anchor, "o pivot `0` tem de ser no-op AO BIT");
    }

    // (2) e (3): sobre a `Cross`, que é simétrica nos dois eixos — assim um deslocamento não se
    // pode confundir com a assimetria da própria silhueta.
    let (x0l, x0h, y0l, y0h) = faixa(ShapeKind::Cross, [0.0, 0.0]);
    // ⚠️ **`1` é a ARESTA** (o `Size` é o SEMI-eixo), logo o deslocamento é `f × meia-extensão`.
    for (eixo, pivot, esperado) in [
        ("x", [1.0_f32, 0.0], (x0h - x0l) * 0.5),
        ("y", [0.0, -0.5], -(y0h - y0l) * 0.25),
    ] {
        let (xl, xh, yl, yh) = faixa(ShapeKind::Cross, pivot);
        let (andou, largura, alvo) = if eixo == "x" {
            (xl - x0l, xh - xl, x0h - x0l)
        } else {
            (yl - y0l, yh - yl, y0h - y0l)
        };
        assert!(
            (andou - esperado).abs() < 1e-9,
            "no eixo {eixo} a forma tinha de andar {esperado} e andou {andou}"
        );
        assert!(
            (largura - alvo).abs() < 1e-9,
            "no eixo {eixo} a forma DEFORMOU-SE: {largura} contra {alvo}"
        );
    }
}

/// ⭐⭐ **O PIVÔ É UM OFFSET, e o `0` dele não quer dizer «no centro».**
///
/// O osso é cortado de `[0, 2s]` — ele pendura-se na CABEÇA por natureza —, logo `pivot = 0`
/// deixa-o pendurado e **`−0,5` é que o centra**. *É essa a diferença entre um «Pivot» e um
/// «Pivot Offset», e ela é o que o dono pediu pelo nome.*
#[test]
fn o_zero_do_pivot_e_o_pivo_natural_da_especie() {
    let lo_de = |kind: ShapeKind, pivot: [f32; 2]| {
        build_shape_path(&ShapeParams {
            kind,
            size: 1.0,
            pivot,
            ..ShapeParams::read(manifest_default)
        })
        .verts
        .iter()
        .fold(f64::MAX, |m, v| m.min(v.anchor[0]))
    };
    assert!(
        lo_de(ShapeKind::Bone, [0.0, 0.0]).abs() < 1e-9,
        "com o offset a `0` o osso continua PENDURADO na cabeca"
    );
    assert!(
        (lo_de(ShapeKind::Bone, [-1.0, 0.0]) + 1.0).abs() < 1e-9,
        "com `-1` (uma ARESTA para tras) ele passa a estar CENTRADO, como qualquer carimbo"
    );
    // ⛔ E o CONTROLO, na direcção oposta: numa forma de pivô natural CENTRADO, é o `+0,5` que a
    // pendura. *Sem ele, um `pivot` que somasse sempre meio semi-eixo passaria a primeira metade.*
    assert!(
        (lo_de(ShapeKind::Circle, [0.0, 0.0]) + 1.0).abs() < 1e-9,
        "um circulo com o offset a `0` fica centrado"
    );
    assert!(
        lo_de(ShapeKind::Circle, [1.0, 0.0]).abs() < 1e-9,
        "e com `+1` ele pendura-se pela ARESTA esquerda"
    );
}

/// ⚠️ **O PIVÔ ENTRA NA CHAVE DA GEOMETRIA** — senão a 1.ª forma cozida volta do cache para todos
/// os outros valores e o controlo fica **inerte depois da primeira vez**.
///
/// É o defeito que o cabeçalho de [`ph2d_node_motion_shape::param::ALL`] narra (o *Pattern Offset*
/// do sculpt3d, 2026-08-09), e a régua é a chave que o shell e o nó calculam pela MESMA porta.
#[test]
fn o_pivot_entra_na_chave_da_geometria() {
    let chave = |pivot: f32| {
        shape_key(|n| {
            if n == ph2d_node_motion_shape::param::PIVOT_X {
                pivot
            } else {
                manifest_default(n)
            }
        })
    };
    assert_ne!(
        chave(0.0),
        chave(0.25),
        "duas posicoes do pivot tem de nomear geometrias DIFERENTES"
    );
    assert_eq!(chave(0.0), chave(0.0), "e a mesma posicao a mesma chave");
}
/// ⭐⭐⭐ **O PIVÔ É RELATIVO AO TAMANHO DA FORMA** — ordem do dono (2026-09-19), medida pela porta
/// do PRODUTO (`read_unit`, que é por onde o shell coze) e não pelo construtor cru.
///
/// A mesma posição do knob tem de dar o MESMO ponto da forma em qualquer `Size`: a geometria é
/// cozida em raio `1` e o tamanho autorado viaja na coluna `size` da instância, logo o
/// deslocamento sai escalado junto. **Medido: a `Pivot X = 1` a aresta esquerda pousa na posição
/// em `size` `0,5`, `1` e `2`.**
///
/// ⚠️ **Ele nasceu de uma SONDA `#[ignore]`** escrita para decidir se a ordem do dono era sobre a
/// UNIDADE ou sobre um defeito. Era sobre a unidade — e *uma sonda que responde a pergunta e não
/// gateia nada deixa a resposta por conta de quem a correr da próxima vez*.
#[test]
fn o_pivot_e_relativo_ao_size() {
    let borda_no_mundo = |size: f32, piv: f32| {
        let get = |n: &str| match n {
            ph2d_node_motion_shape::param::KIND => ShapeKind::Circle as i32 as f32,
            ph2d_node_motion_shape::param::SIZE => size,
            ph2d_node_motion_shape::param::PIVOT_X => piv,
            _ => manifest_default(n),
        };
        let (unit, scale) = ShapeParams::read_unit(get);
        let lo = build_shape_path(&unit)
            .verts
            .iter()
            .fold(f64::MAX, |m, v| m.min(v.anchor[0]));
        lo * f64::from(scale)
    };
    for size in [0.5_f32, 1.0, 2.0] {
        // ⛔ O CONTROLO: sem o pivô a aresta está a UM `size` da posição, e é isso que prova que
        // a régua está a olhar para a grandeza que muda. *Sem ele, uma cura que zerasse o
        // deslocamento em todos os tamanhos passaria a metade de cima.*
        let nu = borda_no_mundo(size, 0.0);
        assert!(
            (nu + f64::from(size)).abs() < 1e-9,
            "sem pivot a aresta fica a um `size` da posicao: {nu} contra {size}"
        );
        let com = borda_no_mundo(size, 1.0);
        assert!(
            com.abs() < 1e-9,
            "com `Pivot X = 1` a aresta tem de POUSAR na posicao em qualquer size ({size}): {com}"
        );
    }
}

/// ⭐⭐⭐ **A FORMA RODA EM TORNO DO PIVÔ — medido de ponta a ponta, em quatro ângulos.**
///
/// Report do dono (2026-09-19): *«a rot da forma não está acontecendo a partir do pivot»*. ⚠️ **As
/// duas metades desta lei viviam em sítios diferentes e nenhuma régua as juntava:** a geometria é
/// cortada com o pivô na origem local (`motion_shape_gen`) e a pose põe o ponto local `q` em
/// `P + basis·(q·size)` (`instance_pose`). *Cada uma estava gateada sozinha, e o que o dono vê é a
/// COMPOSIÇÃO.*
///
/// A régua tem duas metades, e a segunda é o discriminador:
/// 1. o ponto do pivô aterra **EXACTAMENTE** em `P`, em qualquer ângulo;
/// 2. o CENTRO da forma **ORBITA** — ele afasta-se de `P` e o afastamento roda com o ângulo.
///
/// ⛔ Sem a (2), um produto que ignorasse o pivô por inteiro passaria a (1) por vacuidade: com o
/// pivô no centro, o centro TAMBÉM aterra em `P`.
#[test]
fn a_forma_roda_em_torno_do_pivot() {
    use crate::motion_state::MotionState;
    // A geometria internada pelo caminho do app, com o pivô na ARESTA esquerda.
    let mut st = MotionState::new();
    let n = st.doc.graph.add_node("source.shape".to_string());
    st.doc
        .graph
        .set_param(n, "kind", ShapeKind::Circle as i32 as f32);
    st.doc.graph.set_param(n, "size", 1.0);
    st.doc
        .graph
        .set_param(n, ph2d_node_motion_shape::param::PIVOT_X, 1.0);
    super::publish(&mut st, 0.0);
    let out = st
        .pump
        .cook
        .cook(&st.doc.graph, &st.registry, n, 0.0)
        .expect("cook");
    let s = out[0].as_stream();
    let Some(ph2d_nodegraph::attr::Column::Scalar(g)) = s.get("geometry_id") else {
        panic!("sem geometria")
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o handle e' um u32 que viaja num f32, como em todo o lowering"
    )]
    let handle = g[0] as u32;
    let path = st.shape_store.get(handle).expect("o path esta' no store");
    let (lo, hi) = path.verts.iter().fold((f64::MAX, f64::MIN), |(l, h), v| {
        (l.min(v.anchor[0]), h.max(v.anchor[0]))
    });
    assert!(
        lo.abs() < 1e-6,
        "com `Pivot X = 1` a origem local tem de ser a ARESTA: [{lo}, {hi}]"
    );
    let centro_local = 0.5 * (lo + hi);

    // E a POSE real, pela mesma função que o `encode` chama.
    let p_mundo = [3.0_f32, -7.0];
    let size = 0.5_f32;
    for graus in [0.0_f32, 37.0, 90.0, 180.0] {
        let (sin_r, cos_r) = graus.to_radians().sin_cos();
        let inst = ph2d_eval_motion::VectorInstance {
            geometry_id: handle,
            texture_id: 0,
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            premultiplied: 0.0,
            world_pos: p_mundo,
            size: [size, size],
            basis: [cos_r, sin_r, -sin_r, cos_r],
            tint: [1.0; 4],
            anchor: [0.0, 0.0],
        };
        let a = super::instance_pose(&inst, ph2d_vector::Affine::IDENTITY);
        // (1) O PIVÔ não se mexe.
        let piv = a * ph2d_vector::Point::new(0.0, 0.0);
        assert!(
            (piv.x - f64::from(p_mundo[0])).abs() < 1e-9
                && (piv.y - f64::from(p_mundo[1])).abs() < 1e-9,
            "a {graus}° o pivô tem de aterrar NA posicao: ({}, {})",
            piv.x,
            piv.y
        );
        // (2) O CENTRO orbita — e o afastamento roda com o ângulo.
        let c = a * ph2d_vector::Point::new(centro_local, 0.0);
        let raio = f64::from(size) * centro_local;
        let (dx, dy) = (c.x - f64::from(p_mundo[0]), c.y - f64::from(p_mundo[1]));
        let folga = 8.0 * f64::from(f32::EPSILON);
        assert!(
            (dx - raio * f64::from(cos_r)).abs() < folga
                && (dy - raio * f64::from(sin_r)).abs() < folga,
            "a {graus}° o centro tem de estar a {raio} NA DIRECCAO do angulo: ({dx}, {dy})"
        );
    }
}

/// ⛔⛔ **NENHUMA LINHA DO CARTÃO DA FORMA TEM O NOME DE UMA LINHA DO CARTÃO DO SINK.**
///
/// Report do dono (2026-09-19): *«a rot da forma não está acontecendo a partir do pivot»* — e ao
/// procurar a causa apareceu esta, que é minha: o `motion.output` já tinha duas linhas chamadas
/// *«Pivot X»/«Pivot Y»* (o pivô do SINK, em fracção do `size` de CADA LINHA e válido para tudo o
/// que ele desenha), e esta wave pôs duas com o MESMO nome no cartão da forma.
///
/// ⚠️⚠️ **Duas linhas com o mesmo nome no mesmo grafo não são um problema de estética:** as
/// unidades diferem (semi-eixo contra extensão), os âmbitos diferem (uma forma contra todo o
/// sink), e o artista que arrasta a errada vê a lei da outra. *Um controlo que se chama como
/// outro é um controlo que mente sobre o que faz.*
///
/// ⚠️ **É um CENSO e não uma asserção sobre dois nomes:** ele varre as duas tabelas registadas,
/// logo uma linha nova de qualquer um dos lados que colida reprova no dia em que nascer.
#[test]
fn nenhuma_linha_da_forma_se_chama_como_uma_do_sink() {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let rotulos = |id| {
        reg.param_ui(id)
            .unwrap_or(&[])
            .iter()
            .map(|h| h.label)
            .collect::<Vec<_>>()
    };
    let da_forma = rotulos(ph2d_node_motion_shape::MANIFEST.id);
    let do_sink = rotulos(ph2d_node_motion_output::MANIFEST.id);
    // ⛔ O piso: sem ele, uma extracção partida deixa as duas listas vazias e o censo fica verde
    // por vacuidade — a forma de falha MUDA que este repo já pagou em censos por prefixo.
    assert!(
        da_forma.len() > 20 && do_sink.len() > 3,
        "as duas tabelas tem de estar lá: {} e {}",
        da_forma.len(),
        do_sink.len()
    );
    // ⚠️⚠️ **UMA isenção NOMEADA, e ela é PRÉ-EXISTENTE — não é desta wave.** O `Collide` do
    // cartão da forma DECLARA um colisor nas peças dela; o do cartão do sink LIGA o passe de
    // separação que os consome. *São as duas metades de uma coisa só, e o artista precisa das
    // duas ligadas* — mas continuam a ser dois controlos com o mesmo nome, e a decisão de os
    // separar é de quem desenhou aquele par (doc 115), não desta linha.
    //
    // ⛔ **E a isenção tem a metade da OBSOLESCÊNCIA:** se alguém renomear um dos dois, esta
    // entrada deixa de descrever alguma coisa e o gate manda apagá-la. *Uma lista de dívida
    // tolerada sem censo de obsolescência não desce: vira licença.*
    const ISENTOS: &[&str] = &["Collide"];
    let colisoes: Vec<&str> = da_forma
        .iter()
        .filter(|l| do_sink.contains(l))
        .copied()
        .collect();
    for i in ISENTOS {
        assert!(
            colisoes.contains(i),
            "a isencao {i:?} ja' nao descreve nada — apague-a desta lista"
        );
    }
    let novas: Vec<&str> = colisoes
        .into_iter()
        .filter(|l| !ISENTOS.contains(l))
        .collect();
    assert!(
        novas.is_empty(),
        "estas linhas chamam-se igual nos dois cartoes, e as leis sao diferentes: {novas:?}"
    );
}
