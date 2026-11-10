# PROMPT DE MISSÃO — linha `line/InputMap` (cole inteiro numa janela nova)

> Escrito em 10/11 pela janela que abriu a linha e estudou a crítica com o dono. Leitor: a LLM que vai
> cumprir a missão SOZINHA, do início ao fim, sem parar. Tudo o que a conversa de abertura apurou está aqui.

═══════════════════════════════════════════════════════════════════
## 0 · Quem você é e as regras desta missão

Você é o agente da linha **`line/InputMap`**, Modo L, worktree
`/home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap`. O dono (Enio) **não está ao teclado**: a
missão corre do início ao fim **sem parar e sem perguntar**. Decisões de PRODUTO já estão tomadas (§2).
Decisões TÉCNICAS e de acabamento são suas, pelo estado da arte
([memória](../../../project-memory/feedback_finish_choices_go_to_the_state_of_the_art.md)). Só pare nos
dois casos irredutíveis do CLAUDE.md §0.2: **contrato congelado** (§6) ou **rebase a conflitar fora dos
seus arquivos**. Nesses casos escreva o porquê no handoff e pare. Fora deles, nunca pare.

**Nunca pare num pedido de permissão.** Os comandos têm de ter as formas que não disparam pedidos, e o
briefing de CADA subagente leva esta lista, copiada:
- ⛔ `cd X && …` (use caminhos absolutos ou `git -C`), `bash -c` / `sh -c`, `rm -rf` com variável ou glob,
  `kill` de PID que não é seu, escrever em `.git/`, `.claude/` ou no rc da shell.
- ⛔ `sed -i`/`perl -i` sem `grep -c` na mesma linha; `replace()`/`re.sub()` que grava sem `assert` de contagem.
  Edite pela ferramenta `Edit`.
- ⛔ Pesado (cargo test/build/run/nextest/clippy) cru. Vai sempre por `bash scripts/ph2d-run.sh <cmd>`.
- ⛔ Laço de espera sobre `pgrep -f`, vigia de fundo sem `timeout`. O arnês avisa quando um comando de fundo acaba.
- ⛔ `git add -A`/`.`, `git stash` nu, `push`, `--force`, integrar, `ship.sh`, `suite-completa.sh`,
  `nextest-impacted.sh` (são do integrador, por ordem do dono).

═══════════════════════════════════════════════════════════════════
## 1 · Abertura (faça já, nesta ordem)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap && pwd && git branch --show-current`
   (TEM de imprimir `…/Worktrees/line-InputMap` e `line/InputMap`). É o único `cd` permitido. A partir
   daqui, caminhos absolutos.
2. `git cherry main HEAD | grep -c '^+'` e, se o main andou, `git rebase main 2>&1 | tee target/rebase.log`.
   Se o rebase conflitar em `Cargo.lock` ou num arquivo gerado, regenere em vez de editar à mão (DIRETRIZ §1.5.5).
3. Leia: `CLAUDE.md` (já vem carregado), `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` (RELEIA a
   cada passo), DIRETRIZ §1.5.9 e §3.B, `docs/CartoesUI/README.md` + `11_RECEITA_PAINEL_EM_CARTOES.md`,
   `docs/IntegracaoMultiAgente/STACK_VERSOES.md`, `project-memory/MEMORY.md` e
   `project-memory/project_inputmap_three_versions_2026_11_10.md`.
4. **Abra as maquetes como IMAGEM** (Read em cada png) — `docs/InputMap/maquetes/01..05_*.png` — e o texto
   `docs/InputMap/maquetes/critica_inputmap_texto.txt`. A fonte original é
   `/home/enio/Documentos/GameEngineSamples/critica_ph2d.html#inputmap`. Para refazer as fotos, use
   `bash docs/InputMap/ferramentas/fotografa_maquetes.sh <pasta>`.

═══════════════════════════════════════════════════════════════════
## 2 · A missão (ordem do dono, 10/11) e as decisões dele

**Ordem verbatim:** *«Implementaremos as 3 versões sugeridas mantendo a boa aparência das 3 e com um seletor
de tipo de input map no topo do quadro central.»*

**Respostas do dono, dadas antes de começar. Não as pergunte de novo:**
1. **Tela de controles do jogador (C·4) FUNCIONA no jogo**, no Play e no jogo exportado. O jogador troca a
   tecla e a troca fica guardada **fora do projeto**, no computador dele.
