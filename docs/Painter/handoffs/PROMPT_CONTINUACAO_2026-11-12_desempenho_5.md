# Prompt de continuação — linha PainterWatercolor · 12/11 n.º 5: o Painter plenamente otimizado (o que sobra)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-12_desempenho_5.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. ⛔ **NÃO faça `git rebase main`** (merge-base `bc4a6ce930`; quem funde é o
integrador que o dono nomear).

## Ordem do dono (verbatim, continua)

12/11: «vamos seguir até ter o app plenamente otimizado». «Se esforce para resolver todos os problemas de performance do módulo a todo
custo. … Não esqueça do handoff de integração ao final de tudo. Não integre.»

## Leia primeiro

1. [HANDOFF_INTEGRACAO_…_SELECAO_CAMADAS_E_RITMO.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_SELECAO_CAMADAS_E_RITMO.md)
   (n.º 4: §4 o que entrou, §8 aberto, **§12 os três defeitos do dono, o ABERTO mais recente**) e os três anteriores (`…_MISTURA_E_ENVIO.md`, `…_LUZ_E_CAMADAS.md`, `…_DESEMPENHO_TOTAL.md`),
   que continuam a valer.
2. `../missao_4k/BACKLOG.md` e `BACKLOG_AUDITORIA.md`; `../missao_4k/medicoes/DEPOIS_N4.md` (a matriz do app, com o aviso de carga).

## Estado (depois da §12 do handoff)

| item | estado |
|---|---|
| 3 mistura, 4 água do Wet (piso serial do advect), 5 camadas e seleção, 6 cadência | FEITAS (n.º 4) |
| B20.11b formigas em ecrã · B18.4 1.º toque da camada nova (Wet 51,7 → 23,5 ms) · B13.12 curas 1b/3/4 · B18.5 determinismo | FEITOS (§12) |
| sim a 6 traços | ~0,89 sim-s/s (era 0,49) |

## O que fazer, por ordem (até ao fim, sem parar)

1. **A sim a 6 traços em tempo real:** bytes por célula no advect (momentum no bloco, active/uv relidos, dst_rgb) e a cura 1b da
   secagem (x/y escalares no resolve/assenta); `sonda_n_tracos_em_tempo_real`, A×B no mesmo binário, ao bit.
2. **Watercolor Basic, 1.º toque na camada nova** (~64 ms, ~64 MB de memória a mais, por achar).
3. **O nascer da sessão Wet** (10–14 ms no mouse-down, 27 MB).
4. **Re-medir a matriz do app com calma** (load < 5): `bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel`.
5. **O resto:** ruído espalhado do B19.10, B20.6 a parte da placa, B23.2.
6. Por item: medir antes (A×B, mínimo), curar AO BIT, gate por nome com mutação, medir depois, comitar com a tabela. **Fecho:**
   fmt/clippy `PH2D_SUITE=fecho`, alarmes de pincel por nome, matriz final, handoff NOVO + link no CLAUDE.md §5.1,
   `rm -rf target/*/incremental`, binário smoke 2×. Não integre, não envie.

## Regras aprendidas (12/11 n.º 4)

- **O subagente não corre `--ignored` nem a fila pesada:** as sondas pesadas são do coordenador.
- **Relógio com load > 5 só por A×B intercalado ou a 1 fio** (`RAYON_NUM_THREADS=1`): a matriz do app a load 60–90 inventou uma regressão
  da aquarela, desmentida por A×B (`986aab640d`).
- **Resumo/cache por identidade `Weak` do `Arc` não fica velho:** `make_mut`/`get_mut` respeitam o `Weak`.
- **Uma fixture pode perder o fenómeno quando uma cura muda o que o pen-up relê** (`060b3fe257`: o pen-up só relê traço ⊕ pad). O
  controlo do oráculo («o velho reprova») tem de existir em cada gate de oráculo.
- Herdadas: perfil a 1 fio separa o custo do paralelismo; a cache de imagens do Vello é por id do `Blob`; listas de ficheiros para
  `git add/commit` vão em `bash -c '…'` (zsh não parte `$files`).
- **Um conserto que tira trabalho de um gesto pode só MUDÁ-LO de lugar** (o «+» → o 1.º toque): meça o gesto seguinte no app.
- **A placa: crescer o array de camadas não pode limpar o cache das residentes** (`ensure_array`, `116f9c8072`).
- **Antes de construir um salto/sono, meça se os candidatos mudariam o bit** (o B13.12 recusou o sono por tile assim, `8cbfed0328`).
- Vermelhos por decisão (não mexer): os do handoff n.º 2 §7, mais as 3 miniaturas do n.º 4 §7 enquanto o dono não decidir.
