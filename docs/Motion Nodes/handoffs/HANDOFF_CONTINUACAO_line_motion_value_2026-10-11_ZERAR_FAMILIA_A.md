# CONTINUAÇÃO — `line/motion-value`: ZERAR as pendências, família A fechada (11/10)

> Ordem do Enio (11/10): *«vamos até zerar as pendências»*. Regras R1–R9 do topo do
> [`…_SO_O_ESTADO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_SO_O_ESTADO.md). Fila e
> caminhos de saída: [`…_ZERAR_PENDENCIAS.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_ZERAR_PENDENCIAS.md)
> (este ficheiro a substitui como fila viva).

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, base `8f20c60de` (= `main`),
  HEAD = o commit deste ficheiro. **Nada integrado nem enviado.**
- ✅ **Smoke do dono (11/10):** *«smoke de glow está OK»*, mas *«as estrelas estão paradas»* — o
  `force.vortex` nascia no centro do MUNDO e as estrelas estão em `(−4,6; −1,6)`, fora do raio.
  Cura: vórtice + `force.attractor` no MESMO centro + `force.drag` (a órbita estável do doc do
  vórtice), constantes `CENTRO_DAS_ESTRELAS`/`VORTICE`/`ATRATOR`/`ARRASTO`. Gate
  `the_stars_orbit_and_stay_on_screen` (10 s simulados: `1,61` no 1.º s, `2,06` no último, a mais
  longe a `1,70` do centro); mutação com o centro em `(0,0)`: sangra (`6,58` do centro). Foto
  refeita: as estrelas espalhadas em volta do centro. ⏳ Re-smoke das estrelas a girar.

## Família A — FECHADA

1. ~~BUGS #19, o ângulo do `fx.glow` gateado ao contrário~~ — **feito.** Quarta espécie de gate
   `ParamGateAway { param, when, from }` (`ph2d-node-registry/src/ui.rs`, lei num método só:
   `shows(v) = v != from`, a desigualdade exacta do braço literal de `ph2d-bloom::upsample_basis`).
   O `fx.glow` regista-a; os quatro leitores de espécies a conhecem (`Visibility::shows` — nunca
   `mode_hides`, é contínua —, `motion_param_reach::quem_gateia` (extraída), a sonda dos escondidos,
   a nota «sai de» do tutorial). Gates: painel = NÚCLEO em 7 valores de `stretch` (oráculo = a base
   da tenda com dois ângulos) · censo de toda espécie de ponto · todo param escondido de fábrica tem
   dono conhecido pela sonda de alcance (a sonda dava «alcançável» sem perguntar ao `shows` quando
   não conhecia a espécie) · a tabela do tutorial do glow diz «sai de 1». Mutação: 2 rodadas,
   **12/12 sangraram** depois da 2.ª (M5 e M7 sobreviveram à 1.ª e ganharam gate).
   A cena `=70` já dizia ao dono *«ele só aparece quando o Anamorphic sai de 1»* — ensinava o
   contrário do que se via. A banda 3 passou a ESTRELAS (`source.shape` → `motion.duplicator`) a
   rodar num `force.vortex` com `motion.integrate` (a lei do smoke: formas + simulação); o teste da
   banda coze como o app (`motion_shape_gen::publish`) e exige as quatro estrelas.
2. ~~BUGS #18, gate de gesto pelo caminho REAL do rato~~ — **feito.** Era uma regra sem guarda.
   O arnês do rato real saiu do teste do tremor para `ph2d-panel-motion-graph/src/mao_real.rs`
   (`Mao::nova(cena, zoom)`, `gesto`, `fileira(no, i)`), e a catraca EXACTA por ficheiro
   `synthetic_gestures_in_the_graph_tests_only_shrink` conta os `GesturePhase::` sintéticos dos
   `*_tests.rs` (113 em 11 ficheiros): ficheiro novo ou que cresce reprova; que desce reprova até a
   tabela descer. Mutação: as duas direcções sangram; sem o `Move` no `gesto`, 2 dos 3 do tremor.

