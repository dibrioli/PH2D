# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 41: a Shape Color pinta no Wet Paint (2026-10-16)

## §1 — Coordenadas

Sobre o `main` `0aab2c1cb6` (merge-base, sem rebase: 0 atrás). Sem push. [Prompt 41](../NewFeaturesWetPaint/41_PROMPT_shape_color_no_wet_paint.md)
· lista [26](../NewFeaturesWetPaint/26_ABERTOS.md) A7 (feito) · plano [27](../NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md) W6 ✅ ·
[prancha 41](../NewFeaturesWetPaint/41_prancha_shape_color_no_wet_paint.png). Anterior: [P40 §9](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P40_CAPSULA_COM_ESTADO.md).
Commits: `d6236b6679` (o portão de identidade, ANTES do motor) · `5cdf200d46` (motor, pincel, painel, portões,
prancha) · o deste fecho (docs).

## §2 — Medido primeiro (um lote, `wetpaint/sonda_shape_color.rs`)

- **Onde a cor pousa:** o rasto de cada faixa tem a JANELA (pigmento/água por célula, ≤ 4 dabs) e a PONTA
  (`tip_r/g/b`: a cor do pincel + o que apanhou, por `TipClean`/`Pickup`, interpolações LINEARES em sRGB).
- **A recusa do doc 46 §1** («a mistura K–M homogeneíza o carimbo no primeiro passo») era análise: com a cor por
  célula, o desvio de G−R da Texture Color é ×3,4 (Express Oil) a ×16,5 (Oil Bristle) o do traço sem rampa
  **10 s depois**, nos 12 pincéis de fábrica. Só o estado cru de teste (mais molhado que todo pincel de fábrica)
  a mistura toda (×1,0 a 10 s).
- **Custo:** ×1,06–1,11 o traço (raio 48, mínimo de 9 intercalados; dev com `opt-level 2` no pincel e no motor).
- **Identidade:** 7 cenas pinadas no `f6d501defb` (lisa, padrão, cápsula, cerdas, rampa configurada e
  desligada, tom B&W, brocha suja). A do tom saiu da lista: era ao byte a do Voronoi cru — **controlo mudo**
  (a rota da água nunca assava a LUT do tom); agora age.

## §3 — A lei

- **Motor** (`ph2d-wet-paint`): `ColorFn` `(x, y, cobertura) → sRGB 0..255 + alfa`, com cobertura =
  silhueta × padrão (o índice da Falloff Color do Digital e da prévia). Portas `dispatch_pressure_dab_lane_cor`
  e `accumulate_paint_shaped_cor`; as antigas delegam com `None` (os 15 chamadores intactos).
  O dab pousa a cor LIMPA num plano da janela (`tinta_*`, alocado no 1.º dab com cor): a 1.ª tinta da célula
  inteira, as seguintes misturadas pela massa (`ColorMix::mix`). O alfa multiplica o depósito.
  O transfer pousa `limpo · tinta + (ponta − limpo · base)`: `limpo` é o peso da cor limpa em cada texel da
  ponta e segue as MESMAS interpolações (limpeza devolve `clean`, recolha guarda `k`), por isso é **exacto**:
  uma cor constante igual à de um pincel sem ela pousa a mesma tinta suja. Caminhante novo `par::walk_rows4`.
- ⛔ **Recusado no caminho (medido):** (a) escrever a rampa na PONTA a cada janela — apagava a sujidade
  (a brocha deixava de levar a cor apanhada: 0,0 contra +2,5); (b) somar a sujidade em K/S — a recolha é
  linear em sRGB, não K–M, logo não é exacto; (c) indexar a Falloff pela silhueta só — ρ −0,58 contra o Digital.
- **Pincel** (`wetpaint/dab_route.rs`): a cor vem da porta da aguada (`rampa_da_aguada` → `RampaDaAguada::cor`):
  Texture Color pelo valor do Shape (`shape_value_at`, cerdas incluídas), Falloff Color pela cobertura. O padrão
  que vira cor deixa de recortar (`padrao_do_shape_vira_cor`, a lei do P38/P39). A borracha não lê cor.
- **Painel:** `BrushSettings::shape_ramp_offered` saiu (a secção e a prévia em todo meio).
- **Duas curas que a suíte do fecho trouxe:** (a) a MACRO leva a Shape Color no estado inicial
  (`macro_estado.rs`, bloco `shape_color`, aplicado pelas portas do painel — `o_estado_inicial_leva_cada_controlo_do_wet_paint`
  acusou o `Use Color Ramp`); (b) o B&W automático ao escolher um Grain (`on_grain_assigned`) só no Digital e no
  Impasto, onde a rampa do Grain existe — na Aquarela e no Wet Paint ele só desligava a Shape Color (o censo
  acusou o Grain esmaecido da Aquarela a mudar a tinta; gate `escolher_um_grain_no_wet_paint_nao_apaga_a_shape_color`).

## §4 — Portões (vermelhos antes: o de meio e os de cor; a identidade nasceu verde no código antigo)