2. **Projeto novo continua nascendo com as 8 ações de fábrica.** As receitas ficam num «Start from…» sempre
   disponível. Ele acrescenta ou troca as ações; a troca tem confirmação e entra no undo.
3. **A aba abre na ÚLTIMA versão usada** (na 1.ª vez, a B). É preferência do usuário, não do projeto.
4. **O Luau ganha leitura pela AÇÃO** (`"jump"`), e o jeito pela tecla continua funcionando.

**E a consequência técnica que foi dita ao dono:** hoje o gamepad (gilrs, no shell) só alimenta o Luau. O
mapa guarda `Binding::PadButton`/`PadAxis`, mas ninguém os captura nem os resolve no Play. Esta linha
**liga o gamepad ao mapa nas duas pontas**: a captura no «escutando» e a resolução no Play. Sem isso, a frase
«Press a key or a gamepad button» é mentira.

═══════════════════════════════════════════════════════════════════
## 3 · O alvo visual (o que é «a boa aparência das 3»)

Uma **aba no quadro central** (o molde é o editor de animações do Animated Sprite), com um **seletor
segmentado no topo**: `List · Keyboard · Try it`. São rótulos em inglês; você pode afiná-los, mas mantenha três.

- **A · List** (maquetes 01–02): campo «New action name» + Add. Uma busca «Find an action, or press a key»
  em que apertar uma tecla filtra as linhas que a usam e mostra no pé «Up Arrow is in 2 of 11 actions…
  read by…». Cada ação é uma linha com **chips de tecla com o nome inteiro** («Left Arrow»), sem «...» nem
  «K...». O tipo vira ícone (teclado/pad). **Dead zone / Press at só nas linhas que têm manche ou gatilho**,
  com o rótulo inteiro e a escala («Dead zone (0 to 1)»). A escuta acontece no próprio lugar onde a tecla
  cai («Press a key or a gamepad button. Esc cancels.»). O pé explica a tecla repetida de fábrica.
- **B · Keyboard** (maquetes 02–03), a recomendada: à esquerda, a lista de ações **agrupada por quem as lê**
  (PLATFORM PLAYER · TOP-DOWN PLAYER · YOURS), cada uma com uma cor e a contagem de teclas, e `+ Add` e
  `Remove` embaixo, como em Animations. Em cima, um sub-seletor `Keyboard | Gamepad` e o **teclado
  desenhado**: cada tecla leva **uma barra por ação** na cor da ação, e as teclas da ação aberta ficam
  acesas. Clicar numa tecla acrescenta-a à ação aberta. O hover mostra no pé «Up Arrow: jump (Platform
  Player) · move_up (Top-Down Player)». O **comando desenhado** (LT RT LS RS A B X Y, cruz) tem a mesma lei
  das barras. Embaixo fica a ação aberta: título «jump · 2 keys», «Read by …» à direita, as teclas em
  **ladrilhos** (como os quadros) e um ladrilho tracejado «Press a key» que escuta. Com manche, aparece a
  **barra do curso**: 0 · Dead · Press · 1, com a zona morta hachurada, e os dois campos. A ação sem leitor
  mostra o aviso âmbar «Nobody reads reload yet. A Trigger can turn it into a signal: type reload in its Action.».
- **C · Try it** (maquete 04): **receitas** em cartões (Platformer · Top-down · Shooter · Blank) atrás do
  «Start from…». O **teste ao vivo** «Try it ● LIVE / The game is not running»: com o jogo parado, apertar
  acende as teclas no teclado desenhado e enche as barras de força de cada ação (0.00–1.00), e o pé diz
  «You are pressing Up Arrow: jump and move_up are on…». A **manche em círculo**: disco interno (Dead), anel
  tracejado (Press), o ponto ao vivo, «Pushed 0.62 · Strength 0.55 · Pressed». A conta é a do `resolve.rs`:
  (p − dead) ÷ (1 − dead). A **tela de controles do jogador**: a prévia, a caixa «Players can change keys» e
  «Shown in the game» com uma caixa por ação.

**Leis de acabamento (o dono já cobrou «faltou capricho»):**
- Use o estilo dos **widgets dos cartões**, nunca um pintor próprio para botão ou campo. Zero hex, zero `f32`
  literal de UI, zero string solta: tudo vai por tokens e `ph2d-i18n`, **só em inglês**, com tooltips em Plain
  English (HR-15, CLAUDE.md §0.3).
