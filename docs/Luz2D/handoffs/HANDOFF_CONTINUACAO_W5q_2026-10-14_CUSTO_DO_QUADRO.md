# Handoff de continuação — Luz 2D, W5q: o custo por quadro que o dono sente (2026-10-14)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` reaberta sobre o `main`
> `0aab2c1cb6` (W5o + W5p integradas). Quatro commits, um por família (CLAUDE.md §0.10): `da475f827c` (a `c2` do
> High) · `65f6a4357e` (a abertura) · `509c58b4b3` (o anel por ladrilhos) · `6425fb3678` (o bloco do miolo), mais os
> de docs. Nada integrado, nada enviado. Método: [MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md). Toda
> rodada com `nvidia-smi` + `fuser` antes (a placa só da linha, salvo onde se diz) e o `loadavg` ao lado.

## 0. A onda inteira, pelo produto (o `main` contra a linha, 3× intercalado, load ~2)

| | `main` `0aab2c1cb6` | W5q |
|---|---|---|
| High, ms de placa por quadro (`a_nitidez_da_luz`, «custo sem o Automático») | 23,53 · 23,56 · 23,53 | **22,88 · 22,90 · 22,94** |
| Low | 2,81 · 2,82 · 2,82 | **2,74 · 2,75 · 2,74** |
| abertura a frio, primeira luz (`medida_luz2d_w5o_abertura`, cena 5, cache numa pasta vazia) | 1,40 · 1,40 · 1,41 s | **1,21 · 1,20 · 1,23 s** |
| estufa inteira | 3,00 · 2,99 · 3,01 s | **2,81 · 2,80 · 2,81 s** |

O «antes» é o binário de teste compilado NESTA árvore com os ficheiros do `main` (`git checkout 0aab2c1cb6 --` nas
duas crates, compilar, copiar, `git checkout HEAD --`; o «depois» recompilado é igual byte a byte ao de antes da
troca). A imagem é a MESMA em toda a onda (§2–§4: ao bit).

## 1. A `c2` do High: o acumulado renasce PRIMEIRO (`da475f827c`)

Instrumento novo `Solver::medir_fases_da_hrc` / `fases_da_hrc` (`solver_hrc_fases.rs`: um passe por despacho,
somado por etapa e nível) e a régua `medida_luz2d_w5q::a_c2_e_as_etapas` no fim da `medida_luz2d_w5o_lastro`
(`PH2D_W5Q_FASES=1`). Um processo por caso, 2×, as duas rodadas iguais ao centésimo. Com o
`hrc_nasce_duas_vezes` da W5p TODOS os buffers caíam no regime rápido MENOS o acumulado:

| etapa (mín ms) | duas vezes, processo novo | uma vez, novo | Low largado (o melhor) | duas, Low vivo |
|---|---|---|---|---|
| traça n0–n2 | 3,83 | 3,81 | 3,83 | 3,83 |
| sobe n3–n12 | 4,42 | 6,80 | 4,48 | 4,46 |
| desce n0–n11 | 9,34 | 12,79 | 9,36 | 9,82 |
| **soma** | **1,40** | 0,88 | **0,83** | **1,74** |
| saída | 0,19 | 0,15 | 0,15 | 0,19 |
| soma das etapas (um passe por despacho) | 19,18 | 24,43 | 18,66 | 20,04 |

A soma escreve o acumulado e a saída lê-o; nada mais o toca. Variantes de ONDE ele nasce (experimento, saiu da
árvore), um processo por caso, 2×: **o acumulado nascido primeiro depois da largada** = o melhor caso nos dois
estados (novo 18,55 · 18,75; Low vivo 18,72 · 18,71); três nascimentos = dois (19,16); fora do lote largado = TUDO
no lento (24,45). Sem `hrc_nasce_duas_vezes` a ordem antiga fica (o controlo continua o antes da W5p).

Depois (`c2` inteira, mín ms, duas · uma): novo 18,60–18,76 · 24,44; Low largado 18,71–18,75 · 18,74; Low vivo
18,63–18,66 · 21,33. Pelo produto, 3× alternado: High 28,91–28,94 (`PH2D_LIGHT2D_HRC_DUAS=0`) → 23,08–23,11
(W5p: 23,39–23,60). ⚠️ O mecanismo dentro do alocador/driver CONTINUA não provado: o efeito é medido, repetível e
agora vale em todo estado, mas «memória já usada e largada é rápida» não explica o «fora do lote → tudo lento».

