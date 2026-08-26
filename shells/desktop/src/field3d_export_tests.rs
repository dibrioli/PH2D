//! **Os gates da SAÍDA para arquivo** — e a sonda que decidiu qual número o artista lê.
//!
//! ⚠️ O `field3d_export` em si abre um diálogo nativo (`rfd`) e não é alcançável de um teste. O que
//! se prende aqui é a metade **pura**: o que se conta sobre a malha que de facto saiu.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

fn leaf(p: Primitive, x: f32) -> Node {
    Node {
        xform: Xform::at(x, 0.0, 0.0),
        kind: NodeKind::Leaf(p),
        mods: Vec::new(),
    }
}

fn one(p: Primitive) -> FieldDoc {
    FieldDoc::new(vec![leaf(p, 0.0)], NodeId(0)).expect("uma forma")
}

/// Uma peça **fora do centro**, para o bordo e a peça não coincidirem por acidente.
fn two_apart() -> FieldDoc {
    FieldDoc::new(
        vec![
            leaf(Primitive::Sphere { radius: 0.15 }, -0.6),
            leaf(Primitive::Sphere { radius: 0.15 }, 0.6),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
            },
        ],
        NodeId(2),
    )
    .expect("duas esferas afastadas")
}

fn mesh_of(doc: &FieldDoc, depth: u8) -> ph2d_mesh::Mesh {
    let reg = crate::field3d_smoke::sampled_registry();
    ph2d_field_eval::extract::extract(doc, &reg, depth).expect("extrai")
}

/// ⭐ **A SONDA QUE ESCOLHEU O NÚMERO** — o bordo da grade contra o tamanho da malha.
///
/// Há dois números disponíveis e eles **não são o mesmo**:
///
/// | candidato | o que é | por que NÃO serve |
/// |---|---|---|
/// | a caixa do **bordo** (`bounds::bounding_ball().aabb()`) | o cubo que envolve a **esfera** que contém a peça — e a grade ainda lhe soma `PAD_FRACTION` (5 %) por cima | é **andaime**: conservador por construção, e cúbico — um objeto fino reporta o lado maior nos três eixos |
/// | a caixa da **malha** (`Mesh::bounds()`) | o que de facto foi escrito no arquivo | ⭐ é a resposta à pergunta *"que tamanho isto tem no Blender?"* |
///
/// Rode com `--ignored --nocapture`.
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_grid_box_against_the_real_piece() {
    let reg = crate::field3d_smoke::sampled_registry();
    let cases: [(&str, FieldDoc); 3] = [
        ("esfera r=0,40", one(Primitive::Sphere { radius: 0.4 })),
        (
            "caixa FINA 0,80 x 0,80 x 0,04",
            one(Primitive::Box {
                half: [0.4, 0.4, 0.02],
                round: 0.0,
            }),
        ),
        ("duas esferas afastadas", two_apart()),
    ];
    println!(
        "\n{:<28}  {:>22}  {:>22}  razão",
        "peça", "bordo (x,y,z)", "malha (x,y,z)"
    );
    for (name, doc) in cases {
        let ball = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("tem geometria");
        let (lo, hi) = ball.aabb();
        let grid = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
        let b = mesh_of(&doc, 7).bounds();
        let real = [
            b.max[0] - b.min[0],
            b.max[1] - b.min[1],
            b.max[2] - b.min[2],
        ];
        let ratio: Vec<String> = (0..3)
            .map(|k| format!("{:.2}x", grid[k] / real[k].max(1e-6)))
            .collect();
        println!(
            "{name:<28}  {:>6.3}{:>8.3}{:>8.3}  {:>6.3}{:>8.3}{:>8.3}  {}",
            grid[0],
            grid[1],
            grid[2],
            real[0],
            real[1],
            real[2],
            ratio.join(" ")
        );
    }
}

