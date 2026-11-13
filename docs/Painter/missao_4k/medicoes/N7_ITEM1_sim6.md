# N.º 7 item 1 — a sim do Wet a 6 traços a 4K: a `assenta` NÃO é o que falta (13/11)

Ordem do dono 13/11 («1- tente»): mudar a aritmética da `assenta` para a sim a 6 traços chegar a ≥ 1,0 sim-s/s pela MÉDIA.
**Resultado: impossível só pela `assenta`.** Com a `assenta` a custo ZERO a média é 27,1 ms = 0,92 sim-s/s.
O produto fica ao bit (nenhuma mudança de física comitada). Por isso não há alarmes de pincel a correr nem fotos antes/depois.

## Régua

`crates/ph2d-wet-paint/examples/sonda_estagios_4k.rs`, 3840×2160, 6 faixas `Lei::Cobre`, 16 fios, `--profile smoke`,
rotas intercaladas no MESMO processo sobre o mesmo estado (clone do grid); média = a melhor ronda.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && bash scripts/ph2d-run.sh cargo build -p ph2d-wet-paint --example sonda_estagios_4k --profile smoke && PH2D_SONDA_ROTAS=a,z bash scripts/ph2d-run.sh target/smoke/examples/sonda_estagios_4k 6 24 4
```

Rota nova `z` = `MEMO_ASSENTA = 3`: a secagem devolve `sc` sem conta (resultado ERRADO de propósito) = o tecto de qualquer
aritmética da `assenta`.

## Medido (6 traços)

| corrida | load | rota | média ms | sim-s/s | p90 | estágio 1 (secagem) máx | estágio 3 (advect) med | hash |
|---|---|---|---|---|---|---|---|---|
| ANTES (só `a`, 24×3) | 4,8→7,5 | produto | 32,4 | 0,77 | 70,0 | 39,7 | 18,5 | `a9bac64ac5b857c8` |
| A×B 1 (24×3) | 5,1→9,5 | `o` (antes) | 33,4 | 0,75 | 73,1 | 47,5 | 19,4 | — |
| | | `g4` cores na grelha de ¼ de nível | 33,0 | 0,76 | 71,6 | 42,4 | 18,9 | muda |
| | | `g1` grelha de 1 nível | 33,0 | 0,76 | 69,0 | 41,3 | 19,1 | muda |
| | | `t25` Newton até 0,25 de nível (era 0,01) | 32,4 | 0,77 | 73,1 | 45,3 | 20,5 | muda |
| | | `n` = `g4` + `t25` | 32,0 | 0,78 | 67,7 | 45,2 | 20,1 | muda |
| A×B 2 (24×4) | 7,5→6,1 | `o` (antes) | 30,5 | 0,82 | 66,9 | 38,7 | 16,8 | `a9bac64ac5b857c8` |
| | | **`z` tecto: `assenta` grátis** | **27,1** | **0,92** | 45,7 | 14,5 | 16,8 | errado |
| | | `n` | 29,8 | 0,84 | 62,0 | 32,6 | 16,8 | muda |

Leitura: a `assenta` vale ~3,4 ms da média (30,5 → 27,1); a melhor aritmética real tirou 0,7 ms (ruído de carga).
Sem nenhum `assenta` a média continua acima dos 25 ms: o resto é o advect (estágio 3, ~17 ms em TODO passo), o campo de
fluxo (estágio 2, até 10 ms, 1 em 4), o project (estágio 5, até 7 ms, 1 em 3) e o chão da secagem (14,5 ms, 1 em 6).

## ⛔ Recusas MEDIDAS (não reconstruir)

| ideia | medido | porquê |
|---|---|---|
| quantizar `sc`/`uc` do memo (¼ ou 1 nível) | 33,4 → 33,0 ms | os misses do memo não são o custo dominante a esta cena; a secagem fica 41–42 ms |
| Newton com tolerância 0,25 de nível | 33,4 → 32,4 ms | ruído |
| as duas juntas | 30,5 → 29,8 ms | 2 %, muda o pincel sem comprar o alvo |
| qualquer aritmética da `assenta` | tecto 27,1 ms = 0,92 | a `assenta` inteira grátis não chega a 1,0 |

## ABERTO

Para 1,0 sim-s/s a 6 traços faltam ≥ 2,1 ms da média FORA da `assenta`: o advect (16,8 ms todo passo; partes no handoff
n.º 5 §1: momentum 3,4 · núcleo fundido 12,6 · commit 2,8) é a única alavanca do tamanho certo. Não é física da
`assenta`: pede outra decisão (o núcleo do advect ao bit, ou a placa). Os ~10 % de `crossbeam_epoch` em espera ativa
(handoff n.º 5) também continuam não investigados.

## Advecção, cura 1 (13/11): o complemento da troca DENTRO dos blocos — `COMMIT_NO_BLOCO`

Na rota de troca do commit, cada bloco do núcleo fundido copia do grid para o rascunho o complemento das
suas linhas logo depois de as transportar; as linhas fora da faixa copiam-se ao lado do núcleo (`rayon::join`);
o commit vira só a troca dos três planos. O grid e o rascunho finais são os da rota de antes por construção.
Sonda: rota `q` = o commit à parte (o de antes). Carga alta (outro agente): só o A×B vale.

| corrida | load | rota | média ms | sim-s/s | mediana | advect mín (mom · núcleo · commit) | hash |
|---|---|---|---|---|---|---|---|
| 24×4 | 5,5→12 | `q` antes | 33,4 | 0,75 | 29,9 | 3,51 · 13,71 · 2,93 = 20,2 | `a9bac64ac5b857c8` |
| | | **`a` produto** | **32,4** | 0,77 | 29,0 | 3,57 · 14,93 · 0,00 = 18,5 | `a9bac64ac5b857c8` |
| 24×6 | 11,6→10 | `q` antes | 32,6 | 0,77 | 28,0 | 3,44 · 12,55 · 2,81 = 18,8 | `a9bac64ac5b857c8` |
| | | **`a` produto** | **30,7** | **0,82** | 25,2 | 3,44 · 13,65 · 0,00 = 17,1 | `a9bac64ac5b857c8` |

Ganho: ~1,7 ms do advect em todo passo (o commit de 2,8 ms some, o núcleo sobe ~1,1 ms), média −1,0 a −1,9 ms.
Gate por nome `o_complemento_nos_blocos_da_o_commit_a_parte_ao_bit` (troca forçada, blocos 1/3/7/32,
série/paralela/medida, `rf` 1 e 4, o grid escrito FORA da faixa entre passos). Controlos: sem a cauda da linha
e sem as pontas fora da faixa ⇒ vermelho; a primeira versão do gate (cena sem escrita fora da faixa) deixava
a mutação da cauda passar — o rascunho velho já igualava o grid.

### ABERTO para 1,0 (medido com carga alta; reconfirmar calmo)

Média ~30,7 ms ⇒ faltam ~5,7 ms. O resto do advect: momentum 3,4 (passe à parte; no bloco já recusado),
núcleo ~13,6. Perfil (`perf -k`, 6 traços): `transport_row` 13 %, espera do rayon `crossbeam_epoch` 10 %,
`outflow_block` 7,6 %, `flow_at_point` 4,2 % (símbolo próprio — o momentum não o inline), `prepare_row` 4 %.
Próximas a medir: `#[inline(always)]` no `flow_at_point` (ao bit, mesma ordem); os picos (secagem 1 em 6,
campo de fluxo 1 em 4, project 1 em 3) — a média é o relógio, os picos são ~metade do excesso.
Alarmes de pincel NÃO corridos nesta cura: a árvore tinha edições alheias em `ph2d-tool-painter` (outro agente)
que confundiriam a lista dos 77; o motor está provado ao bit pelo hash e pelos gates.

