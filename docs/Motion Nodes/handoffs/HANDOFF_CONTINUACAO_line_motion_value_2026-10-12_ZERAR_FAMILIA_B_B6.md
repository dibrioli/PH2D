# CONTINUAÇÃO — `line/motion-value`: ZERAR, família B — B6 FEITO (mistura em grupo pelo DISPOSITIVO) · família B FECHADA (12/10)

> Ordem do Enio: *«vamos até zerar as pendências»*. Regras R1–R9 do
> [`…_SO_O_ESTADO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_SO_O_ESTADO.md). **Este ficheiro substitui o
> [`…_B5.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_ZERAR_FAMILIA_B_B5.md) como fila viva** (o B5 continua
> a ser o registo da mistura por cópia na placa e da LUT do glow).

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, base `8f20c60de`. Commits: `0ad116c805` (B6), `481e8bad08` (motivo do
  1.º quadro, gates `=135` e M8), `1cc2c0ff13`/`298eb0f2c9` (poll só com leitura em voo; portão), o deste handoff.
  **Nada integrado nem enviado.**
- ✅ **Smoke do dono OK (12/10)**: `=135` passos (12)/(13) no dispositivo. ⏳ Re-smoke `=70` (estrelas).

## B6 — a mistura em grupo vai à placa pelo DISPOSITIVO

- **A recusa incondicional saiu** (`RECUSA_MISTURA_EM_GRUPO`). Cerca nova:
  [`motion_bridge_gpu_mistura.rs`](../../../crates/ph2d-app-motion/src/motion_bridge_gpu_mistura.rs) — um sink que
  mistura em grupo vai ao dispositivo quando (1) a placa compõe a mistura dele — `Misturas::da_mistura`, as MESMAS
  regras da CPU (`da_copia` chama-a): `Copies` sempre, `Everything` só na camada isolada, `Scene` fora dela e Multiply
  sem `DUAL_SOURCE_BLENDING` ⇒ CPU com o motivo; e (2) as linhas são TODAS formas. Motivos ditos pelo nome:
  `RECUSA_GRUPO_SEM_MEMORIA` · `RECUSA_IMAGEM_EM_GRUPO` · `RECUSA_MISTURA_PEDE_O_MUNDO` ·
  `RECUSA_MULTIPLY_SEM_SEGUNDA_SAIDA` · `RECUSA_MISTURAS_DESLIGADAS`.
- **«Todas formas» sem leitura de volta.** O plano só sabe a PRESENÇA de colunas e pelo `pre` a saída é
  «incognoscível» ⇒ prova por plano deixaria toda simulação na CPU (recusado). Lei: a **memória da CPU**
  (`Cook::peek`, `linhas_todas_forma → Option<bool>`) decide antes de cozer, e a **vigia**
  ([`vigia.rs`](../../../crates/ph2d-gpu-cook/src/vigia.rs)) confirma: o baixamento das formas soma num
  `atomic<u32>` as linhas que não são forma (saídas vigiadas, uniform `conta`), cópia no MESMO `submit`, lida sem
  espera no quadro seguinte (`poll` só com leitura em voo); `fell` descarta-a. Vigiada sem `geometry_id` ⇒
  `GpuCookError::ImagemEmGrupo` no host. ⚠️ Janela: uma imagem ligada a um grupo de formas desenha 1–2 quadros pelas
  sprites antes de a vigia o devolver à CPU.
- **Corridas por faixa** (`motion_shape_placa_mistura.rs`): `corridas_por_faixa` — uma por faixa de `GpuFormas::faixas`,
  mistura por `mistura_no_quadro` (a regra da bomba); vazias se tudo `Normal`. `decide_do_dispositivo(…, corridas)`;
  `desenha`/`corridas_do_quadro` usam-nas nas duas rotas ⇒ o `Each Shape` mistura no dispositivo. A shell pede
  `misturas_do_quadro`.

## O halo do `fx.glow` com formas misturadas — FECHADO POR MEDIÇÃO

`o_halo_das_formas_misturadas_e_a_silhueta_nas_duas_rotas` (GPU, `Rgba16Float` do halo, tinta `0,4·0,5·0,6 @ 0,7`,
sobreposição de duas cópias): placa `Normal` = `Add` = `Screen` = **`0,3638 · 0,4548 · 0,5459 · 0,9097`** ao bit; o
tile da rota Vello (`vector_instance_as_tile` no passe de sprites) dá o MESMO, desvio **0**. ⇒ as rotas já concordam:
a fonte do halo é a silhueta em `Normal` (o `over` exacto), e o brilho não depende da mistura das cópias. Nada a mudar.

## Gates e números

- `a_mistura_em_grupo_pelo_dispositivo_desenha_o_que_a_cpu_desenha` (GPU): 3 modos × 3 alcances × camada isolada, 16
  estrelas SOBREPOSTAS tintadas; dispositivo × CPU na camada da placa: alfa **0**, cor **≤ 1**, **0** px > 16 nos 12
  casos que vão à placa; controlo contra o `Normal`: cor 127–166, ~12 400 px > 16. `Everything`/`Scene` fora da
  camada isolada: as duas rotas recusam (`PEDE_O_MUNDO`).
- `uma_imagem_no_sink_que_mistura_em_grupo_fica_na_cpu_e_di_lo` (GPU): só imagens, e a imagem ligada DEPOIS (a vigia
  devolve-o à CPU). `a_vigia_das_imagens` (cozimento): conta 1 em 5, consome, controlos a 0, `ImagemEmGrupo`.
