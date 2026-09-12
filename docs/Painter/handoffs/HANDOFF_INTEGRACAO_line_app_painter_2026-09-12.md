# HANDOFF de INTEGRAÇÃO — `line/app-painter` (W2 **FASE D**, a família nasce e corta)

> **Data:** 2026-09-12 · **Ramo:** `line/app-painter` · **Merge-base:** `67dca9411` · **7 commits**
> · **79 ficheiros** · Linha **NOVA** (worktree criada no dia; sem fases anteriores).
>
> ⚠️ **Isto descreve o mundo em 12/09.** O estado vivo é o `CLAUDE.md §5`.

---

## §1 · O veredito, em números

| grandeza | antes (`main` de 12/09) | depois | Δ |
|---|---:|---:|---:|
| **shell** (`shells/desktop`, como a catraca mede) | 225 394 L / 917 f | **215 394 L / 874 f** | **−10 000 L · −43 f** |
| `crates/ph2d-app-painter` (nova) | — | **10 547 L / 48 f** | +48 f |
| folhas novas (`ph2d-sprite-screen` + `ph2d-preview-slot`) | — | 251 L / 4 f | +4 f |
| o que sobra da família na shell | — | **961 L / 6 f** | a **costura** |

**A prova (`scripts/nextest-list-diff.py`), exacta nos dois sentidos:**

```
antes: 22668 testes (22041 chaves) | depois: 22671 (22044)
MOVED (mesma chave, outro pacote/binário): 79
ONLY-A (perdidos): 0
ONLY-B (novos):    3
```

Os **3** `ONLY-B` são os gates do `FAMILY` que esta linha escreveu (§5.4).

**Portão batched:** `scripts/nextest-impacted.sh` → **16 248 testes**.
**`cargo test -p ph2d-host-desktop --test it` à parte (regra 2 do §1):** **791 passed · 0 failed.**
**Clippy** (`-p ph2d-app-painter -p ph2d-sprite-screen -p ph2d-preview-slot --all-targets`): **zero**
avisos, com **e** sem a feature. **`cargo fmt --all --check`:** limpo.
**`cargo test -p ph2d-app-registry-init`:** verde. **`cargo run -p ph2d-app-sync`:** `ok: 4 blocos`,
**8** famílias.

⭐ **Zero contador partilhado se move** (`collision-surface.sh`, corrido nesta worktree):
`PROJECT_SCHEMA` **128** (base 128) · tripla `(128, 13, 22)` igual · `FLIP_SCHEMA` 13 ·
`DOC_VERSION` 18 · os dois espelhos de registo em **86** · contratos §6 **intocados** · zero
marcadores de conflito · **nenhum ficheiro da linha passa do teto de LOC**.
⚠️ Os três `+name` novos do `Cargo.lock` são as **três crates internas** desta linha — nenhum
pacote externo entrou.

---

## §2 · ⛔⛔ A CORRECÇÃO DE TERRITÓRIO, e ela tem de ser lida antes de tudo

O bloco de abertura dava o alvo como **~37 f / ~9 300 L**, listando `brush_smoke`,
`brush_corner_smoke` e `paint_opacity_smoke` (com os `_tests` irmãos) como roteadores de cena do
Painter. ⛔ **Esses seis ficheiros / 1 159 L são do VECTOR** — são as cenas `PH2D_BUILD_SMOKE=77`,
`=78` e `=79` do **pincel de contorno** (plano 36), construídas de `ph2d_vec_scene`, e movê-las
teria sido violação de território contra a `line/app-vec`.

Em troca, o bloco **omitia três roteadores reais**: `substrate_smoke`, `taper_smoke` e `line_smoke`.
Dois deles apareciam como **âncoras** na primeira medição — precisamente por estarem fora do
conjunto que eu declarei família.

> ⇒ É a regra 5 do §1 (*o PREFIXO não é a família*) a morder pelas **duas** pontas no mesmo dia:
> a palavra «brush»/«paint» **capturou** o que não era meu, e a ausência dela **escondeu** o que era.
> *Abra o cabeçalho de cada ficheiro antes de o mover* — e, quando a família tem roteadores, o censo
> honesto é pela **env que cada um lê**, não pelo nome do ficheiro.

