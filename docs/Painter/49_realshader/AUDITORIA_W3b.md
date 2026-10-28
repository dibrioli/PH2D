# Auditoria W3b (27/10) — 2 lentes sobre `151cf87c10^..65d6a6a16d` + o 2.º smoke do dono

## Smoke do dono (27/10, noite)

- D1. «Smooth Edges só atua na cor e não no relevo. Smooth Edges deve ser para todo o papel, de todos os traços já
  pintados ou futuros.» ⇒ revoga a decisão «Smooth Edges por traço» (`65d6a6a16d`): é da TELA e age no relevo.
- D2. «Transparência ainda não é o suficiente» ⇒ Transmission a 1 = **vidro colorido**: o papel bem claro com o grão,
  só tingido pela cor (escolha do dono entre: vidro colorido · igual ao Blender · controlo de espessura).
- D3. «O painel só vale para os próximos traços não funciona para Transmission e Transparency» — ele mexeu no CARTÃO
  Real Oil. ⇒ Transmission, Transparency e Paper Relief saem do cartão: só no pincel (por célula).
- D4. Rótulo «Transparency (Alpha)» → «Transparency».

## Lente 1 — correção

| # | achado | mecanismo | gate que faltava | por que estava verde |
|---|---|---|---|---|
| C1 | `sombra_sem_furo` estoura em `u8` | `oleo_real.rs:~1535` `s - v[4]` com `s < mediana`: pânico em dev; em smoke/release dá a volta e a célula MAIS CLARA que a mediana recebe a mediana | unit de `sombra_sem_furo` com célula clara e `s ≥ 32` → `s`; os gates de lei `u8` correm também em dev | corridos só em `--profile smoke` (sem overflow-checks); `mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` e `a_orla_rasgada_nao_tem_pontinhos_escuros` VERMELHOS em dev |
| C2 | cartão Real Oil muda a tinta pousada (= D3) | `max(célula, cartão)` em `material.rs:109,144`; leitores `render.rs:464,542`, `oleo_real.rs:1449,1489,1495,473`, `oleo_cache.rs`; o cartão recompõe | o gate do painel percorrer também o cartão | o censo só percorria `KNOB_DEFS` e `FACTOS` |
| C3 | o gate do painel não prova que a célula guarda o material | `tela()` pinta com material NEUTRO | pintar com Transmission/Transparency ≠ 0, pincel a 0, recompor → 0 bytes | vacuidade |
| C4 | o material não veste a orla virgem (a aparência veste) | `transfer.rs:~399` `aparencia::veste_virgens` sem par em `material::pousa` (`:176`) | censo `sett+susp > 0 ∧ material == NEUTRO` | inferido, não corrido |
| C5 | sombra sob a tinta × no papel nu discordam na Transmission | `render.rs:~561` só a Transparency; `composite.rs:~362` soma `w·L(a²)` | costura na orla de tinta só com Transmission | — |
| C6 | números de docs | 12 B × 16 B (+201 MB → ~268); «16 mutações» × 13; `grao_do_papel = 1−(1−r)(1−w)` × código `w` | — | — |

## Lente 2 — costura de UI

| # | achado | mecanismo | gate que faltava | por que estava verde |
|---|---|---|---|---|
| U1 | Glaze layering (K-M) (e Smooth Edges) são do traço mas não viajam no preset nem por ferramenta | `wet_preset_capturar` (`wetpaint_preset.rs:~610`) / `aplicar_corpo` (`:~426`) / `tool_link.rs:20-47` | preset → outro → reaplicar → volta | o gate só mede tinta pousada e traço seguinte |
| U2 | Paper Relief não faz nada no estado de fábrica | `composite.rs:76-82` só com `paper.is_active()` e `substrate_depth > 0` (fábrica 0); `GraoNoFilme::de` → `None`; três «Relief» no painel | o controlo morde no estado em que o artista o encontra (ou diz porquê) | fixtures com Cold Relief 0,6 armado |
| U3 | Glaze: nota i18n mente («apply to all the paint») e o clique recompõe a tela inteira para 0 bytes | `ph2d-i18n/src/lib.rs:~336`; `wetpaint_settings.rs:~904` | censo «controlo do pincel ⇒ o roteador não recompõe» | — |
| U4 | Material do pincel sem retorno antes de pintar | `CenaDoPreview`/`ChaveDoPreview` (`preview.rs`) não levam o Real Oil | todo knob visível do pincel muda a miniatura ou a tela | — |
| U5 | ordem: Transparency · Lock material · Transmission · Paper Relief | `rows()` segue a ordem do enum | gate de ordem do grupo Material | `every_visible_knob_has_a_row` só conta |
| U6 | comentários velhos | `tuning.rs:10` (Look), `ids/wet_tuning.rs:50-53` («quatro grupos») | — | — |

Fora do diff (anterior): o Painter não tem Auto Key em campo nenhum (decisão do dono 19/10 «grava tudo»); mexer num
knob não entra no desfazer.

## Cura (agente U1/U3/U6/C6, 27/10)

- **U1 (Glaze)** — curado: chave `glazeLayering` no `engine` do preset (gravada só ligada; ausente = desligado = fábrica,
  ficheiros e miniaturas da fábrica ao byte) + `WetPaintState::glaze_por_slot` em `tool_link`. Gates
  `wetpaint_preset/glaze_no_preset_tests.rs` (3); mutações M2–M6 vermelhas. O Smooth Edges não (passou a ser da TELA, agente A).
- **U3** — curado: nota `panel.wet_tuning.note` reescrita (valem para os traços seguintes; o Glaze vai no preset); o
  clique no Glaze não recompõe. Censo `wetpaint_settings/pincel_nao_recompoe_tests.rs` (knobs da tabela fora do
  `Rebuild::Paper` com porta no Tuning/cartão/ferramenta da água + 14 factos; tela envenenada + controlo positivo);
  mutação M1 (devolver o `wet_recomposite_full`) vermelha. Fora: Reset do Wet Paint, knobs do cartão Real Oil (B).
- **U6** — curado: `ph2d-wet-paint/src/tuning.rs` (Look saiu) e `ids/wet_tuning.rs` (3 grupos avançados 6..=8, Material 9).
- **C6** — curado: `W3b_painel_so_no_futuro.md` §Custo 12 B (lido em `grid/aparencia.rs`); mutações = 13 (tabela;
  commit dizia 16) no doc e no ESTADO; `grao_do_papel` = `w` no código, escrito no ESTADO e no README §Transmission
  com nota «em revisão, ver Ronda 3» (o agente B muda a lei).
