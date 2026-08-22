# HANDOFF DE INTEGRAÇÃO — `line/Sprite` (DIRETRIZ §1.5.9)

> **Escrito em 2026-08-22, por ordem do Enio.** A linha está FECHADA: não integra, não pusha,
> não faz ship. Este documento é o entregável que passa ao **agente integrador**.
>
> ⚠️ **A tabela do §3 é REFERÊNCIA, não EVIDÊNCIA.** Ela mede esta linha contra o `main` do dia em
> que fechou. Se outra linha fundir antes desta, todo número da coluna «base» muda e este handoff
> passa a descrever um `main` que já não existe — **e ele não reclama**. O integrador **re-roda
> `collision-surface.sh` em cada worktree imediatamente antes de fundir**; a divergência entre as
> duas leituras é ela própria um achado.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/Sprite` |
| HEAD | `caf7a3df5` |
| merge-base com `main` | `ee1432203` |
| commits | **87** |
| ficheiros tocados | **236** |
| suítes no fecho | ecs **197** · render **304** · editor-core **1212** · panel-inspector **176** · host-desktop **3424** · script **17** · clippy limpo |

**O que a linha entregou**, em cinco blocos (o detalhe de cada um está nos planos
`docs/Sprite_projeto/17`, `18` e na auditoria `20`):

1. **Pixels de sprite `Individual` com NOME** — sobrevivem ao save (crate nova `ph2d-sprite-sheet`).
2. **Precisão de 16 bits** — `Precision`, dither, paridade CPU/GPU.
3. **Emissive por sprite.**
4. **Folha (sheet) hand-packed** — arranjo, bake, export; ações na hierarquia.
5. **As três seções do Inspector que a spec declarou em 2026-05 e ninguém construiu:**
   **§5 9-Slice** · **§12 Sockets/Named Anchors** (com **gizmo de canvas**) · e a suíte de
   regressão que 30 testes `#[cfg(any())]` esperavam. ⛔ **A §11 Animation continua por construir**
   (precisa de um `SpriteFrames` que não existe).

---

## 2. Foundational / compartilhado tocado, e porquê

| Onde | Ficheiros | Natureza |
|---|---|---|
| `shells/desktop` | 97 | extract, render loop, input dispatch, undo, persistência, smokes |
| `crates/ph2d-editor-core` | 42 | ids, modelos de Inspector, action bus, ratchets de LOC |
| `crates/ph2d-panel-inspector` | 35 | as seções novas + os cortes por cap |
| `crates/ph2d-render` | 17 | `nine_slice`, `individual`, `renderer`, registo-espelho |
| `crates/ph2d-ecs` | 11 | **componentes novos** + registo |
| `crates/ph2d-sprite-sheet` | 6 | **CRATE NOVA** |
| `crates/ph2d-color` | 4 | `Precision`, `dither` — **passou de dev-dep a dep normal** |
| `crates/ph2d-script` | 1 | só o **terceiro contador de registo** |
| `crates/ph2d-asset`, `-imageio`, `-imageio-png`, `-ui-testkit`, `-tool-painter`, `-tool-registry-init`, `-panel-hierarchy`, `-panel-equalize-sizes` | 1–4 cada | aditivo |
| `CLAUDE.md`, `SKILL_Stack`, `scripts/*`, `project-memory/*` | 1 cada | ⚠️ **ficheiros de coordenação — ver §5** |

⚠️ **Tudo aditivo, exceto três remoções deliberadas** (§4).

---

## 3. Superfície de colisão — SAÍDA DO SCRIPT, colada

```text
SUPERFÍCIE DE COLISÃO — line/Sprite contra main
  merge-base ee1432203   ·   87 commit(s)   ·   236 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                         86   (base: 84)
  ⚠   └ tripla do gate               (86, 13, 14)   (base: (84, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
  ⚠ ph2d-ecs                               63   (base: 57)
  ⚠ ph2d-render (espelho)                  64   (base: 58)
  ⚠ ph2d-script (espelho)                  64   (base: 58)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0159   próximo livre: 0160
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 1 pacote(s) '+name' novo(s):
      "ph2d-sprite-sheet"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
      643 / 700   crates/ph2d-editor-core/src/action_bus.rs  (tem marcador/allowlist — confira o valor congelado)
  ✗   721 / 700   crates/ph2d-render/src/individual.rs
  ✗   876 / 700   crates/ph2d-render/src/renderer.rs
     1644 / 600   shells/desktop/src/app_state.rs  (tem marcador/allowlist — confira o valor congelado)
     6466 / 600   shells/desktop/src/input_dispatch.rs  (tem marcador/allowlist — confira o valor congelado)
      819 / 600   shells/desktop/src/input_dispatch/gizmo_drag.rs  (tem marcador/allowlist — confira o valor congelado)
     1210 / 600   shells/desktop/src/main.rs  (tem marcador/allowlist — confira o valor congelado)
    10167 / 600   shells/desktop/src/render_loop/mod.rs  (tem marcador/allowlist — confira o valor congelado)
     1053 / 600   shells/desktop/src/render_loop/painter_bridge.rs  (tem marcador/allowlist — confira o valor congelado)
      812 / 600   shells/desktop/src/render_loop/sim_extract.rs  (tem marcador/allowlist — confira o valor congelado)
     1165 / 600   shells/desktop/src/render_loop/snapshots.rs  (tem marcador/allowlist — confira o valor congelado)
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.

```

