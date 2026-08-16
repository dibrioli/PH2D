//! **Smoke da UI VIVA** — `PH2D_UI_MOTION_SMOKE=<1|2|3>`.
//!
//! O eixo que o estudo de 2026-08-12 abriu: *o app paga um laço contínuo e desenha uma função
//! escada*. Uma cena por metade do que landou.
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
//!
//! # `=3` — A DOBRA
//!
//! Abre o painel de **física** e manda dobrar uma secção. A F4a deu o `t` ao CABEÇALHO; a F4b
//! deu-o ao CORPO, e a cena existe porque a segunda metade tem **quatro** modos de falha com
//! causas diferentes — o corpo a saltar, o que está por baixo a não subir junto, uma row a
//! transbordar por cima da secção seguinte, e uma row invisível a continuar a responder ao rato.
//! Um roteiro que dissesse só *«veja se desliza»* deixaria três deles passar.
//!
//! ⚠️ **A cena não monta uma secção — ela abre um painel que já as tem.** É a mesma leitura do
//! `=1`: o assunto é chrome, e inventar um painel de demonstração pintaria a dobra num sítio que
//! nenhum artista visita. O painel de física é o escolhido por ser **global** (não pede ferramenta
//! nem selecção) e por trazer secções de altura desigual, que é onde o deslize se lê.
//!
//! ⚠️ **E ela imprime a CONTAGEM que a torna válida**, resolvida do painel (`rows::SECTIONS`) e
//! medida no store (`collapsible_ids`) — nunca um literal aqui, que seria a segunda cópia de um
//! número que o painel declara.

use ph2d_editor::ids;

/// A cena mais nova. ⚠️ **Bumpar isto é o único sítio a tocar quando uma cena entra** — e é ele
/// que impede um `=4` de cair na demo mais recente.
const LAST_SCENE: u32 = 3;

