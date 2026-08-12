//! **Smoke da UI VIVA** — `PH2D_UI_MOTION_SMOKE=<1|2>`.
//!
//! O eixo que o estudo de 2026-08-12 abriu: *o app paga um laço contínuo e desenha uma função
//! escada*. Duas cenas, uma por metade do que landou.
//!
//! ⚠️ **O smoke NÃO arma o carácter.** Ele monta a cena e **manda o artista escolher** em
//! `Settings ▸ Motion` — que é o controlo real, e o único caminho que exercita o menu, a porta
//! única e a persistência de uma vez. É a cicatriz que o `the_smokes_open_the_painter_in_digital`
//! pregou: *um smoke que arma o estado por baixo da mesa salta exactamente a costura que ele
//! existia para provar*.
//!
//! ⚠️ **E o RELÓGIO não se julga aqui.** *A corda dá a mesma forma a 30 e a 120 fps* é propriedade
//! de gate (`the_rope_is_a_fact_of_the_wall_clock_not_of_the_frame_rate`, medido em **0,0000 px**),
//! e um olho não distingue 1 px numa corda a balançar. O que o smoke julga é o LOOK: os números de
//! aparência (`NODES` · `SLACK` · a gravidade de ecrã) saem daqui, como o
//! `RESAMPLE_STEP_FRACTION` do Flip saiu.
//!
//! # `=1` — O CARÁCTER
//!
//! Nada é montado na cena: o assunto é o **chrome**, que já está todo na tela. O roteiro pede três
//! comparações, e a terceira é a que só um segundo arranque responde.
//!
//! # `=2` — A CORDA
//!
//! Abre o card **Fill adjust** no meio do ecrã. Ele é o caso do Pixelator: um card que **nasceu**
//! num sítio (a largada do ColorDrop) e que o artista arrasta dali — a corda liga os dois.
//!
//! ⚠️ **A abertura é SETUP, o arrasto é o teste.** O card só nasce depois de um ColorDrop real no
//! Painter, que é meia dúzia de gestos antes do assunto; o que esta wave construiu é a corda, e o
//! gesto que a exercita é **arrastar o card**. O arrasto é o caminho real e completo (o
//! `arm_fill_modal_drag_if_on_handle` do shell, sem atalhos).

use ph2d_editor::ids;

/// Que cena o valor pede. ⚠️ Um valor que não parseia cai em **1** — o default de um smoke é a
/// cena mais antiga, nunca a mais nova (`=sim` não pode virar a demo da corda por engano).
fn smoke_level(raw: &std::ffi::OsStr) -> u32 {
    raw.to_str()
        .and_then(|s| s.trim().parse().ok())
        .filter(|n| *n >= 1)
        .unwrap_or(1)
}

impl crate::App {
    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn ui_motion_smoke(&mut self) {
        if self.ui_motion_smoke_done {
            return;
        }
        let Some(raw) = std::env::var_os("PH2D_UI_MOTION_SMOKE") else {
            return;
        };
        if self.gfx.is_none() {
            return; // ainda não há mundo; tenta no quadro seguinte
        }
        let level = smoke_level(&raw);

        // ⚠️ **A cena 2 espera o LAYOUT existir, e este `return` é a wave inteira num sítio.**
        // O `hero.last_viewport` é escrito no TOPO do `paint`, e o smoke corre no PRÓLOGO do
        // quadro — no primeiro quadro em que há `gfx` ele ainda mede `0 x 0`. Abrir o card ali
        // punha-o em `(0, 0)` com a âncora no mesmo ponto: comprimento zero, `is_drawable` a
        // recusar, e o card escondido atrás da barra. **O smoke não ficava vermelho — ele
        // montava a cena que não tem o fenómeno**, que é a doença de fixture que este repo já
        // pagou seis vezes. Não marcar `done` é o que o faz tentar de novo no quadro seguinte.
        if level >= 2 && !self.viewport_is_measured() {
            return;
        }
        self.ui_motion_smoke_done = true;

        // O estado ACTUAL, lido do produto — é ele que prova que a wave 3 devolveu a escolha do
        // arranque anterior, e é a única linha do smoke que muda entre duas corridas.
        let (character, reduced) = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| (h.motion.character(), h.motion.reduced_motion()))
            .unwrap_or_default();

