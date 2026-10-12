# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, o Bug #51 e o Randomize Color «a cada traço» (2026-10-12, prompt 24)

## As regras (R1–R9, [PROMPT_LINHAS_RAPIDAS §2](../../IntegracaoMultiAgente/PROMPT_LINHAS_RAPIDAS_2026-10-11.md))

- R1 janela curta: ~150 chamadas ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `cargo-check-narrow.sh` (⚠️ cego a `tests/it`: use
  `cargo check -p ph2d-tool-painter --test it`) → teste.
- R4 no laço, teste por NOME; a suíte e o `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- R5 nada de `until … sleep`: > 2 min = `run_in_background`. R6 chamadas independentes na MESMA mensagem.
- R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset. R8 escreva menos.
- R9 nenhum teste da suíte normal passa de 10 s em `dev`. Cura: (a) `opt-level = 2`; (b) fixture menor que
  ainda morda (prove com mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`. ⛔ Lei nunca.
- Comando pesado por `bash scripts/ph2d-run.sh …`. Fish/zsh: listas de caminhos num `bash -c '…'`.

## §1 — Coordenadas

Sobre `348c81d206` (o `main` não andou: merge-base `9ea782138`). Um commit desta janela. Sem push. Perfil do laço
ANTES (10 sessões): paralelismo 1,15 · test:check 4,1 × · Edit 45 % · contexto/passo 350 mil · início 63 mil.

## §2 — O que mudou

**A lei (#51 a–d).** Uma camada da Shape com cor de textura pinta `textura × tinta`, deslocada pelo desvio
SORTEADO do carimbo — uma porta: `stamp_color::texture_times_tint`, nos três núcleos (`render_color_stamp_mask`,
`accumulate_shape_layer_rgba`, `accumulate_shape_layers_rgba_batch`). Tinta branca e desvio zero = a textura ao
byte. O `Dab` ganhou `color_shift: [dh, ds, dv]` (o `shift_colors_like`, que o reconstruía, saiu); a camada
CHAPADA recebe o desvio uma vez por carimbo (`stamp_color_dynamic.rs`), a de textura por píxel no núcleo.
`ShapeLayers::resolved_colors` dá a tinta por modo: escolha própria · textura com UMA camada = o Color · textura
com várias («Per-Layer Color») = branco · sem RGB = o Color. A chave do carimbo cacheado já tinha `brush_color` +
a versão das camadas ⇒ a tinta está nela. **Color → branco** (`texture_colour_starts_untinted`) ao associar a
imagem com o modo ligado (`set_brush_shape_image_rgba`, captura de camadas NOVA) e ao ligar o modo — nunca na
re-captura automática da MESMA fonte (`capture_layers(automatic)`), e nunca no Wet Paint / Watercolor, que não
lêem a cor da textura (`texture_colour_paints`). **Color INATIVO** em Per-Layer: `BrushSettings::color_inactive`
(= nenhuma camada lê o Color, `brush_color_is_read`); o painel pinta a linha esmaecida (`esmaecer::esmaecido`),
sem alvo de clique (`color_row_without_hit`) e fecha um selector aberto nela.

**Randomize Color «a cada traço».** `BrushSpec::color_jitter_scope: ColorJitterScope` (`PerDab` padrão,
`PerStroke`). O sorteio por traço sai em `Stroke::new` (`stroke_colour_draw`) de um fluxo PRÓPRIO
(`seed ^ "colourST"`); os sorteios por carimbo continuam a correr ⇒ escala/rotação/posição saem ao bit nos dois
modos. A Elipse/Linha/Curva refazem o traço com a semente do editor ⇒ a cor é do traço, não do Move. Viaja no
preset JSON (`CAMPOS_DO_PINCEL` 28 → 29, chave `color_jitter_scope`; ausente = `PerDab`), no instantâneo, no
Reset da secção; painel: segmentado «Every Dab / Every Stroke» (`PAINTER_BRUSH_COLOR_JITTER_{SCOPE,PER_DAB,
PER_STROKE}`, registado no `populate` e no `event.rs`; i18n `brush.{randomize_when,every_dab,every_stroke}`).
O `BrushSpec` NÃO entra no `.ph2dproj` (só o preset) ⇒ sem degrau de schema.

**Os literais do `Dab`:** 34 ganharam `color_shift: [0.0; 3]` pelos erros do compilador (E0063, um script com
contagem, 18 ficheiros); 2 levam o valor real (`dab_build.rs`, `DynDab` no caminho dinâmico); o portão de
paridade do lote passou a correr com desvio (`SHIFT`). Dois testes de carimbo à mão com a cor «implícita»
(`per_layer_color_dynamic_randomize_…`, `…_jitters_custom_layer_colours`) dizem agora o desvio.

## §3 — Portões

- `textura_tinge` (10): os três do #51 vistos VERMELHOS antes da cura, agora leis · Color → branco ao ligar o
  modo · Per-Layer: inactivo e não tinge · Wet Paint: a imagem não apaga a tinta · «a cada traço» (a) traço numa
  cor, o seguinte noutra · (b) com a textura do #51 · a Elipse editada mantém a cor.
- `ph2d-painter-brush`: `per_stroke_colour_changes_only_the_colour` (centro/raio/rotação ao bit nos dois modos) ·
  `the_dab_carries_the_offset_that_made_its_colour` · `per_dab_randomize_is_byte_identical_to_before_the_scope`
  (impressão `0x6e2c_ccd1_5f9e_0cff` tirada no `348c81d206` numa worktree temporária com alvo próprio).
- Painel `seam_color_inativo` (2): o Color inativo não é alvo, o layout não mexe, o clique não abre o selector
  (CONTROLO: activo abre) · o segmento Every Stroke chega ao pincel.
- **Mutação** «sorteia por carimbo» (`stroke_colour` ignorado no `dab_build`): sangram os 4 portões «a cada
  traço» (1 do motor + 3 da ferramenta); reposto, `cmp` com a cópia e `touch`.

## §4 — Custo (medido)

`measure_custo_do_desvio_na_textura` (`--release`, ignorado): o lote dinâmico de um Move (2048², r100, 12
carimbos, todas as camadas de textura), desvio zero × sorteado, no MESMO processo, intercalado, o mínimo de 10
blocos, `load` ~28: **N3 4,20 → 6,27 ms (1,49×) · N16 23,4 → 31,3 ms (1,33×)**. Só paga quem liga o Randomize
com cor de textura; sem desvio é o produto e mais nada. Não otimizado (§0.0: medido, sem tecto a defender).

## §5 — Varredura: o que mais deixa de agir (passo 5)

- **Wet Paint e Watercolor nunca lêem a cor da textura** (`stamp_route.rs`: as duas desviam antes da rota de
  cor, ~105/151; a tinta é `d.color`). Lá o Color e o Randomize agem (chapados) e o «Use Texture Colors» é um
  **controlo morto**. Curado nesta janela só o que o #51 tocava (o Color não vai a branco nem fica inactivo
  nessas mídias). ⏳ ABERTO: esmaecê-lo nelas é um `Dependente` novo em `inercia.rs` + a linha no censo
  `censo_dos_controlos` (a sonda que mede inerte).
