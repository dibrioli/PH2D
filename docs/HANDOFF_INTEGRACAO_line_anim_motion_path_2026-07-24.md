# Handoff de integração — `line/anim` · **o MOTION PATH** (ADR-0141)

> DIRETRIZ §1.5.9. A linha está **fechada**. Não integrei nem pushei — isso é ordem
> explícita do Enio, por um agente integrador dedicado.

**Branch:** `line/anim` · **worktree:** `Worktrees/line-anim` · **13 commits**, ~65 arquivos.

---

## §1 — O que entrou, numa frase

**A posição de um objeto deixou de ser dois números soltos e passou a poder ser um
CAMINHO.** Uma track escalar cujo valor é **distância percorrida** sobre uma trajetória
autorada — que é o que faz o graph editor, as weighted tangents, o speed graph e o
roving continuarem a funcionar **sem uma linha de código nova**, e o que torna a
inclinação que o artista vê no gráfico *literalmente* a velocidade do objeto na tela.

O modelo é o do After Effects, que diz com todas as letras que *Separate Dimensions
**precludes** having Spatial Keyframes*: os dois são **modos**, não um modo mais um
flag. Então o `PropKind` **é** o modo, e a troca entre eles é um gesto explícito que
**relata o que perdeu**.

---

## §2 — Foundational tocado (tudo aditivo, salvo onde marcado)

| crate | o quê |
|---|---|
| **`ph2d-arclen`** | **CRATE NOVA**, zero dependências. O motor de comprimento de arco extraído da `ph2d-vec-scene` (`git mv`, história preservada) porque ganhou um segundo consumidor. |
| `ph2d-vec-scene` | `pub mod arclen` virou re-export ⇒ os 5 sítios de lá compilam **verbatim**. ⚠️ Fingerprint do Zig Zag **RE-PINADA** (ver §5). |
| `ph2d-timeline` | `PropKind::Position = 8` · `TargetBinding.path`/`auto_orient` · **`DOC_VERSION` 11 → 12** · módulos novos `path`/`path_convert`/`doc_path`/`apply_path`/`intent_apply_path` · 3 intents novos · dep `libm = "=0.2.16"` |
| `ph2d-editor-core` | `TrackMenuKind` (enum novo) · `ContextMenuKind` **+2 variants** · ⚠️ **`TimelineHitKind::Row` MUDOU DE FORMA** · 4 ids novos · 2 tabelas de menu novas |
| `ph2d-panel-timeline` | `ADDPROP_BUTTONS` 7 → **8** · `event_track_menu.rs` (split) |
| `ph2d-i18n` | chave `panel.timeline.prop.position` |
| shell | `motion_path_overlay` · `motion_path_smoke` · press/move/release no `input_dispatch` · ⚠️ `prop_for_addprop_id` deixou de ter cópia à mão |

### ⚠️ As DUAS mudanças que quebram compilação alheia

1. **`TimelineHitKind::Row { target }` → `Row { target, menu: TrackMenuKind }`.** Quem
   constrói ou casa esse variant precisa do campo. Sítios no repo: `panel-timeline/tracks.rs`
   e `editor-core/interaction/dispatch/tests/timeline.rs` — os dois atualizados.
2. **`ContextMenuKind` ganhou `TimelineTrackAxis` e `TimelineTrackPath`.** Todo `match`
   exaustivo sobre ele quebra. Sítio único: `context_menu_overlay.rs`.

---

## §3 — Ids, consts e variants novos (valores literais)

```
TIMELINE_ADDPROP_POS       = hash_node_id("timeline.addprop.position")
CTX_MENU_TL_AUTO_ORIENT    = hash_node_id("ctx_menu_tl_auto_orient")
CTX_MENU_TL_TO_PATH        = hash_node_id("ctx_menu_tl_to_path")
CTX_MENU_TL_TO_AXES        = hash_node_id("ctx_menu_tl_to_axes")

PropKind::Position         = 8      (apendado; discriminante é WIRE VALUE congelado)
DOC_VERSION                = 12     (era 11 — v11 é RECUSADO no load, nunca migrado)
PROJECT_SCHEMA             = 29     (INTOCADO)
ADDPROP_BUTTONS.len()      = 8      (era 7)
TIMELINE_PATH_TRACK_MENU   = 3 linhas
TIMELINE_AXIS_TRACK_MENU   = 2 linhas

TimelineIntent::AddPathKey { entity, t, at }
TimelineIntent::ToggleAutoOrient { entity }
TimelineIntent::ConvertPositionMode { entity, to_path }
ContextMenuKind::{TimelineTrackAxis, TimelineTrackPath} { target: u64 }
TrackMenuKind::{Plain, Axis, Path}
AutoOrient::{Off, Active, BlockedByRotationTrack}
```

⚠️ **`PROJECT_SCHEMA` não mudou de propósito** — o `TimelineDoc` viaja como blob DENTRO
do `ProjectFile` e carrega a própria versão. É isso que mantém esta linha **fora** de
qualquer disputa de número com outra linha da mesma janela. **Não bumpe.**

