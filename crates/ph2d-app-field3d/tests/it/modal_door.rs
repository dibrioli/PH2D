//! ⭐ **OS MODAIS DESTA FAMÍLIA PASSAM PELA PORTA** — e o gate viajou com o sujeito dele.
//!
//! # ⛔⛔ Por que ele quase morreu VERDE nesta obra
//!
//! Ele vivia em `shells/desktop/src/modal_tests.rs` e varria o `src/` da shell por ficheiros com o
//! prefixo `field3d_`. Quando a família saiu, esse `src/` deixou de ter **um único** ficheiro com
//! aquele prefixo — e a asserção `bad.is_empty()` sobre uma lista construída a partir de zero
//! ficheiros é **trivialmente verdadeira**. *Um gate que varre um directório por nome fica verde no
//! dia em que o directório muda, e não tem como saber que ficou.*
//!
//! ⇒ Duas mudanças, e a segunda é a que importa:
//!
//! 1. O gate mudou-se para a crate da família (o sujeito dele é esta família);
//! 2. ⭐ **Ele passou a exigir um PISO DE POPULAÇÃO.** Varrer zero ficheiros é agora VERMELHO, com
//!    a mensagem a dizer que o gate perdeu o sujeito. É a metade que faltava — e a que faz este
//!    ficheiro valer mais depois da mudança do que antes dela.

/// Quantos ficheiros de produto esta família tinha quando este piso foi escrito (2026-09-11).
///
/// ⚠️ **É um PISO, não uma igualdade:** a família cresce, e um gate que exigisse o número exacto
/// seria uma catraca a reprovar todo ficheiro novo. O que ele barra é a varredura **vazia** e a
/// varredura **decapitada** — os dois modos em que ela deixa de ter sujeito sem dizer nada.
const PISO_DE_FICHEIROS: usize = 40;

#[test]
fn every_field3d_modal_goes_through_the_door() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    let mut vistos = 0usize;
    for entry in std::fs::read_dir(&dir)
        .expect("o src da família existe")
        .flatten()
    {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.ends_with(".rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        vistos += 1;
        // ⚠️ A agulha é a CHAMADA que bloqueia, não o tipo: construir o `FileDialog` é inofensivo
        // (e os dois módulos continuam a fazê-lo, para montar os filtros).
        //
        // ⚠️ **COMENTÁRIOS FORA**, e isto não é higiene: a primeira versão deste gate leu o arquivo
        // inteiro e reprovou sobre o **comentário** que explica a regra — o texto que diz *"nunca
        // chame isto direto"* contém, por construção, exactamente a agulha. Um gate que lê a prosa
        // sobre a lei em vez do código que a obedece reprova quem a documenta.
        for line in text.lines().filter(|l| !l.trim_start().starts_with("//")) {
            // ⛔ **O PLURAL entrou em 22/09, e a ausência dele era um buraco real:**
            //   `.pick_files()` **não contém** `.pick_file()` (o parêntesis fecha antes do
            //   `s`), logo uma importação de vários ficheiros passava este gate sem uma
            //   palavra. A porta plural (`modal::pick_files`) nasceu no mesmo dia, do lado
            //   da escultura, e o irmão dela trouxe esta linha consigo.
            for needle in [".save_file()", ".pick_file()", ".pick_files()"] {
                if line.contains(needle) {
                    bad.push(format!("{name} chama `{needle}` fora da porta"));
                }
            }
        }
    }
    assert!(
        vistos >= PISO_DE_FICHEIROS,
        "este gate varreu {vistos} ficheiros e esperava pelo menos {PISO_DE_FICHEIROS} — ele \
         perdeu o sujeito (a família mudou-se outra vez?) e estava prestes a afirmar-se VERDE \
         sobre nada"
    );
    assert!(
        bad.is_empty(),
        "diálogo modal aberto sem declarar o congelamento — use `ph2d_app_host::modal::save_file` \
         / `pick_file`:\n  {}",
        bad.join("\n  ")
    );
}
