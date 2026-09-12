---
name: a-rename-by-name-cannot-tell-an-address-from-a-memory
description: "Reescrever um símbolo por NOME destrói a prosa histórica que o cita — num repo onde o porquê vive em doc-comments, a memória é o caso mais comum"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: cbcca673-89ac-4ca6-be35-ef2308fe97cb
  modified: 2026-09-12T03:12:22.082Z
---

Uma reescrita mecânica de `X` → `Y` apanha **duas** populações que se leem igual e têm curas
opostas: os **endereços** (que têm de resolver depois da mudança) e as **memórias** (frases que
dizem *«isto chamava-se X»*). Trocar o nome numa memória torna a frase falsa e apaga o registo.

Medido 2026-09-11 (W2/L3-B, `line/app-sculpt3d`): apagar os cinco alias `field3d_*` reescreveu
26 sítios legítimos **e** estragou frases em 5 crates alheias —
*«Ela nasceu com o nome de um cliente (`field3d_layout::area`)»* passou a dizer que a função
nasceu com o nome que tem hoje. Reverter levou os ficheiros alheios tocados de **15 para 3**.

⚠️ **É a segunda vez na mesma linha**: na Fase A um script guiado pelo nome do ficheiro reescreveu
`sculpt3d.rs` → `sculpt3d/mod.rs` em três ficheiros da `ph2d-i18n`, onde aquele era o vizinho
**deles**.

**Why:** neste repo a prosa carrega o mecanismo e a história, então um nome antigo num
doc-comment é mais provavelmente um registo do que um endereço.

**How to apply:** restrinja a reescrita ao **código** (linhas que não começam por `//`) e faça a
prosa **à mão**, item a item. Depois audite `git diff <merge-base> --name-only` por ficheiros fora
do seu alvo: cada um tem de ter uma razão nomeada. Quando uma frase histórica fica factualmente
desactualizada, **risque-a com a data** em vez de a apagar — ver [[the-ruler-is-the-merge-base-not-a-moving-main]].
