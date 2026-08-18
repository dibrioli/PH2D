---
description: Briefing inicial de uma worktree nova (DIRETRIZ §1.5.8).
argument-hint: [Nome da linha] [Objetivo] [Docs extras a ler]
---
Abra a linha `$1` na worktree dela.

ANTES de ler qualquer arquivo: `cd` na worktree, `pwd`, `git branch --show-current`.
A janela abre na raiz (= main) e o mesmo path relativo existe nas 2 árvores — editar a
errada compila e commita sem erro.

Objetivo da linha:
$2

Leia primeiro: docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md + DIRETRIZ §1.5$3

Regras da linha:
- Foundational você PODE tocar (ADR-0107), projetando para isolamento (módulo irmão /
  ponto de extensão append-only). Contrato congelado (§6) e rebase conflitando fora dos
  seus arquivos: PARE e reporte.
- Inner loop = SÓ `cargo check -p`. Teste/clippy/auditoria 1× no fechamento.
- Você fecha a linha, escreve o handoff (§1.5.9) e PARA. Não integra, não pusha.

Antes de codar: me diga o plano em waves, com o que cada uma entrega e como se prova.