A família verdadeira: **40 ficheiros / 10 060 L**.

---

## §3 · ⛔⛔⛔ A RÉGUA DO FECHO TEM UM FURO, E ELE ERRA *A FAVOR*

`scripts/fecho-da-familia.py` **nunca resolve `super::`** — zero ocorrências da palavra no script.
Dentro de `render_loop/`, `super::X` **é** `crate::render_loop::X`.

Medido nesta família:

| o que a régua diz | o que é |
|---|---|
| `render_loop/bgremoval_preview.rs` — **2** ficheiros a nomeá-lo | **11** |
| âncoras: **6** | **7** — falta o `render_loop/mod.rs`, alcançado por `super::note_preview_px` |
| intermédio `MOVE 20 f / 4 139 L` | `MOVE 11 f / 2 111 L` |

⚠️ **O TECTO (com tudo curado) não muda** — e é por isso que o furo passa despercebido: quem corre
a régua até ao fim vê o número certo. Quem ler o **intermédio** como lista de tarefas move a mais
**9 ficheiros / 2 028 L** e descobre isso ficheiro a ficheiro, que é exactamente o *«oscilar
crate↔shell a cada erro novo»* que o HOWTO §2.12 manda evitar.

⚠️ **A varredura à mão que acha o furo também mente, e eu paguei-a:** dos 20 candidatos `super::`,
**8 eram falsos** — um `#[path]`-child e um `#[cfg(test)] mod tests` inline fazem `super::` ser o
**próprio ficheiro**, não o directório. ⇒ *o `super::` tem duas leituras e a distinção é a
NIDIFICAÇÃO, não o texto.*

**Correcção de uma linha** (para quem tocar no script): tratar `super::X` como
`crate::<dir(ficheiro)>::X` **apenas** quando o ficheiro é declarado por `mod` no `mod.rs` do
directório — e nunca quando ele é `#[path]`-incluído por um irmão.
⛔ **Eu não editei o script**: ele está a ser corrido pelas outras duas linhas desta rodada neste
momento, e mudá-lo debaixo delas é pior que o furo. *É decisão do integrador.*

---

## §4 · As SEIS âncoras, e o que cada uma era de facto

O fecho deu `MOVE 40 / FICA 0` com as seis curadas — a mesma forma da `physics` (*«as 15 625 linhas
estavam presas por SEIS SÍMBOLOS»*). **Nenhuma exigiu porta nova no `AppHost`.**

| # | âncora | o que era | cura |
|---|---|---|---|
| 1 | `app_state.rs` (9 f) | **DOIS TIPOS**, e ambos ALIAS: `PainterPreview = ph2d_tool_runtime::PreviewCache` e `PainterPreviewGpu = BgremovalPreviewGpu` | o 2.º virou a folha `ph2d-preview-slot` (§4.1) |
| 2 | `image_import.rs` (6 f) | ⭐ **FACHADA** de 6 linhas: `pub(crate) use ph2d_image_import::*` | escrever o nome da crate |
| 3 | `bgremoval_preview.rs` (11 f) | **UMA função pura** partilhada por **quatro** assuntos | folha `ph2d-sprite-screen` |
| 4 | `hero_intents/texture_edit.rs` | o funil `read_sprite_source`/`commit_edited_texture` | **fecho de leitura** (§4.2) |
| 5 | `chrome_hit.rs` | `pointer_over_chrome` — **já é a porta #5 do `AppHost`** | fica no invólucro |
| 6 | `input_dispatch/fill_drag.rs` | um predicado sobre um `thread_local` | um **`bool`** que atravessa |
| 7 | ⛔ `render_loop/mod.rs` | `super::note_preview_px` — **invisível à régua** (§3) | fecho `&dyn Fn(u64)` |

### §4.1 — ⛔⛔ A folha da ranhura de pré-visualização, e os DOIS destinos RECUSADOS

