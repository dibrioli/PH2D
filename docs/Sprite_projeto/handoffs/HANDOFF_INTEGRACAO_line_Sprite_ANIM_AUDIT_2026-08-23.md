# HANDOFF — `line/Sprite` · auditoria do transporte da §11 Animation (2026-08-23)

> **Pedido do Enio:** *«às vezes preciso clicar mais de uma vez para checar Playing. Corrija isso e
> faça auditoria completa do sistema de animação da sprite.»*
>
> **Continuação de:** [`HANDOFF_INTEGRACAO_line_Sprite_MOUNT_2026-08-23.md`](HANDOFF_INTEGRACAO_line_Sprite_MOUNT_2026-08-23.md)
> (mesma linha, mesma jornada). A §11 tinha um dia de vida.
>
> **A auditoria completa, com o mecanismo de cada achado e as recusas medidas, está em
> [`docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md`](../21_auditoria_da_animacao_2026-08-23.md).**
> Este handoff é o registro de integração: o que mudou, onde, e o que o integrador tem de saber.

## §1 — O achado que resume a wave

O report era a ponta de uma família. **Todos os quatro defeitos vivem no mesmo estado** — depois de
uma animação de uma volta chegar ao fim —, e nenhum aparece numa sprite que só o painel tocou:

| # | Defeito | Mecanismo |
|---|---|---|
| F1 | a caixa «Playing» precisava de 2 cliques | **dupla fonte de verdade**: pintava do snapshot, decidia do `WidgetStore`; o motor escreve `playing` sozinho e a semente do sync só corre em aresta de entidade/linha |
| F2 | ligar uma animação terminada era um gesto **morto** | `playing = true` com o contador cheio e a imagem na ponta ⇒ o 1.º passo de `advance` re-fecha o ciclo |
| F3 | «Rewind» não movia a imagem | `rewind` zera contadores que ninguém vê; `advance` só reposiciona um frame **fora** do intervalo |
| F4/F5 | escolher outra animação começava-a a meio, ou deixava a sprite muda | os intervalos **partilham o pool** (a tese do modelo), e `SetCurrent` não tocava no `playing` |

⇒ **Uma lei nova, statable numa linha:** *a reprodução que se ESGOTOU volta ao princípio quando
alguém lhe toca — e escolher outra animação é tocar-lhe. Uma pausa explícita não é tocada.*

## §2 — O que mudou, por ficheiro

| Ficheiro | Mudança |
|---|---|
| `crates/ph2d-ecs/src/sprite_anim.rs` | **+2 API**: `SpriteAnimator::is_finished(tag)` e `entry_frame(animator, tag, cells)`. Nada removido, nada renomeado |
| `crates/ph2d-ecs/src/lib.rs` | re-export de `entry_frame` |
| `crates/ph2d-panel-inspector/src/event_anim.rs` | as duas caixas derivam do **snapshot** (`!info.playing`), não do store |
| `crates/ph2d-panel-inspector/src/sync_sections.rs` | as duas caixas passam a **espelho por quadro** (a exceção à lei das irmãs, documentada no sítio) |
| `crates/ph2d-panel-inspector/src/sections/anim.rs` | a linha de aviso de **seleção múltipla**, antes de qualquer controlo |
| `crates/ph2d-editor-core/src/screens/hero/inspector_model_anim.rs` | **−2 campos** de `InspectorAnimInfo`: `library_present` e `mixed` (calculados, nunca lidos) |
| `shells/desktop/src/render_loop/inspector_anim.rs` | `rewind_to_start` (o rebobinar completo, com o `Sprite::frame`) · `current_tag` · os braços `SetCurrent`/`Playing`/`Rewind` · `build_anim_info` perde o parâmetro `selected` e o clone por quadro |
| `shells/desktop/src/render_loop/snapshots.rs` | a chamada acompanha a assinatura |
| `shells/desktop/src/render_loop/mod.rs` | um comentário que **afirmava o contrário do código** (o `tick` de passo grande) |
| `shells/desktop/src/anim_smoke.rs` | dois passos novos no roteiro (Rewind · re-tocar o `attack`) |

### ⚠️ Superfície tocada FORA do módulo

- **`ph2d-ecs`** — só **acrescenta** duas funções públicas ao `sprite_anim`, que nasceu nesta linha.
  Nenhum contrato congelado (§6) é tocado; nenhum registo de componente mudou (segue em **69**),
  e o `PROJECT_SCHEMA` **não se move** (nenhum layout de componente mudou).
