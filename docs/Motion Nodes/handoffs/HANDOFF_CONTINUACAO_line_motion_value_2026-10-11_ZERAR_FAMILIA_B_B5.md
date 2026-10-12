# CONTINUAÇÃO — `line/motion-value`: ZERAR, família B — B5 FEITO (mistura por cópia na placa) + LUT do glow (11/10)

> Ordem do Enio: *«vamos até zerar as pendências»*. Regras R1–R9 do
> [`…_SO_O_ESTADO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_SO_O_ESTADO.md). **Este ficheiro substitui o
> [`…_ZERAR_FAMILIA_B.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_ZERAR_FAMILIA_B.md) como fila viva.**

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, base `8f20c60de` (= `main`). Commits desta janela: `e1d7eb2aa4` (glow),
  `45399f5551` + `3ee494b5ca` + `3db921b434` + o Each Shape na mistura (B5). **Nada integrado nem enviado.**
- ⏳ Smoke do dono: `=135` passos (12)/(13) (`PH2D_ACABAMENTO_PASSO=12|13`), fotografados; e o re-smoke das estrelas
  (`=70`, cura `f0370a2786`).

## LUT do `fx.glow` — FEITO (`e1d7eb2aa4`)

- Régua ÚNICA do degradê assado: `ph2d_color::ramp_lut` (`bake` + `regua::{corpus, degrau, erro_no_ecra, pior}`),
  dois leitores (o `ramp_sample` do filter; o sampler do glow: `Rgba16Float`, peso de 8 bits).
- Medido no sRGB gravado (pior do corpus): `512 → 0,0514` · `2048 → 0,0117` · `4096 → 0,0038` (< 1/255) ⇒
  `HALO_LUT_TEXELS = 4096` (espelho `ph2d-render` também). Os 512 tinham sido medidos em luz linear.
- ⛔ Defeito achado: `bloom.wgsl` lia `u = t`, meia célula ao lado do bake (`k/(n−1)`); `lut_u` corrige. Gate GPU
  `the_halo_lut_is_read_at_the_baked_positions` (mutação ao endereço antigo: −2,6 % a lum 0,005, VERMELHO).
- A LUT assa-se uma vez por texto (memória por thread): `the_halo_lut_bakes_once_per_text`.

## B5 — FEITO (mistura por cópia nas formas da placa)

- **`ph2d-shape-gpu`** (`mistura.rs`, `multiplica.wgsl`, `corte.wgsl`): `Mistura {Normal, Soma, Tela, Multiplica}`,
  `Corrida {copias, mistura, grupo}`, `ShapePass::draw_em_corridas`. Soma `One/One`; Tela `One/OneMinusSrc`;
  Multiplica `OneMinusDstAlpha/Src1` pela 2.ª saída (`DUAL_SOURCE_BLENDING`, pedida em `ph2d-gpu/context.rs` onde o
  adaptador a tem; `multiplica_no_dispositivo` = porta única). Grupo `Copies` com conteúdo antes ⇒ RASCUNHO limpo +
  colagem. Cortes a 1 (passe `Min` de ecrã inteiro) onde o Vello os faz: cor+alfa antes de uma corrida com mistura,
  antes de colar um grupo e no fim; alfa do rascunho antes de o colar. Uma corrida só `Normal` = o `draw` de sempre.
  Com tracejado carregado as corridas usam a variante COMPLETA (a escolha da placa é de UM desenho indirecto).
- **`ph2d-app-motion`**: `Misturas::da_copia` (`motion_shape_placa_mistura.rs`) — `Copies` sempre; `Everything` e
  a linha sem grupo só na camada ISOLADA; `Scene` fora dela e Multiply sem a feature ⇒ Vello, com o motivo no
  terminal (`[formas] … pela cena Vello: <motivo>`). A shell passa `Misturas::do_quadro(gpu, camada_isolada)`
  (`decide_com_misturas`).
- ⭐ **O `Each Shape` ENTRA na mistura** (report do dono 11/10: *«o brilho do relevo não participa do
  Multiply»*): o run-walker é o `ph2d_shape_gpu::Compositor` (`compositor.rs`: cortes, grupo no rascunho, uma
  `Sessao` em várias chamadas); a volta das células (`ph2d-fx-cells/src/volta.rs`) devolve as formas
  JÁ filtradas a uma camada própria, corrida a corrida com a mistura da cópia (`Formas::corridas` ←
  `PlacaDeFormas::corridas_do_quadro`), colada sobre as sprites no fim. Gate
  `each_shapes_effect_enters_the_blend` (Color Overlay `x` + Multiply: sobreposição `x·x` = 161 contra 162;
  mutação «corridas vazias» lê 203, VERMELHO). Só a rota Vello (placa recusada) fica «one piece».
