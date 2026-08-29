---
name: feedback_a_mutation_proof_needs_a_control_on_its_own_filter
description: "Uma prova de mutação com filtro de teste que casa ZERO testes imprime \"SOBREVIVEU\" — conte quantos correram antes de acreditar"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-27T23:48:23.502Z
---

Numa prova de mutação, o filtro (`cargo test -p X <filtro>`) é parte do instrumento. Se ele
casar **zero** testes, o cargo sai `0` e o script imprime **"SOBREVIVEU"** — indistinguível de
um gate genuinamente vazio, e na direção que faz desistir da cura.

Aconteceu em 2026-08-27 (auditoria do Motion): o filtro `cook_lazy` casava zero, porque o
caminho do módulo é `cook::lazy::tests`. Duas mutações reportaram "sobreviveu"; com o filtro
certo as duas morreram.

**Why:** a prova de mutação afirma *"este gate reprova quando o produto quebra"*. Ela tem duas
metades — a mutação chegou ao binário, e o gate correu. Um filtro errado mata a segunda em
silêncio, e o mesmo vale para um `--test` com nome errado ou um crate sem o alvo.

**How to apply:** antes de acreditar num "SOBREVIVEU", faça o arnês **contar quantos testes
correram** com aquele filtro e abortar se for zero (ou menos que o esperado). É o mesmo controle
positivo que se exige de qualquer outra sonda — ver [[reference_topic_mutation_proofs]] e
[[feedback_a_closing_run_with_a_name_filter_never_reaches_a_tree_scanning_gate]]. E prefira
`bash scripts/cargo-test-narrow.sh <crate>` sem filtro quando a suíte for barata: o filtro só
compra tempo, e este é o preço dele.

⛔ O irmão: `| head`/`| tail` num comando cujo veredito importa destrói o exit code
([[feedback_pipe_masks_script_exit_code]]) — na mesma sessão o portão de fecho imprimiu
`error: test run failed` e reportou sucesso.
