---
name: feedback-derived-coordinate-seed-must-match-sample
description: feature com coordenada derivada (ex. tempo remapeado) — todo caminho de AUTORIA deve usar a MESMA transform do caminho de LEITURA; audite juntos
metadata:
  node_type: memory
  type: feedback
  originSessionId: 63a4a831-e323-4bd7-9ba3-274c614260cb
---

O time remap da Timeline quebrou a autoria **3×** sempre pelo mesmo defeito de
classe: uma coordenada DERIVADA (o tempo-fonte = `remapped_time(playhead)`) foi
lida por um caminho e escrita/semeada por OUTRO, com transforms divergentes. No
fix final o `key_value_for` semeava o key de Time via `tr.sample(t)` (flat-clamp
fora do intervalo) enquanto `remapped_time` extrapolava slope-1 → K duplo criava
remap plano = **freeze** = "animação de posição anulada". Unit-green nas duas
tentativas anteriores, product-red no app.

**Why:** quando o valor autorado vive num espaço derivado (tempo remapeado,
offset acumulado, coord local vs mundo), o SEED e o SAMPLE têm que ser inversos
exatos. Se só um dos lados é corrigido, o feature "compila e testa" mas produz
lixo silencioso na composição real. Some-se a isso que unit isolado não pega — o
bug só aparece no caminho completo (seed → apply → render).

**How to apply:** ao mexer/rever qualquer coordenada derivada, **grep TODOS os
sites** que leem E escrevem esse espaço e reconcilie-os na MESMA função de
transform (aqui: `remapped_time` deveria ser a ÚNICA fonte, usada por seed E
sample). Escreva o repro pelo **caminho real** (seed real + `apply_from_doc`),
não por unit isolado ([[feedback_harness_reproduces_mechanism_not_context]]).
E valide no APP rodando antes de dizer "pronto" ([[feedback_tool_unit_green_integration_dead]]).
Vide [[project_motion_keyframes_deferred_timeline_integration]] (o outro
sistema de tempo, `motion.time_remap`, é escopo de cook — não confundir).
