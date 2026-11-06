# Continuação — tinta viva, onda 1 (06/11)

Linha `line/PainterWatercolor`, worktree `Worktrees/line-PainterWatercolor`, **à frente do main e não integrada**. Base `bbfefaf393`. Commits da onda: `c542d4f3c6` · `73c6bb5e72` · `e3bc5f7486` · `4eb160f725`.

**Smoke do dono: «smoke OK» (06/11), dito em bloco.** Ele não comentou caso a caso: o caso Watercolor abaixo não tem veredito próprio.

Desenho, tabelas e decisões: [TINTA_VIVA_2026-11-06.md](../TINTA_VIVA_2026-11-06.md), secção «Construído». Não repetidos aqui.

## O que existe e não se reconstrói

- `wetpaint/adormecida.rs`: `TintaAdormecida` e `wet_adormece` · `wet_acorda` · `wet_adormece_ou_acaba` · `wet_enterra_o_que_mudou` · `wet_acorda_da_camada` · `wet_esquece_as_adormecidas`. Orçamento: `ORCAMENTOS_DE_UNDO = 4`.
- `Engine::enterra_celulas` (`ph2d-wet-paint`): não marca sujo; por isso acordar muda 0 px. A orla recalcula sob o pincel seguinte.
- Portas: `flush_active_to_images` adormece (é a porta de todas as trocas de camada); `set_active_layer`, `mode_switch`, o tique em `ciclo.rs` e `ensure_wet_session` acordam.
- Macro: gravar/tocar chama `wet_esquece_as_adormecidas`, de propósito, para não mudar macros gravadas.

## Números

| Medida | Resultado |
|---|---|
| Traço, mediana em ms a 4096×2560 (sem · sem2 · dorme · dorme+tela) | Digital 28,27 · 28,68 · 29,06 · 27,42 — Watercolor 425,44 · 432,52 · 430,02 · 427,47 — Impasto 59,38 · 58,81 · 58,83 · 56,77 |
| Tique ocioso, mediana em µs | 71,68 · 70,68 · 71,74 · 70,06 |
| Adormecer | 0,01–0,02 ms |
| Memória retida, **sem compressão** | 164 MB em 1080p · 577 MB em 4096×2560, por camada cheia |
| Defeito (px diferentes entre Borracha viva e depois de trocar de modo) | antes 2 483 · depois 0 nos 3 meios |

Régua: `sonda_tinta_adormecida_nos_outros_modos` (ignorada), variantes intercaladas no mesmo processo.

## Verificado em `4eb160f725`

fmt e clippy das 2 crates · `--lib` filtros `wetpaint macro_track tool::layers mode_switch` 497 ok, 195 ignorados · alarmes de pincel 2/2 · `seam_wetpaint` 33/33 · `architecture_workspace_file_loc_cap` 2/2 · binário `--profile smoke` compilado. **Sem provas de mutação ainda; suíte completa só na integração.**

## Hipótese não isolada

Depois de um Watercolor na mesma camada, a Borracha difere (teste aceita < 300 px com Δ ≥ 10; medido 6 574 px, Δ máx 64, 229 com Δ ≥ 10). Grades e base são iguais, e atribuí ao vidro da aguada (`composite.rs:253`, doc 48). **É inferência.** A hipótese «recompor óleo vivo não é idempotente» foi refutada (0 px).

## Próximo, por ordem

1. **Resto do item 2:** dentro do Wet, o guarda (`ciclo.rs:255`) e o undo (`layers/undo.rs:182`) ainda matam a sessão: fill sem trocar de modo, ajuste por menu, desfazer traço de outro meio já acordado. Óleo enterrado não volta se o traço de cima for desfeito.
2. **Item 4:** Borracha inteligente na tinta morta. Foto vivo × imitado para o dono aprovar.
3. **Compressão em RAM (zstd 1), itens 5 e 6, em bloco com a medição:** a compressão não foi medida nem construída; rever o orçamento de 4× depois dela.
4. **Item 5:** opção ao salvar, **ligada por omissão**: campo no FIM do `PaintedDocument`, degrau por `scripts/schema-recount.py`, migração, ida e volta igual ao bit. Primeiro o sem perda (cor morta `canvas_ops` susp_rgb, derivável, caixa/máscara, bytes separados). Quantizar **só com foto aprovada pelo dono**.
5. **Item 6:** preço da compressão, 4K com várias camadas, redo.
6. **Provas de mutação** dos portões de `adormecida_tests.rs`, e fecho.
7. Depois: [PROXIMA_FASE_2026-11-06.md](../PROXIMA_FASE_2026-11-06.md), itens 0–3. A foto 5 refeita ainda não chegou.

`project-memory/MEMORY.md` está acima do alvo de 17 KB: cortar uma entrada ao mexer nele.