        if level >= 2 {
            self.open_fill_card_for_smoke();
        }

        eprintln!(
            "[ui-motion-smoke {level}] carater ao ABRIR: {:?} · reduced motion: {}",
            character, reduced
        );
        eprintln!(
            "[ui-motion-smoke] preferencias em ~/.ph2d/prefs.txt (ausente = os defaults, que \
             sao os de sempre: Discreto, sem reduced)"
        );
        match level {
            1 => print_character_script(),
            _ => print_tether_script(),
        }
    }

    /// O layout já foi medido pelo menos uma vez? (Ver o `return` acima.)
    fn viewport_is_measured(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.last_viewport.w > 1.0 && h.last_viewport.h > 1.0)
    }

    /// Abre o card **Fill adjust** longe da sua âncora, para a corda nascer com comprimento.
    ///
    /// ⚠️ **A posição é ARRASTADA, não escolhida:** o card abre na âncora (é o que um ColorDrop
    /// faz) e o smoke move-o em seguida, pela MESMA porta que o arrasto do artista usa
    /// (`move_fill_modal`). Abri-lo já deslocado exigiria uma segunda porta que só o smoke usa —
    /// e a âncora deixaria de ser *onde ele nasceu*, que é o facto inteiro que a corda desenha.
    fn open_fill_card_for_smoke(&mut self) {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        let v = hero.last_viewport;
        let anchor = (v.x + v.w * 0.30, v.y + v.h * 0.30);
        hero.store.open_fill_modal(anchor.0, anchor.1, 0.5);
        hero.store.move_fill_modal(v.w * 0.28, v.h * 0.34);
        // O slider do card fica vivo de graça; o Done/Cancel encaminha para o PainterTool e sem
        // Painter não faz nada — o que se julga aqui é a CORDA, e o roteiro diz isso.
        let _ = ids::PAINTER_FILL_MODAL_SLIDER;
    }
}

fn print_character_script() {
    eprintln!(
        "\n[ui-motion-smoke 1] O CARATER — o chrome ganhou um relogio, e ele tem duas vozes.\n\
         \n  ONDE OLHAR (e so' aqui, por enquanto): os CHIPS DO RAIL, na coluna da esquerda,\n  \
           e as PILLS DA BARRA DE TOPO. Sao as duas superficies ligadas ao relogio.\n\
         \n  1. Passe o rato POR CIMA de um chip do rail e SAIA, devagar. Duas coisas\n     \
              mudam: o chip CRESCE 3 px e o tint do glifo aquece. E' o crescimento que\n     \
              carrega o carater -- uma fracao de tinta e' clampada em 1,0, entao a\n     \
              ultrapassagem do Expressivo nao teria onde aparecer numa cor.\n\
         \n  2. Abra Settings (a engrenagem) > Motion > EXPRESSIVE. Repita o passo 1:\n     \
              o realce agora CHEGA e SAI. A saida tambem e' suave -- se so' a entrada\n     \
              fosse, seria meia feature (e foi o defeito que a wave 2 curou).\n\
         \n  3. Settings > Motion > DISCRETE. Repita. A diferenca NAO e' 'o mesmo mais\n     \
              devagar': o Discreto CHEGA E ASSENTA sem nunca ultrapassar (zeta = 1, e' a\n     \
              matematica, nao uma promessa). O Expressivo passa 15,5% do tamanho e volta\n     \
              -- o chip POPA. Medido; se nao vir o pop, o numero volta a' mesa.\n\
         \n  4. Settings > Motion > REDUCED MOTION (e' um TOGGLE, nao uma terceira opcao).\n     \
              Com ele ligado, escolha EXPRESSIVE: o fade FICA e o percurso SAI. Sao dois\n     \
              eixos, e 'Expressivo + reduced' e' uma combinacao legitima -- um seletor de\n     \
              tres posicoes tornaria-a inexprimivel.\n\
         \n  5. ⭐ FECHE O APP e rode este smoke outra vez. A primeira linha do terminal tem\n     \
              de dizer o carater que voce escolheu. Se disser Discreto, a wave 3 falhou.\n"
    );
    eprintln!(
        "[ui-motion-smoke 1] (!) O QUE AINDA NAO SE MOVE, e e' esperado: os widgets DENTRO\n  \
         dos paineis (sliders, checkboxes, dropdowns, rows de lista). Catorze tipos de\n  \
         widget tem eixo de hover e tres o leem hoje -- a varredura dos restantes e' a\n  \
         proxima wave, nao um defeito desta. Se o rail e a barra respondem, ela funciona.\n"
    );
    eprintln!(
        "[ui-motion-smoke 1] (!) O que NAO tem de mudar: um clique durante uma transicao e'\n  \
         SEMPRE aceite -- animacao nenhuma atrasa a aceitacao de um gesto. E um NUMERO lido\n  \
         nunca balanca: um valor que oscila esta' ERRADO durante 200 ms, e alguem vai le-lo.\n"
    );
}

