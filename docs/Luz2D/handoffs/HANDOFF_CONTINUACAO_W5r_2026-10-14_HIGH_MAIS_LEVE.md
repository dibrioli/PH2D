# Handoff de continuação — Luz 2D, W5r: o High mais leve, de verdade (2026-10-14)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6`, por cima da W5q ([handoff](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md), não integrada).
> Dois commits, um por família (CLAUDE.md §0.10): `60f9973204` (a holográfica por COLUNAS) · `46ae0e4e30` (os
> arcos numa tabela), mais `e32d265f76` (o fmt do fecho) e os de docs. Nada integrado, nada enviado. Método:
> [MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md); placa conferida (`nvidia-smi`) antes de cada rodada.

## 0. A onda inteira, pelo produto (o «custo sem o Automático» do Inspector)

| | W5q `1231e8828f` | W5r |
|---|---|---|
| High, ms de placa por quadro (`a_nitidez_da_luz`, «custo sem o Automático») | 22,92 · 22,94 · 22,97 | **14,73 · 14,74 · 14,74** |
| Low | 2,74 · 2,75 · 2,75 | **2,38 · 2,39 · 2,38** |
| a `c2` do High (`lastro` low, um processo por caso) | 18,37 (W5q §1: 18,66) | **11,19–11,23** |
| abertura a frio, primeira luz · estufa (`medida_luz2d_w5o_abertura`, mín de 7 intercaladas) | 1,26 · 2,91 s | 1,29 · 2,92 s (igual) |
| a barra do app, High manual (prefs do dono, tela virtual, uma foto cada) | 37 fps · 26,8 ms (`PH2D_LIGHT2D_HRC_COLUNAS=0`, com a tabela) | **50 fps · 20,0 ms** |

3× alternado, load ~25, a placa só da linha (`nvidia-smi` antes e depois). ⚠️ Uma rodada anterior com a
`line-Macros-Automação` (o app) e a `line-MiroClone` (testes) na placa deu 26–62 ms nos dois lados: não conta.
A abertura a load 22–35 oscilou 1,3–2,1 s nos dois lados; com o load a cair, igual ao centésimo.

O «antes» é o binário de teste compilado NESTA árvore com os ficheiros da W5q (`git checkout 1231e8828f --` nas
duas crates, compilar, copiar, `git checkout HEAD --`). A imagem é a MESMA em toda a onda (§2–§3: ao bit).

## 1. Medir antes de mexer: a descida era MEMÓRIA, não conta

`medida_luz2d_w5o_lastro` (`PH2D_W5O_LASTRO=low PH2D_W5Q_FASES=1`, 2×, iguais ao centésimo), grade do High
2496 × 1472 (quadrantes N = 12 e 11), RTX 5060 Ti (448 GB/s, L2 32 MB). Por nível da `hrc_desce`, os 4
quadrantes: ~14,7 M entradas (n10: 19,3 M), 48 B lógicos por entrada (ímpar: T + R por lado; par: R + T + R),
e os bytes OBRIGATÓRIOS = ½ T_n + ½ T_{n+1} + R_{n+1} + R_n:

| nível | ms | M entradas/ms | obrigatório | GB/s |
|---|---|---|---|---|
| n0 | 0,670 | 21,9 | 441 MB | 658 |
| n1 | 0,575 | 25,6 | 397 MB | 690 |
| n2 | 0,708 | 20,8 | 375 MB | 529 |
| n3–n8 | 0,81–0,92 | 15,9–18,7 | 355–366 MB | 389–449 |
| n9 · n10 · n11 | 0,884 · 0,857 · 0,361 | 17,2 · 22,5 · 33,4 | 414 · 417 · 193 MB | 468 · 486 · 535 |

A conta (quatro `atan` e ~30 operações por entrada) é ~7 % do que a placa faz; os níveis do meio andam no teto
da memória. E a ESPINHA: o n1 move os mesmos bytes que o n4 em 0,58 contra 0,92 ms — no n1 a leitura do R de
cima está a ±2 linhas, no n4 a ±16, no n8 a ±256: cada fio de um warp lia uma LINHA diferente
(`v + 2k − 2ⁿ`), e o L2 deixava de segurar o que o nível de cima acabara de escrever. O diagnóstico (o arco
constante, experimento que saiu da árvore), mesmo processo: por linhas 19,7 → 19,7 ms (nada); por colunas
13,8 → 13,1–13,3 (a conta só aparece com a memória destravada).