- **`ph2d-editor-core`** — **remove dois campos** de `InspectorAnimInfo`, struct que nasceu nesta
  linha ontem. ⚠️ **Risco de merge:** uma linha concorrente que tenha acrescentado um campo a essa
  struct funde limpo (adições disjuntas), mas uma que tenha passado a **ler** `mixed`/
  `library_present` quebraria a compilação — `git grep` no `main` de 2026-08-23 dá zero
  consumidores fora deste módulo.

## §3 — Gates novos (todos com mutação a sangrar)

**`crates/ph2d-panel-inspector/tests/seam_anim.rs` — 10 gates, ficheiro NOVO.** A §11 tinha 20
gates da lei pura e 13 do commit, e **zero** que carregassem num pixel — o defeito reportado vive
exatamente entre os dois. Irmão do `seam_player.rs`, com `click_at` real.

- `the_playing_box_asks_the_scene_not_its_own_memory` ⭐ **corrido RED-FIRST** (falhou com
  `Playing(false)` onde tinha de mandar `Playing(true)`), verde depois da cura
- `the_autoplay_box_asks_the_scene_not_its_own_memory` (o irmão latente)
- `every_player_control_reaches_the_bus` · `every_library_control_reaches_the_bus`
- `clicking_a_row_picks_what_plays_and_moves_the_editor_with_it`
- `every_edit_the_model_declares_is_reachable_by_a_gesture` — as **18** variantes de
  `AnimFieldEdit`, com `match` exaustivo (uma variante nova **não compila** até ser amostrada)
- `the_empty_face_offers_only_the_gesture_that_creates_the_player`
- `the_library_fields_show_what_was_authored_not_the_seed`
- `a_multiple_selection_says_so_before_offering_any_control`

**`crates/ph2d-ecs/src/sprite_anim_tests.rs` — +2** (`a_finished_animation_is_told_apart_from_a_paused_one`,
`the_entry_cell_is_the_end_the_effective_direction_starts_from`). Total 22.

**`shells/desktop/src/render_loop/inspector_anim_transport_tests.rs` — ficheiro NOVO** (irmão por
HR-18: o pai chegou a **617/600** depois dos gates novos, e o corte é por LEI — transporte aqui,
autoria lá). 4 gates.

**`shells/desktop/src/render_loop/inspector_anim_tests.rs` — +1**
(`the_panel_and_the_engine_agree_on_a_dangling_playback`).

### As 11 mutações

| # | Mutação | Sangrou em |
|---|---|---|
| M1 | `rewind_to_start` volta a ser `p.rewind(tag)` | 3 gates do transporte |
| M2 | tirar o `is_finished` do braço `Playing` | `turning_playing_back_on_replays…` |
| M3 | `Playing(true)` rebobina **sempre** | `turning_playing_back_on_replays…` (a metade da pausa) |
| M4 | o despacho volta a ler `store().checkbox(id)` | `the_playing_box_asks_the_scene…` (**este foi o red-first**) |
| M5 | `is_finished` usa `>` em vez de `>=` | `a_finished_animation_is_told_apart…` |
| M6 | `entry_frame` ignora a direção | `the_entry_cell_is_the_end…` |
| M7 | `SetCurrent` nunca retoma | `choosing_another_animation_resumes…` |
| M8 | `SetCurrent` retoma **sempre** | `choosing_another_animation_resumes…` (a metade da pausa) |
| M9 | o aviso de seleção múltipla desaparece | `a_multiple_selection_says_so…` |
| M10 | o aviso aparece **sempre** | `a_multiple_selection_says_so…` |
| M11 | `current_dangling` esquece o `fits` | `the_panel_and_the_engine_agree…` |

## §4 — ⚠️ O que foi MEDIDO e NÃO curado (decisão do Enio)

**Um quadro de reprodução mexe num componente registado.** O `SpriteAnimator` guarda o relógio
(`elapsed_ticks`/`repeat_count`/`pingpong_reverse`) e está no `ComponentRegistry` ⇒ entra no
`ProjectState`, que é a unidade do undo. Com a animação a tocar, **um quadro que tenha input regista
um passo cujo conteúdo é só o relógio**.

⚠️ **Não é um defeito da §11 — é uma propriedade do undo do app.** A ponte de física escreve o
`Transform` (também registado) a cada passo enquanto o mundo simula. As três saídas possíveis e por
que nenhuma se aplicou daqui estão na [auditoria §4](../21_auditoria_da_animacao_2026-08-23.md).

## §5 — Estado da linha

