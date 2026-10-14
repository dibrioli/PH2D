# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 34: o Continuous que IGUALA o Spacing pequeno (2026-10-13)

> As regras R1–R9 continuam as do topo do
> [handoff W0/W1](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-13_W0_W1_SHAPE_CASA.md). Relato do dono (3.ª
> rodada): *«fica diferente do traço com spacing menor … RETÂNGULOS … MENOS NÍTIDO»* + *«continuous deve estar
> disponível para os outros modos de pintura também»*. Lei, medidas e recusas: [31 §5](../NewFeaturesWetPaint/31_spacing_por_meio_o_pontilhado.md).

## §1 — Coordenadas

Sobre `785dc38945` (merge-base `757afebd58`; ⚠️ o `main` andou para `0db0bdcf33` durante a janela — rebase é do
integrador). Um commit desta janela. Sem push. Próxima janela: o
[prompt 30](../NewFeaturesWetPaint/30_PROMPT_janela_shape_casa_W3_W4.md) (W3/W4 do plano 27).

## §2 — O que mudou

- **`ph2d-painter-brush/src/capsula.rs`** (novo): `Capsula` (passo `Dab::step`, `n = ⌈máx|D| / sub⌉ ≤ 128`,
  `off_k = −D·k/n` — PARA TRÁS, até ao centro anterior), `sub_passo_px(raio) = máx(SUB_SPACING·2r, 1)` com
  `SUB_SPACING = 0,025` (medido: 1 px e 0,0125 ficam mais densos que o alvo e custam mais; 0,05 ralo), `Junta`
  (as leis: `Uniao` = produto; `Media`, `Centro`, `CentroMax` = chaves de medição/controlo), `com_lei` (escopo,
  repõe), `sub_dabs` (a cápsula como dabs lisos), `Dab::capsula()`.
- **Carimbo** (Digital, cor do Impasto, Composite): `dab::silhouette_capsula` — a ponta INTEIRA em cada sub-dab,
  pelo corte do filme, composta `(1 − Π(1 − c·x_k)) / c` (`c` = opacidade do dab; `1` sob o cap de Accumulate).
  `stamp_dab_textured_masked` / `stamp_dab_ramped` ganharam `capsula: Option<Capsula>`; a caixa cresce para trás;
  o AA do filme desliga-se na cápsula. Rotas: os caches (`stamp_route.rs`), a banda e o dispositivo
  (`stamp_cache.rs::banded_ok`) cedem à rota por píxel quando `lays_capsules`; `region::dabs_bounds` inclui o
  centro anterior.
- **Watercolor** (`watercolor_accum.rs`, `_cor.rs`): os sub-dabs percorridos dentro do dab, do mais antigo.
- **Wet Paint** (`wetpaint/dab_route.rs`): os sub-dabs ENTREGUES ao motor (sub-passo ≥ 1 célula), porque o
  depósito subtrai o portão de papel por dab — compor no hospedeiro dilui ou alarga (medido, recusado).
- **Saiu** a varredura do prompt 32: `TexDabBasis::sweep`, `Varre`, `swept`, `ShapeFrame::Stroke.step` (13 rotas),
  `bristles_sweep_tests.rs`, o aquecimento e o passo nominal do 1.º dab (`warmup.rs`), `drags_bristles`.
- **Painel**: `BrushSpec::lays_capsules` (opção + método com caminho + passo > sub-passo);
  `Dependente::ContinuoSemCerdas` → `ContinuoSemPasso` (esmaece também na água com passo ≤ 1 célula), i18n
  `inerte.bristles_in_shape` → `inerte.method_lays_a_path`.

## §3 — Medido (sonda `tests/capsula.rs::sonda_capsula`, régua normalizada pela tinta contra o traço a 0,025)

Raio 20 / 45, redonda e cerdas, cinco meios: a 0,25 com a caixa **≤ 0,017** (hoje: Digital cerdas 0,21, redonda
0,118; Impasto cerdas 0,20 e o código do prompt 32 ESVAZIAVA o traço; Wet Paint 0,08–0,15 com blocos), tinta
0,99–1,03, costura = a do alvo, abertura ≤ 0,016. Custo (`measure_custo_da_capsula`, raio 45, intercalado, mínimo;
`load` 13–120): 0,18–0,57× o traço a 0,025 nos carimbos e na Watercolor; Wet Paint 0,53–0,90 (pressão 1,43 sob
`load` 120 — reconfirmar calmo). Prancha: [`34_prancha_continuous_capsula.png`](../NewFeaturesWetPaint/34_prancha_continuous_capsula.png).

## §4 — Portões

- `capsula::tests` (pincel, 7): a lei AO BIT contra uma referência escrita à parte (redonda e cerdas; CONTROLO a
  lei do centro muda > 200 px) · a caixa chega ao centro anterior e sem cápsula é o carimbo de sempre ao bit · o
  passo só existe com a caixa e um método com caminho (qualquer ponta; o 1.º dab sem passo; Drag Dot nunca) ·
  as leis da `Junta` · o produto corre a União · os sub-dabs.
- `tests/capsula.rs::o_continuous_iguala_o_spacing_pequeno_nos_cinco_meios` (1,85 s em dev): régua < 0,03, tinta
  ± 5 %, costura ≤ a do alvo + 0,003; CONTROLOS: sem a caixa > 0,04 (salvo o Impasto redondo, sólido), a lei do
  prompt 32 > 0,1 nos meios do carimbo.
- `shape_bristles::o_spacing_continuo_risca_no_digital_e_no_wet_paint` continua; o controlo inverteu (a caixa AGE
  na ponta lisa — pedido do dono).
- Censos do painel `a_linha_esmaecida_e_a_que_nao_age` + `o_censo_dos_controlos_so_encolhe` ✓ (91 s).
- `#[ignore]`: `sonda_capsula`, `measure_custo_da_capsula`.
- Bateria de fecho (`verificador`): `fmt --check` ✓ · clippy `-D warnings --all-targets` em 6 crates ✓ ·
  `PH2D_SUITE=fecho nextest-impacted` 18 305/18 307 em 584 s: (1) `a_altura_de_abertura_de_um_painel_so_encolhe`
  **real e herdado** — a fileira Continuous do smoke anterior (`72a6ee6bc1`): o `HEAD` sem este diff dá os mesmos
  `1 630` contra `1 605`; número novo com a conta escrita (`ROW_H_PX` 25, precedentes Mixing/tokens) ✓;
  (2) `ph2d-app-light2d … a_luz_recua_quando_a_placa_recusa_a_memoria` (wgpu, fora do diff, `load` 50–60):
  sozinho ✓ 1/1 — família de carga, falta o 3/3 calmo. Tectos de LOC ✓ (`stamp_cache.rs` 700/700,
  `stroke.rs` 698 — o próximo campo corta).

## §5 — Aberto (smoke do dono ✓ 13/10: *«funciona muito bem»*)

C13 do [26](../NewFeaturesWetPaint/26_ABERTOS.md): a rota das camadas de COR do Shape (Per-Layer Color) não lê a
cápsula; o AA do filme (Smooth Edges) desliga-se nela. B5 (o Spacing do Wet Paint pelo raio do motor) continua do
dono — por ele o «alvo» do Wet Paint é mais ralo do que o painel pede. C14 (o custo do Wet Paint, calmo), C15 (os
tectos de LOC). O flake wgpu da Luz 2D (§4) é da linha dela: falta o 3/3 calmo.
