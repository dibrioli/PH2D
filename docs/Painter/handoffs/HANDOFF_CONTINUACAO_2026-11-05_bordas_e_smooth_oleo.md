# HANDOFF DE CONTINUAÇÃO — line/PainterWatercolor, 05/11/2026

Leitor: a próxima janela da linha. Base: `main` em `c61f228b05`; linha reaberta por `reset --keep` (cópia anterior em `backup/line-PainterWatercolor-pre-reabertura-2026-11-05`). Não integrar sem ordem do dono.

## Feito nesta onda
- **Hardness por ferramenta** (dono: smoke OK): `tool_link.rs`, macro `hardness_dos_modos` / `hardness_da_borracha`; 8/8 mutações.
- **Borracha toca o anel** (Bug #68, **smoke por fazer**): `largura_da_tinta()`. ⛔ Estender às ferramentas da água muda 10 pincéis de fábrica; está recusado.
- **Régua do grão** só lê o pincel do lote. **Mutação `sombra`** morta. **`prazo-do-agente.sh`** deixa pôr na fila.
- **Cena `PH2D_WETPAINT_SMOKE=4`**: casos da lei antiga no papel padrão (`crates/ph2d-app-painter/src/wetpaint_casos.rs`). A ferrugem da `dono_6` não reproduz.
- **Smoke do dono do handoff de 05/11:** A, B, C, D e Hardness OK; F fica; **E: macro com Grain bugada, NÃO corrigir nesta linha**. Memória `project_smoke_2026_11_05_owner_verdicts`.

## Decisões do dono que mudam o plano
«Adoro os pincéis como estão agora», e não julga furos por foto. O app pinta com a lei **VELHA** (`lei_da_tinta` por omissão; a nova só nos testes). Memória `project_brushes_as_they_are_are_the_reference_2026_11_05`. **Não refaça a régua de furos nem a grelha de 3 h.** E12/E13 paradas.

## ABERTO, por ordem
1. **Bug #69, cura:** Edge Darkening por célula. Mecanismo, números e portão em [BUGS #69](../BUGS_painter.md). Vermelho primeiro. Decida: duas tintas na célula, custo por célula medido, caminho da placa. Depois alarmes de pincel por nome. Confira se a cena 4 deixa de variar por isso.
2. **Smooth Edges 0,3** (ordem do dono) nos óleos com buracos ≥ 5 % à pressão 1, lei velha, papel padrão (grelha de 05/11): Express Oil, Dabber, Dabber 4, Filbert, Filbert Dry, Flat Bristle, Tilted Shade, Knife Grainy, Knife Oily, Knife Smooth 3, Knife Stain, Knife Textured, Metallic Flat. Ficam: Thick Impasto, Knife Dry. A lista foi mostrada ao dono, sem objeção até aqui. Alarme dispara: regrave por ser ordem dele e mostre o relatório no smoke. A grelha cobre só 15 pincéis de óleo.
3. **Pergunta ao dono ainda sem resposta:** lei antiga como oficial e a nova guardada (recomendado), ou continuar a nova? Se ficar a antiga, oferecer consertar laranja e branco (Bug #54), que aparece também no papel padrão; o conserto testado muda 25 miniaturas, logo por foto.
4. Fecho da onda: `/pd-onda-fechar`, smoke em passos, incluindo Borracha (Bug #68) e bordas (#69).

Pequenos, sem pressa: Grid/Crosshatch (#59), Bug #63, ajuda ilustrada F5, `bancada_da_agua`, prova do mesmo byte noutro Linux. Os dois vermelhos do fecho 05/11 não reproduziram (BUGS #65): vermelho deles investiga-se, não se promove. O guarda `prazo-do-agente` recusa a palavra «bancada» em qualquer comando de subagente.
