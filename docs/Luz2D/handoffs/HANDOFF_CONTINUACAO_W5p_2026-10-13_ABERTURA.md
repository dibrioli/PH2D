# Handoff de continuação — Luz 2D, W5p: a luz sem espera (2026-10-13)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre `main`
> `757afebd58`, com a W5o ([handoff](HANDOFF_CONTINUACAO_W5o_2026-10-13_CUSTO.md), `dc081fa8bc`…`2565d66bbf`) e a
> W5p por cima: `f9a6c41460` (a abertura: o shader com um sítio por cálculo, a montagem em fios, as réguas) ·
> `95a2271805` (a face pelo ponto num laço) · os de docs. Nada integrado, nada enviado.
> Método: [MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md); o compilador do driver a frio com
> `__GL_SHADER_DISK_CACHE=0` (cada compilação é a frio — o controlo «de novo» confirma-o), variantes de TEXTO e de
> constantes no mesmo processo, 3 rodadas intercaladas, vale o mínimo.

## 1. O que pesa no compilador do driver: o NÚMERO DE SÍTIOS, não as voltas

A hipótese da W5o (os laços com limites de `override` desenrolados) está **recusada**: com 1 raio e 1 bissecção
a fina custa o mesmo (Low 4,02 → 3,79 s). O NVIDIA embute cada chamada; o custo segue quantos sítios da marcha
(e do mundo fora da grade, com os distantes) o shader tem. Antes: a abertura tinha 3 sítios do raio (o primeiro,
cada fronteira, a bissecção), a luz de um texel 4 laços de lâmpadas (ponto, ponto remapeado, outra face,
transmitida), a fina chamava a luz 2×, a face pelo ponto mais 3 aberturas e uma visibilidade inteira.

`medida_luz2d_w5p_compilador` (`tests/it/medida_luz2d_w5p.rs`; `PH2D_W5P_RODADA=2|3|5` as rodadas de diagnóstico,
sobre o texto do `2565d66bbf` lido do git), s a frio, fina · anel:

| Low (`c3_fina`), mínimo | sem distantes | **com distantes** (o caminho crítico) |
|---|---|---|
| antes (W5o) | 2,73 · 2,08 | **4,44 · 3,47** |
| W5p, abertura e luz num sítio | 0,92 · 0,91 | 1,28 · 1,41 |
| W5p + a face num laço | — | **0,84 · 0,90** |

High (`c3_fina_exata`) com distantes: 5,01 · 3,96 → **0,90 · 0,90**. O piso (uma entrada pequena do mesmo
módulo) 0,01 s: o tamanho do módulo não pesa. Diagnóstico (rodada 2, cortes que não valem como imagem): sem a
transmitida −1,4 s, sem a visibilidade na face −2,15, a marcha trocada por uma leitura −1,9. Os laços OPACOS
(limite que o compilador não conhece) não mudam nada; a técnica que vale é **um sítio de chamada dentro de um
laço** (a luz num laço = a luz cortada: 1,88 contra 1,87 s), mesmo com o limite 2 conhecido.

## 2. A reescrita (WGSL), a mesma conta

- `c3_direta.wgsl`: `transmissao_de(P, A, B, k)` (a marcha e os distantes, o único sítio) · `raio_ao_disco`
  (um raio da abertura; `pelo_ponto` = a W5o) · `soma_da_abertura` (máquina de estados com UM sítio do raio: os
  mesmos raios, a mesma soma, na mesma ordem) · `luz_das_lampadas(P, P0, modo, inicial)` (o ÚNICO laço de
  lâmpadas: `PELA_GRADE` / `EXATA` / `REMAPEADA`; uma `visibilidade`, medida em `em` e somada à da face quando é
  a de trás de um translúcido) · `luz_do_texel` (os passos ponto/outra face/transmitida num laço, com o
  `borda_aqui` e o `c0_tau` que cada um tinha — na exata a outra face lê a borda desligada pelo translúcido, como
  antes). As quatro entradas (fina, fina exata e os dois anéis) usam-no.
- `c3_pe.wgsl`: `na_face_pelo_ponto` (a frente e as duas pontas de trás em até 3 `Troco`, cada um com a conta do
  sítio de antes `soma / den * mul / div`, num laço com um sítio da abertura). A `visibilidade_na_face` fica só
  com o caminho W5g (`PH2D_LIGHT2D_FACE_PONTO=0`).
