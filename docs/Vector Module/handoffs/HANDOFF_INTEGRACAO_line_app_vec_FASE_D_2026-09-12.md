# HANDOFF DE INTEGRAÇÃO — `line/app-vec`, W2 **FASE D** (4.ª rodada) · 2026-09-12

> **Para o agente integrador e para a próxima janela desta linha.** Molde: DIRETRIZ §1.5.9.

| | |
|---|---|
| ramo | `line/app-vec` · merge-base `67dca9411` · HEAD `10075386a` · **7 commits** |
| ficheiros tocados | **59** — e **só** `crates/ph2d-app-vec`, `shells/desktop/{src,tests}` e o `Cargo.lock` |
| shell `shells/desktop` | **225 394 → 213 564** LOC (−11 830, **−5,25 %**) · 917 → **879** ficheiros |
| [`ph2d-app-vec`](../../../crates/ph2d-app-vec/) | 119 → **159** ficheiros · **43 550** LOC |
| o meu território | 125 f / 36 199 L → **88 f / 24 459 L** |
| contadores partilhados | **zero** mexidos · contrato **intocado** · **zero** ADR · **zero** pacotes externos novos |
| ⭐ a catraca `the_shell_only_shrinks` | **VERDE** — o corte coube na folga de obsolescência |

---

## §1 — O que saiu

| commit | o que | LOC |
|---|---|---:|
| `74cecce2c` | o cluster **`fx_live`** + o `stroke_paint` | 3 113 |
| `4edd44d22` | os **ESTADOS DE UI** e o **BOOLEANO** (24 f) | 6 674 |
| `1214725b3` | a **PONTE do vetor** (`render_loop/vector_bridge*`, 6 f) | 2 223 |
| `9c8f9d232` | quatro ficheiros deixam de nomear o LAÇO | **0** — e o commit existe por isso |
| `852eb7809` | ⛔ os **três gates que evaporaram** + o censo que os teria apanhado | — |

⭐ `render_loop/vector*` está a **zero**. O `bool_*` foi de **10 f / 2 458 L** para **1 f / 215 L**.

---

## §2 — ⭐⭐ A PREVISÃO DA FASE C CUMPRIU-SE À LETRA

O §12 daquele handoff dizia: *«quando a `motion` levar `brush_live` + `texture_pattern_live`
(632 L), esta família liberta ~1 900 L — zero trabalho novo»*. A `motion` levou-os para a folha
[`ph2d-vec-art-live`](../../../crates/ph2d-vec-art-live/), e o cluster `fx_live` saiu **inteiro**
(2 178 L) sem uma linha de trabalho deste lado, mais o `vec_stroke_paint` (935 L).

⇒ *uma dependência entre linhas, escrita com o nome do ficheiro no handoff de quem a devia, paga-se
sozinha na rodada seguinte.* É a lei 7 do ESTADO a funcionar na direcção boa.

---

## §3 — ⛔⛔ O ACHADO: **UM ficheiro de 272 linhas prendia 6 674**

O bloco mandava re-medir o fecho antes de mover. Medido com `--curavel`, **antes de tocar em nada**:

| curar… | move |
|---|---:|
| nada | 18 f / 4 186 L |
| `corner_handles` | **18 f** — *nada: não está no caminho* |
| `build_smoke` | 20 f / 2 817 L |
| `app_state` | 28 f / 7 478 L |
| ⭐ **`render_loop/ui_state_bridge`** | **33 f / 7 505 L** — e ele tem **272 linhas** |
| os quatro juntos | 68 f / 18 160 L |

⇒ o contrafactual escolheu o alvo, e a diferença entre a melhor e a pior opção é de **7×**.
*Medir qual cura compra o quê é mais barato do que curar a errada.*

---

## §4 — ⭐⭐⭐ A **FACHADA**, três vezes e em três formas diferentes

A Fase C achou que cinco das seis «âncoras» do esqueleto eram **fachadas de MÓDULO**. Esta fase
achou as outras duas formas:

| forma | exemplo | o que prendia |
|---|---|---:|
| de **MÓDULO** (Fase C) | `crate::skeleton_live` → `ph2d_skeleton_live::skin_live` | 5 das 6 âncoras |
| de **SÍMBOLO** | `crate::vec_snap::VecSnapSettings` → `pub(crate) use ph2d_app_vec::snap::{…}` | **6 f / 2 223 L** |
| de **ALIAS que eu próprio escrevi** | `crate::render_loop::ui_state_bridge` → o alias do commit anterior | 5 f, e uma âncora **falsa** |

