# HANDOFF DE INTEGRAÇÃO — `line/Vector` · **Texture Pattern** (2026-08-27)

> O item **(3)** da fila do Enio ([doc 29 §F2](../29_fila_morph_state_machine_e_texture_pattern.md)) —
> o último que estava aberto. O plano é o [33](../33_plano_texture_pattern.md); **todas as waves
> fecharam**, e o que sobra está no §7 deste ficheiro.
>
> ⛔ **Este documento não autoriza nada.** Integração e ship são **só por ordem explícita do Enio**
> (CLAUDE.md §0.7), por um agente integrador dedicado. A linha fecha aqui e PARA.

---

## 1 — Identidade

| | |
|---|---|
| Branch | `line/Vector` |
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| Merge-base | `330582deb` |
| Commits | **20** |
| Ficheiros | 85 |
| Crates novas | `ph2d-vec-pattern` (folha, serde-only) · `ph2d-asset-id` (folha, extraída) |
| ADR | **nenhum** ⇒ fora de toda disputa de número |
| Contrato congelado (§6) | **intocado** (`node.rs`, `tool.rs` — confirmado pelo `collision-surface.sh`) |

---

## 2 — O que a linha entrega, em uma frase por bloco

- **W1 — o assador** (`ph2d-vec-pattern`): `TileLaw` (grade · tijolo-linha · tijolo-coluna ·
  colmeia) + `bake` + `placement`. CPU puro, zero GPU, zero deps. ⭐ *A lei de ladrilho resolve-se
  **ao assar***: o desfasamento entra nos **pixels do ladrilho**, então o render só vê uma imagem e
  um afim — é isso que mantém o custo de quadro igual ao de um sólido.
- **W2 — a porta de render**: `VectorScene::fill_path_image` + `pattern::fill_pattern`. ⭐ Ela
  **revive dois argumentos que estavam mortos** no `fill_path` (`brush_transform` e a regra de
  preenchimento) — e por isso o padrão respeita `EvenOdd` e um buraco fica vazio.
- **W3 — o dado**: `Paint::Pattern(Box<PatternFill>)`. O `Box` mantém o tamanho do `Paint` **igual**
  (gate a afirmá-lo). Schema: `VEC_SCENE` 14→15, `PROJECT_SCHEMA` 99→100, a **tripla** e o degrau.
- **W4 — fonte 1, uma IMAGEM**: `rfd` pela porta da casa, `AssetDb`, e a persistência a espelhar o
  `collect_sprite_pixels`. ⚠️ A identidade é a dos **pixels** (`insert_image_rgba8`), não a do
  ficheiro — ver §10.
- **W5 — o painel**: a secção *Pattern* inteira (Source… · Use Shape… · Tile · Offset · Size · Gap ·
  Shift X/Y · Angle · Repeat) + o 5.º chip de *Fill Type*, com as quatro condições de UI.
- **W6 — as alças na tela**: construída e **RETIRADA no mesmo dia por ordem do Enio** (§6-quater do
  plano). Ver §7 e §9.
- **W7 — fonte 2, uma FORMA do documento**, viva: editar a forma-fonte **re-assa** o ladrilho no
  mesmo quadro (o *"pattern fills are dynamic"* do Figma), com o ciclo próprio recusado.
- **W8 — o smoke**: `PH2D_BUILD_SMOKE=76`, 7 formas, arte sintetizada (assimétrica nos dois eixos,
  com um quadrante transparente-mas-colorido).
- **W9 — *Shift X/Y***: a POSIÇÃO, que era da alça de mover, passa a ser **uma fase de uma
  repetição** (`0..100 %`) nos eixos do padrão. Ver §9.

---

## 3 — Superfície de colisão (`collision-surface.sh`, colada)

