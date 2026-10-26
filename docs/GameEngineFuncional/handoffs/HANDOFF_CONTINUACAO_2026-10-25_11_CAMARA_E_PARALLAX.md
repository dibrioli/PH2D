# Handoff de continuação — `line/GameEngineFuncional`, plano 11 (câmara presa e fundo em parallax) (25/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`. **Não integrada, não
empurrada.** Antecessor: [`HANDOFF_CONTINUACAO_2026-10-25_10_ARTE_DE_AGACHAR.md`](HANDOFF_CONTINUACAO_2026-10-25_10_ARTE_DE_AGACHAR.md).
Plano: [`11_camara_e_parallax_plano.md`](../11_camara_e_parallax_plano.md) (o §«Feito (25/10)» tem os números).

## Estado

**Smoke do dono: POR RECEBER.** Fotos: [`fotos_11/`](fotos_11/) (`nivel_2_02.png` início com o fundo;
`nivel_2_03.png` câmara parada no fim do nível com o chão até à borda; a última é o editor depois do Stop, as
camadas na pose autorada).

| sha | O quê |
|---|---|
| `c924e11ea8` | J4 — fundo em parallax do nível 2 + Background Layer no Add |
| `b9a7299402` | fix: realce de hover no jogo, cor do céu (linear), fim do nível (`CameraLimits` max x 18 → 17) |
| `970a66f3f1` | fix: no palco o canvas é a janela inteira (sem a costura das docas) |

## O que entrou

- **MEDIDO antes de codar:** a câmara JÁ fazia o J4 (`GameCamera`/`CameraFollow`/`CameraLimits`, lei zona
  morta → amortecer → limites, ligada no Play por `janela_do_jogo.rs::game_camera_preview`) e a paralaxe também
  (`ScrollFactor`/`ScrollRepeat`/`ScrollLimits`/`ScrollMotion`, `parallax_bridge`, secção do Inspector,
  `fase_paralaxe` depois da câmara). Nada reconstruído. O nível 2 só não tinha camada nenhuma.
- **Oráculo Godot 4.7.2 (MIT)** (`ferramentas/camara_godot.gd`; `PH2D_GPU=1 bash scripts/ph2d-run.sh godot
  --headless --resolution 1600x900 -s …`): encostada a um limite o `offset` da `Camera2D` passa o limite (cima
  −1100 vs −1000; direita 2100 vs 2000); nível menor que a vista ignora limites; `Parallax2D` `scroll_scale`
  0,5 anda metade. ⇒ o PH2D já é igual: o `offset` da `GameCamera` soma depois dos limites; o do
  `CameraFollow` entra na mira, antes (preso). **Lei não mudou.**
- **Arte** (`ferramentas/arte_svg.py`; fundo azul do Kenney *tilemap-backgrounds*, zip sha256 `d01a196d…`,
  CC0): `nuvens.svg` (8×2 m) e `montes.svg` (8×6 m, corpo cheio até em baixo, 3 árvores-silhueta). Paleta: céu
  `#DFF6F5`, nuvem `#FFFFFF`, nuvem_sombra `#EEF9F8`, montes `#C2E3E8`, montes_escuro `#ACD5DD`. Os 6 SVG
  antigos saem byte a byte iguais.
- **`ph2d-render`:** `CLOUDS_TILE_KEY = u32::MAX-8`, `HILLS_TILE_KEY = u32::MAX-9`,
  `BACKGROUND_TEXELS_PER_M = 128`; `CLOUDS_TILE` px [3072,256] e `HILLS_TILE` [3072,768], grid [3,1] tile;
  `FACTORY_ART` 6 → 8 (novas no FIM); o cache de `factory_pixels` passou a `FACTORY_ART.len()`.