## 2. A C3 com a placa só da linha (item 2)

`medida_luz2d_w5p_placa`, load 8, 7 intercaladas, o `2565d66bbf` contra a W5p no mesmo processo: **ao bit nos 14
casos**; C3 Low cena 1 2,36 → 1,52 · cena 5 7,54 → 3,85 · cena 3 x = 2 4,30 → 2,33; High cena 2 4,56 → 1,63 · cena 4
8,30 → 3,69 · cena 5 7,99 → 2,92 (a W5p, com outra linha na placa: 2,35 → 1,35 · 7,80 → 3,30 · 8,41 → 3,53). A
razão quase não mudou (×1,6–×2,8). `o_custo_do_miolo`: a base 0,718 (W5o 0,68–0,69), o entorno 0 — iguais.
`a_nitidez_da_luz` muda de razão (×9–10 na W5o → ×8,2): o número novo ficou no comentário do portão.

## 3. O anel só nos ladrilhos 8×8 com corpo (`509c58b4b3`)

`ladrilhos.rs`: um bit por ladrilho da fina, acendido por quadro no CPU a partir das CAIXAS que a captura fina
desenha (formas e sprites, em texels da captura; folga = a vizinha do anel + o `textureGather` = 2 texels da fina
com a captura ×2) e da janela com a moldura de fora (o `classifica` e o `miolo_final` leem-nas). O `c3_anel.wgsl`
sai nos ladrilhos apagados; o `miolo_puxa` do nível 1 (`Nivel::ladrilhos`) lê um apagado como o −1 que o anel lá
escrevia — nenhum valor velho se lê, sem limpar o `miolo_fino` (um corpo que sai deixa o ladrilho apagado, e o
apagado lê-se como fora). A regra do índice vive em `wgsl/ladrilhos.wgsl`, concatenada nos dois módulos (o CPU tem
a mesma em `ladrilhos.rs`; a régua ao bit é o portão de que as duas concordam). Sem captura fina: todos acesos.

Ao bit, `medida_luz2d_w5q_ladrilhos` (mesmo processo, 26 casos: cenas 1–7, a 3 em x = 2, a 4 e a 5 de perto, a 5
que vira a 4 a meio, a caixa do `o_interior_do_opaco` a 2,5 parada e a andar; Low e High): **0 bytes**. ⚠️ O
controlo «sem o anel» (`anel_fora_da_regiao: false`) deu 0 bytes em todo caso — não serve de controlo; o controlo
que vale é `Solver::ladrilhos_so_da_janela` (os corpos não acendem): muda 4 casos (58 611–338 991 bytes). Pelo
produto (`sonda_grava_as_fluencias_w5o`, os sprites por texel): o ficheiro igual ao de `PH2D_LIGHT2D_LADRILHOS=0`;
só a janela muda 139 287 floats; os ladrilhos todos apagados 937 531. Relógio (load 46–99, a placa da linha):
fina+anel −0,05…−0,11 ms (Low cena 5 2,910 → 2,801; High cena 4 3,345 → 3,260); o miolo −0,01. A W5o estimava
+0,23 ms do anel: o que sobra dele é a janela (os fios saem cedo, mas correm).

## 4. O miolo com K relaxações por despacho nos níveis pequenos (`6425fb3678`)

`c3_miolo.wgsl`: `miolo_relaxa_bloco` e `miolo_erro_relaxa_bloco`, grupos de 256 fios que resolvem 16×16 texels
com uma borda de K ≤ 4 na memória do grupo (13,8 KB; o dispositivo pede o limite de FÁBRICA do wgpu, 16 KB — o
adaptador tem 48 KB). A conta de um texel ficou em `relaxado_de` / `erro_relaxado_de`, as ÚNICAS: o passe por
texel e o bloco chamam-nas (a base da W5o continua uma cópia espelhada). O K de cada nível vai no uniforme dele: o
maior de 4 e 2 que deixa os despachos em número par (o resultado tem de acabar na `a`). `Miolo::bloco` = o K
máximo (≤ 1 = a W5o).