`BgremovalPreviewGpu` era uma `struct` no `app_state.rs` cujo próprio doc dizia *«Tool-agnostic —
shared by BgRemoval and the Painter»*. É o caso `GroupDragSnapshot` da Fase C: **o TIPO muda-se
para junto do assunto dele; os CAMPOS da `App` ficam** com quem possui uma pré-visualização em
curso.

Os dois destinos óbvios foram **medidos e recusados** — a tabela vive no header da folha, porque a
próxima pessoa vai propor os mesmos dois:

| destino | por que parecia certo | por que NÃO |
|---|---|---|
| `ph2d-tool-runtime` | é onde o gémeo de CPU (`PreviewCache`) vive | **teto de LOC 650**, está em **633**. O gate declara-se *«the discipline mechanism»*, manda *«push the new logic back into the bridge»* e escreve que **a próxima subida exige ADR**. Os 17 que sobram só chegariam **apagando os comentários de campo** — e o do `arc_token` **regista uma medição**. ⛔ *Apagar prosa medida para caber num teto é gamar a disciplina que o teto existe para impor.* Eu escrevi o tipo lá, o gate reprovou, e a cura foi **REVERTER** |
| `ph2d-render`, ao lado do `IndividualTextureStore` | o `texture_id` é uma ranhura **daquele** store | `individual.rs` está em **708** linhas com a folga do censo em **709** — uma linha. E o `arc_token`/`entity_bits` são escrituração da **ponte**, não vocabulário de desenho |

⇒ folha própria, **zero dependências**. ⚠️ Os dois `type` ficam na shell como alias, e é isso que
deixa os dois ficheiros da remoção de fundo — e toda a prosa que os cita — **byte a byte iguais**.

### §4.2 — O idioma do fecho não foi inventado aqui

Três âncoras caíram por **fecho** (`read_source`, `note_preview_px`) ou por **valor** (`fill_drag_armed`).
⚠️ Nenhuma delas é invenção: a `ph2d-tool-runtime` **já declara por escrito** que ler pixels de um
sprite *«requires `SimWorld + SpriteRenderer + AssetDb`, which this crate refuses to depend on —
they're shell foundation»* e passa um *reader closure*. *Antes de pedir uma porta ao host, escreva
em TIPOS o que a função precisa* — e depois procure quem já respondeu à mesma pergunta.

---

## §5 · ⛔⛔ AS ARMADILHAS QUE ESTA FASE PAGOU

### 5.1 ⛔⛔⛔ **O `#[cfg(test)]` órfão — o defeito MAIS CARO, e era MUDO**

Ao apagar os 25 `mod X;` do `render_loop/mod.rs` deixei para trás os **cabeçalhos** deles: **21**
blocos de doc-comment e **cinco `#[cfg(test)]` órfãos**. Um `#[cfg]` órfão **cola-se ao item
seguinte**: os cinco empilharam-se no `mod push_look_probe;` e as cinco prosas passaram a documentar
o módulo errado.

> **Compilava, e passava.** Nenhum gate deste repo pergunta a quem um `#[cfg]` pertence.

⇒ os cabeçalhos foram extraídos do `HEAD` **um a um** e viajaram com o código; os cinco
`#[cfg(test)]` foram **restaurados na crate** — sem eles, cinco módulos de teste entravam no
**binário do produto**.

### 5.2 ⛔⛔ **O gémeo MUDO do §2.6 pagou-se DUAS vezes, e a regra 2 do §1 apanhou as duas**

| momento | `cargo check --all-targets` | `cargo test --test it` |
|---|---|---|
| depois da folha do afim | **verde** | **1 FAILED** |
| depois do corte das 28 pontes | **verde** | **6 FAILED** |

*Um `check` verde não diz nada sobre um gate que só corre.*

⭐ E entre os seis, **dois eram pisos de população a disparar como desenhados**: o meu (`4`
chamadores onde esperava `14`, com a cura escrita na mensagem — *acrescentar a raiz, não baixar o
número*) e o `a_layout_never_commands_a_panel_a_bridge_owns`, que leu **7** painéis de ferramenta
onde esperava **8**. ⚠️ Esse já tinha a 2.ª raiz da `app-motion` escrita nele, da Fase C; hoje tem a
3.ª. *Um censo que perde um sítio aprova tudo o que vivia nele.*

