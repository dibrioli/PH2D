//! **Arch-gate da costura do WIDTH TOOL** (plano 25 §5) — o gesto inteiro é dele.
//!
//! ## O que este gate protege
//!
//! Quatro maneiras de partir a ferramenta deixam **todos os unit tests verdes**, porque nenhum
//! deles alcança o corpo do `input_dispatch`:
//!
//! 1. **o press cai na cadeia de modo** — `shape_kind_for_mode(..).is_none()` é VERDADEIRO no modo
//!    Width, então um braço posto no `else` dele é código morto no único modo capaz de o alcançar
//!    (é literalmente o defeito que o lápis pagou, e está documentado ao lado);
//! 2. **o release não larga a alça** — ela fica agarrada ao dedo depois de solta;
//! 3. **o move não é despachado** — a alça não segue o cursor e o arrasto vira pan da câmera;
//! 4. **o botão direito não apaga** — a ferramenta ganha um verbo a menos, em silêncio.
//!
//! ## As asserções afirmam RELAÇÃO, nunca distância nem formatação
//!
//! Esta linha já perdeu arch-gates por medir bytes no fonte (integração de 2026-07-23) e, nesta
//! jornada, por uma agulha que o `rustfmt` quebrou em duas linhas quando um argumento novo entrou.

const SRC: &str = include_str!("../src/input_dispatch.rs");

/// A posição da 1ª ocorrência, com a mensagem que nomeia o que se perdeu.
fn at(needle: &str) -> usize {
    SRC.find(needle).unwrap_or_else(|| {
        panic!(
            "o `input_dispatch` nao contem `{needle}` — se foi renomeado, atualize este gate (e \
             confira que o Width Tool ainda funciona: `PH2D_BUILD_SMOKE=42`)"
        )
    })
}

/// **Controle positivo:** os âncoras existem. Um scanner que não acha nada passaria em silêncio
/// por todas as asserções abaixo.
#[test]
fn the_scanner_finds_what_it_scans_for() {
    for needle in [
        "ph2d_tool_vector::DrawMode::Width",
        "crate::width_handles::press(",
        "crate::width_handles::drag(",
        "crate::width_handles::remove(",
        "crate::width_handles::discard_if_untouched(",
        "if shape_kind_for_mode(&self.vec_draw_config).is_none() {",
    ] {
        assert!(
            SRC.contains(needle),
            "controle positivo falhou: `{needle}` sumiu do dispatch"
        );
    }
}

/// **O press do Width corre ANTES da cadeia de modo.** No `else` do `shape_kind_for_mode` ele
/// seria código morto — a lição que o release do lápis pagou, no mesmo arquivo.
#[test]
fn the_width_press_runs_before_the_mode_chain() {
    let press = at("crate::width_handles::press(");
    let chain = at("if shape_kind_for_mode(&self.vec_draw_config).is_none() {");
    assert!(
        press < chain,
        "o press do Width corre DEPOIS da cadeia de modo — `shape_kind_for_mode(..).is_none()` e' \
         VERDADEIRO em modo Width, entao o braco nunca seria alcancado"
    );
}

/// **O release larga a alça, e também ANTES da cadeia** — pela mesma razão.
#[test]
fn the_width_release_is_its_own_arm_before_the_mode_chain() {
    let release = at("if let Some(grab) = self.vec_width_grab.take()");
    let chain = SRC[release..]
        .find("if shape_kind_for_mode(&self.vec_draw_config).is_none() {")
        .map(|i| release + i);
    assert!(
        chain.is_some(),
        "o release do Width nao esta' ANTES de nenhuma cadeia de modo — se ele veio parar depois, \
         a alca fica agarrada ao dedo"
    );
    // E o que ele faz é largar + fechar o passo de undo, não abortar um gesto.
    let arm = &SRC[release..chain.unwrap_or(SRC.len())];
    assert!(
        arm.contains("commit_if_changed"),
        "o release do Width nao fecha o passo de undo — o gesto inteiro ficaria sem Ctrl+Z"
    );
}

/// **O move é despachado, e depois do lápis** (a ordem entre os dois é livre; o que importa é que
/// ele exista na cadeia de early-returns, senão o arrasto da alça vira pan da câmera).
#[test]
fn the_width_move_is_dispatched() {
    assert!(
        SRC.contains("self.vec_width_drag_move(self.last_pointer.0, self.last_pointer.1)"),
        "o move do Width nao esta' na cadeia de arrasto — a alca nao seguiria o cursor"
    );
}

/// **O botão direito APAGA a parada**, e o braço vive na cadeia do Secondary — não numa rota de
/// cancelamento (não há gesto em curso a abortar: um clique é um clique).
#[test]
fn the_secondary_button_removes_a_width_stop() {
    let secondary = at("(ph2d_host::PointerButton::Secondary, PointerKind::Down) if on_canvas =>");
    let remove = at("crate::width_handles::remove(");
    assert!(
        secondary < remove,
        "o `remove` do Width nao esta' na cadeia do botao direito"
    );
    // E ele é gateado no modo certo: apagar uma parada num clique direito de OUTRO modo seria
    // mexer no perfil de quem não pediu.
    let head = &SRC[..remove];
    let guard = head
        .rfind("ph2d_tool_vector::DrawMode::Width")
        .expect("o `remove` nao e' gateado pelo modo Width");
    assert!(
        guard > secondary,
        "o guard de modo do `remove` esta' fora da cadeia do botao direito"
    );
}

/// **Um clique que não arrastou é desfeito no release.** Sem isto o artista vê a fita mudar
/// 13,1% da faixa ao simplesmente clicar na curva — o preço estrutural da re-parametrização do
/// `smoothstep`, que o gesto (arrastar para criar) esconde.
#[test]
fn a_bare_click_is_undone_on_release() {
    let release = at("if let Some(grab) = self.vec_width_grab.take()");
    let discard = at("crate::width_handles::discard_if_untouched(");
    assert!(
        release < discard,
        "o `discard_if_untouched` nao corre no release do Width — um clique solto deixaria uma \
         parada e a fita mudaria sem ninguem pedir"
    );
    // E o drag DERRUBA o `created`, senão todo gesto seria desfeito no fim.
    let drag = at("crate::width_handles::drag(");
    assert!(
        SRC[drag..].contains("created: false"),
        "o arrasto nao derruba o `created` — o release desfaria TODO ponto de largura recem-criado"
    );
}