- **Nada foi integrado, nada foi pushado.** A linha fecha e PARA (§0.7).
- `PROJECT_SCHEMA` **inalterado** por esta wave (segue no valor que o handoff MOUNT registou);
  registos **69 / 70 / 70**; nenhum contrato congelado tocado.
- Recusas medidas desta wave (⛔ não reconstruir sem ler): alcance nos campos da biblioteca ·
  alcance em `from`/`to` · clicar na lista tocar sempre · limpar o `current` ao apagar uma
  animação · dicas de hover. Todas na [auditoria §5](../21_auditoria_da_animacao_2026-08-23.md).

## §6 — Adenda: a barra de frames arrasta (pedido do Enio, mesmo dia)

*«permita arrastar manualmente o slider de frames»* — e ela **não era um slider**: dois retângulos
pintados à mão, sem id e sem entrada no store. Bonita, informativa, morta sob o rato.

**Cura:** `ids::INSP_ANIM_FRAME_SCRUB` registado como `InteractiveState::Slider`. O despachante dá
o salto-ao-clique e o arrasto **sem máquina nova** (a mesma porta da Opacidade). A trilha subiu de
6 para 10 px — o retângulo de acerto **é** a trilha, então a altura dela é o alvo do dedo.

- **A régua mudou de progresso para POSIÇÃO** (`passo/(total-1)`): uma barra que se agarra mede
  «onde está», não «quanto passou».
- **Agarrar PAUSA** (`AnimFieldEdit::SetFrame` põe `playing = false` e zera o `elapsed_ticks`), e o
  clamp ao intervalo é do **commit**, não do painel — o snapshot é de um quadro atrás.

### ⭐ A mutação que SOBREVIVEU, e o que ela mudou no desenho

O gate do seam afirmava apanhar a troca posição↔progresso pela ponta esquerda. **Não apanhava** —
o caminho do clique (`x → 0..1 → célula`) não passa pelo pintor. A causa era mais funda: **a régua
existia em TRÊS cópias** (pintor · `sync` · despacho). Hoje é uma lei em dois sentidos no modelo,
`InspectorAnimInfo::scrub_position` ↔ `scrub_cell`, com gate de ida-e-volta sobre cada célula do
intervalo. *Um sobrevivente não é um gate que falta: é o desenho a dizer onde está a duplicação.*

**Gates novos (3):** `seam_anim::the_frame_bar_is_dragged_not_just_looked_at` ·
`inspector_anim_transport_tests::dragging_the_frame_bar_sets_the_cell_pauses_and_stays_inside_the_range` ·
`inspector_model_anim::the_scrub_position_and_the_cell_are_inverses`.

**Mutações (4):** M12 «a régua volta a progresso» (sobreviveu na 1ª forma; sangra na lei unificada)
· M13 «a barra sai do `populate`» → *«MORTA sob o rato»* · M14 «`scrub_cell` trunca» ·
e as duas metades do commit (sem `playing = false`, sem `clamp`).

⚠️ **`AnimFieldEdit` ganhou a variante `SetFrame(u32)`** — e o
`every_edit_the_model_declares_is_reachable_by_a_gesture` **reprovou a compilação** até ela ter
gesto. *O gate de alcance fez exactamente o trabalho para que foi escrito, no dia seguinte.*

**Suíte:** 17.945/17.945.

## §7 — Adenda: a grelha vê-se no canvas (item 1 do inventário, pedido do Enio)

*«você digita 8 quadros e não vê onde eles começam ou terminam»* — um sprite com grelha desenha
**uma célula**, então nada no canvas dizia onde os cortes caem. Arrastar a barra de frames mostra
*que* algo está errado; não mostra *onde*.

**A caixa «Show sheet on canvas»** (§4 Sprite Sheet) abre a folha: as outras células, esmaecidas,
no lugar delas, com as linhas dos cortes por cima e a célula viva contornada.

| Peça | Ficheiro | Natureza |
|---|---|---|
| o fan-out puro | `render_loop/sim_extract_sheet.rs` | **molde do 9-slice**: função pura + chamador que só coloca |
| as linhas | `render_loop/sheet_grid_overlay.rs` | irmão do `sheet_overlay` (que decora a folha-OBJETO) |
| o interruptor | `sections/sprite_sheet.rs::sheet_preview_row` | irmã por CAP de função (209/200) |
| o id | `ids::INSP_SHEET_PREVIEW` | **vista**, não documento |

### As três decisões

1. **Fantasmas de PRESENTE.** As células extra não existem no `SimWorld`, não entram no undo, não
   são salvas e somem com o interruptor — a natureza do `override_for_entity`, e o oposto do
   9-slice, cujos nove quads **são** o que o sprite é. Partilham o `SimRef` e levam
   `SlicePatchMirror` pela razão que o irmão documenta.
