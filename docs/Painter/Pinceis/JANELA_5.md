# JANELA 5 de 5 — Fechar a missão: o resto da fila, a biblioteca inteira, o fecho e o smoke do dono

> Leia ANTES: `00_MISSAO.md`, `ESTADO.md`, `HANDOFF_JANELA_4.md`, `01_CATALOGO.md`. Nasce em espera: responda
> «JANELA 5 pronta — aguardo o sinal» e pare até o SINAL. ⚠️ Esta janela NÃO passa a vez: ela termina a missão.

## Tarefa

1. **O resto da fila** do `ESTADO.md` (pelo método da JANELA 3: família por agente, o EXAME, nada de «aberto») até o
   catálogo estar cumprido (§6 do contrato: 80–120 pincéis aprovados). O que for impossível (oráculo inexistente,
   motor desproporcional) sai do catálogo com o porquê MEDIDO — não fica pendurado.
2. **A biblioteca como um todo**: a matriz de distinção sobre TODOS os presets (corte/funda os que se confundem); a
   organização por categoria como os grandes apps a fazem (pastas, nomes, rótulos i18n, miniaturas legíveis); a ordem
   no navegador; o desempenho de cada um a 60 Hz no tamanho grande; a memória (nenhum pincel perto do tecto); a
   revisão cega final por categoria (um agente por categoria, pranchas sem rótulo).
3. **O fecho da linha** (DIRETRIZ §1.5.9, `/pd-linha-fechar`): o gate batched 1× sobre o diff acumulado
   (`PH2D_SUITE=fecho` nextest-impacted + clippy `--all-targets` + fmt, pelo `verificador`), as mutações das leis
   novas, o handoff de integração `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_<data>_PINCEIS.md`
   (≤ 8 KB) com a superfície de colisão e os números somados a recontar, a linha do Painter no CLAUDE.md §5 (≤ 700
   bytes), `agent-loop-profile.sh`, `rm -rf target/*/incremental`.
4. **O smoke do dono**: o binário `--profile smoke` compilado na worktree (2.ª compilação colada no handoff); um
   TUTORIAL em PDF (`docs/Painter/Pinceis/tutorial/`, como os tutoriais dos Componentes) em passos numerados, sem
   jargão: o comando com o `cd`, onde clicar, uma página por categoria com a foto dos pincéis dela (oráculo ao lado),
   o que cada um faz e como saber que algo deu errado (CLAUDE.md §0.8).
5. **O relatório final ao dono**, curto e sem jargão: quantos pincéis, por categoria, o que ficou de fora e porquê, o
   comando do smoke. Integração e envio: SÓ quando ele mandar.

## Saída
A missão cumprida (§6 do contrato), `ESTADO.md` final («MISSÃO CUMPRIDA» + SHAs), a árvore limpa, nada nosso a correr.
