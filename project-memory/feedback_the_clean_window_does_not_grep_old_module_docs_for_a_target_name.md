---
name: feedback-the-clean-window-does-not-grep-old-module-docs-for-a-target-name
description: "A janela I de um clean-room não procura o nome de uma ferramenta do alvo nos docs antigos do módulo — o censo de citações cobre o CÓDIGO e os docs/** ainda carregam nomes internos (INC-I1, 16/09)"
metadata:
  type: feedback
---

Ao preparar a missão-E do pincel «Draw Sharp» (`line/sculpt3d`, 2026-09-16), a janela I correu um
`grep -B2 -A8` pelo nome do pincel em `docs/3D/20_divergencias_tools.md`,
`docs/3D/21_plano_modos_e_ferramentas.md` e `docs/3D/04-Ferramentas/04.1-Pinceis.md`. O contexto do
grep trouxe nomes internos do alvo e dois fragmentos de comentário: são notas anteriores à limpeza
de citações. Um R classificou como RELANCE (INC-I1, commit `b7bdb3e48`), e a comparação passou a
ser item obrigatório do R-pós. A espec do E ainda mandava a janela I reconferir linhas desse plano,
e o R-pré apanhou isso.

**Why:** o gate `architecture_no_restricted_source_citations` zerou a dívida **no código**, e os
`docs/**` ficam fora do censo por construção (CLAUDE.md §5, 3D). Um doc do módulo PARECE seguro
porque é nosso, e é exactamente aí que a expressão do alvo sobrevive.

**How to apply:**
- Como janela I, o que se sabe de uma ferramenta do alvo vem da **espec atestada**. Não faça grep
  pelo nome dela em `docs/<Módulo>/` fora de `cleanroom/SPEC_*`. Para saber o que a casa já decidiu,
  peça ao E que o leia e o resuma na espec.
- Uma espec que manda a janela I abrir um doc antigo é um bloqueio de parede: ver
  [[feedback_an_instruction_doc_can_order_what_its_readers_own_fence_forbids]].
- Aconteceu? Faça append cego ao `INBOX_<alvo>.md`, com endereços e sem reproduzir o texto, e peça
  a um R a classificação §6.2. Não continue a construir antes do veredito.
