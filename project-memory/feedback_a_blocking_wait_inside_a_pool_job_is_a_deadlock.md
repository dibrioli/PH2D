---
name: feedback_a_blocking_wait_inside_a_pool_job_is_a_deadlock
description: "Espera sem prazo dentro de um job do rayon é deadlock: `wait_until` rouba jobs injectados por cima do pedaço que espera (Bug #66, 05/11)"
metadata:
  type: feedback
---

Um operário do rayon à espera de um pedaço (`wait_until` → `find_work`) executa jobs injectados POR CIMA do pedaço inacabado, na mesma pilha. Se esse job espera algo que depende do pedaço (um `recv` pelo motor que o estágio paralelo devolve no fim), ninguém acorda: `yield_now` e espera activa não curam, o pedaço continua por baixo.

**Why:** congelou o app do dono em 05/11 (Bug #66); nasceu de uma otimização de velocidade (`no_pool`, R3) e ficou invisível porque o `cfg!(test)` desviava o caminho dos testes unitários. Com 32 núcleos é raro: há quase sempre um fio livre, e nem o app com pool de 2 fios o reproduziu em 6 corridas.

**How to apply:** tudo o que espera fica FORA do job; recuse-o alto com `assert!(rayon::current_thread_index().is_none())` na porta da espera, que apanha a família em toda corrida, sem depender da sorte do impasse. Atalho `cfg!(test)` num caminho de produto: o portão mora em `tests/it`.