| blocos a partir do nível (K = 4) | despachos | miolo Low cena 5 · High cena 5 · caixa Low (mín ms) |
|---|---|---|
| nenhum (a W5o) | 247 | 0,827 · 0,809 · 0,934 |
| 1 (todos) | 125 | 0,921 · 0,898 · 1,038 ⛔ |
| 3 | 139 | 0,792 · 0,774 · 0,910 |
| **4** (`BLOCO_ATE_TEXELS` = 16 384: o nível 3 tem 57 408, o 4 14 352) | **157** | **0,784 · 0,764 · 0,896** |
| 5 | 187 | 0,795 · 0,784 · 0,906 |

Na régua inteira (`medida_luz2d_w5q_bloco`, 1 × 2 × 4 no mesmo processo, os 26 casos): **0 bytes**, o controlo
(um ciclo a menos) muda 24; −0,035…−0,045 ms em todos (Low cena 5 0,824 → 0,780). Pelo produto: o ficheiro igual.
`o_custo_do_miolo`: o bloco também barateava o «sem a base» (1,20 → 1,00 ms; a razão ia a 0,821 > 0,8) — a lei da
base passou a medir-se isolada (`bloco: 1` dos dois lados: 247 · 605, razão 0,712) e o bloco tem teto próprio,
`TETO_DESPACHOS_DO_BLOCO` = 165 (medido 157, o controlo 247).

## 5. A abertura: a C3 começa antes de c0–c2 (`65f6a4357e`)

O módulo da c2/c3, o layout das lâmpadas e a amostra nascem antes; o fio da C3 abre-se antes da C0, do JFA, das
cascatas e da holográfica (0,2 s a frio em série antes dele). O C1 saiu para `solver_c1.rs` (o tecto: solver.rs
700 → 648). 3× intercalado, load 7–8, o antes = `da475f827c`: primeira luz 1,44–1,51 → 1,24–1,32 s; estufa 3,02–3,06
→ 2,78–2,97 s; a `c2` do High (a holográfica nasce agora ao lado da C3) igual: 18,30–18,37 ms nos dois estados.

## 6b. ABERTO, na ordem

1. **A `c2` é agora TRABALHO**: 18,66 ms = traça 3,83 + sobe 4,48 + **desce 9,36** + soma 0,83 + saída 0,15 (§1). O
   regime está resolvido; a próxima alavanca é a conta da descida (metade da `c2`), não o alocador.
2. O miolo nos níveis ≥ 1 corre na GRADE INTEIRA (o anel ligado: ~0,8 ms com a pirâmide toda). A máscara dos
   ladrilhos (§3) desce aos níveis com um OU por 2×2 — o mesmo idioma do `puxa` — e os níveis grandes podiam sair
   cedo fora dos corpos. Medir antes: os níveis 1–3 dominam o tempo do miolo?
3. O bloco (§4) está preso aos 16 KB de FÁBRICA. Com o limite do adaptador (48 KB, `ph2d-gpu`: foundational, todas
   as apps) um nível inteiro cabe num grupo (o 6 tem 897 texels) — as 32 relaxações sem borda nem despacho.
   Medir antes; a `base` da W5o muda de `b` com o limite (ao bit, mas outro custo).
4. A abertura: o caminho crítico é a fina e o anel no fio da C3 (~0,9 s cada a frio, em paralelo).

## ⛔ Recusas MEDIDAS

| tentativa | medido |
|---|---|
| o acumulado fora do lote largado (só os T e os R renascem) | TUDO no regime lento: 24,45 ms |
| três nascimentos em vez de dois | igual a dois (19,16 ms): o acumulado continua no lento |
| o bloco do miolo em TODOS os níveis (125 despachos) | mais lento: 0,827 → 0,921 ms — a borda repetida dos níveis grandes custa mais que o despacho |
| «o miolo é limitado pelo número de despachos» (W5o §6.3) | só nos níveis pequenos: 90 despachos a menos valeram 0,045 ms (~0,5 µs cada) |
| `anel_fora_da_regiao: false` como controlo da régua do anel | 0 bytes em todos os 26 casos — o controlo é `ladrilhos_so_da_janela` |
| (da W5p, continuam) arena única · escrever antes de largar · laços opacos · tempo com outra linha na placa | [handoff W5p](HANDOFF_CONTINUACAO_W5p_2026-10-13_ABERTURA.md) |

