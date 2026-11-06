# HANDOFF — rodada de integração de 06/11 (4 linhas)

Leitor: a próxima LLM e o próximo integrador. Ordem do dono de 05/11: integrar as 4 quando os handoffs ficassem prontos, depois ship + CI, fim de dia e desligar o PC.

## Ordem, medida por `git merge-tree` par a par

PixelArt → PainterWatercolor → GameEngineFuncional → SfxLab, a partir de `c61f228b05`. Conflitos reais: `project-memory/MEMORY.md` (GEF × Painter), `CLAUDE.md` §5.1 (SfxLab × as outras), `Cargo.lock`, `crates/ph2d-app-game/Cargo.toml` e `architecture_every_editable_field_is_a_channel.rs` (GEF × SfxLab).

Handoffs das linhas: [Pixel P9](../../PixelArt/handoffs/HANDOFF_INTEGRACAO_line_PixelArt_2026-11-05_P9.md) · [Painter noite](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-05_NOITE.md) · [GEF Sprite](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-06_SPRITE.md) · [SfxLab](../../SfxLab/handoffs/HANDOFF_INTEGRACAO_line_SfxLab_2026-11-05.md).

Números que somam: `PROJECT_SCHEMA` 217, ecs 118 / render 119 / script 119, ADR 0198.

## O que a árvore combinada pediu

- **Catracas:** shell +28, `ph2d-app-components` +102, `ph2d-app-sprite-anim` +99. Curadas por movimento, sem subir número: crates novas `ph2d-pick-order`, `ph2d-audio-card-facts`, `ph2d-anim-signals`. Folga menor: `ph2d-app-components` 20.
- **27 vermelhos da suíte**, curados em commits por linha de origem (`fix(integ 06/11): …`).
- **`um_instantaneo_declara_o_que_retem`:** 1,145 × `FOLGA` 1,15, bimodal sob carga (+4 MB). O instantâneo não declarava ~13,7 MB/foto: fotos do undo vivo 6,0 · janelas do undo ~3,0 · journals ~3,0. O bimodal era da medição: buffer transitório de 4 MB ainda vivo quando o `base` é lido. Cura: declara e o gate aquece antes de medir; agora 1,016 parado, sob carga e em smoke. `FOLGA` intacta.
- **`nenhum_pincel_de_fabrica_clareia_o_papel_de_cor`:** timeout de 180 s só na suíte; 7,9 s sozinho. Foi para a faixa `10×60 s` do nextest.

## Para o dono decidir (entrou sem ele ver)

1. **SfxLab W2:** o smoke não foi feito; o handoff pede que ele saiba. O kill-criterion do corpo vibrante está aberto.
2. **Painter:** `aa34786e7f` (preparo do 1.º toque) e `706de79944` (piso do alfa da Borracha, `meio_do_dente` fora do Paper) vieram depois do smoke.
3. **Painter, painel:** o Taper Fall Off deixa de esmaecer para a Borracha no Wet; o Edge Darkening some do cartão da Borracha no Wet.
4. **Smooth Edges 0,3 nos óleos (aprovado):** custo medido no traço fino, Dabber 5 px ÷ 40 px 0,85 → 0,73 e Express Oil 0,95 → 0,84. `leis_sistemicas` e `os_fios` passam a medir com Smooth Edges 0; o pin do Express Oil em `a_lei_da_tinta` foi regravado com a prova. `larguras_aprovadas.tsv` e miniaturas não mudaram. Um valor menor nos óleos finos regravaria largura e miniatura: decisão dele.
5. **Áudio:** o chip vazio do Pick mostra «—», porque nem «None» cabe no degrau estreito.

## Aberto

- `a_luz_nasce_na_estufa`: SIGABRT `free(): invalid pointer` no encerramento, 1× na suíte sob load 26–43; 11 corridas sozinhas verdes. Causa não achada e **não promovido** a flake: não casa a assinatura de `FLAKES_DE_CARGA.md`. Se repetir, é corrupção de memória a investigar.
- Oráculo novo de `the_wet_eraser_lifts_the_fluid_and_the_water_survives`: só sangra cortando as DUAS rotas que levam o apagar ao ecrã (`dispatch_pressure_dab_erase` e `borracha_na_base.rs`); cada uma sozinha é redundante.
- Item 0 do plano do Painter: [PROXIMA_FASE_2026-11-06](../../Painter/PROXIMA_FASE_2026-11-06.md).
- Tags `lei-antiga`, `lei-nova-arquivada` e ramo `backup/lei-antiga-2026-11-05`: seguem só locais.

## Lição

`git merge-file --union` e `rerere.autoupdate=true` produziram resolução errada em silêncio nesta rodada: ver `reference_topic_git_hazards`.