- Saíram `luz_fina`, `nas_faces`, `luz_exata`, `luz_exata_no_onde`, `luz_exata_remapeada`,
  `visibilidade_remapeada`, `ate_o_disco`, `visibilidade_pelo_ponto`, `atras_de_dentro`.
- ⚠️ O laço único cita as visibilidades também na exata: a ligação 2 da fina é sempre feita — sem o passe delas,
  as SEMENTES (inteiras; a exata nunca as lê). ⛔ Uma 1×1 nova reprovou o G5 (`g5_o_que_a_placa_alocou…`: alocada
  e não declarada na `PassList`) — o idioma é ligar uma textura que já existe (como a saída da holográfica).

**Ao bit** (`medida_luz2d_w5p_placa`, o `2565d66bbf` contra a W5p no MESMO processo por `Config::modulo_rc`):
cenas 1–6 na vista de fábrica e a 3 com a câmera em x = 2 (o entorno e os distantes), Low e High, 80 quadros —
**0 bytes diferentes nos 14 casos**, depois de cada um dos dois commits; controlo (o antes com `0,5 → 0,49` na
soma da abertura) 2 315 055 bytes.

## 3. A placa também: a C3 2–3× mais rápida

O shader menor cabe em menos registos (mais fios ao mesmo tempo). C3 (preenche + visibilidade + fina), mínimo
de 7 rodadas intercaladas, load 14–19 ⚠️ com outra linha a correr testes na placa (o «antes × depois» é justo,
o absoluto não): Low cena 1 2,35 → 1,35 ms · cena 4 4,16 → 2,26 · **cena 5 7,80 → 3,30** · cena 3 em x = 2 4,30
→ 2,09; High cena 2 4,58 → 1,55 · **cena 4 8,41 → 3,53** · cena 5 8,09 → 2,82. Rebote igual (Low cena 5
0,87 → 0,51). Falta a rodada com a placa só da linha (§6.2).

## 4. A montagem em fios

Linha do tempo (`PH2D_LIGHT2D_COMPILA=1` agora dá o início e o fio de cada pipeline, também as de desenho): a
primeira luz esperava c0–c2 (0,2 s) + a fina e o anel (4 em paralelo) + **0,6 s em série** (preenche, os dois
rebotes um a um, borda, as 13 do miolo). Agora: a fina e o anel compilam nos seus fios enquanto a C3 monta o
resto (`solver_c3.rs`); a C3 inteira num fio ao lado do miolo, da captura fina e da soma do rebote
(`solver.rs`); o rebote sem/com os distantes pelo `Par::em_paralelo`.

## 5. A abertura, de ponta a ponta (`medida_luz2d_w5o_abertura`, cena 5, 1080p, cache do driver numa pasta vazia)

| | primeira luz | estufa inteira |
|---|---|---|
| W5o (load 8–12) | 6,2–6,7 s | ~13 s |
| W5p sem a montagem em fios (load 7–8) | 2,45–2,55 s | 4,9–5,2 s |
| W5p com os fios, antes da face (load ~2) | **1,71–1,75 s** | **3,66–3,69 s** |
| W5p final (load 15–19 ⚠️) | 2,13 · 3,21 s | 3,7 · 6,3 s |
| 2.ª abertura (cache quente) | **0,06 s** | 0,06 s |

O caminho crítico que sobra: c0–c2 (0,2 s) + a fina e o anel (~0,9 s cada a frio, sozinhos).

## 6. ABERTO, na ordem

1. **A `c2` do High e o alocador (W5o §3)** — o experimento escrito foi feito e **recusado** (tabela abaixo). O que
   a matriz ensina: a alocação única e grande cai SEMPRE no regime lento (26,6 ms, o mesmo dos blocos de 4 MB
   dedicados da W5o); os buffers separados ficam rápidos só em memória já usada e largada (18,4). Hipótese para a
   próxima rodada: o regime lento é memória NOVA (dedicada, nunca tocada pela placa); medir uma escrita de
   aquecimento dos buffers da HRC ao nascer (`clear_buffer` ou um passe), e o mesmo com a arena.
2. Medir a C3 (§3) com a placa só da linha e gravar o número no portão `a_nitidez_da_luz`/`o_custo_do_miolo` se
   a razão mudar.
3. O anel só nos ladrilhos com corpo (W5o §6.2) e os 247 despachos do miolo (W5o §6.3) — não começados.
4. A abertura: c0–c2 antes da C3 (0,2 s) podem correr ao lado da fina; o resto da fina (a face ~0,2 s, a c0 ao
   longo da face ~0,2) já tem pouco dono.

