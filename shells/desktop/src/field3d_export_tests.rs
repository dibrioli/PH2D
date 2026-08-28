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
        verb: None,
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
                verb: None,
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

/// ⭐⭐⭐ **O QUADRO DRENA A BANCADA** — sem isto a exportação corre, escreve o arquivo, e a
/// mensagem fica pousada para sempre.
///
/// ⚠️ **Ela substitui dois gates que esta wave escreveu e que a wave seguinte tornou obsoletos.**
/// Eles mediam que a exportação DECLARAVA o congelamento (`crate::modal::stalling`) — a cura certa
/// enquanto a conta corria na thread que desenha. ⛔ *Declarar cura a MENSAGEM e não cura o
/// congelamento*: o Enio voltou no mesmo dia com *"o linux fica cinza"*, que é o compositor a dizer
/// que o programa morreu. Com a conta fora da thread não há congelamento a declarar, e um
/// `note_stall` num trabalhador escreveria num `thread_local` que ninguém lê — um no-op silencioso,
/// que é o defeito que este repo persegue.
///
/// ⚠️ **É um censo de texto, e o motivo é o mesmo do irmão** `the_chrome_clock_reads_the_discounted_dt`:
/// o loop não é alcançável de um teste. O que ele prende é a **costura**, não a lei — a lei tem os
/// gates dela em `field3d_export_job`.
#[test]
fn the_frame_drains_the_export_bench() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop/mod.rs"),
    )
    .expect("o loop existe");
    // ⚠️ Comentários fora: a primeira versão do gate irmão reprovou sobre a PROSA que explica a
    // regra, porque o texto que diz "chame isto" contém, por construção, a agulha.
    let code: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        code.contains("field3d_export_job::take_finished()"),
        "o quadro tem de tirar a resposta da bancada — sem isto o arquivo é escrito e o artista \
         nunca sabe"
    );
    // ⚠️ **A metade JUSTA**: tirar sem mostrar seria o mesmo silêncio com mais passos.
    let drain = code
        .find("field3d_export_job::take_finished()")
        .expect("a agulha está lá");
    assert!(
        code[drain..drain + 400].contains("toasts.push"),
        "a resposta tirada tem de ir para os toasts"
    );
}

/// ⭐⭐ **GRAVA POR INTEIRO OU NÃO GRAVA** — e este gate existe por causa de uma consequência da
/// wave anterior.
///
/// ⚠️ Enquanto a exportação corria na thread que desenha, o artista **não conseguia** fechar o app a
/// meio dela. Tirá-la de lá devolveu-lhe essa capacidade — e com ela a janela em que um `write` a
/// meio deixa **meio arquivo com o nome certo**, que abre noutro programa como uma peça partida.
/// *Uma cura pode abrir a porta que outra fechava.*
///
/// ⚠️ **A régua tem DUAS metades**, e a segunda é a que carrega a lei: os bytes certos, **e** o
/// temporário na pasta do DESTINO. Um temporário no `/tmp` faria o `rename` cair para uma cópia
/// quando o destino estivesse noutro disco — que é exactamente o que se está a evitar.
#[test]
fn the_export_writes_all_or_nothing() {
    let dir = std::env::temp_dir().join("ph2d-export-atomic-gate");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a pasta do gate");
    let target = dir.join("peca.obj");

    // Um arquivo que já existe: é o que um `write` truncado estragaria.
    std::fs::write(&target, b"o conteudo antigo").expect("semeia");
    super::write_atomically(&target, b"o conteudo novo").expect("grava");
    assert_eq!(
        std::fs::read(&target).expect("le"),
        b"o conteudo novo",
        "o destino tem de ficar com os bytes novos, inteiros"
    );

    // ⚠️ Nada de restos: um temporário deixado para trás semeia a pasta do artista com arquivos
    // que ninguém sabe apagar.
    let left: Vec<_> = std::fs::read_dir(&dir)
        .expect("lista")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, vec!["peca.obj".to_string()], "sobrou lixo: {left:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// ⭐⭐ **A EXPORTAÇÃO DIZ ONDE A PEÇA ESTÁ — e só quando a pergunta existe.**
///
/// ⚠️ A malha sai em **mundo**, então uma peça modelada longe da origem aterra fora do
/// enquadramento inicial do outro programa: o artista abre o arquivo, vê o vazio, e conclui que a
/// exportação falhou. ⛔ **E a metade que carrega a lei é a do SILÊNCIO**: dizer
/// `(0,00, 0,00, 0,00)` em toda exportação centrada é ruído sobre uma exportação que correu bem.
///
/// ⭐ **O limiar é DERIVADO**: fala quando a **origem está fora da caixa da peça** — que é
/// exactamente a condição em que *"onde é que isto está?"* deixa de ter resposta óbvia. Nada de
/// épsilon: o recurso é a própria caixa.
#[test]
fn the_export_says_where_the_piece_is_only_when_the_question_exists() {
    let depth = crate::field3d_export::ExportLevel::Draft.depth();

    // Centrada: a origem está DENTRO da peça, e o outro programa enquadra-a sozinho.
    let home = mesh_of(&one(Primitive::Sphere { radius: 0.4 }), depth);
    assert!(!home.positions().is_empty(), "a fixtura tem de ter peça");
    assert_eq!(
        super::piece_origin_note(&home),
        "",
        "uma peça na origem não pode gastar a mensagem a dizer que está na origem"
    );

    // Deslocada para fora da própria caixa: agora a pergunta existe.
    let away = mesh_of(
        &FieldDoc::new(
            vec![leaf(Primitive::Sphere { radius: 0.15 }, 2.5)],
            NodeId(0),
        )
        .expect("uma forma longe"),
        depth,
    );
    let note = super::piece_origin_note(&away);
    assert!(
        note.contains("at ("),
        "uma peça fora do enquadramento tem de dizer onde está, e disse {note:?}"
    );
    assert!(
        note.contains("2.5") || note.contains("2.4") || note.contains("2.6"),
        "e o sítio tem de ser o DELA (x ≈ 2,5), não um número qualquer: {note:?}"
    );

    // ⚠️ **A metade da malha VAZIA**: um centro tirado de uma caixa invertida seria um sítio
    // inventado.
    assert_eq!(
        super::piece_origin_note(&ph2d_mesh::Mesh::default()),
        "",
        "uma malha vazia não tem sítio"
    );
}

/// ⭐⭐⭐ **QUANTO CUSTA EXPORTAR, pelo caminho do PRODUTO.**
///
/// ⚠️ **Ela corre a [`super::cook`]**, não uma reconstrução dela: a sonda irmã em `ph2d-field-eval`
/// mede a cadeia, e esta mede a **exportação**, que é a cadeia mais a extração do nível pedido mais
/// o censo de arestas da malha que ficaria. *Uma sonda que salta a costura mede a metade que já se
/// sabia.*
///
/// ⚠️ **Ela foi APAGADA SEM AVISO duas vezes** (nas W63 e W66) por um corte de texto por índice que
/// engoliu o que estava entre dois marcadores. O arquivo compilava, a suíte ficava verde — *um teste
/// apagado e um teste a passar leem-se igual num sumário* — e só se deu por falta ao tentar
/// corrê-la. Recuperada do `afd161ff0`.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bin ph2d-host-desktop -- \
///     --ignored --nocapture measure_the_export_wall_clock
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
            "{:>6} | {:>4} | {:>7.0} | {:>14} | {verdict:?}",
            level.key().rsplit('.').next().unwrap_or("?"),
            level.depth(),
            t0.elapsed().as_secs_f32() * 1000.0,
            mesh.faces().len(),
        );
    }
}
