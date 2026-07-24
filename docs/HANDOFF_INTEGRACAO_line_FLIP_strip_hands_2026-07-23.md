# Handoff de INTEGRAÇÃO — `line/FLIP` → `main` (a tira ganha MÃOS, 2026-07-23)

> **Para o agente INTEGRADOR.** A linha fechou a wave *"a tira de frames ganha autoria
> direta"*. O implementador parou aqui (CLAUDE.md §0.7).
>
> ⚠️ **PENDENTE DE SMOKE.** Todos os gates estão verdes e a auditoria de 2 lentes rodou (ela
> achou 2 defeitos, os dois corrigidos — §5), mas o veredito do Enio sobre a APARÊNCIA dos
> três gestos ainda não veio. Cena pronta: `PH2D_FLIP_STRIP_SMOKE=1` (§7).

## 1. Identidade

| | |
|---|---|
| branch | `line/FLIP` |
| HEAD | `5b9d51517` |
| base do fork (merge-base) | `df91ef6ec` |
| commits à frente do `main` | **7** |
| `main` andou desde o fork? | **não** (`git rev-list --count HEAD..main` = 0) ⇒ **fast-forward limpo** |

```bash
cd /home/enio/Documentos/Projetos/PH2D     # a árvore PRIMÁRIA
git status --short                          # limpa
git merge --ff-only line/FLIP
```

Se o `--ff-only` recusar, **PARE**: o `main` andou depois desta escrita (DIRETRIZ §1.5.5 —
resolva pelos **ESTÁGIOS do índice**, nunca pelos marcadores, e rode `cargo check --workspace`
depois).

## 2. O que este delta entrega

Os "follow-ups conscientes" que o `docs/Flip/05 §6` declarou em 2026-07-12 — e que esperavam
**a infra de dispatch 2D do painel**. Ela agora existe.

| gesto | antes | agora |
|---|---|---|
| mover a chave no tempo | botões `◀`/`▶`, um quadro por clique | **arrastar a célula** |
| mudar a exposição (hold) | caixa numérica na barra | **arrastar a borda direita da célula** |
| referência fixa (light table, T3.9) | não existia | **Pin** na barra: o quadro vira fantasma além dos vizinhos |

**Nenhuma operação de documento nova**: os dois arrastos caem em `FlipObject::move_frame` e
`set_exposure`, exatamente as que os botões já chamavam. O arrasto é uma segunda forma de
**pedir**, não um segundo caminho para fazer.

## 3. ⚠️ O que o integrador precisa saber ANTES de mesclar

### 3.1 Foundational tocado (`ph2d-editor-core`), todo ADITIVO

| o quê | onde | forma |
|---|---|---|
| **`interaction/flip_strip.rs`** — arquivo NOVO: `FlipStripHitKind` · `FlipStripGesture` · `FlipStripChannel` + os métodos do store | `crates/ph2d-editor-core/src/interaction/` | módulo irmão; **tudo num arquivo** de propósito (§1.5.2.1) |
| `InteractiveState::FlipStripSurface { parent, kind }` | `interaction/state/mod.rs` | **variant apendado** ao enum |
| `WidgetStore.flip_strip: FlipStripChannel` | `interaction/state/mod.rs` + `store_core.rs` | **UM** campo (o irmão da timeline espalhou cinco) |
| 3 hooks de captura | `dispatch/pointer_{down,move,up}.rs` | append ao lado dos hooks da timeline |
| `FLIP_KEY_PIN` · `flip_hold_edge_id(index)` | `ids/chrome/flip.rs` | append |

**A superfície pública nova da `ph2d-flip`:** `ghosts()` ganhou um 5º parâmetro
(`pinned: &[Frame]`). **Um único chamador no workspace** (`flip_pass_ghosts::collect`), já
atualizado; as fixtures da própria crate passam `&[]`.

### 3.2 Símbolos que podem COLIDIR com outra linha

| símbolo | valor | onde |
|---|---|---|
| `FLIP_KEY_PIN` | `hash_node_id("flip.strip.key_pin")` | `ids/chrome/flip.rs` |
| `flip_hold_edge_id(i)` | `flip.strip.holdedge.{i}` (família runtime, como `flip_cell_id`) | idem |
| `InteractiveState::FlipStripSurface` | variant novo, **apendado** | `interaction/state/mod.rs` |
| `BUTTONS` do `panel-flip-frames` | **18 → 19** | `event.rs` |
| `FlipCell.pinned` · `FlipStripSnapshot.current_pinned` | campos novos (tipo do painel, **não serializado**) | `panel-flip-frames/state.rs` |

**Nenhum schema bumpou** — `PROJECT_SCHEMA`, `FLIP_SCHEMA_VERSION`, `DOC_VERSION` e
`VEC_SCENE_SCHEMA_VERSION` **intactos**. Foi decisão, não sorte: ver §4.

### 3.3 Contratos congelados encostados: **NENHUM**

