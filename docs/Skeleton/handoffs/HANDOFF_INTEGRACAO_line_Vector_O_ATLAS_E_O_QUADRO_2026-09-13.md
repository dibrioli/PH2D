# HANDOFF DE INTEGRAÇÃO — `line/Vector` · O `SMOOTH` ERA O ATLAS, A PORTA DO VELLO, E O ORÇAMENTO POR QUADRO

> **2026-09-13** · DIRETRIZ §1.5.9. A linha fecha aqui e **PARA** — não integra, não pusha
> ([`CLAUDE.md §0.7`](../../../CLAUDE.md)). Reabertura de 13/09 sobre o `main` enviado `1d43da737`;
> o item pegado foi o 1.º ABERTO do handoff de 10/09: *«o custo de GPU do `Smooth`, não medido»*.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/Vector` |
| merge-base com `main` | `1d43da737` (o `main` enviado, CI verde) |
| HEAD | o commit de docs que traz este parágrafo (o último de código é `411768603`) |
| commits | **8** (7 de trabalho + 1 de docs: `8fa3035f5` atlas · `35a88bce6` a porta do Vello · `2466f7a30` a sonda de palavras · `f98a93af2` fila · `2f4ef9f28` a sonda do chrome · `83b8b1e52` o orçamento por quadro · `411768603` o fecho) |
| `shells/desktop` | **zero** linhas tocadas |
| contadores partilhados | **zero** (`PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA`, `FLIP_SCHEMA`, `DOC_VERSION`, os três registos) — ver §3 |
| contrato congelado (§6) | **intocado** · zero ADR |

---

## §2 — Foundational / partilhado tocado, e por quê

| ficheiro | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-vector/src/scene.rs` | `draw_stable_image_transformed` (e `draw_stable_image` delega nela) · `probe_bin_info_words` (doc-hidden, irmã da `probe_image_ids`) · `VELLO_BIN_DATA_WORDS` · `CLIPPED_IMAGE_INFO_WORDS` | sim |
| `crates/ph2d-vector/src/lib.rs` | re-exporta as duas constantes · declara a sonda nova | sim |
| `crates/ph2d-vector/src/atlas_probe_pieces_tests.rs` (NOVO) | a sonda do atlas em N peças + 4 gates | sim |
| `crates/ph2d-render/src/vello_keepalive.rs` (NOVO) | a porta que impede um quadro sem recurso tardio de apagar o atlas + 2 gates | sim |
| `crates/ph2d-render/src/vello_pass.rs` | um campo `keepalive` e as **duas** entregas ao Vello (`render_to_intermediate`, `render`) passam por ele | **muda comportamento** só em quadros sem recurso tardio (ver §6.2) |
| `crates/ph2d-render/Cargo.toml` + `Cargo.lock` | `ph2d-vector` como **dev-dependency** (sem ciclo: ela só depende do `vello` e de folhas) | sim |
| `crates/ph2d-render/tests/it/skin_pieces_gpu_cost.rs` (NOVO) | sonda de GPU (buracos/costuras/relógio) + o gate de GPU do quadro vazio | sim |
| `crates/ph2d-editor-core/tests/it/vello_bin_budget_of_an_editor_frame.rs` (NOVO) | sonda: quanto do buffer do Vello o chrome gasta | sim |
| `crates/ph2d-poly2d/src/refine.rs` | **só doc** (a atribuição à camada de recorte, refutada) | — |
| `crates/ph2d-skeleton-live/src/skin_image.rs` | a cache guarda `StableImage` · `SKIN_FRAME_PIECES` · a parte proporcional · o aviso | muda o produto (é a cura) |
| `crates/ph2d-app-skeleton/src/state.rs` | só doc do campo da cache | — |

---

## §3 — Símbolos que podem COLIDIR

### `collision-surface.sh` (⚠️ REFERÊNCIA — re-rode antes de fundir)

(colado no §10 com o resto do portão)

### Itens públicos NOVOS

- `ph2d_vector::{VELLO_BIN_DATA_WORDS, CLIPPED_IMAGE_INFO_WORDS}` · `VectorScene::draw_stable_image_transformed` · `VectorScene::probe_bin_info_words` (doc-hidden)
- `ph2d_skeleton_live::skin_image::SKIN_FRAME_PIECES`

### ⛔ Item público APAGADO

- **`ph2d_skeleton_live::skin_image::RgbaArc`** — o alias só servia a cache antiga. Censo no merge-base:
  **zero** usos fora do ficheiro. ⚠️ A shell re-exporta o módulo inteiro (`skeleton_skin_image`), então
  uma linha paralela que o tenha começado a usar partia aqui.
