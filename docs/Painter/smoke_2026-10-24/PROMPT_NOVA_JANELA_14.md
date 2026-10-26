# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (a meio de N1–N4; a janela 13 trocou pelo tamanho)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10 — janela 14, a meio do bloco N1–N4.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_12.md (os pedidos
do dono), PLANO_JANELA_12.md e este ficheiro inteiro. Delegue leitura e verificação.
```

⛔ Leis: sem bateria por crate nem suíte (dono 25/10) — testes por NOME + `clippy -D warnings`/`fmt` das crates
tocadas; handoff só DEPOIS do smoke do dono. ⚠️ Agentes `sonnet` (verificador, documentador, mutacao) sem cota até
27/10, 13 h: `general-purpose`/fork ou à mão. ⚠️ `cargo-test-narrow.sh` CORTA a lista de falhas de um censo: corra-o
directo (`bash scripts/ph2d-run.sh cargo test -p <crate> --test it <nome> > log`) e guarde o log.

## O que a janela 13 fez (commit WIP no topo)

- **Censo `cada_ferramenta_da_agua_so_ve_o_que_le` reescrito** (ainda `#[ignore]`, ~22 min): (a) controlo inerte na
  ferramenta só falha se AGE no Pincel da água (o chrome é inerte nos dois); (b) o gesto escondido vai DEPOIS das
  riscas, com o Pincel na mão; se encerra a sessão (o Blend com tinta, `set_blend`), a base também (ida ao Blur e volta).
- **Gaveta por ferramenta da água** (`wetpaint_preset/ferramentas.rs`): `NOMES` com `WetSmear/WetBlend/WetWater/
  WetDry/WetBlow`; `Gaveta::chave` explícita (Clone 4, as cinco 5..9); `ferramenta_dos_presets` devolve a da água;
  `ferramenta_do_preset` lê o `paint_mode` antigo do Wet (o Blender Soft de fábrica cai no Blend). `FerramentaDoPincel::
  da_agua/ferramenta_da_agua`. Pincel do Wet aplicado com uma ferramenta na mão volta ao `wet_paint`.
- **Aplicar um preset de ferramenta da água não mexe no Pincel**: escreve só os knobs dela (`WetKnobs::so_da_ferramenta`)
  + os da MÃO; o corpo vai ao slot dela. Para isso passaram a ser POR SLOT (`switch_brush_slot`): a largura prometida
  (`largura_por_slot`) e os knobs da mão (`knobs_por_slot`, só os de `mao::knobs_da_mao` voltam). O gate/papel, as cerdas
  do motor e a física ficam partilhados. Gate `aplicar_um_preset_da_agua_nao_mexe_no_pincel` (familias/mistura.rs) — ⚠️
  ainda NÃO corrido (precisa dos ficheiros de fábrica).
- **N2 fechado**: `a_agua_do_smear.rs` (ph2d-wet-paint): neutro ao bit (impressão digital igual à da lei antiga, medida
  repondo-a), a seco, molha, tecto; 4/4 mutações sangram.
- **Trilho**: gate `as_ferramentas_da_agua_moram_no_trilho_com_o_wet_paint` (editor-core, verde).
- **N4 receitas** (WET_FERRAMENTAS, `nasce_de: "Blender Soft"`, sonda `familias/mistura/sonda_agua.rs`, 6 s):
  Blend Textured/Dots/Hard, Smear Soft/Spikey/Flat/Shattered/Wet; escala do exame Blend 0,45, Smear 0,5. Fotos do
  Rebelle já existentes (`fixtures/rebelle_bateria/{BLEND,SMUDGE}`, `mistura_foto.py`). Gate
  `cada_preset_da_agua_cai_na_foto_do_seu_rebelle` (tabela `FOTOS_DA_AGUA`) e `cada_ferramenta_tem_os_seus_presets_de_
  fabrica` actualizado — ⚠️ ambos por correr. Cortados com motivo: Soft 2/Noisy (o grão ENFRAQUECE o Blend, o contrário
  do Rebelle), Soft Fast (só espaçamento, fixo pela regra), Lines (o Blend não arrasta), Spikey 2.
