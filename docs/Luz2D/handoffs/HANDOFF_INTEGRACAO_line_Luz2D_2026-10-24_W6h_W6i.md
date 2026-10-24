# Handoff de integração — `line/Luz2D` W6h–W6i para o `main` (fecho 2026-10-24)

> DIRETRIZ §1.5.9. A linha FECHOU e PAROU: não integrou nem fez push. Mecanismo de cada onda nos docs linkados:
> [W6h §1–§13](HANDOFF_CONTINUACAO_W6h_2026-10-23_O_CUSTO_DOS_CANTOS.md) (o §14 é substituído pelo §9 daqui) ·
> [W6i](HANDOFF_CONTINUACAO_W6i_2026-10-24_BUG7_POR_PLACA.md) (o §2 dele é substituído pelo §9 daqui) ·
> [BUGS §7](../BUGS_luz2d.md) (bug 7 e o miolo na integrada). Anterior:
> [W6b–W6g](HANDOFF_INTEGRACAO_line_Luz2D_2026-10-22_W6b_a_W6g.md).

## 1. Base

- Branch `line/Luz2D`, worktree `Worktrees/line-Luz2D/`. Merge-base com o `main`: **`b48f8b9512`** (a linha foi
  REBASEADA sobre ele em 24/10, sem conflito: 0 ficheiros em comum). HEAD final = `91bd2ba583` + o commit de docs
  deste handoff (o HEAD inclui este commit). `git log --oneline b48f8b9512..HEAD`: ~30 commits, 49 ficheiros.

## 2. O que a linha entrega

| Onda | O quê | Medido |
|---|---|---|
| W6h §1–§12 | o elo do teto dos cantos até ao cartão LUZ (gate + mutação); réguas versionadas do custo e do mesmo bit dos cantos; a régua de registos (`spikes/regua-registos`); a V2 É o `num_corpo`; duas reconstruções da fina MORREM (máscara de bits; duas pipelines +19…+29 %) | two-strikes; o dono fechou: o custo de hoje fica (+2…+12 % da C3 com os cantos). Produto reposto (32 casos, 0 bytes) |
| W6h §13 | a leitura do arquivo da borda SEM desvio (`c2_hrc.wgsl`) | 2160p Alta `c2` 49,1–49,4 → 46,2–46,3 ms (−3,0 ms, −6 %); 1080p Alta −6,2 %, 1440p Média −5,5 %, 2160p Baixa −5,3 %, 1080p Baixa −1,5 %; o mesmo bit |
| W6i / bug 7 | os 7 testes vermelhos da Radeon 610M curados POR PLACA ([BUGS §7](../BUGS_luz2d.md)): «mesmo bit» por placa (dono 24/10) · G5 por variação · f16 da holográfica ao mais próximo só onde a placa trunca (`o_pack_da_placa_trunca`) · lei de custo do miolo só onde os despachos cortados pesam ≥ 20 % | 23 testes verdes nas duas placas; custo do `c2` na RTX 0,996–1,001 (sempre ligada: +6–13 %, recusado) |
| item 4 | o miolo salta o fora (`Miolo::salta_o_fora`, `override SALTA_FORA`) | `c3_miolo` Radeon ×0,42–0,59 (11,28 → 6,22 ms no bug 7), RTX ×0,92–0,93; 0 texels |
| itens 1–3 | medições: a conta do `de_fora` (RECUSA), o regime da vista `P` (respondido), a família (b) e a r5b | §7 |
| robustez | `o_pack_da_placa_trunca` sem pânico (`883324c3ac`); wrapper da Radeon versionado (`radeon.sh`) | §5 |

O item 4, fases (Radeon bug7 / RTX bug7, ms): cascata desce (58) 3,415/0,267 · ciclo sobe (57) 1,834/0,177 · ciclo soma (6, nível 1) 2,336/0,081 · `c3_miolo` 11,28/0,82 (tabela inteira no BUGS §7). Na integrada o tempo segue o TAMANHO do nível (cada despacho do nível 1,
1280×768, 0,39 ms ≈ 40 MB a ~100 GB/s, a DRAM partilhada), não os despachos. A cura: os passes dos níveis ≥ 1 não
gravam o texel com `k.a < 0` (ninguém o lê); `PH2D_LIGHT2D_SALTA_FORA=0` volta ao antigo.

