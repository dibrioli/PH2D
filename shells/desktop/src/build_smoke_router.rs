//! **Qual MÓDULO é dono deste nível de smoke** — irmão do [`super::build_smoke`] pelo teto de 600
//! LOC da shell (HR-18), cortado por RESPONSABILIDADE.
//!
//! O `build_smoke.rs` fazia duas coisas: ROTEAR os níveis cujas cenas moram noutros módulos, e
//! HOSPEDAR as cenas que ainda vivem nele. A primeira é uma tabela de despacho que cresce a cada
//! wave (foi a cena `=34`, a lei de mistura, que a levou ao teto); a segunda é código de cena. São
//! assuntos diferentes, e agora são arquivos diferentes.
//!
//! ⚠️ **A ORDEM é load-bearing e não mudou.** Os níveis específicos vêm ANTES dos genéricos: um
//! nível roteado aqui nunca chegava ao `match f` do irmão, e um que não é roteado nunca entrava
//! nestes braços. Devolver `true` significa *"já tratei, pare"*.

/// Roteia `level` para o módulo dono dele. `true` = tratado (o chamador retorna).
pub(crate) fn route(app: &mut crate::App, f: u32, level: u32) -> bool {
    // As cenas do ENVELOPE (níveis 11 e 12) vivem no módulo irmão `envelope_smoke` — teto de
    // LOC. Elas só usam os frames 3 e 4 e nenhum braço compartilhado, então sair do `match`
    // aqui é a MESMA sequência de antes: um nível fora de 11/12 nunca entrava nesses braços, e
    // 11/12 nunca chegavam aos genéricos (os específicos vinham primeiro).
    if matches!(level, 11 | 12 | 27) {
        crate::envelope_smoke::frame(app, f, level);
        return true;
    }
    // A cena da PILHA de efeitos (ADR-0132), no módulo irmão `fx_smoke` — mesma razão de
    // LOC, e mesma disciplina: os níveis 13/14 nunca tocam um braço genérico abaixo. 13 é a
    // pilha animada; 14 é a cena do Apply / Convert (estática).
    // A cena do CONTOUR (pesquisa `20_*` #9), no módulo irmão `contour_smoke` — mesma razão
    // de LOC e mesma disciplina: o nível 25 nunca toca um braço genérico abaixo.
    if level == 25 {
        match f {
            3 => app.smoke_contour_build(),
            4 => app.smoke_contour_arm(),
            _ => {}
        }
        return true;
    }
    // A cena da família WARP (Arc/Bulge/Wave/Fisheye/Rise) — irmão `warp_smoke`, mesma razão
    // de LOC: cinco retângulos armados + um pelado para autorar pela seção Effects.
    if level == 26 {
        crate::warp_smoke::frame(app, f);
        return true;
    }
    // A cena do FALLOFF (o campo escalar que modula a força do deformador seguinte) — irmão
    // `falloff_smoke`, mesma razão de LOC.
    if level == 28 {
        crate::falloff_smoke::frame(app, f);
        return true;
    }
    // A cena do TWIST (o remoinho, e o Falloff a modulá-lo) — irmão `twist_smoke`, mesma razão.
    if level == 29 {
        crate::twist_smoke::frame(app, f);
        return true;
    }
    // A cena do KNOT (o entrelace celta over/under) — irmão `knot_smoke`, mesma razão.
    if level == 30 {
        crate::knot_smoke::frame(app, f);
        return true;
    }
    // As cenas do SKETCH (=31) e do HATCH (=32) — irmão `sketch_hatch_smoke`, mesma razão.
    if level == 31 || level == 32 {
        crate::sketch_hatch_smoke::frame(app, f, level);
        return true;
    }
    // A cena do FX RASTER (=33 — Blur/Glow/Drop Shadow, plano 24) — irmão `fx_raster_smoke`.
    // NÃO confundir com `fx_smoke` (=13/14), que é a pilha de deformadores vetoriais (ADR-0132).
    if level == 33 {
        crate::fx_raster_smoke::frame(app, f);
        return true;
    }
    // A LEI DE MISTURA por degrau (=34, plano 24 W6) — irmã da =33, e separada dela porque é
    // um A/B (o mesmo degrau sob duas leis), não um catálogo.
    if level == 34 {
        crate::fx_blend_smoke::frame(app, f);
        return true;
    }
    // A TURBULÊNCIA (=35, plano 24 W6b) — irmã da =34 e no mesmo molde: quatro pares, um knob de
    // diferença em cada.
    if level == 35 {
        crate::fx_turbulence_smoke::frame(app, f);
        return true;
    }
    // GROW / SHRINK (=36, plano 24 W7) — a mesma família: quatro pares, uma coisa de diferença.
    if level == 36 {
        crate::fx_morphology_smoke::frame(app, f);
        return true;
    }
    // COLOR ADJUST (=37, plano 24 W8) — a mesma família: quatro pares, uma coisa de diferença.
    if level == 37 {
        crate::fx_adjust_smoke::frame(app, f);
        return true;
    }
    // DUOTONE + LUMA TO ALPHA (=38, plano 24 W9) — as duas leis que leem o BRILHO da arte: uma
    // manda a resposta para a COR, a outra para a COBERTURA.
    if level == 38 {
        crate::fx_duotone_smoke::frame(app, f);
        return true;
    }
    // GRADIENT MAP (=39, plano 24 W11) — a rampa de N stops, que SUBSUME o Duotone.
    if level == 39 {
        crate::fx_gradient_map_smoke::frame(app, f);
        return true;
    }
    if matches!(level, 13 | 14) {
        crate::fx_smoke::frame(app, f, level);
        return true;
    }
    // O UNDO da pilha de efeitos, AUTO-DIRIGIDO (o report do Enio, 3×) — irmão `fx_undo_smoke`.
    if level == 20 {
        crate::fx_undo_smoke::frame(app, f);
        return true;
    }
    // A cena da W0 (texto + pilha de efeitos sobrevive ao re-cook) — irmão `text_fx_smoke`.
    if level == 21 {
        crate::text_fx_smoke::frame(app, f);
        return true;
    }
    // A cena da W3 (o texto cavalga o caminho) — irmão `text_path_smoke`.
    if level == 22 {
        crate::text_path_smoke::frame(app, f);
        return true;
    }
    // A cena do GESTO (o artista prende o texto pelo painel) — irmão
    // `text_path_gesture_smoke`. Irmã da 22: aquela mostra o motor, esta o caminho até ele.
    if level == 23 {
        crate::text_path_gesture_smoke::frame(app, f);
        return true;
    }
    // A cena do GESTO do Pattern Along Path (plano 23, W3): motivo + guia selecionados, o
    // artista prende pelo painel; daí o `pattern_live::recook` -> `dispatch` desenha as cópias.
    if level == 24 {
        crate::pattern_path_smoke::frame(app, f);
        return true;
    }
    // A cena do LÁPIS (=40, plano 25 W1) — irmã `pencil_smoke`. Ela dá a REFERÊNCIA na tela e
    // **não arma o modo**: o gesto que este smoke prova começa no chip do painel.
    if level == 40 {
        crate::pencil_smoke::frame(app, f);
        return true;
    }
    // A cena da LARGURA VIVA (=41, ADR-0148) — irmã `profile_smoke`. Como a do lápis, ela dá o
    // MATERIAL e **não arma o perfil**: o gesto que este smoke prova começa nos sliders.
    if level == 41 {
        crate::profile_smoke::frame(app, f);
        return true;
    }
    // A cena do WIDTH TOOL (=42, plano 25 §5) — irmã `width_tool_smoke`. Como as duas anteriores,
    // ela dá o MATERIAL e **não entra no modo**: o gesto começa no pill.
    if level == 42 {
        crate::width_tool_smoke::frame(app, f);
        return true;
    }
    // A cena do ALCANCE DO NÓ (=43, plano 25 §6) — irmã `node_reach_smoke`. Mesma disciplina: dá o
    // MATERIAL (arcos com nó de meio marcado) e **não entra no modo Node**.
    if level == 43 {
        crate::node_reach_smoke::frame(app, f);
        return true;
    }
    // A cena dos CORTES (=44, plano 25 §7) — irmã `cut_smoke`. Mesma disciplina: dá o MATERIAL
    // (um anel, dois pares de fitas e uma seta) e **não arma modo nenhum**.
    if level == 44 {
        crate::cut_smoke::frame(app, f);
        return true;
    }
    // A cena das GUIAS e da RÉGUA (=45, plano 25 §9, a W6.2) — irmã `guide_smoke`. Mesma
    // disciplina: dá o material (duas guias e duas formas) e **não arma modo nenhum**.
    if level == 45 {
        crate::guide_smoke::frame(app, f);
        return true;
    }
    // A cena da SIMETRIA de desenho (=46, plano 25 §9, a W6.3) — irmã `symmetry_smoke`. Mesma
    // disciplina: dá o material (um meio-perfil, uma pétala, um controle) e **não arma modo
    // nenhum** — a simetria nasce desarmada porque esse é o default do produto.
    if level == 46 {
        crate::symmetry_smoke::frame(app, f);
        return true;
    }
    // A cena do ALINHAMENTO do traço (=47) — irmã `align_smoke`. Mesma disciplina: dá o material
    // (o trio de quadrados iguais, a rosquinha e a linha aberta) e **nasce toda em Centre**, que é
    // o default do produto — quem escolhe Inner/Outer é o artista, no painel.
    if level == 47 {
        crate::align_smoke::frame(app, f);
        return true;
    }
    // A cena do AUTO LAYOUT (=50, plano UI/UX W2) — irmã `layout_smoke`. Mesma disciplina: dá o
    // material (duas molduras com seis filhos em desordem) e **nasce SEM fluxo** — quem escolhe
    // Row é o artista, no painel, que é a costura inteira desta wave.
    if level == 50 {
        crate::layout_smoke::frame(app, f);
        return true;
    }
    // A cena da BOOLEANA VIVA (=48) — irmã `bool_smoke`. Mesma disciplina: dá o material (os três
    // rigs) e **nasce com o modo Live em OFF**, que é o default do produto — quem liga é o artista.
    if level == 48 {
        crate::bool_smoke::frame(app, f);
        return true;
    }
    // ⭐ A cena da BOOLEANA VIVA DENTRO DE UM ESTADO (=74) — irmã `ui_states_bool_smoke`. Ao
    // contrário da `=48`, ela nasce **PRONTA**: o que ela prova é a ANIMAÇÃO da troca de operação,
    // e uma animação que exige quinze cliques antes de aparecer não é smokável (*feature nova =
    // auto-play*). O rig 2 é o CONTROLE, com o material idêntico e sem pose nenhuma.
    if level == 74 {
        crate::ui_states_bool_smoke::frame(app, f);
        return true;
    }
    // A cena da MOLDURA (=49) — irmã `frame_smoke`. Duas molduras idênticas, só uma recortando: o
    // par CONTROLE, para a resposta ser visível sem tocar num controle.
    if level == 49 {
        crate::frame_smoke::frame(app, f);
        return true;
    }
    // A cena dos TOKENS (=51) — irmã `token_smoke`. Mesma disciplina: dá o material (o card, o
    // CONTROLE idêntico ao lado e a forma sem traço) e **nada nasce bindado** — quem prende é o
    // artista, e é isso que faz a costura painel→shell→ECS→desenho ser de facto exercitada.
    //
    // ⚠️ **Ela nasceu em `=50` e MUDOU para `=51` em 2026-08-02.** O auto layout tomou o 50, que
    // já era desta cena, e como o router é uma lista de `if` o primeiro vence: a cena dos tokens
    // ficou **inalcançável em silêncio** — `PH2D_BUILD_SMOKE=50` corria o layout e ninguém tinha
    // como saber. Quem moveu foi ela (e não o layout) porque o layout é o número que o artista
    // acabou de usar num smoke aprovado. O gate `no_two_smoke_scenes_claim_the_same_level` é o
    // que impede a próxima colisão de ser silenciosa.
    if level == 51 {
        crate::token_smoke::frame(app, f);
        return true;
    }
    // A cena das ÂNCORAS (=52, plano UI/UX W3) — irmã `anchor_smoke`. Mesma disciplina: dá o
    // material (uma moldura com quatro peças coladas onde um artista as poria) e **nada nasce
    // ancorado** — quem arma a regra é o artista, e é isso que faz a costura painel→shell→ECS→
    // desenho ser de facto exercitada.
    if level == 52 {
        crate::anchor_smoke::frame(app, f);
        return true;
    }
    // A cena dos COMPONENTES (=53, plano UI/UX W5) — irmã `component_smoke`. Mesma disciplina: dá
    // o material (um botão de duas peças e um controle) e **nada nasce sendo componente** — quem
    // promove, coloca e destaca é o artista, e é isso que faz a costura painel→shell→ECS→desenho
    // ser de facto exercitada.
    if level == 53 {
        crate::component_smoke::frame(app, f);
        return true;
    }
    // A cena da ÂNCORA DE ESCALA (=54) — irmã `gizmo_anchor_smoke`. Mesma disciplina: dá o
    // material (a forma, os dois pinos que a medem, a moldura e o controle) e **não arma
    // modificador nenhum** — quem carrega no Ctrl, no meio do arrasto, é o artista.
    if level == 54 {
        crate::gizmo_anchor_smoke::frame(app, f);
        return true;
    }
    // A cena do MESTRE REDIMENSIONADO (=55) — irmã `component_resize_smoke`. Ao contrário da `=53`,
    // ela nasce com o componente JÁ armado e duas cópias vivas: o que este smoke prova não é o
    // gesto de criar, é o que acontece às cópias quando a alça do mestre anda.
    if level == 55 {
        crate::component_resize_smoke::frame(app, f);
        return true;
    }
    // A cena das DIFERENÇAS (=56) — irmã `component_pieces_smoke`. Como a `=53`, ela **não arma
    // componente nenhum**: o que esta prova é a lista de PEÇAS (o override, o Update Main, o
    // Swap), e criar/colocar já é gesto provado — repeti-lo é barato e não esconde costura.
    if level == 56 {
        crate::component_pieces_smoke::frame(app, f);
        return true;
    }
    // A cena da ORDEM DE Z (=57) — irmã `zorder_smoke`. Duas perguntas de olho: o FILHO aparece
    // (a lei do Godot) e os botões de z-order fazem alguma coisa (eles escreviam na porta errada).
    if level == 57 {
        crate::zorder_smoke::frame(app, f);
        return true;
    }
    // A cena dos VARIANTS (=58) — irmã `variant_smoke`. Como a `=53` e a `=56`, ela **não arma
    // componente nenhum**: aqui é o gesto de marcar os quatro irmãos como mestres que FAZ deles um
    // conjunto de versões, então armá-lo escondia a metade mais importante do desenho.
    if level == 58 {
        crate::variant_smoke::frame(app, f);
        return true;
    }
    // A cena dos TOKENS (=59) — irmã `tokens_smoke`. ⚠️ Ela não monta geometria de assunto
    // nenhum: o sujeito do smoke é o PRÓPRIO EDITOR (o painel re-veste a janela ao vivo), e as
    // formas que ela põe são o CONTROLE de que a arte não é chrome.
    if level == 59 {
        crate::tokens_smoke::frame(app, f);
        return true;
    }
    // A cena da PELE POR-WIDGET (=60) — irmã `widget_skin_smoke`. Quatro formas vestidas + uma
    // NUA, que é o controle: marcar uma forma é a única coisa que a transforma.
    if level == 60 {
        crate::widget_skin_smoke::frame(app, f);
        return true;
    }
    // A cena dos ESTADOS de UI (=61) — irmã `ui_states_smoke`. Três hospedeiros: um que muda
    // pose+escala+cor com um FILHO a acompanhar, um que muda SÓ a cor (e cujo custo em `Plan` a
    // cena imprime), e um SEM estado nenhum, que é o controle.
    if level == 61 {
        crate::ui_states_smoke::frame(app, f);
        return true;
    }
    // A cena do PAINEL GERADO (=62) — irmã `ui_panel_smoke`. Uma moldura com quatro filhos
    // VESTIDOS e um que é só desenho (o controle), e o código-fonte que ela descreve impresso.
    if level == 62 {
        crate::ui_panel_smoke::frame(app, f);
        return true;
    }
    // A cena do REFLUXO (=63) — irmã `text_wrap_smoke`. Dois textos com a MESMA frase: um com
    // caixa (várias linhas, selecionado) e um sem (o controle, numa linha só).
    if level == 63 {
        crate::text_wrap_smoke::frame(app, f);
        return true;
    }
    // A cena da HIERARQUIA (=64) — irmã `ui_nested_smoke`. Um menu com dois itens, e os três são
    // hospedeiros: o menu não pode fechar quando o cursor desce para um item dele.
    if level == 64 {
        crate::ui_nested_smoke::frame(app, f);
        return true;
    }
    // A cena da MOLA (=65) — irmã `ui_spring_smoke`. Quatro faixas com a MESMA viagem e motores
    // diferentes: a mola, o `Cubic In-Out` que é o A/B dela, o `Back Out` que mudou de
    // comportamento nesta wave, e o `Cubic Out` de fábrica, que é o controle.
    if level == 65 {
        crate::ui_spring_smoke::frame(app, f);
        return true;
    }
    // A cena do SIZING (=66) — irmã `sizing_smoke`. O vocabulário de tamanho do Figma: o abraço
    // (Hug), o piso (Min) e o fora-do-fluxo (Absolute), com uma quarta moldura de CONTROLE.
    if level == 66 {
        crate::sizing_smoke::frame(app, f);
        return true;
    }
    // A cena da ROLAGEM (=67) — irmã `scroll_smoke`. Uma lista que não cabe e um CONTROLE que
    // cabe, na mesma caixa: a roda rola a primeira e dá zoom sobre a segunda.
    if level == 67 {
        crate::scroll_smoke::frame(app, f);
        return true;
    }
    // ⭐ A cena da TABELA SINAL → PAPEL (=68) — irmã `signal_smoke`. Um botão grita um NOME e
    // OUTRA forma responde; o terceiro retângulo é o CONTROLE que não escuta nada.
    if level == 68 {
        crate::signal_table_smoke::frame(app, f);
        return true;
    }
    // ⭐ A cena da GRADE (=69) — irmã `grid_smoke`. As cores formam COLUNAS numa grade e ficam
    // ESPALHADAS no `RowWrap` que é o controle, com os MESMOS filhos.
    if level == 69 {
        crate::grid_smoke::frame(app, f);
        return true;
    }
    // ⭐ Os NÓS DE VÁRIAS FORMAS (=70) — irmã `multi_node_smoke`. Três formas, a do meio ESCALADA:
    // uma caixa sobre as três apanha os 12 nós, e arrastar um leva todos na mesma distância de tela.
    if level == 70 {
        crate::multi_node_smoke::frame(app, f);
        return true;
    }
    // ⭐ O LAÇO (=71) — irmã `lasso_smoke`. Uma fileira ALTERNADA: as azuis são o alvo e cada
    // âmbar fica entre duas delas, então nenhum retângulo separa o conjunto que o pedido nomeia.
    if level == 71 {
        crate::lasso_smoke::frame(app, f);
        return true;
    }
    // ⭐ O RÓTULO DE DISTÂNCIA (=72) — irmã `snap_label_smoke`. Duas superfícies na MESMA tela
    // (a régua e a ficha), porque a pergunta da wave é se elas dizem o mesmo número.
    if level == 72 {
        crate::snap_label_smoke::frame(app, f);
        return true;
    }
    // ⭐ O X/Y DO NÓ (=73) — irmã `node_xy_smoke`. A cena é uma VIGA LARGA de propósito: com um
    // nó só, o modelo da mediana e o de escrever-o-alvo dão o mesmo resultado, e o defeito que a
    // wave recusa (colapsar a seleção numa coluna) só existe no plural.
    if level == 73 {
        crate::node_xy_smoke::frame(app, f);
        return true;
    }
    // ⭐ A MÁQUINA DE ESTADOS DO MORPH (=74, plano 32 W6) — irmã `morph_states_smoke`. Mesma
    // disciplina: dá o material (três formas bem diferentes e um Morph entre duas delas) e **nada
    // nasce ligado** — nenhuma seta, nenhuma condição. Quem as desenha e escolhe é o artista, e é
    // exactamente essa a costura que a wave existe para provar.
    if level == 74 {
        crate::morph_states_smoke::frame(app, f);
        return true;
    }
    false
}
