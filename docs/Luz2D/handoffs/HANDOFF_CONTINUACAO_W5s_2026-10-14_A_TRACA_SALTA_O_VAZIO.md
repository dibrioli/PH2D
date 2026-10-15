# Handoff de continuação — Luz 2D, W5s: a traça salta o vazio (2026-10-14)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6`, por cima da W5q ([handoff](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md)) e da W5r
> ([handoff](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md)), ambas não integradas. Commits:
> `f4adfe29b1` (a traça salta o vazio + um fio por sonda) · `397dd0196c` (a dobra justa do despacho) ·
> `8e59ebf1e5` (o fmt) · `40f5b518c7` (o G4 com controlo). Nada integrado, nada enviado. Método:
> [MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md); placa conferida antes de cada rodada.

## 0. A onda inteira, pelo produto (A/B intercalado, 3×)

⚠️ O app da `line-PainterWatercolor` estava aberto na placa (6–20 %) nessa rodada: o A/B vale, os absolutos
estão inflados dos DOIS lados (a W5r mediu 14,73 em placa limpa).

| | antes (binário da W5r, `ad484f9c52`) | W5s |
|---|---|---|
| High, Inspector «custo sem o Automático» (`a_nitidez_da_luz`), ms | 15,08 · 15,77 · 15,54 | **10,98 · 12,29 · 11,83** |
| Low, ms | 2,37 · 2,37 · 2,38 | **2,17 · 2,34 · 2,17** |
| `c2` do High por processo (`lastro` low, mesmo par de binários), ms | 11,48 · 11,51 · 11,50 | **7,71 · 7,71 · 7,74** |
| abertura a frio (`medida_luz2d_w5o_abertura`, cache vazio): primeira luz · estufa | 1,24–1,28 · 2,84–2,88 s | 1,28–1,30 · 2,93 s (igual a ±0,05; uma pipeline a mais, `hrc_ladrilhos`) |
| barra do app, High manual (cena 5) | W5r: 50 fps · 20,0 ms | **59 fps · 16,9 ms** e **58 fps · 17,1 ms** |

A barra: prefs do dono com `light_quality=high` só na cópia do HOME isolado do `fotografa_cena.sh`; placa só da
linha. `PH2D_LIGHT2D_HRC_TRACA=0` (a traça antiga, já com a dobra nova): 52 fps · 18,9 ms. Colado ao teto de 60
da tela. Fotos: 46 píxeis diferentes entre final e traça antiga contra 33 entre duas fotos finais (ruído).
⚠️ As prefs do dono (`~/.ph2d/prefs.txt`) estão hoje em `light_quality=low`, `light_auto=0`.

## 1. Medir antes de mexer

**(a) O vazio domina.** Contador na placa (override `hrc_traca_conta`, SAIU da árvore): fração dos intervalos
T0–T2 que só cruzam texels SEM cobertura.

| cena (High) | intervalos vazios |
|---|---|
| cenas 1–7 e 8 | 98–99,9 % |
| 3→3 [2,0] alt 2 | 95,7 % |
| 4→4 alt 1 | 91,5 % |
| caixa 2,5 | 88,7 % |
| Flip (camada) | 98,8 % |
| Motion + sprites com alfa | 99,1 % |
| cena cheia | 0 % |

Low semelhante (caixa 2,5: 85–87 %). A máscara por ladrilho 8×8 fica a ≤ 1,5 ponto do critério exato texel a
texel; o leque da sonda a ≤ 0,5 ponto do raio a raio.

**(b) As duas premissas da W5r §6b.1.** Premissa 1 VALE: a holográfica lê `COBERTURA` na escala 1 SEM a borda
exata (`BORDA_EXATA` é `false` nos pipelines dela) — dentro ⟺ bilinear ≥ ½; com as quatro amostras do gather a
0 é sempre «fora» (τ 1, `luz` falsa) e o traço devolve o VACUO exato (a emissão só é lida quando dentro). Mesmo
com a borda exata o texel `floor(p)` está entre os quatro do gather. Premissa 2 NÃO VALE para as caixas do CPU:
`ladrilhos.rs` (W5q) é da captura FINA e só dos opacos (`capturar_opacos`: sem emissores), e as camadas do
Flip/Motion entram como UM sprite do tamanho da grade. ⇒ a máscara vem da própria captura, na placa.

## 2. Os cortes da traça (`f4adfe29b1`)

- **`hrc_ladrilhos`**: um u32 por ladrilho 8×8 da grade efetiva, um grupo 8×8 por ladrilho, `atomicOr` em
  memória do grupo. Corre UMA vez por quadro (a cobertura não muda entre os rebotes; o Low corre a holográfica
  2× por quadro).
