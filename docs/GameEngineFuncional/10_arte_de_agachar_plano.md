# 10 — A arte de agachar do herói de fábrica (plano, 25/10)

Pedido: o desenho do herói encolhe com o corpo agachado (plano 09 deixou o corpo baixo e a cabeça desenhada
~27 px dentro da laje). Células «crouch» (parado agachado) e «crouch_walk» (a andar agachado) na folha, com os
PÉS na linha das outras células e o topo a casar com o corpo agachado; personagens novos tocam-nas sozinhos.
Antecessor: [09](09_abaixar_plano.md) · [handoff 09](handoffs/HANDOFF_CONTINUACAO_2026-10-25_09_ABAIXAR.md) ·
arte: [06](06_arte_vetorial_plano.md).

## Medido (antes de desenhar)

- **A cadeia é determinística:** o Blender portátil 5.2.3 refaz `boneco_poses.json` byte a byte, e o
  `arte_svg.py` refaz os 6 SVG de `ph2d-render/src/atlas/fabrica/` byte a byte (25/10, sem mudar nada).
- **A grelha 8 × 4 é GRAVADA no ficheiro** (`SpriteGrid { hframes: 8, vframes: 4 }`, posto pelo Add) e as tags
  COPIAM-SE para o objecto ao nascer (`object_add::anima`). ⇒ mudar o número de linhas da folha desalinharia os
  UV de todo ficheiro antigo. **Há 10 células livres** sem mudar a grelha: linha 0 col. 6–7, linhas 2 e 3
  col. 4–7 (`jump` e `fall` só usam 4).
- **Geometria (1 = 1 m = a célula):** sola 0,45 abaixo do centro; topo do capacete de pé
  `anca 0,242 + sz·(0,27 + 0,03 + 0,27) = 0,81`; corpo de pé 0,80, **agachado 0,50** (`AGACHADO` = 0,15,
  a cápsula encolhe o troço recto 0,30 por cima, os pés não mexem). O capacete sozinho tem 0,54 de altura ⇒ o
  agachar TEM de achatar (squash ~0,8) — é o *squash & stretch* do plano 06.
- **Regras derivadas** (`AnimRules::of_character`) só leem o `CharacterAnimations` (4 nomes, sem agachar) e
  NÃO conhecem as tags; `effective` corre no cartão (`card.rs::rules_of`) e no jogo (`personagem.rs`).
- Sem condição E nas regras: «a andar agachado» não se exprime com as poses de hoje.

## Desenho

1. **Referência Blender** (`boneco_blender.py`): dois parâmetros novos de pose com o neutro de sempre —
   `neck` (a cabeça desce no tronco; 0 = igual) e o pé de apoio com altura (`pe(x, alto)`; 0 = igual) — e dois
   ciclos: `crouch` (respira, 4 desenhos × 200 ms, repete) e `crouch_walk` (dois passos arrastados, 4 desenhos
   × 133 ms, repete). Os ciclos antigos ficam com os mesmos números.
2. **Vetor** (`arte_svg.py`): cada ciclo tem a sua (linha, coluna inicial): `crouch` → linha 2 col. 4,
   `crouch_walk` → linha 3 col. 4. Os grupos novos vão no FIM do SVG.
3. **Folha** (`hero_sheet.rs`): `HERO_SHEET_TAGS` 4 → 6 (`crouch` 20–23, `crouch_walk` 28–31); grelha,
   tamanho e chave iguais.
4. **Pose `CrouchWalk`** (append-only, `CharacterPose::ALL` = 6): agachado E a correr. `DERIVED` continua as 4.
   Cartão `ANIM_POSES` 6, `ANIM_POSE_CROUCH_WALK` = 5, só com o agachar armado; i18n «Crouch-walking».
5. **Regras derivadas com as tags** (`AnimRules::of_character_with(c, has_tag)`, `effective(…, has_tag)`):
   no TOPO, `CrouchWalk → crouch_walk` e `Crouch → crouch`, cada uma SÓ se a tag com esse nome existe no
   objecto que anima (o próprio ou o filho com animador, como o `personagem::alvo`). Ficheiro antigo (4 tags)
   ⇒ as 4 de sempre, bit a bit. Nomes por convenção (`CROUCH_TAG`, `CROUCH_WALK_TAG`), como o `Default` do
   `CharacterAnimations`; renomear a tag tira a regra derivada (a 1.ª edição grava as regras e o rename
   passa a levá-las).
6. **Smoke `=3`:** sem a regra à mão — o Add já dá as 6; o condutor agacha sob o tecto e toca `crouch_walk`
   e `crouch`.

Sem campo gravado novo ⇒ **sem degrau de `PROJECT_SCHEMA`**; sem componente novo.

## Gates

- Folha: pés de `crouch`/`crouch_walk` na sola declarada (texel, ±1,5) em TODA célula; margem transparente;
  cada animação mexe; tempos = os do Blender; a grelha proposta continua 8 × 4.
- **Ficheiro antigo igual:** os texels das 22 células antigas têm o hash medido ANTES desta onda; o
  `boneco.svg` começa pelos grupos antigos sem mudança.
- **Topo agachado ≈ corpo** (app-game, onde a cápsula vive): a linha mais alta das células de agachar fica a
  ±0,03 m do topo do corpo agachado; CONTROLO na mesma régua: o `idle` fica a ±0,03 m do topo de pé.
- Regras: sem as tags ⇒ as 4 (bit); com as tags ⇒ as 6 por esta ordem; o Add dá as 6; agachado parado toca
  `crouch`, a andar `crouch_walk`, levantar devolve.
- Cartão/seam: «Crouch-walking» só com o agachar armado; escolhê-lo grava `Pose(CrouchWalk)`.

## Fora desta onda

Agachar no ar · rastejar · transição animada de levantar/baixar.