- **`SkinImageCache`** muda de valor: `(u32, u32, Arc<Vec<u8>>)` → `ph2d_vector::StableImage`.

---

## §4 — O que a linha entrega

### §4.1 — F6-e: o report *«Smooth bugado quebrando a forma»* era o ATLAS

A pele desenhava cada triângulo pela porta CRUA, que cunha um **id do atlas por chamada** — N peças
eram N cópias inteiras da imagem num atlas que pára em `8192²`, e o que não cabe **não é desenhado,
em silêncio**. A decisão é da CPU (`Resolver`), logo mede-se sem ecrã:

| imagem | peças | porta crua: peças que NÃO aparecem | estável |
|---|---:|---:|---:|
| `320×96` (o smoke) | `216` | `0` — e `25,3 MB` reenviados por quadro | `0` |
| `320×96` | `7 776` (o report) | **`5 651`** | `0` |
| `1024×1024` | `216` (**o `Fast`, o modo de omissão**) | **`152`** | `0` |

### §4.2 — F6-f: um quadro sem recurso tardio apagava o atlas do Vello

Medido na GPU pelo `VelloPass`: imagem → **quadro vazio** → imagem → imagem = `255 → 0 → 0 → 0`.
Mecanismo no `vello` 0.10: sem patch nenhum o `Resolver` sai por `resolve_solid_paths_only` com
atlas de largura `0`, o `render.rs` troca a textura por uma de `1×1` e depois por uma NOVA em branco,
e o `ImageCache` da CPU não reenvia. **Vale para todo utilizador de `StableImage`**; a cura é a porta
única (`vello_keepalive`).

### §4.3 — F6-g: o orçamento do `Smooth` é do QUADRO

| quem gasta o buffer fixo do Vello (`1 << 18` palavras) | medido |
|---|---|
| chrome do editor, painéis de omissão (texto incluído — Inter embutida) | `109` |
| chrome, TODOS os painéis abertos | `1 167`–`1 188` |
| uma peça da pele | `11` + bins `1,14`–`3,98` |
| cena só com a pele (GPU) | desenha `17 496` · **em branco a `21 600`** · pânico `≥ 31 104` (debug) |

⇒ `SKIN_FRAME_PIECES = 131 072 ÷ 15 = 8 738`, repartido **proporcionalmente** entre as imagens
presas; dentro dele **a tolerância decide**. ⛔ O tecto antigo de `1 024` por imagem passava por cima
da tolerância e não protegia o quadro.

**GPU**, `320×96`, alvo limpo por grelha:

| peças | zoom 4 (carga `2,2`) | zoom 8 | costura (zoom 8) |
|---:|---:|---:|---:|
| sem recorte | `1,15 ms` | `1,25 ms` | – |
| `216` | `0,99 ms` | `1,58 ms` | `33 476` px |
| `3 456` | `2,17 ms` | `2,46 ms` | `132 164` px |
| `7 776` | `3,61 ms` | `5,02 ms` | `259 231` px |

---

## §5 — Gates novos, e as provas de mutação

| gate | onde | mutação | resultado |
|---|---|---|---|
| `a_skinned_image_is_one_atlas_resident_however_many_pieces_and_frames` | `ph2d-skeleton-live` | uma `StableImage` nova por peça | ✅ RED na asserção dos ids |
| `the_smooth_pieces_of_all_skinned_images_share_one_frame_budget` | `ph2d-skeleton-live` | `max_pieces: o.max_pieces` (tecto por imagem) | ✅ RED na asserção do orçamento |
| `every_hand_off_to_vello_goes_through_the_keepalive` | `ph2d-render` (lib) | a entrega do `render` sem a porta | ✅ RED (`2` entregas, `1` porta) |
| `a_stable_image_survives_a_frame_without_late_bound_resources` | `ph2d-render` (it, **GPU, `#[ignore]`**) | a entrega do `render_to_intermediate` sem a porta | ✅ RED `(255, 0, 0)` |
| `a_scene_without_late_bound_resources_leaves_with_one_and_one_that_has_them_is_not_copied` | `ph2d-render` (lib) | — | decisão sem GPU |
| `the_raw_port_mints_one_atlas_copy_per_piece_even_from_one_arc` | `ph2d-vector` | — | o CONTROLO do gate abaixo |
| `a_stable_image_split_into_pieces_takes_one_atlas_slot_and_loses_no_piece` | `ph2d-vector` | — | corrida gémea como oráculo |
| `a_skin_piece_costs_eleven_vello_bin_info_words_and_the_cost_is_linear` | `ph2d-vector` | — | fixa o `11` |
| `the_bin_data_words_constant_is_the_one_vello_allocates` | `ph2d-vector` | — | contra o `RenderConfig` do `vello_encoding` |