- **`ph2d-app-game` `object_add`:** `BACKGROUND_LAYER` (`object_add.game.background_layer`, `ENTRIES` 5 → 6),
  `CAMADA_K` 0,4, `CAMADA_LADRILHOS` 4, `ladrilho_m`, `veste_camada(world, e, art, ladrilhos, k)` (Sprite
  atlas, `SpriteGrid` frame do meio, `UvTransform` [n,1], `TextureRepeat`, `ScrollFactor`, `ScrollRepeat` tile
  [lw,0]); nasce com `ZIndexOverride(-1)`. Dica ilustrada `dicas::BACKGROUND_LAYER`; i18n rótulo «Background
  Layer» + `.tip`.
- **Nível 2 (`smoke.rs`):** Sky = Sprite com `ScrollFactor` [0,0], `ZIndexOverride(-3)`, 40×16 m em [0,0],
  `CEU_COR` linear [0.7379, 0.9216, 0.9131] (= `#DFF6F5` pela curva sRGB IEC 61966-2-1; o tingimento é LINEAR —
  escrito como 223/255 saía `#F0FBFA`, medido na foto); `CAMADAS_2`: Clouds k 0,15 (−2) e Hills k 0,4 (−1),
  `autorada(no_mundo, k)` sobre `CENTRO_DO_INICIO` [-5.9, 3]. `CameraLimits` do nível 2: max x 18 → 17 (fim do
  chão).
- **Dois defeitos achados pela FOTO e curados:**
  (a) contorno de hover do editor desenhado no jogo (fio azul de 2 px = a aresta de cima da camada Hills):
  `shells/desktop/src/render_loop/fase_pointer_subjects.rs` não escolhia objecto apontado com `no_palco()` (os
  outros auxiliares já perguntavam); +4 linhas na shell, `the_shell_only_shrinks` verde.
  (b) as docas fechadas reservam `DOCK_SEAM_PX` = 6 px de cada lado e o layout do palco herdava a reserva: a
  câmara media 1588 px de canvas numa janela de 1600 ⇒ 6 px de mundo além dos limites de cada lado (montes
  depois do fim do chão). `ph2d-editor-core::frame_layout::palco` põe `draw_area = viewport`. Vale para todo
  jogo (janela e ecrã cheio).
- **Dev-dependency nova:** `ph2d-app-game` → `ph2d-preview-drive` (só testes). `Cargo.lock` mudou por isso.

## Superfície que um merge pode partir

- `FACTORY_ART` `[_; 8]` e as chaves `u32::MAX-8`/`-9` (números que somam entre linhas: contar).
- `ENTRIES` do jogo = 6 (outra linha que acrescente ao Add colide).
- Exports novos do `ph2d-render`: `CLOUDS_TILE(_KEY)`, `HILLS_TILE(_KEY)`, `BACKGROUND_TEXELS_PER_M`.
- `frame_layout` (foundational de `ph2d-editor-core`) e `fase_pointer_subjects` (shell, +4 linhas).
- Teste `o_nivel_2_nasce_so_pelo_menu` agora espera o Sky com `ZIndexOverride` −3.
- Sem campo gravado, sem componente novo, **sem degrau de `PROJECT_SCHEMA`**.

## Prova de fecho (DELTA)

Gates novos (verdes): app-game `a_camara_do_nivel_2_nao_mostra_fora_dos_limites` (16:9/4:3/21:9, percurso
ida-e-volta a y 1/2,5/12 + fundo do buraco, cantos da janela ±½ px, controlo sem `CameraLimits` foge > 1 m; os
limites = bordas desenhadas), `o_fundo_anda_a_fraccao_autorada_e_cobre_a_vista` (Δ = (1−k)·Δc a menos de um
inteiro de ladrilhos, céu cobre, nuvens/montes cobrem a largura em 21:9, montes descem abaixo da borda de
baixo, chão parado), `a_camada_de_fundo_do_menu_nasce_pronta`; render `as_artes_antigas_sao_as_de_antes`
(FNV-1a `0xea58a1ac470a2c84`, medido ANTES da arte nova) e `os_azulejos_repetem_sem_costura` estendido às duas
novas (opacidade de borda só para o chão); editor-core `no_palco_o_canvas_e_a_janela_inteira`; host-desktop
`o_palco_do_jogo_nao_tem_realce_de_hover` (fonte do quadro).

