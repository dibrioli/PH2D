# I1 — a tecla sem o reparse: o que sobra na thread do quadro e a regra de Fatin

Pergunta (I1 do [`03_onda2.md`](../../03_onda2.md)): com o reparse do tree-sitter num worker
(`ph2d-code-intel` `GrammarWorker`), a tecla num ficheiro de 7 500 linhas ainda custa mais do que num de
51? A regra de Fatin, ao pé da letra: na vista de ~43 linhas, `7 500 − max(51 a, 51 b) ≤ |51 a − 51 b|`,
onde `51 a`/`51 b` são DUAS réplicas da mesma célula, intercaladas com as outras na mesma rodada. O ruído
mede-se nessa rodada; não se escolhe.

## Como re-correr

```
PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it bancada -- --ignored --nocapture
bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-code-intel --test it sonda_quadro -- --ignored --nocapture
```

Perfil `smoke` (otimizado), RTX 5060 Ti (Vulkan), 16/10. Outras linhas a correr: `loadavg` na linha de
cada tabela. Variantes no MESMO processo, em blocos rodados ([`MEDIR_VELOCIDADE.md`](../../../DevOps/MEDIR_VELOCIDADE.md)).

## 1. A cauda da tecla crescia: era LARGAR a árvore velha na thread do quadro

Rodadas 1–2 (n = 250, `load ~30–43`): a mediana já passava, mas a fase **tecla** tinha cauda O(ficheiro)
(p99 `0.89` ms a 7 500 contra `0.10` a 51) e o **quadro** também (`0.29–0.40` contra `0.04–0.15`).
Sonda sem placa (`sonda_quadro`, 51 × 7 500 intercalados, 1 200 teclas e 400 quadros por lado), só o que
ficou na thread do quadro, ms (min / p50 / p99 / p99.9):

| linhas | `edit_deferred` | `job` + `submit` | dreno + `adopt` + `job` (por quadro) | largar a árvore velha AQUI | entregá-la ao worker |
|---|---|---|---|---|---|
| 51 | 0.000 / 0.000 / 0.003 / 0.004 | 0.000 / 0.000 / 0.003 / 0.007 | 0.000 / 0.000 / 0.001 / 0.002 | 0.013 / 0.021 / 0.064 / 0.066 | 0.000 / 0.001 / 0.002 / 0.003 |
| 7 500 | 0.000 / 0.003 / 0.009 / 0.011 | 0.000 / 0.000 / 0.005 / 0.007 | 0.000 / 0.001 / 0.005 / 0.006 | **0.151 / 0.333 / 0.944 / 1.027** | 0.000 / 0.001 / 0.006 / 0.006 |

`loadavg 18.5`. Antes da separação, o dreno com o largar junto custava `0.167 / 0.502 / 1.696 / 1.989` a
7 500 (`0.013 / 0.037 / 0.117 / 0.124` a 51). Porquê: o reparse refaz a cadeia das declarações de topo
(I0a: cada uma é recusada como «fragile» e reconstruída), logo cada árvore velha tem milhares de nós só
dela. **Cura:** `Syntax::adopt_into` devolve as árvores largadas (`Retired`) e o `GrammarWorker::retire`
liberta-as na thread do worker.

## 2. A regra de Fatin depois da cura (n = 1 000 por célula)

Tecla→pixel, ms. Rodada 3 (`loadavg 9.8 → 5.3`, 59,5 s):

| cena | quantil | 51 (a) | 51 (b) | ruído | 7 500 | excesso | veredito |
|---|---|---|---|---|---|---|---|
| parada | min | 1.076 | 1.064 | 0.012 | 1.029 | −0.047 | passa |
| parada | p50 | 1.157 | 1.147 | 0.010 | 1.122 | −0.035 | passa |
| parada | p99 | 1.930 | 1.891 | 0.039 | 1.833 | −0.097 | passa |
| parada | p99.9 | 2.630 | 2.714 | 0.084 | 2.471 | −0.243 | passa |
| a tocar | min | 1.106 | 1.104 | 0.003 | 1.099 | −0.007 | passa |
| a tocar | p50 | 1.184 | 1.183 | 0.002 | 1.179 | −0.006 | passa |
| a tocar | p99 | 2.429 | 1.961 | 0.468 | 2.421 | −0.008 | passa |
| a tocar | p99.9 | 2.679 | 2.578 | 0.101 | 2.882 | +0.203 | REPROVA |

Rodada 4 (`loadavg 5.6 → 9.6`, 80,8 s): passa em min e p50 parada, p99.9 nas duas, p99 a tocar;
REPROVA em p99 parada (`+1.229`, ruído `0.016`) e p50 a tocar (`+0.052`, ruído `0.030`).

**Leitura.** O quantil que reprova muda de rodada para rodada, e o ruído de um par de réplicas na cauda
vai de `0.02` a `1.8` ms dentro da MESMA rodada: na cauda, a régua de um par não decide. As fases dizem
onde aterra (rodada 3, a tocar, min / p50 / p99 / p99.9):

| fase | 51 (a) | 51 (b) | 7 500 |
|---|---|---|---|
| tecla | 0.01 / 0.01 / 0.03 / 0.03 | 0.01 / 0.01 / 0.03 / 0.06 | 0.01 / 0.01 / 0.03 / 0.04 |
| quadro (+cena) | 0.02 / 0.03 / 0.08 / 0.09 | 0.02 / 0.03 / 0.07 / 0.08 | **0.05 / 0.05 / 0.11 / 0.13** |
| paint | 0.44 / 0.47 / 0.88 / 1.03 | 0.44 / 0.46 / 0.83 / 0.95 | 0.40 / 0.42 / 0.80 / 0.96 |
| placa + leitura | 0.60 / 0.67 / 1.91 / 2.19 | 0.60 / 0.66 / 1.08 / 2.08 | 0.61 / 0.67 / 1.91 / 2.29 |

- A tecla, a pintura e a placa do EDITOR já não crescem com o ficheiro, em nenhum quantil.
- O que cresce, sistemático nas duas rodadas, é o «quadro (+cena)» da célula a tocar: `+0.03` ms de p50.
  É a cena a correr o próprio script de 7 500 linhas (`scene_sync` + `scene_tick` do `ScriptHost`), não o
  editor. Na célula parada (sem cena) é `0.00` nos dois.
- ⇒ **Veredito ao pé da letra: a regra passa na cena parada em todos os quantis da rodada 3, e na cena a
  tocar passa no min. O p50 a tocar tem `+0.05` ms de excesso, todo ele da VM (o custo do script maior).**
  As caudas trocam de veredito entre rodadas: é ruído da máquina partilhada, com a fase de cada uma ao lado.

## ⛔ Recusas medidas

| Ideia | Porquê não |
|---|---|
| Largar a árvore velha na thread do quadro | `0.33` ms de p50 e `~1` ms de p99.9 a 7 500 linhas (tabela 1) |
| Esperar que o trabalho em voo volte antes de mandar o seguinte | uma 2.ª tecla no mesmo quadro só aparecia no realce DOIS quadros depois (vermelho em `gramatica::o_realce_alcanca_a_tecla_no_quadro_seguinte`); o novo trabalho substitui o velho e o worker salta os substituídos |
| Regra de Fatin com n = 250 | o p99.9 era UMA tecla e o veredito trocava de célula entre rodadas; a bancada passou a 1 000 por célula |