- As cores das ações vêm de uma paleta categórica de tokens. Se não existir, crie-a como tokens novos, com a
  regra de cor estável por ação (por `ActionId`, não pela posição).
- Zero texto cortado e zero número que não muda nada. Os números da tabela «O ideal» da crítica são a régua:
  22 → 0 números mortos, 38 → 0 cortes. Prove-os num teste.
- **Antes de dar por pronto, ponha foto e maquete LADO A LADO** (montage) e confira margem, peso, proporção,
  canto, cor e o que NÃO está na maquete
  ([memória](../../../project-memory/feedback_a_mockup_is_compared_side_by_side_before_the_smoke.md)).

═══════════════════════════════════════════════════════════════════
## 4 · O mapa do código (levantado em 10/11 — confira antes de confiar)

- **A janela de hoje é FLUTUANTE, dentro do `HeroScreen` (`ph2d-editor-core`):** a abertura fica em
  `ids/menus_settings.rs:39-46` (`CTX_MENU_SETTINGS_INPUT_MAP`), com a linha de menu em
  `screens/hero/menu_tables_settings.rs:32`. Os demais arquivos:

  | O quê | Onde |
  |---|---|
  | Estado | `interaction/state/input_map_ops.rs`: `open_input_map:21`, `listen_for_binding:80`, `capture_bound_key:123`, `capture_if_listening:167` |
  | Pintura | `screens/hero/chrome/input_map.rs` (589 LOC), chamada em `screens/hero/paint.rs:627` (`INPUT_MAP_SURFACE`) |
  | Lógica da janela | `chrome/input_map/{apply,layout,pedidos}.rs` |
  | Ids | `ids/chrome/input_map.rs` |
  | Captura de tecla | `interaction/dispatch/key.rs:58-60` |
  | Arrasto | `shells/desktop/src/input_map_drag.rs` |
  | Textos | `ph2d-i18n/src/input_map.rs` |
  | Testes | `ph2d-editor-core/tests/it/the_input_map_*.rs`, `number_input_mapped_link.rs`, `shells/desktop/tests/it/the_input_map_window_can_be_moved.rs`, `shells/desktop/src/project_input_map_tests.rs` |

  ⇒ A janela **sai**: «Settings › Input Map…» passa a abrir a aba. Os testes dela migram para a aba ou
  morrem com ela. Nada fica órfão (o censo de ids e as gates de wiring olham).
- **`ph2d-input`** (puro, sem backend, de propósito):
  - `action.rs`: `Binding{Key, PadButton, PadAxis{axis,positive}}:22`, `ActionId(u32):47`,
    `InputAction{name,bindings,dead_zone,press_point}:51`.
  - `map.rs`: `InputMap:42`, `create:111`, `with_player_defaults:69`, os 8 nomes em `:12-33`.
  - `resolve.rs`: `Sample`, `ActionState`, `tick`, `Input{pressed,just_pressed,strength,axis}:128`.
  - `gamepad.rs`: `GamepadButton`, `GamepadAxis`, `GamepadState`.
  - **Confira** se o `resolve.rs` já consulta o `GamepadState` (não foi lido).
- **Gamepad no app:** gilrs entra em `shells/desktop/src/app_state.rs:363` e `app_state_devices.rs:16`
  (`init_gamepads`); ache o adaptador. Hoje chega só ao Luau: `ph2d-app-components/src/script_input.rs:30-35`
  e `ph2d-script/src/host.rs:132-136` (`provide_input`).
- **Quem lê nomes de ação:** o Platform Player e o Top-Down Player em `ph2d-physics-ecs/src/player_input.rs:73-81`;
  o Trigger em `ph2d-app-components/src/trigger_bridge.rs:29-37,65` (`NoMapa`); o Luau lê por tecla. Os grupos
  «quem lê» e o aviso «Nobody reads» derivam disto: componentes da cena + Triggers + scripts que chamam a ação
  nova. É uma tabela de leitores, sem estado novo no motor.