- ⛔ **Defeito antigo corrigido:** a camada filtrada decidia o Vello dela por `PlacaDeFormas::ativa`, que o
  `desenha` já consumiu ⇒ o Vello corria por cima da placa com a cena VELHA (a shell não a codifica com a placa
  ativa). Agora `desenhou`. Gate `a_placa_que_desenhou_cala_a_cena_velha_da_camada`.
- **Gates:** `cada_mistura_por_copia_compoe_como_o_vello` (24 casos × modo, cópias SOBREPOSTAS, desvio ≤ 0,00096
  contra a composição do Vello escrita à mão; controlo Normal); a tabela do doc 118 pela placa NO PRODUTO
  (`the_output_blend_crosses_the_filtered_layer…`: 3 modos × 3 alcances × 2 colagens, ≤ 1 nível do Vello);
  `an_each_shape_step_runs…` (Vello «one piece», placa sem queixa); `cada_mistura_vai_a_placa_so_onde…` (tabela da rota,
  CPU); `the_copies_steps_go_to_the_plate` (=135). Mutações: `ativa`↔`desenhou`, Multiply→Tela, os 5 cortes/isolamento
  — todas VERMELHAS.
- **Custo** (`o_preco_do_vello_na_camada`, release, 12 blocos intercalados, mínimo, load 56–92, `Everything` na
  camada isolada, ms):

  | janela · discos | sem formas | Vello na camada | placa (Soma · Tela · Multiplica) |
  |---|---:|---:|---:|
  | 1600×972 · 256 | 0,10–0,12 | 0,49–0,50 | 0,24 · 0,22 · 0,22 |
  | 1600×972 · 4096 | 0,10–0,12 | 1,01–1,37 | 0,35 · 0,33 · 0,35 |
  | 4K · 256 | 0,29–0,36 | 1,14–1,41 | 0,58 · 0,47 · 0,53 |
  | 4K · 4096 | 0,33–0,45 | 1,73–2,23 | 0,74 · 0,72 · 0,60 |

## Aberto desta família (por lei, com o porquê)

- **Fora da camada isolada**, `Everything`, `Scene` e a linha sem grupo misturam com o MUNDO por baixo; a camada
  da placa começa vazia ⇒ ficam no Vello (`MotivoDoVello::PedeOMundo`). Levá-las pediria a placa a pintar sobre o acumulador
  do mundo (o Vello grava a cor separada do alfa — doc 121 §2.1).
- **Rota do DISPOSITIVO:** `RECUSA_MISTURA_EM_GRUPO` continua (cozimento na CPU para sinks que agrupam). Levantá-la
  pede saber, sem leitura de volta, que as linhas do sink são todas FORMAS (uma imagem iria ao passe de sprites,
  em luz linear e sem alcance); o host já sabe as `faixas` por sink ⇒ as corridas por faixa são o caminho.
- O halo do `fx.glow` (`redesenha`) repete as cópias em `Normal` (silhueta), como antes.

## A fila que resta (C, D, E — inalterada)

**C — Desempenho:** 7. `scan` por subgrupos na cadeia pequena + grupos de ligação/uniforms por cozimento (doc 130
§3.1). 8. `Boundaries` híbrida e `cook_taps_only`. 9. `opt-level` da `ph2d-contact`. 10. (W3b) placa do
`SpriteRenderer` na silhueta; 4 096 formas = 4 páginas; U2. 11. `Bezier`/`Tint` OKLCH na CPU; a fita > 32 MiB.
12. Engasgo de `0,115 ms`/tique na troca de rota. **D — Produto:** 13. linha de COR num gesto; vida e dano pedem
dois objectos · 14. colisor pelo NOME (`RECUSA_COLISOR`) na CPU. **E — Docs:** 15. `33` docs do Motion > 40 KB.

## Fecho desta janela

Portão §1.5.9 (`verificador`), duas rodadas. 1.ª (até `3db921b434`): 4 vermelhos determinísticos curados — censo
HR-15 (motivos ⇒ `MotivoDoVello` dito no `eprintln!`), tecto de LOC (`motion_fx_lut_tests.rs`), clippy do glow, fmt.
2.ª (com o Each Shape na mistura, `733217c2c7`): `nextest-impacted` **22 343/22 343**; `check --workspace
--all-targets` `-D warnings`, fmt, typos, machete verdes; clippy `-D warnings` verde depois do `expect` de
`too_many_arguments` no `Celulas::pagina`. GPU (`PH2D_GPU=1`): `ph2d-shape-gpu` it 15/15, `ph2d-fx-cells` 1/1 + 4/4,
`ph2d-app-motion` placa/camada 14/14, `ph2d-render` 10/10. Mutações da janela (9) todas VERMELHAS.

binário do smoke (`cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida):
`Finished smoke profile [optimized] target(s) in 0.25s` — zero `Compiling`.