⚠️ **`DOC_VERSION` 12 é quebra dura.** Postcard é posicional: um blob v11 não é "curto",
os bytes significam outra coisa a partir do campo novo. Recusado no load, que é a
política deste documento desde o ADR-0133.

---

## §4 — Contratos congelados (§6): **NENHUM tocado**

Conferido por **grep e pelos gates**, não por auto-relato:

| gate | resultado |
|---|---|
| `ph2d-nodegraph::architecture_contract_surface` | ok (3) |
| `ph2d-editor-core::architecture_tool_contract_surface` | ok (4) |
| `ph2d-vector-doc::architecture_vector_contract_surface` | ok (11) |

E `git diff --name-only main...HEAD` não toca `ph2d-nodegraph`, `ph2d-vector-doc`,
`ph2d-vector-traits` nem crate de contrato nenhuma.

`PropKind` **não é** um contrato congelado da §6 — o doc dele já declara o discriminante
como *frozen wire value, append-only*, e `Position = 8` é apêndice.

---

## §5 — ⚠️ O que a integração precisa saber sobre OUTRAS linhas

**(a) `inv_arclen` mudou os BITS que devolve, e é compartilhada.** Trim, Pattern Along
Path, Zig Zag e texto-em-caminho (`line/Vector`) chamam a mesma função. A bisseção de 40
iterações virou **Newton com cerca de bisseção** — 9× mais barato, e **duas respostas
certas para a mesma raiz não caem nos mesmos bits** (a bisseção para por *contagem*, o
Newton por *tolerância*). Medido: **1,3e-10 unidades de mundo**, contra ~1e-2 de um
pixel. A fingerprint `fx_zigzag_tests` foi **re-pinada** seguindo o protocolo escrito no
próprio gate, e ganhou um gate irmão de **magnitude** (`ph2d-arclen`), porque um hash diz
"diferente" e não distingue 1e-10 de tudo.

> **Se a `line/Vector` rebasar por cima disto**, a fingerprint dela já está no valor novo.
> Se ela tiver re-pinado a MESMA constante do lado dela, o merge textual não conflita e o
> número errado passa — **confira `0xd52a_2b63_cd39_0e29` na árvore combinada**.