- **O molde da aba central:**
  - O trait `DocumentSurface` (`ph2d-editor-core/src/doc_surface.rs:183`: `tabs/paint/pointer/key/compose/close/commands`),
    com a aba em `DocTab:20`.
  - A instalação: `ph2d-app-sprite-anim/src/editor/mod.rs:310-314` (`install`), chamada em `shells/desktop/src/init.rs:385`.
  - O `frame` em `editor/mod.rs:319`, chamado em `render_loop/fase_snapshots_publish.rs:9`.
  - A abertura: `request_open:38`, chamada em `render_loop/inspector_commits.rs:344`.
  - A pintura vive numa crate própria, `ph2d-anim-editor/` (`paint*.rs`, `layout.rs`, `strip_bar.rs`).
  - Copie esta forma: **uma crate de pintura `ph2d-input-map-editor` + uma família `ph2d-app-input-map`**,
    instalada pelo shell em ~3 linhas. A shell só encolhe (`the_shell_only_shrinks`).
- **Seletor segmentado:** não há widget genérico. O mais próximo é `PanelTab` (`floating_panel.rs:38-46`). Se
  faltar, crie o widget segmentado na galeria de widgets (`ph2d-widget-sync`, showcase, a11y) e use-o.
- **Projeto:** `shells/desktop/src/project.rs:152` (`input_map`, fora do undo global; os comentários «v96»
  envelheceram). `PROJECT_SCHEMA = 231` em `project_schema.rs:419`. Campo novo ⇒ +1, contado nos três sítios
  (`python3 scripts/schema-recount.py`), **sem migração nem leitor velho** (§0.11).
- **Smoke:** `PH2D_GAME_SMOKE` em `ph2d-app-game/src/smoke.rs:87`, cenas 1–5. **CONTE** a próxima cena livre no
  roteador (não use 6 de memória; outra linha pode ter tomado). A foto do app é
  `bash docs/Components/ferramentas/fotografa_cena.sh <VAR=valor> <larg> <alt> <saida.png> [espera]`
  (kwin virtual; ⛔ nunca `spectacle`).

═══════════════════════════════════════════════════════════════════
## 5 · Desenho técnico (decidido; afine com medição, não com gosto)

1. **Aba central** `Input Map` via `DocumentSurface`. Um só modelo (o `InputMap` do projeto) e três vistas, sem
   estado paralelo. Todo gesto que muda o mapa vira um pedido que o shell aplica com **undo**. Hoje o mapa
   está fora do undo global: decida e escreva o porquê. Receita, remover e trocar ações têm de ser desfazíveis.
2. **Seletor** `List · Keyboard · Try it` no topo. A última escolha fica nas preferências do usuário (ache o
   armazém de preferências existente; não invente outro).
3. **Gamepad, as duas pontas:**
   - (a) O adaptador gilrs publica o `GamepadState` também para o resolvedor do mapa no Play.
   - (b) No «escutando», o próximo botão ou eixo que passe do Press vira `Binding::PadButton`/`PadAxis`.
   - (c) O teste ao vivo (C) lê o mesmo estado com o jogo parado.
   - Sem comando ligado, a frase da escuta diz só o que é verdade: o texto é condicional.
4. **Dead/Press** continuam por ação (como a Godot). Só aparecem quando a ação tem manche ou gatilho.
5. **Leitores:** uma função pura `readers(scene, scripts) -> BTreeMap<ActionName, Vec<Reader>>`, que alimenta os
   grupos de B, o «Read by», o aviso «Nobody reads» e a frase da tecla repetida. A repetição de propósito
   (jump × move_up) é explicada; a repetição por engano é avisada.
6. **Receitas** (Platformer · Top-down · Shooter · Blank) são tabelas de dados em `ph2d-input`, com testes de que
   os nomes batem com os que o Platform Player e o Top-Down Player leem.
7. **Tela de controles do jogador:**
   - No projeto (schema +1): `players_can_change_keys: bool` e `shown_in_game: Vec<ActionId>`.
   - No Play e no jogo exportado: um overlay «Controls» (tecla de abrir configurável, por omissão Esc ou F1;
     meça o que não colide com o Play) em que o jogador clica numa ação e aperta a tecla nova.
   - A troca do jogador é gravada **fora do projeto**, num arquivo por jogo no diretório de config do usuário,
     chaveado pela identidade estável do projeto, e aplicada por cima do mapa ao carregar.
   - Ache como o jogo exportado roda hoje antes de desenhar; se a exportação não tiver runtime de UI, a tela
     entra no runtime que o Play usa e a exportação herda-o.
8. **Luau:** `ph2d.input.action("jump")` devolve `pressed`/`just_pressed`/`strength`, ou três funções; siga o
   idioma da API que já existe. Também: as definições do `luau-lsp`, a doc do script e um teste. O jeito pela
   tecla fica.
