# Handoff de INTEGRAÇÃO — `line/FLIP` → `main` (a tira ganha MÃOS, 2026-07-23)

> **Para o agente INTEGRADOR.** A linha fechou a wave *"a tira de frames ganha autoria
> direta"*. O implementador parou aqui (CLAUDE.md §0.7).
>
> ✅ **SMOKE OK (Enio, 2026-07-24)** — aquecimento, mover e esticar aprovados. Do mesmo
> smoke saíram duas ordens, ambas LANDADAS nesta linha: **ghost a 0,25 de opacidade** na
> cena (metade do que ele viu; pinado no gate) e **o hold aplicado em TEMPO REAL** durante
> o arrasto (§2.1 — o mover mantém o contorno + commit no soltar, aprovado como estava).
> Todos os gates verdes; auditoria de 2 lentes rodou (2 defeitos achados e corrigidos — §5).
>
> ➕ **FASES SEGUINTES na mesma linha (ordens do Enio 2026-07-24, "siga"):** o arrasto de
> **SELEÇÃO** (§2.2 — ✅ smoke OK 2026-07-24) e o **Shift & Trace, metade do SHIFT** (§2.3
> — o 8º `FlipMode`; **pendente de smoke**, Teste 5 do roteiro / linha *g* da §7). O remap
> de sessão virou UMA porta (pins + seleção + folhas do trace).

## 1. Identidade

| | |
|---|---|
| branch | `line/FLIP` |
| HEAD | o tip da branch — confira com `git rev-parse line/FLIP` (último descrito aqui: o commit da §2.3, Shift & Trace) |
| base do fork (merge-base) | `df91ef6ec` |
| commits à frente do `main` | **15** (7 da wave + cena do smoke reconstruída ×2 + hold vivo/ghost 0,25 + handoffs + §2.2 + §2.3) |
| `main` andou desde o fork? | **não** na última conferência (`git rev-list --count HEAD..main` = 0) ⇒ **fast-forward limpo**; re-confira antes do merge |

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

### 2.1 O hold é VIVO; o mover não (pós-smoke, Enio 2026-07-24)

O mover mantém contorno + commit no `End` (o `index` do hit é posição na lista do Begin —
aplicar por Update a reordenaria sob o gesto). O **hold aplica a cada Update**, e o vivo é
seguro por três fatos, cada um com gate: `set_exposure` não move a chave arrastada nem
reordena a lista · o undo segue **um passo por gesto** porque o `post_frame_undo` suprime o
auto-commit com `held_button` preso (nada foi ensinado à fila) · e a **régua do gesto é
CONGELADA no Begin** (`StripDrag::ruler`) — esticar muda o total de quadros e a tira
re-escala; uma régua viva leria o mesmo x como um quadro maior a cada aplicação =
**realimentação positiva** (gate `the_holds_mapping_is_frozen_at_the_grab`, mutação da régua
viva sangra). O preview do hold morreu (a própria célula estica); o do mover fica.

### 2.2 A SELEÇÃO viaja junta (fase seguinte, ordem do Enio 2026-07-24)

O follow-up nomeado da própria wave: pegar uma célula **marcada** (multiframe W7) move a
seleção INTEIRA pelo mesmo delta; pegar uma não marcada segue movendo só ela. O desenho em
três fatos, todos gateados e mutação-provados (`strip_drag.rs`, doc do módulo):

- **O limite do grupo é o vizinho NÃO marcado** (+ o piso `0`): o grupo anda rígido, então
  marcada nunca colide com marcada; a interseção dos limites por-chave
  (`selection_delta_bounds`) trava o grupo, que encosta e para — a regra do gesto de uma
  célula, generalizada.
- **A ordem de emissão garante que todo `move_frame` pousa**: para a direita, a mais à
  direita anda primeiro (duas marcadas adjacentes movidas `+1` colidiriam na outra ordem — o
  destino ainda estaria ocupado pela irmã, e o `move_frame` RECUSA); para a esquerda, o
  espelho. Gate do shell prova o contrato contra o `move_frame` REAL
  (`a_selection_drag_lands_every_one_of_its_moves`).
- **Uma marcada sozinha é o gesto de sempre** (os limites degeneram nos por-índice, a
  emissão é um pedido só) — o caso comum clique-e-arrasta não muda um byte.

O preview vira **um contorno por marcada** (cada um com a própria exposição); o guard de
obsolescência ganhou a 3ª forma (sessão de grupo cuja célula pega perdeu a marca ⇒ larga).

⚠️ **E um bug LATENTE da wave anterior fechou de carona:** o remap pós-move cobria só os
PINS — a **seleção** (também chaveada por quadro) ficava órfã quando a chave marcada era
movida ou empurrada, **já no arrasto de uma célula** (acento apagado, multiframe mirando um
quadro sem chave). `remap_pin_after_move`/`remap_pins_after_hold` viraram
**`remap_session_after_move`/`_hold`** (`flip_strip_pins.rs`): UMA porta que remapeia pins
E seleção — o próximo estado chaveado por quadro entra ali, não numa 3ª cópia da regra.

### 2.3 Shift & Trace — o SHIFT (fase seguinte, ordem do Enio 2026-07-24, "Siga")

O item do backlog `docs/Flip/04 §4` (OpenToonz), fatia 1: **o papel que desliza no
lightbox**. Um 8º `FlipMode` (**Trace**, chip na 3ª fileira do painel do Flip, ao lado do
Colorize): arrastar no canvas DESLOCA o fantasma sob o cursor; **Ctrl gira** em torno do
centro da arte; **Reset Shifts** (seção do modo) devolve tudo. **Só a exibição** — o
desenho, a pose autorada e o documento nunca mudam; o animador posiciona a referência,
volta ao Draw e traça com ela deslocada.

