# Handoff de continuação — Luz 2D, W5t: a subida paga por fio (2026-10-15)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6`, por cima da W5q ([handoff](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md)), da W5r
> ([handoff](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md)) e da W5s
> ([handoff](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md)), todas não integradas. Commits da W5t:
> `7be12178f8` (perf: despacho 3D) · `707c659e04` (test: a régua). Nada integrado, nada enviado. O smoke da W5s
> continua pendente do dono. Placa RTX 5060 Ti (448 GB/s, L2 32 MB); grade do High 2496×1472.

## 0. A onda inteira, pelo produto

Inspector «custo sem o Automático» (`a_nitidez_da_luz`, 6 corridas alternadas, um processo cada):

| | 1D (`PH2D_LIGHT2D_HRC_3D=0`) | 3D (W5t) |
|---|---|---|
| High, ms | 10,92 · 11,19 · 11,08 | **10,31 · 10,40 · 10,85** |
| Low, ms | 2,17–2,76 | **2,13–2,65** |

⚠️ As corridas 1 e 6 tinham um app de outra linha na placa: o A/B vale, os absolutos estão inflados.

Barra do app (cena 5, High manual, `fotografa_cena.sh`, HOME isolado com cópia das prefs): **57 · 48 fps** (3D)
contra **56 · 55** (1D). Presa ao teto de 60 da tela e com o app da `line-UIUX` na placa em todas: NÃO compara.
A cena é igual ao píxel nos dois lados (`compare` AE = 0 no recorte da cena; só o texto da barra muda).

## 1. Medir antes de mexer: a premissa «limitada pela memória» caiu

A pergunta da onda: a subida e a descida (84 % da `c2`) eram limitadas pela MEMÓRIA, logo ler/escrever menos
bytes ganharia. Construídos e medidos os DOIS cortes de bytes, ambos AO BIT (régua no mesmo processo, os 32 casos
da W5s × Low/High × 3 variantes: 0 bytes diferentes; dois controlos por texto do módulo mutado via
`Config::modulo_rc` — «o mapa lido sempre vácuo» e «o b do R lido do vizinho» — mudaram os 32 casos cada).

**(a) T esparso.** Um mapa de 1 bit por entrada de T (por quadrante e nível, ~28 MB no High): quem escreve T não
grava o VACUO, quem lê não o lê (lei por construção: o leitor substitui o mesmo VACUO que leria). Contagem
(instrumento que leu o mapa de volta; saiu da árvore):

| cena | T não-vácuo | bytes por quadro |
|---|---|---|
| High, cena 1 | n0–n4 0,0 % · n8 0,6 % · n12 4,6 % | traça 558 → 9 MB · sobe 2 556 → 64 MB · desce 4 433 → 2 797 MB |
| Low, caixa 2,5 | n0 1,0 % … n6 35,6 % · n10 67,9 % | sobe 136 → 52 MB |
| cena cheia | — | sobe 2 556 → 2 564 (nada a saltar) |

**(b) R em 6 bytes.** O 4.º meio sempre 0 não se guarda: rg num u32 + b de duas entradas noutro; escrita por
grupo em memória partilhada. Desce 2 797 → 2 107 MB, soma 118 → 88.

**Resultado no regime rápido** (`c2` do High por processo, `lastro`, 2×), ms:

| | antes | esparso | r6 | os dois |
|---|---|---|---|---|
| vazia | 7,89 | 8,48 | 8,87 | 9,34 |
| cheia | 10,24 | 13,01 | 11,24 | — |

Por etapa (vazia): sobe 0,21–0,23 → 0,27–0,29 por nível (esparso, quase sem bytes!); desce 0,34 → 0,375
(esparso) e → 0,435 (r6). ⇒ A premissa NÃO vale depois da W5r (por colunas): a desce move ~360 MB lógicos por
nível em 0,34 ms (> 2× a DRAM: o L2 serve); a sobe esparsa, sem ler nem gravar, custa MAIS que a antiga ⇒ o
custo é POR FIO (a conta do índice, as barreiras e os atómicos do grupo, a leitura dependente do bit). Ambos
RECUSADOS; o código está na tag local `w5t-recusados-esparso-r6` (commit `d2aaf141c6`, fora da linha).

