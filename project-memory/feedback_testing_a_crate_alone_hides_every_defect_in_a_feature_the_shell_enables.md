---
name: feedback-testing-a-crate-alone-hides-every-defect-in-a-feature-the-shell-enables
description: "`cargo test -p <crate>` usa as features de omissão dele; correr o mesmo alvo JUNTO com o shell unifica features e acorda gates que varrem um registry — dois passaram verdes sobre 147 528 px² de defeito"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T19:11:59.723Z
---

Um gate que **varre um registry** (todos os painéis, todas as ferramentas, todos os nós) mede
apenas o que **esta build regista**. E `cargo test -p <crate>` constrói o crate com as features de
**omissão dele** — que quase nunca são as do binário.

⇒ correr os pacotes **um a um** e correr os mesmos pacotes **numa invocação só** dão populações
diferentes, porque o cargo **unifica as features** dos pacotes seleccionados.

**Medido na `line/UIUX`, 2026-08-31:**

| corrida | `flip_frames` registado? | veredito |
|---|---|---|
| `cargo test -p ph2d-panel-registry-init` | **não** (a feature não está nas de omissão) | ✅ verde, 2× |
| `… -p ph2d-panel-registry-init -p ph2d-host-desktop` | **sim** (o shell liga-a) | ❌ dois gates vermelhos |

O que eles apanharam: a tira do Flip pintava **292 px numa banda de 240**, começando 52 px acima
dela — **147 528 px² de painel por cima da área de desenho**, durante uma entrega inteira, com as
duas suítes verdes.

> *Uma suíte verde crate-a-crate não é a suíte do PRODUTO: ela é a suíte da build mais pobre que
> aquele crate consegue ter.*

⚠️ E o ✗ **lê-se como flake**: passa sozinho, reprova em conjunto, e a diferença parece ser carga.
Não é — é a **população**. O sinal que os separa: um flake de carga muda de teste entre corridas; o
de feature reprova **sempre o mesmo caso, com o mesmo número**.

**Why:** o oráculo destes gates é o registry, e o registry é função do `Cargo.toml` de quem corre —
a mesma raiz de [[feedback_a_registry_cannot_tell_a_missing_feature_from_a_typo_ask_the_tree]],
vista do outro lado: ali um id certo era acusado; aqui um defeito real ficava mudo.

**How to apply:** no gate de fecho, corra os crates de gate **na mesma invocação do shell**
(`cargo test -p ph2d-host-desktop -p ph2d-panel-registry-init …`), nunca só `-p <crate>`. E ao
escrever um gate que varre um registry, **imprima a contagem da população** — um piso `>= N` não
chega se o N foi calibrado na build pobre.

Relacionadas: [[feedback_a_registry_cannot_tell_a_missing_feature_from_a_typo_ask_the_tree]] ·
[[project_ci_runs_26_of_313_workspace_members]] ·
[[feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate]] ·
[[feedback_a_new_feature_can_empty_an_existing_gates_population]]

## ⛔⛔ Adenda 2026-09-01 — a DIREÇÃO OPOSTA: a feature que o shell **não** tem

O caso original era *o shell LIGA uma feature que a crate sozinha não tem*. Existe o simétrico, e
custou um report do Enio (*«Widget Lab não abriu»*) sobre um painel com **10 gates verdes**:

O `shells/desktop` põe `default-features = false` no `ph2d-panel-registry-init` e **re-declara**
cada painel como feature própria. Um painel no `default` **daquela** crate e sem linha no shell é
**compilado para fora do binário** — sem erro, sem aviso, sem uma linha vermelha em lado nenhum.

⚠️ **E o que fica na tela é o pior resultado possível:** a linha do menu continua **pintada e
clicável**, porque quem a pinta é a `ph2d-editor-core`, que não sabe nada de features do shell.
⇒ um **botão morto** produzido não por código em falta, mas por **um manifesto**.

**Por que nenhum gate o via:** todos corriam na `ph2d-panel-registry-init`, onde o painel
*existe*. `every_window_menu_row_reaches_a_consumer`, `..._reachable_by_the_z_order_walk` e
`build_typed_registry_matches_enabled_features` estavam os três verdes — o último por ser
*consistente* com um painel desligado.

**How to apply:**
- Ao acrescentar um painel/módulo atrás de feature, a lista de sítios **não acaba na crate dele**:
  procure quem o consome com `default-features = false` e re-declara features.
- ⭐ A cura durável é um gate que lê **os dois manifestos** e exige
  `default(registry) ∩ panel-* ⊆ default(shell)` —
  `shells/desktop/tests/every_panel_the_registry_ships_reaches_the_binary.rs`.
- ⚠️ Esse gate nasceu com o **próprio parser partido** (dividia por vírgula antes de descascar
  comentários e perdia 6 painéis) e só o **controlo positivo do corpus** (`>= 20`) o apanhou.
  *Um gate que lê manifesto precisa de saber quantas entradas devia ver.*
- ⛔ **Um smoke que "não abriu" pode não ser código.** Antes de depurar o painel, pergunte
  *ele está no binário?* — `strings <bin> | grep <id>` distingue as duas hipóteses em segundos.
