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
> **SELEÇÃO** (§2.2), o **Shift & Trace / SHIFT** (§2.3) e o **PEEK F1/F2/F3** (§2.4) —
> ✅ **todos com smoke OK (2026-07-24)**. E **as duas promessas quebradas do balde** (§2.5,
> engine-only, provada por gates): o Gap Closure fecha `reach = o VÃO` (pareamento
> ponta-a-ponta — o colinear era CEGO) e o `trap_px` sobrevive ao clamp de `MAX_SIDE`
> (porta única fill+colorize). O remap de sessão virou UMA porta (pins + seleção + folhas
> do trace).

## 1. Identidade

| | |
|---|---|
| branch | `line/FLIP` |
| HEAD | o tip da branch — confira com `git rev-parse line/FLIP` (último descrito aqui: `37d6a5d8c`, os vãos pendentes da §2.6) |
| base do fork (merge-base) | `df91ef6ec` |
| commits à frente do `main` | **23** (7 da wave + cena do smoke ×2 + hold vivo/ghost 0,25 + handoffs/docs ×4 + §2.2 + §2.3 + §2.4 + §2.5 + §2.6 + aparência + pendência do helper) |
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
- ~~**Aberto, nomeado**: o **PEEK** (F1/F2/F3) é a fatia 2~~ — **FECHOU na §2.4.**

### 2.4 O PEEK — F1/F2/F3, o flip de papel (fatia 2, mesma ordem "siga")

Com a tool Flip ativa, **segurar F1/F2/F3** mostra só o desenho **anterior / atual /
seguinte** da camada ativa, na cor real, **sem fantasmas e sem mover o playhead** —
levantar a folha do lightbox para conferir o arco; **soltar volta**. Fecha o item
F1/F2/F3 do `docs/Flip/04 §4`.

- **Duas metades puras** (`flip_peek.rs`): `key_transition` (a política — press só arma
  com a tool Flip; **release SEMPRE desarma**, mesmo com a tool trocada no meio, senão a
  tecla presa deixaria o peek preso) e `peek_frame` (o retime — ⚠️ a âncora é a **CHAVE
  ATIVA**, não o quadro cru: em meio-hold `prev_drawing_key(quadro)` devolve o início da
  exposição ATUAL, o mesmo desenho da tela, e um peek que mostra o que já se vê não é um
  peek; sem vizinho, fica).
- **A costura**: `key_input` consome F1/F2/F3 pela transição pura (o release de teclas
  alheias passa); `collect_layers` ganhou o 7º parâmetro `peek` e retima só a
  camada-alvo (a MESMA resolução do alvo do preview); o shell (`present.rs`) passa
  `ghosts: None` durante o peek (a folha na mão não é pilha translúcida) e ignora peek
  no play.
- ⚠️ **Uma mutação SOBREVIVEU e nomeou a fixture**: "retimar TODAS as camadas" ficou
  verde com um BG de chave única — retimar quem não tem vizinho não o move; o gate
  ganhou um BG com chaves próprias, onde o erro mostra. (3 mutações do passe + 3 da
  política, todas sangrando agora.)
- **F2 não colide com o rename do graph**: aquele F2 é do keymap do painel de grafo
  (Motion); o peek só arma com a tool FLIP ativa, e teclas fora dela não são consumidas.

### 2.5 As duas promessas quebradas do balde (mesma ordem "siga"; engine-only)

As duas dívidas nomeadas do BUGS #23, pagas juntas (detalhe completo no adendo de lá):

- **O Gap Closure fecha `reach = o VÃO`.** O "4× o vão" medido não era ergonomia — era
  MECANISMO: no vão canônico (traço em dois tempos, pontas COLINEARES frente a frente) o
  `ray_hit` trata colinear como paralelo, as extensões se atravessavam sem "colidir", e o
  vão só fechava quando o raio alcançava uma parede DISTANTE por acidente. Cura: **pontas
  emparelhadas** (`gap.rs` passe 3, a ponte do Harmony) — pontas que se apontam a
  `dist ≤ reach` fecham pela reta entre elas; guard de direção (hachura lado a lado não
  vira tubo); emenda ponta-na-ponta não gera par degenerado. O gate da disjunção trocou o
  controle positivo de 4,0 para **1,0 = o vão** (e 0,9 segue vazando).
