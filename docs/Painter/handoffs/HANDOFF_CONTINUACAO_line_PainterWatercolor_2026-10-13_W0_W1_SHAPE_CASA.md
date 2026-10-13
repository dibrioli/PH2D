# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, plano 27 (a seção Shape é a casa dos pincéis): W0 medida e W1 feita (2026-10-13, prompt 28)

## As regras (R1–R9, [PROMPT_LINHAS_RAPIDAS §2](../../archive/integracao-jornadas/PROMPT_LINHAS_RAPIDAS_2026-10-11.md))

- R1 janela curta: ~150 chamadas ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `cargo-check-narrow.sh` (⚠️ cego a `tests/it`) → teste.
- R4 no laço, teste por NOME; a suíte e o `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- R5 nada de `until … sleep`: > 2 min = `run_in_background`. R6 chamadas independentes na MESMA mensagem.
- R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset. R8 escreva menos.
- R9 nenhum teste da suíte normal passa de 10 s em `dev`; só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`.
- Comando pesado por `bash scripts/ph2d-run.sh …`. Fish/zsh: listas de caminhos num `bash -c '…'`.

## §1 — Coordenadas

Sobre `0d8487ce0f` (o `main` não andou: merge-base `757afebd58`). Três commits desta janela (W0+W1; o smoke do dono; W2 + a investigação do Spacing). Sem push.

## §2 — W0 (medir, sem mudar o produto) — tudo no [27 §6](../NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md)

- **Tabela do #47 inteira** (`wetpaint/sonda_shape_casa.rs::sonda_shape_casa_tinta_por_tipo`): o #47 é uma FAMÍLIA —
  Noise 0,004 · Clouds 0,013 · Musgrave 0,005 · Stucci 0,002 · Voronoi 0,048 do redondo no Wet Paint; no Digital
  0,15–0,94.
- **Leis candidatas** (`sonda_shape_casa_leis_do_dente`, pigmento que pousa, dab real pelo `on_dab`): a lei **B**
  (o padrão do Shape DEPOIS do portão) deixa todo redondo ao bit e leva o Noise do Wet Round de 0,004 a 0,093; a
  **A** (portão × envolvente) muda os redondos (Wet Round +36 %, pastéis ×2,7) — recusada. O resto do Noise sob B é
  a correlação cerdas × padrão (ambos presos ao centro do dab).
- **Oráculo** (Rebelle, WebSocket; fixtures `fixtures/rebelle/ponta_textura_*`, script `ferramentas/ponta_textura_rebelle.*`):
  ponta texturada comum MULTIPLICA e acumula, sem limiar; o limiar só aparece onde o autor escureceu o grão.
  ⛔ O papel não desenha sob Wine (D6 ponta × dente e D7 Shape × Grain não se medem lá).
- **Censo:** `BrushShape::{Flat, Fan}` mortos no produto (saem na W5); os 4 knobs das cerdas e onde se escondem — 27 §6.4.

## §3 — W1: Bristles é o 30.º tipo do Shape (e do Grain)

- **UM gerador:** `ph2d-painter-brush` passa a depender de `ph2d-wet-paint` e usa `build_bristle_texture` /
  `sample_bristle` do motor. Cache do ladrilho por THREAD (`texture/bristles.rs`, chave = os bits dos 4 sliders) —
  um cadeado partilhado serializaria as caminhadas paralelas do carimbo.
- **Sliders = knobs:** `ph2d_wet_paint::brush::unidade` — `BristleKnobs::{from_unit, to_unit}`, Count · Strength ·
  Size · Felt, faixa e passo do Tuning (`KNOB_DEFS`), valor refeito como `inteiro × num ÷ den` ⇒ ida-e-volta AO BIT
  em todo passo (`30 ÷ 1000` é o literal `0.03`). Sem slots de tom (a Strength é o nível).
- **Píxeis:** `texture::sample` devolve `sample_bristle(dx, dy)` a partir do centro do dab (sem mapping, Size,
  Angle, Offset); `sample_kind` (pré-visualizações) lê o ladrilho a 64 px por unidade da pegada.
  `TextureKind::samples_in_pixels` ⇒ `is_cacheable`/`is_canvas_cacheable`/`dab_mask_cacheable` recusam (nenhum
  carimbo independente do raio guarda as cerdas).
- **Padrão do tipo = o tapete do Wet Round com a Strength 2** (não 1): a 1 os picos (0,16–0,32) ficam sob o
  `height_film::W_TAIL` 0,35 e o traço do Impasto sai VAZIO (medido 0 · 1,5 → 43 688 · 2 → 192 835).
- i18n: `texture_kind.bristles` + `pattern_param.{bristle_count, bristle_strength, bristle_size, felt}`.

## §4 — Portões

