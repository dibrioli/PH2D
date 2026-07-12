---
name: feedback-nonreproduction-is-not-proof-of-fix
description: "Bug intermitente que para de aparecer NÃO está corrigido — cheque o git diff antes de aceitar \"resolveu\""
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2763a9af-144e-488a-b803-b06687b3c3ed
---

Num bug **intermitente**, a não-reprodução **não é prova de correção**. Se o Enio disser *"alguma coisa
que vc fez deve ter resolvido"*, **cheque o `git diff` antes de concordar** — e diga não se o diff não
sustenta.

Caso real (Painter, retângulos per-layer, 2026-07-11): após 3 runs sem o artefato, o Enio concluiu que
eu tinha corrigido. O diff provou o contrário: **+21 linhas, TODAS dentro de `if std::env::var_os(...)`**
(instrumentação) + testes. **Zero mudança de comportamento** ⇒ nada podia ter corrigido. O bug seguia
vivo, só dormente ([BUGS_painter.md #11](../../../Documentos/Projetos/PH2D/docs/Painter/BUGS_painter.md)).

**Why:** é o falso-negativo do Bug #2 **invertido**. Lá, um binário stale fez um fix CERTO parecer morto
(custou 3 rounds). Aqui, a não-reprodução faz um bug VIVO parecer morto — e o custo é pior: shipar o bug
e perder todo o espaço de busca já eliminado. Aceitar a hipótese confortável é o modo mais barato de
desperdiçar uma investigação inteira.

**How to apply:**
1. Antes de aceitar "resolveu": `git diff` / `git status`. Mudou só teste + código atrás de `env::var_os`?
   Então **não corrigiu** — diga isso claramente, mesmo contrariando o Enio.
2. Bug intermitente que some ⇒ registre como **ABERTO/dormente**, não como resolvido. Documente o que foi
   **descartado** (vale tanto quanto a causa) e deixe uma **armadilha armada** (instrumentação env-gated,
   custo zero desligada) pra capturá-lo quando reaparecer.
3. Corolário do inverso: **rebuild limpo antes de declarar um fix MORTO** (Bug #2 lição #1).

Relacionado: [[feedback_harness_reproduces_mechanism_not_context]] (parar o harness cedo e instrumentar o
app) · [[feedback_no_industrial_claims_without_verification]] · [[project_painter_t19_latent_red_macos_2026_05_28]].