- **Máscara** (`stamp_route.rs` ~290): pinta com `Dab { color: preto/branco, ..*d }` e o `brush.color` igual ⇒
  com a cor de textura ligada a tinta branca dá a textura, a preta dá preto. Antes do #51 pintava SEMPRE a cor
  da textura (pior). ⏳ Decidir se a máscara deve ignorar a cor da textura (luminância).
- Sem efeito do #51: depósito sólido (`solid_deposit.rs:234`, nunca teve Randomize) · GPU (só `d.color`; a rota
  de cor vem antes) · Simetria/Tiling copiam o `Dab` inteiro · Composite/Smear/Blur/Clone não lêem cor.

## §6 — Fecho

Bateria (`verificador`, em série, `load` 14–24): `ph2d-painter-brush` · `ph2d-tool-painter` lib + `it` em `dev` e
`ci-test` (`ci-test` 1576/1576; em `dev` 2 flakes de CARGA da máscara, `the_mask_stroke_cost_does_not_follow_the_canvas`
e `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`, alternam a `load` 23 e passam sós) ·
`ph2d-paint-gpu` · `ph2d-app-painter` · censos registry 134 · fmt · clippy `-D warnings` · typos · tecto 700 ·
`nextest-impacted` 18 123/18 125. Os 2 vermelhos determinísticos eram meus, curados e re-corridos verdes:
(1) `texture_color_layer_paints_its_own_rgb_not_the_flat_tint` afirmava a lei VELHA ⇒ reescrito
`…_its_rgb_times_the_tint` (tinta branca = textura; vermelho × azul = preto); (2) `o_censo_so_encolhe` acusou o
«Every Stroke» inerte com o Randomize a zero ⇒ `Dependente::QuandoDoRandomize` (esmaecido, dica
`inerte.randomize_above_zero`) + a pré-condição «Randomize Hue 0,8» no `censo_dos_controlos` (age). ⚠️ O
`a_linha_esmaecida_e_a_que_nao_age` leva 66 s em `dev` (pré-existente; R9 aberto). Binário `smoke` compilado.
Perfil (10 sessões, antes → depois): paralelismo 1,15 → 1,15 · test:check 4,1 → 4,3 × · Edit 45 → 42 % (lotes
por script com `assert`) · contexto/passo 350 → 353 mil.
