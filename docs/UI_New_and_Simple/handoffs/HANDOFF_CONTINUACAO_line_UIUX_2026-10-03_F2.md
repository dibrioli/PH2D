# Continuação — `line/UIUX`, 2026-10-03 — a F2 do spec/06 (os modos), NESTA linha, sem integrar

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd` + `pwd` + `git branch --show-current` ANTES de ler.

## §1 — Estado

- **Base (merge-base = `main`) `1ad60a1ce`**; o `main` não andou. `git cherry main HEAD` = **28 `+`**
  com este ficheiro, nenhum integrado (ordem do dono: não integrar agora). Árvore limpa.
- **A F1 (o menu Add de objectos) está FECHADA e com o smoke do dono APROVADO (03/10, «Smoke OK»).**
  O handoff de integração dela é
  [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md)
  (cita o da escala, de 02/10). Os dois continuam válidos para os commits que descrevem; o fecho
  da F2 escreve um handoff de integração NOVO, sobre o diff ACUMULADO desde a base, que cita os dois.

## §2 — A TAREFA: a F2 do [`spec/06`](../spec/06_tipos_e_modos_de_objeto.md), junta com a F3 da Imagem

Leia o spec/06 INTEIRO (o §2.1 é a F0 medida, o §3.2 o desenho do modo, o §4 as fases) e a
`00_DECISOES_DO_ENIO.md` §D3 e §D6. As escolhas do dono no §6 do spec estão fechadas: não as
re-litigue.

**Porque a F2 não sai sozinha:** um modo que aparece no selector e não faz nada é um controlo morto
(D6). ⇒ o 1.º modo entra JUNTO com o módulo que ele abre sobre a entidade. Pela medição da F0, esse
é **Image ▸ Paint**: o Painter já pinta a imagem seleccionada (`PainterTool::bound_doc`,
`ph2d-tool-painter/src/tool/mod.rs:313`) e recusa sem `Sprite` (`painter_canvas_input.rs:358`).

**O que a F2+F3(Image) entrega:**
- `ObjectMode` (só os modos que algum tipo declara E que já funcionam) e um `ModoActivo
  { entidade, modo }` — ⛔ o tipo NÃO se guarda: pergunta-se ao marcador
  (`ph2d_app_components::component_attach::kind_of`, completo desde a F1).
- O selector *«Object Mode ▾»* como 1.º pulldown do cabeçalho da área (`AreaMenu`,
  `WidgetStore::set_area_commands`, `screens/hero/tool_bar.rs::bar_rail`; o 3D Model já publica ali
  a vista e o sombreamento). As faces = os modos do tipo do activo.
- Entrar em Paint = o Painter sobre ESSA entidade; sair = Object. ⚠️ Antes de desenhar, MEÇA como
  o Painter é aberto hoje (o toggle de módulo / a ferramenta) — o pill antigo sai na MESMA fase
  (spec/06 §5: dois caminhos para o mesmo módulo divergem).
- O cadeado (escolha 4): num modo de criação, clicar noutro objecto NÃO troca.
- O botão direito do canvas em modo Object abre o menu Add (escolha 3; ficou da F1 — hoje ele tem
  donos: o vetor cancela a forma, o Motion e o Painter abrem menus). O pedido é o mesmo do `+`:
  `HierRequest::AddRoot`.
- A família declara os modos como declarou as entradas (`ENTRIES` → a composição junta as compiladas
  em `render_loop/fase_object_add.rs::FAMILIES`).

## §3 — Medido nesta janela (e o que NÃO está medido)

- ⛔ **O `Tab` JÁ TEM DONO:** alterna o modo zen (`shells/desktop/src/input_handlers.rs:121`), e no
  modo Node do vetor anda pelos nós (`keyboard_cadeia.rs:114`). O Tab do Blender (Object ↔ último
  modo) **colide** — é decisão de PRODUTO: pergunte ao dono antes de o ligar, com a recomendação.
- ⚠️ **69 sítios** chamam `replace_selection(` (shell + `ph2d-app-*` + editor-core). O cadeado não
  se põe sítio a sítio: precisa de UMA porta. Precedente a ler primeiro:
  `render_loop/fase_hierarchy_select_lock.rs` (já existe um cadeado de selecção na shell).
- ⚠️ Limites herdados para a F3 dos outros: o **Model** é um por cena (a ponte coze só a 1.ª raiz
  `FieldObject`, `ph2d-app-field3d/src/scene.rs:296`); o **Flip** escreve sempre no 1.º desenho
  (`ph2d-app-flip/src/autokey.rs:54`). Ordem medida da F3: Image → Sculpt → Flip → Model → Vector.
- NÃO medido: como o Painter é ligado hoje, e quem escreve o estado «módulo aberto» de cada pill.

## §4 — Restrições (medidas)

- `screens/hero.rs` **698/700**, `hero/paint.rs` **700/700**, `hero/left_rail.rs` 692 ⇒ o
  `ModoActivo` vive num ficheiro IRMÃO ou num sub-estado, nunca solto no `HeroScreen`.
- A shell cresceu **+255** líquido na linha (catraca `the_shell_only_shrinks` verde, com folga):
  código de família vai para `crates/ph2d-app-<família>`; a shell só compõe.
- Fotos: `docs/Components/ferramentas/fotografa_cena.sh` (o XTest não chega: o clique prova-se num
  seam `ph2d-ui-testkit` / `HeroScreen::apply_event` com `register_all_panels`). Para abrir um
  estado na foto, um gatilho de smoke de uma vez (precedente: `PH2D_OBJECT_ADD_SMOKE=1`).
- Explorador com perguntas ESTREITAS (≤ 30 passos, uma por módulo): os largos estouram, e um deles
  deu nesta janela uma resposta errada (o Model «por entidade») que só a leitura do código desfez.
- ⚠️ O guarda de recursos lê o TEXTO do comando: um heredoc que contenha `cargo run` é recusado.
  Escreva docs pela ferramenta `Edit`/`Write`.

## §5 — Como a onda fecha

DIRETRIZ §1.5.9: gate batched 1× sobre o diff acumulado desde `1ad60a1ce`; mutação dos gates novos
(controlo VERDE antes; nesta janela uma mutação sobreviveu porque o gate lia o esperado da própria
função testada — escreva o lado independente); handoff de integração NOVO que cita os dois
anteriores; `rm -rf target/*/incremental`; e por último o build de smoke 2× com a 2.ª saída
colada. ⛔ Não integrar nem pushar.
