# HANDOFF DE INTEGRAÇÃO — `line/components`, W2 **FASE D**, 2026-09-12

> **A linha FECHA aqui e PARA** ([`CLAUDE.md §0.7`](../../../CLAUDE.md)). Integrar e shipar são
> ordem explícita do Enio, por um **agente integrador dedicado** (DIRETRIZ §1.5.3–1.5.4).

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/components` |
| HEAD | `1e358e69f` |
| merge-base com `main` | `67dca9411` |
| commits | **4** |
| ficheiros | **120** (`+2 875 / −2 019`) |
| rebase de abertura | **356 commits, ZERO próprios** (fast-forward) |
| handoff anterior | [`…2026-09-10`](HANDOFF_INTEGRACAO_line_components_2026-09-10.md) |

**A obra:** nasce `crates/ph2d-app-components` com **65 ficheiros / 17 677 LOC**, e a shell desce de
**191 091 → 174 013** linhas (`−17 078`) e de **678 → 620** ficheiros.

⚠️ **A medida da catraca é outra** (ela inclui `shells/desktop/tests/`): **208 442** linhas contra o
`TETO_LOC` de `229 394`.

---

## §2 — A PROVA (HOWTO §3)

```
nextest-list-diff        ONLY-A = 0   ·   ONLY-B = 1   ·   205 MOVED
                         (antes 22 668 testes → depois 22 669)
cargo test --test it     812 / 812                    ⚠️ à parte, regra 2
cargo test -p ph2d-app-registry-init   5 / 5
clippy -p ph2d-app-components --all-targets            ZERO avisos
collision-surface.sh     zero contadores partilhados a mexer-se
```

O único `ONLY-B` é o gate novo do registo (`o_registo_das_fixturas_e_o_do_produto`), nomeado no §5.

⚠️ **O portão batched (`nextest-impacted.sh`) correu a `load 63`** — ver o §9 sobre a família de
flakes de recurso antes de olhar para este diff.

---

## §3 — Símbolos que podem COLIDIR (`collision-surface.sh`, HEAD `1e358e69f`)

```
▸ SCHEMAS
    PROJECT_SCHEMA          128   (base: 128)     ⇒ delta 0
      └ tripla do gate  (128, 13, 22)  (base: idem)
    FLIP_SCHEMA              13   (base: 13)      ⇒ delta 0
    DOC_VERSION (timeline)   18   (base: 18)      ⇒ delta 0
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)    86   (base: 86)      ⇒ delta 0
    ph2d-script (espelho)    86   (base: 86)      ⇒ delta 0
