# Handoff de continuação — Luz 2D, W5w: o fecho e o regime da c2 (2026-10-15) — FECHA a linha (W5q–W5w)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D` sobre o `main`
> `0aab2c1cb6` (o main não andou), por cima da W5v ([handoff](HANDOFF_CONTINUACAO_W5v_2026-10-15_O_FUNDO_DOS_OPACOS.md)),
> W5u ([handoff](HANDOFF_CONTINUACAO_W5u_2026-10-15_O_CUSTO_POR_FIO.md)), W5t
> ([handoff](HANDOFF_CONTINUACAO_W5t_2026-10-15_A_SUBIDA_PAGA_POR_FIO.md)), W5s
> ([handoff](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md)), W5r
> ([handoff](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md)) e W5q
> ([handoff](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md)), todas não integradas. Nada integrado, nada
> enviado. Commits da W5w: `14f1f3dcfa` (feat: o instrumento do custo por janela + renascer no processo +
> `docs/Luz2D/ferramentas/mede_custo_no_app.sh`) · `166265f7a8` (style: `cargo fmt` de `quadro.rs`,
> `quadro_ambiente.rs` e `tests/it/medida_luz2d_w5v.rs` — o fmt vinha vermelho desde a W5v). O dono aceitou o 1 ulp
> da W5v em 15/10 (`Config::miolo_salta_o_fundo = true`). Placa RTX 5060 Ti. `c2` = ms de GPU do passe holográfico.
> Este handoff é o ÚLTIMO da linha: o link do CLAUDE.md §5.1 «Luz 2D / Último:» passa a ser este.

## 1. Gate batched do fecho (régua = merge-base `0aab2c1cb6`)

- `nextest-impacted` (`PH2D_SUITE=fecho`): **11 753 corridos, 11 751 verdes, 2 vermelhos de RELÓGIO de placa**:
  `o_custo_do_miolo::o_custo_do_miolo_desce_com_a_base` (razão 0,804, teto 0,8; tempos ~2,0/2,5 ms contra 0,72/1,07
  em placa calma) e `g4_custo_independe_das_luzes::g4_o_custo_do_rc_nao_depende_do_numero_de_luzes` (14,09 % contra
  5 %). Durante a corrida estavam na placa o app do dono (68 % SM) e o de outra linha (`line-motion-value`), load
  ~129; isolados, ainda com esses apps, alternaram verde/vermelho, e o controlo do miolo (o custo antigo contra si
  mesmo) chegou a reprovar ⇒ ruído de placa partilhada, não regressão. Com a placa livre (só kwin): **3/3 verdes
  cada um**. É um membro da família [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) por PLACA partilhada
  (`PH2D_GPU=1` não exclui apps abertos fora do `ph2d-run`); NÃO promovido aqui (§7 item 4).
- `check --workspace --all-targets` com `CARGO_BUILD_WARNINGS=deny`: verde. `clippy --workspace --all-targets -D
  warnings` (`PH2D_SUITE=fecho`): verde. `check-standalone-optional` (11 crates) e `check-workflow-packages` (32
  nomes / 394 membros): verdes. `cargo machete`: verde. `censos-da-arvore-combinada`: 117/117 (controlo do filtro
  12/12). `file_loc_caps` 4/4.
- **Delta W5w (depois do gate):** fmt verde (após `166265f7a8`), `check` deny verde, `clippy -p ph2d-app-light2d -p
  ph2d-host-desktop` verde; por nome: `redimensionar_a_janela_nao_recompila_a_luz` 1/1, `smoke_cena_1`,
  `smoke_cenas_w4`, `a_nitidez_da_luz`, `o_interior_do_opaco`, `o_miolo_das_formas` (exit 0), `file_loc_caps` 4/4,
  `architecture_workspace_file_loc_cap` 2/2. Holográfica: 7 passaram · 9 ignorados (as medições). `quadro.rs` 674
  LOC (teto 700).

## 2. O instrumento (W5w)

- `PH2D_LIGHT2D_CUSTO=1` imprime agora por JANELA de 60 leituras: «[luz2d] janela g<geração> w<janela> · total
  min/med · c2 min/med · fina min/med» (antes: uma amostra solta a cada 60). Uma leitura de custo sai a cada ~5
  quadros.
