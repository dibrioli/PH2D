---
name: every-guard-written-with-lt-or-gt-is-blind-to-nan
description: "Toda guarda e toda régua escritas com `<`/`>` deixam passar NaN em silêncio — quem procura NaN tem de perguntar por ele"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: fdffb0c7-03ac-4fef-92aa-928a20f03b2e
  modified: 2026-09-20T17:14:33.289Z
---

Em IEEE-754 **toda comparação de ordem com `NaN` é falsa**. Isso morde nas duas
pontas, e em 2026-09-20 mordeu nas duas **no mesmo defeito**:

- **No produto:** `if w <= 0.0 { return; }` existe para parar um dab sem nada a
  dar, e com `w = NaN` ela **não retorna** — o `NaN` atravessa a guarda e envenena
  o canal.
- **Na régua que eu escrevi para o achar:** `if d > pior.0 { pior = d }` é um
  máximo por comparação, e ele **salta todo `NaN`** ⇒ devolveu `0,0` sobre uma
  peça inteiramente envenenada. O gate passou **por vácuo** na primeira corrida.

**Why:** uma régua escrita para achar um valor inventado não pode usar a ORDEM
dos `f32` para o achar, porque o valor que ela procura não está nessa ordem. E um
`NaN` num canal por-vértice não pinta um pixel errado: ele contamina a
interpolação da **face inteira**, que se lê como uma mancha de aresta dura.

**How to apply:** antes de confiar num `min`/`max`/`clamp`/`<`/`>` sobre `f32`
que um utilizador ou uma cadeia de contas alimenta, pergunte `is_nan()`
**explicitamente** — e num gate que caça valores inventados, procure o `NaN`
**primeiro**, antes de qualquer agregação. `NaN.clamp(0,1)` devolve `NaN`;
`NaN.max(0.0)` devolve `0.0`; as duas leem-se iguais no código e têm consequências
opostas. Ver [[a-law-written-in-one-place-is-not-a-law-only-a-door-is]] para a
outra metade do mesmo defeito: a guarda existia numa das duas cópias da lei.