2. **O interruptor não passa pelo barramento.** A shell lê `hero.store.checkbox(…)` no quadro. Uma
   `EditorAction` levá-lo-ia ao commit, ao undo e ao save — e o artista reabriria o projeto com a
   folha aberta sem se lembrar de a ter aberto. ⚠️ É a única caixa do Inspector que ninguém
   despacha, e o gate afirma-o pelo **estado que ela deixa**, não por um evento.
3. **Quem decide lê o que o canvas lê.** A caixa aparece a partir do `hframes` do **snapshot**, e
   não do campo do store — a lei que a caixa «Playing» pagou no mesmo dia. Pelo campo, ela
   existiria a meio de uma edição, antes de o mundo ter grelha, e ligá-la não mostraria nada.

### Gates (14 novos) e o buraco que fica declarado

`sim_extract_sheet` (7): sem grelha não há folha · a viva nunca ganha fantasma · a grelha abre
`+X`/`−Y` · a sub-UV é a **mesma** que o extract dá àquele frame · o fantasma soma ao pivô e
esmaece · o flip espelha a grelha · a decisão de abrir.
`sheet_grid_overlay` (5): sem grelha não há retículo · abre à volta da viva e para baixo · segue o
pivô autorado · o flip abre para o outro lado **e a viva não se move** · ⭐ **as linhas caem sobre
as células que os fantasmas desenham** (o gate que liga os dois módulos, que derivam a posição por
caminhos diferentes — sem ele cada um passa sozinho e as linhas caem no meio dos desenhos).
`the_sheet_grid_switch_…` (2): só existe onde há grelha · o clique muda o valor que a shell lê.

⚠️ **O que NÃO tem gate, e está escrito no código:** o laço que de facto emite as células. O
`sim_extract::run` pede um `SpriteRenderer` vivo, então ele **não é alcançável de um teste** — o
mesmo buraco que deixa o fan-out do 9-slice sem gate de emissão e os quatro goldens da spec por
escrever (falta o arnês headless). O que se fez foi **encolher o resíduo**: a decisão saiu para
`should_open` (gateada), e o que fica é um `for` sobre duas funções já gateadas.

**Mutações (5):** a viva ganha fantasma · o `Y` da grelha inverte (sangra nos **dois** módulos, que
é a prova de que o gate cruzado morde) · o retículo ignora o pivô · o retículo não espelha · o
interruptor aparece sempre.

⛔ **Fora, com motivo:** clicar numa célula para escolher o frame — pede hit-test de canvas a
competir com a seleção, e a barra de frames e o campo `Frame` já respondem a isso.

**Smoke:** `PH2D_ANIM_SMOKE=1`, passo 9.

## §8 — Adenda: pintar uma folha (report com foto do Enio, 2026-08-23)

*«ao tentar usar o canvas para editar veja como fica… Vc criou a imagem quadrada mesmo? ou está
sendo achatada? Para editar com o painter precisamos de cada quadro no seu lugar. e precisamos de
um preview animado enquanto editamos com o painter.»*

**A imagem não estava achatada — o QUAD estava.** Sob pré-visualização de ferramenta o extract
troca o `atlas_uv` pelo rect **inteiro** da textura transitória (que é o bake da imagem toda), e o
quad continuava a ser o de **uma** célula: oito células dentro de uma ⇒ tira esmagada 8:1.

⚠️ **E o caminho do PONTEIRO fazia a mesma conta.** O `sprite_image_to_screen_affine` mapeia a
imagem inteira sobre o `Sprite::size` — o que deixava o render e o ponteiro **consistentes um com o
outro e errados com o artista**. É por isso que a cura não podia viver só no render: os dois
passaram a chamar `sim_extract_sheet::unfolded_quad`, e o `affine` é o seam de **todos** os
overlays do Painter (pincel, gizmos de seleção, curva, fill) — os dez chamadores seguem juntos.

### ⭐ A versão que foi medida e substituída, no mesmo dia

A 1.ª forma ancorava a folha desdobrada **na célula viva**, para a arte não saltar ao pegar no
pincel. **Media errado o preço:** o `Sprite::frame` continua a andar durante a pintura (o tique é
independente), então o desvio mudaria a cada quadro e a folha **deslizaria debaixo do pincel**.