- Verdes por nome: tool-painter `ferramenta_tests mao_tests wetpaint_preset tests_doc22 tool_link slot showwet
  borracha_por_meio`; app-painter (feature `panel-painter-layers`) `presets biblioteca categoria` (a pasta «Smear» entrou
  em «Blend & Erase»; a gaveta do Pincel já não tem essa categoria).

## O que o censo acusou (corrida de 22 min, todas as 5 salvo nota)

À vista, age no Pincel e NÃO na ferramenta: a secção **Shape** inteira (Kind, Size X/Y, Angle, Offset X/Y, Ramp,
Reset, «alpha from image» `painter_brush.shape_alpha_from_image`), a **Marca** (`painter_brush.marca.roundness`,
`.flipX/.flipY`, `.anglePerStroke` + os `.chip`), **Jitter Rotate** (+ `jitter_rotate_chip`); `dabhandle.1` (Smear,
Blend, Wet); só no **Blow**: `pen.speedOpacity`, `pen.dabRotation`, Shape Follow, `PAINTER_WETPAINT_LIVE_WATER`.
Controlo vazio (escondidas que nem no Pincel agem nesta bancada): `DAB_QUINA`, `wetpaint_paper_visual`,
`shape_per_layer_color` — o controlo deve ser POR LINHA (`LinhaDaFerramenta`: viva se algum id dela age), não por id.
Nenhuma escondida mudou texels da ferramenta.

## Por fazer (por ordem)

1. `LinhaDaFerramenta::Forma` (o carimbo das ferramentas da água é o redondo delas: Shape + Marca + Jitter Rotate) nas
   linhas `DA_AGUA*` de `ferramenta_linhas.rs`, `ids_da` no censo, e o painel a esconder (procure onde `le(…
   DepositoMolhado)` esconde e siga); as do Blow numa linha própria ou esmaecidas com o porquê; `dabhandle.1` medir
   antes (age no Dry/Blow?). Controlo por linha. Correr o censo directo, tirar o `#[ignore]`.
2. Regerar SÓ os 8 (`PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Blender Textured,Blender Dots,Blender Hard,Smear Soft,Smear
   Spikey,Smear Flat,Smear Shattered,Smear Wet" … gera_os_presets_de_fabrica -- --ignored`; lento: ~15 min cada em
   debug — tente `--profile smoke`). Então: `cada_preset_da_agua_cai_na_foto_do_seu_rebelle`,
   `aplicar_um_preset_da_agua_nao_mexe_no_pincel`, `cada_ferramenta_tem_os_seus_presets_de_fabrica`,
   `cada_ferramenta_da_mistura_se_reconhece_pela_foto` (o Blender Soft com o caminho novo mediu orla 13 · fundo 1 —
   dentro de 17 ± 6 · 9 ± 25), e os censos de fábrica de `presets_de_fabrica.rs` (contagem, regra do Spacing, volta ao
   byte, miniatura).
3. Mutação: `no_trilho` e `slot_da_ferramenta` (o gate do preset que não mexe no Pincel deve sangrar), a filtragem de
   knobs do preset de ferramenta.
4. `clippy -D warnings` + `fmt` das crates tocadas; smoke ao dono em passos numerados (onde está cada ferramenta na
   barra, o Water do Smear no cartão, os presets novos no navegador de cada ferramenta).

## Abertos medidos (para o handoff)

- O Smear da água não abre o canal LIMPO do SMUDGE do Rebelle (resto 0,09 em 15 px): arrasta igual (110 × 119) mas o
  rasto só leva pigmento — lei do motor, não de preset.
- Os abertos do PROMPT_12 (decisões 1–3 do dono, escondidos do Digital, Borracha do Wet larga, cauda do Digital).