⚠️ Os quatro controlos (árvore limpa) correram **1 teste cada** e passaram; os quatro RED foram
**falhas de teste**, nenhuma de compilação (conferido nos registos).

---

## §6 — Coisas que uma leitura rápida do diff entende ao contrário

1. **«O `Smooth` passou a gastar MUITO mais peças.»** Numa dobra forte, sim — por desenho: o tecto
   de `1 024` por imagem era um palpite que passava por cima da tolerância. Uma imagem a `k = 6`
   são `7 776` peças: `~3,6–5,0 ms` de GPU e `~2,5 ms` de CPU (medido a 10/09). O `Smooth` é
   **opt-in** e o `PH2D_SKIN_PIECES` afina o orçamento do quadro.
2. **«O `VelloPass` copia a cena em todo quadro.»** Não: só num quadro **sem nenhum recurso tardio**
   (sem texto, imagem nem gradiente). No editor não há nenhum; um runtime sem texto pagaria a cópia de
   uma cena só de caminhos sólidos.
3. **«`RefineOptions::default().max_pieces` ainda é `1024`, logo o tecto ficou.»** É o neutro da
   folha; o produto substitui-o sempre pela parte do quadro.
4. **«O gate de GPU protege a cura do Vello.»** Só a metade do `render_to_intermediate`, e ele é
   `#[ignore]` (o CI não o corre). A metade do `render` (o ecrã) é o **censo** de texto.
5. **«A sonda do chrome diz que o editor quase não gasta o buffer, logo metade é demais.»** É um PISO:
   painéis sem selecção nem documento, e a arte do canvas não passa pelo `paint_hero_screen`. A outra
   metade é da arte, que **não foi medida**.
6. **«As costuras são novas.»** Não: existem desde a F6 (`Fast` incluído); só não havia régua.

---

## §7 — As premissas que a medição derrubou

1. ⛔⛔ **«O limite é a camada de recorte, e não há como o medir sem ecrã»** (handoff de 10/09) —
   era o atlas, e mede-se na CPU.
2. ⛔ **«`1 024` fica do lado seguro»** — o lado seguro de quê: o recurso real é por QUADRO.
3. ⛔ **A 1.ª tabela da sonda de GPU mentiu duas vezes:** `21 600` peças leram os números EXACTOS de
   `7 776` (um desenho que não acontece deixa a textura com o quadro anterior); e com o alvo «limpo»
   por uma cena VAZIA tudo leu buraco já com 2 peças — **foi assim que a F6-f apareceu**.
4. ⚠️ A leitura *«75 caminhos é pouco demais, o texto não entrou»* estava errada: o
   `without_system_fonts` regista a Inter embutida.

---

## §8 — O que fica ABERTO

| item | estado |
|---|---|
| ⛔⛔ **As costuras** (fundo a espreitar até ~29 % em cada aresta de triângulo, todo modo) | a família: sobrepor `~0,5 px` (perfeito em arte opaca, dobra a composição em translúcida) · ou o **pipeline de triângulos texturados**, com razão agora medida três vezes |
| ⚠️ **A ordem de profundidade da pele** | desenhada na fase de overlay do Vello por ordem de ARQUÉTIPO — duas imagens presas sobrepostas podem trocar de frente. **Não medido** |
| ⏳ **A reserva da arte do canvas** no buffer do Vello | não medida; é por isso que a pele fica com metade |
| ⏳ Malhas `Fast` que sozinhas passam do orçamento | não há o que cortar; aviso único no stderr |
| ⏳ O mapa dobra sobre si em dobras fortes · F4 *«undo tem poucos passos»* | como no handoff de 10/09 |

---

## §9 — O que o INTEGRADOR faz

1. `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` nesta worktree.
2. **Nenhum contador partilhado se move** — confirme no `main` do dia, não na coluna `base:`.
3. ⚠️ **O `VelloPass` é de TODOS**: a porta nova vale para os cinco construtores do produto (a shell,
   o FX vectorial, os dois bakes do Motion e a sonda do padrão). Ela só age em quadros sem recurso
   tardio, e aí é a cura.
4. ⚠️ **`RgbaArc` saiu** (§3) — uma linha que o use parte na árvore combinada.