## 3. Fora das crates da luz (tudo ADITIVO)

`git diff --name-only b48f8b9512 HEAD` fora de `crates/ph2d-(app-)light2d*` e `docs/Luz2D`: só `Cargo.lock` (+2 linhas:
as dev-deps `ph2d-panel-inspector` e `ph2d-ui-testkit` no `ph2d-app-light2d`) e `spikes/regua-registos/` (workspace própria
fora do build principal: `Cargo.toml`, `Cargo.lock`, `src/main.rs`, dois `.py`). Nenhum ficheiro de shell, painel,
i18n, editor-core ou `ph2d-gpu` (`git diff --stat b48f8b9512 HEAD -- crates/ph2d-gpu` vazio).
Env var nova de produto: `PH2D_LIGHT2D_SALTA_FORA` (`=0` compara); as outras (`MAIS_PROXIMO_HRC`, `PH2D_CANTOS_*`, `PH2D_BORDA_TOPO_*`) são de instrumento.

## 4. Contratos e schemas — NÃO tocados

- `git diff b48f8b9512 HEAD -- . ':!docs' | grep -E '^[+-]' | grep -cE 'NodeOp|OpResolver|NodeManifest|RasterEditTool|CanvasPaintTool|PanelEvent|PROJECT_SCHEMA|ph2d-vector|ph2d-imageio|Serialize|Deserialize|serde'` → **0**.
- Nenhum formato gravado, nenhum ADR novo (último 0188), nenhum registo de ecs/render/script mexido, nenhum diagnóstico no
  produto; réguas `#[ignore]` registadas, sem órfãos. Rodar `bash scripts/collision-surface.sh` pelo caminho absoluto
  no integrador (os contadores são DELTA zero).

## 5. Gates e provas

- **Gates novos:** `a_leitura_da_borda_sem_desvio_nao_muda_um_bit` (0 texels nas 5 vistas; mutação M1 164 931 · M2 1 690 432);
  `o_custo_do_miolo::saltar_o_fora_do_miolo_nao_muda_um_bit` (0 de 3 932 160 texels; controlo um V-ciclo a menos 155 496 RTX /
  155 550 Radeon); `saltar_o_fora_nao_muda_um_bit_com_a_cena_a_mexer` (8 quadros — disco que anda/some/volta, região fina a
  mudar, translúcido 0,45, até 3 corpos, lâmpada a andar; 4 configs: fábrica · base fora do grupo · base fora + bloco 1
  (= `PH2D_LIGHT2D_CORTES=0`) · base no grupo + bloco 1; 0 texels por quadro nas duas placas; controlo 864 093 RTX / 864 154
  Radeon; 6,2 s na RTX, 74 s na Radeon); `o_teto_dos_cantos_chega_a_frase_do_cartao_luz`; os 4 `bug7_por_placa` +
  `bug7_o_mesmo_bit_por_placa` + holográfica.
- **Prova de mutação do miolo (NVIDIA), 4/4 morrem:** M1 relaxa salta também o anel → 157 707 texels · M2 desce salta os de
  dentro → 157 704 · M3 padrão desligado → morre no assert do padrão (hoje `const`) · M4 o salto no nível 0 → 617 654.
  Revisão independente (sonnet): nenhum leitor alcança o texel saltado.
- **Robustez (`883324c3ac`):** `o_pack_da_placa_trunca` não dá pânico — falha da medição (placa perdida, leitura recusada)
  responde «trunca» (paga a correção, o mesmo bit onde a placa arredonda), num âmbito de erro próprio que não se lê como recusa
  da montagem; `paga_o_mais_proximo` saiu do `solver.rs` (701 → 699). Outros do fecho: `dc2ca45bb3` (rustfmt),
  `91bd2ba583` (clippy; `.clone()` de `Config` Copy nas réguas, assert constante → `const`, `is_multiple_of`, a catraca das
  citações: o comentário de `medida_luz2d_cantos_bits` citava um script `.py`).
