# Prompt de continuação — linha PainterWatercolor · 13/11 n.º 9 (depois da janela n.º 8)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-13_desempenho_9.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`, sobre o
`main` `4f46a3573c` (não faça `git rebase main` se `git log HEAD..main` for vazio). Ordem do dono (continua): «vamos seguir até ter
o app plenamente otimizado» · 13/11: «faça o que for melhor para resolver tudo logo». Não integre, não envie.

## Leia primeiro
1. O prompt n.º 8 ([PROMPT_CONTINUACAO_2026-11-13_desempenho_8.md](PROMPT_CONTINUACAO_2026-11-13_desempenho_8.md)): a fila, as regras.
2. [`../missao_4k/medicoes/N8_NOTAS.md`](../missao_4k/medicoes/N8_NOTAS.md): tudo o que a janela n.º 8 mediu e recusou.
3. O handoff n.º 7 (ONDA7_DESEMPENHO) §6–§7.

## Feito na janela n.º 8 (commits)
| commit | o quê |
|---|---|
| `971585a122` | item 2: WB apagar/duplicar/grupo na camada nova = tinta ∪ vidro (app: 90/72/78 → 22/16/17 ms) |
| `7649aacc5f` | cartão **Performance** com «Faster wet paint» (o palpite), gravado em prefs `wet_guess`; **AVX2 MEDIDO e recusado** (≈ 0; tecto 4 %) — o dono foi avisado |
| `cbd47ce04b` | item 3: Impasto apagar/duplicar/grupo = tinta ∪ altura ∪ cobertura + 1 texel (gate 36 casos; app NÃO re-medido em calma) |
| `3026ea2e53` | o `[quadro-lento]` imprime as partes do Painter no quadro |
| `69158c3890` | **defeito achado**: o Ctrl+Z logo após um traço Wet sobre aguada (papel de cor) não desfazia o vidro do escorrido — a absorção agora o leva (`vidro_no_reset`). Teste `o_composite_wet_sobre_aguada_volta_ao_bit_no_desfazer` com relógio FIXO (era vermelho 1 em 3 com o de parede) |

## Aberto, por ordem
1. **Re-medir em calma (load < 5)** a matriz NOVA=1: Impasto (item 3), WB (item 2) — a bateria 2 correu com load 15–19.
2. **Custo da cura do vidro** (`69158c3890`): quando o escorrido escreveu o vidro, o pen-down/Ctrl+Z seguinte clona o vidro
   inteiro da camada (7 B/px: ~58 MB a 4K). Medir no Ctrl+Z do Wet sobre aguada; se pagar, rebobinar só os tiles do journal.
3. **Item 4/5 (Wet trocar / Ctrl+Z na camada nova)**: as partes (N8_NOTAS, bateria 2) dizem preview 18–30 ms com a **dobra
   do relevo 12–16 ms** — porque a dobra inteira (`fold_full`)? E quadros de 35 ms com o Painter a 0,4 ms (FORA dele).
4. **Impasto duplicar**: Painter 7 ms de 35 — o resto fora do Painter (shell/apresentação/placa).
5. Itens 6–10 do prompt n.º 8 (toque Filbert/Composite, CPU parado = shell, B19.10/B20.6/rayon, espera do pen-down, `fork_par_raw`).
6. Vermelhos ANTERIORES vistos (não desta linha nesta janela): 4 de Impasto (`the_impasto_deposit_is_pinned_plane_by_plane`,
   `o_pigmento_mistura_como_tinta_no_impasto`, `impasto_shine_glints…`, `wax_bleeds…`), os de preset (`cada_preset_de_fabrica_…`
   57×56, `receita_e4` 46×45…), `h2_os_cartoes::the_top_and_the_basics…`, `the_shell_only_shrinks_and_every_app_crate_too`
   (app-painter/app-pixel), `quantas_entradas…a_carga…` (`wet_tuning` 12×14). E um FLAKE de isolamento:
   `o_motor_nao_le_a_ponta_alem_da_pegada_e_a_tela_e_a_do_oraculo_ao_byte` reprova SÓ a correr junto com outro teste do Wet
   (contagem de leituras partilhada); sozinho 4/4 verde.
7. Fecho da linha: fmt/clippy (`PH2D_SUITE=fecho`), alarmes de pincel por nome, matriz normal e NOVA=1, handoff NOVO + link no
   CLAUDE.md §5.1, `rm -rf target/*/incremental`, binário smoke 2×.

## Regras novas (n.º 8)
- Rota de sonda nova: confira que a LETRA não existe já (`x` era `ADVECT_FASES`; o AVX2 mediu-se a 1.ª vez errado).
- Medir no app com outro agente a compilar a frio dá load 15–19: corra a matriz SEM agentes pesados ao lado.
- O executável do smoke envelhece em silêncio: recompile ANTES da foto (a 1.ª foto do cartão era o app de antes).
- `rustfmt` em `lib.rs` formata a árvore toda da crate — confira `git status` depois.