fn print_tether_script() {
    eprintln!(
        "\n[ui-motion-smoke 2] A CORDA — o pedido do Pixelator.\n\
         \n  O card 'Fill' esta' aberto no ecra, ja' ARRASTADO para longe do sitio onde\n  \
           nasceu. A corda liga os dois: uma ponta no nascimento, a outra no card.\n\
         \n  1. Settings > Motion > EXPRESSIVE (ela so' simula com decoracao ligada).\n\
         \n  2. Agarre a FAIXA DO TITULO do card ('Fill') e arraste-o pelo ecra. Julgue:\n       \
              - ela PENDURA (nao e' uma linha reta a seguir o card);\n       \
              - e e' uma CURVA LISA, sem lados nem quinas;\n       \
              - ela SEGUE com atraso, como um cordao com peso;\n       \
              - e NAO ESTALA quando voce sacode o card depressa.\n\
         \n  3. Largue e espere: ela assenta e PARA. Uma corda que continua a tremer\n       \
              sozinha esta' com o amortecimento errado.\n\
         \n  4. Settings > Motion > DISCRETE, sem largar o card do olho: a corda vira uma\n       \
              RECTA entre os mesmos dois pontos. A RELACAO sobrevive inteira; o que sai e'\n       \
              o peso. E em Discreto ela nao simula nada -- e' recta, nao 'reta parecida'.\n\
         \n  5. Volte a EXPRESSIVE: ela cai da recta NOVA, nao voa do sitio antigo.\n"
    );
    eprintln!(
        "[ui-motion-smoke 2] (!) OS NUMEROS DE APARENCIA SAO SEUS, e e' para isto que o smoke\n  \
         existe: 28 nos · folga 1,22 (quanto ela e' mais longa que a recta) · gravidade de\n  \
         2600 px/s². Se ela pendurar de menos, a folga sobe; se abanar como gelatina, a\n  \
         gravidade desce. Diga o que ve' e eu mexo -- nenhum destes tres sai de um teste.\n"
    );
    eprintln!(
        "[ui-motion-smoke 2] (!) O que este smoke NAO julga: a corda dar a MESMA forma a 30 e a\n  \
         120 fps. Isso e' gate (medido em 0,0000 px com as pontas paradas) porque um olho nao\n  \
         distingue 1 px numa corda a balancar -- e era 29,7 px antes da correcao.\n"
    );
}
