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
/// Duas formas, porque os hosts usam as duas: `p.verts = …` (blend, morph, envelope, live
/// shape) e `p.verts.clear()` seguido de `extend` (o conector, que remonta a polilinha).
const VERTS_REWRITE: [&str; 2] = [".verts = ", ".verts.clear()"];

/// Só hosts vivos: `*_live.rs`. Exclui os `*_tests.rs` (que montam fixtures e escrevem
/// `verts` legitimamente) e o `blend_smoke.rs` — e evita o falso positivo que o gate irmão
/// (`settle_skips_every_derived_geometry`) pagou ao varrer por nome de arquivo largo demais.
fn live_hosts() -> Vec<(String, String)> {
    fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .expect("src/")
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with("_live.rs"))
        .filter_map(|n| {
            let src = fs::read_to_string(format!("{}/src/{n}", env!("CARGO_MANIFEST_DIR"))).ok()?;
            VERTS_REWRITE
                .iter()
                .any(|m| src.contains(m))
                .then_some((n, src))
        })
        .collect()
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
        hosts.len() >= 5,
        "só {} hosts vivos reescrevem `verts` — a forma viva, o conector, o blend, o morph e \
         o envelope existem, então o detector cegou (alguém mudou como se reescreve a \
         geometria?). Um gate que não vê nada passa sempre.",
        hosts.len()
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