⛔ **A régua do fecho conta as três como shell**, porque ela resolve caminhos de módulo
textualmente e só reconhece alias declarado no `main.rs` — é a primeira coisa que ela imprime.
Medido: **14 ficheiros da shell são fachadas de símbolo hoje.**

⭐ **E ela erra sempre no sentido CONSERVADOR** — diz que move MENOS. *Uma régua que subestima não
autoriza nada: ela desiste cedo, e por isso passa despercebida* (o erro famoso desta wave, o `30×`
da `motion`, é no sentido oposto e foi apanhado no dia).

⚠️⚠️ **E a lição é sobre o ALIAS, que é uma ferramenta desta linha:** ele existe para o corte não
tocar em sítios de chamada, e paga-se com uma âncora falsa no fecho. *A mesma linha que mantém o
diff pequeno faz a shell parecer acoplada.*

---

## §5 — ⛔⛔⛔ TRÊS GATES EVAPORARAM-SE NUM `git mv`, E **NADA** ACUSOU

O `bool_reach_tests.rs` era um `#[cfg(test)] mod` de topo do `main.rs`. Mudou-se para a crate, a
declaração foi apagada de lá e **nenhuma foi escrita aqui**. Um ficheiro `.rs` que nenhum `mod`
declara **não é compilado**.

| instrumento | veredito |
|---|---|
| `cargo check -p ph2d-app-vec --all-targets` | **verde** |
| `cargo check -p ph2d-host-desktop --all-targets` | **verde** |
| `cargo clippy` | **verde** |
| a suíte da crate | **verde** — com três testes a menos |
| `cargo test --test it` | **verde** |
| ⭐ **o `ONLY-A` da prova de perda** | **acusou os três** |

> *Não há nada para verificar num ficheiro que não entra no build.* É literalmente para isto que
> aquele número existe — e é a razão de o fecho de uma linha **não** poder ser *«as suítes passam»*.

⇒ escrevi o censo que o teria apanhado:
[`every_file_in_this_crate_is_declared_by_some_mod`](../../../crates/ph2d-app-vec/src/no_orphan_module_tests.rs),
com **piso de população** (≥ 100 ficheiros) e um **controlo positivo** que prova que o scanner vê as
**duas** formas de declaração. ⚠️ A segunda é a que importa: esta crate declara quase todos os
irmãos de teste por `#[path]`, e um censo que só lesse `mod x;` acusaria dezenas de ficheiros
legítimos — *um censo que acusa o legítimo morre por descrédito, não por estar errado.*
Provado por mutação nas duas pontas.

---

## §6 — ⚠️⚠️ A MINHA PRÓPRIA SONDA MENTIU-ME **QUATRO** VEZES

Fica escrito porque a próxima linha vai reconstruir esta prova:

| # | o furo | o efeito |
|---|---|---|
| 1 | contei `#[test]` do fonte e comparei com o `nextest list`, que **não lista `#[ignore]`** | um falso «perdido» |
| 2 | `git grep -A4` **cola linhas de ficheiros diferentes** | nomes fabricados |
| 3 | cortei a chave no último `::` da LINHA — o `nextest list` separa pacote de teste por **ESPAÇO** | **197** falsos «novos» |
| 4 | o regex `[a-z_0-9]+` corta um nome com **maiúscula** a meio (`a_pure_formula_refuses_ExpressionDriven`) | falsos «perdidos» |

⇒ **a extracção válida é a do próprio [`scripts/nextest-list-diff.py`]**: `^(\S+)\s+(\S+)$`, com a
chave a ser o último `::` do **segundo** campo. *Uma régua caseira que reimplementa uma que existe
erra nos quatro sítios em que a original já acertou.*

---

## §7 — ⛔ E um gate PROMETIA um censo de obsolescência que não existia

O `architecture_no_downcast_to_concrete_tool_in_shell` diz, por escrito, na allowlist:
*«The stale-check below ensures the allowlist only contains files with REAL downcasts»*. **Não havia
stale-check nenhum** — o ficheiro tinha UM teste, e ele só olha os ficheiros que **não** estão na
lista, logo uma entrada podre é, para ele, invisível. A entrada `src/render_loop/vector_bridge.rs`
apodreceu à vista nesta fase e nada se queixou.

