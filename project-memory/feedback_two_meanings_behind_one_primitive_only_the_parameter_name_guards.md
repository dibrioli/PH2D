---
name: feedback-two-meanings-behind-one-primitive-only-the-parameter-name-guards
description: "VecPathId e Entity::to_bits são ambos u64: o compilador não separa os dois, o gate escrito da assinatura também não, e o app morre no smoke — a convenção do irmão é a única barreira"
metadata:
  type: feedback
---

Smoke do Enio, 2026-08-25 (`PH2D_BUILD_SMOKE=75`, quadro 1639):
`PH2D PANIC … "Attempted to initialize invalid bits as an entity"`, e o processo morre.

A causa: `morph_of_selection(sim, sel: &[u64])` fazia `Entity::from_bits(b)` — mas a seleção do
editor vetorial é uma lista de **`VecPathId`**, não de bits de entidade. `pub type VecPathId = u64`
e `Entity::to_bits() -> u64`, então **o compilador não tinha como ajudar**.

**Medido** (`bevy_ecs 0.18`, e o número muda o diagnóstico):

| `Entity::from_bits(v)` | resultado |
|---|---|
| `0` | ⛔ **PÂNICO** |
| `1` | `PLACEHOLDER` |
| `2`,`3`,`4` | entidade de **lixo** (`4294967293v0`), sem componente nenhum |

⇒ o defeito tinha **duas caras**: com ids pequenos a seção simplesmente **nunca achava** o objecto
(silêncio total), e com o id `0` o app **morria**. ⭐ E o `0` não é canto: `VecScene` deriva
`Default`, então `next_id` nasce em `0` e a **primeira forma de toda cena** tem id `0` — clicar
nela era o gesto que matava.

**Why:** dois significados atrás do mesmo primitivo passam por *todas* as redes automáticas — tipo,
lint, e o gate. ⚠️ **O meu gate ficou VERDE sobre o pânico** porque eu o escrevi a partir da
*assinatura* (`&[u64]` ⇒ alimentei bits de entidade) em vez de a partir do **chamador**. E a 1ª
tentativa de o corrigir ainda não apanhava nada: ela usava `[1, 2, 3]`, que decodificam para lixo
mas **não entram em pânico** — a mutação sobreviveu, e foi isso que me obrigou a medir.

**How to apply:**
1. **Antes de escrever uma função que recebe ids, procure a IRMÃ** que já responde à mesma pergunta
   e copie a assinatura dela. Aqui o `host_of_selection` declara `selected: &[VecPathId]` e resolve
   pelo `VecEntityMap` — a dois ficheiros de distância. *Quando dois significados partilham um tipo
   primitivo, o NOME do parâmetro é a única barreira que resta*
   ([[feedback_a_house_param_name_carries_a_contract_pick_another_word]]).
2. **Escreva o gate a partir do CHAMADOR, não da assinatura.** Um gate que constrói o argumento à
   sua maneira prova a assinatura; só o que constrói como o produto constrói prova a costura
   ([[feedback_tool_unit_green_integration_dead]]).
3. **Ponha o valor DEGENERADO na fixtura** — o `0`, o vazio, o primeiro. Foi ele que matou, e uma
   fixtura sem ele aprova a cura errada
   ([[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]).
