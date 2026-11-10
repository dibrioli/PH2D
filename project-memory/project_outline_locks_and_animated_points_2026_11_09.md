---
name: project_outline_locks_and_animated_points_2026_11_09
description: "Dono 09/11 (smoke GEF02): Detail/Alpha/Specks travados num Outline editado à mão fora do Edit shape on canvas; pontos animados + AutoKey off = modal para apagar a trilha; forma filha de forma fica"
metadata:
  node_type: memory
  type: project
  originSessionId: e999243b-e316-4bdd-8699-49b279ca29fe
  modified: 2026-11-09T18:56:07.801Z
---

Decisões do dono no smoke de 09/11 (`line/GameEngineFuncional02`, Outline):

- **Detail, Alpha cutoff, Ignore specks under**: livres enquanto o Outline segue a imagem. Depois da 1.ª edição de
  pontos à mão (`custom`), TRAVADOS fora do «Edit shape on canvas»; dentro destravam e mudar um deles RETRAÇA da
  imagem (troca os pontos à mão, um Ctrl+Z). Aparência travada = só inativo, nunca escondido. Substitui a regra de
  `13902fc1ec` («trava enquanto edita»), que o dono leu ao contrário no smoke.
- **Pontos com animação gravada**: AutoKey LIGADO ⇒ mexer grava nova chave. AutoKey DESLIGADO ⇒ o gesto não muda
  nada e abre um modal que pede para apagar a trilha dos pontos; aceitar apaga e a edição volta.
  Com trilha, mudar o NÚMERO de pontos (criar, apagar, Alt+clique, fundir, Merge, retraço) bloqueia com o mesmo modal
  MESMO com AutoKey ligado (dono 09/11); sem trilha tudo livre.
- **Add ▸ Collision Shape com uma forma selecionada** cria a FILHA da forma (como hoje); o arrasto dela na
  Hierarchy tem de funcionar (bug visto 09/11).
- «Remove Shape» fora também da Loose Shape — apaga-se pela Hierarchy (smoke OK 09/11).

**Why:** o dono quer que um ajuste à mão nunca se perca por acidente: só se refaz o contorno onde os pontos se veem,
e a animação só se apaga com permissão explícita.
**How to apply:** qualquer controlo novo que retrace o Outline segue a mesma trava; qualquer gesto novo de pontos
passa pelo mesmo modal. Plano: `docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md`. Lei geral:
[[project_autokey_records_everything_2026_10_19]].