9. Fios mortos que a crítica achou:
   - Corrija o comentário velho de `map.rs` («as setas, e não o WASD»).
   - Confira se o W ainda abre um painel do editor; se abrir, isso briga com o jogo.

═══════════════════════════════════════════════════════════════════
## 6 · Ondas (uma por vez; cada uma fecha com prova)

| Onda | Entrega | Prova (teste por NOME + foto) |
|---|---|---|
| **W1** | Crates novas + aba central + seletor + «Settings › Input Map…» abre a aba + a janela flutuante sai (código, ids, testes, i18n) | seam test: menu → aba aberta; seletor troca e lembra; censo de ids verde |
| **W2** | **A · List** inteira (chips, busca por tecla, escuta no lugar, Dead/Press só com manche, pé) | seam tests de cada controle; teste «0 números mortos, 0 cortes» sobre o mapa de fábrica + o de 11 ações da foto do dono |
| **W3** | **B · Keyboard** (lista agrupada por leitor, teclado com barras, hover, ladrilhos, Add/Remove, aviso «Nobody reads»), com a função `readers` | testes de `readers`; clique na tecla acrescenta; cor estável por ação |
| **W4** | **Gamepad nas duas pontas** + o comando desenhado + a barra do curso | resolução Pad no Play (teste com `GamepadState` sintético); captura de botão e de eixo |
| **W5** | **C · Try it**: receitas, teste ao vivo com o jogo parado, manche em círculo | a conta do círculo = `resolve.rs`, ao bit; ao vivo não muta o projeto |
| **W6** | Tela de controles do jogador no Play e no exportado + gravação fora do projeto + schema +1 | round-trip do arquivo do jogador; o overlay troca a tecla e o Play obedece |
| **W7** | Luau pela ação + doc + `luau-lsp` | teste de script lendo `jump` com tecla trocada pelo jogador |

Para cada versão (A, B, C) e para cada estado das maquetes, crie **cenas de smoke** no roteador
`PH2D_GAME_SMOKE` (números CONTADOS), com o mapa de 11 ações da foto do dono
(fábrica + fire, reload sem tecla, laser). São elas que se fotografam.

═══════════════════════════════════════════════════════════════════
## 7 · Como trabalhar sem encher a janela (obrigatório)

- **A janela principal só decide, desenha e confere.** Ler muitos arquivos vai ao `explorador`. Compilar,
  testar e portões vão ao `verificador` (devolve só as falhas). Provas de mutação vão ao `mutacao`. Handoff e
  memória vão ao `documentador`. Implementação grande e disjunta vai a `general-purpose` com
  `isolation` desligada, na MESMA worktree, cada um com arquivos DISJUNTOS escritos no briefing.
- **Paralelo sempre que for independente:** várias chamadas Agent na MESMA mensagem (ex.: W2-pintura ‖
  W2-testes ‖ i18n/tokens; W3 ‖ W4 em crates diferentes). Dois agentes nunca no mesmo arquivo.
- **O briefing de cada agente** é autocontido e leva:
  - o caminho absoluto da worktree;
  - os arquivos que pode tocar;
  - o que entregar;
  - a lista de formas proibidas do §0;
  - «inner loop = `bash scripts/cargo-check-narrow.sh <crate>`; teste por nome = `bash scripts/cargo-test-narrow.sh <crate> <nome>`»;
  - «45 min IMPOSTOS»;
  - «devolva ≤ 40 linhas: o que fez, commits, o que ficou aberto».
- **Commits pequenos e frequentes**: `git -C <wt> commit --no-verify -m "…" -- <paths>`, nunca `-A`. Arquivo
  NOVO: `git add -- <path>` antes (um commit com paths não pega arquivo por rastrear).
- Passou de ~300 mil tokens por passo: delegue cada rodada inteira. Não resuma à força a meio de uma onda.

═══════════════════════════════════════════════════════════════════
## 8 · Controle de qualidade e de aparência (em cada onda)

1. **Costura:** todo controle interativo tem as 7 pontas fiadas JUNTAS (DIRETIVA §2) e um seam test
   `ph2d-ui-testkit` que dirige o evento real e afirma o efeito. Painel interativo sem seam test é barrado pela gate.
2. **Foto:** cada cena de smoke é fotografada com `fotografa_cena.sh` (a 1600×1000 e a uma janela pequena,
   1280×720, para provar que nada corta).