- **O deslocamento é por CHAVE — a folha** (`FlipStrip.trace: BTreeMap<Frame, Pose>`,
  sessão como pins/seleção, zero schema): deslocar a folha 4 desloca o fantasma dela em
  toda camada. **3º cliente da porta `remap_session_*`** (a arquitetura da §2.2 previu).
- **O shift compõe depois da pose, antes do objeto** (`art_to_world_traced` em
  `flip_transform.rs`; o passe o recebe por `GhostSources.trace` → `GhostRef.shift`).
  Identidade delega ao caminho antigo — **byte a byte** (gateado com mapa vazio).
- **O hit segue o olho**: menor `|Δ|` = o fantasma que o render pinta POR CIMA; e
  pergunta à caixa POSADA (folha já deslocada é pega onde ESTÁ). O Down **consome sempre**
  no modo (a razão do Edit: cair adiante entregaria o clique ao gizmo de objeto).
- **Os gates de varredura do painel morderam no nascimento, como projetado**
  (`FlipMode::ALL` 7→8): as duas tabelas (`each_mode_shows_only_its_own_attributes` ·
  `size_is_shared...`) ganharam a linha do Trace — só o Reset aparece, nada vaza, sem Size.
- **Ids novos** (hash): `flip.mode.trace` + `flip.trace.reset`. O Reset é drenado por
  `flip_strip::apply_panel_event` (a porta que já possui o `strip` — testável sem janela),
  **não** por um braço inline no render_loop.
- **Aberto, nomeado**: o **PEEK** (F1/F2/F3 — mostrar SÓ o desenho vizinho com a tecla
  presa) é a fatia 2; precisa de roteamento de key-release no shell.

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
| `panel-flip-frames`: `ruler` 4 · `strip_drag` 12 (7 + os 5 da seleção, §2.2) | 16 |
| `panel-flip-frames/tests/seam.rs` (ponteiro REAL: toque, os 2 arrastos, os 18 botões) | 4 |
| `ph2d-flip::onion` (light table) | 4 |
| shell: `flip_strip_drag` 9 (6 + os 3 da seleção) · `flip_strip_pin_tests` 5 (2 + os 3 do trace, §2.3) · `flip_strip_smoke` 2 · `flip_trace` 4 · `flip_pass` +1 (o shift no model) | 21 |
| arch-gates de shell (ordem do frame · a costura do pin) | 3 |
| `ph2d-panel-flip/tests/seam.rs`: as 2 tabelas de varredura ganharam a linha do **Trace** (`FlipMode::ALL` 7→8 as fez morder no nascimento, como projetado) | — |

**21 mutações, 21 sangram** — as 10 da wave + as 5 da §2.2 (fan-out do grupo cravado na
célula pega · emissão sempre na ordem da lista · preview só da pega · remap de move sem a
seleção · remap do empurrão sem a seleção; e o guard do grupo obsoleto ganhou caso
próprio) + as 6 da §2.3 (o passe ignora o mapa · `pick` pelo mais DISTANTE · hit na caixa
não-posada · rotação em torno da ORIGEM · remap de move sem as folhas · Reset sem o
clear):

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

## 7. O SMOKE — aprovado 2026-07-24; roteiro para o re-smoke pós-merge

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
| c | **arrastar a borda direita** da caixa larga (a de 6): ela estica **EM TEMPO REAL** (sem contorno — pós-smoke 2026-07-24) e as seguintes são EMPURRADAS |
| d | na caixa de **1 quadro** a barrinha do hold **não aparece** — a caixa inteira é de mover (deliberado) |
| e | **Pin** na última chave + voltar ao quadro 0: a bola verde aparece como vulto, **e a vizinha amarela continua lá** |
| f | **Shift+clique** na primeira e na última caixa (marcam) + arrastar uma delas: DOIS contornos, as duas pousam JUNTAS ao soltar, o destaque acompanha; arrastar uma NÃO marcada move só ela (§2.2, ✅ smoke OK 2026-07-24) |
| g | **Trace** (painel do Flip): arrastar o vulto o desliza (a arte fica); Ctrl+arrastar gira; voltar ao Draw mantém a folha deslocada; **Reset Shifts** devolve (§2.3, pendente de smoke — Teste 5 do roteiro) |

## 8. O que fica ABERTO (nomeado, não escondido)

| item | gatilho |
|---|---|
| **Persistir os pins** no documento | custa um bump de `PROJECT_SCHEMA` (recusa projetos salvos). Decisão de produto do Enio |
| ~~Arrastar uma **SELEÇÃO** de células~~ | **FECHADO e SMOKADO 2026-07-24** (§2.2) |
| ~~Shift & Trace~~ (a metade do SHIFT) | **FECHADO 2026-07-24** (§2.3) — pendente de smoke (Teste 5 / linha *g*). A metade do **PEEK** (F1/F2/F3) fica: fatia própria, precisa de key-release no shell |
| Zoom/pan da tira | ela **sempre cabe**, por desenho (`05 §6`) — só vira pergunta se um documento longo mostrar que a lasca ficou ilegível |
| Backlog anterior da linha | pré-segmentação 4K · `trap_px` × `MAX_SIDE` · o `reach` do Gap Closure · a exceção `rayon` · timeline global — **inalterados** |

## 9. Depois da integração

1. `./scripts/ship.sh` **completo**, e corrija todo `✗` antes de qualquer push.
2. **Push só por ordem EXPLÍCITA do Enio** (CLAUDE.md §0.7).
3. **Atualize a §5 do `CLAUDE.md`** com a entrada desta wave — uma §5 que não descreve o que
   está no `main` faz a próxima LLM reconstruir o que existe.
