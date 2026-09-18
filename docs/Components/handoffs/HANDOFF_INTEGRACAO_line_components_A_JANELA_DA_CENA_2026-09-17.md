# HANDOFF — `line/components` · A JANELA DA CENA · 2026-09-17

> **O que o dono ganha:** com a timeline (ou o Motion) aberta, o clique deixa de apontar para outro
> sítio do mundo. Medido: com o centro partido a `55 %` e a superfície da foto de 17/09, o **mesmo
> pixel** resolvia para um ponto **`3,50 m` ao lado no Y e `5,96 m` no X** — numa vista de `10 m`
> de altura, `35 %` e `60 %` do que se vê.
>
> ⚠️ **Leia a §4 antes do diff:** seis coisas que uma leitura rápida entende ao contrário.

## §1 — Os contadores, como DELTA

| contador | delta | porquê |
|---|---|---|
| `PROJECT_SCHEMA` | **0** | nenhum componente, nenhum byte gravado |
| registos do `ph2d-ecs` e os dois espelhos | **0** | — |
| `LIVE_SECTIONS` / `any_live_section` | **0** | — |
| contrato congelado (§6) | **0** | nenhuma assinatura pública de crate mudou |
| ADR | **0** | a lei já existia por escrito desde 2026-07-25; o que faltava era um GATE |
| ficheiros da shell | **+1** (`scene_mapping.rs`) · **−3 parâmetros** em fases do quadro | — |

## §2 — A lei, e as TRÊS vezes que ela foi paga antes deste gate

O doc do `ph2d_app_motion::field_gizmo::scene_window_wh` escreve-a desde 2026-07-25: *«todo
mapeamento mundo↔tela do chrome da cena TEM de usar isto»*. Sob um split do centro a cena desenha
num sub-rectângulo `[0, 0, w, h·t]` e **a projecção MUDA** — não é um recorte.

| data | quem foi posto na porta | quem ficou de fora |
|---|---|---|
| 2026-07-25 | a grade e o gizmo | tudo o resto |
| 2026-08-25 | o **pan** (a cena andava `t` vezes o que o cursor andava) | tudo o resto |
| 2026-09-17 (manhã, wave do HUD) | o `vec_world_at` (o botão, `~340 px` ao lado) | **50 chamadas** |
| **esta wave** | **~90 sítios** — e um **CENSO** impede a quinta | — |

⛔⛔ *Três curas, três vezes UM consumidor.* **Um gate é a única coisa que muda isso.**

## §3 — O que a wave entregou

| peça | onde |
|---|---|
| a **porta única** (duas formas, e a razão de serem duas) | `shells/desktop/src/scene_mapping.rs` |
| as duas portas antigas a **DELEGAR** | `connector_gesture.rs` · `field_gizmo_host.rs` |
| a conversão | **40** ligações `let … = gfx.surface.size()` + **8** chamadas inline + **33** da família do dedo + **7** nas fases do quadro |
| o **censo** (4 metades) | `shells/desktop/tests/it/todo_aponte_passa_pela_janela_da_cena.rs` |
| a premissa REESCRITA do gate do HUD | `o_cursor_e_mapeado_pela_banda_da_cena.rs` |

**Mutação: 7 de 7** — [`mutacao_janela_da_cena.sh`](../ferramentas/mutacao_janela_da_cena.sh).
Portão: `15 090` testes verdes · censos da árvore combinada `90/90` · clippy `-D warnings` a zero.

## §4 — ⚠️ Seis coisas que uma leitura rápida do diff entende ao contrário

1. **A porta tem DUAS formas e isso NÃO é a duplicação de volta.** `AppGfx::scene_window(&self)`
   pede o `AppGfx` inteiro; metade dos gestos do canvas corre dentro de um
   `if let Some(hero) = gfx.hero_screen.as_mut()`, onde o compilador **recusa** esse empréstimo.
   ⭐⭐ **Foi isto, e não desleixo, que fez a lei ser violada em ~90 sítios:** *a porta pedia mais
   do que o sítio tinha para dar, e o caminho que COMPILAVA era o errado* (`gfx.surface.size()`,
   que toca um campo). A segunda forma (`scene_mapping::janela(split, size)`) pede campos
   **disjuntos** — e as duas chamam a mesma função.