- `PH2D_LIGHT2D_RENASCE=<k>` larga o solver da vista a cada k janelas no MESMO processo (renasce no quadro seguinte).
- Roteiro versionado `docs/Luz2D/ferramentas/mede_custo_no_app.sh <larg> <alt> <low|medium|high> <quadros> <saida>
  [VAR=valor…]`: um processo do app na cena 5 numa `kwin --virtual` (X11 na Xwayland dela, HOME isolado com prefs
  `light_quality` / `light_auto=0`, cache de shaders do dono), relógio da placa (`nvidia-smi`) a cada 2 s em
  `placa.csv`; recusa binário mais velho que o código.

## 3. Números do app (o que o Inspector mostra)

Cena 5, rodada intercalada, 2 processos por caso, mediana do total por janela (ms), placa só com kwin (CPU com load
20–176 de outra linha em parte da rodada). Cada célula: processo 1 · processo 2 (intervalo das medianas).

| janela | Low | Medium | High |
|---|---|---|---|
| 1920×1080 (AOC; vista 1920×1052) | 7,2–7,4 · 7,8–8,4 | 7,5–8,0 · 7,9–8,5 | 15,3–16,0 · 16,4–16,8 |
| 2560×1080 (LG) | 8,4–8,5 · 8,6–9,2 | 8,8–9,1 · 8,3–9,1 | 25,0–26,3 · 20,6–21,2 |

`fina` mediana: 1920 Low 2,87 (W5v antes 3,21) · Medium 1,83 · High 2,38; 2560 Low 3,48 (antes 3,83) · Medium 2,07 ·
High 3,0–3,2. `c2`: Low 1,24 ou 1,41–1,59 (também em degraus) · Medium 2,3–2,9 · High: §4.

## 4. Item 4 do §6 da W5v — a c2 do High varia entre processos: MEDIDO, sem corte

- Dentro do processo a c2 só muda quando o solver RENASCE e fica estável enquanto ele vive; o relógio da placa ficou
  fixo em P0 2 827 MHz / 14 001 MHz em todas as corridas ⇒ não é relógio nem processo: é ONDE a memória cai em cada
  montagem.
- Os valores caem em DEGRAUS discretos (c2, ms): 1920 High ~8,8–9,1 / ~9,5–10,3 / ~12,1–13,6 / ~14,5–16,6 (raro
  20–21); 2560 High ~11,5–11,9 / ~12,3–13,2 / ~16,6–17,2 / ~19–19,7 (raro 22–25). O piso (≈ 9,0 / 11,9) é o da régua
  `medida_luz2d_w5v_caso` (9,33).
- Com `hrc_nasce_duas_vezes=0` (`PH2D_LIGHT2D_HRC_DUAS=0`) a sequência por geração é determinística e igual entre
  processos (2560: 22,5 → 25,4 → 12,6 → 17,9 → 13,1 → 21,4 → 13,1 nos dois). Com `=1` (o padrão) a 1.ª montagem do
  app caiu em 15,3–16,4 (1920) e 16,6–18,4 (2560) em parte dos processos, e noutros em 8,9–10,6.
- **Pista para a próxima onda:** o buffer único (> bloco do `gpu-allocator` 0.28 ⇒ alocação dedicada) cai SEMPRE no
  lento; os separados (subalocados em blocos) caem no lento às vezes ⇒ o regime parece seguir o TIPO/sítio da
  alocação (dedicada × subalocada, página, compressão), não a distância. Próximo passo: ler o bloco/offset que o
  `gpu-allocator` dá a cada buffer (`wgpu-hal` `as_hal` / log do alocador) e correlacionar com o degrau, ANTES de
  qualquer corte. Ganho em jogo no app: 1920 High até −6 ms (15,3 → 9,0), 2560 até −8 ms.

## 5. Para o integrador

Nada de contrato congelado, sem schema, sem registo. A W5w só acrescenta o campo privado `custo_janela` em
`LuzDoQuadro` e o tipo `JanelaDeCusto` (`pub(super)`) em `quadro_leituras.rs`; o resto como no §4 do
[handoff W5v](HANDOFF_CONTINUACAO_W5v_2026-10-15_O_FUNDO_DOS_OPACOS.md) (`Config::miolo_salta_o_fundo`, override
`FUNDO_SEM_LUZ`, `UniformeNivel.exata`, `config_da_vista`/`config_do_entorno` públicas, `solver_miolo.rs` 697 LOC).
O texto dos shaders mudou desde o main ⇒ a 1.ª abertura recompila a luz. CLAUDE.md §5.1 Luz 2D: trocar só o link de
«Último:» para este handoff; a frase do módulo não muda.

## 6. Smoke do dono (cache e binário)