▸ CONTRATO CONGELADO (§6)   intocado
▸ ADR                        esta linha não cria nenhum
▸ Cargo.lock                 1 '+name': `ph2d-app-components` (aresta INTERNA)
▸ MARCADORES DE CONFLITO     nenhum
▸ TETOS DE LOC               nenhum ficheiro da linha passa do teto
```

⭐ **Zero contadores partilhados movidos** — é o perfil correcto de uma linha que só muda endereços
(HOWTO §4: *mover código não muda serialização; se eles se mexeram, a linha fez mais do que a
tarefa*).

**Símbolos novos:** a crate `ph2d_app_components` e o `ph2d_app_components::scene_ctx::SceneCtx`.
⚠️ **35 itens passaram de `pub(crate)` a `pub`** — a superfície que a shell nomeia, aberta uma a
uma pelo compilador e não em bloco.

---

## §4 — ⭐ O que a MEDIÇÃO corrigiu no briefing (leia antes do diff)

O bloco de reabertura foi derivado de um **censo por prefixo**, e a regra 5 da própria Fase D diz
porque isso erra. Corrido o `scripts/fecho-da-familia.py` (autoteste 6/6 primeiro):

| o bloco dizia | a medição diz |
|---|---|
| `instance_*` + `component_*` = 53 f / 15 167 L | **62 f / 17 665 L** — falta o `instantiate*`, os roteadores e o `master_editing*` |
| *«~7 roteadores»*, com `PH2D_SIGNAL_TABLE_SMOKE` | **CINCO**, e aquele **não existe** (ver §5) |
| *«a armadilha é o SNAPSHOT e o UNDO»* | ⛔ **falso** — o `ProjectState::capture` **nunca apareceu como âncora** |
| — | ⭐ o bloqueador real é o `init.rs::build_component_registry`: **91 % do fecho** |

⭐⭐ **O fecho, em números:** sem curar nada movem **8 ficheiros / 1 431 LOC**; curando só o
registo, **12**; curando as cinco âncoras, **56 / 16 014**. *Contar citações teria dito «53
ficheiros movem» e a resposta era 8.*

---

## §5 — ⛔⛔ A âncora que valia 91 %, e a recusa que a mediu

`init.rs::build_component_registry` tinha **33 usos em 29 ficheiros, TODOS testes** — zero produto.
Ela não pode viajar: regista componentes de **cinco crates irmãs**, logo é composição e fica na
shell por desenho (ESTADO §3), e **uma crate nunca pode chamar o `bin`**.

**O precedente na árvore** é a `ph2d-app-physics`: seis ficheiros de teste dela compõem o registo
com o subconjunto que cada um usa. ⛔ **Aqui isso está REFUTADO por medição** — as fixturas desta
família usam `ph2d_render::Sprite` (**54×**), `ph2d_physics_ecs::{Collider, RigidBody,
ColliderShape, PlatformPlayer}` (**23 f**) e `ph2d_field_ecs::FieldObject`, que são exactamente os
componentes cujo **descarte silencioso** estes gates existem para apanhar.

⇒ [`component_registry_for_tests::registo`] faz **as mesmas cinco chamadas, na mesma ordem** —
incluindo o `ph2d_skeleton_ecs`, que **nenhum ficheiro desta família usa**. Custa uma linha de
`Cargo.toml` e **remove a lista de isenções inteira**: sem isenções não há catraca para apodrecer
(§5.0), e o gate exige **igualdade exacta**.

### ⛔⛔ E a 1.ª redacção desse gate era DECORATIVA — só a mutação o disse

Ela comparava o `init.rs` com uma **constante escrita à mão** (`AS_MINHAS`). Apagar a linha do
`ph2d_skeleton_ecs` de `registo()` deixava-o **VERDE**: os dois lados que ele comparava continuavam
a ter cinco, e **nenhum deles era a função**. *Uma lista escrita à mão ao lado da coisa que ela
descreve é uma TERCEIRA resposta à mesma pergunta, e é sempre a que não é executada.*

Hoje os dois lados são extraídos **do corpo das duas funções** pelo mesmo extractor, com piso de
população (`>= 4` de cada lado). Mutação verificada ⇒ **RED**, com as duas listas impressas.

---

## §6 — ⭐ CINCO roteadores, e o bloco listava QUATRO

Contados por `var(_os)?("PH2D_…")` — que é o que uma env **LIDA** parece, ao contrário de uma
citada em prosa (HOWTO §2.12):

| roteador | `max_level` | contado em |
|---|---:|---|
| `PH2D_AUDIO_2D_SMOKE` | 1 | interruptor `is_none()` |
| `PH2D_GAME_CAMERA_SMOKE` | 1 | idem — ⚠️ lido por `camera_2d_smoke.rs`: **o nome do ficheiro mente** |
| **`PH2D_INSTANCE_SMOKE`** | **7** | `match` com os braços `"1"`..`"7"` |
| `PH2D_SIGNAL_ACTION_SMOKE` | 1 | interruptor |
| `PH2D_TIMER_SMOKE` | 1 | interruptor |

- ⛔ **`PH2D_SIGNAL_TABLE_SMOKE` NÃO EXISTE** — nenhum ficheiro do repo o lê; o
  `signal_table_smoke.rs` é um nível do `PH2D_BUILD_SMOKE` (`=68`), que fica na shell.
- ⛔ **`signal_smoke.rs` não é desta família** — é a cena do R0/timeline (ADR-0143), e o próprio
  ficheiro abre a avisar: *«Não confundir com o `crate::signal_smoke`»*.
- ⭐⭐ **E o `PH2D_INSTANCE_SMOKE` não estava em lista NENHUMA** — nem nos `Smokes:` do `CLAUDE.md`
  §5, nem no handoff de 10/09 —, porque aquelas listas foram escritas **por wave** e ele é anterior
  a todas elas. *Uma lista de roteadores mantida a cada jornada descreve as jornadas, não a
  família.*
- ⚠️ `PH2D_INSTANCE_LOG` é **diagnóstico** e não entra.

---

## §7 — O corte: `SceneCtx`, e ZERO sextos métodos no `AppHost`

Os **13** `impl crate::App` viraram funções livres sobre um [`scene_ctx::SceneCtx`] de 8 campos —
o molde é o `MotionSceneCtx` da `line/app-motion`. Escrito em **tipos**, o que uma cena pede é o
mundo, a cena vectorial, o registo e o relógio: **tipos de crates irmãs**, não perguntas à shell.

⚠️ **O prólogo FICA** (`shells/desktop/src/components_scenes.rs`, 6 métodos): os três guardas de
arme (*já corri? · a env está posta? · o mundo já subiu?*) e a construção do contexto. *O que sai
são os CORPOS; o que decide a ordem do quadro fica.* ⛔ Um latch dentro da crate seria ela a ter
opinião sobre **quando** o quadro a chama.

⭐ **E o `sync_instances` mostrou que a lei já era pura:** `sync_instances(sim, registry, bridge,
echo, docs)` vive na crate desde sempre, e o `impl App` era só **quem lhe entrega os cinco**. Essa
ponte foi inteira para a shell — é a fronteira *lei pura ↔ ponte* que o bloco pedia, encontrada
onde ela de facto estava.

⚠️ **Os 7 campos de `App` desta família FICAM lá**, e a decisão é dupla: quatro são latches que só
a crate lê (moveriam), mas o `game_camera_smoke_done` **é lido pelo `render_loop`** e o
`game_camera_preview`/`instance_echo` são estado que a shell possui. Mover uns e não outros faria
o prólogo perguntar em dois sítios diferentes.

---

## §8 — ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **Os ficheiros MANTIVERAM o prefixo, e isso não é o HOWTO §1.3 ignorado.** Ali o prefixo **era o
   nome da família** (`field3d_gizmo.rs` → `gizmo.rs`); aqui a família é `components` e
   `instance_`/`component_` são **dois assuntos dentro dela**. ⛔ E despi-los **COLIDE**:
   `instance_smoke.rs` e `component_smoke.rs` reduzem-se ambos a `smoke.rs`. ⭐ O efeito é que a
   reescrita **interna** foi zero, e as armadilhas §2.2/§2.3 não tiveram onde morder.
2. **`ph2d-field-ecs` e `ph2d-skeleton-ecs` no `Cargo.toml` não são deps a mais.** Nenhum ficheiro
   da família nomeia um tipo delas — elas entram **pelo registo**, e tirá-las faz as fixturas
   medirem um catálogo mais pobre que o do produto, em silêncio.
3. **A crate depende da `ph2d-app-physics`, e é legítimo** — uma crate-motor irmã não é a shell
   (ADR-0075). O `component_seed` semeia um `Collider` e um `PlatformPlayer` pelas portas dela.
4. **`components_ctx` devolve `Option` e não entra em pânico** — no arranque ainda não há `gfx`, e
   a cena tenta no quadro seguinte. Era assim antes, dentro de cada método.
5. **O `render_loop` perdeu duas re-exportações e isso é a CURA** — `seed_attached_{collider,player}`
   eram `pub(crate) use ph2d_app_physics::…`, com o comentário a dizer *«esta linha é só o
   ENDEREÇO»*. O único consumidor era a tabela `SEEDS`. *O `render_loop` era, ali, uma FACHADA.*
6. **`mod hierarchy` deixou de ser `pub(crate)`, e a razão saiu com a função.** O comentário
   justificava-o pelo `refuses_reparent`, que se mudou para o `instance_verbs_walk` (junto da lei
   que ele compõe). ⛔ Rust **não avisa** de visibilidade a mais: isto apodreceria calado.
7. **Os dois `*_seam_tests.rs` na shell não são dívida** — são a costura, e o nome segue o
   precedente que a `vec`/`skeleton` já deixaram (`morph_arrow_seam_tests`,
   `skeleton_shell_seam_tests`). Cada um diz no cabeçalho **quando volta**.

---

## §9 — ⛔⛔ Os 12 gates de `tests/it/` que reprovaram, e as TRÊS espécies

⚠️⚠️ **O `cargo check --all-targets` estava VERDE com 12 gates vermelhos** — regra 2: o
`nextest-impacted` filtra `shells/desktop/tests/it/`. É a diferença entre 0 e 12.

### (a) ENDEREÇO — 7 gates, falha ALTA

`read_to_string` de ficheiros que se mudaram. Reendereçados para
`../../../crates/ph2d-app-components/src/`. ⚠️ **TRÊS `..`**: de `shells/desktop/src`, dois sobem
só até `shells/`, e a 1.ª tentativa parou em dois — a mensagem de erro imprimia
`shells/crates/…`, que é o que a torna barata.

### (b) ⛔⛔ A AGULHA QUE É DADO, NÃO CAMINHO — 2 gates

A reescrita em massa `crate::X` → `ph2d_app_components::X` correu sobre `shells/desktop/**`, o
`tests/it/` incluído — e ali dentro essas cadeias são **dados sobre OUTRO ficheiro**. Das quatro
agulhas afectadas:

| agulha | o ficheiro que ela MEDE | veredito |
|---|---|---|
| `the_apply_ladder:58` | `render_loop/mod.rs` (**shell**) | ✅ certa — a shell passou mesmo a escrever o caminho novo |
| `the_recipe_mark:66` | `render_loop/hierarchy_duplicate.rs` (**shell**) | ✅ certa |
| `the_apply_ladder:75` | `instance_verbs.rs` (**crate**) | ⛔ errada — lá dentro é `crate::` |
| `the_apply_ladder:101` | `instance_apply_deep.rs` (**crate**) | ⛔ errada |

*É a §2.12 ao contrário: ali um censo lê PROSA como código; aqui uma reescrita escreveu DADOS como
código.* As duas erradas reprovaram alto.

### (c) ⛔⛔ A JANELA DE BYTES FIXA — 1 gate

`&body[arm..arm + 1600]`. Alongar identificadores em **17 caracteres** empurrou os dois
`Toast::warning` do braço para fora da janela, e o gate reprovou sobre produto **correcto**.

⛔ **Alargar o número seria PIOR:** medido, o braço tem `1816` bytes com os toasts a `1498`/`1727`
— e há **outro par a `3573`/`3816`, de um braço vizinho**. Uma janela de `4000` ficaria verde a
contar os toasts de outra pessoa. ⇒ a fatia passa a ser **o braço**, fechado na chaveta dele.

*Um número de bytes é uma agulha cujo sentido depende do comprimento dos nomes.*

### ⛔⛔ E a §2.13 mordeu-me DUAS vezes no MESMO dia

A agulha do `reordering_between_siblings_is_still_allowed` dizia `pub(crate) fn refuses_reparent(`
— **e fui eu que a re-ancorei nessa forma, horas antes, no commit `b62c842f1`**, depois de ler a
§2.13 que diz exactamente isto. É a 5.ª ocorrência no repo e a 1.ª em que o mesmo agente a paga
duas vezes no mesmo dia. *Visibilidade é precisamente o que uma fronteira nova muda por construção.*

---

## §10 — ⛔⛔ O censo MUDO: um piso sobre a ÁRVORE não é um piso sobre o SUJEITO

O `the_fallback_name_of_an_unnamed_recipe_is_not_an_old_word` varre `shells/desktop/src` inteira e
**já tinha piso** (`files.len() > 100`), com o doc a explicar porque a população é derivada.

Ele ficou **VERDE** depois da mudança — e a medir menos: dos `24` fallbacks que existe para ler,
**3 mudaram-se para a crate**, e ele passou a ver `21`. ⛔ **O piso não o apanha porque conta
FICHEIROS DA SHELL**, que continuam a ser centenas.

⇒ a varredura passa a nomear **as duas casas do sujeito**, e o censo fica **mais forte do que era
antes da mudança** (HOWTO §2.7).

⚠️⚠️ **E a 1.ª guarda que escrevi para isso era DECORATIVA:** somava as duas árvores e exigia
`>= 20` sobre uma população de `24` com `21` numa delas — a mutação que apaga a crate da varredura
**SOBREVIVEU**. *Uma folga é um ponto cego com o tamanho exacto da folga.* Hoje o piso é **por
raiz** (`>= 15` na shell, `>= 1` na família) e a mesma mutação **mata-o**.

⭐ **Duas guardas minhas nasceram decorativas nesta jornada** (esta e a do §5), e as duas só
falaram sob mutação. *Um gate que nunca se viu VERMELHO não afirma nada.*

---

## §11 — As duas COSTURAS que ficam na shell, com data de validade

| módulo | gates | o que atravessa | volta quando |
|---|---:|---|---|
| `component_attach_seam_tests.rs` | 2 | `render_loop::inspector_presence_probe::slice` → um builder **privado** do `render_loop` | a família da **Sprite** sair (o 9-Slice é dela) |
| `instance_paint_seam_tests.rs` | 1 | `hero_intents::texture_rebind::rebind_to_individual` | idem |

⛔ **Elas não podiam vir, e a razão é estrutural, não de arrumação:** uma crate **nunca pode chamar
o `bin`**. O arnês está em `ph2d_app_components::test_support` (feature `test-support`), do tamanho
do que ATRAVESSA: **seis** itens.

⭐ **A sonda já estava meio migrada, e foi ela que ensinou a cura:** no mesmo ficheiro, o `slice`
aponta para um builder privado do `render_loop` e o `physics`/`player` **já** apontam para
`ph2d_app_physics`. Foi esse par lado a lado que mostrou que o `component_seed` podia chamar a
crate directamente.

**Medido no fim:** o fecho da família na shell é **2 ficheiros / 232 LOC**, cada um preso por
exactamente uma destas duas âncoras.

---

## §12 — ⚠️ O que só o `ship.sh` apanha, e o que JÁ está vermelho

- ⛔ **A catraca `the_shell_only_shrinks` reprova pela metade de OBSOLESCÊNCIA**: a shell tem
  **208 442** linhas contra um tecto de `229 394` (folga de `20 952`). ⚠️ **É esperado** (regra 1),
  e o número é do **INTEGRADOR**, sobre a árvore combinada e **depois** do `cargo fmt --all`.
  ⛔ Esta linha não lhe tocou.
- **Clippy:** `ph2d-app-components` a **ZERO**. O que sobra na corrida é **todo pré-existente e
  fora deste diff** (`git diff main --name-only` não toca nenhum destes ficheiros):

  | # | onde |
  |---:|---|
  | 18 + 1 | `crates/ph2d-app-motion/*` (`needless_borrow` · `Default`) |
  | 1 | `crates/ph2d-app-sculpt3d/src/keys.rs` — o bloqueador que o ESTADO §6 nomeia |
  | 1 | `shells/desktop/tests/sculpt_source/mod.rs` |
  | 1 | `crates/ph2d-preview-drive/src/lib.rs` (`len` sem `is_empty`) |

  ⚠️⚠️ **Os dois últimos contradizem o ESTADO §6**, que os dá por dissolvidos (*«clippy limpo nas
  duas crates, verificado»*). Re-medidos hoje, **existem**. *Uma lista de bloqueadores envelhece
  mais depressa do que se pensa* — é o que aquela mesma secção avisa sobre si própria.
  ⛔ **Não os curei**: o `sculpt_source` foi tocado pelo `clippy --fix` e **revertido** por não ser
  território desta linha (regra 9).
- **Zero pacotes externos novos** no `Cargo.lock` ⇒ nada para o `machete`/`deny`/`audit`.
- **Ficheiros novos** que o `typos` nunca leu: os 5 desta linha (`scene_ctx.rs`, `test_support.rs`,
  `component_registry_for_tests.rs`, `components_scenes.rs`, os dois `*_seam_tests.rs`).

---

## §13 — ⏳ O que fica ABERTO

1. ⭐⭐ **`asset_*` · `prefab_stage*` · `variant_*_smoke` · `nest_smoke` são família por ASSUNTO e
   NÃO vieram** — **18 f / 6 244 L**. O bloco não os viu porque mediu prefixo (ADR-0165 é o índice
   de assets, e as variantes são a F5). ⛔ **Medidos, eles não são de graça:** arrastam âncoras
   novas — `render_loop/sim_extract.rs` (1 155 L) e `image_import.rs` —, que são da **Sprite**.
   ⇒ **5.ª rodada, junto com ela**, não esta.
2. **Os 7 campos de `App` desta família** continuam lá (§7). Tirá-los pede que o `render_loop`
   deixe de ler o `game_camera_smoke_done`.
3. **As duas costuras do §11**, com o gatilho escrito no cabeçalho de cada uma.
4. ⚠️ **O `PH2D_INSTANCE_SMOKE` não está nos `Smokes:` do `CLAUDE.md` §5** — descoberto por este
   corte. A linha do §5 no §14 já o inclui.
5. **Tudo o que o handoff de 10/09 deixou aberto** continua aberto: esta jornada **não tocou em
   produto** — é mudança de endereço, e o `ONLY-A = 0` afirma-o.

---

## §14 — A linha do `CLAUDE.md §5`, PRONTA A COLAR

⚠️ **Esta linha NÃO edita o `CLAUDE.md`** — o próprio §5 escreve que *«ele só se edita na
integração»*, e três linhas a fechar no mesmo dia colidiriam ali. O integrador cola isto na entrada
**Componentes / instâncias**:

> ⭐⭐ **E A FAMÍLIA SAIU DA SHELL** (12/09, W2 Fase D): os **62** ficheiros (`instance_*` ·
> `component_*` · a porta `instantiate*` · os 4 roteadores · o `master_editing*`) vivem em
> [`ph2d-app-components`](crates/ph2d-app-components/), e a shell desce de **191 091 para 174 013**
> linhas. ⭐ **O bloqueador valia 91 % do fecho e não era o que o briefing dizia:** não é o
> `ProjectState::capture` (que nunca apareceu como âncora — o corte *lei ↔ ponte* já estava feito),
> é o `init.rs::build_component_registry`, com **33 usos em 29 ficheiros, TODOS testes**. ⛔ Ele não
> viaja (regista cinco crates irmãs ⇒ é composição), e a cura **não** é o precedente da física (cada
> teste compõe o subconjunto que usa): medido, as fixturas desta família usam `Sprite` (54×),
> `Collider`/`RigidBody` (23 f) e `FieldObject` — *exactamente os componentes cujo descarte
> silencioso estes gates existem para apanhar* ⇒ o registo das fixturas faz **as mesmas cinco
> chamadas** e um gate exige **igualdade exacta**, sem isenções. ⭐⭐ **São CINCO roteadores e o
> briefing listava QUATRO:** o `PH2D_INSTANCE_SMOKE` (`max_level 7`) **não estava em lista nenhuma**
> — nem aqui, nem no handoff de 10/09 — porque as listas foram escritas *por wave* e ele é anterior
> a todas; e o `PH2D_SIGNAL_TABLE_SMOKE` que o bloco nomeava **não existe**. ⛔⛔ **E o
> `cargo check --all-targets` estava VERDE com 12 gates vermelhos em `tests/it/`**, em três espécies
> — endereço (7, falha alta) · **a agulha que é DADO e não caminho** (2: a reescrita em massa mudou
> cadeias que descrevem OUTRO ficheiro; duas das quatro ficaram certas por medirem a shell) · e **a
> janela de BYTES fixa** (1: alongar nomes em 17 caracteres empurrou dois toasts para fora de
> `1600`, e alargar o número apanharia os toasts do braço vizinho ⇒ a fatia passa a ser o braço).
> ⚠️ **E um censo com piso ficou MUDO na mesma:** o `> 100` conta ficheiros da SHELL e sobrevive a
> perder 3 dos 24 fallbacks para a crate — *um piso sobre a árvore não é um piso sobre o sujeito*.
> ⭐ Prova exacta (`ONLY-A = 0`, 205 `MOVED`, `--test it` 812/812, zero contadores partilhados) e as
> **sete** leituras que o diff inverte:
> [handoff de 12/09](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_app_components_2026-09-12.md).
> **Smoke novo no roteador:** `PH2D_INSTANCE_SMOKE=1..7`.

---

## §15 — Portão de fecho (batched, 1× sobre o diff acumulado)

| passo | resultado |
|---|---|
| `cargo fmt --all` | árvore limpa (31 ficheiros) |
| `cargo clippy -p ph2d-app-components --all-targets` | **ZERO** |
| `cargo check -p ph2d-host-desktop --all-targets` | verde, zero avisos |
| **`cargo test -p ph2d-host-desktop --test it`** (regra 2, à parte) | ⭐ **812 / 812** |
| `cargo test -p ph2d-app-registry-init` | 5 / 5 |
| `nextest-list-diff` | `ONLY-A = 0` · `ONLY-B = 1` · 205 `MOVED` |
| `collision-surface.sh` | zero contadores partilhados |
| `scripts/nextest-impacted.sh` | ver §16 |
| `target/*/incremental` | reclamado (DIRETRIZ §1.5.9 item 7) |

---

## §16 — `scripts/nextest-impacted.sh` sobre o diff acumulado

```
Summary [135.508s]  16 246 tests run:  16 245 passed (1 slow), 1 failed, 8 686 skipped
   FAIL  ph2d-editor-core::it architecture_the_shell_only_shrinks::the_shell_only_shrinks
```

⭐ **A única reprovada é a catraca da shell, pela metade de OBSOLESCÊNCIA** — ver o §12. É esperada
(regra 1), e o número é do integrador.

⚠️ **A corrida foi feita a `load 63`** (pico do fan-out de 16 246). Isso é relevante para a família
de flakes de recurso (`CLAUDE.md` §5.0) — e **nenhum membro dela reprovou**: passaram todos, e os
últimos catorze da corrida são precisamente eles (`a_round_live_offset_costs_like_the_other_joins`,
`measure_normals_parallel_speedup`, `the_cost_of_depth_is_linear_not_explosive`,
`o_pen_down_do_filtro_e_linear_nos_vertices`, `apply_from_doc_is_zero_alloc_steady_state`, …).
*A nota fica porque a ausência de flakes numa corrida carregada é informação, não silêncio.*