## Advecção, rodada 2 (13/11, 01:37–01:41): embutir, pedaços do rayon, número de fios — nada compra

Variantes no MESMO binário `smoke`, chave em execução (código provisório, revertido — nada no produto):
rota `i` = o momentum chama uma cópia `#[inline(always)]` do `flow_at_point` (mesma expressão); rota `lN` =
`.with_min_len(N)` nos cinco `walk_rows*` SEM redução (cada linha escreve só a sua ⇒ ao bit por construção);
`RAYON_NUM_THREADS` em processos à parte. 6 traços, 24 passos × 8 rondas, `ph2d-run.sh` (CPU ≤ 1600 %).

| corrida | load | rota | média ms | sim-s/s | mediana | p90 | hash |
|---|---|---|---|---|---|---|---|
| 16 fios | 5,2→9,9 | `a` produto | 30,0 | 0,83 | 25,3 | 67,5 | `a9bac64ac5b857c8` |
| | | `i` embutido | 30,3 | 0,82 | 25,5 | 69,2 | `a9bac64ac5b857c8` |
| | | `l4` | 30,0 | 0,83 | 24,9 | 69,5 | `a9bac64ac5b857c8` |
| 12 fios | 9,9→9,1 | `a` | 33,4 | 0,75 | 26,8 | 76,6 | `a9bac64ac5b857c8` |
| 14 fios | 9,1→8,2 | `a` | 31,4 | 0,80 | 26,3 | 71,7 | `a9bac64ac5b857c8` |
| 24×4 (1.ª) | 8,6→5,7 | `l16` | 43,4 | — | 73,5 | 194,9 | `a9bac64ac5b857c8` |

