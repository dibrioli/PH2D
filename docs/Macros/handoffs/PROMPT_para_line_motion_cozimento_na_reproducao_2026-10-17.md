# PROMPT para a `line/motion-value` — o cozimento do Motion escolhido pelo TAMANHO da cena (2026-10-17)

> Origem: a linha das Macros, W8 ([plano](../05_plano_W8.md), decisão 2d). O dono decidiu (17/10), ao ler as
> opções: **«escolher pelo tamanho»** — e que o trabalho é da linha do Motion, que está em desenvolvimento.
> Cole o bloco abaixo na janela da `line/motion-value`.

```
Decisão do dono (17/10), vinda da linha das Macros: o cozimento do Motion escolhe CPU ou placa pelo
TAMANHO da cena, com o ponto de troca MEDIDO — e não pela variável de ambiente sozinha.

O que se mediu (Macros W7, docs/Macros/04_plano_W7.md item 6; sonda
crates/ph2d-app-motion/src/macro_track/palcos_custo_tests.rs `sonda_n_palcos_do_motion`, smoke, máquina
calma, mínimos, intercalado no mesmo processo): numa cena de 36 formas com campos (a da
PH2D_MACRO_SMOKE=2), por palco, placa sincronizada no fim do quadro × CPU:
  N = 1: 0,302 × 0,023 ms · N = 4: 0,249 × 0,023 ms · N = 16: 0,241 × 0,024 ms
⇒ nesta cena a CPU custa 10–15× MENOS. O palco da reprodução nasce de MotionState::new(), que lê
PH2D_GPU_COOK (crates/ph2d-app-motion/src/motion_state_verbos.rs, `gpu_enabled`): herda a escolha do app.

O que fazer (CLAUDE.md §0.0 — medir antes de limitar; MEDIR_VELOCIDADE.md):
1. Meça o ponto de troca: o custo por quadro CPU × placa em função do tamanho da cena (formas, pontos,
   campos — o que de facto pesa no cozimento), variantes no MESMO processo, intercaladas, o mínimo. A
   tabela vai para o doc-comment da constante e para o handoff.
2. A política: abaixo do ponto de troca coze na CPU, acima na placa; `PH2D_GPU_COOK=0|1` continua a forçar
   (para bissectar). Decida e escreva se a política vale para o editor inteiro ou só para os palcos da
   reprodução das macros (os palcos são N cópias a cozer no mesmo quadro — o custo de sincronizar a placa
   multiplica-se por N).
3. Um gate que prove a escolha nos dois lados do ponto (uma cena pequena coze na CPU, uma grande na placa),
   e que a resposta é a mesma nas duas portas (a CPU é a canónica, ADR-0126).
4. Smoke ao dono com a cena das Macros (PH2D_MACRO_SMOKE=2) e uma cena grande do Motion.
```