/// ⭐ **O tamanho que a exportação diz é o da MALHA que saiu** — não o do andaime.
///
/// ⚠️ **A sonda irmã mede a diferença, e ela não é pequena:** a caixa da grade é o cubo que envolve
/// a **esfera** de bordo mais 5 % de folga, então ela é **cúbica** e conservadora por construção.
/// Numa peça fina os dois números divergem por mais de uma ordem de grandeza no eixo curto — dizer o
/// do andaime seria responder *"que tamanho tem a caixa em que eu desenhei"* a quem perguntou
/// *"que tamanho tem a peça"*.
#[test]
fn the_reported_size_is_the_mesh_that_shipped_not_the_grid_that_built_it() {
    let doc = one(Primitive::Box {
        half: [0.4, 0.4, 0.02],
        round: 0.0,
    });
    let mesh = mesh_of(&doc, 7);
    let said = super::piece_size(&mesh);
    let b = mesh.bounds();
    for (k, s) in said.iter().enumerate() {
        let real = b.max[k] - b.min[k];
        assert!(
            (s - real).abs() < 1e-6,
            "o eixo {k} disse {s} e a malha mede {real}"
        );
    }
    // ⭐ O eixo CURTO é o que separa as duas respostas: a caixa da grade é cúbica.
    let reg = crate::field3d_smoke::sampled_registry();
    let ball = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("tem geometria");
    let (lo, hi) = ball.aabb();
    assert!(
        (hi[2] - lo[2]) > said[2] * 5.0,
        "esta fixture existe para os dois números DIVERGIREM no eixo curto — \
         grade {} contra malha {}; se convergiram, ela deixou de provar o que prova",
        hi[2] - lo[2],
        said[2]
    );
}

/// ⚠️ **Uma malha VAZIA não inventa um tamanho.** O `Aabb::EMPTY` é invertido de propósito, e
/// subtrair as pontas dele daria números negativos — que num toast leem como um defeito da peça.
#[test]
fn an_empty_mesh_reports_zero_instead_of_a_negative_size() {
    let empty = ph2d_mesh::Mesh::default();
    assert_eq!(
        super::piece_size(&empty),
        [0.0; 3],
        "uma malha sem vértices tem de reportar zero, não a caixa invertida"
    );
}

/// ⭐⭐ **A EXPORTAÇÃO DECLARA O CONGELAMENTO QUE ELA CAUSA** — o report do Enio de 2026-08-25
/// (*"a mensagem não aparece"*) escrito como gate.
///
/// # ⚠️ O mecanismo é o de 2026-08-22, e a causa é outra
///
/// O relógio do chrome envelhece os toasts com o `wall_dt` do quadro **menos o congelamento
/// declarado** (`modal::chrome_dt`). Uma exportação que congela o loop por minutos e **não declara**
/// faz o quadro seguinte cobrar esses minutos à mensagem que ela acabou de escrever — que morre
/// antes de ser pintada uma segunda vez. ⚠️ Da cadeira isso lê-se como *"o botão não faz nada"*.
///
/// ⛔ **A mutação que este gate mata é exactamente a que reabre o defeito:** tirar o
/// `crate::modal::stalling` de [`super::cook`]. Sem ele o produto compila, exporta certo, e a
/// mensagem some.
///
/// ⚠️ **Ele corre o PRODUTO**, não um censo de texto: um gate que procurasse o nome da porta no
/// arquivo passaria verde sobre uma chamada morta ao lado do caminho real.
#[test]
fn the_export_declares_the_freeze_it_causes() {
    let reg = crate::field3d_smoke::sampled_registry();
    let doc = one(Primitive::Sphere { radius: 0.4 });
    // O relógio é por thread e partilhado com outros gates: começa limpo.
    let _ = crate::modal::take_stall();
    assert!(
        crate::modal::take_stall() <= 0.0,
        "o controle: sem ninguém a congelar, não há nada declarado"
    );

    let (mesh, _) = super::cook(&doc, &reg, super::ExportLevel::Draft).expect("a peça cozinha");
    let declared = crate::modal::take_stall();

    assert!(
        !mesh.positions().is_empty(),
        "a porta não pode comer a malha — sem esta metade, `stalling` podia devolver `default()`"
    );
    assert!(
        declared > 0.0,
        "a exportação congelou o loop e não declarou nada ({declared} s) — a mensagem que ela \
         escreve a seguir vai viver um quadro só"
    );
}

/// ⭐⭐⭐ **QUANTO CUSTA EXPORTAR, pelo caminho do PRODUTO** — o report do Enio de 2026-08-25
/// (*"o tempo de exportação numa malha de 1mi de faces é alto"*) medido onde ele o sentiu.
///
/// ⚠️ **Ela corre a [`super::cook`]**, não uma reconstrução dela: a sonda irmã em `ph2d-field-eval`
/// mede a cadeia, e esta mede a **exportação**, que é a cadeia mais a extração do nível pedido mais
/// o censo de arestas da malha que ficaria. *Uma sonda que salta a costura mede a metade que já se
/// sabia.*
///
/// ```text
/// cargo test -p ph2d-host-desktop --release -- --exact \
///     field3d_export::tests::measure_the_export_wall_clock --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_the_export_wall_clock() {
    use crate::field3d_export::ExportLevel;
    let reg = crate::field3d_smoke::sampled_registry();
    let doc = one(Primitive::Sphere { radius: 0.45 });
    println!("nível | prof | ms | quads que saem | veredito");
    for level in ExportLevel::ALL {
        let t0 = std::time::Instant::now();
        let (mesh, verdict) = super::cook(&doc, &reg, level).expect("cozinha");
        println!(
            "{:>6} | {:>4} | {:>6.0} | {:>14} | {verdict:?}",
            level.key().rsplit('.').next().unwrap_or("?"),
            level.depth(),
            t0.elapsed().as_secs_f32() * 1000.0,
            mesh.faces().len(),
        );
    }
}