- **Wrapper da Radeon:** `docs/Luz2D/ferramentas/radeon.sh` (`9df9b299ee` + `458e020f75`): não exporta `PH2D_GPU_VEZ` e segura o
  `ph2d-gpu-testes.lock` com `flock -w 1200` (exit 75 se ocupado); o antigo pendurava desde a fila da integrada do main.
  Uso: `bash scripts/ph2d-run.sh bash docs/Luz2D/ferramentas/radeon.sh cargo test …`; confirmar com
  `cargo run -p ph2d-gpu --example o_que_a_placa_anuncia` (só `RADV RAPHAEL_MENDOCINO`/`radeonsi`).

## 6. O fecho (sobre o diff acumulado desde `b48f8b9512`)

- `nextest-impacted` (`PH2D_SUITE=fecho`): NVIDIA **4369 passados / 1 falhado / 26 816 pulados** (784 s, load ao fim 132);
  Radeon pelo wrapper 4369 / 1 / 26 816 (1674 s). A única falha, nas duas:
  `ph2d-editor-core::architecture_no_restricted_source_citations::as_citacoes_em_comentario_so_descem` (a citação do `.py`),
  curada em `91bd2ba583` e verde por nome.
- Arch-gates da shell (`arch_safe_clamp_only`, `architecture_the_shell_only_shrinks`, `architecture_workspace_file_loc_cap`,
  `file_loc_caps`): verdes. clippy `--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings` (a linha do
  `ship.sh`): verde depois de `91bd2ba583`. Testes do `o_pack_da_placa_trunca` verdes nas duas placas depois de `883324c3ac`.
- ⚠️ `cargo fmt --all --check`: a linha limpa; ✗ só em `crates/ph2d-gpu` (`sonda_onboard.rs`, `lib.rs`, `vez.rs`,
  `tests/it/a_vez_da_placa.rs`) — HERDADO do `main` `b48f8b9512`, não da linha. Do integrador.
- Flakes: nenhum. Auditoria (2 lentes, sonnet): lente 1 (correção) sem defeito de resultado (o pânico, curado);
  `mesmo_bit_por_placa` é permissiva na RADV (≤ 1 ULP do f16 em qualquer texel — decisão do dono, por placa; na NVIDIA exige
  0); a lei de custo do miolo («só onde os despachos cortados pesam ≥ 20 %») só confere na RTX — na Radeon cobre-a o teto
  `a_base_do_miolo_corta_os_despachos`; `Solver::config()` devolve a config EFETIVA (a correção `false` onde a placa
  arredonda): reconstruir um Solver noutra placa a partir de uma `config()` clonada perde a correção (nenhum chamador do
  produto o faz). Lente 2 (regras): §3 e §4 acima.
- Perfil do loop (`agent-loop-profile.sh`, 20 sessões): paralelismo 1,16/passo (alvo ≥ 1,5) ✗ · respostas por sessão 142 ✓ ·
  test:check 2,3× ✗ · edições sem conferir 9,7 % ✗ · contexto relido 448 mil ✗ · contexto no início 66 mil ✓.
  `rm -rf target/*/incremental` feito (15 GB).

## 7. Medições da jornada (itens 1–3; nada no produto)

**Item 1 — a conta do `de_fora` em n 11–12 a 2160p Alta (~2,8 ms): RECUSA.** Instrumento
`ferramentas/item1_desce_um_diag.patch` (logs `target/item1/`). `c2` da vista − A (mínimo / mediana pareada); NVIDIA rodada
válida A×A −0,03/−0,04, A = 47,26 ms; Radeon A = 1365,6 ms, A×A +6,8/+1,3:

| forma | NVIDIA | Radeon | bit NVIDIA | bit Radeon | registos NV / VGPR AMD |
|---|---|---|---|---|---|
| Fa (as 2 sondas do `de_fora` desdobradas) | +0,07/+0,16 | +2,1/−0,9 | 28 texels (1 ULP): reprova | 0 | 38/32 |
| Fc todos os níveis (um cone por fio, `hrc_desce_um` sem `if` de uniforme, C1′) | +2,98/+3,05 (+6,5 %) | +206/+225 (+16,5 %) | 0 | 0 | 32/24 |
| Fc 9–12 | +0,41/+0,55 | +76/+67 | 0 | 0 | 32/24 |
| Fc 10–12 | +0,29/+0,23 | +48/+46 | 0 | 0 | — |
| Fc 11–12 | +0,14/+0,07 | +22/+37 | 0 | 0 | — |