**Mutação 11/11 sangra** (limites ignorados, max x 18, tile 0, 2 ladrilhos, montes 2 m acima, ponte sem
escrever, ordem do atlas, uma cor da terra, `draw_area` da costura, hover sem palco, ZIndex +1).

Por nome verdes: render `atlas::`, app-game `camara_tests`/`smoke`/`object_add`/`arte_tests`/`personagem`/
`encaixe`, i18n `a_tabela`, app-sprite-anim `sheet`, editor-core `architecture_workspace_file_loc_cap`/
`the_shell_only_shrinks`, host-desktop `the_app_never_reshapes_a_still_screen`. `cargo fmt --all` limpo; clippy
`-D warnings --all-targets` de `ph2d-render`, `ph2d-app-game`, `ph2d-i18n`, `ph2d-editor-core`,
`ph2d-host-desktop` limpo. **Suíte de impacto NÃO corrida** (do integrador, ordem do dono). Smoke build, 2.ª
corrida: `Finished smoke profile [optimized] target(s) in 0.31s`, zero `Compiling`.

## Smoke (para o Enio)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=2 ./target/smoke/ph2d-host-desktop`
2. Menu **Run ▸ Play in a Window**; segure **→** para correr, **↑** para saltar.
3. Tem de acontecer: o céu fica parado; as nuvens quase não andam; os montes com árvores andam devagar; o chão
   passa à velocidade do boneco. No começo (à esquerda) e no fim (a bandeira) a câmara pára e o chão chega à
   borda da janela.
4. Deu errado se: aparece um fio/risco azul atravessando o jogo; depois da bandeira vê-se montes por baixo de
   onde o chão acabou; o fundo anda junto com o chão; o céu tem uma cor diferente do resto do céu.
5. (opcional) **Add ▸ Game ▸ Background Layer**: passe o rato para ver a dica com desenho; o objecto nasce com
   os montes; no Inspector, secção **Parallax**, mude o número (0 = parado, 1 = anda com o nível).

## ABERTO

- Herdado (handoff 10): `AnimRules` sem nome de script nem lei em `property_laws::LEIS` · barra de largada das
  regras sem seam · «This frame» `1.330x` vs `1.33x` · R1 vista dupla ao vivo · View ▸ Focus mode morto ·
  `ph2d.disconnect` · a faixa do editor não rola ao arrastar junto à borda · corpo sem troço recto (bola) a
  agachar abaixo do piso pode encostar e travar (sem gate) · suíte de impacto (integrador).
- Herdado (menor): o desenho é mais largo que a cápsula (±0,27 de pé, ±0,30 agachado, raio 0,2) — encostado à
  parede o capacete pinta uns cm por cima dela.
- Herdado: regra derivada por NOME de convenção — renomear a tag `crouch` tira a regra derivada (a 1.ª edição
  do cartão grava as regras e daí o rename leva-as).
- **Novo:** o editor (fora do Play) mostra as camadas na pose autorada — com a câmara do editor longe, os
  montes acabam a 2,6 m abaixo do chão e vê-se o céu por baixo (é a lei: a paralaxe só anda com a câmara do
  jogo).
- **Novo:** a Background Layer escalada pelo Transform desalinha a repetição (o ladrilho é em metros do mundo).
- **Novo:** `CameraLimits` do nível 1 continua a max x 18 (não medido).

## PRÓXIMO

**J5 do [plano 03](../03_plano_jogo_de_plataforma.md):** moedas que se apanham (som + contador no HUD): `Area`
+ sinal «entrou», `Audio Player`, HUD (`Label`), destruir objecto. Medir primeiro o que existe (a `moeda.luau`
já some ao tocar; `UiCanvas`/`UiLabel`/`Counter`; `AudioSource2D`); plano `12_*.md` antes de codar.
