---
name: feedback_two_good_hypotheses_failing_refutes_the_family_not_the_two
description: Quando duas curas teoricamente certas não movem o número, o modelo está errado — pare de supor e construa a régua que diz ONDE o defeito mora
metadata:
  type: feedback
---

**Duas hipóteses boas que falham não refutam duas coisas — refutam a FAMÍLIA a que
as duas pertencem. E a resposta certa não é a terceira hipótese: é a régua que
LOCALIZA.**

⛔ **Caso medido (quad-remesh, 2026-08-23).** Os quads saíam enviesados (mediana
`27°` contra `6°` do oráculo). Duas curas foram construídas, cada uma com mecanismo
sólido e cada uma nomeada por uma medição anterior:

| cura | porque era boa | resultado |
|---|---|---|
| o interior do patch segue o campo cruzado | a sonda mostrara que a 2.ª família de linhas não fica ortogonal à 1.ª | ⛔ `27°` → `27°` |
| o domínio com os lados ∝ segmentos | um patch `13 × 6` recebia 13 e 6 divisões sobre um quadrado `1 × 1`: **toda célula nascia com aspecto 2,17** | ⛔ `27°` → `27°`, e **piorou** a cauda noutra peça |

⭐ **A régua que veio a seguir respondeu em cinco minutos o que as duas curas não
responderam em horas.** Mediana do enviesamento **por fase de origem** de cada face:

```text
    canto 0°    arco 26°    centro 0°    raio 56°    grade 26°
```

⇒ **está em TODA a parte, e a grade interior mede o mesmo que o resto.** Isso exclui
de uma vez **toda** a família «uma construção local está errada» — que era a família
das duas curas.

**Why:** uma hipótese que falha gasta o custo de a construir e devolve um bit. Uma
régua que localiza devolve a *partição do espaço de hipóteses*. Quando a primeira boa
hipótese falha isso é ruído; quando a segunda falha, o sinal é sobre o **modelo**, e
insistir com uma terceira é pagar o mesmo preço pelo mesmo bit.

**How to apply:**
1. ⭐ **Ao segundo falhanço, PARE de propor curas.** Pergunte: *que medição
   particiona as hipóteses restantes?* Normalmente é uma decomposição do defeito por
   **fase**, por **região** ou por **proveniência** — não uma medida melhor do mesmo
   agregado ([[feedback_a_global_extreme_is_not_a_per_face_ruler]]).
2. ⭐ **Corra o CONTROLO da fase que ninguém suspeita.** Aqui foi o alisamento: a
   hipótese «é ele que enviesa» é natural, e medi-la a `0`, `6` e `20` rondas mostrou
   que ele **repara** (grade `27° → 26° → 25°`). Um suspeito ilibado com número é tão
   valioso quanto um culpado.
3. ⭐ **Guarde as curas falhadas LIGÁVEIS e desligadas, com a tabela ao lado** — não
   as apague. A canalização que elas exigiram costuma ser o ganho durável: aqui, o
   campo cruzado passou a **chegar** à fase que o precisava, o que era a primeira
   coisa que faltava. Ver [[feedback_documented_decision_chesterton_fence]] e
   [[feedback_if_relaxation_cannot_move_the_median_the_defect_is_in_the_connectivity]].
4. ⚠️ **A régua nova costuma trazer o achado que ninguém procurava.** A mesma jornada
   mediu a *holonomia* do campo dentro de um patch — `29°` a `44°`, onde devia ser
   ~`0°` — e isso **explica** por que a cura não podia funcionar (o campo lá dentro
   não é consistente) **e** aponta a fase culpada, que é outra.
