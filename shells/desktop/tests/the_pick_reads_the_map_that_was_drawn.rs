//! **Arch-gate: o PICK lê o mapa que foi DESENHADO** — a lei que o produto já declarava.
//!
//! ## A lei, e onde ela estava escrita
//!
//! O `vec_gizmo_pick.rs` afirma, no próprio doc-comment:
//!
//! > **A forma que se VÊ é a que se PEGA** … A pergunta *"o que está desenhado aqui?"* é feita ao
//! > **MESMO mapa** que o `ph2d_vec_render::dispatch` consome.
//!
//! ⚠️ **A fiação contradizia-a.** O `dispatch` recebe a FUSÃO de nove produtores de
//! `LiveGeometry` (`render_loop/mod.rs`: offset · pattern · contour · symmetry · profile ·
//! instance, e depois a booleana e o alinhamento, que TRANSFORMAM o mapa); os seis sítios de pick
//! da `input_dispatch` passavam **só o `offset_live`**. Medido pela porta do produto, com uma
//! simetria armada: **3 de 3 pontos da metade espelhada estão desenhados e o clique atravessa**
//! (`vec_gizmo_pick_tests::the_pick_does_not_see_what_the_renderer_draws`).
//!
//! ## Por que um gate de TEXTO
//!
//! Os gates de comportamento entregam o mapa **à mão** ao `pick_at_world`, então provam o
//! hit-test e são **CEGOS à fiação**: com o defeito reinstalado eles ficam todos verdes. Quem
//! decide qual mapa chega é o corpo da `input_dispatch`, que nenhum unit test alcança — a mesma
//! razão do irmão `the_frame_draws_the_live_offset_geometry`.
//!
//! ## A propriedade, e não o endereço
//!
//! Aquele irmão já nasceu vermelho uma vez por ancorar numa **janela de bytes**, e a lição está
//! escrita nele. Aqui nada é medido em distância: o gate **deriva** de qual argumento o
//! `dispatch` recebe e exige que seja esse o valor guardado — se alguém renomear o binding, o
//! gate acompanha; se alguém guardar OUTRO mapa, ele sangra.

const RENDER: &str = include_str!("../src/render_loop/mod.rs");
const INPUT: &str = include_str!("../src/input_dispatch.rs");

/// O campo onde o mapa desenhado descansa até o próximo clique.
const FIELD: &str = "vec_live_drawn";

/// As portas que respondem *"o que está desenhado aqui?"* **no módulo VETORIAL**. Cada uma tem de
/// receber o mapa desenhado — e a lista é o que faz a sétima porta nascer coberta em vez de
/// esquecida.
///
/// ⚠️ **Qualificadas pelo módulo, e não pelo nome nu** — o Flip tem o próprio
/// `flip_gizmo_view::pick_all_at_world`, com outra assinatura e outro mapa. A 1ª versão deste gate
/// procurava `pick_all_at_world(` cru e **nasceu vermelha sobre o pick do Flip**: um nome de função
/// não identifica um assunto quando dois módulos respondem à mesma pergunta sobre coisas
/// diferentes.
const PICK_DOORS: &[&str] = &[
    "vec_gizmo_view::pick_all_at_world(",
    "vec_gizmo_view::pick_in_world_rect(",
    "vec_gizmo_view::contains_world(",
    "envelope_gesture::press(",
];

/// A lista de argumentos de uma chamada que começa em `needle`, do `(` ao `);` correspondente.
fn args_of<'a>(src: &'a str, needle: &str, from: usize) -> Option<(usize, &'a str)> {
    let call = src[from..].find(needle)? + from;
    let end = src[call..].find(");")? + call;
    Some((end, &src[call..end]))
}

/// **O mapa guardado é o MESMO que o `dispatch` desenhou.**
///
/// Derivado, não literal: o gate lê qual argumento vai no `live` do `dispatch` e exige que a
/// atribuição ao campo use esse nome. Guardar um mapa diferente — ou guardá-lo ANTES de a fusão
/// terminar (a booleana e o alinhamento ainda a transformam) — deixaria o pick a ler uma resposta
/// que ninguém desenhou.
#[test]
fn what_is_stored_is_what_was_drawn() {
    let (end, args) = args_of(RENDER, "ph2d_vec_render::dispatch(", 0)
        .expect("a chamada do `dispatch` sumiu do render_loop");
    let live_arg = args
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with('&') && l.ends_with(','))
        .and_then(|l| l.strip_prefix('&'))
        .and_then(|l| l.strip_suffix(','))
        .expect("o `dispatch` não recebe nenhum argumento por referência — assinatura mudou?");
    // O 4º argumento é o `live`; os anteriores também são referências, então a busca acima pega o
    // primeiro. O que interessa é o nome que o `store` usa, e ele TEM de estar entre os args.
    let store = format!("self.{FIELD} = ");
    let at_store = RENDER
        .find(&store)
        .unwrap_or_else(|| panic!("o campo `{FIELD}` deixou de ser escrito no render_loop"));
    assert!(
        at_store > end,
        "o mapa é guardado ANTES de o `dispatch` o receber — a fusão ainda pode ser transformada \
         (a booleana e o alinhamento correm sobre ela), e o pick leria um mapa parcial"
    );
    let stored = RENDER[at_store + store.len()..]
        .lines()
        .next()
        .and_then(|l| l.strip_suffix(';'))
        .expect("a atribuição do mapa desenhado não termina em `;`");
    assert!(
        args.contains(stored),
        "o campo `{FIELD}` guarda `{stored}`, que NÃO é um argumento do `dispatch` — o pick \
         passaria a ler um mapa que ninguém desenhou.\nargumentos:\n{args}"
    );
    let _ = live_arg;
}

/// **Toda porta de pick recebe o mapa desenhado, e nenhuma recebe o `offset_live`.**
///
/// ⚠️ Com CONTROLE POSITIVO nas duas pontas: se a varredura não achar porta nenhuma ela falha em
/// vez de passar vazia — *um gate que percorre uma lista vazia é verde sobre qualquer coisa*.
#[test]
fn every_pick_door_is_handed_the_drawn_map() {
    let mut seen = 0usize;
    for door in PICK_DOORS {
        let mut from = 0usize;
        while let Some((end, args)) = args_of(INPUT, door, from) {
            from = end + 1;
            seen += 1;
            assert!(
                args.contains(FIELD),
                "`{door}` não recebe o mapa desenhado — o que os outros oito produtores de \
                 `LiveGeometry` põem na tela fica visível e NÃO-CLICÁVEL.\nargumentos:\n{args}"
            );
            assert!(
                !args.contains("offset_live"),
                "`{door}` ainda recebe o `offset_live` — é o mapa de UM produtor, e o `dispatch` \
                 desenha NOVE.\nargumentos:\n{args}"
            );
        }
    }
    assert!(
        seen >= PICK_DOORS.len(),
        "a varredura achou {seen} chamadas para {} portas — se o pick mudou de casa, mova este \
         gate com ele em vez de o deixar a passar vazio",
        PICK_DOORS.len()
    );
}
