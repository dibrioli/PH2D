//! **Arch-gate: todo host que reescreve `verts` foi CONFRONTADO com a alça de raio.**
//!
//! A alça de raio autora um `corner_radius` **dentro do vértice**. Um host vivo que
//! reescreve `path.verts` varre esse raio — e o sintoma é o pior que existe: a alça
//! aparece, funciona, e o trabalho some um frame depois, sem erro nenhum.
//!
//! Foi assim que a política ficou errada por quatro objetos (ADR-0132 §5): ela perguntava
//! `is_live_shape` — **um** host — e a linha ganhou conector, blend, morph e envelope
//! depois disso. Quem escreveu o 2º não viu erro de compilação, e não havia gate a cobrar.
//! [[feedback_a_condition_that_enumerates_its_readers_rots]]
//!
//! # O que este gate cobra, e o que ele NÃO cobra
//!
//! Ele **não** decide a resposta — recusar não é sempre o certo, e o blend prova isso (a
//! escrita dele é condicional: autorar o spine PARA a reescrita, então lá a alça deve
//! continuar existindo). O que ele cobra é que a pergunta tenha sido **feita**: o host
//! aparece, pelo nome, no módulo que decide.
//!
//! É um contador de símbolos e vale ZERO como auditoria — mas o que ele guarda é uma
//! omissão mecânica, que é exatamente o modo como esta política falhou.

use std::fs;

/// A assinatura de "eu reescrevo a geometria autorada deste path".
///
/// Três formas, porque os hosts usam as três: `p.verts = …` (blend, morph, live shape),
/// `p.verts.clear()` seguido de `extend` (o conector, que remonta a polilinha) e
/// `p.replace_cooked(…)` (a porta única do re-cozimento — o envelope e o texto).
///
/// ⚠️ **A terceira entrou depois de o gate ter cegado por ela.** O `envelope_live` escrevia
/// os campos à mão; quando passou pela porta única, a assinatura dele desapareceu do
/// detector e o controle positivo abaixo caiu de 5 para 4 hosts — exatamente a falha que
/// aquele `assert` existe para gritar. Uma porta NOVA de reescrita tem de entrar aqui no
/// MESMO commit em que nasce, senão este gate passa a guardar menos do que promete.
const VERTS_REWRITE: [&str; 3] = [".verts = ", ".verts.clear()", ".replace_cooked("];

/// As árvores onde um host vivo pode morar, relativas à raiz da workspace.
///
/// ⛔⛔ **A shell deixou de ser a única, e o censo NÃO seguiu sozinho** (W2, 2026-09-12). O
/// `shape_live` — *a forma viva*, o primeiro dos cinco que a mensagem abaixo nomeia — mudou-se
/// para `ph2d-app-vec` numa fase anterior, e este censo continuou a ler `5` porque o
/// `skeleton_live.rs` da shell **também** acabava em `_live.rs` e entrou no lugar dele.
///
/// ⚠️ *O piso segurou o NÚMERO enquanto a POPULAÇÃO trocava por baixo dele* — a mesma forma da
/// catraca sem censo de obsolescência (`CLAUDE.md` §5.0), um nível abaixo: um controlo positivo
/// que conta quantos não vê **quais**. Só quando a pele saiu da shell é que o número caiu e isto
/// ficou visível.
///
/// ⇒ O censo varre as três árvores onde a lei de facto vive. A convenção `*_live.rs` vale nas
/// três, e é por isso que o módulo da pele se chama `skin_live.rs` dentro da crate dele.
const ARVORES: [&str; 3] = [
    "shells/desktop/src",
    "crates/ph2d-app-vec/src",
    "crates/ph2d-skeleton-live/src",
];

/// Só hosts vivos: `*_live.rs`. Exclui os `*_tests.rs` (que montam fixtures e escrevem
/// `verts` legitimamente) e o `blend_smoke.rs` — e evita o falso positivo que o gate irmão
/// (`settle_skips_every_derived_geometry`) pagou ao varrer por nome de arquivo largo demais.
fn live_hosts() -> Vec<(String, String)> {
    // CARGO_MANIFEST_DIR = shells/desktop; dois pais = a raiz da workspace.
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("shells/desktop tem dois pais")
        .to_path_buf();
    let mut achados = Vec::new();
    for arvore in ARVORES {
        let dir = raiz.join(arvore);
        let Ok(entradas) = fs::read_dir(&dir) else {
            panic!(
                "a árvore `{arvore}` do censo não existe — o caminho mudou e este gate passaria a medir menos"
            )
        };
        for e in entradas.filter_map(Result::ok) {
            let Ok(n) = e.file_name().into_string() else {
                continue;
            };
            if !n.ends_with("_live.rs") || n.ends_with("_live_tests.rs") {
                continue;
            }
            let Ok(src) = fs::read_to_string(dir.join(&n)) else {
                continue;
            };
            if VERTS_REWRITE.iter().any(|m| src.contains(m)) {
                achados.push((n, src));
            }
        }
    }
    achados
}

#[test]
fn every_live_host_that_rewrites_verts_is_named_by_the_radius_handle_policy() {
    let policy = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/corner_handles.rs"
    ))
    .expect("corner_handles.rs");

    let hosts = live_hosts();
    assert!(
        hosts.len() >= 6,
        "só {} hosts vivos reescrevem `verts` — a forma viva, a PELE, o conector, o blend, o \
         morph e o envelope existem, então o detector cegou (alguém mudou como se reescreve a \
         geometria, ou um host mudou de árvore sem entrar em `ARVORES`?). Um gate que não vê \
         nada passa sempre. Achados: {:?}",
        hosts.len(),
        hosts.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
    );

    for (host, _) in &hosts {
        let stem = host.trim_end_matches(".rs");
        assert!(
            policy.contains(stem),
            "o host `{host}` reescreve `verts`, e o `corner_handles.rs` nunca o menciona. A \
             alça de raio autora um `corner_radius` DENTRO do vértice: se essa reescrita for \
             incondicional, a alça vai funcionar e o raio vai sumir no frame seguinte, em \
             silêncio. Decida — recusar (junte-o a `has_derived_verts`) ou permitir (como o \
             blend, cuja escrita PARA quando o artista assume o spine) — e escreva a razão lá."
        );
    }
}

/// **A política tem de continuar sendo UMA.**
///
/// Quem DESENHA a alça (`corner_handles::view`) e quem a AGARRA (`input_dispatch`) fazem a
/// mesma pergunta. Se um dos dois passar a perguntar outra coisa, a alça fica invisível e
/// ainda assim agarrável — ou visível e inerte, que é o mesmo bug de costas.
/// [[feedback_two_doors_to_the_same_question_diverge]]
#[test]
fn the_grab_side_asks_the_same_question_as_the_paint_side() {
    let dispatch = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch.rs"
    ))
    .expect("input_dispatch.rs");
    assert!(
        dispatch.contains("corner_handles::has_derived_verts"),
        "o lado que AGARRA a alça deixou de chamar `corner_handles::has_derived_verts` — as \
         duas metades da política divergiram, e a alça passa a ser agarrável onde não é \
         desenhada (ou o contrário)"
    );
}
