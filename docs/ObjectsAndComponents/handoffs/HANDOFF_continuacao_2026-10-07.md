# Continuação — line/ObjectsAndComponents (07/10, fim da onda 0)

- **Branch** `line/ObjectsAndComponents` · base `2a925859d` · 3 commits, todos em `docs/ObjectsAndComponents/`
  (zero código, zero foundational).
- **Feito:** a DEFINIÇÃO — [`01_objeto_e_componente.md`](../01_objeto_e_componente.md) (estado medido,
  oráculo Godot 4.7.2 corrido, leis §4, tabela §5, Timeline §6, fases §8). **As quatro escolhas do dono
  estão respondidas (§9), todas na recomendação** — não re-litigar.
- **Instrumento:** `ferramentas/godot_taxonomia.sh` → `godot_4.7.2_taxonomia.txt` (283 nós · 416 recursos).

## Próxima onda (1): a Hierarchy nova — F0 + F1 + F2 + F3 num só bloco (CLAUDE.md §0.10)

As três fases mexem na MESMA superfície (a linha e a árvore do `ph2d-panel-hierarchy`) ⇒ um plano,
um lote, um gate.

- **F0** — vocabulário: «Prefab» em docs e mensagens onde hoje se lê «Componente» para o molde (o
  ecrã já diz Prefab: `asset.kind.component` ⇒ `"Prefab"`). Renomear a crate fica FORA.
- **F1** — objecto **Cena** na raiz. `PhysicsSettings` (`ProjectFile.physics`) e `SceneAudio` passam a
  componentes dele; migração lê o global antigo; `PROJECT_SCHEMA` sobe (contar com
  `python3 scripts/schema-recount.py`, nunca escolher). Antes: conferir se já existe raiz implícita
  (`RootOrder`/`MasterRoot`) e como o Undo (`ProjectState`) a vê.
- **F2** — ícones de componente na linha + clique leva ao cartão do Inspector (os 7 sítios da costura,
  DIRETIVA §2; teste `ph2d-ui-testkit`).
- **F3** — sub-linhas de conteúdo (Camadas da Imagem · Quadros do Flip · Ossos do Esqueleto) por
  ponto de extensão que a FAMÍLIA declara (como o `ObjectTypeDecl` da spec/06 §3.1).

Depois: onda 2 = **F4** (Animação por objecto, a maior); onda 3 = **F5** (Motion objecto) + **F6**
(busca por intenção + filtro).
