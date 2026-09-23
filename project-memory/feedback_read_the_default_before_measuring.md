---
name: feedback-read-the-default-before-measuring
description: "Uma condição que junta «a placa sabe MARCHAR?» com «sabe PINTAR?» tira o dispositivo do modo de omissão — e toda a wave foi medida num caminho que o artista não toma"
metadata:
  type: feedback
---

⛔⛔⛔ **Antes de medir uma cura, pergunte em que MODO o artista abre a coisa — e prove-o lendo o
`#[default]`.**

Medido em 2026-09-23 (`ph2d-app-field3d`). Uma wave inteira do modelador implícito mediu o quadro do
**dispositivo** (`931 → 124` linhas de WGSL, `32,53 → 16,63 ms`, o divisor do prévio de `2` para
`1`), e o dono reportou *«ao arrastar fica grosseiro ainda»*. O despacho do quadro chamava a placa
numa condição só:

```rust
let pelo_dispositivo = matches!(p.shading, Shading::Render) && !lamps.is_empty() && takes_the_frame(..);
```

⚠️ **Ela junta DUAS perguntas** — *«a placa sabe MARCHAR esta peça?»*, que é **geometria** e não tem
modo, e *«sabe PINTÁ-LA?»*, que pede material, céu e olhar — e o `#[default]` do modo é o `Matcap`,
que o doc dele chama *«a omissão de um modelador»*. ⇒ **ao abrir uma cena o arrasto vai todo pela
CPU** (`90,17 ms` a `1920×1080`, divisor `D=3`, contra `16,63` e `D=1`), e a wave, o corpus de
`22` cenas e duas curas anteriores foram todos medidos num caminho que o artista não toma.

⭐⭐ **E a cura não transferiu, com o mecanismo:** na CPU a mesma fórmula compra `2 %` (e `5 %` PIOR
numa das telas), porque a CPU **já tinha a poda** — ela especializa a árvore por ladrilho e corre a
fita em JIT com SIMD, logo a contagem de instruções não a domina; o que lhe sobra são os PASSOS, e
a fórmula dobra-os. *Uma cura serve o motor em que a grandeza que ela ataca manda.*

**Why:** uma condição composta esconde qual das partes está a decidir, e um `&&` com um modo à
esquerda **curto-circuita** antes de a parte interessante ser avaliada. O sintoma é uma tabela de
ganhos verdadeira sobre um caminho e muda sobre o que ship.

**How to apply:**
1. **Antes da 1.ª medição, leia o `#[default]`** do enum que governa o caminho e confirme que a sonda
   corre nele. Se a sonda arma o modo, ela mede outro programa.
2. Quando uma condição junta duas perguntas, **parta-a e nomeie cada uma** — mesmo que a resposta de
   hoje seja a mesma. A composta é onde uma cura se esconde.
3. ⛔⛔ **E separá-las pode ser maior do que parece:** aqui a separação fez **testes de unidade
   comuns tomarem a placa**, que nesta máquina é partilhada, e três testes que **desenham** um
   quadro passaram a `SIGSEGV` (`NVVM compilation failed: 3`) ao sair do processo. ⇒ a cura foi
   revertida e ficou **dívida gateada por uma catraca AO CONTRÁRIO** — um gate que reprova no dia em
   que alguém a curar, para que a cura venha com a atribuição.
4. ⚠️⚠️ **E uma bissecção de UMA corrida não bissecta nada** numa família de sinais assim: a minha
   deu a metade acusada como verde, e a corrida seguinte do MESMO estado deu vermelho.

Ver [[reference_topic_measurement_discipline]] ·
[[feedback_a_probe_that_arms_a_module_by_env_var_measures_another_program_than_the_pill]] ·
[[reference_flip_fit_cache_ratio_is_a_load_flake]].
