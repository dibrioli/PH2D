# HANDOFF DE INTEGRAÇÃO — `line/app-vec` (W2/L4, **FASE B, 2.ª volta**) — 2026-09-12

> **Estado: FECHADA, e o FIM DA LINHA GATEADO FOI ALCANÇADO.** A chave `"vec"` saiu da catraca
> `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`, que fica só com a `motion`.
> A linha não integra e não faz ship (CLAUDE.md §0.7).

## §1 — Identidade

| | |
|---|---|
| branch | `line/app-vec` |
| HEAD | `8371823b0cdaa1ac1a87594314d8afcb40e3932d` |
| merge-base | `27ab894298c74dbeba700eb30c1eab0e795c7d7d` |
| commits | **8** |
| diffstat | 77 files changed, 4066 insertions(+), 3075 deletions(-) |

⚠️ **O rebase foi um FAST-FORWARD:** o HEAD desta worktree era **ancestral** do `main` — a Fase B
de 11/09 já tinha sido integrada. Zero conflitos, e o merge-base é o `main` de hoje.

---

## §2 — O que saiu, medido

| | antes (o `main` de hoje) | depois |
|---|---:|---:|
| `shells/desktop` — LOC (a régua da catraca) | 398 037 | **390 646** |
| `shells/desktop` — ficheiros | 1 561 | **1538** |
| família `vec_*` na shell | 92 | **76** |
| `crates/ph2d-app-vec/src` — ficheiros | 33 | **60** |

⚠️ **O `TETO_LOC` do `the_shell_only_shrinks` NÃO foi tocado** (regra 1 do BLOCOS §1). Quem o
reconta é o integrador, sobre a árvore combinada, **depois** do `cargo fmt --all`.

### Crates novas (duas, as duas FOLHAS PARTILHADAS)

| crate | o que é | porquê ela existe |
|---|---|---|
| [`ph2d-skeleton-live`](../../../crates/ph2d-skeleton-live/) | o esqueleto **vivo no documento**: prender uma forma ou uma imagem aos ossos, os segmentos, a corrente, a âncora de IK | ⛔⛔ **Não podia ser a `ph2d-skeleton-ecs`, e o motivo é um CICLO**: o `bind` precisa do mapa da `ph2d-vec-entities`, e essa **já depende** da `skeleton-ecs`. *Uma folha nova não é escolha de gosto quando a alternativa é um ciclo.* |
| [`ph2d-image-import`](../../../crates/ph2d-image-import/) | o ficheiro de imagem entra e vira uma sprite na cena | 507 L, e o **único** dos seis candidatos com **zero** `crate::` para outro módulo da shell |

E **duas folhas em crates que já existiam**: `ph2d_vec_scene::cook_tinted` (era
`build_smoke::shape`, 4 linhas, **22** chamadores de cinco famílias) e
`ph2d_vec_entities::filter::set_filter` (11 chamadores).

⭐ **As quatro portas da shell FICAM, a delegar numa linha.** Apagá-las obrigaria a reescrever
~60 sítios de chamada de famílias que não estão abertas — a wave que o §6 do handoff anterior
recusou fazer, e que continua recusada.

---

## §3 — ⛔⛔ O FECHO RE-MEDIDO: o número que o bloco pedia

O §3 do handoff de 11/09 media quatro cenários sobre um grafo cuja **raiz** (`name_unique`,
`vec_entities`, `morph_set`, `off_canvas`) saiu da shell em 12/09. Refeito:

| | ficheiros | LOC |
|---|---:|---:|
| candidato (os 92 `vec_*` + 6 `vector_bridge*`) | 98 | 28 111 |
| **soldado por `#[path]`** | 98 | 28 111 |
| **movia hoje, no início da jornada** | **21** | **4 953** |
| ainda move (no fim da jornada) | 15 | 3 404 |