### 5.3 ⛔⛔ **MOVER UM FICHEIRO TROCA O REGIME DE TETO QUE O GOVERNA**

`painter_bridge.rs` carregava um `// ph2d-loc-cap: mid-refactor` no topo. Esse marcador é honrado
pelo teto da **shell** (`file_loc_caps`, 600) e ⛔ **é inerte em `crates/`**, onde manda o
`architecture_workspace_file_loc_cap` (700), que só aceita uma entrada em `FILE_OVERAGE_OK` com
assinatura do Coordenador.

⇒ o ficheiro atravessou a fronteira **a 1093 linhas com uma isenção que deixou de existir**, e o
portão batched apanhou-o. ⛔ A cura não foi uma entrada nova na lista de folgas (`CLAUDE.md` §5.0:
*a cura é corte por responsabilidade*): dois irmãos novos,
**`painter_bridge_upload`** (a faixa de uplode — o corte que a própria mensagem do gate sugere) e
**`painter_bridge_phases`** (as cinco fases curtas do quadro). `1093 → 691`.
⚠️ O marcador inerte **saiu**: *uma isenção que o gate em vigor não lê é pior que nenhuma — ela
promete uma folga que não existe.*

### 5.4 ⛔ **O meu próprio censo leu a PRÓPRIA prosa, no dia em que foi escrito**

O gate `todo_roteador_declarado_esta_no_family` perguntava `fonte.contains("pub const NIVEIS: u32")`
e acusou o **`family_tests.rs`**, que traz a agulha **duas** vezes: no doc-comment que a explica e
dentro do literal de string que a procura. É a HOWTO §2.12 a morder o ficheiro escrito para a
honrar. ⇒ a marca passou a ser a linha **COMEÇAR** pela declaração.

⚠️ E o **piso** desse mesmo censo nasceu em `7` e ficou obsoleto no dia seguinte (população real:
**37**) — corrigido para `30` no commit seguinte. *Uma catraca sem censo de obsolescência não desce:
ela vira licença*, e isto foi escrito por quem citou essa lei no commit anterior.

### 5.5 ⛔ **O `#[allow]` que eu escrevi e tive de apagar**

A 1.ª versão do `bind_document` levava `#[allow(clippy::too_many_arguments)]` (12 argumentos).
⛔ Silenciar um diagnóstico é armengo — e este repo tem esse allow **proibido por escrito**: o caso
gémeo do `sculpt3d/keys.rs` está **aberto há dias** por essa razão exacta.
⇒ a cura é a que a queixa pede: **agrupar em struct** (`BindCtx`, construída no sítio da chamada,
campos nomeados — o molde do `CanvasCtx` da `physics`).
⚠️ **E o molde mordeu na hora:** a 1.ª versão punha o `renderer` no contexto **e** deixava o fecho
capturá-lo → `E0500`. *O empréstimo disjunto só existe quando cada campo é nomeado UMA vez* ⇒ o
fecho **recebe** o renderer.

### 5.6 ⚠️ **Compilar a crate SOZINHA revela código morto que a shell nunca via**

Na shell, `panel-painter-layers` era feature **`default`** — o `#[cfg]` estava sempre ligado. A
crate compila-se também **sem** ela, e aí apareceu um relógio reatribuído e nunca lido. Curado por
**estrutura** (a atribuição entrou no bloco do `cfg`; a declaração tem duas formas, `mut` e não-`mut`),
⛔ nunca por `#[allow]`.
⚠️ E a feature é a armadilha §2.4 na forma exacta: ela é **declarada na crate** e **reencaminhada
pela shell**. Sem isso o `cfg` seria falso **por construção** e a crate compilaria verde com o
painel de camadas desligado.

### 5.7 ⚠️ **Renomear um teste que se move lê-se como teste PERDIDO**