/// ⭐⭐⭐ **A GRADE QUE ALIMENTA A CADEIA É A DO `Draft`, NÃO A DO NÍVEL PEDIDO** — a cura do report
/// do Enio de 2026-08-25 (*"o tempo de exportação numa malha de 1mi de faces é alto"*).
///
/// ⛔ **A mutação que este gate mata custava 8 min 15 s**: alimentar a cadeia com a malha do nível.
/// Medido na esfera, nível `Max` (1 120 158 quads de entrada): **495 244 ms**, dos quais 97 % é a
/// fase zero a mastigar um milhão de faces até 2 436 quads — e o resultado é `6,4°`, **o mesmo** que
/// a grade do `Draft` dá em 4 613 ms.
///
/// ⚠️ **A régua é a CONTAGEM de faces, não a profundidade**: `meshes_for` podia devolver a
/// profundidade certa e extrair a malha errada. As duas malhas têm de ser malhas diferentes, e a
/// que alimenta tem de ser a que o `Draft` produz.
#[test]
fn the_grid_that_feeds_the_chain_is_the_draft_grid() {
    use crate::field3d_export::ExportLevel;
    let reg = crate::field3d_smoke::sampled_registry();
    let doc = one(Primitive::Sphere { radius: 0.45 });
    let draft = mesh_of(&doc, ExportLevel::Draft.depth());

    // No `Draft` não há segunda extração a fazer — e dizê-lo com `None` é o que impede a
    // exportação mais barata de pagar duas.
    let (feed, mesh) = super::meshes_for(&doc, &reg, ExportLevel::Draft).expect("cozinha");
    assert!(
        feed.is_none(),
        "no nível da própria grade da cadeia não pode haver uma segunda extração"
    );
    assert_eq!(mesh.faces().len(), draft.faces().len());

    for level in [ExportLevel::Fine, ExportLevel::Max] {
        let (feed, mesh) = super::meshes_for(&doc, &reg, level).expect("cozinha");
        let feed = feed.expect("acima do Draft a cadeia come outra malha");
        assert_eq!(
            feed.faces().len(),
            draft.faces().len(),
            "a cadeia tem de comer a grade do Draft, e comeu {} faces",
            feed.faces().len()
        );
        // ⚠️ **A metade JUSTA**: sem ela, um `meshes_for` que devolvesse a grade do Draft nas DUAS
        // posições passaria — e o artista levaria a malha grossa no nível que pediu fino.
        assert!(
            mesh.faces().len() > feed.faces().len() * 2,
            "a malha que o artista leva é a do NÍVEL: {} contra {} do Draft",
            mesh.faces().len(),
            feed.faces().len()
        );
    }
}

/// ⭐ **SERIALIZAR TAMBÉM DECLARA O CONGELAMENTO** — a outra metade do report do Enio de
/// 2026-08-25.
///
/// ⚠️ **Meio congelamento declarado é uma mensagem que morre metade das vezes.** Cozer a malha e
/// escrevê-la em texto são duas paradas do loop, e a segunda cresce com a peça: um OBJ de 934 k
/// triângulos são dezenas de MB. ⛔ A mutação que este gate mata é tirar o `stalling` de
/// [`super::bytes_of`] — o arquivo continua correcto e a mensagem volta a sumir nas peças grandes.
#[test]
fn the_serialisation_declares_the_freeze_it_causes() {
    let doc = one(Primitive::Sphere { radius: 0.4 });
    let mesh = mesh_of(&doc, crate::field3d_export::ExportLevel::Draft.depth());
    let _ = crate::modal::take_stall();

    let bytes = super::bytes_of(ph2d_mesh::MeshFormat::Obj, &mesh);
    let declared = crate::modal::take_stall();

    assert!(
        bytes.len() > 1024,
        "a porta não pode comer os bytes: saíram {}",
        bytes.len()
    );
    assert!(
        declared > 0.0,
        "escrever a malha congelou o loop e não declarou nada ({declared} s)"
    );
}
