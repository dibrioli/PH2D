//! **ONDE AS COISAS FICAM na janela do Input Map** — a terceira irmã de [`super`], cortada por
//! teto de LOC (700) e por responsabilidade: ali *como se desenha*, no [`super::apply`] *o que os
//! gestos fazem*, e aqui **a sequência das linhas e as larguras** — as duas perguntas que quem
//! conta e quem pinta têm de responder com o MESMO número.
//!
//! ⚠️ **Foi a divergência entre estas duas contas que produziu os dois reports com foto de
//! 2026-08-24** (*"estreito e sem scroll"* e *"labels emboladas"*): a altura saía de uma função e
//! a sequência do desenho era re-derivada à mão dentro do laço.

use super::InputMap;
use ph2d_i18n::tr;
use ph2d_input::ActionId;
use ph2d_tokens::{ROW_H_PX, Spacing};

/// **A largura do cartão, DERIVADA dos tokens** — não um número cravado.
///
/// ⛔⛔ **Report do Enio (2026-08-24, com foto): *"estreito e sem scroll"*.** A primeira largura
/// (`9 × Xl4` = 432 px) foi calculada para uma linha que já não existe — e era estreita **por
/// baixo do limiar do widget**: o `paint_slider_with_chip` **EMPILHA o rótulo numa linha própria**
/// quando o espaço aperta ([`slider_with_chip_is_stacked`]), e o doc dele diz *"quem chama tem de
/// avançar"*. Eu não avancei ⇒ os números caíam **por cima da acção seguinte** e a última saía do
/// cartão. *Um widget que muda de forma sob pressão exige que quem o coloca lhe pergunte a altura.*
///
/// A largura agora é **`13 × Spacing::Xl4`** (medido: `Xl4 = 48` ⇒ **624 px**), e sai da linha mais
/// larga, que é a da **ACÇÃO**:
///
/// | parte | largura |
/// |---|---|
/// | nome da acção | ~130 |
/// | dois `slider_with_chip` (rótulo 36 + número 48 + trilho mínimo 60 + folgas) | `2 × 160` = 320 |
/// | os dois ícones (`+` e lixo) | `2 × 28` = 56 |
/// | margens e folgas | ~40 |
///
/// ⇒ ~546 px de conteúdo, e o `13 × Xl4` é o degrau de token que o cobre com folga para um nome
/// de acção longo. ⚠️ O trilho mínimo **é do widget** — abaixo dele o rótulo empilha, e foi isso.
pub(super) fn window_w() -> f32 {
    Spacing::Xl4.px() * 13.0 // LITERAL-PX-OK: multiplicador do token, derivacao acima
}

/// A coluna do rótulo dos dois números. ⚠️ **Mais estreita que a `DEFAULT_LABEL_W` (70) de
/// propósito**: "Dead" e "Press" são curtos, e os 70 px do default empurravam a linha para além do
/// limiar de empilhamento — que foi o defeito da foto.
pub(super) fn zone_label_w() -> f32 {
    Spacing::Xl4.px() * 0.75 // LITERAL-PX-OK: fraccao do token
}

/// A coluna do número. Vem do próprio `number_input`, que é quem sabe quanto um número ocupa.
/// O trilho mínimo que o `slider_with_chip` exige antes de **empilhar** o rótulo.
///
/// ⚠️ **É o espelho do `SLIDER_CHIP_MIN_SLIDER_W` do widget**, e está aqui porque a conta da
/// largura da janela precisa dele ANTES de pintar. O `debug_assert` do pintor e o gate
/// `the_zone_numbers_never_stack_at_the_windows_width` são os dois guardas de que os dois números
/// concordam.
pub(super) const ZONE_MIN_TRACK: f32 = 60.0; // LITERAL-PX-OK: espelho do piso do widget (ver acima)

pub(super) fn zone_chip_w() -> f32 {
    Spacing::Xl4.px() // LITERAL-PX-OK: a coluna do numero, um degrau de token
}