---

## §10 — O portão batched, e o smoke

Régua = merge-base `1d43da737`.

| passo | resultado |
|---|---|
| `BASE=<merge-base> bash scripts/nextest-impacted.sh` | ✅ **13 667 passaram, 0 falharam** (11 351 saltados) — carga `72` no fim, e nenhum membro da família de flakes reprovou |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✅ |
| `cargo clippy --workspace --all-targets` | ✅ zero avisos — ⚠️ a 1.ª corrida apanhou **um** `type_complexity` na sonda de GPU, curado por alias |
| `cargo fmt --all --check` | ✅ — ⚠️ a 1.ª corrida apanhou os ficheiros escritos à mão desta wave, curado por `cargo fmt -p` nas quatro crates |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` · `doc-index.sh --check` | ✅ |
| provas de mutação (§5) | ✅ 4 RED na asserção certa, 4 controlos com 1 teste cada |

### `collision-surface.sh`, colado

```text
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 1d43da737   ·   6 commit(s)   ·   17 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      22   (base: 22)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                               85   (base: 85)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)
▸ CONTRATO CONGELADO (§6) — intocado nos dois
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️ **Auditoria, duas lentes** (DIRETIVA §3):

- **LENTE correção · CLAIM** *uma imagem presa é UM residente do atlas em quantas peças for* · **TRAÇO**
  `fase_vector_edit_overlay.rs` → `draw_skinned_images` (`skin_image.rs`) → `stable_image` (cache por
  `AssetId`) → `draw_stable_image_transformed` (`scene.rs`) → `ImageBrush::new(image.data.clone())`
  (mesmo `Blob`, mesmo id) → `Resolver::resolve` → `ImageCache::get_or_insert` (`Occupied`) ·
  **ASSERÇÃO-VERMELHA** `a_skinned_image_is_one_atlas_resident_however_many_pieces_and_frames`
  (mutação: `StableImage` nova por peça ⇒ RED) · **NÃO-CHECADO-PELA-COMPILAÇÃO** que o id sobreviva
  a um quadro sem recurso tardio (é a F6-f) · **LOC LIDAS** ~1 900 (skin_image, scene, atlas_probe_tests,
  image_cache.rs e resolve.rs do `vello_encoding`, render.rs do `vello`).
- **LENTE wiring · CLAIM** *toda entrega ao Vello passa pela porta que mantém o atlas* · **TRAÇO** as
  cinco construções de `VelloPass` do produto → `render_to_intermediate` / `render` →
  `keepalive.scene_for_vello` → `Renderer::render_to_texture` · **ASSERÇÃO-VERMELHA** o censo
  `every_hand_off_to_vello_goes_through_the_keepalive` (a do ecrã) + o gate de GPU (a intermédia) ·
  **NÃO-CHECADO** o custo da cópia num runtime sem texto (nenhum existe hoje) · **LOC LIDAS** ~700.

### O smoke compilado

`target/*/incremental` reclamado antes (`19 GB`). Depois do commit `411768603`, na worktree:

```text
=== build 1
    Finished `smoke` profile [optimized] target(s) in 1m 09s
=== build 2
    Finished `smoke` profile [optimized] target(s) in 0.29s
```

Zero linhas `Compiling` na 2.ª. O comando entregue ao dono é o mesmo pacote, perfil e árvore:

```text
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=1 PH2D_BONE_LOG=1 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **O que o smoke do dono pode e não pode ver:** a imagem do smoke é `320×96`, logo o defeito do
`Fast` com arte grande (§4.1, última linha) **não aparece nela** — esse lado está provado pelo gate e
pela sonda, não pela cena. O que ele vê é o `Smooth` numa dobra forte: antes partia a forma, agora
tem de a manter; e o log `[bone] pele suave:` diz quantas peças e que parte do orçamento.

---

## §11 — A UMA LINHA proposta para o `CLAUDE.md §5` (entrada **Vector**, depois da 2.ª mídia)

> ✅ **O `Smooth` «bugado» era o ATLAS** (13/09, [handoff](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_O_ATLAS_E_O_QUADRO_2026-09-13.md)): cada peça copiava a imagem inteira para o atlas do Vello (o `Fast` também partia com arte grande) · um quadro sem recurso tardio apagava toda `StableImage` (curado no `VelloPass`) · e o orçamento passou a ser do QUADRO (`SKIN_FRAME_PIECES`, do buffer fixo do Vello). ⏳ As **costuras** entre triângulos e a ordem de profundidade da pele ficam abertas.