3. **Lado a lado:** um agente revisor INDEPENDENTE (não o que pintou) recebe a foto + a tira da maquete
   (`docs/InputMap/maquetes/`), faz `montage` e devolve a lista de diferenças: margem, peso, proporção,
   canto, cor, texto cortado, idioma e o que sobra. Corrija tudo e repita até a lista vir vazia. Guarde as
   montagens finais em `docs/InputMap/fotos/` (versionadas).
4. **Determinismo e verdade:** a conta do círculo e da barra do curso batem com o `resolve.rs` ao bit; o
   «ao vivo» nunca escreve no projeto; a troca do jogador nunca escreve no `.ph2dproj`.

═══════════════════════════════════════════════════════════════════
## 9 · Fecho da linha (depois da W7) — `/pd-linha-fechar`

1. `cargo fmt` · `PH2D_SUITE=fecho bash scripts/ph2d-run.sh cargo clippy --workspace --all-targets -- -D warnings`
   · `cargo machete` · `bash scripts/check-standalone-optional.sh` · `bash scripts/check-workflow-packages.sh`.
   Por último, os testes do item e os portões tocados **por nome**. Crates novas: a crate inteira com
   `PH2D_SUITE="motivo: crate NOVA"`. Registo de painel e schema: os portões deles por nome.
   `#[cfg(target_os` tocado ⇒ `cargo check --target aarch64-apple-darwin`.
2. **Provas de mutação** (`mutacao`) em quatro leis:
   - Dead/Press só com manche;
   - o aviso de tecla repetida e de «Nobody reads»;
   - a resolução Pad no Play;
   - o mapa do jogador por cima do projeto.

   Cada mutação tem de sangrar SÓ o gate certo.
3. **Auditoria final por ≥ 2 lentes independentes**, em agentes paralelos que NÃO escreveram o código:
   correção e wiring · aparência (lado a lado) · determinismo e persistência. Cada claim usa o TEMPLATE da
   DIRETIVA §3 (LENTE, CLAIM, TRAÇO com file:line, ASSERÇÃO-VERMELHA, NÃO-CHECADO, LOC LIDAS). Achado
   confirmado se corrige na hora, e roda de novo só o que falhou.
4. `git rebase main` (com log) e `bash scripts/censos-da-arvore-combinada.sh` (o que só a soma vê: HR-15 e
   tectos de LOC). Cure o que for seu.
5. **Handoff de integração** (`documentador`, ≤ 15 KB) em
   `docs/InputMap/handoffs/HANDOFF_INTEGRACAO_line_InputMap_<data>.md`, com os itens 1–11 de DIRETRIZ §1.5.9:
   - a saída colada de `bash scripts/collision-surface.sh`;
   - os ids, tokens e cenas novos com valores, e o degrau do schema;
   - os arquivos foundational tocados e porquê (editor-core: a janela que saiu, o widget segmentado);
   - o delta de LOC da shell e o que foi apagado de partilhado;
   - «impactados e suíte completa ficam para a integração»;
   - a saída de `bash scripts/agent-loop-profile.sh`;
   - a frase para o CLAUDE.md §5.1, ≤ 300 caracteres (módulo novo «Input Map»).
6. `rm -rf /home/enio/Documentos/Projetos/PH2D/Worktrees/line-InputMap/target/*/incremental`. Depois,
   **binário do smoke**:
   `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, rodado 2×; a 2.ª saída
   («Finished», zero «Compiling») vai no handoff.
7. Varra a máquina: `pgrep -af 'ph2d|cargo|rustc'` e `fuser -v /dev/dri/*`. Não pode sobrar nada seu.
8. **Relatório ao dono** (CLAUDE.md §0.8): curto, sem jargão, em PT-BR, dizendo o que ele consegue FAZER
   agora. **Smoke em passos numerados:**
   - (1) o comando completo com o `cd` da worktree;
   - (2) onde clicar, com o nome da tela;
   - (3) o que tem de acontecer;
   - (4) como saber que deu errado.

   Um bloco por versão (List, Keyboard, Try it), um para o comando de videogame e um para a tela de
   controles no Play. As fotos e as montagens vão como **links markdown clicáveis relativos à raiz do repo
   principal** (`[foto](Worktrees/line-InputMap/docs/InputMap/fotos/…png)`). Depois, **PARE** e aguarde
   ordem de integração.