```
  merge-base 330582deb   ·   19 commit(s)   ·   85 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        100   (base: 99)
  ⚠   └ tripla do gate               (100, 13, 15)   (base: (99, 13, 14))
  ⚠ VEC_SCENE_SCHEMA                       15   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES — ph2d-ecs: — · espelhos 78 / 78 (base 78)
▸ CONTRATO CONGELADO (§6) — node.rs intocado · tool.rs intocado
▸ ADR — esta linha não cria ADR
▸ Cargo.lock — 2 pacotes '+name' novos: "ph2d-asset-id", "ph2d-vec-pattern"
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️⚠️ **OS TRÊS NÚMEROS SE CONTAM CONTRA O `main` DO DIA, NUNCA SE COPIAM DAQUI.** Este plano já viu
**duas** recontagens (`96 → 97`, `98 → 99`) e a linha `line/components` mexeu no mesmo degrau em
26/08. E ⚠️ **a colisão passa MUDA quando duas linhas escrevem o MESMO literal** — o git não sabe o
que o número significa. Os **três** sítios são
[`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) (o valor **e** a escada) e
[`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs) (a tripla).

---

## 4 — Foundational / partilhado tocado, e por quê

| Ficheiro | Porquê |
|---|---|
| `crates/ph2d-vector/src/scene.rs` | `fill_path_image` — a **porta única** de "uma imagem preenche um caminho". Sem ela o `brush_transform` do peniko não era alcançável de lado nenhum. |
| `crates/ph2d-editor-core/src/ids/chrome/vector_texture_pattern.rs` | bloco de ids **append-only** (o molde dos irmãos). Nenhum id existente renomeado. |
| `crates/ph2d-asset-id/` (**nova**) | O `AssetId` foi **extraído** de `ph2d-asset` para uma folha, porque a `ph2d-vec-scene` é pura e não pode depender daquela crate. ⭐ `ph2d-asset/src/id.rs` virou um `pub use` ⇒ **os 78 sítios de chamada não mudaram**. |
| `shells/desktop/src/project*.rs` | o degrau `99 → 100` + a tripla. ⚠️ Ver §3. |
| `crates/ph2d-vec-scene/src/path_bake_xform.rs` (**novo**) | extracção por teto de LOC — ver §5. |
| `crates/ph2d-vec-render/src/path_bounds.rs` (**novo**) | idem. |
| `crates/ph2d-panel-vector/src/event_texpat.rs` (**novo**) | idem. |

---

## 5 — ⚠️⚠️ O que só o gate BATCHED apanha (e apanhou QUATRO vermelhos)

**Os gates de teto de LOC moram em `ph2d-editor-core/tests/` e VARREM A ÁRVORE.** Nenhum filtro de
nome de crate os alcança ⇒ **um fecho por `cargo test -p <crate>` diz verde com eles vermelhos**.
Medidos contra o `main` em 27/08, os quatro estavam vermelhos **há dias**:

| Ficheiro | Antes | Cap | Curado por |
|---|---|---|---|
| `ph2d-vec-render/src/lib.rs` | 714 | 700 | `path_bounds.rs` (as CAIXAS em px de tela — ali ninguém desenha) |
| `ph2d-vec-scene/src/path_ops.rs` | 711 | 700 | `path_bake_xform.rs` (um afim ENTRA na geometria e o frame desaparece) |
| `ph2d-panel-vector/src/event.rs` | 617 | 600 | `event_texpat.rs` (o molde exacto do `event_contour`) |
| `event_clicks.rs::forwards_plain_click` | 205 | 200 | `is_boolean_click` (a 2.ª família extraída daquela cadeia) |

⛔ **Nenhum por allowlist** (CLAUDE.md §5), e cada corte é por RESPONSABILIDADE.

E o resto do que só o `ship.sh` vê: `fmt` (roda em toda a árvore), `machete` (as duas crates novas
declaram só o que usam), `deny`/`audit` (⚠️ **zero deps externas novas** — as duas crates novas são
folhas com `serde` e nada mais), `typos`.

---

## 6 — Ordem, dependências, e o que NÃO foi smokado

A ordem é a das waves (W1 → W9); nenhuma depende de outra linha. **Nada aqui espera outro módulo.**

**Smokado pelo Enio (5 corridas, todas com report):** os quatro reticulados, os três modos de
repetição, o `Clamp`, o `Column`, os filtros por cima do padrão, as alças (recusadas), a arte-forma
viva, e o contorno.

⏳ **Não smokado:** o *Shift X/Y* (nasceu depois da última corrida dele) e o **reabrir de um
`.ph2dproj` gravado com um padrão de arte-imagem** — o gate cobre a ida e volta do `AssetId`, mas o
ficheiro real só o Enio o produz.

---

## 7 — O que fica ABERTO (bloqueio ou decisão do Enio — **não é trabalho parado**)

1. ⛔ **As alças de canvas foram RETIRADAS** (*"não ficou legal"*, 27/08) — veredito de produto, sem
   mecanismo nomeado. O código existiu em `001b8ba43`. ⛔ **Não reconstruir sem ler o §6-quater do
   plano**: uma 2.ª tentativa começa perguntando *o que ficou pior*. Cerca executável:
   `the_pattern_has_no_canvas_handles_anymore`.
2. ⏳ **Uma forma que nasceu sem contorno não pode ganhar um** (destapado pelo report do contorno,
   §6-quinquies do plano). A recusa do `restyle_selected_strokes` é deliberada e comentada, mas a
   secção *Stroke* é oferecida e fica **inerte** para essa forma. Illustrator, Figma e Inkscape
   **todos** deixam acrescentar. ⇒ ou existe um verbo, ou a secção **some** (a lei que a secção
   *Pattern* já obedece). ⛔ Não construído: é outra secção, e para as formas que o Enio desenha
   funciona.
3. ⏸️ **Padrão no TRAÇO** (o *"as a fill or stroke"* do Figma) — a última lacuna de paridade que o
   plano nomeia. ⚠️ **Preço MEDIDO:** o modelo certo é `StrokeSpec.color: Rgba8` → `.paint: Paint`,
   mas o `Paint` carrega um `Vec` e um `Box` ⇒ **o `StrokeSpec` deixa de ser `Copy`**, e são
   **287 menções em 13 crates**, das quais **22 copiam-no para fora de um *place*** e quebram; mais
   `VEC_SCENE` + a tripla + um degrau. ⛔ A saída barata (um `stroke_pattern` à parte no `VecPath`)
   está recusada: daria ao traço **duas fontes de tinta**.
4. ⏸️ **Size é UM número** (o lado maior, aspecto preservado) e **Gap é UM número** para os dois
   eixos. Os dois são expressáveis hoje (escalar a forma; o dado guarda os dois vãos). Se um smoke
   pedir, o desenho é um **cadeado de aspecto**, não dois campos soltos.
5. ⏸️ **Navegador de assets** — o ADR-0165 chama-se *index before browser*; a W4 usa a porta de
   ficheiro que a casa já tem e que tem gate.
6. ⛔ **Padrão VECTORIAL ladrilhado** (resolução infinita): é irmão do `PathEffect::Hatch` na pilha
   de efeitos, **não** do `Paint`. Outra wave, outro dono.
7. ⛔ **Uma instância de Motion pinta a `fallback`** — fronteira declarada, com gate.

---

## 8 — A UMA LINHA do `CLAUDE.md §5` (o integrador escreve; a narrativa fica AQUI)

Substituir o item **(3)** da fila do Vector por:

> ✅ **(3) O TEXTURE PATTERN FECHOU** ([plano 33](docs/Vector%20Module/33_plano_texture_pattern.md) W1–W9, [handoff](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_texture_pattern_2026-08-27.md)) — `Paint::Pattern`, quatro reticulados (grade · tijolo · coluna · colmeia), três repetições (`Tile`/`Mirror`/`Clamp`) e **duas fontes de arte**: uma imagem, ou ⭐ **uma FORMA do documento, VIVA** (editar a forma-fonte re-assa o ladrilho no mesmo quadro — o modelo do Figma). ⭐⭐ **O Vello 0.8 ladrilha NATIVAMENTE** e a lei do reticulado resolve-se **ao assar**, então uma forma com padrão custa **um** comando de desenho, como um sólido — provado pelo `Encoding`, sem relógio. ⚠️ **O padrão conserva a ORIENTAÇÃO sob afim** (sonda dos dois eixos unitários) — é o único preenchimento desta casa que o faz, e o gradiente radial não pode porque um radial do peniko **é circular**. ⛔ **As alças de canvas foram construídas e RETIRADAS por decisão do Enio** (27/08, *"não ficou legal"*): a posição vive nas fileiras **Shift X/Y**, uma **fase de UMA repetição** (`0..100 %`, e `100` é o mesmo que `0` — a faixa é a periodicidade, não um palpite), e há gate a impedir que voltem. ⚠️ **O `Clamp` ENQUADRA e o enquadramento é DERIVADO, nunca gravado** (dois reports do Enio: escrevê-lo destruía a lei afinada e voltar não a devolvia). ⚠️ E o *"pattern anula stroke"* **era a CENA DO SMOKE**: a ferramenta de forma veste `stroke` sempre, `..VecPath::default()` não, e o `restyle_selected_strokes` recusa quem não tem um ⇒ a secção *Stroke* ficava **pintada e inerte só ali**. Cena **`=76`** · `PROJECT_SCHEMA` **100** e `VEC_SCENE` **15** (⚠️ **reconte**).

---

## 9 — ⚠️ As coisas que uma leitura rápida do diff entende ao CONTRÁRIO

1. **O `pattern_handle.rs` apagado não é código abandonado.** Ele foi construído, smokado, e
   **recusado pelo dono**. A recusa está no §6-quater do plano com o que existia.
2. **`Shift X/Y` não é "mais dois sliders".** É a POSIÇÃO, que antes só a alça escrevia — retirar
   três alças foi retirar **um** ajuste, não três (escala e rotação já tinham fileira).
3. **A tolerância `per * 1e-12` no `set_shift_axis` não é medo de floats.** Sem ela a ida e volta
   `origin → fase → origin` (que corre a **cada quadro** de arrasto) muda o último bit e **cada
   quadro vira um passo de undo**.
4. **`set_shift_axis` mexer só na parte fraccionária não é optimização.** Teleportar a origem é
   invisível no `Tile` (um período é a identidade) e **troca a fase do reflexo** no `Mirror`.
5. **`draw_path_isolated` passou a EXIGIR o mapa de ladrilhos** — não é um argumento a mais. Era a
   segunda porta de desenho dentro da primeira, e era o *"filters anula pattern"*.
6. **O `AssetId` mudou de crate sem mudar um sítio de chamada** (`ph2d-asset/src/id.rs` é um
   `pub use`). O diff parece grande e é um `mv`.
7. **A fileira de chips passou pelo `paint_segmented_group_adaptive`** porque o 5.º chip não cabia —
   e com isso os **quatro antigos voltaram à largura de antes**. Não é um ajuste estético.

---

## 10 — ⚠️ As premissas do plano que a IMPLEMENTAÇÃO refutou

1. *"o `StrokeSpec` é outra casa"* (§7, sobre o padrão no traço) — **subestimava o preço**: o
   problema não é a casa, é o `Copy`. Números no §7 item 3.
2. *"o `insert_image_bytes` serve"* — **não serve.** Ele cunha o id do **ficheiro**; só o
   `insert_image_rgba8` (o id dos **pixels**) volta igual depois de um save. Com o primeiro, reabrir
   o projecto daria um id novo e a fonte apontaria para o nada, **sem erro nenhum**.
3. *"a restrição obriga o rótulo `Tile` a encolher"* — **dissolveu** quando a fileira passou a
   refluir.
4. *"o `Clamp` precisa de escrever `size`/`origin`"* (a 1.ª cura) — **errado**, e o report seguinte
   provou-o. Um modo de **apresentação** não consome o documento.

---

## 11 — ⛔ A lição metodológica desta linha

**Um «não reproduzi» é uma afirmação sobre a POPULAÇÃO que se mediu.** A caça ao *"pattern anula
stroke"* durou três mensagens e os gates estavam todos certos: eles construíam a forma **como o
produto a constrói** (com traço). A cena do smoke construía-a de outra maneira, e era só ali que o
defeito aparecia. ⇒ *Pergunte **em que formas** antes de entregar um instrumento.* A 3.ª pergunta ao
Enio devia ter sido *"em quais?"*, e custava uma linha; o `PH2D_PATTERN_LOG=1` teria dito o mesmo
exigindo dele uma corrida com variável de ambiente.

Corolário, hoje com gate: **uma cena de smoke montada por código não herda o que a ferramenta de
autoria garante** — ela tem de nascer no estado em que o artista a encontraria, senão mede um objecto
que o produto nunca produz.

---

## 12 — Onde ler o mecanismo

- [Plano 33](../33_plano_texture_pattern.md) — §0 (o medido, com endereços) · §1 (estado da arte e o
  que cada um abandonou) · §2 (a porta única de cada pergunta) · §5 (gates + os cinco fenómenos da
  fixtura) · **§6-quater** (a retirada das alças) · **§6-quinquies** (o contorno) · §7 (o que não faz).
- [Doc 29 §F2](../29_fila_morph_state_machine_e_texture_pattern.md) — o pedido original.
- Provas de mutação: 9/9 na wave do *Shift*, com os três controlos no arnês; as contagens de cada
  wave anterior estão nas mensagens de commit.