## 2. ⛔ A armadilha do regime lento

(W5p/W5q/W5r já a conheciam.) O mapa de 28 MB criado DEPOIS da «dança» do `hrc_nasce_duas_vezes` pôs TODOS os T
e R no regime lento: o binário novo com as chaves desligadas dava **19,3 ms** contra 7,7 do antigo (intercalado
por processo, 3×); a sobe custa ~0,59 por nível, como na W5r. Experimento (saiu da árvore): o mapa criado ANTES
da dança 7,91 · DENTRO dela 7,89 · sem mapa 7,81 · os tamanhos de T/R antigos com o mapa depois 19,30.

**Lei:** buffer GRANDE novo da holográfica nasce dentro do lote que renasce (ou antes), nunca depois. E a régua
«4 instâncias no mesmo processo» caiu no lento (High 1D 20,2 na cena 1 da régua do 3D): as etapas comparam-se por
processo.

## 3. O que entrou: o despacho em 3D (`7be12178f8`)

`Config::hrc_grade_3d` (`true`; só por colunas; a traça só com um fio por sonda): grupos [⌈Y/64⌉, nx, k] na
traça (k = 1), na subida (2ⁿ+1) e na descida (2ⁿ); `(v, xi, k) = global_invocation_id`, sem as divisões do 1D
(decompor). Override `GRADE3D_HRC` no `c2_hrc.wgsl`; passos com `[u32; 3]`. App: `PH2D_LIGHT2D_HRC_3D=0` compara.

`c2` do High por processo (`lastro`, 2×, placa só da linha, load 2–8), ms:

| | antigo | 1D novo | 3D |
|---|---|---|---|
| vazia | 7,68 · 7,71 | 7,69 · 7,71 | **7,23 · 7,23** |
| cheia | 10,04 · 10,07 | 10,07 · 10,06 | **9,42 · 9,41** |

Low por processo, 2×: vazia 0,556 → **0,523 · 0,522**; cheia 1,488 · 1,484 → **1,418 · 1,416**.

Régua no mesmo processo (`medida_luz2d_w5t_3d`, 32 casos): **0 bytes diferentes** em todos; controlo (a subida
3D a ler a linha v−1) mudou os 32. Low −2 a −4 % (cena 1 +3 %: 0,613 → 0,634); High igual a −2 % (instâncias no
lento, §2).

Etapas (High vazia): sobe n7–n10 0,21–0,27 → 0,18–0,21 (−15 a −22 %), n11 0,345 → 0,301; desce n10–n11
−13/−15 %, n0–n9 −1 a −4 %; traça n0 0,340 → 0,303. ⇒ a subida n10 passa de 63 a ~92 M entradas/ms (n8–n9 ~86):
o item §6b.2 da W5s (n10–n12 lentos, «talvez o L2») era a conta do índice — **fechado**.

## 6b. ABERTO, na ordem

1. A descida (≈ 3,9 ms, metade da `c2`) quase não ganhou com o 3D nos níveis do meio: o custo é por fio e não de
   bytes. Candidatos com mecanismo a provar: mais trabalho por fio (um fio por sonda nos níveis baixos partilha as
   leituras de T entre cones vizinhos); menos instruções de desempacotar.
2. Um T esparso SEM barreiras nem leitura dependente (ex.: ballot de subgrupo) — só vale se a cena cheia não
   piorar (a versão 1D piorou +27 %).
3. O pior caso do Low (W5s §2, +0,8 %) — decisão do Enio.
4. O instrumento das etapas com `PH2D_W5S_NIVEL=low` não termina (pré-existente, W5s §6b.5); nesta onda o Low
   mediu-se pela `c2`.