⇒ A folha desdobrada centra-se no **pivô** e não depende do frame. O que salta é uma vez, ao abrir,
e lê-se como *«a folha abriu»*. ⚠️ A pré-visualização da grelha (`Show sheet on canvas`) faz o
**contrário** e também está certa — ali a célula viva **é** o quad real do sprite, então a folha
tem de se dispor à volta dela. *Dois modos, duas âncoras, e a diferença é qual dos dois desenha a
célula viva.* ⛔ O gate que afirmava a igualdade das duas disposições **saiu**, com o motivo escrito
no topo do ficheiro de testes.

### O preview animado, e por que ele é consequência e não enfeite

Com o quad a cobrir a imagem inteira, o `Sprite::frame` **deixa de ter efeito visível**: o artista
pinta oito desenhos e não vê a animação que eles formam. O `anim_preview_quad` é a resposta — uma
célula, por fora da folha (acima: sobreposta taparia o que se pinta), com a sub-UV do frame vivo,
que anda porque o tique anda.

**Gates novos (2) + 3 mutações:** o quad cobre a folha e **ignora o frame vivo** (M20: não desdobra
⇒ volta o esmagamento) · o preview segue o frame vivo e fica **fora** da folha (M21: congela no
frame 0 · M22: fica em cima da folha).

## §9 — ⛔ Recusa medida: «H Frames não atualiza em tempo real»

*«Com show sheet on canvas checado, se mudo H frames, a imagem não atualiza em tempo real.»*

**Medido: as setas e o arrasto JÁ atualizam ao vivo.** O `apply_number_stepper_if_hit` levanta
`ValueChanged` no clique (`pointer_down`), e o arrasto levanta-o **a cada `Move`**
(`pointer_move`). O que não atualiza é **digitar**: teclar levanta `TextChanged`, e o commit (com o
`ValueChanged`) só sai no Enter ou ao sair do campo.

⛔ **Isso é o comportamento de TODOS os ~141 campos numéricos do app**, e mudá-lo para um deles
faria a §4 responder ao teclado de forma diferente de todos os vizinhos.

⛔ **A alternativa medida — a pré-visualização seguir o BUFFER digitado — foi recusada pelo preço:**
exige um «grelha pendente» atravessando o extract **e** a célula viva a re-fatiar com ele (senão os
fantasmas mostrariam 7 e o quad real 8), o que ripplaria por nove gates para cobrir um caso que
duas gestos já resolvem, um deles com as setas visíveis ao lado do campo.

⇒ **Se o Enio insistir, o caminho está descrito acima e é ~30 linhas.** A recusa é de preço, não de
desenho.

## §10 — Adenda: os dois defeitos do 2.º smoke da folha (report com foto)

*«as imagens ficaram deslocadas da grade ao ativar o painter e o preview não está animado»*

### D1 · As linhas desalinhadas — e o defeito era MEU, da adenda anterior

A §8 deu à folha pintada uma âncora nova (o **pivô**, para ela não deslizar sob o pincel) e
**esqueceu o overlay**, que continuou a dispor a grelha à volta da **célula viva**. As duas contas
— `pivô − (lcol + ½)·cw` e `pivô − hf·cw/2` — só coincidem quando `lcol = hf/2 − ½`, que não é
inteiro: elas **nunca** coincidem. O desvio é `(lcol + ½ − hf/2)·cw`, e vale meia célula no caso
fotografado (8 células, viva na 4).

⇒ O `lattice` passa a ter **duas disposições**, e a diferença é *qual dos dois modos desenha a
célula viva*: dobrada, o quad real **é** a célula viva; desdobrada, há um quad só e a viva ocupa o
slot dela. ⚠️ **A 1.ª versão do gate escreveu «meia célula sempre» e sangrou na hora** — a fórmula
é a acima, e meia célula é o valor de UM caso.

### D2 · A pré-visualização parada — e a lei pura era a razão

Bastava o artista ter pausado uma vez (**arrastar a barra de frames pausa**, por desenho) para a
célula de pré-visualização nascer estática. ⚠️ E abrir o guarda do tique **não chegava**: a
[`ph2d_ecs::advance`] também desiste com `playing == false` — é ela que define o que «tocar»
significa.

⇒ A pré-visualização corre sobre uma **cópia** do animador (`playing = true`,
`loop_override = Some(true)` — uma de uma volta congelaria na última célula), e do que ela produz
volta só o **relógio** e o frame. O `playing` do documento fica intacto: sair da ferramenta devolve
a cena como estava, e o gate afirma-o.

⚠️ **Terceiro defeito, encontrado ao arrumar:** a sub-UV da célula de pré-visualização usava o
`sheet_uv` (um rect do **atlas**) sobre a textura transitória (a imagem **inteira**). Acertava por
acidente num sprite de textura própria, onde os dois são o rect unitário, e mostraria lixo num de
atlas. *Um acerto que depende da fonte não é um acerto.*

