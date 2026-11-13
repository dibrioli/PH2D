# Prompt de continuação — linha PainterWatercolor · 12/11 n.º 6: o Painter plenamente otimizado (o que sobra)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-12_desempenho_6.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. ⛔ **NÃO faça `git rebase main`:** a linha JÁ foi rebaseada em 12/11 (pelo comando de
assumir) e está sobre o `main` `4f46a3573c`. Cópia de segurança da cabeça de antes: ramo `backup/PainterWatercolor-antes-rebase-2026-11-12`
(`8246bd897a`). Quem funde é o integrador que o dono nomear.

## Ordem do dono (verbatim, continua)

12/11: «vamos seguir até ter o app plenamente otimizado». «Se esforce para resolver todos os problemas de performance do módulo a todo
custo. … Não esqueça do handoff de integração ao final de tudo. Não integre.»

## Leia primeiro

1. [HANDOFF_CONTINUACAO_2026-11-12_ONDA5_VIDRO_E_MEDIA.md](HANDOFF_CONTINUACAO_2026-11-12_ONDA5_VIDRO_E_MEDIA.md) (n.º 5: o rebase, o vidro do WB,
   a sim pela média, as recusas medidas, o §5 aberto por ordem).
2. [HANDOFF_INTEGRACAO_…_SELECAO_CAMADAS_E_RITMO.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_SELECAO_CAMADAS_E_RITMO.md) (n.º 4, §12) e os
   três anteriores (`…_MISTURA_E_ENVIO.md`, `…_LUZ_E_CAMADAS.md`, `…_DESEMPENHO_TOTAL.md`), que continuam a valer.
3. `../missao_4k/BACKLOG.md` (B18.4 «Cont. 6», B13.12) e `BACKLOG_AUDITORIA.md`; `../missao_4k/medicoes/DEPOIS_N4.md` (a matriz do app, com o aviso de carga).

## Estado (depois da onda n.º 5)

| item | estado |
|---|---|
| 3 mistura, 4 água do Wet, 5 camadas e seleção, 6 cadência · B20.11b · B13.12 curas 1b/3/4 · B18.5 | FEITOS (n.º 4) |
| WB: cópia do vidro por quadro (58 MB → 0; tique 7,4–8,3 → 2,6–3,5 ms a 4K) e vidro que nasce zerado (1.º toque camada nova 206 → 91 MB) | FEITOS (n.º 5) |
| sim a 6 traços | **0,77 sim-s/s pela MÉDIA** (4 traços 1,00); curas ao bit esgotadas (3 recusadas medidas) — o resto é física, decisão do dono |
| WB 1.º toque | ainda ~38–43 ms; COW canvas+vidro por traço pede undo em janela (ADR) |
| alarmes de pincel | 77 vermelhos, lista idêntica com e sem a cura; origem provável: o rebase (não verificado) |

## O que fazer, por ordem (até ao fim, sem parar)

1. **Wet:** o 3.º evento do 1.º traço na camada nova (19,9 ms · 91 MB) e o «+» de 67 ms.
2. **WB 1.º toque:** reutilizar os buffers de tela inteira por sessão (coverage/color/water/backdrop), ao bit; o COW por traço do canvas+vidro =
   undo em janela/tiles (mudança de desenho: ADR antes, e diga ao dono).
3. **Re-medir a matriz do app com calma** (load < 5): `bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel` (e `NOVA=1`).
4. **Os 77 alarmes:** corra os dois por nome na cópia de segurança (`backup/PainterWatercolor-antes-rebase-2026-11-12`) para saber se vieram do rebase.
5. **O resto:** ruído do B19.10, B20.6 a parte da placa, B23.2; a espera ativa de ~10 % do rayon no perf da sim.
6. **Sim a 6 traços pela média:** só com decisão do dono (muda a aritmética da `assenta`) — pergunte-lhe em palavras simples, não construa.
7. Por item: medir antes (A×B, mínimo; tempo real = MÉDIA), curar AO BIT, gate por nome com mutação, medir depois, comitar com a tabela. **Fecho:**
   fmt (só os ficheiros da linha) / clippy `PH2D_SUITE=fecho`, alarmes de pincel por nome, matriz final, handoff NOVO + link no CLAUDE.md §5.1,
   `rm -rf target/*/incremental`, binário smoke 2×. Não integre, não envie.

## Regras aprendidas

Novas (12/11 n.º 5):
- **A régua do tempo real é a MÉDIA, nunca a mediana:** o passo lento não se recupera.
- **Um `Weak` como chave de identidade morre quando alguém faz `make_mut` com ele vivo** (o valor muda de `Arc`): quem move tem de religar.
- **Alocação ≥ 32 MB é `mmap`:** libertada e pedida por quadro = faltas de página em todo quadro; a 1080p o glibc esconde-o.
- **`perf -e page-faults -c 1` acha a cópia mas sem `-g` não diz quem:** instrumente por fases com `/proc/self/stat`.
- **`cargo fmt --all` nesta árvore formata ficheiros de outras linhas vindos do main:** reverta-os, formate só os da linha.

De 12/11 n.º 4:
- **O subagente não corre `--ignored` nem a fila pesada:** as sondas pesadas são do coordenador.
- **Relógio com load > 5 só por A×B intercalado ou a 1 fio** (`RAYON_NUM_THREADS=1`): a matriz do app a load 60–90 inventou uma regressão
  da aquarela, desmentida por A×B (`986aab640d`).
- **Resumo/cache por identidade `Weak` do `Arc` não fica velho:** `make_mut`/`get_mut` respeitam o `Weak` (mas veja a regra nova: ao MOVER, o `Weak` morre).
- **Uma fixture pode perder o fenómeno quando uma cura muda o que o pen-up relê** (`060b3fe257`). O controlo do oráculo («o velho reprova») tem de
  existir em cada gate de oráculo.
- Herdadas: perfil a 1 fio separa o custo do paralelismo; a cache de imagens do Vello é por id do `Blob`; listas de ficheiros para
  `git add/commit` vão em `bash -c '…'` (zsh não parte `$files`).
- **Um conserto que tira trabalho de um gesto pode só MUDÁ-LO de lugar** (o «+» → o 1.º toque): meça o gesto seguinte no app.
- **A placa: crescer o array de camadas não pode limpar o cache das residentes** (`ensure_array`, `116f9c8072`).
- **Antes de construir um salto/sono, meça se os candidatos mudariam o bit** (o B13.12 recusou o sono por tile assim, `8cbfed0328`).
- Vermelhos por decisão (não mexer): os do handoff n.º 2 §7, as 3 miniaturas do n.º 4 §7 e os 77 do n.º 5 §4 enquanto o dono não decidir.
