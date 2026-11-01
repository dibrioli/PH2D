# E10d2 — Absorbency (a tela bebe a água do filme)

Agente D2, 01/11. Controlo da TELA (F3 §1.3 #19, §9): secção comum **Paper**, id novo `PAINTER_WETPAINT_ABSORBENCY`,
só com o meio Wet Paint, nunca no preset. F3b (5): o `paperGate` é limiar de CONTACTO, não absorção.

## 1. Levantamento — onde a água do filme sai para o papel hoje

| Sítio | Lei | É «o papel bebe a água a uma taxa»? |
|---|---|---|
| `Knob::Retention` (`tuning/defs.rs:110`, 0,995 por passe) → `drying.rs` `dry_cell`: `new_film = f·r − evap·(…)` | fuga de **1.ª ordem** do filme (`f·r`, `r = e^{−kΔt}`) | **SIM** — é a transferência filme → papel com taxa `k` (Washburn linearizado) |
| `Knob::Evaporation`/`BaseEvaporation` (`sim.rs` estágio 1, `evap_scale`) | perda de ordem **zero** (µm/passe) — fluxo constante por área | não: evaporação (Dry speed) |
| `EdgeDarkening` (mesma linha, `(1−e)`) | perda extra na orla (C97) | não: orla |
| `Capillary`/`CapillaryGate` (`solver.rs:79-80, 209`) | o filme fino segue o DENTE (descida no relevo) | não: escoamento lateral no papel |
| `GateSaturation` / `PaperGate` (`trail/deposit.rs`) | quanto passa no dente ao pousar | não: contacto (F3b (5)) |
| campo `wet` (`solver.rs:258`, `wet_byte_from_paper`) | byte derivado do papel para o freio | não: não há reservatório de água no papel |
| Dryer (`PAINTER_WETPAINT_DRY_ABSORBENCY`, knob `dryer`) | ferramenta que tira água onde passa | não: ferramenta local |
| GPU `ph2d-wet-gpu/src/seca.wgsl:239` | a mesma linha `f·retention − …` | espelho da CPU (ver §6) |

## 2. A lei (5 linhas)

A absorção existe: é a fuga de 1.ª ordem `f·r` (a água que o papel bebe; a massa que sai leva o pigmento a assentar,
como hoje). A Absorbency `A` multiplica a TAXA: `r_A = r^A = e^{−A·kΔt}` (`Params::retencao`, `sim.rs`).
Estável a todo `Δt` (`0 < r^A ≤ 1`), conservativa (a água que sai assenta o pigmento pela fracção perdida, o mesmo
caminho), e `A = 1` devolve `r` SEM `powf` (ramo explícito) — o motor de antes ao bit, por construção. `A = 0` = papel
que não bebe (só a evaporação). Corre nas duas leis da tinta (`Velha` e `Pigmento` passam ambas por `dry_cell`).

## 3. Medição (sonda `absorcao_sonda`, um processo, a lei do produto, papel por omissão, tela direita, 40 px)

`bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib absorcao -- --include-ignored --nocapture`.
Água = Σ`film`; molhadas = células com `film > 0,01`; orla = largura 10–90 % da borda de cima na coluna do meio (px);
t em ticks de 1/60 s depois de soltar; seca = primeiro tick (de 30 em 30) sem células molhadas.

| pincel | A | água t0 | t30 | t120 | molh. t0 | t30 | t120 | orla t120 | orla seco | seca (ticks) |
|---|---|---|---|---|---|---|---|---|---|---|
| Wet Round | 0 | 4540 | 4433 | 4199 | 9089 | 8698 | 8221 | 29 | 1 | 2130 |
| Wet Round | 0,25 | 4540 | 4416 | 4129 | 9089 | 8694 | 8209 | 29 | 1 | 1890 |
| Wet Round | **1** | 4540 | 4366 | 3923 | 9089 | 8692 | 8172 | 29 | 1 | 1440 |
| Wet Round | 4 | 4540 | 4171 | 3198 | 9089 | 8658 | 8043 | 29 | 19 | 780 |
| Wet Round | 16 | 4540 | 3471 | 1392 | 9089 | 8505 | 7536 | 19 | 19 | 330 |
| Wet Round | 64 | 4540 | 1656 | 0 | 9089 | 7818 | 0 | 20 | 20 | 120 |
| Ink Brush | 0 | 6377 | 6321 | 6136 | 8959 | 9011 | 9060 | 2 | 2 | 3570 |
| Ink Brush | 0,25 | 6377 | 6298 | 6035 | 8959 | 9011 | 9051 | 2 | 1 | 3060 |
| Ink Brush | **1** | 6377 | 6227 | 5741 | 8959 | 9009 | 9028 | 2 | 1 | 2250 |
| Ink Brush | 4 | 6377 | 5950 | 4700 | 8959 | 9005 | 8953 | 2 | 2 | 1230 |
| Ink Brush | 16 | 6377 | 4962 | 2095 | 8959 | 8986 | 8680 | 2 | 2 | 510 |
| Ink Brush | 64 | 6377 | 2395 | 44 | 8959 | 8732 | 1593 | 3 | 3 | 180 |

Leitura: a água no filme cai com `A` como manda `r^A`; o tempo de secar vai de 35 s (A 0) a 2 s (A 64) no Wet Round.
No Wet Round a borda SECA passa de dura (1 px, a orla escura da evaporação) a macia (19–20 px) a partir de `A ≈ 4`:
o papel bebe antes de a orla juntar o pigmento — a dica («high = soft edges») confere. O Ink Brush mantém a borda
dura (2–3 px) em toda a faixa: a forma dele é do depósito, não da secagem.

## 4. Faixa e o que a acaba

`0 ..= 64`, neutro `1` (`ABSORCAO_NEUTRA`, `ABSORCAO_MAX`, `sim.rs`; re-exportados em
`PAINTER_WETPAINT_ABSORBENCY_FAIXA`). O fim é FÍSICO: a 64 o Wet Round fica sem filme aos 2 s (t120 = 0) e
`r^64 = 0,726` por passe — acima disso a poça some em poucos passes, antes de o escoamento a mover; não há aguada a
medir. `0` é o papel que não bebe (só evapora).

## 5. Ficheiros

- Motor: `crates/ph2d-wet-paint/src/sim.rs` (`Sim::absorcao`, `Params::absorcao`, `Params::retencao`, consts),
  `drying.rs` (`DryConst::new` lê `p.retencao()`).
- Ferramenta: `tp/wetpaint.rs` (`WetPaintState::absorcao`), `tp/wetpaint/session.rs` (`WetEngineFacts::absorcao`,
  reconcile → `e.sim.absorcao`), `tp/wetpaint/authored_actions.rs` (facto), `tp/wetpaint_settings.rs` (rota + Reset),
  `tp/wetpaint_settings/absorcao.rs` (setter, rota, teste), `absorcao_sonda.rs` (sonda), `ids/painter_wetpaint.rs`
  (id, faixa, FIELDS 41), `tp/brush_settings.rs` + `tp/snapshot.rs` (`wet_absorbency` para o painel).
- Painel: `paint_watercolor_paper.rs` (`paint_absorbency_row`, `card_row`, só com `brush.wetpaint`, nos dois
  caminhos da secção), `seccoes.rs` (PAPER), `populate_wetpaint.rs` (dica), `brush_fallback.rs`.
- i18n: `painter_layers.rs` (`paper.absorbency` «Absorbency»), `painter_wet_cartao.rs` (`absorbency_tip`).

## 6. Testes e abertos

- `a_absorcao_e_da_tela_e_nao_viaja_no_preset` — verde (novo).
- ph2d-wet-paint por nome (`advect_jacobi o_invariante_soma_massa tinta historia product_rewet seca drying`): 93
  verdes; `fechar_e_o_que_se_via` vermelho — trabalho em voo da D1 (camada fina), não desta lei (o neutro é `r` ao
  bit por construção).
- ⚑ Abertos: (a) o caminho GPU `ph2d-wet-gpu/src/seca.rs:55` lê `p.k(Knob::Retention)` — deve ler `p.retencao()`
  para a placa seguir a Absorbency (fora dos meus ficheiros); (b) a Absorbency não grava no projecto (`snapshot.rs`
  do documento), como o Tilt; (c) `Knob::Retention` continua no registo (o Dry speed absorve-o na F3) — se sair, a
  Absorbency passa a carregar a constante `r` sozinha; (d) os portões do painel ficaram por correr (disco cheio).
