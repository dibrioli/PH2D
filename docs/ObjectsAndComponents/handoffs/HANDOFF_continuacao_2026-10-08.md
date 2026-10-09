# Continuação — line/ObjectsAndComponents (08/10, fim da onda 2)

- **Ondas 1 e 2 fechadas** (a Hierarquia nova; a Animação por objecto): handoffs
  [onda 1](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_2026-10-08.md) e
  [onda 2](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda2_2026-10-08.md); o que entrou e as
  premissas que caíram: [doc 01 §11 e §12](../01_objeto_e_componente.md). **Smoke das duas aprovado
  pelo dono em 08/10** (a onda 2 inclui a cópia de Prefab com clips próprios, pedida no smoke).
- ⚠️ Nenhuma das duas está no `main` (`git cherry main HEAD`): a onda 3 segue na MESMA branch, por
  cima — o integrador leva as três juntas.
- Abertos da onda 2 (nome «andar 2»; undo em dois passos; a animação própria não toca sozinha no
  Play): handoff da onda 2, §10.

## Próxima onda (3): F5 + F6

- **F5 — Motion como objecto-gerador** (doc 01 §8, escolha 9.3 do dono: vira objecto na árvore, com
  o que cria como filhos; pode haver vários). ⚠️ O `ph2d-app-motion` é da `line/motion-value` (viva:
  `git worktree list`) — medir a superfície de colisão ANTES (`collision-surface.sh`).
- **F6 — busca por intenção** no *Add Component* + **filtro da Hierarquia por componente**.
- Antes de construir: MEÇA se a composição já exprime o item (CLAUDE.md §5.0) e confira o código.
- `PROJECT_SCHEMA` = 187 hoje; se a onda 3 gravar algo novo, recontar com `python3 scripts/schema-recount.py`.