## Para o integrador

- Foundational tocada: só `ph2d-light2d-gpu`. Campos PÚBLICOS novos: `Config::anel_por_ladrilhos` (por omissão
  `true`), `Miolo::bloco` (`MIOLO_PADRAO`: 4) — todo literal do workspace usa `..`. Funções `#[doc(hidden)]` novas:
  `Solver::medir_fases_da_hrc`, `fases_da_hrc`, `ladrilhos_so_da_janela`. Buffer novo na `PassList`:
  `luz2d/buf/ladrilhos` (com o miolo; o G5 verde). Nenhum contrato congelado (§6).
- Ficheiros novos: `src/ladrilhos.rs`, `src/solver_c1.rs`, `src/solver_hrc_fases.rs`, `src/wgsl/ladrilhos.wgsl`;
  o módulo das cascatas concatena `ladrilhos.wgsl` depois do `c3_anel.wgsl` (a lista `FICHEIROS_DO_RC` da régua W5p
  tem 12, e o ficheiro novo conta vazio no `ANTES`). LOC: `solver_miolo.rs` 691, `solver_c3.rs` 681, `solver.rs`
  659 — perto do tecto, nenhuma linha nova sem corte.
- App: `PH2D_LIGHT2D_LADRILHOS=0` (o anel na grade inteira). Testes: `tests/it/medida_luz2d_w5q.rs` (três réguas,
  `#[ignore]`); `medida_luz2d_w5p::Vista::nova_com`; `o_custo_do_miolo` com o bloco.
- O texto dos shaders mudou (o anel, o miolo): a primeira abertura do dono depois da integração recompila a luz
  (~1,2 s); as seguintes vêm do cache.

## Gate e binário

Gate batched do fecho (verificador + a janela, régua = merge-base `0aab2c1cb6`): fmt e clippy `-D warnings` (forma
do `ship.sh`) — vermelhos na 1.ª corrida (`n % k == 0` → `is_multiple_of`; o fmt de um `eprintln!`), curados em
`052ebcc1d1` e verdes; check `--workspace --all-targets` deny verde; censos 117/117 e 12/12. `nextest-impacted`
(`PH2D_SUITE=fecho`, `PH2D_GPU=1`, load 2,6 → 30): **11 753 / 11 753**. ⚠️ Uma corrida anterior foi cortada a meio
pelo fim da sessão (15 «failed» = `SIGTERM`, 661 por correr) — não conta; a placa esperou ~40 min pela
`line/motion-value`. `agent-loop-profile.sh`: paralelismo 1,16/passo, `test`:`check` 4,2×, edições sem conferir
3,9 %, contexto 336 mil por passo (as 20 sessões mais recentes, não só esta). `target/*/incremental` reclamado
(23 GB).

Cache do dono (o `fotografa_cena.sh` com `HOME` = uma cópia do `~/.ph2d/prefs.txt` e
`__GL_SHADER_DISK_CACHE_PATH=/home/enio/.cache/nvidia` — sem ele o `HOME` isolado põe o cache noutra pasta),
`PH2D_LIGHT2D_SMOKE=5`, load ~30: 1.ª abertura **1,70 s** (44 pipelines), 2.ª **0,04 s**; o `GLCache` 1 219 → 1 221 MB
(acima de 1 024: ninguém o aparou). Binário do smoke (2.ª corrida, depois de `052ebcc1d1`):

```
▸ linha line_luz2d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_TEMPO=1 PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. A cena: a lâmpada no centro, oito formas coloridas com sombra e o interior delas iluminado. No terminal, «a luz
   acendeu N s».
3. Esperado: N abaixo de 0,5 s (o cache foi aquecido para esta versão: 0,04 s medido).
4. **Settings ▸ 2D Light**, **High**, desligue **Automatic**, clique a **Lâmpada**: o Inspector diz «Quality: High
   (manual), NN ms of GPU per frame» — ~23 (antes desta onda 23,5; antes da W5p 29).
5. A mesma imagem pelo caminho antigo: feche e abra com `PH2D_LIGHT2D_LADRILHOS=0` à frente do comando — a cena
   tem de ser IGUAL. Errado: N acima de 1 s, a imagem diferente entre as duas aberturas, ou o número do High acima
   de 24.