- **`hrc_traca` um fio por SONDA** (`Config::hrc_fio_por_sonda`, o `SONDA_HRC`): os 2ⁿ+1 intervalos num laço, o
  meio em A lido uma vez; o salto pelo LEQUE (triângulo A, B₀, B_{2ⁿ}; `Config::hrc_salta_o_vazio`, o
  `SALTA_O_VAZIO_HRC`), folga 0,001 para o arredondamento. Ambos `true` por omissão.

Régua `medida_luz2d_w5s_salta` (mesmo processo, 4 variantes antes/salta/sonda/os dois × 32 casos: os 15 da W5q +
Flip pela camada + Motion e sprites com alfa (um que tapa, um que brilha) + cena cheia, Low e High), com o
CÓDIGO FINAL: **0 bytes diferentes nos 32 × 3**; controlo (`Solver::hrc_sem_mascara`: a máscara não escrita,
todo raio salta) muda os 32.

Etapas por processo (High vazia): traça 3,36 → **0,65 ms** (n0 0,34 · n1 0,19 · n2 0,13); sobe, desce e saída
iguais; a máscara 0,046 ms (o 1.º desenho, um fio por ladrilho a ler 64 texels, custava 0,091). `c2` por
processo, High: vazia 10,95 → 8,30 · cheia 11,18 → 10,68 (só os cortes).

Low por processo (3×, placa limpa): vazia 0,705–0,712 → 0,553–0,554 ms (−21 %); cheia 1,472–1,475 →
1,484–1,488 (**+0,012 ms, +0,8 %**: a máscara e a verificação, nada a saltar). O 1.º desenho dava +0,03: as duas
`textureDimensions(cobertura)` por verificação (na NVIDIA, consultas à unidade de textura, a que já limita a
traça) trocadas pelo uniforme `cas.tamanho` (= grade efetiva = tamanho da textura; o solver guarda a
`grade_efetiva` na config) tiraram 2/3. Pior caso do High: melhor (régua do mesmo processo 21,25 → 20,95; por
processo 11,18 → 10,05 com a dobra).

## 3. A dobra justa (`397dd0196c`)

A hipótese da W5r §6b.2 (T10–T12 de 36–48 MB passam o L2 de 32 MB) **CAIU**: as etapas lentas eram exatamente os
despachos com > 65 535 grupos, dobrados em [65 535, 2] com 30–45 % da 2.ª fila VAZIA. T_n por quadrante: n3–n9
28–32 MB, n10 35/39, n11 46/39, n12 46. Dobrados por igual ([⌈g/y⌉, y], `grupos_1d`), sem mudar memória: sobe
n≥10 1,20 → 0,81 ms (41–46 → 63 M entradas/ms; n3–n9 72), desce n10–n11 0,95 → 0,71; `c2` por processo vazia
8,30 → 7,67 · cheia 10,68 → 10,05. O custo era lançar os grupos vazios. Ao bit (5.ª variante da régua, 32 casos,
0 bytes).

## 4. O G4 (`40f5b518c7`)

`g4_o_custo_da_holografica_nao_depende_do_numero_de_luzes` ficou vermelho no gate do fecho (5,11 · 5,21 %, load
4,5–5,4; verde 1 de 3 a 4,89). Medido: no código da W5r (binário de `ad484f9c52`, intercalado) já dava
4,39–4,87 %; depois 4,41–5,68 % (a base ~0,02 ms mais baixa). O Δ por luz é igual com a traça antiga e a nova
(0,034–0,038 ms nas 4 variantes).

**Mecanismo: a placa cobra os VALORES, não as luzes.** 1 luz + céu + as 9 999 formas translúcidas de opacidades
variadas custa o MESMO que 10 000 luzes (0,791–0,794 contra 0,792–0,795; 1 luz 0,751–0,754; só o céu, uniforme,
não muda nada). O que cresce é a soma (0,048 → 0,058–0,066 ms), que não tem conta por luz — a compressão de
dados uniformes da placa é o candidato, NÃO provado.

**Cura:** o G4 mede no mesmo solver um CONTROLO (1 luz, céu, translúcidos variados); a lei: nenhuma contagem de
luzes > 5 % acima do maior entre 1 luz e o controlo. Medido 3×: −0,58 · +0,25 · +0,28 %. Prova de mutação:
controlo verde (+0,43 %); M1 64 amostras extra por ENTRADA num emissor → VERMELHO (+24,31 %); M2 a mesma conta em
TODA amostra → verde (+0,15 %; as cenas sobem por igual 0,75 → 1,7 ms, o que a lei não vê por desenho);
restaurado, verde.

## 5. O que o número do Inspector passa a significar

Passa a depender do **CONTEÚDO**: a traça custa onde há cobertura (num ladrilho 8×8 com algo, ou vizinho dele);
o vazio é quase de graça. Cena típica −27 % no High; o pior caso (todo ladrilho 8×8 com conteúdo) é o de antes
(High um pouco melhor, Low +0,8 %). O G4 continua a valer: a lei é o número de luzes.

## 6b. ABERTO, na ordem