⚠️ **Os dois `✗` de LOC NÃO são regressão — medi-os.** O script compara contra o teto genérico
(700); ambos têm entrada própria em `architecture_workspace_file_loc_cap.rs`:
`individual.rs` **721 / 722** (esta linha **encolheu-o** de 969, e a catraca desceu junto) e
`renderer.rs` **876 / 1000**. O gate real está verde. *Um `✗` do mapa não é um `✗` do gate.*

### Leitura, símbolo a símbolo

| Símbolo | Valor desta linha | Base | Nota para o integrador |
|---|---|---|---|
| `PROJECT_SCHEMA` | **86** | 84 | **DOIS degraus** (85 e 86). ⚠️ A escada (`project_schema.rs`) e a tripla (`project_schema_tests.rs`) são ficheiros IRMÃOS — um degrau escrito no errado funde limpo e evapora. A tripla está em `(86, 13, 14)`. |
| registo `ph2d-ecs` | **63** | 57 | +6 componentes |
| registo `ph2d-render` (espelho) | **64** | 58 | mantém `ecs + 1` |
| registo `ph2d-script` (espelho) | **64** | 58 | ⚠️ **ficou 2 atrás durante a jornada** e só o `collision-surface` o viu — ver §7 |
| `LIVE_SECTIONS` | **15** | 13 | lista ORDENADA: `+INSP_LIVE_SLICE_*`, `+INSP_LIVE_ANCHOR_*` |
| slots de nota (`SectionNotes`) | **15** | 13 | ⚠️ array indexado; a família da física **deslocou-se de 9-12 para 10-13** |
| `EditorAction` | +11 variantes | — | `InspectorSliceEdit`, `InspectorAnchorEdit`, `InspectorSpritePrecisionChange`, `InspectorSpriteEmissiveChange`, `Hier{MergeToLayers,ExportImage,PackSheet,ArrangeSheet,BakeSheet,ExportSheet,RemoveFromSheet}` |
| ADR | **nenhum criado** | 0159 | fora de toda disputa de número |
| contratos congelados | **intocados** | — | ver §4 |

**Componentes ECS novos** (nomes canónicos — é por eles que o save indexa):
`ph2d::ecs::SpritePixels` · `SpriteSheetRef` · `SpriteSheetFrame` · `SpriteEmissive` ·
`SliceNine` · `NamedAnchorList`.

**Ids novos** (famílias inteiras, em ficheiros próprios — colisão improvável, mas grepáveis):
`ids/inspector_slice.rs` · `ids/inspector_anchor.rs` · `ids/inspector_sampling.rs` ·
`ids/live_sections.rs`. ⚠️ `INSP_ANCHOR_ROW` é um array de **64** entradas, e o comprimento dele
**é** `ph2d_ecs::ANCHORS_MAX` — há gate na shell a prender os dois.

**Ratchets de LOC que esta linha MOVEU** (as tolerâncias só descem; um merge que as suba é erro):
`apply_event_impl` 477→**292** · `paint_inspector` 431→**380** · `sync_sprite_fields` **removida**
(o cluster saiu) · `ph2d-render/src/individual.rs` 969→**722**.

---

## 4. Contratos congelados encostados

