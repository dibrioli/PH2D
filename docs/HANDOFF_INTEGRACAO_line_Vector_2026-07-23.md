# HANDOFF DE INTEGRAÇÃO — `line/Vector` · Texto em caminho + Pattern along path + Picker

> **Regra H.** Entregável de fechamento. A linha está **FECHADA**; NÃO integra nem faz ship sozinha.
> Aguarda ordem EXPLÍCITA do Enio, via agente integrador dedicado.
>
> **SUPERSEDE** [`HANDOFF_INTEGRACAO_line_Vector_textpath_2026-07-22.md`](HANDOFF_INTEGRACAO_line_Vector_textpath_2026-07-22.md)
> (aquele cobria só o Texto em caminho, HEAD `94dcc8c46`). A linha ganhou depois o **Pattern along
> path** (W1–W4) e o **Picker de caminho-guia** — este handoff é o completo e atual.

## 1. Branch / HEAD / base

- **Branch:** `line/Vector`
- **HEAD:** `845133103`
- **Base (merge-base com `main`):** `13a04c7aa` (2026-07-21 23:36 — já contém a física e o offset vivo)
- **25 commits.** Árvore limpa. Três blocos:
  - **Texto em caminho** W0–W5 (`762279f71` … `b9ebf651e`) — 13 commits, o do handoff superseded.
  - **Pattern along path** W1–W4 + End/Offset/Slide + fixes (`ba72ac6ce` … `5ff80271a`) — 11 commits.
  - **Picker de caminho-guia** (`845133103`) — 1 commit (esta sessão).

## 2. O que a linha entrega

Três features de vetor, **rígidas** (glyphs/motivos transladam+giram, não deformam — o oposto do
Envelope), todas **não-destrutivas** (a fonte que o Node edita nunca é tocada; o que se vê é derivado
e re-cozido por frame):

1. **Texto em caminho** (`<textPath>` do SVG / *Type on a Path* do Illustrator). Motor arco→afim por
   glyph + vínculo (componente ECS `VecTextPath`) + seção de painel + alça de canvas.
   Plano: [`docs/Vector Module/22_plano_texto_em_caminho.md`](Vector%20Module/22_plano_texto_em_caminho.md).
   Detalhe/racional: o handoff superseded (§2/§3).
   - **Bônus da W0** (bug VIVO independente): o re-cook de texto apagava a pilha de efeitos em
     silêncio ao editar uma letra — curado pela porta única `VecPath::replace_cooked`.
