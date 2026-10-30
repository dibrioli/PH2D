# 18 — A6: a miniatura do que a câmera vê (plano, 30/10)

Plano [17](17_camera_no_canvas_plano.md) A6; handoff [18](handoffs/HANDOFF_CONTINUACAO_2026-10-30_18_CAMERA.md) §ABERTO 1.
Lei: a miniatura é FIEL (sprites, vetor, Flip, Motion, luz 2D) ou não existe. Meia-miniatura de sprites = recusada.
É, na prática, o R1 recusado em [04](04_R0_N0_plano.md):79 («desenhar o mundo 2× por quadro»).

## Medição (30/10, `fotografa_cena.sh` 1930×1040, `PH2D_FLUID_PROFILE=1 PH2D_LIGHT2D_CUSTO=1`, load 3,4)

| Cena | GPU/quadro | Partição |
|---|---|---|
| `PH2D_GAME_SMOKE=4` | 1,30–1,42 ms | `render.vello` 1,13–1,24 (mundo + chrome) · sprite 0,07–0,08 · compositor 0,03–0,05 · tonemap 0,02 |
| `PH2D_LIGHT2D_SMOKE=8` | luz 3,80 ms (min) + 0,53–0,71 | luz: grade 1280×704 da janela 1930×1012; c2 2,32 · miolo 0,61 · preenche 0,49 · fina 0,25 |

Quadro preso ao vsync (16,1 ms; `cpu-encode` 5,9–8,8 ms). Miniatura 320×180 ≈ 1/36 dos píxeis da janela ⇒ a 2.ª
passagem na placa é estimada em < 1 ms com luz (a grade da luz escala com o alvo). **Por medir depois da etapa 3:**
o CPU da 2.ª codificação (vetor + `recolher` da luz) — régua: `cpu-encode` com a miniatura On × Off, intercalado.

## O que a leitura (explorador ×2) estabeleceu

- Câmera e tamanho vêm de `gfx.camera` / `surface.size()` (`present.rs:62`). Aceitam câmera+alvo por parâmetro:
  `render_with_streams` (`renderer_draw.rs:99`, `submit` próprio por chamada ⇒ 2 chamadas sequenciais não colidem
  no `camera_buffer`), Flip `pass::render`, placa do Motion `desenha_o_quadro`, Vello `render_to_intermediate`.
- **Vetor:** codificado com a câmera do EDITOR embutida (`fase_hero_scene.rs:47` `cam_affine` →
  `ph2d_vec_render::dispatch`), na MESMA `VectorScene` que gizmos e chrome (`present_chrome.rs:170-180`); só com
  faixas vai ao HDR. ⇒ `append` com outra afim é recusado (leva o chrome e a câmera errada): 2.ª codificação do
  DOCUMENTO numa `VectorScene` própria com a afim da câmera do jogo.
- **Luz 2D:** `LuzDoQuadro` é estado de UMA vista (`mundo_para_tela`, `alvo_px`, `resolvido`, `vigia`,
  `governador`; `quadro.rs:66-109`, criada em `init.rs:174`) ⇒ 2.ª instância para a miniatura.
- **Vello:** `vello_pass.intermediate` é um alvo só ⇒ `VelloPass` (ou intermediário) próprio.
- **Vazam se reusados:** fantasmas do onion (slot `extra`, HDR), gizmos/moldura/ossos (cena do chrome), grade.
- Retrato do asset (`asset_card_portrait.rs:11-14`) é CPU sobre pixels — esta capacidade NÃO o serve (o handoff 18
  dizia o contrário).

## Etapas (cada uma com o seu gate por nome)

1. **Alvos da vista** (`ph2d-render`): `VistaDoMundo { game_rt, tonemap, vello }` no tamanho da miniatura.
2. **Luz da vista**: 2.ª `LuzDoQuadro` (sem montagem em fundo partilhada), alimentada com a câmera do jogo.
3. **Vetor da vista**: `dispatch` só do documento com a afim da câmera do jogo, sem overlays.
4. **Flip e Motion**: as passagens com a câmera e o tamanho da vista; conferir `FlipRenderer`/`FlipCompose`,
   `motion_fx`/`layer_compositor` (`write_buffer` antes do submit) e `present_frost::lift` (destrutivo?).
5. **Composição** num canto do canvas (blit), moldura fina, só com a câmera seleccionada e Preview On.
6. **Cartão Camera**: «Preview: Off | On» volta (`CameraCanvasView.preview`), gesto do editor como Outside/Editor View.
7. **Prova de fidelidade**: a miniatura × o mesmo mundo desenhado em tamanho cheio com a câmera do jogo (o Play),
   reduzido — por cena de smoke com vetor + Flip + Motion + luz; e a medição On × Off.

Código novo em crate (`ph2d-app-game` / `ph2d-render`); a shell só encolhe (`the_shell_only_shrinks`).