5. ⚠️ `docs/Components/ferramentas/fotografa_cena.sh` tem crases dentro de um heredoc sem aspas (linhas ~116–117:
   `[env]`, `.cargo/config.toml` executados como comando a cada foto: «comando não encontrado» / «Permissão
   negada»). Não é da luz: reportar ao dono do roteiro.

## ⛔ Recusas MEDIDAS e cuidados

| tentativa | medido |
|---|---|
| T esparso (mapa de 1 bit) | sobe 0,22 → 0,28 por nível com quase zero bytes; cheia 10,24 → 13,01 (§1a) |
| R em 6 bytes | desce 0,34 → 0,435 por nível; `c2` 7,89 → 8,87 (§1b) |
| «a subida/descida é limitada pela memória» (depois das colunas) | a desce serve ~360 MB lógicos por nível em 0,34 ms (> 2× a DRAM); o custo é por fio |
| buffer grande criado depois da dança | 19,3 contra 7,7 ms (§2) |
| régua «4 instâncias no mesmo processo» para etapas | cai no lento; compare por processo |
| (da W5s/W5r/W5q, continuam) | [W5s](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md) · [W5r](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md) · [W5q](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md) |

## Para o integrador

- Foundational tocada: só `ph2d-light2d-gpu`. Campo PÚBLICO novo `Config::hrc_grade_3d` (`true`; todo literal do
  workspace usa `..`). Override novo `GRADE3D_HRC`.
- O texto do `c2_hrc.wgsl` mudou: a 1.ª abertura depois da integração recompila a luz. Nenhum contrato congelado
  (§6).
- Testes: `tests/it/medida_luz2d_w5t.rs` (`#[ignore]`); o `lastro` lê `PH2D_W5T_3D`; `medida_luz2d_w5s` expõe
  casos/camadas/sprites como `pub(crate)`.
- A tag local `w5t-recusados-esparso-r6` é só arquivo: NÃO empurrar.

## Gate e binário

Gate do fecho (verificador, régua = merge-base `0aab2c1cb6`, load 12–19 → 37): fmt verde; clippy `-D warnings`
(forma do `ship.sh`, `PH2D_SUITE=fecho`) verde; check `--workspace --all-targets` deny verde; censos 117/117 e
12/12; `nextest-impacted` (`PH2D_SUITE=fecho`, `PH2D_GPU=1`) **11 753 / 11 753**. Por nome: `g3_` 8/8, `g4_…luzes`,
`g5_` 2/2, `holografica` 16/16, `g12_` 2/2, `o_topo_passa_a_grade`, `a_nitidez_da_luz` 6/6.

Cache do dono (`fotografa_cena.sh`, `__GL_SHADER_DISK_CACHE_PATH=/home/enio/.cache/nvidia`, `PH2D_LIGHT2D_SMOKE=5`,
HOME isolado com cópia das prefs em High): 1.ª abertura **1,34 s** (42 pipelines), 2.ª **0,06 s**; `GLCache`
1 387 → 1 383 MB. `target/*/incremental` reclamado (3,4 GB). Binário do smoke (2.ª corrida, depois de `707c659e04`):

```
▸ linha line_luz2d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

Régua do loop (`agent-loop-profile.sh`): paralelismo 1,16 (✗ ≥ 1,5) · `test:check` 2,6× (✗) · contexto por
passo 277 mil (✗ ≤ 250 mil) · edições sem conferir 0,9 % (✓).

## Smoke do dono

⏳ Pendente (o da W5s também).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_TEMPO=1 PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. A cena de sempre: lâmpada no centro, oito formas coloridas com sombra. No terminal «a luz acendeu N s», N < 0,5.
3. As prefs já estão em High manual. Clicar a **Lâmpada**: o Inspector diz «Quality: High (manual), NN ms of GPU
   per frame» — ~10,3–10,4 (W5t; antes ~10,9–11,2). Barra ~57 fps (teto da tela). Arrastar a lâmpada depressa.
4. Comparar: `PH2D_LIGHT2D_HRC_3D=0` à frente — cena IGUAL, o número ~0,6 ms mais alto. Errado: imagem diferente
   entre os dois, ou o High acima de 12.
