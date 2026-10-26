# Handoff de continuação — `line/GameEngineFuncional`, plano 10 (arte de agachar) (25/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`. **Não integrada, não
empurrada.** Antecessor: [`HANDOFF_CONTINUACAO_2026-10-25_09_ABAIXAR.md`](HANDOFF_CONTINUACAO_2026-10-25_09_ABAIXAR.md).
Plano: [`10_arte_de_agachar_plano.md`](../10_arte_de_agachar_plano.md) (o §«Feito» tem os números medidos).

## Estado

**Smoke do dono: OK (25/10)** — «Muito bom! Perfeito.» Os 5 passos (as 6 regras no cartão; andar agachado
com o desenho baixo sob o tecto; parado agachado a respirar com os pés no chão; soltar ↓ lá debaixo; sair e
levantar) aprovados. Fotos: [`fotos_10/`](fotos_10/).

| sha | O quê |
|---|---|
| `5244f105e8` | arte + tags + pose `CrouchWalk` + regras derivadas com tags + gates |
| `ddf3bd4baf` | fotos do smoke e o medido no plano 10 |

## O que entrou

- **Referência Blender** (`ferramentas/boneco_blender.py`): parâmetros `neck` (cabeça desce no tronco) e
  `pe(x, alto)` (pé de apoio acima do chão), neutros = os de sempre ⇒ os 4 ciclos antigos byte a byte iguais;
  ciclos `crouch` (4 × 200 ms, repete) e `crouch_walk` (4 × 133 ms, repete). Blender PORTÁTIL 5.2.3 (o do
  sistema pede `libcudart.so.13`): estava em
  `/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/db96261a-…/scratchpad/blender/blender-5.2.3-linux-x64/blender`
  (pasta temporária — pode sumir; re-descarregar de `download.blender.org`, sha256 `78486c01…`).
- **Vetor** (`ferramentas/arte_svg.py`): `POSICOES` (ciclo → linha, coluna inicial); grupos novos no FIM do SVG.
- **Folha** (`ph2d-render/src/atlas/hero_sheet.rs`): `HERO_SHEET_TAGS` 4 → 6 — `crouch` 20–23, `crouch_walk`
  28–31, nas células antes VAZIAS. **Grelha 8 × 4, tamanho e chave iguais** (a grelha é gravada nos ficheiros;
  mudar linhas desalinharia os UV de todo ficheiro antigo).
- **Pose** `CharacterPose::CrouchWalk` (append-only, `ALL` = 6): agachado E a correr
  (`personagem.rs::vale`). `CROUCH_DERIVED = [CrouchWalk, Crouch]`, `conventional_tag()` → `"crouch"` /
  `"crouch_walk"`. Cartão `ANIM_POSES` 6, `ANIM_POSE_CROUCH_WALK` = 5 (só com o agachar armado), i18n
  `when_pose_5` «Crouch-walking», `POSE_KEYS` 6.
- **Regras derivadas com tags** (`ph2d-physics-ecs/src/components/anim_rules.rs`):
  `of_character_with(c, has_tag)`, `effective(rules, character, has_tag)` (assinatura MUDOU),
  `AnimRules::of_entity(world, e)` e `anim_target(world, e)` — a porta única do «quem anima» (o próprio ou o
  1.º filho com `SpriteAnimator`); `card.rs::rules_of` e `personagem.rs` usam-na (antes o `alvo` vivia só em
  `personagem.rs`). As do agachar entram no TOPO só se a tag existe ⇒ ficheiro antigo = as 4, bit a bit.
- **Smoke `=3`:** sem regra à mão (o Add já dá as 6); o condutor toca `crouch_walk` → `crouch` → `idle`.

## Superfície que um merge pode partir

- `AnimRules::effective` ganhou o 3.º argumento `has_tag` — todo chamador noutra linha parte a compilar
  (`|_| false` = o comportamento de antes).
- `CharacterPose` +1 variante (`CrouchWalk`): match exaustivo noutra linha parte.
- `ANIM_POSES` 5 → 6, `HERO_SHEET_TAGS` 4 → 6 (números que somam entre linhas: contar).
- `ph2d_physics_ecs::anim_target` (export novo).
- Sem campo gravado, sem componente, **sem degrau de `PROJECT_SCHEMA`**; shell `shells/desktop`: 0 linhas.

## Prova de fecho (DELTA)

Gates novos (verdes): render `as_celulas_antigas_sao_as_de_antes` (FNV-1a das 22 células antigas
`0x075f757e4fa1596a`, medido ANTES da arte); `as_tags_sao_as_linhas`/`cada_animacao_mexe`/
`a_sola_declarada_e_a_do_desenho` reescritos para tags fora da 1.ª coluna (sola de `crouch`/`crouch_walk` em
TODA célula); physics-ecs `as_do_agachar_so_entram_com_a_sua_tag`; app-game
`as_derivadas_com_o_agachar_tocam_as_celulas_dele`, `o_desenho_agachado_tem_o_topo_do_corpo` (física real,
±0,03 m, controlo `idle` contra o corpo de pé), `o_personagem_do_menu_deriva_as_seis_regras`,
`o_nivel_3_agacha_sob_o_tecto_e_a_regra_toca` (agora `crouch_walk`/`crouch`/`idle`); seam
`a_andar_agachado_so_se_oferece_com_o_agachar_armado`; app-sprite-anim `the_rule_mirrors_are_the_physics`
(o comentário de `ANIM_POSES` citava-o e ele NÃO existia).

**Mutação 6/6 sangra.** Por nome verdes: render `atlas::` (45), physics-ecs anim_rules/character_anim,
app-game personagem (10)/smoke (12)/object_add/arte, panel-inspector seam_anim, app-sprite-anim card,
editor-core inspector_model_anim / `architecture_workspace_file_loc_cap` / `the_shell_only_shrinks`, i18n
`a_tabela`, panel-registry-init `o_inspector_armado`. `cargo fmt --all` limpo; clippy `-D warnings
--all-targets` das 7 crates tocadas limpo. **Suíte de impacto NÃO corrida** (do integrador). Smoke build, 2.ª
corrida: `Finished smoke profile [optimized] target(s) in 0.34s`, zero `Compiling`.

## ABERTO

- Herdado: `AnimRules` sem nome de script nem lei em `property_laws::LEIS` · barra de largada das regras sem
  seam · «This frame» `1.330x` vs `1.33x` · R1 vista dupla ao vivo · View ▸ Focus mode morto ·
  `ph2d.disconnect` · a faixa do editor não rola ao arrastar junto à borda · corpo sem troço recto (bola) a
  agachar abaixo do piso pode encostar e travar (sem gate) · suíte de impacto (integrador).
- **Novo (menor):** o desenho é mais largo que a cápsula (±0,27 de pé, ±0,30 agachado, raio 0,2) — encostado à
  parede o capacete pinta uns cm por cima dela.
- Regra derivada por NOME de convenção: renomear a tag `crouch` tira a regra derivada (a 1.ª edição do
  cartão grava as regras e daí o rename leva-as).

## PRÓXIMO

**J4 do [plano 03](../03_plano_jogo_de_plataforma.md):** a câmara segue o jogador presa aos limites do nível, e
o fundo em parallax. Medir primeiro o que já existe (`GameCamera`, componentes de câmara de
`ph2d-app-components`, parallax?) antes de desenhar; plano `11_*.md` antes de codar.
