# Medir velocidade sem esperar horas — a régua INTERCALADA

> Uma porta só para «a versão B é mais rápida que a A?». Nasceu no doc 121 §9.16 (`line/motion-value`,
> 05/10): a mesma rodada de comparação levou **4–5 h** pelo método antigo e **37 s** por este, com a máquina
> carregada (`load 20`–`29`) e o mesmo resultado ao `0,01` ms. Ordem do dono: *«não podemos esperar horas»*.

## ⛔ O que NÃO fazer (medido)

| hábito | o que custou |
|---|---|
| um PROCESSO por medida (abrir dispositivo, compilar shaders, medir `0,3` s, fechar) | `1`–`2` s de arranque por `0,3` s de régua, `360` vezes |
| esperar a máquina CALMA antes de cada medida (`sleep 20` + média de carga de 1 min) | `2 h` de espera com a máquina OCIOSA; `+2 h 35` presa quando outras linhas compilavam |
| um BINÁRIO por variante (`const` trocada por script e recompilada) | `15 ×` release `≈ 25 min` |
| comparar médias de corridas SEPARADAS no tempo | a soma dos ruídos das duas janelas |

## ✅ O método

1. **Variantes são caminhos do MESMO processo**, escolhidos em execução: no GPU, constantes `override` do WGSL
   passadas ao criar o pipeline (`PipelineCompilationOptions::constants`); no CPU, uma flag/enum no objeto. Peça
   nova que se vai medir NASCE assim — nunca como `const` a trocar por script.
2. **Crie todas as variantes lado a lado** (uma instância de cada) e aqueça-as fora da régua.
3. **Intercale em BLOCOS curtos por ordem RODADA** (ex.: `7` rodadas × blocos de `20` quadros/iterações, a
   variante inicial roda a cada rodada): uma carga alheia cai em todas por igual.
4. **O resumo é o MÍNIMO das rodadas** (interferência só SOMA tempo, nunca tira), com a mediana ao lado como
   controlo — mínimo e mediana separados por mais que o efeito ⇒ a régua não decide, repita.
5. **No GPU a régua é o relógio da placa**, não a parede: `ph2d_gpu::pass_profiler::drain(&device)` devolve os
   totais por rótulo desde a última chamada (chame `end_frame` no último quadro do bloco antes). A SOMA dos
   passes, não um passe sozinho (o relógio por passe mente na fronteira — doc 121 §9.13).
6. **UMA compilação no perfil `smoke`** (otimizado e incremental). O relógio da placa não depende da otimização
   do Rust; se a régua for de CPU, o `smoke` também é otimizado. ⛔ **A × B entre DUAS compilações** (a variante é
   código, não uma flag): `CARGO_INCREMENTAL=0` nas duas. O incremental com `codegen-units = 16` muda o código
   gerado pelo histórico das compilações — o MESMO fonte variou até ~5 G instruções (1,3 %) no `mede_o_dab` do
   Flat Bristle 240 (12/11, PainterWatercolor): dois «−2,9 %» comitados eram isso; limpo, 375,12 × 375,17 G.
7. **Nenhuma espera de calma.** Imprima o `loadavg` no início e no fim, ao lado — é contexto, não porta.

## ⚠️ O que este método NÃO substitui

- A PAREDE do app inteiro (a cena a `60 fps` no produto, `fotografa_cena.sh`): é outra pergunta, e é curta.
- O aquecimento: o 1.º quadro de uma cena nova é um evento próprio — meça-o à parte (doc 121 §9.15 c2: a
  média de uma sonda que começa fria escondia um quadro de `76` ms).
- A iGPU divide a memória com o CPU: carga MUITO alta (compilações pesadas) ainda soma tempo — o mínimo e a
  intercalação mantêm o «A contra B» justo, não o número absoluto.

## §CI — A × B em corridas reais (04/11)

1. Use um ramo `spike/*` e VÁRIAS pushes nele (cada push = uma corrida; A e B em paralelo, ≥ 3×, o mínimo por passo — o mesmo job varia 18–30 min).
2. `save-if` desligado nas corridas de medição: medir não pode mudar a cache.
3. A SEMENTE vem primeiro: a chave da cache muda quando o pin ou o workflow muda, e a 1.ª corrida é fria. Meça a segunda.
4. As caches de 10 GB evacuam-se umas às outras (`gh api repos/dibrioli/PH2D/actions/cache/usage`): uma corrida de medição pode
   expulsar a semente da outra. Confira `No cache found` e `Cache Size` em cada.
5. Régua de passos: `python3 scripts/ci-passos.py` (mínimo por passo, `A:id B:id`).

## Local sob carga (04/11)

- A régua é o CPU (user+sys) do teste, não a parede: a parede soma a fila de outras linhas. Parede e CPU lado a lado, o mínimo.
- Ocupação da suíte: `python3 scripts/junit-ocupacao.py <junit.xml>` (por janela, limiares, por binário) — mostra vagas paradas
  (foi assim que se viu o bloqueio de 210 s da Luz 2D).
- Dois perfis do nextest no mesmo ficheiro, mesma compilação, ABAB, o mínimo.

## Exemplo vivo

`sonda_intercalada` ([`crates/ph2d-app-motion/src/motion_shape_placa_gpu_intercalada_tests.rs`](../../crates/ph2d-app-motion/src/motion_shape_placa_gpu_intercalada_tests.rs))
e o roteiro [`docs/Motion Nodes/ferramentas/mede_intercalado.sh`](../Motion%20Nodes/ferramentas/mede_intercalado.sh):
`9` variantes × `6` cenas × `2` placas em `37 s`. ⛔ Na RTX, um processo que cria dezenas de pipelines pode
morrer com SIGSEGV DEPOIS de acabar (a thread da cache de shaders do driver): `__GL_SHADER_DISK_CACHE=0` só no
processo da sonda.