- `texture::bristles::tests` (3): o ladrilho do Shape = o do motor AO BIT para os knobs dos 12 presets + o padrão;
  a lei em píxeis (o mesmo texel com raio 4 e 60 = `sample_bristle`; CONTROLO o Noise muda com o raio); nenhum
  carimbo cacheado (CONTROLOS: o pincel liso e o Noise ladrilhado são cacheáveis).
- `brush::unidade::tests::a_ida_e_volta_pelo_slider_e_exacta_em_todo_passo` (motor).
- `tests/shape_bristles.rs` (2): Digital e Impasto pintam com a ponta de cerdas, menos que a redonda,
  determinístico, Count e Size mudam o traço · cada tipo (30) e cada parâmetro tem texto.
- Bateria de fecho (`verificador`): `fmt --check` ✓ · clippy `-D warnings --all-targets` nas 5 crates ✓ (um `type_complexity` curado) · `nextest-impacted` (base = merge-base, árvore suja) **18 291/18 291** ✓ em 653 s (`load` 17) · gates de arquitetura (camadas, ciclos, LOC, contratos) 155/155 ✓ — a dependência nova `painter-brush → wet-paint` passou no gate de camadas.

## §4b — Smoke do dono (13/10, 2.º commit): cada parâmetro do Shape age sobre as cerdas

O 1.º Size, Angle, Offset, Rake/Flow e o Jitter Rotate estavam MORTOS com Bristles nos três meios (censo
`sonda_cada_controlo_com_as_cerdas_no_shape`). Cura: `bristles::sample_dab` aplica a transformação inteira do
Shape em píxeis (neutro = exacto). Nomes: Bristle Count/Strength/Size · Felt. As cerdas mostram-se e digitam-se
nas unidades do Tuning (`param_escala`, as 4 portas: Shape, Grain, camada de textura, papel). Detalhe e portões:
[27 §6.6](../NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md).
Bateria do 2.º commit: fmt ✓ · clippy `-D warnings --all-targets` ✓ · `nextest-impacted` **18 294/18 294** ✓ (489 s) ·
suítes do painel 239 ✓ · censo das cerdas: zero controlo do Shape morto nos três meios.

## §4c — W2 (3.º commit): o Wet Paint lê as cerdas do Shape

Shape Bristles = o `texv` do motor pela porta do Grain (`texture::bristles_at` em células; a cerda segue só o rotor
por dab, `FootprintDeform::so_o_rotor`); Shape ≠ Bristles = ponta lisa. Fábrica e presets antigos chegam com o Shape
em Bristles; gravar não põe as cerdas no mapa; o Tuning esconde os 4; `Dependente::CerdasDaAgua`. **As 12 miniaturas
de fábrica ao byte pela rota nova.** Fixturas de teste do Wet Paint com o Shape vazio passaram a pôr as cerdas (o
vazio é agora a ponta lisa) — passaram sem mudar nenhum número. Censos: as linhas do Rotate e do Image no Wet Paint
saíram da lista dos inertes (agem); o Offset/Size XY passaram a esmaecer (`paint_num_xy`); o Follow NÃO é das cerdas
(roda o Grain). Detalhe e tinta por tipo: [27 §6.7](../NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md).
**Investigação do Spacing** (relato do dono): [31](../NewFeaturesWetPaint/31_spacing_por_meio_o_pontilhado.md) —
medida, causa e desenho; próxima janela = [prompt 32](../NewFeaturesWetPaint/32_PROMPT_janela_spacing_continuo.md),
depois o [prompt 30](../NewFeaturesWetPaint/30_PROMPT_janela_shape_casa_W3_W4.md).
Bateria W2: fmt ✓ · clippy `-D warnings` ✓ (com `panel-painter-layers`) · `nextest-impacted` 18 295/18 296 (1 prazo a
`load` alto) e, de novo a `load` 116–237, 2 de relógio (`the_mask_stroke_cost…`, `the_authoring_pause…`): sozinhos 2/3 e 3/3
— família de carga (o caminho da máscara não passa pelo diff).

## §5 — Estado e o que fica ABERTO

- O Blend do Wet Paint usa o carimbo do MOTOR (o tapete do Tuning escondido, com os valores do preset) — o Shape
  não o alcança; a ponta lisa deposita 4,7× a tinta das cerdas (conferir no smoke, prompt 30).
- O Paper do Watercolor também lista o Bristles (o dropdown deriva de `TextureKind::COUNT`); mídia congelada, sem
  dano — a ponta não muda o papel de quem não o escolhe.
- W3 = a lei B no motor (um 3.º canal `sv` em `stamp_row_shaped`); W4 o painel por mídia (C1); W5 Flat/Fan + doc.

## §6 — Smoke ao dono (Digital)

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`
→ Painter ▸ Shape ▸ Texture ▸ **Bristles** → riscar: ponta de cerdas (riscas finas ao longo do traço); mexer em
**Count** (menos/mais pelos) e **Size** (pelos mais finos/grossos) muda o traço. Errado: traço liso igual ao
«None», ou vazio.