### ⭐ E um gate que estava verde por acidente

O `a_sheet_under_a_tool_preview…` comparava o frame final com o inicial. Depois de 20 tiques a
`walk` tinha dado a volta e **voltado ao 0** — a asserção media o ponto final de um ciclo. Hoje ela
conta **quantos frames distintos** passaram.

### O predicado que estava em três sítios

«Uma ferramenta pré-visualiza esta entidade?» era lido pelo tique, pelo extract e pelo overlay,
escrito nos três. A mutação que o tirava do overlay **compilava e passava** — o `draw` não é
alcançável de um teste. ⇒ `is_tool_previewed`, uma função com gate, e a lista de fontes
(`tool_preview_bits`) calculada **uma vez por quadro** no `render_loop`.

**Gates novos (3), mutações (3):** o retículo desdobrado casa com o quad pintado (e difere do
dobrado pela fórmula) · a folha em pintura toca com o transporte parado **e não o liga** · uma
resposta só a quem está sob pré-visualização.

## §11 — Adenda: a caixa do gizmo envolve a folha

*«quando Show sheet on canvas estiver checado o gizmo da sprite deve englobar todas as células. Veja
que agora ele fica com o tamanho de uma célula no centro.»*

O `snapshots::build_view` derivava a caixa de `sprite.size` + `resolve_anchor` — a célula. Agora ela
sai do **mesmo `Lattice`** que desenha as linhas, pelos dois modos (dobrada/desdobrada).

⚠️ **Escalar a caixa escala a folha toda, e isso sai de graça:** as células fantasma derivam de
`Sprite::size`, então mover/girar/escalar o sprite move a folha inteira. A caixa passa a cercar o
que o gesto de facto move — que é o que ela devia sempre dizer.

### ⭐ A mutação que SOBREVIVEU, e o que ela mudou

Desligar a caixa nova em `build_view` **compilava e passava a suíte inteira**: aquele closure precisa
de `HeroScreen` + `PresentWorld` + câmara e **não é alcançável de um teste**. ⇒ a **escolha** saiu
para `sheet_grid_overlay::gizmo_box` (com gate nas duas metades: cresce com a folha aberta, e **não**
cresce numa sprite sem grelha ou com a caixa desmarcada), e no fio ficaram duas linhas.

*É a terceira vez nesta linha que um sobrevivente aponta o mesmo remédio: quando o arnês não existe,
encolhe-se o resíduo e diz-se qual metade ficou de fora. Fingir que ele existe, não.*

**Gates novos (2), mutações (2):** o retículo é a caixa que o gizmo envolve · a caixa cresce só com
a folha aberta (M27: nunca cresce · M28: cresce sempre, e mata a sprite normal).

## §12 — Adenda: **o Ctrl+Z passa a distinguir PRÉ-VISUALIZAÇÃO de DOCUMENTO**

Enio, 2026-08-23: *«precisamos corrigir o CtrlZ para ambas»* — **ambas** = a §11 e a física. É a
§4 deste handoff, e a auditoria [21 §4](../21_auditoria_da_animacao_2026-08-23.md) já dizia que a
cura estava na terceira saída da tabela: *o conceito que o app não tinha*.

**A lei**, e ela vive em [`preview_drive.rs`](../../../shells/desktop/src/preview_drive.rs):

> O documento é o valor **AUTORADO**. O que um motor está a escrever agora é pré-visualização:
> vê-se, não se guarda nem se desfaz.

O motor continua a escrever no mundo — **um só sink**, e o render lê o campo de sempre. O que muda é
a **captura**: ela repõe o autorado durante a fotografia e devolve o vivo a seguir.

### O que a construção MEDIU e a tabela da auditoria não sabia

| # | medição | consequência |
|---|---|---|
| 1 | O passo nasce **por CLIQUE**, não por quadro: o `any_input_this_frame` não é levantado por mover o cursor | vinte cliques com algo a correr = vinte Ctrl+Z mudos. É assim que o defeito se sente |
| 2 | Logo a saída 2 (tirar o relógio do componente registado) é **necessária e não suficiente** | no instante do clique o `Sprite::frame` já avançou desde o último baseline ⇒ o defeito fica inteiro, e ainda move o `PROJECT_SCHEMA` |
| 3 | O tique da §11 anda no **passo fixo do relógio de parede**, não no `playhead` | «enquanto toca» **não tem fim** ⇒ a saída 1 é pior do que a tabela dizia |
| 4 | São mesmo **dois casos**: o relógio nunca foi documento; a pose do solver é documento com um escritor a mais | um mecanismo só serve os dois porque a granularidade é o **CAMPO** |

