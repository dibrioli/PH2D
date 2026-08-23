# 21 — Auditoria da §11 Animation (2026-08-23)

> Pedido do Enio: *«às vezes preciso clicar mais de uma vez para checar Playing. Corrija isso e
> faça auditoria completa do sistema de animação da sprite.»*
>
> ⚠️ **Ao contrário da [auditoria 20](20_auditoria_do_inspector_2026-08-21.md), esta foi APLICADA.**
> A §11 tinha um dia de vida e o report do Enio era a ponta de uma família — o que se segue é o que
> a medição encontrou e o que ficou curado, com o gate de cada cura e a mutação que a prova.
>
> **Leia primeiro o §0 (placar) e o §5 (⛔ recusas medidas).** *Um ❌ «recusado com motivo» e um ❌
> «ninguém fez» leem igual numa lista.*

## §0 — Placar

| # | Achado | Estado |
|---|---|---|
| F1 | **A caixa «Playing» tinha DUAS fontes de verdade** — pintava do snapshot, decidia do store | ✅ curado |
| F2 | **Ligar uma animação que já ACABOU era um gesto morto** — ela parava-se outra vez no mesmo tique | ✅ curado |
| F3 | **«Rewind» não movia a imagem** — repunha contadores que ninguém vê | ✅ curado |
| F4 | **Escolher uma animação SOBREPOSTA começava-a a meio** | ✅ curado |
| F5 | **Escolher outra depois de uma se esgotar deixava a sprite muda** — a promessa da própria cena de smoke | ✅ curado |
| F6 | **A §11 não tinha UM gate que carregasse num pixel** — é por isso que o F1 shipou | ✅ curado (10 gates novos) |
| F7 | Dois campos do snapshot **calculados e nunca lidos**, um deles clonando a biblioteca de **toda** a seleção por quadro | ✅ curado |
| F8 | A seleção múltipla **não se dizia**, e as edições da §11 não se espalham | ✅ curado |
| F9 | `ph2d_ecs::animator_state` **exportada sem consumidor**, com o painel a reimplementar a lei sem oráculo | ✅ curado (gate de paridade) |
| F10 | Um comentário no `render_loop/mod.rs` **afirmava o contrário do código** | ✅ curado |
| F11 | **Um quadro de reprodução mexe num `SimComponent` registado** ⇒ com a animação a tocar, um quadro com input regista um passo de undo | ⚠️ **medido e RECORDADO** — família pré-existente, decisão do Enio (§4) |
| F12 | **A barra de frames era só DESENHO** — dois retângulos sem id, sem entrada no store: bonita, informativa e morta sob o rato | ✅ curado (pedido do Enio) — §2-bis |

## §1 — O defeito reportado, e por que ele era «às vezes»

**Mecanismo.** A §11 *pintava* a caixa a partir do snapshot
([`sections/anim.rs`](../../crates/ph2d-panel-inspector/src/sections/anim.rs) —
`.value(info.playing)`, que é o mundo) e *decidia* a partir do valor guardado no `WidgetStore`
([`event_anim.rs`](../../crates/ph2d-panel-inspector/src/event_anim.rs) — `store().checkbox(id)`).
As duas concordam **enquanto só o painel escrever**.

E o `playing` é justamente o campo que o **motor** escreve por conta própria: uma animação que não
repete põe-se a `false` sozinha ao chegar ao fim — sem que a entidade nem a linha aberta mudem, que
eram as duas arestas em que a semente do `sync` corria. A partir daí:

| o que o artista vê | o que o store lembra | o que o clique mandava |
|---|---|---|
| caixa **vazia** (o snapshot diz `false`) | `Checked` | `Playing(false)` — a uma cena já parada |

⇒ o primeiro clique não fazia nada, o segundo ligava. **«Às vezes»** porque só acontecia depois de
o motor mexer no facto: numa sprite que só o painel tocou, os dois valores nunca divergiam.

**Cura.** O clique afirma **o contrário do que estava no ecrã** (`!info.playing`), que é o que um
toggle promete. E o store passa a espelhar o snapshot **todo o quadro** — ele já não decide nada,
só publica o estado para a árvore de acessibilidade, e um valor que ninguém usa para decidir tem de
acompanhar quem decide.

**Gate:** `seam_anim::the_playing_box_asks_the_scene_not_its_own_memory` (três casos; o terceiro é o
defeito). **Vermelho antes da cura, verde depois** — a corrida red-first está no handoff.

## §2 — A família por baixo dele: o transporte inteiro estava meio-ligado

