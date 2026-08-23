---
name: feedback_generalising_an_index_law_needs_a_derived_thickness_not_an_epsilon
description: "Ao trocar uma lei de ÍNDICE (`i < cols`) por uma de GEOMETRIA, a espessura tem de sair da estrutura (uma fileira), nunca de um epsilon — sobre a fixtura regular as duas concordam, e sobre uma curva o epsilon devolve UM ponto"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-23T22:22:03.471Z
---

`motion.soft_body`: o pino era *a linha de topo* (`i < cols`). Generalizei para
*o `y` máximo do repouso, a menos de uma tolerância relativa à altura* — e todos
os gates passaram, porque numa **grelha** a fileira seguinte está a um espaçamento
inteiro e qualquer tolerância pequena devolve exactamente `0..cols`.

A cena mostrou o defeito: um **DISCO** assim pregado fica preso pelo seu ponto
mais alto, **um só**. Ele balança como um pêndulo e a envergadura cresce **1,74×**
em dois segundos.

**Why:** um epsilon é uma afirmação sobre *ruído de representação*; a lei antiga
era uma afirmação sobre *estrutura* (a fileira de cima). Sobre a fixtura regular
as duas coincidem — e é precisamente por isso que a suíte inteira fica verde. Num
conjunto onde a estrutura é contínua, o epsilon colapsa a fatia a um ponto e o
gesto (uma barra que segura) vira outro gesto (um prego).

A cura é **derivar a espessura**: meia FILEIRA, onde a fileira sai da grelha
equivalente da nuvem. Numa malha isso reduz-se, por aritmética, a `0..cols`.

**How to apply:**
1. Ao substituir uma lei de índice por uma de geometria, escreva a barra na
   unidade que a lei antiga usava — *a que distância está o vizinho seguinte?* —
   e só depois pergunte como responder isso para o caso geral.
2. ⚠️ **A fixtura que a lei antiga usava não pode falsificar a nova.** Ela é o
   caso em que as duas concordam por construção; o gate que interessa vive numa
   forma que a lei antiga não exprimia ([[reference_topic_fixture_discipline]]).
3. Isto foi apanhado por um **smoke com a forma nova**, não pela suíte. Uma
   generalização que só tem gates sobre o caso antigo está **por testar**.
4. Prova de mutação directa: repor o epsilon mata o gate da cena. Sem ela, a
   escolha de «meia fileira» seria um número sem defensor
   ([[reference_topic_mutation_proofs]]).