## A fila que resta

**B — A camada e os filtros** (factos já levantados, 11/10 — não refazer):
3. Retângulo da camada `scene_viewport` × `cena_visivel` (`motion_filter.rs:497`, `motion_lens.rs:160`,
   `na_escala.rs:50`): só difere com painéis por cima da cena; o passe corre só no retângulo.
   ⇒ **medir** (sonda, junto da família C) e provavelmente fechar por medição.
4. `fx.filter` sem os modos de mistura e com rampa de 2 stops (`fx-filter/src/lib.rs:16,466,474`):
   a `FxOp` já tem `blend` (lei CPU `ph2d-blend-mode::apply`, WGSL `fx_stack_shader.rs:192`) e 8
   stops (`op.rs:264`); o painel de VETOR já os autora. ⇒ ✅ **DONO DECIDIU (11/10):** *«sim se a
   performance for boa, pois aqui podemos ter milhares de objetos»*. ⇒ fazer, com a tabela de
   custo ao lado: cada modo e a rampa de 2 a 8 stops sobre a camada com milhares de formas
   (`4 096` e `32 761`), placa e CPU de reserva, contra o Normal de hoje; se algum modo custar
   caro, a tabela decide antes de ele entrar. O oráculo da lei: ImageMagick (`-compose`,
   permissivo) sobre entradas nossas (§0.9).
5. `Multiplica` na camada / `DUAL_SOURCE_BLENDING`: existe no wgpu 29.0.4, `ph2d-gpu/src/context.rs:69`
   não o pede. Custo medido do Vello na camada: `0,49–2,25 ms` (doc 129 parte_1 l.238; parte dele já
   existia). ⇒ técnico (delegado): medir se o adaptador da estação o tem; se sim, rota por placa com
   o Vello de reserva por adaptador.
6. ~~Filtros de todos os fios na camada~~ — **fechado por medição** (doc 129, item 4: nenhuma cena
   tem duas saídas com filtro; uma camada por saída partiria a ordem por linha entre saídas; o
   cartão avisa `outputs_land_differently`). ⚠️ A redação «uma camada por FIO» dos handoffs
   anteriores estava errada: a recusa é sobre SAÍDAS.

**C — Desempenho** (máquina partilhada a `load 26–272` hoje: nenhum relógio valeu; medir com
`MEDIR_VELOCIDADE.md`, variantes no mesmo processo, intercaladas, o mínimo):
7. `scan` por subgrupos na cadeia pequena (`1,7–6×` a CPU) + o passe recria grupos de ligação e
   uniforms por cozimento ([doc 130 §3.1](../130_o_passe_na_placa.md)).
8. `Boundaries` (rota híbrida) e `cook_taps_only` que coze todas as tomadas.
9. `opt-level` da `ph2d-contact`.
10. (W3b) rota de placa do `SpriteRenderer` na silhueta; `4 096` formas = 4 páginas; U2.
11. `Bezier`/`Tint` OKLCH caros na CPU de reserva; a fita acima de `32 MiB`.
12. Engasgo de `0,115 ms`/tique na troca de rota a meio — medir se se vê.

**D — Produto:** 13. uma linha de COR inteira num gesto; vida e dano pedem dois objectos ·
14. colisor escrito PELO NOME (`RECUSA_COLISOR`) ainda na CPU.

**E — Documentação:** 15. `33` docs do Motion acima de 40 KB (o maior `115_…`, 170 KB) — lote
mecânico para o `documentador` com `python3 scripts/doc-split.py --keep … --archive …`.

## Fecho desta janela

Portão §1.5.9 pelo `verificador`: `nextest-impacted` `19 363/19 364` — a falha era o `no_magic_numeric` no `VIEWPORT` do arnês (movido de um `*_tests.rs`), curada com `LITERAL-PX-OK` e reconferida; clippy `-D warnings`, fmt e machete limpos. ⚠️ Tectos: `ph2d-node-registry/src/lib.rs`
cresceu 12 linhas (campo + duas portas da espécie nova).
