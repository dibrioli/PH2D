//! **Arch-gate: o chip de variant religa a instância pela porta do SWAP** (plano UI/UX W5c).
//!
//! # Porque é um arch-gate, e não um gate de unidade
//!
//! Os gates de `vec_variants::tests` provam as duas PORTAS — que `target_of` endereça o irmão
//! certo e que `swap_main` religa e muda o desenho. Todos passariam com a fiação a escrever
//! `inst.main` **à mão**: a instância mudaria de mestre e ficaria a guardar overrides que apontam
//! peças do mestre ANTIGO — diferenças que nada desenha, e que o *Reset Overrides* continuaria a
//! oferecer. É a regra que a W5b mediu, e ela mora dentro do `swap_main`, não no chamador.
//!
//! Essa metade vive no laço de frame, que exige janela — nenhum teste de unidade a alcança. Mesma
//! classe do `the_arrange_buttons_write_the_z`.

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// O braço do `match` que honra um verbo de componente — do `=>` à chave que o fecha.
///
/// ⚠️ Conta CHAVES em vez de ancorar no braço seguinte: um vizinho que muda de nome levaria a
/// janela junto, que é o proxy que esta suíte já viu expirar duas vezes.
fn arm_of(s: &str, sig: &str) -> String {
    let at = s
        .find(sig)
        .unwrap_or_else(|| panic!("`{sig}` mudou de forma — reancore este gate"));
    let open = at + s[at..].find('{').expect("o braco tem corpo");
    let mut depth = 0i32;
    for (i, c) in s[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return s[open..=open + i].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("chaves desequilibradas a partir de `{sig}`");
}

/// **O clique num chip resolve pelo `target_of` e religa pelo `swap_main`.**
#[test]
fn the_variant_arm_resolves_and_swaps() {
    let arm = arm_of(
        &src("render_loop/mod.rs"),
        "ComponentEdit::Variant(axis, value) =>",
    );
    assert!(
        arm.contains("vec_variants::target_of("),
        "o braço do variant não resolve o alvo pela porta que PUBLICOU os chips — uma segunda \
         travessia daria uma ordem que pode divergir, e o chip `Large` escolheria `Medium`"
    );
    assert!(
        arm.contains("vec_component_pieces::swap_main("),
        "o braço do variant não religa pela porta do Swap — escrever `main` à mão deixaria a \
         cópia a guardar diferenças que apontam peças do mestre ANTIGO"
    );
}

/// **E ele NÃO escreve o vínculo por outra via.**
///
/// ⚠️ A metade que o gate acima não cobre: `target_of` e `swap_main` podiam estar lá **e** uma
/// terceira linha escrever `main` depois, o que é o defeito com todos os gates verdes.
#[test]
fn the_variant_arm_never_writes_the_link_by_hand() {
    let arm = arm_of(
        &src("render_loop/mod.rs"),
        "ComponentEdit::Variant(axis, value) =>",
    );
    for forbidden in [".main =", "VecInstance::new(", "insert(inst"] {
        assert!(
            !arm.contains(forbidden),
            "o braço do variant escreve o vínculo por `{forbidden}` — o descarte dos overrides \
             incompatíveis vive dentro do `swap_main`, e uma segunda escrita o pula em silêncio"
        );
    }
}

/// **As fileiras publicadas saem da MESMA porta que resolve o clique.**
///
/// ⚠️ Sem isto o painel poderia pintar chips de uma travessia e o clique resolver noutra — as duas
/// dariam a mesma resposta hoje, e seria por onde passariam a divergir amanhã.
#[test]
fn the_published_rows_come_from_the_resolving_door() {
    let s = src("render_loop/mod.rs");
    let at = s
        .find("set_variant_rows(")
        .expect("o sítio que publica os variants mudou de forma — reancore este gate");
    let before = &s[at.saturating_sub(1200)..at];
    assert!(
        before.contains("vec_variants::rows_and_targets("),
        "as fileiras publicadas não vêm do `rows_and_targets` — a lista que o artista vê deixaria \
         de ser a que o clique resolve"
    );
}