**(b) Um arch-gate da `line/Vector` foi REESCRITO.**
`shells/desktop/tests/the_patternpath_handles_are_drawn_and_dragged.rs` media **`< 1200`
bytes** entre dois presses no `input_dispatch`. É a **3ª instância** no repo de
[[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]. Duas coisas
foram feitas: o meu bloco mudou-se para **depois** do cluster de Select do vetor (não
pertencia ao meio dele), e a asserção passou a afirmar a **propriedade** — *nada genérico
entre os dois presses* — em vez de uma distância.

**(c) `node_id_collisions` passou a DEDUPLICAR por `(id, label)`.** O `Delete Track`
aparece em duas tabelas de menu com o **mesmo id de propósito** (é a mesma ação; dois ids
seriam duas portas). Conferido que o gate **ainda pega uma colisão de verdade** — dois
labels diferentes com o mesmo hash continuam vermelhos.

---

## §6 — O que só o `ship.sh` pega

- **`cargo machete`** — a `ph2d-arclen` é dependida por `ph2d-vec-scene` e
  `ph2d-timeline`; o `libm` novo é usado (`atan2f` no `apply_path`). Nenhuma dep órfã
  esperada, mas o machete não roda aqui.
- **`cargo deny` / `audit`** — o `libm = "=0.2.16"` já está no `Cargo.lock` do repo (mesmo
  pin de `ph2d-ecs`/`physics`/`flip`/`wet-paint`); a `ph2d-arclen` tem **zero** deps. O
  `Cargo.lock` ganhou **uma linha** (a crate nova).
- **`typos`** — os docs desta linha são em pt-BR com termos técnicos em inglês; não rodei.
- **A matriz 3-OS + o `physics-ecs-c9`** — ⚠️ **relevante**: o auto-orient escreve
  `Transform.rotation`, e esse hash compara os três OSes bit a bit. Por isso o ângulo sai
  de **`libm::atan2f`** e não de `f32::atan2`, cuja última casa a std não promete igual em
  toda plataforma. **Nenhum corpo físico usa Position hoje**, então o c9 não deve mudar —
  mas é o número a olhar se ele mudar.

---

## §7 — O gate de fechamento, rodado

| | |
|---|---|
| `cargo fmt --all -- --check` | limpo |
| `cargo clippy --workspace --all-targets` | **0** warnings, 0 errors |
| `cargo test --workspace` (debug) | verde |
| `cargo test --workspace --release` | verde |
| `architecture_workspace_file_loc_cap` · `architecture_panel_loc_cap` · shell `file_loc_caps` | verdes |
| `arch_safe_clamp_only` | verde |
| perf `#[ignore]` em release | **20,3 µs/frame a 100 entidades = 0,122 %** (orçamento 0,2 %) |

**Debug E release**, deliberadamente: a `line/FLIP` pagou para aprender que rodar só com
`--release` **esconde pânico** (o `voronoi.rs` panicava em debug por 3 commits).

⚠️ **O gate de LOC nasceu VERMELHO nesta corrida e foi consertado**, em dois arquivos que
as últimas linhas do fechamento estouraram: `interaction/types.rs` (708 → 693, o enum de
menu foi para o irmão `types_menu.rs` — *o que está sob o cursor* e *que menu isso merece*
são perguntas diferentes) e `intent_apply.rs` (703 → 700, o snap desceu para dentro do
helper). Vale o registo porque é a 2ª vez nesta linha que o teto morde no último passo: o
gate de LOC **não** roda num `cargo test -p` filtrado, e por isso ele entra no fechamento
por nome, nunca de carona.

**Gates novos: ~50. Mutações: 24, e 24 sangram** — duas delas só depois de eu construir a
camada que faltava (a busca binária diluída no frame; a ordem do press).

---

## §8 — O que SMOKAR (nesta ordem)

```
env PH2D_PATH_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

1. Abra a timeline (`L`). A trajetória aparece: **fio âmbar fraco** (a forma) coberto de
   **losangos** (o tempo).
2. ⚠️ **Olhe o ESPAÇAMENTO dos losangos.** Ele **é** a velocidade — juntos nas pontas
   (o ease), esparramados no meio. Medido: **0,0016 contra 1,5521** (971×). *É a coisa
   inteira que a figura existe para dizer.*
3. **Play.** O objeto segue o fio e passa por cada losango no quadro que ele marca.
4. O gráfico mostra **uma track só**, e o valor dela é *distância*. A inclinação que você
   vê ali **é** a velocidade na tela.
5. Clique no outro objeto: a trajetória **some**. Ela é do que está na mão.
6. **Arraste um quadrado** (uma âncora): a curva responde, os losangos se reacomodam, o
   objeto faz o caminho novo **no mesmo compasso**. Um Ctrl+Z desfaz o arrasto inteiro.
7. Com o objeto selecionado, **K** acrescenta uma âncora onde ele está.
8. Botão direito na **label** da row de Position → *Convert to Separate Axes*. E numa row
   de Translation X → *Convert to Motion Path*.

```
env PH2D_PATH_SMOKE=2 cargo run -p ph2d-host-desktop --release
```

1. **Play.** A seta **laranja** encara para onde vai (auto-orient), varrendo **118,8°**
   sem uma única key de rotação.
2. A seta **azul** tem a mesma trajetória e o mesmo pedido — e uma track de **Rotation**.
   Ela **não gira**: o auto-orient está **RECUSADO**, e o prólogo imprime
   `laranja Active | azul BlockedByRotationTrack`.
3. Apague a track de Rotation dela (botão direito na label → *Delete Track*): ela passa a
   girar. **O pedido sobreviveu à recusa.**
4. Botão direito na label da trajetória → *Auto-Orient* desliga a laranja.

---

## §9 — Aberto, nomeado, com o gatilho

- **A conversão não mostra o RELATÓRIO na tela.** `ConversionReport` conta os instantes
  desemparelhados e os eases perdidos, e o gesto do menu **descarta** o relatório: o
  efeito já está na tela (a trajetória aparece, ou os dois eixos aparecem), mas o *custo*
  fica invisível. **Gatilho:** o primeiro artista que perder um ease e não souber por quê.
  O canal natural é um toast — a infra existe (`ph2d-editor-core::progress` tem a coluna).
- **As alças de TANGENTE não se arrastam** — as âncoras sim. Hoje a forma da curva vem do
  Auto Bezier e do lugar das âncoras. **Gatilho:** o primeiro smoke que peça uma curva que
  o Auto Bezier não faz. O `PathAnchor.auto` já existe justamente para o arrasto de alça o
  limpar.
- **Andar para trás não vira o objeto**, e é **decisão** (gate pinado): o ângulo vem da
  geometria da curva, não do vetor velocidade — é isso que remove o bug publicado do AE
  (*"flips when stopping motion"*). Um flip automático de 180° no meio de uma track é uma
  descontinuidade que o artista não controla.
- **`intent_apply.rs` está em 700/700.** A próxima linha obriga o split.
- **Nada assa uma trajetória num corpo físico.** O bake da física escreve
  `TranslationX/Y`; um corpo com Position não é assável hoje. **Gatilho:** alguém querer
  assar sobre um objeto em modo Path — a resposta provável é o bake usar
  `path_to_separate` primeiro.

---

## §10 — Docs desta linha

- [`docs/architecture/decisions/0141-*.md`](architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md) — o ADR, com a Fatia 0 e a lei VERIFICADA no produto
- [`docs/Timeline/05_pesquisa_motion_path.md`](Timeline/05_pesquisa_motion_path.md) — a pesquisa de campo (AE · Harmony · Blender · Spine · Rive)
- [`docs/Timeline/06_plano_motion_path.md`](Timeline/06_plano_motion_path.md) — o plano por fatias, com as seis fechadas