Eu renomeei o gate do afim ao movê-lo para a folha, e o `nextest-list-diff` — que indexa por
**NOME** — leu `ONLY-A = 1`. O nome voltou ao original. *Um nome melhor não vale uma prova mais
fraca.*

### 5.8 ⚠️ **Um `assert` DEPOIS do laço que escreve deixa a árvore meio-editada**

Um dos meus scripts de reescrita contava enquanto escrevia e só verificava no fim; a contagem
falhou com 5 dos 6 ficheiros já gravados. *Conferir antes de escrever é a diferença entre um
no-op e uma árvore inconsistente* — os scripts seguintes contam primeiro.

---

## §6 · ⚠️ PREMISSAS MINHAS QUE A MEDIÇÃO DERRUBOU

1. ⛔ **«Os nove ficheiros presos ao `app_state` precisam de um tipo que se move.»** — **Meio
   falso.** Eram **dois** tipos e os dois eram **alias**; um deles apontava já para uma crate. Só o
   segundo era uma `struct` de verdade. *Contar consumidores não é contar símbolos.*
2. ⛔ **«O `ph2d-tool-runtime` é o destino óbvio do `PreviewGpu`.»** — **Recusado por um teto que
   se declara mecanismo de disciplina.** Eu já tinha escrito o tipo lá quando o gate falou.
3. ⛔ **«O bloco `super::X` que a minha varredura achou é todo real.»** — **8 de 20 eram falsos**
   (§3).
4. ⚠️ **«A régua do fecho vê tudo o que o compilador vê.»** — **Não vê `super::`**, e erra a favor.
5. ⚠️ **«Os quatro ficheiros de gesto são corpos por mover.»** — Medidos, eles são a **costura**:
   `painter_grid_erase.rs` é orquestração pura sobre `App` (chama outros métodos de `App`), e o
   maior deles é `impl App` quase de ponta a ponta. §7.

---

## §7 · O que FICA na shell, e é DESENHO (961 L / 6 f)