Por nível (fases, NVIDIA): um cone perde em todos de 0 a 10; 11–12 no ruído (±0,15). O ganho «~−1,5 ms nos níveis 9–12»
do W6g §43 (d) não existe no código de hoje. F-b (inteiros para o uniforme) não foi construída: mexe no layout do uniforme
por ganho pequeno — fica ABERTA só se houver ideia de bit.

**Item 2 — o regime da soma da vista `P`: respondido** (`medida_luz2d_regime_app`, `9ee32a6ce8`, lote
`ferramentas/regime_app_abab.sh`; só NVIDIA; 12 processos da app + 13 de controlo válidos; logs `target/item2/`). O regime
da régua vem de DUAS vistas vivas: `P` sozinha 46,2–48,8 ms = a `identidade`, em 13/13; depois de a `identidade` nascer ao
lado, `P` 53,6–61,9 (placa a 11,5–14,2 de 16,3 GB, não esgota). Sem fases, 2160p Alta: `P` +7,3…+13,3 ms sobre a
`identidade`; soma n 0 (fases finais) `P` 12,5–15,2 vs 5,4–7,0. Não há regime a 1080p Alta (10,32–10,40 todos). A app real:
ao nascer cai 12/12 (+4,5…+8 ms — o regime das páginas novas do W6g §49, que o vigia já desfaz); renascida: r1 não cai
(5/6 = `P` sozinha), r2 (load 1–9) +3,5 ms acima de `P` sozinha em 6/6, a meio caminho, nenhum ao nível de `P`.
Calibração 1080p: a app custa +3,5 % sobre o solver cru.

**Item 3 (b) — a família (b).** A origem é o W6d «O disco em zoom» §3
(`HANDOFF_CONTINUACAO_W6d_2026-10-18_O_DISCO_EM_ZOOM.md:76-85`); o «37» é a contagem de texels pretos, NÃO uma secção:
a referência «W5g §37–38» dos handoffs W6g–W6i estava errada. Re-medida com `medida_luz2d_w6d_borda` (NVIDIA): antes da W6d
40, lei W6d 37, padrão de hoje 38 (+1 da imagem a zoom 3,2); por zoom (polígono/imagem) 0,8 → 5/1 · 1,6 → 7/2 · 3,2 →
10/13; os pretos ficam logo dentro da borda (d 0 a −0,44 texel) a 87,6°–93,7° da lâmpada (o terminador do disco); placa
0,06–1,2 contra o gabarito 0,43–1,50; alheia à borda (10 com a lei W6d, a cunha e a 10 + 9; 17 sem a borda exata). A lista
`PH2D_W6D_LISTA` corre a zoom 3,2, não 1,6 como diz o comentário. Mecanismo não visto completo — ABERTA.

**Item 3 (a) — a r5b: RECUSA.** `git log --all -S` por `pre_lista`/`prelista`/`r5b`/`c3_fina_pre`: a pré-lista nunca esteve
em commit; não reproduzível sem a reconstruir, e o caminho que a usaria está recusado (W6g §12). O fenómeno geral
(`8f22e182c7`, patch `ferramentas/item3_cantos_vezes.patch`, régua `medida_luz2d_cantos_aquecidos`): o passe dos cantos da
r5c corrido N vezes (o 1.º é fantasma igual), `c3_fina` ms: cena 5 sem cantos 1,925 · 1× 2,495 · 2× 3,069 · 3× 3,588 (cada
+0,57/+0,57/+0,52, A×A −0,007); cena 6 0,912 · 1,448 · 1,959 · 2,517 (+0,54/+0,51/+0,56, A×A 0,000) ⇒ nem a cache aquecida
nem o relógio da placa explicam os +0,10 da r5b.

## ⛔ Recusas MEDIDAS (esta jornada)