O report era a porta. Por baixo dela, **três gestos do tocador não faziam o que prometem**, e os
três só aparecem depois de uma animação de uma volta chegar ao fim — o mesmo estado que expôs o F1.

### F2 · Ligar uma animação terminada era um gesto MORTO

Com `repeat: Some(1)` gasto, pôr `playing = true` deixava a imagem na ponta do intervalo **com o
contador cheio**: o primeiro passo de `advance` via `at_end`, `will_continue == false`, e fechava o
ciclo outra vez. A caixa ficava marcada por um quadro e desmarcava-se sozinha.

⇒ **«Pausado» e «terminado» leem-se igual no `playing == false` e não são a mesma coisa.** A
distinção é [`SpriteAnimator::is_finished`](../../crates/ph2d-ecs/src/sprite_anim.rs) — *ligar uma
que acabou é pedir para a rever; retomar uma pausa continua de onde estava*.

### F3 · «Rewind» repunha contadores e não a IMAGEM

`SpriteAnimator::rewind` zera `elapsed_ticks`, `repeat_count` e o ping-pong — **nada disso se vê**.
O `advance` só reposiciona um frame que caia **fora** do intervalo, então a sprite ficava na célula
onde tinha parado. O botão não fazia nada; e com um `repeat` finito não fazia nada **duas** vezes.

⇒ **Rebobinar é repor o ciclo E pôr a imagem no princípio**, e a ponta certa segue a direção
efetiva ([`entry_frame`](../../crates/ph2d-ecs/src/sprite_anim.rs)).

### F4 · Escolher uma animação sobreposta começava-a a meio

A tese do modelo é que as animações **partilham o pool de células** (`idle` 0-3 e `walk` 0-7 na cena
de smoke). O `advance` só reposiciona o que cai fora do intervalo ⇒ sair de uma `walk` na célula 2
para a `idle` **não** reposicionava nada. *A propriedade que faz o modelo valer a pena era a que
partia esta lei.*

### F5 · E escolher outra depois de uma se esgotar deixava a sprite muda

A cena de smoke diz, por escrito: *«Clicar em `attack` → toca uma vez e fica na última célula.
Clicar em `walk` volta ao ciclo.»* Não voltava: o `attack` tinha posto `playing = false`, e
`SetCurrent` não lhe tocava. O artista ia procurar a caixa de `Playing` — e caía no F1.

⇒ **Uma lei, duas portas:** *a reprodução que se ESGOTOU volta a tocar quando alguém lhe toca, e
escolher outra animação é tocar-lhe.* Uma **pausa explícita** não é tocada por isto (o `is_finished`
é falso a meio de um ciclo), então folhear a lista em silêncio continua a ser possível.

**Gates:** `inspector_anim_transport_tests::{rewinding_puts_the_picture_back_at_the_start,
choosing_an_overlapping_animation_still_starts_it_at_its_own_beginning,
turning_playing_back_on_replays_an_animation_that_had_finished,
choosing_another_animation_resumes_one_that_had_run_itself_out_but_not_a_pause}`.

## §2-bis — F12: a barra de frames arrasta (pedido do Enio)

*«permita arrastar manualmente o slider de frames»* — e ela não era um slider: eram **dois
retângulos pintados à mão**, sem id, sem entrada no store. Bonita, informativa, e morta sob o rato.

**Cura:** um `Slider` registado. O despachante dá-lhe o salto-ao-clique (`pointer_down`) e o
arrasto (`pointer_move`) **sem uma linha de máquina nova** — é a mesma porta da Opacidade e do
Emissive. A trilha subiu de 6 para 10 px, e o número não é gosto: o retângulo de acerto **é** a
trilha (o despachante deriva o valor de `rect.x`/`rect.w`), então a altura dela é o alvo do dedo.

⚠️ **A régua teve de mudar de PROGRESSO para POSIÇÃO.** A barra media `(passo+1) / total` — com
essa régua o primeiro frame já se desenha com uma fatia preenchida e o polegar não pousa sobre a
célula. *Uma barra que só informa pode medir «quanto já passou»; uma que se agarra tem de medir
«onde está».*

⚠️ **Agarrar a barra PAUSA a reprodução**, e a pausa é o verbo: enquanto a reprodução corre o tique
também escreve o `Sprite::frame`, e o dedo e o relógio disputariam o mesmo campo — a imagem
piscaria entre a célula arrastada e a que o relógio acabou de pôr. *Quem pega no volante conduz*, e
a caixa `Playing` di-lo.