`ph2d-wet-paint` `tests/it/cor_por_celula.rs` (5): mistura pela massa · sujidade exacta (2 janelas, `Pickup`
0,5, `TipClean` 0,1) · o alfa pesa · as duas rotas por linhas ao bit (com limpeza e recolha) · um traço sem cor depois de um com cor pousa a ponta.
`ph2d-tool-painter` `wetpaint/shape_color_tests.rs` (9): identidade (6 cenas) · o tom age · Texture Color ×2,5 e
ρ > 0,45 (traço, medido 0,66) e > 0,6 (carimbo, 0,86) contra o Digital · Falloff ±8 de G−R e ρ > 0,3 (0,51) ·
os 12 pincéis de fábrica ×2,5 a 10 s · o padrão que vira cor pousa a tinta da ponta lisa (1,000; controlo
0,821) · cápsula ρ > 0,85 (0,97) · a brocha leva a cor apanhada com a rampa (> 0,85; medido +1,7) · escolher um Grain não liga o B&W na água.
`ph2d-panel-painter-layers` `a_rampa_da_shape_aparece_em_todo_meio` (era `…so_onde_o_meio_a_oferece`).
**Censo** (`sonda_as_pre_condicoes`, 2 estados novos × 4 meios): no Wet Paint cada controlo da secção age ou
fica inerte como no Digital ⇒ nada a esmaecer.

## §5 — Mutação

**14/14 sangram** (agente de mutação + 2 à mão; controlo verde antes, âncoras 1×): `cor_arg = None` · índice
`fall` sem o padrão · sem o alfa · tinta sempre sobrescrita · sem a sujidade · `limpo` sem a recolha · `limpo` sem
a limpeza · o padrão-cor a recortar · `rampa_da_aguada` filtrada a nada · o painel sem a secção no Wet Paint ·
`tinta = false` · não apagar `tinta_na_janela` (sobrevivia: ganhou `o_traco_sem_cor_depois_de_um_com_cor_pousa_a_ponta`)
· sem o `ensure_shape_ramp_lut` (o tom) · o B&W automático em todo meio.

## §6 — Aberto

- O carimbo de um CLIQUE só não pousa tinta no Wet Paint (a janela do rasto aterra no 2.º dab e o soltar larga
  o resto) — anterior a esta linha, não medido contra o Rebelle.
- Com Spacing 1,5 sem o Continuous o traço sai quase em branco (`tinta` 31 860 contra 911 298) — anterior,
  igual sem a rampa.
- A Shape Color NÃO entra no preset do Wet Paint (nem na de fábrica): guardá-la é degrau de formato
  (`ph2d-brush-presets`, doc 37 §4) — pergunta ao dono se um pincel salvo deve levar as rampas.
- Luz 2D: os dois `g4_…_nao_depende_do_numero_de_luzes` vermelhos também isolados (fora deste diff).

## §6b — Smoke do dono (16/10): OK, e dois defeitos

1. **O traço cortado por uma recta** (anterior ao P41; a rampa só o tornou visível) — `83c347df9b`. A janela do rasto
   media-se pelo `spacing` do motor (2 células), mas na porta do pincel os dabs estão a uma CORDA (o espaçamento do
   pincel): o 2.º/3.º dab da janela passava de `âncora ± half` e perdia o pigmento (`trail/deposit.rs`: fora da janela
   só a umidade). Cura: `fit_to` com `4 · max(spacing, corda)` + rede de segurança — o dab que não cabe faz a janela
   CRESCER no lugar (âncora fixa, todos os planos recentrados, `trail/janela.rs`); o Blend passa a medir a janela.
   Crescer a janela já não repõe a ponta na cor base. Portões `the_window_holds_the_whole_dab.rs` (4) e
   `janela_do_rasto_tests.rs` (2: aresta 0,79 → 0,21; o pincel do dono 33 655 px → 0). Identidade ao byte. Mutações 4/4.
2. **A prévia do pincel sem a Shape Color** — `ad43c2f0d7`/`c171649604`. A cena da prévia (`wetpaint_preset/preview.rs`)
   e a chave não levavam a rampa: `EstadoDaShapeColor` (`shape_ramp.rs`) + `shape_ramp_version` na chave. A miniatura
   do preset fica SEM a rampa (B13). Mutações 3/3.

## §7 — Fecho

`cargo fmt --check` ✓ · clippy `-D warnings --all-targets` (`PH2D_SUITE=fecho`; wet-paint, tool-painter,
panel-painter-layers, pigment) ✓ · `nextest-impacted` (base `0aab2c1cb6`) **18 479/18 484** em 509 s (`load` até
37): os 2 do P41 curados acima e refeitos por nome (censo esmaecido, macro, `seam_accumulate_por_meio`,
`grain_and_stencil` 24, `shape_color_tests` 9, `cor_por_celula` 5 ✓); `o_custo_do_miolo` passou sozinho (carga);
os dois K3 da Luz 2D no §6. Smoke `--profile smoke` 2×: «Finished … in 0.21s», zero «Compiling».