⭐ **A solda arrasta ZERO ficheiros extra** — os 98 são fechados entre si. Na Fase B o
`vec_entities` soldava **30 de 50**; a folha levou esse nó consigo.

### ⚠️⚠️ A régua corrigiu-se TRÊS vezes, e as três a favor de mover demasiado

*É a 5.ª, 6.ª e 7.ª ocorrência do padrão nesta família — quatro na Fase A, quatro na Fase B.*

| # | a sonda dizia | a verdade | o furo |
|---|---:|---:|---|
| 1 | «o fecho é a shell INTEIRA» (1 320 de 1 320) | 98 | tratou `mod X;` no `main.rs` como **aresta dura bidireccional**. Não é: o `main.rs` é a raiz de composição, e um `mod` lá é a **superfície de reescrita**, não um acoplamento. Com ele, todo ficheiro toca todo outro pelo pai comum — *verdade trivial e inútil* |
| 2 | `main.rs` solda 5 ficheiros | solda 0 | o **controlo negativo** apanhou-o: a solda depende de o FILHO ser **independentemente endereçável**, não de quem é o pai. `#[path] mod x;` no `main.rs` faz `crate::x` (uma linha a reescrever); dentro de `vec_gizmo_view.rs` faz `crate::vec_gizmo_view::x` — submódulo, viaja com o pai |
| 3 | **52** ficheiros movem | **21** | o ponto fixo removia ficheiros **soltos**, e a solda é um **GRUPO**: `vec_bucket.rs` (fica) solda `vec_bucket_tests.rs` (movia). Partir um par soldado não compila |

⛔ **E o controlo positivo que a Fase B pagou continua no lugar:** a sonda **não branqueia
strings** ao construir o grafo, porque *o caminho de um `#[path]` É uma string* — branqueá-la
apagava a aresta que domina tudo.

---

## §4 — ⛔⛔ O ACHADO DESTA VOLTA: o censo era por PREFIXO DE NOME DE FICHEIRO

O conjunto candidato desta linha — em **ambas** as fases anteriores — era «os ficheiros
`shells/desktop/src/vec_*.rs`». **A família não se chama toda assim.** Quatro módulos dela moram em
nomes próprios e nunca entraram em censo nenhum:

`ui_panel_spec` (a árvore autorada como painel vivo) · `widget_icon` · `svg_import` ·
`svg_import_smoke` — e os **quatro são puros** (`App` ×0, `gfx` ×0).

⭐ **E entre eles vinha o QUINTO ROTEADOR**, o `PH2D_VEC_SVG_SMOKE`. O bloco listava quatro e
mandava **contar**; contados, são cinco.

⇒ *A unidade da posse é o **ASSUNTO**, nunca o nome do ficheiro.* É a §2.7 do HOWTO um nível
acima: lá um censo por prefixo varre **zero** e fica verde; aqui varreu **menos**, e a diferença
leu-se como *«não há mais nada para mover»*.

---

## §5 — ⭐ O FIM DA LINHA, gateado

`cargo test -p ph2d-app-registry-init`: **5/5**. `"vec"` fora da catraca.

Os **cinco** roteadores vivem em `ph2d_app_vec`, com a env lida lá e `max_level: 1` **contado**
(são interruptores — `var_os(..).is_some()`, uma cena cada; não há `match` de níveis em nenhum):

`PH2D_VEC_APPEARANCE_SMOKE` · `PH2D_VEC_BONE_SMOKE` · `PH2D_VEC_FADE_SMOKE` ·
`PH2D_VEC_STACK_SMOKE` · `PH2D_VEC_SVG_SMOKE`

⛔ **O que NÃO é declarado, e porquê.** O `PH2D_BUILD_SMOKE` e o `PH2D_UI_MOTION_SMOKE` aparecem
como smokes do Vector no `CLAUDE.md` §5 e **continuam a ser lidos pela shell**. Declará-los diria
que a crate responde por cenas que ela não encaminha — e **nenhum gate do registo o apanharia**,
porque eles medem a FORMA do nome e o `max_level`, nunca se a env é lida deste lado. *Um registo
que se pode mentir sem reprovar só vale o cuidado de quem o escreve.*