### ⭐ A mutação que SOBREVIVEU, e o que ela encontrou

A primeira versão do gate afirmava que arrastar até à ponta esquerda apanhava a troca
posição↔progresso. **Não apanhava**: trocar a régua **do pintor** passou a suíte inteira. O caminho
do clique (`x → 0..1 → célula`) não passa pelo pintor, então mudar só o desenho não move asserção
nenhuma — o polegar deixaria de pousar em cima do frame e nada diria nada.

⇒ A causa era mais funda: **a régua existia em TRÊS cópias** — o pintor, o `sync` e o despacho.
Hoje é uma lei em dois sentidos no modelo (`InspectorAnimInfo::scrub_position` ↔ `scrub_cell`), com
gate de ida-e-volta sobre **cada** célula do intervalo (`the_scrub_position_and_the_cell_are_inverses`).

*Um sobrevivente não é um gate a mais que falta: é o desenho a dizer onde está a duplicação.*

## §3 — O que deixou isto shipar, e o que passou a impedi-lo

**F6 — a §11 tinha gates dos dois lados e nenhum no MEIO.** A lei pura tinha 20 no `ph2d-ecs`; o
commit tinha 13 na shell. **Zero** carregavam num pixel. O F1 vive exatamente entre os dois: o
painel pinta certo, o commit escreve certo, e o **despacho** lê a fonte errada.

⚠️ E o gate que parecia cobrir isto **não cobria**: o `every_id_the_inspector_paints_can_actually_be_clicked`
afirma que todo id pintado está **registado** no store — o *segundo* dos três sítios que o próprio
doc dele nomeia (*pintar · registar · despachar*). O F1 é o **terceiro**, e um id perfeitamente
registado que despacha a partir da fonte errada passa por ele sem um arranhão.

⇒ [`seam_anim.rs`](../../crates/ph2d-panel-inspector/tests/seam_anim.rs), irmão do `seam_player.rs`,
com a mesma disciplina: todo clique passa pelo `click_at` **real**. Inclui
`every_edit_the_model_declares_is_reachable_by_a_gesture` — as 18 variantes de `AnimFieldEdit`, com
o `match` exaustivo a garantir que uma variante nova **não compila** até alguém a amostrar.

**F9 — e uma lei que já vivia duas vezes, sem oráculo.** `ph2d_ecs::animator_state` estava
exportada, com gate próprio, e **nenhum consumidor**; o painel reimplementava-a em
`InspectorAnimInfo::current_dangling`. A duplicação é arquitetural (o painel não vê o motor), como o
`ph2d_ecs_dir_label` a espelhar `AnimDirection::label` — e, como ali, quem a prende é um gate da
**shell**: `the_panel_and_the_engine_agree_on_a_dangling_playback`.

**F7 — dois campos do snapshot calculados e nunca lidos.** `mixed` e `library_present`. O primeiro
custava, **por quadro**, um clone da biblioteca da primária mais um de cada entidade selecionada —
para nada. Saíram os dois.

**F8 — e o que a seleção múltipla precisava de dizer não era «elas discordam».** As edições da §11
**não** se espalham (o índice que elas carregam só significa alguma coisa na biblioteca da ativa),
então marcar cinco goblins e renomear uma animação muda **um**, em silêncio. A seção passa a dizê-lo
antes de oferecer controlo nenhum. Gate:
`a_multiple_selection_says_so_before_offering_any_control` (o oráculo é a **geometria** — um aviso
é texto, e texto que despacha mente).

**F10 — um comentário que afirmava o contrário do código.** O `render_loop/mod.rs` dizia *«nunca um
passo grande — um salto atravessaria o fim de um ciclo sem o fechar»* sobre um `tick` que faz
exatamente um passo grande. A afirmação já tinha sido **refutada por mutação** quando o tick foi
simplificado; o comentário ficou. *Comentário velho mente.*

## §4 — ⚠️ F11: o que a auditoria encontrou e NÃO curou

**Um quadro de reprodução mexe num componente registado.** O `SpriteAnimator` guarda
`elapsed_ticks`/`repeat_count`/`pingpong_reverse` e está no `ComponentRegistry` ⇒ ele entra no
`ProjectState`, que é a unidade do undo. O `post_frame_undo` desiste quando não houve input
(`!had_input`), mas **num quadro com input e com a animação a tocar, o diff é não-vazio** e um passo
é registado — cujo conteúdo é só o relógio.