**Nenhum.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` estão intocados, e o
`collision-surface` confirma-o.

⚠️ **Três remoções deliberadas, todas em superfície NÃO congelada e todas nascidas nesta linha
(nunca chegaram ao `main`):**

| O que saiu | Porquê — todos MEDIDOS |
|---|---|
| `SliceTileMode::Adaptive` + campo `stretch_value` | o resultado é binário e o resto é fixo: todo o curso do slider era morto menos **um** ponto, invisível. Um controlo contínuo sobre um resultado de duas posições. |
| `SliceDrawMode::Tiled` | era o `Sliced` mais `S ⇒ repeat`, o que tornava **esticar uma região inexprimível**. Um modo que é o outro menos uma capacidade. |
| `SliceFieldEdit::{Attach, Detach}` + o botão «× Remove 9-Slice» | duas portas para «o 9-slice está ligado?», ao lado da caixa. Sem componente e com ele desligado desenham igual, gravam igual e mostram a mesma seção. |

⚠️ **É por causa destas que o `PROJECT_SCHEMA` foi a 86:** o blob do `SliceNine` é name-keyed mas
**posicional por dentro**, e um projeto gravado antes delas seria lido torto em silêncio.

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

1. ⚠️ **`ph2d-sprite-sheet` é CRATE NOVA** — primeira vez que `machete` / `deny` / `audit` a veem.
2. ⚠️ **`ph2d-color` passou de `dev-dependencies` a `dependencies`** na `ph2d-editor-core`
   (o tipo `Precision` atravessa o snapshot). O `machete` olha para isto.
3. ⚠️ **`smallvec` entrou na `ph2d-ecs`** — **não** é dep nova no projeto (a `ph2d-vector-doc`, a
   `-font` e a `ph2d-editor-core` já a usam, mesma versão, mesma feature `serde`).
4. **fmt/typos pré-fork**: 236 ficheiros, muitos com prosa densa em PT. O `typos` corre no ship e
   **não** correu aqui.
5. **clippy latente**: as cinco crates da linha estão limpas com `--all-targets`; o ship corre a
   workspace inteira **com features**.
6. **RUSTSEC**: advisory-db avança sozinha — um `cargo audit` verde ontem não é verde hoje.

---

## 6. Ordem, dependências e o que smoke-testar

**Ordem:** os 87 commits são lineares e não têm dependência fora da própria ordem. O rebase é
`--ff-only` contra o `main` do dia.

### Smokes desta linha

```
cd <worktree> && env PH2D_SLICE_SMOKE=1 cargo run -p ph2d-host-desktop --release
```
| env | cena | estado |
|---|---|---|
| `PH2D_SLICE_SMOKE=1` | 9-slice: os CANTOS, contra um sprite normal | ✅ smokado pelo Enio |
| `PH2D_SLICE_SMOKE=2` | a EMENDA contra a borda (paridade + `Whole`) | ✅ smokado |
| `PH2D_SLICE_SMOKE=3` | esticar contra repetir (a grelha decide) | ✅ smokado |
| `PH2D_SOCKET_SMOKE=1` | §12: três formas de âncora **+ o gizmo arrastável** | ✅ smokado |
| `PH2D_SHEET_SMOKE=1` | folha hand-packed | ✅ smokado |
| `PH2D_EMISSIVE_SMOKE=1` | emissive por sprite | ✅ smokado |
| `PH2D_DITHER_SMOKE=1` | dither de 16→8 bits | ✅ smokado |

### ⛔ O que NÃO foi smokado, e o integrador deve saber

- **A persistência ponta-a-ponta do 9-slice e das âncoras.** Há gate de round-trip em memória
  (`save.rs::the_sprite_authoring_components_survive_the_disk`), mas **ninguém gravou e reabriu um
  `.ph2d` real** — o `io_menu` continua stub, com path fixo e sem diálogo.
- **Um sprite ESPELHADO com 9-slice.** O defeito foi encontrado e curado na auditoria de fecho,
  com gate red-first; nenhum smoke o encena.
- **Bulk-select** das seções novas: há gates, não há smoke.
- **A §12 sob rotação/escala do sprite:** coberto por teste, não por olho.

---

## 7. Achados do fecho que o integrador deve conhecer

1. ⚠️ **O terceiro contador de registo (`ph2d-script`) ficou 2 atrás** e passou a jornada inteira
   assim. Nenhuma suíte que esta linha corre passa por lá — nem o `cargo check -p`, nem as cinco
   crates do gate batched. **Quem o apanhou foi o `collision-surface.sh` deste handoff.**
   ⚠️ **É a SEGUNDA vez na mesma linha**, e a nota que descreve o mecanismo já estava escrita ao
   lado do número desde a primeira. *Uma nota que descreve o mecanismo não o impede.*
   → **Ao integrar N linhas, rode `cargo test -p ph2d-script --lib` explicitamente.**
2. **Duas notas de diferido envelheceram e foram corrigidas** (o `#[ignore]` do golden W5 dizia que
   o `NamedAnchorList` não existia; a spec §3.5 descrevia cinco controlos apagados).
3. **`docs/Sprite_projeto/03_inspector_secoes.md` §3.5 ganhou uma tabela `⛔ Recusas MEDIDAS`** com
   cinco linhas — é o que impede alguém reconstruir o que foi medido e rejeitado.

---

## 8. `CLAUDE.md §5` — a UMA LINHA a editar na integração

O bullet **Sprite Inspector** existe e diz que 9-Slice/Animation/Sockets **nunca foram
construídas**. Ao integrar, a linha **Aberto** passa a:

> **Aberto:** ⛔ a **§11 Animation** continua a única das três por construir (pede um `SpriteFrames`
> que não existe) · ⛔ **nada consome uma âncora** — o ADR-0072 §2.6 (API de runtime: Rust, Luau,
> MCP) é autoria sem consumidor · o `AnchorData::user_data` não tem UI, com o `variant_editor`
> órfão a apontar-lhe · os 4 goldens seguem `unimplemented!()` (falta o arnês headless) · UI real
> de Save/Open (o `io_menu` é stub).

⛔ **Não acrescente parágrafo de jornada ao §5** — a narrativa é este handoff.

---

## 9. Higiene

`rm -rf target/*/incremental` corre imediatamente a seguir a este documento (item 7 do §1.5.9).

⚠️ **Dois ficheiros por versionar ficam na raiz da worktree de propósito:** `Sprite_Sheet.png` e
`Sprite_Sheet.json` — saída de um smoke de export, **não** entram no commit.
