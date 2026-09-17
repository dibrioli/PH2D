//! ⭐⭐⭐ **O CENSO DAS COSTURAS DA PELE — quem lê a malha já POSADA** (F9 W0, `docs/Skeleton/01_a_fila.md`).
//!
//! # Porque este censo existe ANTES de uma linha de código da F9
//!
//! Hoje a CPU deforma cada vértice de cada imagem presa, por quadro
//! ([`ph2d_skeleton_live::skin_image::attach_skin_meshes`]), e pousa o resultado **já posado** num
//! `SpriteMesh` da instância. A F9 quer que a **placa** passe a posar — e no dia em que isso
//! acontecer o `SpriteMesh` deixa de conter posições posadas e passa a conter o **repouso** mais os
//! índices e pesos dos ossos.
//!
//! ⛔⛔ **Todo leitor que hoje espera posições POSADAS na CPU parte-se nesse dia — e parte-se em
//! SILÊNCIO:** ele continua a receber uma malha, com o número certo de vértices, na posição de
//! REPOUSO. O anel do pincel desenha-se onde a arte não está; a caixa do gizmo envolve o sítio
//! errado; o ponteiro aponta para outro texel. É à letra o *«controlo desenhado por um mapa e
//! agarrado por outro»* que esta fila já pagou uma vez (F6-m).
//!
//! # O que este ficheiro é, e o que ele NÃO é
//!
//! Ele **não** decide nada sobre a GPU. Ele fixa a POPULAÇÃO: *estes são os leitores, e cada um traz
//! escrita a resposta que vai precisar*. Um leitor novo que apareça durante a W1–W3 reprova aqui, o
//! que é a diferença entre uma lista que alguém tem de se lembrar de estender e uma que não fica
//! verde sem a extensão.
//!
//! ⚠️ **A fila do módulo listava-os em prosa e a lista estava incompleta** — ela não nomeia a grelha
//! da folha de quadros ([`sheet_lattice`]). *Uma lista escrita à mão ao lado de um scanner é duas
//! respostas à mesma pergunta, e a que envelhece é a escrita.*

use std::path::Path;

/// As três portas públicas que entregam a malha desenhada, mais o próprio componente.
const PORTAS: [&str; 4] = ["drawn_mesh_of", "drawn_instance_of", "mesh_uv", "SpriteMesh"];

/// ⭐ **O LADO DE DENTRO — quem PRODUZ, DESENHA ou DECLARA a malha.**
///
/// Estes não são costuras: são o motor. Eles mudam **com** a F9, por construção, e é por isso que
/// ficam fora da população de leitores (um censo que os contasse mediria o próprio produtor).
const MOTOR: [&str; 7] = [
    "crates/ph2d-render/src/lib.rs",                    // re-exporta as portas
    "crates/ph2d-render/src/picking.rs",                // as três portas vivem aqui
    "crates/ph2d-render/src/sprite_mesh.rs",            // o componente
    "crates/ph2d-render/src/sprite_collect.rs",         // o passe que DESENHA
    "crates/ph2d-render/src/sprite_mesh_warp.rs",       // a deformação
    "crates/ph2d-render/src/sprite_mesh_warp_probe.rs", // a sonda dela
    "crates/ph2d-skeleton-live/src/skin_image.rs",      // `attach_skin_meshes`: o produtor
];

/// ⭐⭐⭐ **AS COSTURAS, e a resposta que cada uma precisa quando a GPU posar.**
///
/// ⚠️ **As respostas são de DUAS espécies, e a distinção é o que torna a W3 orçável:**
/// - **UM PONTO** — quem só precisa de saber onde está *um* sítio (o ponteiro, o anel, a caixa).
///   Esses resolvem-se na CPU **com a mesma lei**, sobre um vértice ou um triângulo, e **não**
///   precisam da malha inteira: o custo é `O(1)`, não `O(vértices)`.
/// - **A MALHA INTEIRA** — quem precisa de todos os vértices posados (os fantasmas do onion, que
///   posam a mesma malha com OUTRA pose). Esses ou continuam na CPU, ou pedem leitura da GPU.
const COSTURAS: [(&str, &str); 10] = [
    (
        "crates/ph2d-app-painter/src/canvas_map.rs",
        "UM PONTO: o mapa de canvas resolve o texel sob um ponto",
    ),
    (
        "crates/ph2d-app-painter/src/painter_bridge_brush_ring.rs",
        "UM PONTO: o anel do pincel segue o cursor sobre a arte",
    ),
    (
        "crates/ph2d-sprite-screen/src/anel_do_pincel.rs",
        "UM PONTO: a mesma lei do anel, do lado da folha partilhada",
    ),
    (
        "crates/ph2d-sprite-screen/src/uv_sob_o_ponteiro.rs",
        "UM PONTO: a UV sob o ponteiro — a porta que o gate do ponteiro defende",
    ),
    (
        "crates/ph2d-sprite-screen/src/sheet_lattice.rs",
        "MALHA: as linhas da grelha de uma folha de quadros seguem a deformacao",
    ),
    (
        "crates/ph2d-timeline-onion/src/lib.rs",
        "MALHA: os fantasmas posam a MESMA malha com as poses de outros instantes",
    ),
    (
        "shells/desktop/src/forwarding_picker.rs",
        "UM PONTO: o conta-gotas amostra o texel que a arte desenha ali",
    ),
    (
        "shells/desktop/src/input_dispatch/painter_canvas_input.rs",
        "UM PONTO: a entrada de canvas do Painter resolve onde o dedo tocou",
    ),
    (
        "shells/desktop/src/render_loop/bgremoval_preview_gpu.rs",
        "MALHA: a previa da Remocao de fundo desenha a arte deformada",
    ),
    (
        "shells/desktop/src/render_loop/snapshots_gizmo.rs",
        "UM PONTO: a caixa do gizmo e' o envelope dos vertices posados",
    ),
];

