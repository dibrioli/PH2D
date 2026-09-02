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
