# Continuação — line/GameEngineFuncional, 09/11 → a ferramenta Align (GEF 28)

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` · branch `line/GameEngineFuncional`
· base `main` `f254e42bf1`. ⛔ Não tocar na `line-GameEngineFuncional02` (outra janela). Nada integrado, nenhum push.

## O que esta janela fez (4 commits sobre `f254e42bf1`)

| Commit | O quê |
|---|---|
| `ec1c1e9ea2` | Pivot de qualquer folha deixa o desenho no lugar; Ctrl sobre o pivô = encaixe (não desseleciona); Trim/Padding/Make Square **quadro a quadro** numa folha cortada (`ph2d-anim-sheets::cell_edit`); imagem escolhida troca a folha à vista; Edit mostra a folha inteira (`ImagemAberta` sem Painter); ilhas do BG removal com a pose da fonte |
| `c6f9aa778e` | «+» cria animação **vazia** na folha à vista (`SpriteAnimFrames`: lista vazia = sem quadros; `remove` = intervalo; `InspectorAnimRow.frames: Option`; `click_base` saiu); marca de inserção anda; a tira mostra o quadro que toca; drop na miniatura de 2 folhas (`INSP_IMAGE_SHEET_PICK` em `texture_slot_pick`); a imagem editada herda o nome (`imagem::da_folha/editada`, folhas extra contam como vivas na `library()`); moldura do USO (`ph2d-anim-sheets::use_frame`, `::uso` com `ImageUse` movido da app-crate pela catraca) |
| `6bb5777f33` | a moldura/etiqueta do uso **só com a folha inteira à vista** (dono: sobre o resultado final atrapalha); trocar a imagem da folha à vista **avisa** (`ShownSwap` + `swap_toast`: corte mantido / corte refeito) |
| `c39453e29a` | estudo `docs/GameEngineFuncional/28_alinhar_folhas_estudo.md` |

## Smoke do dono (09/11)

- OK: encaixe do pivô (mantém o ponto do centro do desenho — decisão do dono), Trim por quadro, nome «run» após Trim,
  animação nova vazia, marca, drop na miniatura, moldura em Edit (foto 2 dele: «ficou ótimo»).
- ⚠️ **Por confirmar à mão** (o dono não reportou ainda): o 3.º round — sem etiqueta fora do Edit e o aviso amarelo ao
  trocar a imagem da folha. Peça-o junto com o smoke da onda A.
- Não reproduzido: a moldura azul do gizmo que sumiu no Edit no PC dele (nas fotos aparece); a moldura verde do uso
  cobre a necessidade.

## Aberto (fora da Align)

- «Delete empty animations» não apaga animações de 0 quadros (só as de buracos de folha apagada) — deliberado.
- O «+» do cartão Animation do Inspector não seleciona a animação nova (o do editor sim) — anterior a esta janela.
- Memória `project_gef_animated_sprite_delivered_without_owner_smoke_2026_11_07.md` atualizada nesta janela.

## Próximo: GEF 28, onda A

Leia `docs/GameEngineFuncional/28_alinhar_folhas_estudo.md` (decisões do dono no fim: **sem giro**; onion skin =
quadro anterior + referência fixada). Onda A = dados + render: `offset`/`scale` por folha (`ExtraSheet` e o equivalente
da folha 0), deslocamento por célula (mapa esparso `cell → [dx,dy]` px, mora na FOLHA), sobe o `PROJECT_SCHEMA`
(`python3 scripts/schema-recount.py`), o render compõe Transform × folha × célula, e `sheet_geometry` devolve a geometria
já composta para a caixa do gizmo, o encaixe e a colisão lerem a mesma. Gates de geometria por nome. Depois B (a prévia
Align no `ph2d-anim-editor`), C (por quadro + «Alinhar pelos pés/centro»), D (cena de smoke `PH2D_GAME_SMOKE=5` ou nova,
com fotos pelo `fotografa_cena.sh` antes de mandar ao dono).