2. **A conversão é o SEGUNDO ARGUMENTO e mais nada.** A banda começa em `[0, 0, …]`, logo o ponto
   de ecrã não precisa de translação: só o **tamanho** passado difere. Sem essa propriedade isto
   seria uma reescrita de coordenadas em 90 sítios.
3. **Os DOIS eixos erravam, não só o `y`.** O `screen_to_world` deriva `aspect = w/h` e
   `half_w = half_h·aspect`; com o `h` a encolher o `aspect` **cresce**. No centro horizontal o `x`
   coincide por simetria (`nx = 0`) — e é por isso que a leitura *«está deslocado para baixo»*
   descreve o sintoma e não a conta. Há gate sobre as duas metades.
4. **Fora do split é BYTE-IDÊNTICO**, e é isso que torna a troca de ~90 sítios segura: o caminho de
   omissão do app não muda um bit. ⚠️ A prova dessa identidade **já existia** noutra crate
   (`field_gizmo_tests`) e a porta **herda-a por DELEGAÇÃO** — há gate a proibir que ela refaça a
   conta, porque uma quarta cópia é como as três primeiras nasceram.
5. **Três parâmetros de fases do quadro DESAPARECERAM**, e isso é o resultado, não um refactor à
   parte: quando a fase passou a derivar a banda do `gfx`, o compilador disse que ela já não
   precisava da janela do quadro. ⭐ O `fase_hero_document_verbs` deixou de a receber de todo.
6. **O gate do HUD reprovou, e estava a funcionar.** Ele lia o fonte do `connector_gesture.rs`, onde
   a conta vivia; a conta mudou de casa. Foi **reescrito com a morte da premissa visível no diff** e
   ficou mais forte — ganhou a metade *«as duas portas antigas ainda DELEGAM»*.

## §5 — ⛔ As três premissas minhas que a medição derrubou

1. *«são 50 sítios»* — o `screen_to_world` são 70 (duas das 72 que o censo via eram **comentários**),
   e a família inteira do dedo é **~90**: o `FlipFrame`, o `CanvasCtx` da física e o `PickWorld`
   carregam o par `(camera, janela)` para uma inversão que acontece noutra crate.
2. *«há 13 casos MISTOS para discutir»* — não há nenhum: todo uso «outro» era
   `GRAB_PX·height_world/h`, `stroke_hit_r(&camera, win)` ou `px_per_world = h/height_world`, que
   são a **mesma** conversão pixel↔mundo e erram pelo mesmo mecanismo. *A população é uniforme.*
3. *«a régua está pronta»* — ela acusou **os dois sítios que documentam a lei** (lia comentários) e
   **os dois que estavam certos** (não sabia ler `let (sw, sh) = …`). As duas cegueiras têm agora
   prova de mutação própria.

## §6 — ⏳ ABERTO, com o número

A wave fecha **por onde passa o DEDO** (toda inversão `screen_to_world`). Fica de fora, **medido e
nomeado**, o outro lado do par — a **projecção** e as tolerâncias:

* `render_loop/fase_selection_highlight.rs` (5) · `fase_sim_extract.rs` · `fase_motion_bridge.rs` ·
  `fase_vector_overlays.rs` (2) — desenham com `surface.size()` ao lado da câmera;
* `bone_undo_probe.rs`, `vec_trim.rs` e os `stroke_hit_r` que ainda recebem a janela por outro
  caminho.
* ⛔ **O `fase_game_camera` fica FORA de propósito**: `camera_2d::aspect_of(surface.size())` é a
  câmera do **JOGO** (o `GameCamera` do TOP-20 #7), outro assunto — a régua larga apanha-o e seria
  a acusação errada.

O instrumento que os enumera é o próprio gate: alargue o `resolve` a `world_to_screen` e ele
imprime a lista.

## §7 — Como smokar

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && cargo run -p ph2d-host-desktop --profile smoke
```

Abra a **timeline** (o centro parte-se) e clique num objecto do canvas: ele tem de ser escolhido
onde o dedo está. Antes desta wave, com o centro partido, o clique caía metros ao lado.