⇒ escrevi a metade que faltava (reprova nas **duas** formas: a entrada que não existe e a que existe
mas já não tem downcast) + o controlo positivo de população. Provado por mutação nas duas.

⚠️ *Uma catraca sem censo de obsolescência vira LICENÇA; uma que **diz** ter o censo é pior, porque
quem lê deixa de o procurar.*

---

## §8 — As espécies de gate partido, e quem apanhou cada uma

| espécie | quantos | como falha | quem a apanhou |
|---|---:|---|---|
| `include_str!` (§2.6) | **5** | ⭐ em COMPILAÇÃO | `cargo check --all-targets` |
| o **gémeo em RUNTIME** (§2.6) | **8** | ⛔ compila, explode só ao correr | **`--test it` À PARTE** (regra 2) |
| o **censo por directório** (§2.7) | **2** | ⛔ ficaria verde a medir nada | ⭐ o **piso de população** |
| o **módulo órfão** (§5) | **1** | ⛔⛔ nada o vê | ⭐ o `ONLY-A` |
| a agulha que nomeia a **visibilidade** (§2.13) | **5** | reprova sem a lei mudar | a corrida |

⭐ **A cura dos resolvedores não foi emendar caminhos**: eles tentam as **duas árvores**, com o
`panic` a nomeá-las. *Emendar o caminho cura este movimento; resolver as duas cura o próximo.*
⚠️ E o **basename** é onde a forma muda: na shell os ficheiros moravam em `render_loop/` e dentro da
crate tudo é plano — *uma fronteira nova não preserva a pasta de onde se veio.*

⚠️ **QUARTA forma de visibilidade desta wave: `pub(super)`** (5 itens). Dentro de `render_loop/` ela
alcançava o laço; do lado de cá não alcança nada — e nenhuma varredura por `pub(crate)` a vê.

⚠️ **E a §2.1 tem uma TERCEIRA forma de `impl App`**: o piloto achou `^impl App` e `impl crate::App`;
esta família escreve **`impl crate::app_state::App`**, e o censo de ambos os anteriores não a vê.

---

## §9 — ⚠️ `cargo check -p <crate>` mede um programa que ninguém constrói

Duas vezes nesta fase um aviso apareceu por eu verificar a crate **sozinha**:

- `vector_bridge_publish.rs` tem 10+ `#[cfg(feature = "panel-vector")]`, e a feature só liga quando a
  **shell** a pede. Sozinha, a crate compila com a publicação do painel **desligada** — a §2.4, *«o
  pior modo de falha da wave»*, com a fiação correcta e o aviso na mesma.
- `ph2d-preview-drive::len` sem `is_empty` — o `is_empty` é `cfg(test | test-support)`, e no grafo da
  shell a **unificação de features** liga-o e o aviso desaparece.

⇒ **verifique a crate com as features que a shell liga**, senão o veredito descreve outro programa.

---

## §10 — A PROVA (e o que lhe falta, dito)

⛔ **Eu não capturei o baseline do `nextest list` no início desta rodada** — o da Fase C está **35
commits** atrás do meu merge-base e não serve. A prova que fica é equivalente e tem duas metades:

1. **O raio de impacto**: `git diff --name-only main..HEAD` toca **apenas**
   `crates/ph2d-app-vec` (41), `shells/desktop/tests` (9), `shells/desktop/src` (8) e o `Cargo.lock`.
   **Zero** outras crates ⇒ um teste só pode ter-se perdido aí.
2. **Nos 56 ficheiros `.rs` do merge-base que esta linha tocou**: 246 testes não-ignorados
   declarados, e **`ONLY-A` = 0** contra a listagem de hoje (com a extracção do §6 corrigida).

| | |
|---|---|
| `cargo nextest list --workspace` | **22 672** |
| `cargo test -p ph2d-host-desktop --test it` **À PARTE** | **814 / 814** |
| portão batched (`nextest-impacted.sh`) | **16 248 / 16 249** |
| `cargo fmt --all --check` | limpo |
| clippy nas crates tocadas | limpo |
| `collision-surface.sh` | todo contador na base · contrato intocado · zero ADR · zero pacote externo |
| ⭐ `the_shell_only_shrinks` | **VERDE** |

### O único vermelho é uma FLAKE NOMEADA

`measure_normals_parallel_speedup` ([`ph2d-mesh`](../../../crates/ph2d-mesh/)) — **membro nomeado**
da família de flakes de carga (`CLAUDE.md` §5.0). As três assinaturas:

