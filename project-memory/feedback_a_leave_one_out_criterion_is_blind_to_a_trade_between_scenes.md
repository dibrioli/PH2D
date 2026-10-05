---
name: feedback_a_leave_one_out_criterion_is_blind_to_a_trade_between_scenes
description: "Critério «tirar o pedaço não melhora NENHUMA cena mais que X %» tira um pedaço que é uma TROCA (−16 % numa cena, +2 % noutra); e arrumar registos não mede nada quando a obra corre em 2 ondas"
metadata:
  type: feedback
---

Doc 121 §9.17 (`line/motion-value`, 05/10). Para decidir se os pedaços do §9.15 se dobravam, escrevi antes de
medir: «dobra-se se tirá-lo do `F` não melhora nenhuma cena mais de `2 %`». À letra ele tirava o **B1**: sem ele as
conformes contínuas descem `2,2 %` (um passe de `0,015` ms), mas as densas tracejadas sobem `+16 %`, as densas
contínuas `+11 %`. O critério era cego à TROCA entre cenas. Fiquei com o critério anterior que já o aceitara
(nenhum arranjo pior que `+5 %`) e escrevi a falha do critério no doc, em vez de o reescrever em silêncio.

Na mesma rodada: os quatro candidatos à emissão tracejada levaram o `cs_escreve` de `128` VGPRs e `28` derrames a
`112` e **zero**, `47 → 29` KB de código — e o tempo não mexeu (`72` cópias = duas ondas: não há ocupação a ganhar).

**Why:** um critério por-cena com «nenhuma» é um MÍNIMO sobre cenas — basta a cena mais barata para decidir contra a
mais cara. E «menos registos ⇒ mais rápido» só vale quando a ocupação é o tecto.

**How to apply:** um critério de manter/tirar diz a TROCA aceitável (ex.: «nenhuma cena pior que `+5 %` e a soma das
cenas-alvo melhor»), nunca «nenhuma cena melhora sem ele». Antes de atacar registos, conte as ONDAS da cena-alvo; com
poucas, a régua é o comprimento do caminho por fio. Família: [[reference_topic_measurement_discipline]].