- Camada filtrada: tabela do doc 118 e `Each Shape` (`x·x`) pela 3.ª rota `Dispositivo` — **igual** à placa da CPU.
- `the_copies_steps_cook_on_the_device` (GPU): a `=135` 12/13 pelo `coze_o_quadro`: 1.º quadro
  `RECUSA_GRUPO_SEM_MEMORIA`, depois `device: HIBRIDO` com corridas Multiplica/Tela. CPU:
  `as_corridas_por_faixa_separam_os_grupos_de_cada_sink`; textos: os 5 fios da cerca, as corridas na shell.
- **Mutações (10): todas VERMELHAS** — vigia ignorada · memória sempre «formas» · motivo da placa nunca recusa ·
  vigia desligada · `atomicAdd` fora · recusa do host fora · corridas vazias · `grupo: 0` (sobrevivia; ganhou o gate
  de CPU acima) · `fell` sem `descarta` · leitura não consumida.

**Custo** (`o_preco_da_mistura_em_grupo_cpu_contra_placa`, release, `Add·Copies`, estrelas com `wiggle` (rota
híbrida), quadro inteiro com espera da placa, 12 blocos × 3 variantes intercalados, mínimo, load ~3):

| estrelas | CPU (bomba + placa) | dispositivo com mistura | dispositivo `Normal` (controlo) |
|---:|---:|---:|---:|
| 1 024 | 0,54 ms | 0,55 ms | 0,56 ms |
| 4 096 | 0,72 ms | 0,57 ms | 0,58 ms |
| 16 384 | 1,60 ms | 0,70 ms | 0,71 ms |

⇒ a vigia e as corridas custam **~0**; o dispositivo ganha 1,3× a 4k e 2,3× a 16k (uma 1.ª corrida a duas variantes
leu o piso a 0,83 ms; as de três repetiram 0,55–0,57).

## Smoke ao dono (fotografado)

`=135` passos 12/13 (`PH2D_ACABAMENTO_PASSO`): os discos escurecem / clareiam onde se tocam; relevo, sombra e a
simulação no grafo. Terminal: `[formas] 256 copias de 1 geometrias pela PLACA (do dispositivo)` do 2.º quadro em diante.

## Aberto

- Por lei: fora da camada isolada `Everything`/`Scene`/linha sem grupo ficam na CPU (pedem o mundo); uma IMAGEM num
  grupo fica na CPU — ditos no terminal.
- ⛔ **Vermelho ACHADO, anterior ao B6:** `as_cenas_de_varias_saidas_pela_placa_dao_o_que_a_cpu_da` (GPU,
  `#[ignore]`) — `=129` (corpo mole + colisor, sem mistura), 2.ª saída (`soft_body → transform`, rota HÍBRIDA):
  `P` dispositivo × CPU `2,7422905e-3` (amplitude 4,686, rel. 5,85e-4) contra a barra absoluta `1e-3`; desenho Δ 0.
  **Com o código de antes do B6 (`cb52282796`) dá o MESMO número ao bit** ⇒ não é do B6. Próxima janela: medir se a
  deriva é de lei (a barra devia ser relativa, como a `=85`) ou defeito do sufixo do `transform`.

## A fila que resta (C, D, E — inalterada)

**C — Desempenho:** 7. `scan` por subgrupos na cadeia pequena + grupos de ligação/uniforms por cozimento (doc 130
§3.1). 8. `Boundaries` híbrida e `cook_taps_only`. 9. `opt-level` da `ph2d-contact`. 10. (W3b) placa do
`SpriteRenderer` na silhueta; 4 096 formas = 4 páginas; U2. 11. `Bezier`/`Tint` OKLCH na CPU; a fita > 32 MiB.
12. Engasgo de `0,115 ms`/tique na troca de rota. **D — Produto:** 13. linha de COR num gesto; vida e dano pedem
dois objectos · 14. colisor pelo NOME (`RECUSA_COLISOR`) na CPU. **E — Docs:** 15. `33` docs do Motion > 40 KB.
⭐ Nota para C.8: a `=135` e as estrelas a mexer da sonda vão pela rota HÍBRIDA (o `wiggle`/a galáxia no prefixo).

## Fecho desta janela

Portão §1.5.9 (`verificador`, load 6). Vermelhos curados: `fn_loc_caps` (`fase_vector_bands` 208 → 200: a conta
saiu para `misturas_do_quadro`), censo HR-15 (a cerca é a 4.ª irmã isenta — razões atrás de
`PH2D_MOTION_ROUTE_LOG`), e o `poll` da vigia só com leitura em voo. `nextest-impacted` 22 341/22 344 → os 2 do
código curados + `measure_player_budget` (flake de carga: 3/3 sozinho); `check --workspace` warnings=deny, clippy
`-D warnings`, fmt, typos, machete ✓. GPU: `ph2d-shape-gpu` 15/15, `ph2d-fx-cells` 5/5, `ph2d-render` 10/10,
Motion mistura/formas/filtro/halo/`=135`/death 17/17, cozimento 264/265 com `gpu_collide::crossing_the_reach…`
flake de relógio (3/3 sozinho, load ~2); o `=129` acima. Mutações 10/10 VERMELHAS.

binário do smoke (`cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida):
`Finished smoke profile [optimized] target(s) in 0.20s` — zero `Compiling`.