`medida_luz2d_w5o_lastro` (um processo por caso, 2×, a placa só da linha, load 5–8), `c2` do High mín, ms:

| | processo novo | Low criado e largado | Low vivo | 1 GB escrito e largado |
|---|---|---|---|---|
| buffers separados (o produto) | 24,6 · 24,7 | 18,4 · 18,4 | 21,2 · 21,3 | 18,4 · 18,4 |
| controlo: separados, ligação leitura+escrita | 24,7 · 24,7 | 18,3 · 18,4 | 21,3 · 21,3 | 18,4 · 18,4 |
| arena, offsets alinhados a 256 B · 64 KB · 2 MB | 26,6 | 26,6 | 26,6 | 26,6–26,8 |

(A arena obriga a ligar leitura e escrita como escrita — o wgpu segue o uso do buffer INTEIRO num despacho; o
controlo prova que isso não pesa. O código do experimento saiu da árvore; a recusa fica aqui.)

## ⛔ Recusas MEDIDAS

| tentativa | medido |
|---|---|
| laços com limite opaco ao compilador (raios, bissecções, o pé, a de trás, a c0 da face) | fina 4,02 → 4,05 s: nada |
| contar o custo do compilador pelas voltas (1 raio, 1 bissecção) | 4,02 → 3,79 s: o custo são os sítios |
| os buffers da HRC numa alocação só (W5o §3, o próximo passo escrito) | 26,6 ms em todo estado (separados: 18,4–24,6) |
| ler um tempo da placa com outra linha na placa ou a placa sem exclusão | a matriz da W5p esperou ~25 min por 4 apps de 3 linhas |

## Para o integrador

- Foundational tocada: só `ph2d-light2d-gpu` — campo PÚBLICO novo `Config::modulo_rc: Option<&'static str>`
  (instrumento; todo literal usa `..`), funções públicas `constantes_da_fina` e `constantes_comuns_da_c3` (a régua
  e a montagem leem a mesma lista; `solver_c3_constantes.rs`). Nenhuma alocação nova. Nenhum contrato congelado (§6). `solver.rs` com 700 linhas (o tecto).
- Instrumentos: `tests/it/medida_luz2d_w5p.rs` do app (`_compilador`, `_placa`), `PH2D_LIGHT2D_COMPILA=1` com
  início e fio.

## Gate e binário

Gate batched do fecho (verificador, régua = merge-base, `PH2D_GPU=1`, load 36–69): fmt, check `--workspace
--all-targets` deny, clippy `-D warnings` (forma do `ship.sh`), censos 117/117 e 12/12 — verdes. `nextest-impacted`
**10 038/10 040**: (1) `g5_o_que_a_placa_alocou_e_o_que_a_pass_list_declara` — defeito real (a 1×1 nova), curado em
`aa30e1cc97` e verde por nome; (2) `g4_o_custo_da_holografica_nao_depende_do_numero_de_luzes` — a família de flakes
de carga ([FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md)): 5,8 % na suíte (load 52), 1/3 sozinho com load
60–65 o tempo todo; o diff não toca no relógio das cascatas. ⚠️ Falta vê-lo sozinho com load baixo (a máquina não
desceu de 80 até ao fecho). A suíte correu SEM o prefixo `PH2D_SUITE=fecho` (a regra chegou depois); não se
relançou (regra do dono 13/10: uma vez no fecho).

App real, cache do driver REAL, as preferências do dono (`fotografa_cena.sh`, `HOME` com a cópia do
`~/.ph2d/prefs.txt`, `PH2D_LIGHT2D_SMOKE=5`, load 87): a 1.ª abertura depois dos shaders novos **2,18 s** (W5o:
9,6 s), a 2.ª **0,05 s** — o cache do dono fica AQUECIDO para este binário. Binário do smoke (2.ª corrida, depois
de `aa30e1cc97`):

```
▸ linha line_luz2d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_TEMPO=1 PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. Olhe o centro da cena: a luz da lâmpada e as sombras das oito formas aparecem logo. No terminal, a linha
   «a luz acendeu N s».
3. Esperado: N abaixo de 0,5 s (o cache já foi aquecido para esta versão). Antes desta onda, a primeira vez depois
   de uma atualização levava ~10 s; agora, numa atualização que mexe na luz, ~2 s — e as seguintes, imediatas.
4. Errado: N acima de 1 s nesta abertura, ou a cena diferente da de ontem (a imagem tem de ser a MESMA).
