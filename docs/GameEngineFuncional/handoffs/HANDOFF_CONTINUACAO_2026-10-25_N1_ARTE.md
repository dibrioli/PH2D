# Handoff de continuação — `line/GameEngineFuncional`, N1 + amostra de arte (25/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`; HEAD antes deste doc
`60a533e2ee`. **Não integrada, não empurrada.** Antecessor:
[`HANDOFF_CONTINUACAO_2026-10-25_R0_N0.md`](HANDOFF_CONTINUACAO_2026-10-25_R0_N0.md). Planos:
[`05_N1_plano.md`](../05_N1_plano.md) · [`06_arte_vetorial_plano.md`](../06_arte_vetorial_plano.md) (§4 «A amostra — feita») ·
**próximo item:** [`07_animacao_num_cartao_plano.md`](../07_animacao_num_cartao_plano.md).

## Estado

- **N1** (plano 05): menu Add ▸ Game (Sprite, Animated Sprite, Static Body, Character, Collision Shape) com
  dicas ilustradas; o nível 2 (`PH2D_GAME_SMOKE=2`) só se monta pelo menu; Auto Animate = componente
  `CharacterAnimations`. **Smoke do dono: OK (25/10)** — A (menu Add + dicas), B (Character + Static Body à
  mão, cartão Auto Animate, Play in a Window), C (nível 2).
- **Amostra de arte** (plano 06 §4), commit `fdb2e321eb`: Kenney Pixel Platformer (CC0) em vetor plano sobre a
  referência do Blender. **Smoke do dono: «ficou legal»**, com 1 relato — depois de um salto a animação de
  correr deixava de tocar. Cura em `60a533e2ee` (`personagem.rs`, `fn troca` põe `playing=true`; gate
  `depois_do_salto_que_acabou_a_corrida_toca`, vermelho antes). O dono confirmou depois: **Smoke OK**.

## Commits da onda

| sha | O quê |
|---|---|
| `55878c0922` | N1 — animações do personagem no Inspector (catálogo, cartão no Platform Player, censos) |
| `01fd0e339e` | N1 fecho da onda — fmt, clippy, rótulo «Auto Animate», o nível 2 abre no personagem |
| `917b5f3868` | plano 06 — arte vetorial plana (decisões do dono 25/10) |
| `fdb2e321eb` | a amostra: boneco, chão, moeda, bandeira, nível 2 |
| `60a533e2ee` | fix — depois do salto a corrida volta a tocar |

## Superfície que um merge pode partir (colisão)

- **Chaves de atlas RESERVADAS novas** em `crates/ph2d-render/src/atlas/fabrica.rs` (`FACTORY_ART`):
  `u32::MAX-2` (herói) … `u32::MAX-7` (GROUND `-3`, PLATFORM `-4`, DIRT `-5`, COIN `-6`, FLAG `-7`).
  **Integrador: confira colisão com chaves de outras linhas** (`git grep "u32::MAX -"`).
- `ph2d-render` ganhou a dependência `ph2d-imageio-svg` (já no grafo release via `ph2d-asset`);
  `ph2d-app-game` ganhou dev-dep `ph2d-vec-svg`. Conflito provável em `Cargo.toml`/`Cargo.lock`.
- `sprite.wgsl`: `sample_tiled` (gradiente da UV desembrulhada para sprites com wrap). Gate de placa
  `um_azulejo_repetido_nao_tem_costura` (vermelho antes da cura).
- Folha do herói 8×4 células de 192 px, SOLE 0,45; tags: idle 6×167 ms, run 8×67, jump 100/133/167/133 uma vez,
  fall 4×100. Cápsula 0,15/0,2. Azulejos em 3 cópias (terra 3×3), o sprite pinta a célula do meio.
- Nível 2: blocos azulejados, 5 moedas (Animated Sprite + `moeda.luau`), bandeira, céu `#C2E3E8`
  (`ZIndexOverride -1`), câmara altura 8; «Friend» removido.

## Como regenerar a arte (Blender PORTÁTIL 5.2.3)

`/usr/bin/blender` pede `libcudart.so.13` (cuda não instalado) — use o portátil:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional
PH2D_GPU=1 bash scripts/ph2d-run.sh <blender-portátil>/blender -b --factory-startup -P docs/GameEngineFuncional/ferramentas/boneco_blender.py -- <saída>
python3 -I docs/GameEngineFuncional/ferramentas/arte_svg.py docs/GameEngineFuncional/ferramentas/boneco_poses.json crates/ph2d-render/src/atlas/fabrica
```

## Prova de fecho (DELTA)

fmt + `clippy -p ph2d-render -p ph2d-app-game --all-targets` limpos. Testes por nome, verdes: render
`atlas::hero_sheet` 7 · `atlas::fabrica` 4 · GPU `tiling_seam` 1 · `sprite_wgsl` 3; app-game `o_nivel_2` 2 ·
`o_personagem` 1 · `a_andar` 1 · `a_arte_de_fabrica` 1 · `object_add` 4 · `personagem` 4.
**`nextest-impacted` foi PARADO** (load 129; o conjunto impactado ≈ repo inteiro porque `ph2d-render`
mudou; regra do dono: suíte completa só antes da integração) — **o integrador TEM de a correr**.
Smoke (`--profile smoke`): 2.ª corrida 0,24 s, zero `Compiling`.

## Smokes

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=2 cargo run --profile smoke -p ph2d-host-desktop
```

(pacote conferido em `shells/desktop/Cargo.toml`). Para o dono: Add ▸ Game ▸
Character; Play in a Window; andar, saltar, ver correr/saltar/cair.

## ABERTO

- **Plano 07 (próximo):** a animação num cartão só — decisão do dono pós-smoke.
- R1 (vista dupla ao vivo) · View ▸ Focus mode morto na UI nova · sem `ph2d.disconnect` (plano 03).
- O resto da arte (plano 06, as outras peças depois da amostra).
- Fecho da linha (DIRETRIZ §1.5.9) e a bateria do integrador.