- **zero** linhas do meu diff naquela crate;
- **3 de 3 verde sozinha**, a `load 16,41` · `15,33` · `15,33` — *e passar sob carga alta, sozinha, é
  mais forte do que passar na máquina calma*;
- ela já está na lista, com o mecanismo (uma razão entre dois relógios).

⛔ **Não toquei no `TETO_LOC`.**

---

## §11 — ⚠️ SEIS coisas que uma leitura rápida do diff entende ao contrário

1. **«O commit `9c8f9d232` não faz nada — zero linhas.»** Ele faz o `render_loop/mod.rs` cair de
   **12 usos / 5 ficheiros** para **1 / 1** no fecho: a régua passa a mostrar o bloqueador
   **verdadeiro**. *Uma âncora falsa custa a próxima fase inteira a perseguir o alvo errado.*
2. **«`ui_state_bridge` vivia em `render_loop/` ⇒ era do LAÇO.»** É **puro** (zero `App`, zero
   `gfx`), a única coisa da shell que ele nomeava era um ficheiro `vec_*`, e o `CLAUDE.md` §5 lista
   *«estados de UI + Smart Animate»* sob **Vector**. ⛔ O laço e os campos **ficaram**.
3. **«O `bool_smoke` e o `morph_states_smoke` ficaram por esquecimento.»** Recebem `&mut App` e leem
   `gfx` — são corpos de cena do `PH2D_BUILD_SMOKE`.
4. **«`pub(crate)` → `pub` em 117 sítios afrouxou a API.»** É o que a fronteira **obriga** (§2.13).
5. **«O gate do downcast perdeu uma entrada.»** Ganhou o **censo** que o comentário dele já prometia.
6. **«A crate tem um ficheiro de teste a mais (`no_orphan_module_tests`).»** Ele é a resposta a um
   defeito que apagou três gates em silêncio (§5).

---

## §12 — O que a FASE E herda

⭐⭐ **O tecto honesto está medido: dos 88 ficheiros / 24 459 L que sobram, TRINTA (5 776 L) tocam a
`App` ou o `gfx`** — são pontes e corpos de cena, e **ficam por desenho** (HOWTO §4). *Todo o resto
está preso atrás deles*, e é isso que a causa-raiz diz:

| nº | causa | quem a cura |
|---:|---|---|
| 10 | `nomeia build_smoke.rs` | ⛔ **não é desta linha**: o `PH2D_BUILD_SMOKE` despacha para **64 módulos** e atravessa famílias (`brush_*` é do painter, `variant_*`/`component_smoke` dos componentes). É um **roteador partilhado**, e sair daqui pede uma wave com dono próprio |
| 6 | `nomeia app_state.rs` | o corte *lei ↔ ponte* em 5 ficheiros — ⚠️ as metades de lei são **finas** (`vec_snap` tem 37 L de lei contra 209 de ponte), e o ganho medido caiu para **13 f / 1 522 L** |
| 5 + 4 | `layout_live` (nomeia · `#[path]`) | cascata interna: a raiz é o `layout_live_anchors_tests` nomear o `vec_frame_resize`, que é ponte |
| 4 | `envelope_live` | cascata interna |
| 4 | `#[path]` com `vec_gizmo_pick` | a solda que o bloco avisa |
| 3 | `#[path]` com `morph_set_world_tests` | cascata interna |

⏸️ **Agrupar os `vec_*` em `src/vec/` continua por fazer, e continua a ser a mesma decisão:** não tira
uma linha da shell e põe um diff grande no `main.rs`, que é um dos ficheiros de costura onde as três
linhas desta rodada se encontram (regra 10). ⭐ Numa rodada em que esta linha corra **sozinha**, vale
a pena — e aí a ferramenta do fecho passa a funcionar sem `--extra`.

⭐⭐ **E duas melhorias medidas no `scripts/fecho-da-familia.py` que NÃO commitei**, porque três
linhas o usam nesta rodada e um conflito ali seria meu: (a) `--fora <ficheiro>`, para tirar de `S` o
que um `--extra` largo arrastou — com o controlo de que um `--fora` que não casa **aborta**; (b)
**listar** o conjunto que move, em vez de só o contar. As duas pagaram-se hoje: sem a (a) eu teria
medido o `texture_pattern_live` da `motion` como meu, e sem a (b) não haveria como escolher a fatia.
O patch vive em `/tmp/…/scratchpad/fecho.py` e são ~25 linhas.
