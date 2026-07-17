---
name: feedback_two_engines_one_state_is_worse_than_a_slow_engine
description: "Se dois motores simulam o MESMO estado, recuse a otimização inteira — meio laço é pior que nenhum"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 62ac077f-09f4-41be-9a44-14a0a85668a9
---

Quando um caminho rápido assume **estado sequencial**, ele tem de assumir o laço
**INTEIRO**. Se um pedaço do laço volta pro caminho lento, os dois passam a manter
cada um a sua cópia do estado — e o rápido integra números calculados sobre a
trajetória do outro, todo tick. Recuse a reivindicação inteira: o caminho lento já
responde certo, e devagar-e-certo é melhor que rápido-e-mentindo.

**Why:** GPU/M5 Fase 3. Se o `motion.integrate` roda na GPU mas um nó do laço de
forças não tem kernel, o plano deixa uma fronteira ali — e o pump, pra cozinhar
esse nó, **re-cozinha o integrate com o `prev` DELE**. Duas simulações do mesmo
estado, divergindo. O ADR-0123 não nomeou isso: ele previa o gather por `id`
(recusa por-nó), e a armadilha é da **composição**, não do nó.

Não basta o nó dono do estado estar do lado rápido: a regra é sobre o **laço**. O
teste é de grafo e é barato: *algum nó reivindicado é fonte de aresta `pre` E
sobrou fronteira?* → recuse tudo.

**How to apply:** todo caminho rápido com estado precisa de um gate com um nó
**sem** cobertura DENTRO do laço, afirmando que nada foi reivindicado (não que "a
parte boa" foi). "Reivindicar só o prefixo" soa conservador e é o bug.
Ver [[feedback_layered_defenses_need_per_layer_gates]] (as camadas de recusa
apontam pro mesmo lado no produto — gateie cada uma isolada) e
[[project_motion_keyframes_deferred_timeline_integration]].