/// A raiz da workspace. `CARGO_MANIFEST_DIR` = `crates/ph2d-editor-core`.
fn raiz() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
}

/// Todo `.rs` de produto da workspace (⛔ sem testes: uma fixtura pode ler a malha à vontade).
fn fontes() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for base in ["crates", "shells"] {
        varre(&raiz().join(base), &mut out);
    }
    out
}

fn varre(dir: &Path, out: &mut Vec<(String, String)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target" || n == "tests") {
                continue;
            }
            varre(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs")
            && !p.file_name().is_some_and(|n| {
                n.to_string_lossy().ends_with("_tests.rs") || n.to_string_lossy() == "tests.rs"
            })
        {
            let Ok(s) = std::fs::read_to_string(&p) else {
                continue;
            };
            // ⚠️ A prosa SAI antes de varrer: um doc-comment que EXPLICA a costura contém o nome da
            // porta, e sem isto o censo lê a própria explicação como se fosse um leitor.
            let codigo: String = s
                .lines()
                .map(|l| l.split_once("//").map_or(l, |(antes, _)| antes))
                .collect::<Vec<_>>()
                .join("\n");
            let rel = p
                .strip_prefix(raiz())
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, codigo));
        }
    }
}

/// Quem toca numa das portas, tirando o motor.
fn leitores() -> Vec<String> {
    fontes()
        .into_iter()
        .filter(|(rel, src)| {
            !MOTOR.contains(&rel.as_str()) && PORTAS.iter().any(|porta| src.contains(porta))
        })
        .map(|(rel, _)| rel)
        .collect()
}

/// ⭐ **CONTROLO POSITIVO do scanner** — sem isto, um `varre` partido devolveria zero leitores e
/// **as duas metades abaixo ficariam verdes por vácuo**, que é a forma muda que este repo já pagou.
#[test]
fn o_scanner_ve_o_motor_e_ve_as_portas() {
    let fontes = fontes();
    assert!(
        fontes.len() > 400,
        "o scanner varreu so' {} ficheiros — ele esta' partido",
        fontes.len()
    );
    for m in MOTOR {
        let (_, src) = fontes
            .iter()
            .find(|(rel, _)| rel == m)
            .unwrap_or_else(|| panic!("o motor `{m}` mudou de sitio — o censo perdeu o sujeito"));
        assert!(
            PORTAS.iter().any(|p| src.contains(p)),
            "`{m}` esta' declarado como MOTOR e ja' nao toca em malha nenhuma"
        );
    }
}

/// ⛔⛔ **NENHUM leitor fora da tabela.** Um leitor novo aparece aqui — e não no dia do smoke.
#[test]
fn ninguem_le_a_malha_posada_sem_estar_no_censo() {
    let esperados: Vec<&str> = COSTURAS.iter().map(|(f, _)| *f).collect();
    let novos: Vec<String> = leitores()
        .into_iter()
        .filter(|rel| !esperados.contains(&rel.as_str()))
        .collect();
    assert!(
        novos.is_empty(),
        "leitor(es) da malha posada fora do censo da F9: {novos:?}\n\
         Acrescente cada um a `COSTURAS` COM a resposta que ele precisa quando a placa posar \
         (UM PONTO ou a MALHA inteira) — e' isso que torna a W3 orcavel"
    );
}

/// ⛔ **E toda entrada da tabela ainda descreve alguém** — a metade que impede a catraca de virar
/// licença: uma linha órfã lê-se como trabalho pendente para sempre.
#[test]
fn toda_costura_do_censo_ainda_existe_e_ainda_le_a_malha() {
    let vivos = leitores();
    for (f, resposta) in COSTURAS {
        assert!(
            vivos.contains(&f.to_string()),
            "`{f}` esta' no censo da F9 ({resposta}) e ja' nao le' malha nenhuma — \
             apague a linha, senao ela descreve trabalho que nao existe"
        );
        assert!(
            resposta.starts_with("UM PONTO") || resposta.starts_with("MALHA"),
            "`{f}`: a resposta tem de dizer a ESPECIE (`UM PONTO` ou `MALHA`), e diz `{resposta}`"
        );
    }
}