Partes do advect (mín): `a` 3,18 · 13,47 · `i` 3,11 · 13,26 · `l4` 3,22 · 13,65 · `l16` 3,42 · 13,37 ms (ruído).
Perfil `perf -k` sem `-g` (rota `a`, média 29,8): `transport_row` 16,0 % · `outflow_block` 8,9 % ·
`momentum_row` 8,4 % · `crossbeam_epoch` pin 7,9 % · `smooth_velocity` 5,1 % · `flow_rows` 5,0 % ·
`prepare_row` 4,9 % · `assenta` 4,6 % · `dry_cell` 4,0 %.

**Leitura:** a MEDIANA já está em ~25 ms (= 1,0); a média de 30 ms é toda dos picos (p90 ~68 ms: o passo
em que secagem 1/`dry_every`, campo 1/4 e project 1/3 coincidem). Os picos têm dependência de dados em série
(secagem → campo lê o filme; project lê a velocidade do advect): adiantá-los ou sobrepô-los muda QUANDO se
calcula ⇒ muda o bit. O `crossbeam_epoch` é o pin de cada tentativa de roubo dos operários ociosos — o
sintoma do desequilíbrio, não um custo de parede: agrupar linhas (`l4`, `l16`) não o converte em tempo.
O `transport_row` é um *gather* com retro-traço dependente de dados e soma `f64` em ordem fixa: não
auto-vetoriza; SIMD manual ao bit (gather de 4 cantos × 3 planos, a mesma ordem de soma) é a próxima
alavanca de tamanho certo, de várias horas, não feita.

### ⛔ Recusas MEDIDAS (rodada 2)

| ideia | medido | porquê |
|---|---|---|
| `#[inline(always)]` no `flow_at_point` do momentum | 30,0 → 30,3 ms; momentum 3,18 → 3,11 | já embutido de facto (`momentum_row` 8,4 % sem símbolo próprio do amostrador) |
| `with_min_len(4)` nos andadores por linha | 30,0 → 30,0 ms | ao bit, mas o pin do rayon é ociosidade, não sobrecarga |
| `with_min_len(16)` | 43,4 ms (rodada de 4, carga a variar) | pedaços grandes pioram o equilíbrio |
| `RAYON_NUM_THREADS` 12 / 14 (fatia de 16) | 33,4 / 31,4 contra 30,0 ms | menos fios = menos parede; sem sobre-subscrição a corrigir |