⚠️ E o doc-comment do `FAMILY` que **explicava o `routers: &[]`** foi **reescrito no mesmo commit**:
*uma dívida cumprida e não apagada lê-se como dívida aberta.*

---

## §6 — ⛔⛔ UM GATE ESTAVA A CONTAR A POPULAÇÃO ERRADA HÁ UMA FASE INTEIRA

O `every_host_that_rewrites_verts_faces_the_radius_handle` tem um piso de `>= 5` e **nomeia** os
cinco hosts que reescrevem `verts`. O `shape_live` — *a forma viva*, o primeiro deles — mudou-se
para `ph2d-app-vec` na **Fase A**, e o censo continuou a ler `5`, porque o `skeleton_live.rs` da
shell **também** acabava em `_live.rs` e entrou no lugar dele.

⭐⭐ **O piso segurou o NÚMERO enquanto a POPULAÇÃO trocava por baixo dele.** É a mesma forma da
catraca sem censo de obsolescência (`CLAUDE.md` §5.0), um nível abaixo: *um controlo positivo que
conta **quantos** não vê **quais***. Só quando a pele saiu da shell é que o número caiu para 4 e
isto ficou visível.

**Cura:** o censo varre as **três** árvores onde a lei de facto vive (`ARVORES`), o piso é **6**, e
a mensagem **imprime os achados**. ⚠️ E o módulo da pele chama-se `skin_live.rs` dentro da crate
(não `bind.rs`) porque o censo filtra por `*_live.rs` — e o `corner_handles.rs`, que é a
**POLÍTICA**, nomeava-o por `skeleton_live::recook`: renomear sem lhe tocar partia um elo vivo.

---

## §7 — ⚠️ AS ARMADILHAS DO HOWTO §2 QUE ESTA VOLTA PAGOU

| § | armadilha | como apareceu aqui |
|---|---|---|
| 1.3 | dependências invisíveis até a crate existir | **SETE** nesta volta: `ph2d-vec-boolean` (a lâmina do `cut_line`) · `ph2d-painter-brush` (o lazy-mouse do lápis) · `ph2d-panel-authored` · `ph2d-ui-codegen` · `ph2d-vec-svg` · `ph2d-unique-name` · e na folha do esqueleto mais `ph2d-vector`, `ph2d-vec-skin`, `ph2d-asset`. *O número é a medida de quanto a shell escondia* |
| 2.4 | **a FEATURE não viaja com o código** | a `serde` da `ph2d-poly2d` é **opcional** e quem a ligava era a SHELL. Numa crate que não a declara, `postcard::to_allocvec(&Mesh2d)` deixa de compilar — ⭐ e esta falha **alto**, que é a metade boa |
| 2.5 | `#[cfg(test)]` é invisível da outra crate | **três** itens atravessaram: `TokenCtx::factory` (4 ficheiros da shell o constroem) e os dois auxiliares `quadro`/`pior_desvio`. ⛔ Copiá-los seriam duas definições da mesma régua |
| 2.6 | `include_str!` **e o gémeo em runtime** | o `include_str!` falhou alto; **três** gates de `read_to_string` de caminho fixo só falharam **a correr** |
| 2.9 | a agulha nomeia um ENDEREÇO | 5 agulhas re-apontadas — ⚠️ e **uma delas eu apontei para o sítio ERRADO** (§8) |
| 2.13 | a agulha que nomeia a VISIBILIDADE | **52** `pub(crate)` → `pub`; e o `the_two_halves_read_the_glyph` ancorava em `pub(crate) fn icon_face(` ⇒ larga a visibilidade |