### As três decisões de desenho

1. **O ledger entra na ASSINATURA da `ProjectState::capture`**, e não numa função-irmã «com ledger».
   *Uma segunda porta é exactamente como o defeito voltaria* — quem capturasse pela antiga
   fotografava o instante. Com ledger vazio (o caso normal) o custo é zero e o resultado é
   byte-a-byte o de antes, e há gate.
2. **A lei da OUTRA MÃO:** se o que o motor encontra não é o que ele deixou, alguém autorou por
   cima ⇒ o autorado passa a ser essa mão. Sem ela, uma edição feita a meio de uma corrida ficava
   **para sempre** por baixo do memo do início dela.
3. **A `settle` faz a corrida virar UM passo:** enquanto o motor conduz não há passo nenhum; quando
   ele larga, a captura seguinte vê o vivo e regista um — *«desfaz a corrida»*.

⚠️ **Vale para o SAVE pela mesma porta** (a lei do módulo: undo e save gravam a MESMA captura).
Gravar a meio de uma reprodução guarda a célula que o artista escolheu. Para a física é a frase que
o ADR-0131 já lhe dá: *runtime-truth + **bake opcional*** — e **depois** de a corrida parar a pose
caída é documento outra vez.

### ⛔ O terceiro membro da família, medido e por curar

A **timeline** escreve `Transform` a partir das curvas enquanto o playhead toca
(`ph2d-timeline`: `apply.rs`, `apply_prop.rs`) — o mesmo defeito. Fora desta wave por duas razões
medidas: (1) a escrita mora **dentro** da crate da timeline, então declarar de lá inverteria a
dependência ou exigiria que o `apply` devolvesse a lista de entidades escritas — API de outro
módulo; (2) do lado do shell a alternativa é um **censo de todas as poses por quadro de
reprodução**, e esse custo é um número que ninguém mediu. ⚠️ E o `timeline_bridge::run` **não tem
arnês headless nenhum** (zero chamadores fora do laço de quadro): a wave começa por construir um.

**12 gates** (11 no ledger + 1 na porta REAL da física, com `PhysicsBridge` vivo), **9 mutações**,
todas sangraram.

## §13 — Adenda: **importar Aseprite (`.ase`)**

Enio, 2026-08-23: *«Precisamos Importar Aseprite (.ase)»*. Duas peças: a crate-folha
[`ph2d-aseprite`](../../../crates/ph2d-aseprite/) (pura: bytes → quadros RGBA8 + tags) e a costura
[`ase_import.rs`](../../../shells/desktop/src/ase_import.rs).

⚠️ **Clean-room a partir da spec pública.** O Aseprite é GPLv2; a especificação do formato é
documentação que o próprio projeto publica. Ler um formato descrito não é obra derivada do programa
que o escreve, e nada foi traduzido de fonte dele.

### As leis

* **O corte entre as duas portas é o que cada uma SABE.** O par `.png`+`.json` (que já existia) traz
  **rectângulos com nome** ⇒ N sprites soltas. O `.ase` traz a **autoria** ⇒ **UMA** sprite com
  grelha + a biblioteca de animações, que é o modelo da §11. Não é o mesmo import com outra extensão.
* ⚠️ **A ORDEM DOS QUADROS É O CONTRATO.** Uma `AnimationTag` indexa **células**; uma tag do Aseprite
  indexa **quadros**. Os dois só coincidem enquanto a folha for empacotada em linha, da esquerda
  para a direita e de cima para baixo. Por colunas dá uma folha bonita e todas as animações trocadas.
* **Uma TIRA sempre que ela couber** — é o que o Aseprite exporta por omissão e faz o `hframes` do
  inspector ser legível. O teto é `MAX_SHEET_EDGE_PX = 8192`, e o recurso é **memória de GPU**
  (`max_texture_dimension_2d`); quando estoura, a mensagem traz **os dois** números.
* **`repeat: 0` do ficheiro é `None` na §11** — as duas dizem *para sempre* com valores diferentes, e
  trocá-los faria toda animação importada tocar **uma vez** e parar.
* **Um ficheiro sem tags recebe uma**, com o nome do ficheiro, a cobrir todos os quadros (é o que o
  próprio Aseprite faz ao exportar) — senão a sprite nasce com grelha e nada para tocar.