`PanelEvent` (4 variants) ficou intocado — e é a espinha do desenho. Um arrasto 2D tem
começo, percurso e fim; forçá-lo num `SetValue` custaria um variant num contrato congelado
para expressar mal o que a família de gesto (`GraphGesture`, `TimelineGesture`) já expressa
duas vezes. `Tool`, `RasterEditTool`, `CanvasPaintTool`, `NodeOp`/`OpResolver`/`NodeManifest`:
não encostados (conferido por grep).

### 3.4 Arquivos COMPARTILHADOS tocados (onde um merge futuro morde)

| arquivo | mudança | risco |
|---|---|---|
| `interaction/state/mod.rs` · `store_core.rs` | 1 variant + 1 campo + 1 init | append em 3 pontos |
| `dispatch/pointer_{down,move,up}.rs` | 1 bloco cada, ao lado do bloco da timeline | append |
| `shells/desktop/src/main.rs` | 4 `mod` novos | append |
| `shells/desktop/src/render_loop/mod.rs` | 1 chamada de drain + 1 de smoke | 2 linhas, **e a ORDEM da 1ª é gateada** (§4) |
| `render_loop/{flip_pass,flip_pass_ghosts,present,flip_bridge}.rs` | `GhostSources` atravessa a cadeia | assinatura interna, 1 call site |

### 3.5 Splits por LOC (HR-18) — o que mudou de casa

`flip_strip.rs` (604) e `flip_strip_tests.rs` (653) estouraram. Três arquivos novos, o corte
**por responsabilidade**: `flip_strip_resolve.rs` (os 4 resolvedores — *sobre que chave
estamos falando*) · `flip_strip_pins.rs` (o light table) · `flip_strip_pin_tests.rs`.
⚠️ **`current_tween_interval` é re-exportado** de `flip_strip` (tem consumidor fora:
`flip_tween_correct.rs`) — caller paths intactos, o padrão do `inspector_model_physics`.
E `paint_cells::paint` bateu 201/200 ⇒ a célula virou `paint_cell`.

## 4. As decisões que não são detalhe

1. **O documento muda UMA vez, no fim do arrasto.** Um gesto = um passo de undo (a fila
   global é por diff sobre o `ProjectState`, do qual o `FlipDoc` faz parte) — e, o que
   morde: o `index` do hit é uma posição na lista de células **do frame do Begin**. Aplicar
   a cada Update reordenaria a lista sob o próprio gesto.
2. **O drain roda ANTES do `flip_bridge::publish`**, com arch-gate
   (`the_strip_drag_lands_before_the_snapshot.rs`): senão o snapshot deste frame descreve a
   tira de antes do gesto e a célula pisca de volta por um frame. O gate afirma a relação
   **posicional**, nunca distância em bytes — a lição que a `line/Vector` pagou hoje.
3. **A chave ENCOSTA na vizinha** em vez de ser recusada: `move_frame` devolve `false` num
   destino ocupado, e um gesto que às vezes não faz nada ensina intermitência, não a regra.
4. **`floor`, não `round`, na régua**: um quadro é uma FAIXA de pixels. Arredondar faria meia
   célula de arrasto mover a chave um quadro inteiro. (A régua de *scrub* arredonda de
   propósito — lá o handle é um PONTO.)
5. **Os pins são estado de SESSÃO**, e a razão é o custo: o `FlipDoc` viaja DENTRO do
   `ProjectState` **sem versão própria**, então levá-los ao documento seria um campo apendado
   numa struct serializada ⇒ bump de `PROJECT_SCHEMA`, que **recusa todo projeto já salvo** —
   numa janela em que outras linhas também bumpam. **Persistir é decisão do Enio** (§8).

## 5. A auditoria de 2 lentes achou DOIS defeitos, os dois meus

**Lente 1 (costura).** O **Pin nasceu MORTO sob o mouse**: pintado, na lista `BUTTONS`, com
braço no shell — e **fora do `populate`**. O Down do dispatcher só torna ativo um id que
carrega `InteractiveState` no store.

⚠️ **Por que nenhum gate pegava** (e é o que vale para a próxima barra): o gate de pintura
prova que ele PINTA; o `every_toolbar_button_reaches_the_bus` entrega o `WidgetEvent` **já
construído** (pula a focabilidade); e o `architecture_panel_wiring_parity` é **cego para esta
barra**, porque ela registra os hits num **LAÇO** sobre a tabela de itens — não há
`register(ids::X)` literal para ele achar (a mesma cegueira que as 36 células da matriz de
colisão da física documentaram). Gate novo: **`every_toolbar_button_answers_a_real_pointer`**
— pinta, **CLICA** com o ponteiro do dispatcher e exige o evento, para os **18** botões.

**Lente 2 (estado autorado).** **As duas features desta wave se quebravam mutuamente**: um pin
guarda o número do quadro, e arrastar a chave — ou esticar um hold, que **empurra** as
seguintes — deixava o pin apontando um quadro vazio. O fantasma sumia sem ninguém ter soltado
nada. O pin agora acompanha os dois movimentos, e **só quando o documento de fato mudou**. O
delta do empurrão é lido **antes** da escrita (depois dela a exposição já é a nova e a
diferença some — o seed-versus-sample de sempre).