2. **Pattern along path** (o *Pattern Brush* rígido do Illustrator / item #11 da pesquisa `20_*`). Um
   **motivo** (uma forma qualquer) se repete ao longo de um **guia**, cada cópia girada para a
   tangente. **Sem refit** — o afim rígido comuta com Bézier (o que o separa do Envelope, que precisa
   de `fit_to_bezpath`). Controles: **Spacing** (a CONTAGEM de cópias é AUTOMÁTICA, função do
   espaçamento), **Start/End** (o trecho tilado, por slider OU por duas fichas âmbar na curva),
   **Slide** (desliza o trecho inteiro), **Offset** (perpendicular), **Side** (o lado).
   Plano: [`docs/Vector Module/23_plano_pattern_along_path.md`](Vector%20Module/23_plano_pattern_along_path.md).
3. **Picker de caminho-guia** (Enio 2026-07-23: *"um botão Picker onde o usuário primeiro seleciona a
   shape, depois aperta o botão e seleciona o path. Dessa forma é mais correto"*). O gesto de duas
   mãos partilhado pelo Pattern E pelo Texto: **seleciona a forma → aperta "Pick Path" → clica o
   caminho.** A fonte é capturada no arm, o guia é apontado pelo clique — sem a adivinhação por bbox
   do `link_candidate` (a raiz do *"escolhendo a si mesmo"*). A auto-ligação por DOIS selecionados
   **fica** (as duas portas coexistem, aparecem em seleções diferentes: 1 vs 2).

## 3. Foundational tocado, e por quê

| Crate | O quê | Isolamento |
|---|---|---|
| `ph2d-vec-scene` | módulos NOVOS `recook.rs`, `arc_path.rs`, `text_path.rs`, `pattern_path.rs`; `GlyphFrame` (afim por cópia); `+closest_arc` | arquivos próprios; `fx_zigzag` delega ao `ArcPath` (byte-idêntico, fingerprint pinado) |
| `ph2d-ecs` | componentes NOVOS `VecTextPath` (`vec_text_path.rs`) **e** `VecPatternPath` (`vec_pattern_path.rs`) | arquivos próprios; registro append-only. **⚠️ contador ver §5** |
| `ph2d-editor-core` | arquivos NOVOS `ids/chrome/vector_textpath.rs`, `vector_patternpath.rs`; **2** entradas em `VECTOR_SECTIONS` | ⚠️ `VECTOR_SECTIONS` é lista compartilhada — as 2 entradas foram ao **FIM** (só ADICIONAR) |
| `ph2d-i18n` | **2** chaves: `panel.vector.section.textpath`, `.patternpath` | tabela compartilhada, só ADICIONAR |
| `ph2d-panel-vector` | arquivos NOVOS `paint_textpath.rs`/`state_textpath.rs`, `paint_patternpath.rs`/`state_patternpath.rs`/`populate_patternpath.rs`; `+track_slider_event` em `event.rs` | seções novas, isoladas. ⚠️ `state.rs` e `lib.rs` no teto de LOC (ver §5) |
| `ph2d-vec-render` | arquivo NOVO `text_handle.rs` (a ficha grande/colorida) + re-export | isolado |
| `ph2d-render`, `ph2d-script` | **só o contador de componentes** (`35→36`) | ver §5 |
| `shells/desktop` | arquivos NOVOS `vec_text_ride.rs`, `pattern_live.rs`, `vec_pick.rs`, `vec_guide.rs`, `pattern_path_smoke.rs`, `text_path_*_smoke.rs`; edições em `render_loop/mod.rs`, `input_dispatch.rs`, `vec_overlay.rs`, `app_state.rs`, `main.rs`, `build_smoke.rs`, `vec_glyph*.rs`, `vec_text*.rs` | a shell é da linha |

**O Picker** é o menor drop: só o arquivo novo `shells/desktop/src/vec_pick.rs` (`PathPick` +
`hover_outline`) + `link_explicit` em `vec_text_ride.rs` + a costura de painel/render_loop/input_dispatch
das seções que já existiam. **Zero componente novo, zero schema** — ele reusa `VecTextPath`/`VecPatternPath`.

## 4. IDs / consts / variants / schema novos

Todos os ids de UI são **hash de string** (colisão é de *nome*, não de número). As strings novas:

```
# Texto em caminho
vector.section.textpath   vector.textpath.link     vector.textpath.pick     vector.textpath.detach
vector.textpath.flip      vector.textpath.flip.off vector.textpath.offset   vector.textpath.offset.num
# Pattern along path
vector.section.patternpath   vector.patternpath.link   vector.patternpath.pick   vector.patternpath.detach
vector.patternpath.flip      vector.patternpath.flip.off
vector.patternpath.spacing(.num)  .start(.num)  .end(.num)  .slide(.num)  .offset(.num)
```

- **Componentes ECS:** `ph2d::vec_text_path` (`VecTextPath`) e `ph2d::vec_pattern_path`
  (`VecPatternPath`) — cada um cunha `stable_type_id` própria; presença = cavalga, ausência = solto.
- **Consts de interação** (`LITERAL-PX-OK`): `vec_text_ride::HANDLE_R_PX` · o hit-radius `10.0` do
  clique/hover do Picker · `SPACING_MIN=0.25`/`SPACING_MAX=4.0`/`OFFSET_MAX=2.0` (faixa de PARÂMETRO
  no domínio do documento, em `ph2d-panel-vector/src/lib.rs`).
- **Enum runtime** (não-doc, não-schema): `crate::vec_pick::PathPick{PatternMotif, TextObject}`.
- **Smoke levels:** **21, 22, 23** (texto) + **24** (pattern) — lista compartilhada `build_smoke.rs`.

**Schema: NENHUM bump.** `PROJECT_SCHEMA` (29) e `VEC_SCENE_SCHEMA_VERSION` (13) **intactos** — os
dois vínculos são componentes OPCIONAIS (blob-key própria), não campos apendados. (Os planos previam
apender aos params, o que bumparia `PROJECT_SCHEMA` e recusaria todo projeto salvo — corrigido nas
duas features.)

## 5. O que o `ship.sh`/árvore combinada pega e o `cargo test -p` NÃO

⚠️ **O contador de componentes do ECS é TRÊS e os três subiram por DOIS** (esta linha registra
`VecTextPath` **e** `VecPatternPath`): **`ecs 33→35`, `render 34→36`, `script 34→36`**. Os de
`ph2d-render`/`ph2d-script` só rodam nas suítes deles (rodei os três localmente), mas o **gate da
árvore combinada** é a rede final. Se outra linha não-integrada registrar componentes, **os três
números se CONTAM, não se escolhem** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]): o
valor certo é `base + nº de componentes das linhas fundidas`, e não está em nenhum lado do conflito.

