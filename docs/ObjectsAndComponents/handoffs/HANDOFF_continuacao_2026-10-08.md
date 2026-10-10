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

## Onda 3 fechada (08/10)

- F5 (o Motion como objecto) e F6 (busca por intenção + filtro da Hierarquia) estão fechadas:
  [handoff da onda 3](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda3_2026-10-08.md).
- As **TRÊS ondas vão juntas** na branch `line/ObjectsAndComponents` (nenhuma está no `main`).
- A próxima onda é **decisão do dono** — ver §10 do handoff novo (abertos a–g).

## (Histórico) Onda 3 como estava planeada: F5 + F6

- **F5 — Motion como objecto-gerador** (doc 01 §8, escolha 9.3 do dono: vira objecto na árvore, com
  o que cria como filhos; pode haver vários). ⚠️ O `ph2d-app-motion` é da `line/motion-value` (viva:
  `git worktree list`) — medir a superfície de colisão ANTES (`collision-surface.sh`).
- **F6 — busca por intenção** no *Add Component* + **filtro da Hierarquia por componente**.
- Antes de construir: MEÇA se a composição já exprime o item (CLAUDE.md §5.0) e confira o código.
- `PROJECT_SCHEMA` = 187 hoje; se a onda 3 gravar algo novo, recontar com `python3 scripts/schema-recount.py`.

## Onda 4 (09/10) — fechar o que ficou aberto, ANTES de integrar

Ordem do dono: *«antes de integrar vamos fazer tudo o que está em aberto»*. A lista (10 itens) e as três
escolhas dele estão no [doc 01 §14](../01_objeto_e_componente.md). As ondas 1–3 continuam fora do `main`
(smoke das três aprovado; onda 3 em `497bae316`); a onda 4 segue POR CIMA, na mesma branch, e o integrador
leva as quatro juntas. ⚠️ Os itens 6–8 tocam `ph2d-app-motion` — `line/motion-value` viva: colisão primeiro
(`collision-surface.sh` nas duas worktrees). O item 1 sobe o `DOC_VERSION` da timeline (19 → 20, com
migração provada contra fixtura gravada ANTES).

## Onda 4 fechada (09/10)

- Os 10 itens do [doc 01 §14](../01_objeto_e_componente.md) estão fechados:
  [handoff da onda 4](HANDOFF_INTEGRACAO_line_ObjectsAndComponents_onda4_2026-10-09.md).
- As **QUATRO ondas vão juntas** na branch `line/ObjectsAndComponents`; nenhuma está no `main`.
- O que fica aberto: §10 do handoff novo (a–g). **Próxima onda = decisão do dono.**
