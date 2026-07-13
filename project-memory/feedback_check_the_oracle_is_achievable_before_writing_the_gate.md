---
name: feedback-check-the-oracle-is-achievable-before-writing-the-gate
description: Um gate herdado pode exigir uma propriedade que NENHUMA implementação correta tem — escrevê-lo às cegas leva a afrouxá-lo até não medir nada
metadata:
  type: feedback
---

Antes de escrever o gate que um plano/handoff **prescreveu**, pergunte: *a propriedade que ele exige
existe?* Um oráculo pode ser **fisicamente inalcançável**, e aí o gate nasce vermelho por um motivo
que não é bug — e o caminho de menor resistência é **afrouxar a tolerância até ficar verde**, que
produz um gate que não mede nada e uma linha inteira de confiança falsa.

**O caso (Multiband, 2026-07-13).** O handoff da linha anterior mandava, textualmente: *"a soma das
bandas sem compressão tem de ser byte-idêntica ao input — esse é o gate que você escreve primeiro."*
**É impossível.** Um crossover Linkwitz-Riley soma para um **allpass**: magnitude plana (±0,0000 dB) e
**fase rodada**. O impulso somado difere do input em **0,0713 de fundo de escala**. Nenhum crossover
real satisfaz byte-identidade — nem um subtrativo (`HP = x − LP`), porque em f32 `(x−b)+b ≠ x`.

Provei isso em **~20 linhas de Python, antes de escrever uma linha de Rust**. Se tivesse escrito o
gate prescrito às cegas, ele ficaria vermelho com o código CERTO, e a "correção" óbvia teria sido
relaxar a tolerância até passar.

O gate certo era outro: **magnitude plana** (a propriedade que de fato existe), com o neutro
byte-idêntico onde ele já morava — no `is_bypass` (short-circuit no ponto neutro), não no crossover.

**Why:** um handoff é escrito por quem **não implementou aquilo**. A prescrição de gate dele é uma
hipótese, não um requisito — e uma hipótese sobre matemática que ele não rodou. Herdar o gate sem
checar a física é herdar o erro, e o erro se disfarça de "tolerância mal calibrada".

**How to apply:** todo gate cujo oráculo é uma **propriedade numérica forte** (byte-identidade,
reconstrução exata, soma unitária, invariância) merece 20 linhas de Python/scratch **antes** do Rust:
implemente o certo e o errado, **meça os dois**, e ponha a barra entre eles. Se o certo não atinge a
propriedade exigida, **a propriedade está errada, não o código** — troque o oráculo e diga no handoff
por quê, senão o próximo agente reescreve o mesmo gate impossível. Barra escolhida sem as duas
medições é chute ([[feedback_loose_oracle_hides_systematic_bias]]); mutação que não morde pode ser
mutação cega ([[feedback_mutate_the_code_not_just_the_test]]).
