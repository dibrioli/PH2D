# CONTINUAÇÃO — `line/MiroClone` W9 «Fidelidade ao Excalidraw» (briefing, 2026-10-10)

> Para a janela que ASSUME a linha (`/pd-linha-assumir`, rota «linha reaberta» do
> [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)):
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. W5+W5b+W6+W7+W8 continuam por integrar na mesma branch (integra-se tudo junto, por ordem
> do dono); a W9 vai por cima.

## 1. Onde a linha está

- **W8 fechada** — [handoff de integração](HANDOFF_INTEGRACAO_line_MiroClone_W8_2026-10-10.md); smoke do dono
  OK (10/10, §12). Depois do smoke, três correcções pedidas pelo dono, feitas e com teste (`346f3a18b`):
  arrastar a bolha de comentário (re-prende ao objecto sob a ponta), «Resolve thread» por extenso no fundo do
  cartão, e resolver/mover no `Ctrl+Z` do quadro (`TeamOp` no `History`). ⏳ **O dono ainda não as viu** —
  o 1.º passo da W9 é perguntar-lhe pelo smoke delas (comando no §12 do handoff W8 / cena 10).
- **O estudo da W9 está FEITO**: [`../07_fidelidade_excalidraw.md`](../07_fidelidade_excalidraw.md)
  (`ac95616c2`) — Excalidraw 0.18.1 e o `next` `0.18.0-143b5b6` corridos como ORÁCULO sobre 70 casos nossos
  e o ficheiro do dono (`ferramentas/excalidraw_oracle/entradas/w9_dono_bv.excalidraw`), 220 saídas, duas
  corridas iguais. ⛔ Não refazer o estudo; ler §2 (as leis XA/XB/XV com número), §3 (recusas), §4 (a tabela
  do formato inteiro), §5 (M1–M10), §6 (o plano, os testes vermelhos, a cena 11).

## 2. A decisão do dono (10/10)

*«like Excalidraw»* — **o quadro INTEIRO adopta o vão**: as setas que o artista DESENHA também param
`5 + sw/2` antes da forma (não só as importadas). Registado no estudo §5 M1 e §6.1 (`ph2d-board-edit`).

## 3. O que fazer (sem voltar a perguntar)

Na ordem do estudo §5, num bloco (CLAUDE.md §0.10):

1. **M3** a «redonda» de 2 pontos é recta; 3+ pontos ⇒ `Route::Spline` (Catmull-Rom 1/6, XA1).
2. **M1/M6/M4** `Anchor::Orbit` + o vão (XB1) no roteador, no leitor, no escritor (as DUAS gramáticas de
   ligação, XV3) e no gesto de ligar do quadro (a decisão do §2).
3. **M2** a ponta que encolhe `min(base, último trecho/2)` em TODO o quadro (hoje uma seta ≤ 20 px fica sem
   ponta — defeito).
4. **M5, M7–M10** `stickynote`, `fillStyle`/`roughness 2`/semente, alinhamento e família do texto, pontas de
   cardinalidade e triângulo vazado, caneta de largura constante — e o resto da tabela §4 que tenha cura.
5. **Cena 11** (`PH2D_BOARD_SMOKE=11`, estudo §6.3): o ficheiro do dono ao lado da foto do Excalidraw, setas
   curtas a mostrar a ponta a encolher; fotografada antes de mandar.

Protocolo: cada lei com o teste VERMELHO visto antes da cura (estudo §6.2, os oráculos leem as fixtures
`saidas/w9_*` — nada a re-correr); formato do quadro sobe (9) com leitor do 8 e fixture GRAVADA pelo build
antigo; fechar com `/pd-linha-fechar` (portão batched 1×, auditoria de 2 lentes, prova de mutação, cena
fotografada, handoff de integração W9, CLAUDE.md §5.1). Não integra, não pusha.

## 4. Lembretes medidos nesta linha

- O `cargo-test-narrow.sh` contou um erro de compilação como «0 testes» e o `cargo-check-narrow.sh` esconde
  avisos: o `cargo check --all-targets` com `CARGO_BUILD_WARNINGS=deny` à parte.
- Os testes de uma cena correm a função DO SMOKE (`scene_team`), nunca uma cópia (a cópia escondeu um
  sobrevivente da mutação na W8).
- Um gate que compara ficheiros escritos byte a byte entre perfis de compilação lê o último dígito de uma
  conta como «o escritor mudou» — compare a árvore com tolerância de arredondamento.
- ⛔ Falar com o dono SEMPRE em PT-BR (também depois de uma compactação do contexto).