⚠️ **Isto NÃO é um defeito da §11: é uma propriedade do undo do app.** A ponte de física escreve o
`Transform` (também registado) a cada passo enquanto o mundo simula, e produz a mesma coisa. A
família é *«enquanto alguma coisa se move sozinha, um quadro com input regista um passo»*.

**As três saídas, e por que nenhuma se aplicou aqui:**

| saída | por que não |
|---|---|
| suprimir o undo enquanto toca (como o `ui_state_live` faz para uma preview de 150 ms) | uma animação em ciclo **nunca** pára ⇒ o undo deixaria de registar o que o artista faz |
| tirar o relógio do componente registado (a lei da linha de física: *config, nunca estado vivo de solver*) | remove o passo por-quadro mas **não** o passo por-avanço-de-frame (o `Sprite::frame` é documento e tem de ficar registado), e move o `PROJECT_SCHEMA` |
| ensinar o undo a ignorar componentes de «preview» | é um conceito que o app não tem, e vale para a física antes de valer para aqui |

⇒ ✅ **AUTORIZADO pelo Enio em 2026-08-23:** *«precisamos corrigir o CtrlZ para ambas»* — **ambas**
= a animação **e** a física, que é a outra metade da família. ⚠️ Isso põe a cura na terceira saída
da tabela (o conceito que o app não tem: *estado de PREVIEW contra estado de DOCUMENTO*), e não
numa correção local à §11 — a §11 sozinha não pode legislar para o undo do app inteiro.
⏳ **Na fila, não iniciado.** Quem pegar começa por medir se `Transform` sob física e
`SpriteAnimator` sob reprodução são o mesmo caso ou dois: o primeiro é pose de documento com um
escritor a mais, o segundo é relógio que nunca foi documento.

## §5 — ⛔ Recusas MEDIDAS

| # | O que foi tentado / proposto | Por que foi recusado |
|---|---|---|
| R1 | Dar `set_number_range` aos campos da biblioteca (`frame_ms`, `hold`, `delay`, `repeat`), como o `speed` tem | ⛔ **O alcance torna o arrasto PROPORCIONAL ao intervalo** ([`store_core.rs`](../../crates/ph2d-editor-core/src/interaction/state/store_core.rs)). O `speed` cabe (±100); um `frame_ms` de `[1, 60 000]` daria ~600 ms de salto por pixel — o knob ficaria inutilizável. O número mentiroso durante o arrasto **corrige-se sozinho ao soltar** (o `sync` re-lê a biblioteca todo o quadro). *Curar um teto e estourar a ergonomia não é curar.* |
| R2 | Dar alcance a `from`/`to` limitado à grelha de hoje | ⛔ **Removeria um fluxo real**: autorar o intervalo e **depois** aumentar a folha. O «out of grid» a vermelho na linha da lista é a afordância certa — ele informa em vez de impedir. |
| R3 | Clicar numa linha da lista **tocar sempre** | ⛔ Apagaria a pausa explícita: quem desmarcou `Playing` deixaria de poder folhear a lista em silêncio. A regra que shipou é mais estreita e statable numa linha (§2, F5), e o gate afirma as **duas** metades. |
| R4 | Fazer o `SetCurrent` limpar o `current` quando a animação é apagada | ⛔ O estado «pendurado» é **mostrado** (texto a vermelho + «out of grid» na linha), e ele é honesto: a §12 tomou a mesma decisão para uma montagem cuja âncora sumiu. Limpar em silêncio esconderia o que o artista precisa de ver. |
| R5 | Dicas de hover na §11 | ⛔ **Nenhuma** outra seção do Inspector as tem (só a §14 Platform Player); exigi-las aqui seria esta seção a legislar para o painel inteiro. |

## §6 — Onde ler

- A lei pura: [`sprite_anim.rs`](../../crates/ph2d-ecs/src/sprite_anim.rs) ·
  gates em `sprite_anim_tests.rs` (22)
- O relógio: [`sprite_anim_tick.rs`](../../shells/desktop/src/render_loop/sprite_anim_tick.rs)
- O snapshot + o commit: [`inspector_anim.rs`](../../shells/desktop/src/render_loop/inspector_anim.rs) ·
  gates em `inspector_anim_tests.rs` (autoria) e `inspector_anim_transport_tests.rs` (transporte)
- A costura de clique: [`seam_anim.rs`](../../crates/ph2d-panel-inspector/tests/seam_anim.rs) (10)
- A spec: [`08_animation_inline.md`](08_animation_inline.md)