⚠️ **`VECTOR_SECTIONS`** (lista compartilhada em `ids/chrome/vector.rs`) — **2** entradas novas
(`VECTOR_SECTION_TEXTPATH`, `VECTOR_SECTION_PATTERNPATH`) ao FIM. O gate
`every_section_header_is_registered_as_collapsible` pina `len() == 23`. Numa fusão com outra linha
que a tenha tocado, **só ADICIONE, nunca reordene** ([[feedback_a_shared_list_is_merged_against_todays_main]]).

⚠️ **i18n** (`ph2d-i18n/src/lib.rs`) — 2 chaves ao fim, só ADICIONAR.

⚠️ **Níveis de smoke 21/22/23/24** — se outra linha os tomou, renumere os desta (o valor se conta).

⚠️ **LOC no teto:** `ph2d-panel-vector/src/state.rs` = **599/600** e `lib.rs` no limite — o Picker
removeu um comentário órfão de `state.rs` (que mentia sobre o `mod effects`) para caber. A próxima
adição de estado a este painel **obriga split**. Nenhum arquivo estourou; os gates de LOC (workspace
+ painel + shell) estão **verdes**.

⚠️ **3 dívidas red-latentes da wave de parâmetros (mesma linha) foram greenadas no commit do Picker**
— o gate `no_magic_numeric` (as consts SPACING sem `LITERAL-PX-OK`), o `no_tofu_glyphs` (um `→`
U+2192 num `expect` de `pattern_live_tests.rs`) e o `panel_files_under_loc_cap`. Elas nunca rodaram
com `cargo test -p` nas waves anteriores; o `ship.sh` do integrador as pegaria. **Já estão verdes.**

## 6. Contratos congelados encostados

**NENHUM.** `VectorOp`/`Vertex`/`Segment`/… (gate `architecture_vector_contract_surface`, escaneia só
`ph2d-vector-doc`+`-traits`) intactos. `NodeOp`/`OpResolver`/`NodeManifest` idem. `Tool`/`PanelEvent`
idem (o Picker é gesto de canvas + botão de painel, nada de sub-trait novo).

## 7. O que smoke-testar

Todos `--release`, na worktree. **Já foram TODOS smokados e APROVADOS pelo Enio** nesta jornada.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=21 cargo run -p ph2d-host-desktop --release   # texto W0: efeito sobrevive ao re-cook
env PH2D_BUILD_SMOKE=22 cargo run -p ph2d-host-desktop --release   # texto W2/W3: o MOTOR (onda + 2 círculos)
env PH2D_BUILD_SMOKE=23 cargo run -p ph2d-host-desktop --release   # texto W4/W5 + Pick: o GESTO (prender + alça + Picker)
env PH2D_BUILD_SMOKE=24 cargo run -p ph2d-host-desktop --release   # pattern: o Picker (seta -> Pick Path -> clica o arco)
```

- As cenas 23 e 24 **verificam a própria premissa** e imprimem o roteiro numerado: se aparecer
  `PARE e reporte`, a mesa não está posta.
- A 24 abre já com só o motivo selecionado → o painel mostra **"Pick Path"**. Aperta, clica o arco;
  clique no vazio ou botão direito desiste. Alternativa (2 selecionados) = **"Pattern on Path"** auto.

## 8. Verificação local (não é ship — o integrador roda o `ship.sh`)

- Suítes das crates tocadas: **verdes** (ph2d-host-desktop, -vec-scene, -vec-render, -editor-core,
  -panel-vector, -ecs, -render, -script, -i18n).
- `clippy --all-targets` limpo (shell + painel).
- Gates de LOC (workspace + painel + shell) verdes. **Release build** verde.
- Seam do painel: **35** verdes (incl. os 2 pickers + as seções que oferecem só o que aplica).
- **NÃO rodei o `ship.sh` completo** (é do integrador): fmt-skew, machete, deny, audit, typos e a
  **árvore combinada** (o contador de componentes cross-crate, §5) são a rede que só ele fecha.

## 9. Aberto (não bloqueia a integração — refinamentos nomeados)

- Picker: sem realce da FONTE (só do guia sob o cursor); sem tecla Escape dedicada (o botão direito e
  o clique no vazio já desistem). Detalhe em [`docs/Vector Module/23_plano_pattern_along_path.md`](Vector%20Module/23_plano_pattern_along_path.md) §7.
- Pattern: a pose do motivo é ignorada (v1 — as cópias substituem o desenho); escalar as cópias por
  gizmo é decisão de produto adiada. Sem memo (0,597 ms/200 cópias, medido dentro do orçamento).

## 10. Estado: linha PRONTA + handoff. **PARO e espero.**
