//! ⭐⭐⭐ **UM APERTO NO CANVAS LARGA O TECLADO QUE UM CAMPO DO PAINEL SEGURAVA — e a ORDEM é a lei.**
//!
//! # O report
//!
//! Enio, 2026-09-07: *«a tecla del para deletar o mesh parou de funcionar e não temos undo/redo para
//! Cloth»*. **Um defeito, dois sintomas, e nenhum deles é do pincel de tecido.**
//!
//! Cada fileira de slider do painel de escultura regista um **chip numérico**
//! (`InteractiveState::NumberInput`) ao lado do cursor. Tocar num põe o foco do teclado nele — e
//! voltar ao barro **não o tirava**: o `sculpt3d_pointer_down` toma o aperto e devolve `return`
//! antes do `forward_to_hero`, que é o **único** sítio onde a partida de foco corria. `focus_id`
//! ficava preso naquele chip para o resto da sessão, e o `sculpt3d_key` recusa na primeira linha
//! quando um campo de texto tem o foco ⇒ morriam juntos `Delete`, `Ctrl+Z`, `Ctrl+Shift+Z` e **todo**
//! atalho da cena 3D.
//!
//! # ⚠️ Por que este gate lê o FONTE, dito na cara
//!
//! A propriedade é **posicional dentro de uma função** — *a soltura corre antes de quem devolve
//! cedo* —, e o caminho vivo dela exige uma `AppGfx` com surface de janela real, que um teste não
//! constrói (a mesma cerca que o undo do filtro 3D já nomeou). ⛔ Um gate que chamasse uma função
//! pura de decisão mediria **se dois booleanos se combinam**, e não a única coisa que pode partir:
//! alguém mover a soltura para depois de um `return`, ou apagá-la.
//!
//! ⚠️ **E ele recusa ficar VÁCUO**: cada âncora é exigida por si. Se um consumidor for renomeado, o
//! gate reprova a dizer que perdeu a âncora — nunca passa por não a ter encontrado. *Um censo que
//! aprova quando não acha nada é a forma mais cara de gate verde.*

/// O corpo do `on_mouse_input`, do cabeçalho dele até ao `fn` seguinte no mesmo nível.
fn corpo_do_on_mouse_input(fonte: &str) -> &str {
    let inicio = fonte
        .find("pub(crate) fn on_mouse_input(")
        .expect("o `on_mouse_input` mudou de nome — este gate perdeu o sujeito");
    let resto = &fonte[inicio..];
    // O `fn` seguinte ao mesmo nível de indentação fecha o corpo. Sem fim explícito o gate mediria
    // o ficheiro inteiro, e uma soltura escrita 2 000 linhas abaixo passaria.
    let fim = resto[1..]
        .find("\n    pub(crate) fn ")
        .or_else(|| resto[1..].find("\n    fn "))
        .map_or(resto.len(), |i| i + 1);
    &resto[..fim]
}

/// ⭐⭐ **A soltura corre ANTES de cada consumidor que devolve cedo.**
///
/// Os três — a cena de escultura, a janela de modelagem e a alça do gizmo de âncora — tomam o
/// aperto e devolvem antes do despachante. ⚠️ **Um quarto que nasça abaixo da soltura herda a cura
/// de graça**; um que nasça acima dela é exactamente o defeito de 07/09 outra vez, e a cura é mover
/// a chamada, nunca alargar este gate.
#[test]
fn a_soltura_do_foco_corre_antes_de_quem_toma_o_aperto_e_devolve_cedo() {
    let fonte = crate::input_text::dispatch();
    let corpo = corpo_do_on_mouse_input(&fonte);

    let soltura = corpo.find("forward_blur_to_hero(").unwrap_or_else(|| {
        panic!(
            "o `on_mouse_input` deixou de largar o foco do painel: sem isso, tocar num chip \
             numerico e voltar ao canvas mata `Delete` e `Ctrl+Z` da cena 3D para o resto da sessao"
        )
    });

    for consumidor in [
        "self.sculpt3d_pointer_down(",
        "self.field3d_pointer_down(",
        "self.try_open_anchor_gizmo_drag(",
    ] {
        let encontrado = corpo.find(consumidor).unwrap_or_else(|| {
            panic!(
                "este gate perdeu a ancora `{consumidor}` — ele foi renomeado ou saiu do \
                 `on_mouse_input`, e sem ela o gate aprovaria sem medir nada"
            )
        });
        assert!(
            soltura < encontrado,
            "`forward_blur_to_hero` corre DEPOIS de `{consumidor}`, que devolve cedo — o foco do \
             painel fica preso e o teclado daquele modulo morre (report do Enio, 2026-09-07)"
        );
    }
}

/// ⚠️ **A soltura pergunta pelo CHROME, e a pergunta tem de ser a mesma que os consumidores fazem.**
///
/// ⛔ Se ela usasse outra régua, um aperto que a cena toma poderia não ser um aperto que a soltura
/// solta — e o defeito voltava só para uma parte da moldura, que é a forma dele mais cara de achar.
#[test]
fn a_soltura_usa_a_mesma_regua_de_chrome_que_o_consumidor() {
    let fonte = crate::input_text::dispatch();
    let corpo = corpo_do_on_mouse_input(&fonte);
    let bloco = &corpo[corpo
        .find("forward_blur_to_hero(")
        .expect("sem soltura não há régua a conferir")
        .saturating_sub(600)..];
    assert!(
        bloco[..700.min(bloco.len())].contains("pointer_over_chrome"),
        "a soltura deixou de se guardar por `pointer_over_chrome` — a régua dela e a do \
         `sculpt3d_pointer_down` divergiram"
    );
}