1. A traça n0 (0,34 ms) agora escreve VACUO em T0 (~235 MB): a sobe/desce também poderiam ler a máscara (T de
   vácuo funde em vácuo) — não medido.
2. Sobe n≥10 a 63 contra 72 M entradas/ms: resíduo (talvez o L2), ~0,1 ms.
3. A desce é a maior etapa (4,16 ms) e a sobe 2,3.
4. O R com a 4.ª metade sempre 0 (W5r §6b.3) e os itens 2–4 da W5q §6b continuam.
5. ⚠️ O instrumento das etapas (`PH2D_W5Q_FASES=1`) com `PH2D_W5S_NIVEL=low` não termina (pré-existente: com todas
   as chaves desligadas também) — no Low meça só a `c2`.

## ⛔ Recusas MEDIDAS e cuidados

| tentativa | medido |
|---|---|
| o L2 como causa das etapas lentas | a dobra justa devolveu a taxa sem mudar memória (§3) |
| o campo de distância (JFA, `livre()`) como teste de vácuo | uma leitura só, mas o JFA pode errar a semente mais próxima e então não é exato; não medido, recusado por desenho |
| a máscara das caixas do CPU | premissa 2 (§1b): `ladrilhos.rs` é da captura fina e só dos opacos |
| a verificação RAIO a RAIO | o pior caso pagava-a em todo raio; o leque amortiza |
| `textureDimensions` na verificação | +0,018 ms no Low cheio; vale o uniforme `cas.tamanho` |
| a máscara por um fio por ladrilho a ler 64 texels | 0,091 ms contra 0,046 com o grupo 8×8 |
| relógio com outra linha na placa | o app e a suíte da `line-PainterWatercolor`: High 15,9 contra 14,7 |
| (da W5q/W5r, continuam) | [handoff W5r](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md) |

## Para o integrador

- Foundational tocada: só `ph2d-light2d-gpu`. Campos PÚBLICOS novos `Config::hrc_salta_o_vazio`,
  `Config::hrc_fio_por_sonda` (`true`; todo literal do workspace usa `..`).
- Buffer novo `luz2d/buf/hrc_com_cobertura` (`hrc::buffers`; G5 verde); `hrc::n_passes` +1. Entrada nova
  `hrc_ladrilhos` no `c2_hrc.wgsl` (binding 16 do grupo 3; chama-se `com_cobertura` porque `ladrilhos` já existe
  no módulo pelo `c3_anel.wgsl`). `Holografica::gravar(p, k)` (crate). Instrumento `Solver::hrc_sem_mascara`
  (`#[doc(hidden)]`).
- App: `PH2D_LIGHT2D_HRC_TRACA=0` (a traça antiga). Testes: `tests/it/medida_luz2d_w5s.rs` (`#[ignore]`),
  `Vista::quadro_com` (W5p); o `lastro` lê `PH2D_W5S_SALTA`, `PH2D_W5S_SONDA`, `PH2D_W5S_CENA=cheia`,
  `PH2D_W5S_NIVEL=low`. G4 reescrito (`holografica.rs`).
- O texto do `c2_hrc.wgsl` mudou: a 1.ª abertura do dono depois da integração recompila a luz. Nenhum contrato
  congelado (§6).

## Gate e binário

Gate do fecho (verificador, régua = merge-base `0aab2c1cb6`): fmt (curado em `8e59ebf1e5`); clippy `-D warnings`
(forma do `ship.sh`, `PH2D_SUITE=fecho`) verde; check `--workspace --all-targets` deny verde; censos 117/117 e
12/12; `nextest-impacted` (`PH2D_SUITE=fecho`, `PH2D_GPU=1`, load 19 → 38) **11 753 / 11 753**. Por nome, depois
do `40f5b518c7` (o G4 era o único vermelho): `g3_` 7 (+1 ignorado), `g5_` 2, `holografica` verde.

<!-- BINARIO -->

## Smoke do dono

⏳ Smoke do dono pendente.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_TEMPO=1 PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. A cena: a lâmpada no centro, oito formas coloridas com sombra e o interior delas iluminado (a mesma de
   ontem). No terminal, «a luz acendeu N s» — N abaixo de 0,5 s.
3. As suas preferências estão em Low manual: **Settings ▸ 2D Light ▸ High** (Automatic desligado). Clique a
   **Lâmpada**: o Inspector diz «Quality: High (manual), NN ms of GPU per frame» — ~11–12 (ontem ~15). A barra de
   baixo: ~59 fps (ontem ~50), no teto da tela. Arraste a lâmpada depressa pelas formas: as sombras seguem sem
   tranco, agora com folga.
4. Comparar: `PH2D_LIGHT2D_HRC_TRACA=0` à frente do comando — a cena IGUAL, a barra ~52 fps e o número sobe ~3 ms.
   Errado: imagem diferente entre os dois, ou o High acima de 14.