* **Cel LIGADO** (tipo 1) é o modo de falha nº 1 do formato: o Aseprite guarda um quadro
  não-redesenhado como referência, e tratá-la como ausente faz a animação **piscar** exactamente nos
  quadros que o artista deixou como estavam.

### ⛔ A recusa medida que ele REABRIU

A **duração por-FRAME** (spec §8.12) foi recusada por *«não haver quem a produza»*. Há: é este
importador. O que shipa **aproxima pela duração mais comum da tag e DIZ**, nomeando a tag e o número
— aproximar em silêncio faria o *hold* de antecipação do artista desaparecer sem uma linha a
explicar. ⏳ **Pôr a duração por-quadro no modelo é decisão de produto** e move o `PROJECT_SCHEMA`;
a crate já devolve o dado (`AseTag::uniform_duration_ms` ⇒ `None` **é** a informação).

### O resíduo por gatear, declarado

O `import_ase` pede um `SpriteRenderer` vivo ⇒ a função inteira não é alcançável de um teste. Foi
por isso que as três decisões saíram para funções **puras** (`grid_for`, `pack`,
`library`/`tag_from_ase`), que é onde um erro faria a animação sair trocada. Ficam por gatear: a
subida da textura, o `hframes`/`vframes` escritos na sprite, e o `size` dividido pela grelha —
**três linhas**, cobertas pelo smoke.

⭐ **O smoke ESCREVE o ficheiro** (`PH2D_ASE_SMOKE=1`), então testá-lo não precisa do Aseprite
instalado, e larga-o pelo **mesmo** `import_ase` do drag & drop. Há gate a correr o escritor do
smoke pelo **leitor real** — é o que impede o escritor dos gates e o do smoke de divergirem.

**37 gates novos** (27 na crate + 10 na costura), **17 mutações**; uma sobreviveu (a opacidade de
camada fixa a 255 passava a suíte inteira, porque todos os outros gates usavam camadas opacas) e
gerou o gate que faltava.

### §13-bis — ⭐ O oráculo que faltava: **18 ficheiros do Aseprite REAL, 18 lidos**

Os gates da crate afirmam *«sabemos ler o que descrevemos»* — o escritor deles é nosso. A pergunta
que sobrava era a fidelidade ao programa real, e ela foi respondida no mesmo dia com ficheiros
escritos pelo **próprio Aseprite**:

* as **12 fixturas de teste do repositório oficial** (`aseprite/aseprite`, `tests/sprites/`), que
  existem justamente para cobrir os cantos: `tags3` (as três direcções), `tags3x123reps`
  (repetições), `link` (cel ligado), `groups3abc` (grupos), `z-order`, `2f-index-3x3` (indexado),
  `3x2tilemap-grayscale` (escala de cinza + tilemap), `bg-index-3`, `1empty3`, `abcd`,
  `file-tests-props`, `point4frames`;
* **2 exemplos MIT** de um gerador de terceiros (`neomura-c-tool-aseprite`), um deles com **4 370
  quadros numa tag só** — que é o caminho em que a folha deixa de caber numa tira e passa a
  quase-quadrada;
* **4 personagens animados** (`player.ase` 12 quadros · `Samurai` 10 · `RainCoat` 8 · `Mage` 7).

**Resultado: 18 lidos, 0 recusados.** ⚠️ E o instrumento é permanente:
`cargo run -p ph2d-aseprite --example ase_info -- <ficheiro|pasta>` corre o **mesmo** `parse` do
produto e imprime tamanho, quadros, tags (com direcção, repetições e a duração — dizendo quando ela
varia dentro da tag) e as notas. ⚠️ Ele conta também os **pixels pintados no 1.º quadro**: é a
diferença entre *«leu»* e *«leu alguma coisa»* — um leitor partido devolve quadros do tamanho certo,
vazios.

**Dois achados que só ficheiros reais dão:**

1. **A duração varia por quadro em ficheiros reais e comuns** — o `example.ase` tem `50..500 ms`, e
   as **três** tags dele variam por dentro. A recusa reaberta (§13) não é teórica: ela dispara no
   primeiro ficheiro de terceiros que se largue.
2. **Personagens reais chegam SEM tags** (os quatro, todos) — o que torna a regra *«um ficheiro sem
   tags recebe uma, com o nome do ficheiro»* o caminho **normal**, e não a excepção que ela parecia.

⛔ Os ficheiros **não entram no repositório**: são binários de terceiros, com licenças que vão de
MIT a nenhuma. Eles vivem em `~/Downloads/ase-para-testar/` na máquina do Enio, e o que fica
versionado é o **instrumento** que os lê.