| o quê | L | porquê |
|---|---:|---|
| `input_dispatch/painter_canvas_input.rs` | 503 | o **gesto de canvas** (`impl App`) + o gate puro `canvas_down_accepts`. É ele que fala com o `chrome_hit` (a porta #5) |
| `input_dispatch/painter_falloff_input.rs` | 121 | idem, o falloff |
| `input_dispatch/painter_curve_input.rs` | 87 | idem, a curva |
| `input_dispatch/painter_grid_erase.rs` | 69 | idem, o botão direito do Grid Stamp |
| `input_dispatch/painter_canvas_mods.rs` | 38 | os modificadores **fora de banda** (o `CanvasPaintTool` é contrato **congelado** §6 e o ponteiro dele não os leva) |
| `hero_intents/image_edit/painter.rs` | 143 | o **Apply**, que passa pelo funil `commit_edited_texture` — a porta de produção que oito ferramentas de imagem partilham |

⭐ **É o padrão de chegada**, e melhor que o das irmãs: a `physics` deixou 3 182 L em 13 ficheiros e
a `flip` 2 271 em 36. *O que sai são os CORPOS; o que decide a ordem do quadro fica* — e um gesto de
canvas **é** a ordem do quadro.

⚠️ **Se uma linha futura quiser parti-los**, a medição já está feita: os quatro tocam `gfx` (10×),
`tools`, `modifiers`, `pending_painter_move`, os dois contadores de carimbo e `last_pointer`. ⛔ Um
trait de extensão **não serve** (os corpos precisam do `gfx` e nenhuma porta do `AppHost` devolve
handle); o molde é o `CanvasCtx` da `physics`, e o ganho seria ~800 L contra 23 sítios de chamada
que hoje ficam byte-idênticos.

---

## §8 · O FIM DA LINHA (gateado) — o que o bloco pediu, e onde está

| pedido do bloco | estado |
|---|---|
| os roteadores passam para a crate, **contados** na forma `PH2D_*_SMOKE` | ✅ **seis**, cada um a ler a própria `env` na crate |
| ⛔ `PH2D_PAINT_PERF` · `PH2D_PREVIEW_DIAG` · `PH2D_PREVIEW_DUMP` **fora** do `FAMILY` | ✅ são diagnóstico; a ausência está **documentada no `lib.rs`** |
| o `const FAMILY` declara cada `max_level` **CONTADO no `match`** | ✅ e a contagem tem **duas espécies** (§8.1) |
| ⛔⛔ registar-se sem declarar roteador REPROVA | ✅ declara seis |
| `cargo test -p ph2d-app-registry-init` verde | ✅ |

### §8.1 — ⚠️ Os `max_level` têm DUAS espécies nesta família

| roteador | forma | `max_level` |
|---|---|---:|
| `PH2D_IMPASTO_SMOKE` | `match` de **dois braços** (`=2` abre a tela de **4096²**, onde a regressão da dobra vivia) | `2` |
| os outros **cinco** | `var_os(..).is_some()` — de **PRESENÇA**, não de nível | `1` |

⛔ Declarar mais do que `1` nos cinco seria **prometer uma cena que ninguém escreveu**: *um nível
declarado diz ao dono que ele tem uma cena para ver*, e `=2` ali abre exactamente a mesma que `=1`.

⭐ O `match` do impasto saiu para uma porta **pura** (`edge_for`), e é isso que torna o `NIVEIS`
mensurável: *o ambiente é global ao processo, logo um gate que o escrevesse mediria o teste vizinho.*

**Três gates novos, um por defeito** (`family_tests.rs`): o `NIVEIS` mentir sobre o `match` (medido
pelas **duas** pontas) · o `FAMILY` mentir sobre o `NIVEIS` (são dois sítios e dois sítios divergem)
· e o **roteador órfão**, por censo do directório com piso de população. Prova de mutação: um 7.º
ficheiro que declare `NIVEIS` e não esteja no `FAMILY` reprova com a lista dos sete.

---

## §9 · Para o INTEGRADOR

1. ⭐ **Zero contadores partilhados, zero contrato, zero ADR, zero pacote externo.** As três crates
   novas são `path` deps internas.
2. ⚠️ **Ficheiros de costura que esta linha tocou** (conflito em região diferente é legítimo):
   `app_state.rs` (só o corpo de dois `type`), `main.rs` (−1 `mod`), `render_loop/mod.rs` (−25 `mod`
   + 21 cabeçalhos órfãos + 3 argumentos novos no sítio da chamada do `dispatch`),
   `render_loop/sim_extract.rs` e `sim_extract_sheet.rs` (uma re-exportação e uma morada de prosa),
   `render_loop/bgremoval_preview.rs` (−65 L, um `use`), `forwarding.rs` (1 linha),
   `input_dispatch/*` e `shells/desktop/Cargo.toml` + `Cargo.lock`.
3. ⚠️ **Uma linha do `CLAUDE.md` §5** (módulo Painter) — o texto está no §10.
4. **Símbolos novos que podem colidir:**
   ```
   ph2d_sprite_screen::{sprite_image_to_screen_affine, unfolded_quad, cell_count}
   ph2d_preview_slot::PreviewGpu
   ph2d_app_painter::{FAMILY, shape_grab, painter_bridge_upload, painter_bridge_phases, …}
   ph2d_app_painter::painter_bridge_phases::BindCtx
   feature "panel-painter-layers" em ph2d-app-painter (reencaminhada pela shell)
   ```
5. ⚠️ **A `ph2d-tool-runtime` NÃO foi tocada** (o teto dela ficou em 633/650) — mas eu escrevi lá e
   revertim: se um merge trouxer um `PreviewGpu` dela, é resíduo meu e deve ser descartado.
6. ⛔ **O furo da régua do fecho (§3) é decisão sua.** Eu não editei
   `scripts/fecho-da-familia.py` porque as outras duas linhas desta rodada estão a corrê-lo agora.
7. ⚠️ **Os gates de GPU não correram** (`#[ignore]`, precisam de adapter) — *skip gracioso não é
   verde*. Os que esta linha atravessa são os `painter_preview_*` e o `painter_stamp_device`; eles
   compilam e estão listados, mas ninguém os executou nesta jornada.
8. ⚠️ **A suíte do Painter em DEBUG e os `--ignored` com `--test-threads=1`** (precedente registado
   do módulo) **não foram corridos** — esta linha não tocou no motor (`ph2d-tool-painter` está
   intocada), só na metade de composição, mas o precedente fica nomeado.

---

## §10 · A UMA LINHA para o `CLAUDE.md` §5 (módulo **Painter**)

> ⭐⭐⭐ **E A METADE-SHELL SAIU (12/09, W2 Fase D):** a família vive em
> [`ph2d-app-painter`](crates/ph2d-app-painter/) (48 f / 10 547 L) e a shell desce de **225 394 para
> 215 394** linhas, com `ONLY-A = 0`, `ONLY-B = 3` e 79 `MOVED`. ⭐ **As 40 linhas-ficheiro estavam
> presas por SEIS SÍMBOLOS** e nenhum pediu porta nova no `AppHost`: duas eram **fachadas** (o
> `image_import` de 6 linhas; o `PainterPreview`, que já era alias de uma crate), uma era a porta #5
> que já existia, e três caíram escritas em TIPOS — um `bool`, um fecho de leitura (o idioma que a
> `ph2d-tool-runtime` já declarava) e um contador. ⭐⭐ Duas folhas novas:
> [`ph2d-sprite-screen`](crates/ph2d-sprite-screen/) (o afim `imagem-px → ecrã-px`, partilhado por
> **quatro** assuntos) e [`ph2d-preview-slot`](crates/ph2d-preview-slot/) (a ranhura de GPU que o
> `app_state.rs` guardava por inércia — ⛔ **não** foi para a `ph2d-tool-runtime`, onde o gémeo de
> CPU vive, porque o teto de LOC dela se declara *«the discipline mechanism»* e os 17 de folga só
> chegariam apagando prosa **medida**). ⛔⛔ **E mover um ficheiro TROCA O REGIME DE TETO que o
> governa:** o `painter_bridge.rs` atravessou a fronteira a 1093 linhas com um `// ph2d-loc-cap:`
> que é **inerte** em `crates/`, e partiu-se em três por responsabilidade (`1093 → 691`).
> ⚠️⚠️ **E a régua do fecho tem um furo que erra A FAVOR — ela não resolve `super::`**: leu `2`
> ficheiros onde havia `11` e perdeu uma âncora inteira. O que fica na shell são **961 L em 6
> ficheiros**, os gestos de canvas e o Apply — a costura, por desenho.
> [Handoff](docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_app_painter_2026-09-12.md) (⚠️ o §5 tem as
> **oito** armadilhas — entre elas cinco `#[cfg(test)]` órfãos que se colaram ao módulo vizinho **em
> silêncio** — e o §6 as **cinco** premissas minhas que a medição derrubou).

---

## §11 · O que fica ABERTO

1. ⏳ **Os quatro ficheiros de gesto** (`input_dispatch/painter_*`, 780 L) podem partir-se em
   corpo + invólucro pelo molde do `CanvasCtx`. **Não é dívida** — é o padrão de chegada —, mas a
   medição está no §7 para quem decidir o contrário.
2. ⛔ **O furo do `super::` na régua do fecho** (§3) — a correcção está escrita; a edição é do
   integrador, por causa das outras linhas.
3. ⚠️ **`painter_bridge.rs` está a `691` de `700`** — **9 linhas de folga**. O próximo bloco que lhe
   entrar estoura o teto, e a cura continua a ser corte (os dois irmãos já existem e recebem-no de
   graça).
4. ⚠️ **Os gates de GPU desta família nunca correram** (§9.7). Ninguém sabe se eles passam.
5. ⏳ **A `ph2d-app-painter` não tem `#[cfg(test)] test-support`** — nenhum `#[cfg(test)] pub fn`
   dela atravessa para a shell hoje. Se um gate da shell vier a precisar de um, é a feature do
   HOWTO §2.5, do tamanho do que atravessa.