- **O `trap_px` sobrevive ao clamp de `MAX_SIDE`** (`Grid::px_from_requested`, porta
  única consumida pelo balde E pelo Colorize): o raio do Trap é promessa na escala
  PEDIDA; num desenho grande a grade cede resolução e o raio cru inflava a bola na razão
  do clamp (a "bola de 21,6 doc a 10× de zoom" do doc 09) — no balde isso RECUSAVA com
  `BallTooFat` um clique com folga de sobra (gate red-proven, corredor 2000×2 doc).
  ⚠️ **Achado honesto:** o oráculo comportamental do lado do Colorize NÃO separa (a
  atribuição unifica as câmaras pela moldura de papel externa — medido, cru e convertido
  idênticos); está dito no próprio gate em vez de um verde-por-construção, e a prova
  mora no gate do balde + no gate da porta com os números da lei do clamp à mão.

**Sem UI nova e sem schema**: os dois são a engine honrando o que os sliders já
prometiam. Verificação manual (linha *i* da §7): desenhar uma caixa em DOIS traços
deixando um vão, medir o vão a olho, digitar esse número no Gap Closure — o balde enche.

### 2.6 O Gap Closure AO VIVO (mesma ordem "siga"; fecha o carry-over da W4, doc 06 §8)

A killer feature de UX do GP: em **modo Fill**, cada vão que o alcance ATUAL fecha
aparece como um **segmento verde** no canvas (pontas = pontos reais do desenho), e
**Ctrl+roda** sobre o canvas ajusta o Gap 1 px por tique **com o slider acompanhando** —
o artista vê o que o clique vai selar antes de clicar. A roda CRUA segue sendo zoom
(**divergência deliberada do GP**, que toma a roda inteira durante o fill; inspecionar o
line-art é load-bearing — documentada no próprio `on_mouse_wheel`).