/// **UMA LINHA do corpo** — e esta enumeração é a fonte ÚNICA da altura **e** do desenho.
///
/// ⛔⛔ **Report do Enio (2026-08-24, com foto): *"labels emboladas"*.** O indicador da escuta era
/// pintado **depois** de o cursor vertical já ter avançado, então ele caía em cima da linha da
/// FACE VAZIA e os dois textos desenhavam-se um por cima do outro, ilegíveis. A altura era contada
/// por uma função e o desenho **re-derivava a sequência à mão** dentro do laço — *duas contas da
/// mesma coisa divergem, e estas divergiram dentro do mesmo `for`*. Agora há uma sequência só:
/// quem conta e quem pinta percorrem a MESMA lista, e o `y` de cada linha é o índice dela.
///
/// ⚠️ **A CONTAGEM não pode depender da escuta.** Armar uma acção não pode mudar o tamanho da
/// janela nem empurrar as linhas de baixo — é por isso que a face vazia CARREGA o `armed` em vez
/// de existir uma linha a mais quando a escuta está ligada, e há gate a afirmá-lo
/// (`arming_an_action_never_changes_the_line_count`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum BodyLine {
    /// A linha da acção: nome · Dead · Press · `+` · lixo.
    Action { row: usize },
    /// Uma ligação da acção `row`.
    Binding { row: usize, bi: usize },
    /// A face vazia de uma acção sem ligações — o texto dela MUDA com a escuta.
    Empty { row: usize, armed: bool },
    /// O mapa não tem acção nenhuma. ⚠️ **Face vazia, nunca desaparecimento**: a cura de "não há
    /// rota" é dizer onde ela está.
    NoActions,
}

/// **A SEQUÊNCIA das linhas do corpo** — a porta única de que fala [`BodyLine`].
pub(super) fn body_lines(map: &InputMap, listening: Option<ActionId>) -> Vec<BodyLine> {
    if map.is_empty() {
        return vec![BodyLine::NoActions];
    }
    let mut out = Vec::with_capacity(map.len() * 2);
    for (row, a) in map.actions().iter().enumerate() {
        out.push(BodyLine::Action { row });
        if a.bindings.is_empty() {
            out.push(BodyLine::Empty {
                row,
                armed: listening == Some(a.id),
            });
        } else {
            for bi in 0..a.bindings.len() {
                out.push(BodyLine::Binding { row, bi });
            }
        }
    }
    out
}

/// Quantas linhas o corpo do cartão tem. ⚠️ **Deriva da mesma lista que o pintor percorre**, e
/// passa `None` de propósito: a escuta não move uma linha.
fn body_rows(map: &InputMap) -> usize {
    body_lines(map, None).len()
}

/// **O QUE A FAIXA DO TÍTULO DIZ** — `None` = o título de sempre, `Some` = a escuta.
///
/// Existe pelo motivo exacto da [`super::binding_label`]: *o pintor e o gate precisam da MESMA
/// frase*. E aqui isso não é elegância — foi **medido**.
///
/// ⛔⛔ **A primeira sonda desta lei era a TINTA da janela inteira, e ela SOBREVIVEU à mutação.**
/// Armar uma acção muda a tinta por **duas** razões independentes: a faixa passa a dizer o que se
/// espera, e o `+` daquela linha troca de estilo. Uma mutação que fizesse a faixa dizer sempre
/// `Input Map` deixava o segundo sinal intacto, e o gate ficava verde sobre a janela muda.
/// *Uma sonda que soma dois sinais não diz qual dos dois falhou.*
///
/// ⚠️ Uma escuta apontada a uma acção **que já não existe** cala-se, em vez de entrar em pânico:
/// apagar a linha armada é um gesto que a janela oferece.
pub(super) fn title_text(map: &InputMap, listening: Option<ActionId>) -> Option<String> {
    let a = map.get(listening?)?;
    Some(format!(
        "{} {} \u{2014} {}",
        tr("input_map.listening.title"),
        a.name,
        tr("input_map.listening")
    ))
}

/// **A altura do que NÃO rola** — a faixa do título e a linha do nome novo, mais as margens.
///
/// ⚠️ Está aqui, e não escrita duas vezes, porque o pintor precisa dela para saber onde o corpo
/// começa e a [`input_map_window_size`] para saber quanto sobra. *Era a terceira cópia desta conta
/// no ficheiro.*
pub(super) fn chrome_h() -> f32 {
    Spacing::Sm.px() * 2.0 + (ROW_H_PX + Spacing::Xs.px()) * 2.0
}