/// Que cena o valor pede. ⚠️ Um valor que não parseia **ou que nomeia uma cena que não existe**
/// cai em **1** — o default de um smoke é a cena mais antiga, nunca a mais nova (`=sim` não pode
/// virar a demo da corda por engano, e um `=4` a caminho de uma cena futura não pode virar a da
/// dobra em silêncio).
fn smoke_level(raw: &std::ffi::OsStr) -> u32 {
    raw.to_str()
        .and_then(|s| s.trim().parse().ok())
        .filter(|n| (1..=LAST_SCENE).contains(n))
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
        if level == 2 && !self.viewport_is_measured() {
            return;
        }

        // ⚠️ **A cena 3 espera o painel PINTAR, e o mecanismo é OUTRO.** O conjunto de secções
        // dobráveis é semeado pelo `populate`, que corre dentro do paint, e o smoke corre no
        // PRÓLOGO do quadro: abrir o painel e contar no mesmo instante dá **zero**, que é
        // indistinguível de *«o painel não montou»* — exactamente o número que a mensagem trata
        // como PARE. Não marcar `done` é o que o faz tentar de novo no quadro seguinte.
        let folds = if level == LAST_SCENE {
            self.open_physics_for_smoke();
            let n = self.collapsible_section_count();
            if n == 0 {
                return;
            }
            n
        } else {
            0
        };
        self.ui_motion_smoke_done = true;

        // O estado ACTUAL, lido do produto — é ele que prova que a wave 3 devolveu a escolha do
        // arranque anterior, e é a única linha do smoke que muda entre duas corridas.
        let (character, reduced) = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| (h.motion.character(), h.motion.reduced_motion()))
            .unwrap_or_default();

        if level == 2 {
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

        // ⚠️ **O `reduced motion` DESLIGA o que as tres cenas medem, e por isso e' um PARE e nao
        //    um readout.** As tres familias que elas exercitam -- `Travel` (o carater), `Surface`
        //    (a dobra e a rolagem) e `Decoration` (a corda) -- devolvem `None` do `law_of` com ele
        //    ligado: sem mola, tudo CHEGA no quadro em que muda. O produto esta' certo e ha' gate
        //    a pina'-lo (`reduced_motion_still_takes_the_surface`); o que estava errado era o
        //    smoke deixar o artista medir a ausencia da feature e ler isso como a feature partida.
        //
        //    ⚠️ E o roteiro conspirava: o passo 3 de cada cena manda LIGAR o reduced. Quem o
        //    deixou ligado de uma corrida anterior comeca no passo 3 a achar que esta' no 1 --
        //    a preferencia sobrevive ao arranque (e' esse o ponto dela) e vem de um ficheiro FORA
        //    do repositorio, logo e' invisivel a toda a varredura.
        if reduced {
            eprintln!(
                "\n[ui-motion-smoke {level}] ⚠️ PARE -- o seu `reduced motion` esta' LIGADO.\n\n  \
                 Ele DESLIGA, de proposito, exactamente o que estas cenas medem: sem mola, tudo\n  \
                 aparece e desaparece no quadro em que muda. Nao ha' o que julgar -- e' a feature\n  \
                 a obedecer-lhe, nao a feature partida.\n\n  \
                 Desligue por um dos dois, e volte a correr:\n\n    \
                   * na app: pill Settings > Motion > Reduced Motion (fica gravado); ou\n    \
                   * no ficheiro: ~/.ph2d/prefs.txt, `reduced_motion=0`.\n\n  \
                 O passo 3 do roteiro manda liga-lo OUTRA VEZ, no fim -- e' ai' que o salto e' a\n  \
                 resposta certa. Comecar com ele ligado e' correr o passo 3 achando que se corre\n  \
                 o passo 1.\n"
            );
            return;
        }

        match level {
            1 => print_character_script(),
            2 => print_tether_script(),
            _ => print_fold_script(folds),
        }
    }

    /// Quantas secções o despacho sabe dobrar **agora** — o conjunto que o `populate` semeou,
    /// somado sobre todos os painéis que já pintaram.
    ///
    /// ⚠️ **Não é `ph2d_panel_physics::rows::SECTIONS.len()`**, e a diferença é o que a torna
    /// oráculo: aquele número diz o que o painel *declara*, este diz o que o store *tem*. Uma
    /// secção declarada que o `populate` esqueceu aparece na diferença entre os dois — e a
    /// mensagem imprime os dois lado a lado por isso.
    fn collapsible_section_count(&self) -> usize {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map_or(0, |h| h.store.collapsible_ids().len())
    }

    /// Abre o painel de FÍSICA — global, logo alcançável sem ferramenta nem selecção.
    ///
    /// ⚠️ **A chave é RESOLVIDA do painel** (`<PhysicsPanel as Panel>::ID`) e não escrita à mão: o
    /// shell já carrega duas cópias do literal `"physics"` no atalho do `W`, e uma terceira aqui
    /// seria a que sobrevive ao dia em que o id mudar — com o smoke a alternar a visibilidade de
    /// um painel que não existe e todos os gates verdes (a cicatriz que o `visibility_key` do
    /// painel autorado documenta).
    fn open_physics_for_smoke(&mut self) {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        hero.panel_visibility.insert(
            <ph2d_panel_physics::PhysicsPanel as ph2d_editor::panel::Panel>::ID,
            true,
        );
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
         \n  ONDE OLHAR: os CHIPS DO RAIL, na coluna da esquerda, e as PILLS DA BARRA DE\n  \
           TOPO. Nao sao as unicas -- hoje o hover interpola no app inteiro --, mas sao\n  \
           as que carregam o CARATER, porque nelas o realce e' TAMANHO. Uma fracao de\n  \
           tinta e' clampada em 1,0, entao a ultrapassagem nao tem onde aparecer numa cor.\n\
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
              Deixe EXPRESSIVE e passe o rato pelos chips outra vez. O que tem de\n     \
              acontecer: o chip PARA de crescer (nada se mexe) e o TINT continua a\n     \
              chegar e a sair. Sao dois eixos, e 'Expressivo + reduced' e' combinacao\n     \
              legitima -- um seletor de tres posicoes tornaria-a inexprimivel.\n     \
              O gatilho vestibular e' a AREA a deslocar-se, nao a tinta a mudar; por isso\n     \
              o fade sobrevive de proposito, e nao por esquecimento.\n\
         \n     Na cena 2 o mesmo toggle faz a CORDA desaparecer inteira -- decoracao em\n     \
              reduced e' AUSENTE, nao atenuada.\n\
         \n  5. ⭐ FECHE O APP e rode este smoke outra vez. A primeira linha do terminal tem\n     \
              de dizer o carater que voce escolheu. Se disser Discreto, a wave 3 falhou.\n"
    );
    eprintln!(
        "[ui-motion-smoke 1] (!) A VARREDURA JA' ACONTECEU, e esta nota dizia o contrario\n  \
         ate' 2026-08-16: ela afirmava que 'tres tipos leem o eixo hoje' e mandava esperar\n  \
         a proxima wave. Medido, sao SEIS familias com porta no store (button, checkbox,\n  \
         toggle, slider, dropdown, scrollbar) mais o IconButton, que fechou a estrada pelo\n  \
         proprio TIPO. Passe o rato por um slider, um checkbox e um polegar de barra de\n  \
         rolagem DENTRO de um painel: eles respondem. Quem moveu o numero que tornava algo\n  \
         inalcancavel tinha de reconferir a nota, e nao reconferiu.\n"
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

/// O roteiro da dobra. `folds` é a contagem MEDIDA no store; ver [`super::App::collapsible_section_count`].
fn print_fold_script(folds: usize) {
    let declared = ph2d_panel_physics::rows::SECTIONS.len();
    eprintln!(
        "\n[ui-motion-smoke 3] A DOBRA — o corpo de uma seccao interpola.\n\
         \n  O VOCABULARIO primeiro, porque o resto nao se le sem ele:\n\
         \n    seccao = cada faixa com TITULO e uma setinha. O painel de FISICA tem cinco:\n             \
                       World · Solver · Air · Damping · Sleep.\n    \
               corpo  = o monte de controles DEBAIXO do titulo (os sliders, os chips).\n    \
               dobrar = clicar no TITULO para fechar/abrir. E' o gesto de sempre.\n\
         \n  O QUE MUDA: ate' agora, clicar no titulo fazia o corpo SUMIR de um quadro para o\n  \
           outro, e as seccoes de baixo SALTAVAM para cima. A setinha ja' girava suave (isso\n  \
           landou em 15/08) e o conteudo debaixo dela pulava -- as duas metades do mesmo\n  \
           cabecalho contavam historias diferentes sobre o mesmo instante. Agora o corpo\n  \
           ENCOLHE e o que esta' abaixo acompanha.\n\
         \n  O painel de FISICA esta' aberto: ele declara {declared} seccoes, e o store tem\n  \
           {folds} dobra(s) semeada(s) (a soma de TODOS os paineis ja' pintados, logo >= {declared}).\n"
    );
    if folds < declared {
        eprintln!(
            "[ui-motion-smoke 3] ⚠️ ha' menos dobras semeadas do que o painel declara — PARE.\n  \
             O `populate` esqueceu uma seccao, e nenhum passo abaixo diz alguma coisa.\n"
        );
        return;
    }
    eprintln!(
        "  1. Settings > Motion > EXPRESSIVE. Clique no titulo WORLD para FECHAR, e julgue\n     \
              QUATRO coisas de uma vez -- sao quatro causas diferentes, e um roteiro que\n     \
              dissesse so' 'veja se desliza' deixava tres passar:\n\
         \n       (a) o CORPO encolhe, em vez de desaparecer de repente;\n\
         \n       (b) SOLVER, AIR, DAMPING e SLEEP sobem JUNTO, acompanhando o encolhimento.\n           \
                   Se elas so' saltarem no fim, o painel nao encolheu com a dobra: ela\n           \
                   ficou decorativa (o 'y' de saida nao esta' escalado);\n\
         \n       (c) nada TRANSBORDA -- nenhum controle do World aparece por cima do titulo\n           \
                   SOLVER enquanto o World fecha (o recorte da CENA);\n\
         \n       (d) a SETINHA do World roda no mesmo compasso do corpo dele. As duas\n           \
                   metades do mesmo cabecalho tem de contar a mesma historia sobre o mesmo\n           \
                   instante; ate' esta wave a setinha rodava e o corpo saltava debaixo dela.\n\
         \n  2. Reabra o World e, A MEIO da abertura, passe o rato na banda que um controle\n     \
              AINDA NAO ocupou -- onde ele vai estar, antes de la' chegar. Nada pode\n     \
              acender. Um controle invisivel que responde ao rato e' o recorte de HIT em\n     \
              falta -- e essa metade NAO se ve': so' se apanha procurando-a.\n\
         \n  3. Settings > Motion > REDUCED MOTION. Dobre outra vez: ela tem de SALTAR.\n     \
              O corpo de um painel a deslizar E' area grande a deslocar-se, que e' o\n     \
              gatilho vestibular -- a dobra entra na familia da SUPERFICIE (a mesma da\n     \
              rolagem de painel), e reduced mata-a. O tint de hover\n     \
              ao lado continua a desvanecer: sao dois eixos, nao um interruptor so'.\n\
         \n  4. DISCRETE e depois EXPRESSIVE, dobrando em cada um. Muda o PESO da chegada,\n     \
              e mais nada: ela nao ultrapassa em nenhum dos dois. Uma seccao que passasse\n     \
              do fim e voltasse mostraria conteudo para alem do fim -- exactamente o que o\n     \
              clamp de cada painel existe para proibir.\n\
         \n  5. ⭐ O CONTROLE, e e' a metade que NAO se ve: com tudo PARADO, o painel tem de\n     \
              estar exactamente como sempre esteve. Aberta e quieta, nenhum recorte e'\n     \
              empurrado e o 'y' sai verbatim; fechada e quieta, o corpo nao e' medido nem\n     \
              pintado. Se um painel parado ficar 1 px diferente, a neutralidade quebrou --\n     \
              e e' ela que deixou isto entrar em dez paineis de uma vez.\n"
    );
    eprintln!(
        "[ui-motion-smoke 3] (!) A LEI E' A MESMA nos dez paineis (inspector, vector,\n  \
         painter-layers, physics, audio-editor, audio-mixer, sculpt3d, wet-tuning,\n  \
         motion-params, authored). Este smoke abre UM porque o roteiro precisa de um sitio,\n  \
         nao porque a dobra seja dele -- dobre noutro painel e tem de ser identico.\n"
    );
    eprintln!(
        "[ui-motion-smoke 3] (!) O QUE FICA DE FORA, de proposito: a galeria de widgets\n  \
         (widget/showcase). Ela nunca recebeu a F4a -- o cabecalho dela nao le o 't' -- e e'\n  \
         ferramenta de dev, nao chrome do app. Migra-la pedia a F4a primeiro.\n"
    );
}

#[cfg(test)]
#[path = "ui_motion_smoke_tests.rs"]
mod tests;