### ⛔⛔ E a armadilha NOVA desta volta: `cargo check -p <shell> --all-targets` NÃO compila os testes de uma DEPENDÊNCIA

`cargo check -p ph2d-skeleton-live` ficou **verde** com **quatro** gates partidos dentro da crate,
e `cargo check -p ph2d-host-desktop --all-targets` também — porque `--all-targets` no pacote da
shell não alcança os *test targets* das crates de que ela depende. Quem os apanhou foi o
`cargo nextest list --workspace`.

⇒ **Uma linha que cria ou alimenta uma crate corre `--workspace`, não `-p`.**

---

## §8 — ⚠️ AS PREMISSAS MINHAS QUE A MEDIÇÃO DERRUBOU

1. ⛔⛔ **«O fecho é a shell inteira»** — furo da régua, não do código (§3, #1).
2. ⛔ **«52 ficheiros movem»** — a solda é um grupo (§3, #3).
3. ⛔⛔ **«O censo `vec_*` descreve a família»** — quatro módulos ficaram de fora, e um deles
   carregava o 5.º roteador (§4).
4. ⛔⛔ **`ANCHOR_NAME = "IK Target"`** — escrevi-o **de memória**; o valor real é **`"IK Goal"`**.
   O compilador só o apanhou porque a const ficou **ausente**: com o nome certo e o valor errado,
   **nada** teria falhado e todo objecto que o gesto cria mudava de nome em silêncio. *Uma
   assinatura e um literal lêem-se do ficheiro.* (O mesmo dia deu a irmã: `governed(…, chain: u16)`
   quando é `u32`, apanhada em três sítios.)
5. ⛔ **«A agulha do marquee vai para onde o ficheiro foi»** — o módulo `vec_marquee` saiu para a
   crate, mas o **SUJEITO** do gate (`marquee_shape_for_press`, um `impl App` que lê
   `vec_draw_config`) ficou na shell. A minha correcção compilou e **reprovou a correr** — a mesma
   forma do gate original. *Re-apontar uma agulha exige perguntar onde ficou o SUJEITO dela.*
6. ⛔ **«O `skeleton_live` pode ir para a `ph2d-skeleton-ecs`»** — ciclo (§2).
7. ⚠️ **Uma reescrita por NOME quase destruiu PROSA:** o `ph2d_app_vec::` → `crate::` acertou num
   comentário do `lib.rs` que descreve o que os ficheiros **de fora** escrevem. Reposto.

---

## §9 — A PROVA (§3 do HOWTO)

| item | resultado |
|---|---|
| **(a) nenhum teste se perde** | **`ONLY-A = 0`**. `22 665 → 22 666`; **100 `MOVED`**. O único `ONLY-B` é `the_dialog_offers_the_svg_line`, criado de propósito ao partir um gate por SUJEITO (§10) |
| **(b) roteadores iguais** | **108 → 108**, `diff` vazio (shell + todas as crates, dos dois lados) |
| **(c) a shell encolheu** | **398 037 → 390 646** LOC · 1 561 → 1 538 ficheiros |
| **(d) `--test it` À PARTE** (regra 2) | **816 passed · 0 failed** |
| **(e) `ph2d-app-registry-init`** | **5/5** — e `"vec"` fora da catraca |
| **(f) fmt · typos · clippy** | `fmt --check` ✓ · `typos` ✓ · clippy `--all-targets --all-features` nas 5 crates ✓ |
| **(g) `collision-surface.sh`** | **ZERO** contador partilhado se moveu: `PROJECT_SCHEMA` 128 = base · `FLIP_SCHEMA` 13 = base · `DOC_VERSION` 18 = base · os dois espelhos de registo 86 = base · contrato congelado **intocado** · **nenhum** ADR · zero marcadores de conflito · nenhum teto de LOC estourado. Os 2 «pacotes novos» do Cargo.lock são as **minhas** crates internas |

⚠️ **Sobre o relógio:** não entrego um `--timings`. A razão é aritmética — **−1,9 %** da shell,
sobre os `16,8 s` de front-end que a auditoria mediu, é **~0,3 s**, abaixo da variância entre
corridas — e a máquina esteve a `load 9–77` durante a jornada, o que pela regra do `CLAUDE.md`
§5.0 **anula** qualquer leitura. *A LOC é exacta e independente de carga.*

---

## §10 — ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **As quatro portas da shell que «não fazem nada»** (`build_smoke::shape`, `fx_live::set_filter`,
   `bone_gesture::create`, `image_import::*`) são **delegações de propósito**: elas mantêm ~60
   sítios de chamada de famílias fechadas byte a byte iguais. Apagá-las é a wave seguinte.
2. **`vec_app_bridge.rs` NÃO é um pedido de sexto método de `AppHost`.** É o HOWTO §1.5 à letra:
   quem precisa de um método por campo precisa é de **tirar o campo da `App`** — e o
   `vec_draw_config` é campo desta família, com sítio marcado no `VecState`.
3. **O `ONLY-B = 1` não é um teste inventado**: é metade de um gate que tinha dois sujeitos (a lei
   do SVG, que foi com o importador; e o que o **diálogo** oferece, que é da shell).
4. **O `skin_live.rs` não se chama assim por gosto** — o censo do `every_host_that_rewrites_verts`
   filtra por `*_live.rs`, e a política nomeia-o (§6).
5. **Os campos `vec_bone_smoke_{img,pend}` não «sumiram» da `App`** — mudaram-se para o `VecState`,
   ao lado do `bone_smoke_step` que sempre lhes pertenceu.
6. **O `probe_the_smoke_sequence` não foi apagado** — voltou para a shell porque o sujeito dele (a
   sequência, que inclui uma POSE) ficou lá.
7. **O piso do gate de `verts` subiu de 5 para 6 e isso NÃO é afrouxar** — é a população real, que
   estava a ser contada errada (§6).

---

## §11 — O que a FASE C herda, em ordem de valor

1. ⭐⭐ **Ainda movem HOJE, sem curar nada: 15 ficheiros / 3 404 LOC** — entre eles as quatro
   pontes de smoke (agora finas) e o `vec_bucket`/`vec_svg_export`, que pedem o corte
   *lei pura ↔ ponte `impl App`* que esta volta fez em seis ficheiros.
2. **Os 30 bloqueadores que sobram**, por número de pedintes: `app_state` (5) · `bool_live` (4) ·
   `instance_sync` (3) · `render_loop/mod` (3) · `fx_bridge`, `offset_live`, `instance_docs`,
   `instance_verbs`, `input_dispatch`, `profile_live`, `texture_pattern_live`, `envelope_live` (2
   cada). ⚠️ A maioria é da **3.ª rodada de famílias** (as `instance_*`, as `*_live` de outras
   famílias) — **não** é trabalho desta linha.
3. ⏸️ **A família do ESQUELETO sair da shell** — `bone_pose` · `bone_limit` · `bone_pick` ·
   `bone_gesture` · `skeleton_goal` (≈1 800 LOC, dois `impl App` pequenos). Esta volta tirou a
   **lei pura** que a `vec` precisava; o resto é uma wave coerente e com dono próprio.
4. ⏳ Os **14** testes de integração-da-shell da família re-parentados (medido na Fase B: +3 194 LOC).

---

## §12 — O smoke

⚠️ **A extracção não muda produto** — o smoke confirma a **INÉRCIA**.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

As outras quatro, o mesmo comando trocando a variável: `PH2D_VEC_STACK_SMOKE=1` ·
`PH2D_VEC_APPEARANCE_SMOKE=1` · `PH2D_VEC_FADE_SMOKE=1` · `PH2D_VEC_SVG_SMOKE=1`.

⚠️ **O caminho é o da WORKTREE desta linha**, não o do primário.