/// **O TAMANHO da janela e o TETO da rolagem** — `(largura, altura, rolagem máxima)`.
///
/// ⚠️ **A mesma conta que o pintor faz**, e é por isso que ela mora numa função: a shell precisa do
/// rectângulo para saber se a roda é dela, e duas contas da mesma coisa divergem — com o sintoma a
/// ser *"a roda funciona no meio da janela e não na ponta"*.
///
/// ⚠️ **A altura é a CLAMPADA — a mesma que o pintor desenha.** A primeira versão devolvia a
/// pedida e o doc dela afirmava o contrário; a auditoria de 2026-08-24 mostrou que a roda e o
/// arrasto passavam a testar um rectângulo que **não está na tela** assim que a lista transborda.
/// *Um doc que afirma o que a função não faz é pior que nenhum doc.*
#[must_use]
pub fn input_map_window_size(map: &InputMap, viewport_h: f32) -> (f32, f32, f32) {
    let row_h = ROW_H_PX;
    let gap = Spacing::Xs.px();
    let chrome_h = chrome_h();
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de linhas
    let want_body = (row_h + gap) * body_rows(map) as f32;
    // ⛔⛔ **O TETO É O TRANSBORDO, não o conteúdo inteiro** — auditoria 2026-08-24, apanhado por
    // QUATRO lentes independentes. Com o conteúdo inteiro, a roda levava a lista `body_h` px para
    // ALÉM do fim: o cartão ficava **vazio** e nada na tela dizia como voltar.
    //
    // ⚠️ E a altura devolvida é a **CLAMPADA**, a mesma que o pintor desenha. A versão anterior
    // devolvia a pedida, e o doc dela afirmava que clampava — então a roda e o arrasto testavam um
    // rectângulo que **não é o que está na tela** assim que a lista passa da viewport.
    let want_h = chrome_h + want_body;
    let h = want_h.min(viewport_h.max(chrome_h + row_h));
    let body_h = (h - chrome_h).max(row_h);
    (window_w(), h, (want_body - body_h).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::{BodyLine, body_lines, body_rows, title_text};
    use ph2d_input::InputMap;

    /// **Um mapa sem acção nenhuma ainda tem UMA linha** — a face vazia.
    ///
    /// ⚠️ Alcançável: o mapa nasce com os seis verbos do jogador, e apagar os seis é um gesto que a
    /// janela oferece. *A cura de "não há rota" é a face vazia, nunca o desaparecimento.*
    #[test]
    fn an_empty_map_still_has_a_face() {
        assert_eq!(
            body_lines(&InputMap::new(), None),
            vec![BodyLine::NoActions]
        );
        assert_eq!(body_rows(&InputMap::new()), 1);
    }

    /// **A FAIXA DO TÍTULO NOMEIA a acção armada, e diz o que fazer.**
    ///
    /// **Mutação que deve sangrar:** devolver `None` sempre — a faixa volta a dizer só
    /// `Input Map` e armar uma acção que já tem teclas fica **mudo**.
    #[test]
    fn the_title_strip_names_the_action_it_is_listening_to() {
        let mut m = InputMap::with_player_defaults();
        let fresh = m.create("casa");
        assert_eq!(
            title_text(&m, None),
            None,
            "sem escuta a faixa nao pode inventar um aviso"
        );
        let armed = title_text(&m, Some(fresh)).expect("armada, a faixa tem de falar");
        assert!(
            armed.contains("casa"),
            "a faixa nao diz DE QUEM e' a escuta -- com a lista rolada, nada na tela o diz: {armed}"
        );
        assert!(
            armed.contains(ph2d_i18n::tr("input_map.listening")),
            "a faixa nao diz o que fazer a seguir: {armed}"
        );
        // O CONTROLE: apagar a linha armada e' um gesto que a janela oferece, e um id orfao tem de
        // se calar em vez de entrar em panico.
        m.remove(fresh);
        assert_eq!(title_text(&m, Some(fresh)), None);
    }

    /// **A CONTAGEM não depende da escuta** — a metade estrutural do gate de pintura
    /// `arming_an_action_paints_a_sign_without_moving_a_single_row`.
    #[test]
    fn arming_an_action_never_changes_the_line_count() {
        let mut m = InputMap::with_player_defaults();
        let fresh = m.create("casa");
        let calm = body_lines(&m, None);
        let armed = body_lines(&m, Some(fresh));
        assert_eq!(
            calm.len(),
            armed.len(),
            "armar uma accao mudou o numero de linhas: a janela muda de tamanho e as linhas de \
             baixo saltam"
        );
        assert_ne!(
            calm, armed,
            "armar uma accao nao mudou linha nenhuma: a face vazia nao sabe que esta' a escuta"
        );
        // O CONTROLE: a contagem tem de bater com o que o pintor percorre.
        assert_eq!(body_rows(&m), calm.len());
    }
}