## Advecção, rodada 3 (13/11, 01:42–02:00): o transporte em lanes — `TRANSPORTE_LANES` (commit `6c1a3527b8`)

**Restrições medidas antes de escolher:** `unsafe_code = "forbid"` na workspace inteira ⇒ `core::arch` (AVX2 +
`is_x86_feature_detected!`) está FORA sem decisão de política; o alvo é `x86-64-v2` (SSE4.2: f64 de 2 lanes, sem
gather); nenhum `mul_add` na crate e o Rust não contrai ⇒ sem FMA. Escolhida a opção «blocos de 8 lanes em
arrays fixos que o LLVM vetoriza» (`advect_lanes.rs`), com o escalar (`transport_cell`) como cauda e oráculo.

**Onde pesa o `transport_row` (perf annotate, escalar):** espalhado — a `divsd` do `1/want` (~7 % por derrapagem),
os ramos `want > 0` / `active` / retro-traço (~6 %), as conversões `cvtss2sd` dos 4 cantos × 3 planos. Não é um
gargalo único de gather. **Não é banda de memória:** o núcleo escala 4 → 8 → 16 fios 38,6 → 20,4 → 12,7 ms.

| corrida | load | rota | média ms | mediana | núcleo mín | hash |
|---|---|---|---|---|---|---|
| lanes v1 (recolhe a cor sempre), 24×6 | 7,5→6,5 | `v` escalar | 28,6 | 23,9 | 13,21 | `a9bac64ac5b857c8` |
| | | `a` lanes | 29,4 | 24,4 | **14,16** (pior) | `a9bac64ac5b857c8` |
| lanes v1, 4 / 8 / 16 fios (12×3) | ~7 | `v` × `a` | — | — | 38,6×41,1 · 20,4×21,9 · 12,7×13,1 | — |
| v2: cor só se alguma lane troca, 24×6 | 3,6→4,0 | `v` | 27,9 | 23,0 | 12,53 | `a9bac64ac5b857c8` |
| | | **`a`** | **27,8** | 23,0 | **12,18** | `a9bac64ac5b857c8` |
| v3: + divisão só com troca, 24×8 | 7,6→9,3 | `v` | 28,5 | 23,4 | 14,68 | `a9bac64ac5b857c8` |
| | | **`a`** | **28,2** | 24,2 | **14,39** | `a9bac64ac5b857c8` |

Leitura: sem ramos, toda célula ativa pagava as 12 leituras da cor e a divisão que a escalar salta com `want = 0`
(células que só puxam água) — a v1 perdia ~7 %. Com a cor e a divisão só onde alguma lane troca, as lanes ganham
~0,3 ms do núcleo (2–3 %), média −0,1 a −0,3 ms: ficam por omissão (mesmo hash), mas **não** chegam a 1,0.
Gate `o_transporte_em_lanes_da_o_escalar_ao_bit` (controlos no commit). O `outflow_block` NÃO foi tocado: é um
scatter-add cuja ordem de soma por célula de saída é a lei do bit; vetorizá-lo pede outro desenho.

### ⛔ Recusas MEDIDAS (rodada 3)

| ideia | medido | porquê |
|---|---|---|
| lanes que recolhem os 4 cantos × 6 planos sempre | núcleo 13,21 → 14,16 ms | a escalar salta cor e divisão com `want = 0`; sem ramos isso vira trabalho |
| AVX2 por `core::arch` | não construído | `unsafe` proibido na workspace (política do dono/ADR, não da linha) |
| reassociar a soma `want` (como controlo) | gate verde | 1 ulp em f64 some na saída f32 — mutação equivalente, não serve de controlo |

### ABERTO
A média continua ~28 ms (0,89 sim-s/s; mediana 23 = 1,0). O que resta do núcleo é conta escalar espalhada; o
próximo degrau de tamanho certo é AVX2 com gather (precisa de abrir `unsafe` num módulo isolado — decisão de
política) ou os picos (secagem/campo/project a coincidir, ver rodada 2).
