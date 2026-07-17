---
name: feedback-a-negative-search-needs-a-positive-control
description: Um grep que volta VAZIO pode estar mentindo — prove que a busca funciona antes de concluir "não existe"
metadata:
  type: feedback
---

Um resultado **vazio** de busca é uma afirmação forte ("isto não existe no repo") e é **a saída mais
fácil de fabricar por acidente**. Antes de concluir ausência, **rode um controle positivo com a MESMA
ferramenta e os MESMOS flags**: procure algo que você SABE que está lá. Se o controle também vier
vazio, a busca está quebrada — não o repo.

**Why:** aconteceu duas vezes na mesma sessão (wave do envelope/warp, 2026-07-16), em ferramentas
diferentes, e as duas vezes o vazio era MENTIRA:

- `git grep -ril "mls\|puppet\|arap" -- crates/ shells/` → **vazio**. Eu quase concluí que o repo não
  tinha homografia nenhuma. Um `grep -rn "fn homography"` simples achou **duas** na hora
  (`ph2d-node-motion-four-point-warp/src/lib.rs:126` e o `transform_geom.rs` do Painter). O `git grep`
  composto tinha engolido o padrão em silêncio.
- Um agente de pesquisa concluiu "DeepFaceLab não tem MLS" porque o `grep` do shell dele era uma
  **função que roteava pro `ugrep --ignore-files`**, que honra o `.gitignore` — e o `.gitignore` do
  projeto era `*` + `!*.py`. O `command grep` achou na hora. Ele também notou que **zeros do
  GitHub code-search não valem nada sem controle** (a busca dele voltou 0 para `mls` *e* 0 para
  `numpy` no mesmo repo).

O padrão comum: **a ferramenta falhou de um jeito que se parece exatamente com "não achei"**. Regex
composto não suportado, alternação que precisa de `-E`, `\b` que não casa com `_` (`\bmls\b` **nunca**
casa `mls_rigid_deformation`), pathspec errado, ferramenta que filtra por `.gitignore`, índice que só
cobre o branch default.

**How to apply:**
- Vai afirmar ausência? **Controle positivo primeiro**, mesma ferramenta, mesmos flags.
- Prefira `grep -rn` simples e UM padrão por vez a um `git grep` composto — o composto tem mais
  formas de falhar em silêncio, e você não vê nenhuma delas.
- Desconfie de `\b` com identificadores (`_` é word char) e de alternação sem `-E`.
- Vale para o agente que você despachou também: se ele voltar com um negativo, pergunte pelo controle.

## O gêmeo: o resumo de busca FABRICA (e o falso positivo é pior que o falso negativo)

Mesma raiz — confiar na saída da ferramenta sem controle —, mas o sintoma é o oposto: em vez de
esconder o que existe, **inventa o que não existe, com confiança e citação**.

Na mesma wave, três agentes independentes bateram nisto:
- Um resumo asseverou, em **cinco consultas separadas**, uma frase sobre o Illustrator (*"you need 5
  points on your shape..."*) que **não existe em fonte nenhuma** — grep em ~15 páginas buscadas: zero.
  Era uma **mutação de uma citação real com um racional fabricado colado**. O controle de frase exata
  provava que a busca funcionava.
- Outro: um resumo afirmou que o blog do Fridrich Štrba discutia o formato CMX. A página diz
  *"No posts matching the query: CMX."*
- Outro: o resumo parafraseou a Adobe (*"resulting paths"*) onde o PDF oficial diz *"the distorted
  paths"* — e a paráfrase era justamente a hipótese que se queria testar.

**How to apply:** **nunca cite o resumo — puxe a primária.** Se a página não abre (a helpx da Adobe
bloqueia tudo), diga *"phrase-verified"* ou *"não-verificado"*, **não** ponha entre aspas. E ausente-
da-fonte-citada ≠ falso: a frase das "4 quinas" do envelope era **real, mas noutra thread** — o resumo
acertou o fato e errou a fonte, que é o modo de falha mais difícil de pegar.

Irmãos desta: [[feedback_pipe_masks_script_exit_code]] — lá o `| grep` troca o `$?`; aqui a própria
busca mente. E [[feedback_nonreproduction_is_not_proof_of_fix]] — não-reprodução não é prova. Nos
três, **o sintoma é sucesso silencioso**.