## 2. A holográfica por COLUNAS (`60f9973204`)

T e R guardam-se com `v` o índice MAIS RÁPIDO — `((k·nx + xi)·Y + v)` — e o despacho decompõe a entrada na
mesma ordem (`decompor`; `t_saida[e]`/`r_saida[e]` continua a ser o índice): toda leitura de T e R (as
«diagonais» `v + 2k − 2ⁿ`, o par, o de cima) fica contígua entre fios vizinhos, e o plano `k` de um nível só
é lido pelos planos `≈ 2k` do seguinte (reuso a curta distância). A soma corre por ladrilhos 8×8 de (u, v): oito
v seguidos do R0 e oito texels seguidos do acumulado em TODO quadrante (por linhas, os quadrantes 1 e 3 escreviam
o acumulado com passo W). Só os endereços mudam. `Config::hrc_por_colunas` (`true`); o override `COLUNAS_HRC`.

`medida_luz2d_w5r_colunas` (mesmo processo, 7 rodadas intercaladas, load 16–28): **0 bytes** nos 26 casos das
réguas W5q × Low e High; o controlo (sem o «+1», a soma que a onda reescreve) muda os 26. Pelo produto,
um processo por caso, 2×:

| etapa (mín ms) | por linhas | por colunas |
|---|---|---|
| traça n0–n2 | 3,80 | 3,51 |
| sobe n3–n12 | 4,45 | 2,72 (n3–n9 ~0,40 → ~0,22 cada) |
| desce n0–n11 | 9,28 | 5,28 (n0–n9 ~0,87 → ~0,42 cada) |
| soma | 0,83 | 0,49 |
| `c2` | 18,37 · 18,37 | **11,95 · 12,04** |

A subida tinha a MESMA causa (o `t_de_baixo(2xi+1, v + 2c − h)`) e caiu com o mesmo corte (item 3 do pedido).

## 3. Os arcos numa tabela (`46ae0e4e30`)

Com a memória destravada, a conta da descida apareceu (§1). `hrc_arcos` escreve `arcos[2ⁿ + m]` (n = 0..=N,
`4 << (N+1)` B = 32 KB, `luz2d/buf/hrc_arcos`) UMA vez ao nascer, num envio próprio, com a MESMA `arco_conta`; o
`arco` lê a tabela (binding 12; o gerador escreve pelo 13, noutro layout). `Config::hrc_arcos_em_tabela`
(`true`); o override `ARCOS_TABELA`. Ao bit nos 26 × 2 (três variantes no mesmo processo). Pelo produto
(`PH2D_W5R_TABELA=0|1`, 2×): desce 5,30 → **4,42** (cada nível ~0,415 → ~0,34), `c2` 12,1–12,5 → **11,19–11,23**;
traça e sobe iguais. ⚠️ Havia testes da `line-MiroClone` na placa nessa rodada; a traça e a sobe saíram iguais
às da rodada anterior (sem eles) ao centésimo.

## 6b. ABERTO, na ordem

1. **A traça é agora a maior etapa (3,54 ms: n0 0,97 · n1 1,10 · n2 1,45).** ~444 M chamadas de `meio` em 3,5 ms
   (≈ uma por SM por ciclo): limitada pela conta e pelo `textureGather`, não pela memória. Alavancas ao bit: (a)
   um fio por sonda com os `k` num laço, o `meio(A)` partilhado (−1/8 · −2/17 · −4/55 das chamadas, ~0,3 ms);
   (b) pular o espaço VAZIO — um raio que só cruza ladrilhos apagados (`ladrilhos.rs`, a W5q) dá `VACUO` exato,
   SE o `dentro_exato` com cobertura 0 nas quatro amostras nunca disser «dentro» (conferir) e se toda cobertura
   da captura das cascatas nascer de uma caixa que acende ladrilho (Flip, Motion, sprites, rebote): a cena
   típica fica muito mais barata, o pior caso igual. Precisa de régua ao bit com esses conteúdos.
2. **sobe n10–n12 e desce n10 custam ~2× por entrada** (sobe n3–n9 ~70 M entradas/ms, n10–n12 ~45). Hipótese
   (não provada): por quadrante, T10–T12 (36–48 MB) passam o L2 (32 MB) e o nível seguinte já não os acha lá.
   ~0,5 ms.
