# Prompt de continuação — linha PainterWatercolor · 11/11 (b): duas regressões a 4K

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-11b_regressao_toque_e_undo_4k.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler; `git rebase main`.

## Contexto (ler só isto antes de agir)

- Handoff da missão: [HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-11_POS_MISSAO_4K.md](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-11_POS_MISSAO_4K.md) (o que NÃO reconstruir).
- Medição A×B que abre esta onda: [SEGUNDA_RODADA_AxB.md](../missao_4k/medicoes/SEGUNDA_RODADA_AxB.md) + roteiros em
  `docs/Painter/missao_4k/medicoes/segunda_rodada/` (`ab.sh`, `ab2.sh`, `agrega.py`: têm `SP`/`LW` fixos, ajuste antes de correr).
  Sonda do app: `docs/Painter/ferramentas/mede_pen_up_4k.sh` (Xvfb, Radeon, papel Textured + relevo do dono).
- A = `e81e56bb1f` (antes da 2ª rodada) · B = `57b4ddbac7` (fecho). Medido 11/11, n=5 intercalado, mínimo:
  traço, pós-soltar e mistura km (B32) e luz em faixas (C2/C3) **descartados** — iguais ou melhores.

## As duas regressões (só a 4K)

1. **Primeiro toque (pen-down) a 4K**: Impasto 11,7→20,8 ms (+9,1) · Filbert 70,9→76,8 (+5,9) · Wet 57,3→62,8 (+5,5).
   Bissecção parcial: `355869fd78^` = 11,7 · `9a7337ac68` = 21,0 (Impasto) ⇒ culpado nos 39 commits da cadeia
   B29.1 «tecto largo» (`stroke_cover/tecto.rs`, `TexelLargo` u16, pré-traço recomposto no carimbo). Sem chave env.
2. **Ctrl+Z do Filbert a 4K**: 54,5→61,8 ms (+7,3); 1080p igual. Nenhuma chave explica
   (`PH2D_COMPOSITE_LUZ_GESTO` simples×cheia 62,2×66,0; `PH2D_WET_MISTURA` nulo). Intervalo inteiro A..B por bissectar.

## Ordem de trabalho

1. **Um bloco** (CLAUDE.md §0.10): monte TODOS os binários da bissecção de uma vez — pontos médios de ambos os
   intervalos, cada um em worktree temporária com o PRÓPRIO `CARGO_TARGET_DIR`, `--profile smoke` — e meça-os numa
   única matriz intercalada (toque Impasto 4K + Ctrl+Z Filbert 4K). Repita até isolar o commit de cada regressão.
2. Com o commit em mãos: cure **sem mudar o pixel** (bit-paridade com o B atual nos alarmes de pincel e nos testes do
   item; os alarmes de pincel continuam vermelhos e NUNCA se regravam — só o dono aprova). Se a cura exigir trocar a
   lei (ex.: o pré-traço recomposto ser preciso no carimbo), meça a alternativa e deixe a decisão ao dono no handoff.
   Prováveis curas: trabalho do tecto largo feito por janela suja e não pela camada inteira no pen-down; cópia/
   conversão u16 de camada inteira no undo.
3. Remeça A×B×cura com `ab.sh` (mesma matriz da tabela da medição) e grave o resultado ao lado de `SEGUNDA_RODADA_AxB.md`.
4. Fecho da onda: testes do item por NOME, fmt/clippy das crates tocadas 1×, `the_shell_only_shrinks`/tectos de LOC,
   handoff curto + link no §5.1 do Painter, `rm -rf target/*/incremental`, remova worktrees temporárias.

## Regras desta onda

- ⛔ RTX proibida (`PH2D_GPU=1`); só a Radeon integrada. Pesado por `bash scripts/ph2d-run.sh`; nada de `pgrep -f` em laço.
- ⛔ Nada de push nem integração (só por ordem do Enio). Commits por paths ou índice temporário; nunca `git add -A`/stash.
- Delegue a bissecção a um agente (esforço medium); a janela principal decide e confere.
- Resposta ao dono: curta, PT-BR, sem jargão, smoke em passos numerados.