| Recusa | Porquê (medido) |
|---|---|
| Um cone por fio sem o `if` de uniforme, em qualquer conjunto de níveis | +0,07…+3,05 ms na NVIDIA, +22…+225 na Radeon (§7 item 1) |
| O `de_fora` desdobrado em 2 sondas | 28 texels (1 ULP) na NVIDIA: a ordem da FMA muda o bit |
| O ladrilho do miolo na memória de grupo (`ferramentas/item4_recusado/`) | RTX ×1,53–1,68, Radeon ×1,47–2,40: a cache já reaproveita as leituras; pesa a DRAM |
| Reconstruir a pré-lista da r5b | nunca esteve em commit; o caminho que a usaria está recusado (W6g §12) |
| Correção do f16 da holográfica sempre ligada | +6–13 % do `c2` na RTX, critério 2 % (W6i) |
| A máscara de bits e as duas pipelines da fina | W6h §6 e §11 (two-strikes) |
| Exigir 0 texels entre formas em toda placa | decisão do dono: seria lutar contra o compilador de cada placa (RADV) |

## 8. Smokes do dono (depois de integrado)

Os de sempre, a partir da raiz do repo integrado (`PH2D_LIGHT2D_SMOKE=1..8`). Para o miolo (item 4) a luz deve sair IGUAL
com a cura ligada e desligada — o ganho é só de velocidade, sobretudo na placa integrada.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
   (cena 6 = `PH2D_LIGHT2D_SMOKE=6`; depois do merge troque o `cd` pela raiz do repo).
2. Na janela: a cena «Volume» (lâmpada no meio de um chão com seis corpos). Procure o cartão LUZ no Inspector e mexa na
   lâmpada.
3. Deve acontecer: faces acesas viradas para a lâmpada e escuras atrás; arestas nos cantos; luz suave. Repita com
   `PH2D_LIGHT2D_SALTA_FORA=0` na frente do comando: tem de parecer idêntico.
4. Deu errado se: pontos pretos nas bordas, a luz piscar, a imagem diferente entre as duas corridas, ou o app mais lento com a
   cura ligada.
5. Cena das formas do Motion: `PH2D_MOTION_OBJETO_SMOKE=1 PH2D_LIGHT2D_SMOKE=8` com o mesmo comando.

O binário de smoke fica compilado na worktree (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`;
a 1.ª corrida 1 min 23 s, a 2.ª, zero `Compiling`):

```
    Finished `smoke` profile [optimized] target(s) in 0.69s
```

## 9. ABERTO (substitui W6h §14 e W6i §2)

1. O resto de +3,5 ms da app renascida (item 2): falta o instrumento da soma n 0 da app sem contaminar a vida seguinte
   (W6g §51 c).
2. A família (b): o mecanismo do terminador do disco (§7 item 3 b).
3. F-b do item 1 (inteiros para o uniforme): só com ideia de bit.
4. A fragilidade do «ninguém o lê»: um leitor novo do `a(l)` do miolo (`textureSample` bilinear, visualização da pirâmide)
   precisa de filtrar por `k` ou de reativar a escrita; o portão só olha a FLUÊNCIA.
5. Ficheiros colados ao tecto de 700: `quadro.rs` 698, `solver_miolo.rs` 697, `solver_hrc.rs` 692, `config.rs` 690,
   `solver.rs` 699 — qualquer linha nova reprova; a próxima onda corta antes de crescer. `tests/it/holografica.rs` tem 741
   linhas (o gate só varre `src/`).
6. O fmt do `ph2d-gpu` herdado do `main` (do integrador).
7. O smoke do dono sobre o item 4 na integrada.

## 10. Avisos ao integrador

- Rebasear/conferir o merge-base com `collision-surface.sh` pelo caminho absoluto; `Cargo.lock` tem só as 2 dev-deps.
- Os gates de placa saltam sem GPU («sem GPU — saltado» passa): correr pelo `ph2d-run.sh` nesta máquina; a Radeon só pelo
  `radeon.sh` (§5).
- Provas «mesmo bit» da NVIDIA exigem 0; na RADV ≤ 1 ULP do f16 (lei por placa).