3. O R guarda uma 4.ª metade sempre 0 (`Par(soma, 0.0)`): 25 % dos bytes do R. Empacotar 6 B pede dois cones
   por fio; com a descida já servida em boa parte pelo L2 (~770 GB/s efetivos no meio), o ganho é incerto —
   medir antes.
4. Os itens 2–4 da W5q §6b continuam (o miolo nos níveis ≥ 1, o bloco preso aos 16 KB, a abertura).

## ⛔ Recusas MEDIDAS e cuidados

| tentativa | medido |
|---|---|
| «a descida é limitada pela conta» | por linhas, o arco constante não mudou nada (19,7 → 19,7 ms) |
| ler as etapas de uma instância criada DEPOIS de várias outras no mesmo processo | a 3.ª caiu no regime lento do alocador: sobe n5–n9 0,22 → 0,59 ms SEM usar o arco. As etapas comparam-se por processo (`lastro`) |
| (da W5q/W5p, continuam) arena única · acolchoar · `MemoryUsage` · laços opacos · fundir passos em níveis grandes · tempo com outra linha na placa | [handoff W5q](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md) |

## Para o integrador

- Foundational tocada: só `ph2d-light2d-gpu`. Campos PÚBLICOS novos: `Config::hrc_por_colunas`,
  `Config::hrc_arcos_em_tabela` (por omissão `true`) — todo literal do workspace usa `..`. Buffer novo na
  `PassList`: `luz2d/buf/hrc_arcos` (`hrc::buffers`; o G5 verde). Entrada nova no `c2_hrc.wgsl`: `hrc_arcos` (um
  pipeline e um envio ao nascer). Nenhum contrato congelado (§6).
- App: `PH2D_LIGHT2D_HRC_COLUNAS=0` (por linhas). Testes: `tests/it/medida_luz2d_w5r.rs` (`#[ignore]`);
  `medida_luz2d_w5q::casos` passou a `pub(crate)`; o `lastro` lê `PH2D_W5R_COLUNAS` e `PH2D_W5R_TABELA`.
- O texto do `c2_hrc.wgsl` mudou: a primeira abertura do dono depois da integração recompila a luz; as seguintes
  vêm do cache.

## Gate e binário

Gate batched do fecho (verificador, régua = merge-base `0aab2c1cb6`): fmt vermelho na 1.ª corrida (o `use`
do ambiente, a constante `ARCOS_TABELA`), curado em `e32d265f76`; clippy `-D warnings` (forma do `ship.sh`)
verde; check `--workspace --all-targets` deny verde; censos 117/117 e 12/12. `nextest-impacted`
(`PH2D_SUITE=fecho`, `PH2D_GPU=1`, load 33 → 73 → 39): **11 753 / 11 753**. Os portões da cascata e do gabarito
(`holografica`, `g3_`) por nome a meio: 7 + 7 verdes.

Cache do dono (`fotografa_cena.sh` com `__GL_SHADER_DISK_CACHE_PATH=/home/enio/.cache/nvidia`,
`PH2D_LIGHT2D_SMOKE=5`): 1.ª abertura **2,69 s** (41 pipelines, load ~25), 2.ª **0,08 s**; o `GLCache` 1 269 →
1 272 MB. Binário do smoke (2.ª corrida, depois de `0a15a36c08`):

```
▸ linha line_luz2d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.52s
```

## Smoke do dono

✅ **Smoke OK pelo dono (14/10).**

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_TEMPO=1 PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. A cena: a lâmpada no centro, oito formas coloridas com sombra e o interior delas iluminado. No terminal, «a luz
   acendeu N s» — N abaixo de 0,5 s (cache aquecido: 0,08 s).
3. Com o High manual (as prefs do dono já o têm; senão **Settings ▸ 2D Light ▸ High**, desligue **Automatic**),
   clique a **Lâmpada**: o Inspector diz «Quality: High (manual), NN ms of GPU per frame» — ~15 (antes 23).
   A barra de baixo: ~50 fps (antes ~37). Arraste a lâmpada: as sombras seguem mais lisas.
4. A mesma imagem pelo caminho antigo: `PH2D_LIGHT2D_HRC_COLUNAS=0` à frente do comando — a cena IGUAL, o
   número volta a ~23 (por linhas, a tabela não muda nada: §1) e a barra a ~37 fps. Errado: imagem diferente, ou o High acima de 17.