- **A porta é o passo 1 do clique**: `ph2d_flip_fill::preview_closures()` — o `fill_at`
  DELEGA nela, então a tela e o clique não podem discordar sobre quais vãos fecham
  (gate de contrato red-proven sobre o vão canônico do BUGS #23).
- **O custo foi MEDIDO antes do desenho** (`tests/measure_closures.rs`, fica no repo):
  **5 ms** num quadro típico (60 traços), **339 ms** num pesado (300) — recompute por
  frame REFUTADO, e o síncrono por tique de scroll também. Daí o **worker**
  (`flip_gap_live.rs`, o padrão do ajuste ao vivo do Colorize: um em voo, alvo
  coalescido), com duas diferenças deliberadas: **display-only** (zero interação com o
  undo) e **resultado STALE descartado pela chave** (fingerprint de conteúdo + alcance)
  — um helper velho na tela é a feature mentindo. Baratear o kernel (o BVH que o GP usa
  na colisão) é wave própria do engine, nomeada no §8.
- **A roda escreve pelas MESMAS duas metades do slider** (`store.set_slider_value` —
  que **JÁ EXISTIA** em `store_core.rs`, com recentro do chip vinculado; quase criei a
  2ª porta — + o `SetValue` que o tool clampa). O knob do painel acompanha.
- A pose da projeção virou **porta única** (`flip_transform::active_pose`): a autoria
  (`flip_active_pose`) e o overlay chamam a mesma, com borrows diferentes.
- **Unpaint NÃO ganha helper nem roda** (não roda o solver — helper ali prometeria um
  fechamento que o clique não faz); a porta do modo é `wants_gap_helpers`, perguntada
  pelo tick, pelo overlay e pela roda.
- Smoke: **Teste 4** na cena do balde — caixa com vão DELIBERADO de 0,2 unidade em
  tinta FINA (o trade está comentado na cena: fora do alcance da solda 0,12 · dentro do
  teto de 40 px do slider na câmera default).

**Aparência corrigida pós-smoke (Enio 2026-07-25, `9281fb6b8`):** o helper funcionava,
mas ficava "mal desenhado a depender do zoom" (dot flutuante + stub diagonal). O
`closures()` sela um vão de dois jeitos e o overlay desenhava os dois igual: um **par
ponta-a-ponta** (a PONTE de um vão — as duas pontas são pontos reais) e uma **extensão**
(uma ponta esticada na tangente até bater numa parede — a outra ponta é um ponto de
CORTE arbitrário). O dot gordo no corte era o nó que não existe. Cura pela própria lei do
módulo (*as pontas são o FATO, o segmento é a promessa*): `preview_closures` passou a
devolver **`GapHelper { seg, a_is_tip, b_is_tip }`** (anota se cada extremo é uma ponta
real; **o motor do fill ignora os flags** — a porta segue única, o `fill_at` extrai
`.seg`), e o overlay desenha em **duas camadas** — ponte verde cheia com dot nas 2
pontas, extensão como fio fino translúcido **sem dot no corte**. Gate red-proven
`the_helper_marks_only_real_tips_never_a_cut_point` (mutação "sempre tip" sangra). Sem
schema, sem UI nova.

**Só os vãos PENDENTES (Enio 2026-07-25, `37d6a5d8c`):** *"preview ainda abre se zoom de
perto"*. O overlay mostrava TODO fechamento, inclusive nas **junções de traços que se
sobrepõem** — onde a **solda das juntas (`weld`) já veda** porque os corpos pintados se
tocam. ⚠️ **É o que o "de perto" nomeia com precisão:** no zoom de perto o `reach` em doc
ENCOLHE (é `gap_px × px_to_world`), então só as pontas MUITO próximas entram no alcance —
e essas são justamente as junções cobertas; os vãos legítimos (mais largos) caem fora. A
referência do GP é *"helpers visíveis só nos gaps pendentes"*: um vão só é PENDENTE se a
tinta não o cobre, `dist(a,b) > meia-largura(a) + meia-largura(b)` — a MESMA lei da solda,
perguntada para NÃO desenhar onde ela já vedou. `GapHelper` ganhou **`pending: bool`**
(pela porta única `weld::closest_on_segment`, agora `pub(crate)` — uma só lei de
cobertura); o overlay desenha só os pendentes; **o motor do fill ignora o flag** (soma
solda + fechamentos de qualquer jeito, o redundante é parede inofensiva). Gate red-proven
`a_helper_is_pending_only_where_the_paint_does_not_bridge` (junção coberta = não-pendente;
vão de verdade = pendente; mutação "sempre pending" sangra). Sem schema, sem UI nova.

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
| shell: `flip_strip_drag` 9 (6 + os 3 da seleção) · `flip_strip_pin_tests` 5 (2 + os 3 do trace, §2.3) · `flip_strip_smoke` 2 · `flip_trace` 4 · `flip_peek` 3 · `flip_pass` +4 (o shift no model · o peek ×3) | 27 |
| arch-gates de shell (ordem do frame · a costura do pin) | 3 |
| §2.5: `gap` +3 (colinear fecha no vão · hachura não pareia · emenda não degenera) · `tests` +2 (trap no clamp red-proven · a porta com números à mão) · disjunção re-cravada em `reach = vão` · colorize +1 (contrato no clamp, com o achado honesto no doc) | 6 |
| §2.6: engine +2 (contrato preview==clique · o helper marca só ponta REAL, red-proven) + shell `flip_gap_live` 7 (porta do modo/Unpaint · roda 1 px+clamp · reach 0 sem worker · instala+cacheia · segue o alcance · fingerprint invalida · stale descartado) — a costura da roda no `on_mouse_wheel` fica com o smoke (Teste 4), o padrão da costura do PEEK | 9 |
| `ph2d-panel-flip/tests/seam.rs`: as 2 tabelas de varredura ganharam a linha do **Trace** (`FlipMode::ALL` 7→8 as fez morder no nascimento, como projetado) | — |

**36 mutações, 35 sangram + 1 sobrevivente DOCUMENTADO** — as 10 da wave + as 5 da §2.2 (fan-out do grupo cravado na
célula pega · emissão sempre na ordem da lista · preview só da pega · remap de move sem a
seleção · remap do empurrão sem a seleção; e o guard do grupo obsoleto ganhou caso
próprio) + as 6 da §2.3 (o passe ignora o mapa · `pick` pelo mais DISTANTE · hit na caixa
não-posada · rotação em torno da ORIGEM · remap de move sem as folhas · Reset sem o
clear) + as 3 da §2.4 (o passe ignora o peek · retimar TODAS as camadas — ⚠️ sobreviveu à
1ª fixture e a nomeou: BG de chave única não move ao ser retimado; o gate ganhou um BG
com vizinho próprio · âncora no quadro CRU em vez da chave ativa) + as 4 da §2.5
(pareamento morto · guard de direção morto · porta identidade · trap cru no balde — e o
**sobrevivente documentado**: trap cru no COLORIZE, porque o oráculo de lá não separa; o
porquê está no doc do próprio gate) + as 7 da §2.6 (preview com alcance escalado · worker
nunca sai · cache ignorado relança sempre · roda sem clamp · porta aceita Unpaint ·
instalar stale incondicional · fingerprint constante cego a edição) + a 1 da aparência (`a_is_tip`/`b_is_tip` cravados em `true` => o dot volta ao ponto de corte):

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
| g | **Trace** (painel do Flip): arrastar o vulto o desliza (a arte fica); Ctrl+arrastar gira; voltar ao Draw mantém a folha deslocada; **Reset Shifts** devolve (§2.3, ✅ smoke OK 2026-07-24) |
| h | **F1/F2/F3 SEGURADOS** numa caixa do meio: só o desenho anterior/atual/seguinte na tela, sem vultos, playhead parado; soltar volta; na 1ª caixa F1 fica (§2.4, ✅ smoke OK 2026-07-24) |
| i | **(§2.5, opcional)** caixa em DOIS traços com um vão: digitar o TAMANHO DO VÃO no Gap Closure enche; e Trap alto + zoom forte não recusa mais com `BallTooFat` |
| j | **(§2.6, cena PRÓPRIA: `PH2D_FLIP_FILL_SMOKE=1`, Teste 4)** modo Fill: com Gap 0 a caixa de baixo VAZA; **Ctrl+roda** sobe o Gap (o slider acompanha), o helper aparece tapando o vão quando o alcance o atinge, e o clique preenche; a roda SEM Ctrl segue sendo zoom. ⚠️ **Aparência (aprovada 2026-07-25):** a PONTE do vão é verde cheia com dot nas 2 pontas; as extensões são fios finos SEM dot no corte (o "dot flutuante" acabou) — confira em vários zooms |

## 8. O que fica ABERTO (nomeado, não escondido)

| item | gatilho |
|---|---|
| **Persistir os pins** no documento | custa um bump de `PROJECT_SCHEMA` (recusa projetos salvos). Decisão de produto do Enio |
| ~~Arrastar uma **SELEÇÃO** de células~~ | **FECHADO e SMOKADO 2026-07-24** (§2.2) |
| ~~Shift & Trace~~ (SHIFT **e** PEEK) | **FECHADO e SMOKADO 2026-07-24** (§2.3 + §2.4, linhas *g* e *h*) |
| Zoom/pan da tira | ela **sempre cabe**, por desenho (`05 §6`) — só vira pergunta se um documento longo mostrar que a lasca ficou ilegível |
| ~~Ajuste modal ao vivo do Gap Closure~~ | **FECHADO 2026-07-25** (§2.6) — pendente do smoke (linha *j*) |
| **BVH na colisão do `gap::closures`** | wave própria do ENGINE, nomeada pela medição da §2.6 (339 ms num quadro pesado é o custo O(raios×paredes); o GP usa BVH ali). O worker torna o custo pagável hoje; o BVH o tornaria barato |
| Backlog anterior da linha | pré-segmentação 4K · a exceção `rayon` · timeline global — **inalterados** (~~`trap_px` × `MAX_SIDE`~~ e ~~o `reach` do Gap Closure~~ **FECHARAM na §2.5**) |

## 9. Depois da integração

1. `./scripts/ship.sh` **completo**, e corrija todo `✗` antes de qualquer push.
2. **Push só por ordem EXPLÍCITA do Enio** (CLAUDE.md §0.7).
3. **Atualize a §5 do `CLAUDE.md`** com a entrada desta wave — uma §5 que não descreve o que
   está no `main` faz a próxima LLM reconstruir o que existe.