## 6. Gates + provas de mutação

| onde | nº |
|---|---|
| `ph2d-editor-core::interaction::flip_strip` (o canal) | 5 |
| `panel-flip-frames`: `ruler` 4 · `strip_drag` 7 | 11 |
| `panel-flip-frames/tests/seam.rs` (ponteiro REAL: toque, os 2 arrastos, os 18 botões) | 4 |
| `ph2d-flip::onion` (light table) | 4 |
| shell: `flip_strip_drag` 6 · `flip_strip_pin_tests` 2 · `flip_strip_smoke` 2 | 10 |
| arch-gates de shell (ordem do frame · a costura do pin) | 3 |

**10 mutações, 10 sangram:**

| mutação | o que morre |
|---|---|
| célula sem `InteractiveState` no store | os 2 seams de célula (toque + arrasto) |
| grip sem hit registrado | o seam da borda |
| **Pin fora do `populate`** (o bug original) | `every_toolbar_button_answers_a_real_pointer` |
| aplicar a cada `Update` em vez do `End` | 5 gates de unidade |
| alvo absoluto em vez de relativo | 3 |
| `round` no lugar de `floor` | 2 |
| ignorar os pins no `ghosts` | 3 (2 no modelo + 1 no shell) |
| `pinned: &[]` no `present` (o degrau que exige GPU) | o arch-gate da costura |
| drain DEPOIS do publish | o arch-gate de ordem |
| delta do hold lido depois da escrita | o gate do empurrão dos pins |

**Verde rodado na worktree:** `nextest-impacted.sh` → **5393/5393** · `clippy --all-targets`
limpo nas 4 crates · `file_loc_caps` (shell) · `architecture_workspace_file_loc_cap` ·
`architecture_panel_loc_cap` · `no_magic_numeric` · `architecture_panel_wiring_parity` ·
`arch_safe_clamp_only` · `no_tofu_glyphs`.

## 7. O SMOKE — o que falta para o veredito

```bash
# ⚠️ ANTES da integração o smoke só existe na WORKTREE — rodar da raiz abre o
# main, onde a env é ignorada: app vazio, sem faixa, sem cena (aconteceu no
# smoke de 2026-07-23: "não há retângulo nenhum"). Depois do merge, raiz.
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && \
  env PH2D_FLIP_STRIP_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

A cena imprime `[strip-smoke] cena montada: a bola quicando em 4 chaves (0, 4, 5, 11; …)` —
**se essa linha não aparecer, pare**: o resto não significa nada (árvore ou env errada). A cena
é a **bola quicando** (4 poses: alto-esquerda vermelha · caindo amarela · ESMAGADA no chão
ciano · alto-direita verde) sobre um chão fixo — as duas cenas anteriores (barras) reprovaram
por leitura (*"só vejo 4 linhas"* · *"não há retângulo nenhum"*), e o roteiro agora chama as
células de **caixas** para não colidir com nada do canvas. O onion vai **sem fade por
distância** (`fade = false`, gateado): com `1/Δ` o vulto do Pin a Δ=11 cai no piso
`GHOST_MIN_ALPHA = 0.1` — invisível, e o teste 3 não teria veredito. O roteiro completo sai no
terminal; em resumo:

| # | conferir |
|---|---|
| a | **arrastar a caixa**: o contorno mostra onde ela vai cair, e ela só pousa ao SOLTAR; encosta na vizinha e para |
| b | um **clique** simples continua levando o playhead até a chave (tremor de mão não pode mover nada) |
| c | **arrastar a borda direita** da caixa larga (a de 6): ela cresce e as seguintes são EMPURRADAS |
| d | na caixa de **1 quadro** a barrinha do hold **não aparece** — a caixa inteira é de mover (deliberado) |
| e | **Pin** na última chave + voltar ao quadro 0: a bola verde aparece como vulto, **e a vizinha amarela continua lá** |

## 8. O que fica ABERTO (nomeado, não escondido)

| item | gatilho |
|---|---|
| **Persistir os pins** no documento | custa um bump de `PROJECT_SCHEMA` (recusa projetos salvos). Decisão de produto do Enio |
| Arrastar uma **SELEÇÃO** de células | hoje o gesto é por célula; a multi-seleção existe (multiframe) e o canal já carrega `mods` |
| Zoom/pan da tira | ela **sempre cabe**, por desenho (`05 §6`) — só vira pergunta se um documento longo mostrar que a lasca ficou ilegível |
| Backlog anterior da linha | pré-segmentação 4K · `trap_px` × `MAX_SIDE` · o `reach` do Gap Closure · a exceção `rayon` · Shift & Trace · timeline global — **inalterados** |

## 9. Depois da integração

1. `./scripts/ship.sh` **completo**, e corrija todo `✗` antes de qualquer push.
2. **Push só por ordem EXPLÍCITA do Enio** (CLAUDE.md §0.7).
3. **Atualize a §5 do `CLAUDE.md`** com a entrada desta wave — uma §5 que não descreve o que
   está no `main` faz a próxima LLM reconstruir o que existe.