- Cache de shaders (`/home/enio/.cache/nvidia`, binário final, HOME isolado): a luz acende em 0,10 s (Low, 68
  pipelines) · 0,09 (Medium, 67) · 0,07 (High, 68) · 2.ª abertura High 0,07 · Low 0,06; zero erros; GLCache 1 567 →
  1 567 MB (quente). Foto da cena 5 na tela virtual 1920×1080 conferida (lâmpada no centro, oito formas com sombra,
  interiores iluminados).
- `target/*/incremental` reclamado (15 GB debug + 1,2 GB smoke). Binário do smoke, 2.ª corrida:
```
▸ linha line_luz2d · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.23s
```
- Régua do loop (`agent-loop-profile.sh`, 20 sessões): paralelismo 1,14 (✗ ≥ 1,5) · respostas/sessão 154 (✓) ·
  test:check 2,0× (✗ ≤ 1) · edições sem conferir 0,8 % (✓) · contexto por passo 296 mil (✗ ≤ 250 mil) · contexto
  inicial 65 mil (✓).

**Passos do smoke (para o dono; prefs dele hoje: `light_quality=low`, `light_auto=0`):**

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke` (abre em segundos, sem compilar).
2. Na cena: uma lâmpada no centro e oito formas coloridas com sombra. Clicar «Lâmpada» na Hierarchy ⇒ o Inspector diz
   «Quality: Low (manual), NN ms of GPU per frame».
3. Esperado no AOC (1920×1080, janela maximizada): Low ~7–8, Medium ~7,5–8,5, High ~15–17; no LG (2560×1080): Low
   ~8,5–9, Medium ~8,5–9, High ~21–26.
4. O High pode abrir num número e, fechando e abrindo o app, abrir noutro (ex.: 9 ou 15 no AOC) — é o achado do §4,
   não defeito novo.
5. Comparar com a W5v: com `PH2D_LIGHT2D_FUNDO=0` à frente do comando, a cena fica IGUAL e o Low ~0,4 ms mais alto.
   Deu errado se: a imagem difere entre os dois, há faixas pretas dentro das formas, ou o Low passa de ~9,5 no AOC.

## 7. ⛔ Recusas MEDIDAS

Código retirado; não reconstruir sem ler isto.

| Recusa | Medida |
|---|---|
| A vista nascer 2× ou 3× (monta, larga, poll, monta) | Não cura: 1920 nasce 2×: 15,6 / 9,7 / 9,7; 3×: 13,6 / 8,9 / 13,5. |
| Buffers grandes da holográfica (acumulado, T, R) num buffer SÓ com as fatias seguidas, a 2 MiB ou com desvio de 4 KiB × k | Determinístico mas SEMPRE o regime mais lento: 19,9–21,0 (1920) e 25,9–27,7 (2560); a distância entre fatias não muda nada ⇒ «partition camping» refutado. Exigiu declarar `t_a`/`t_b`/`r_cima` `read_write` no shader e no layout (o wgpu vê o uso por buffer inteiro e exige o mesmo acesso shader/layout). Bits da holográfica 7/7. |
| `read_write` sozinho, com os buffers separados | Neutro (= hoje). |
| As da W5v/W5u/W5t/W5s/W5r/W5q | Continuam: ver os [W5v](HANDOFF_CONTINUACAO_W5v_2026-10-15_O_FUNDO_DOS_OPACOS.md), [W5u](HANDOFF_CONTINUACAO_W5u_2026-10-15_O_CUSTO_POR_FIO.md), [W5t](HANDOFF_CONTINUACAO_W5t_2026-10-15_A_SUBIDA_PAGA_POR_FIO.md), [W5s](HANDOFF_CONTINUACAO_W5s_2026-10-14_A_TRACA_SALTA_O_VAZIO.md), [W5r](HANDOFF_CONTINUACAO_W5r_2026-10-14_HIGH_MAIS_LEVE.md), [W5q](HANDOFF_CONTINUACAO_W5q_2026-10-14_CUSTO_DO_QUADRO.md). |

## 8. ABERTO, por ordem

1. O regime da c2 do High (§4, pista do alocador).
2. Mais fina no Low (borda exata −0,9, passo, média no eixo — mudam a imagem ⇒ produto; W5v §6 item 3).
3. O §6b da W5u (a cheia por DRAM).
4. Os dois portões de relógio sob placa partilhada (§1): o integrador decide se entram em
   [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) (promover = tabela + faixa do nextest).

Smoke do dono: ✅ OK (15/10) — cobre W5s, W5t, W5u, W5v e W5w (binário deste handoff). A linha está pronta para integrar, por ordem do dono.
