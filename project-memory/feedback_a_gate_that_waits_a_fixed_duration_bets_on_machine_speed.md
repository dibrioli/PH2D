---
name: feedback-a-gate-that-waits-a-fixed-duration-bets-on-machine-speed
description: "Gate que espera um `sleep` fixo aposta na velocidade da máquina — troque por CONDIÇÃO, e se a grandeza só é carimbada pelo outro lado, a espera tem de DIRIGIR o produto"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e6737742-4fe5-47ef-a299-f60a403fc03e
  modified: 2026-07-31T01:36:26.163Z
---

Um gate que sincroniza com `std::thread::sleep(<duração>)` não está esperando o
fenômeno: está **apostando em quanto tempo esta máquina leva**. A aposta perde de dois
jeitos independentes, e na integração de 2026-07-30 o MESMO gate
(`the_worker_reports_what_a_step_costs`, o worker da água do Wet Paint) perdeu dos dois
no mesmo dia — sob a suíte inteira em paralelo (7800 testes disputando CPU) e no perfil
**DEBUG**, onde tudo corre ~16× mais devagar que o release em que a pausa foi calibrada.

**Why:** o gate afirma um FATO ("o handshake foi medido"); a pausa é só o meio de chegar
lá. Um `sleep` fixo transforma um gate de fato num gate de *timing*, que fica vermelho
por motivo que não é o dele — e um gate que falha por motivo alheio é um gate que alguém
vai silenciar em vez de acreditar.

**How to apply:** espere **até** a condição valer, com um prazo generoso, em vez de por um
tempo. Duas armadilhas que a primeira reescrita ainda pegou:

1. Se a leitura **consome** a janela (`take_*` que zera contadores), o laço tem de
   **ACUMULAR** o que cada leitura tirou — um poll que descarta a leitura vazia perde
   justamente o balde que ele espera.
2. Se a grandeza só é carimbada quando o produto **age** (ali: o `away` só existe quando o
   motor VIAJA para o frame), a espera tem de **dirigir o produto** a cada volta. Um laço
   que apenas dorme nunca preenche o que ele espera — a 1ª versão falhou em debug depois
   de esgotar 10 s de poll.

Depois de mexer, **re-prove a mutação**: o gate ficou mais robusto ou perdeu os dentes?
(ali: sem o `note_away` ele fica RED com o diagnóstico exato — [[reference_topic_mutation_proofs]]).

Relacionadas: [[reference_topic_gate_discipline]] · [[feedback_a_ship_x_can_be_the_environment_not_the_code]].
