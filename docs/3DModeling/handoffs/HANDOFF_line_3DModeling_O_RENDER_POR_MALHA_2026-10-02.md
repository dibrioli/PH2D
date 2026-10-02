# Handoff — `line/3DModeling`: o RENDER POR MALHA (2026-10-02)

> **Wave, não integração.** Decisão: [ADR-0176](../../architecture/decisions/0176-o-render-do-modelador-e-uma-malha-de-jogo.md)
> (⚠️ número provisório). Base `1ad60a1ce`. Commits: `b19820b72` extract_parts · `37c5ba25c` (LOC
> 712→698) · `04d2ef088` malha_render/malha_render_tri + par extract · `b5b4a3c4d` mesh-forward ·
> `f3ad6ef3b` integração field3d · `2a4f7d485` qualidade/faixa/par/sombra/cena 37 · `84ba27b38`
> painel/censo/clippy.
> Ordem do dono: *«o Render ainda não está em tempo real como uma game engine»* — e tem de rodar em mobile.

## §1 — O que existe

| Peça | Caminho | Lei |
|---|---|---|
| Partes | `crates/ph2d-field-eval/src/extract_parts.rs` | rótulo de **SÓLIDO** nas amostras de dentro, 6-vizinhança; a célula que cruza une os cantos. Esfera oca com bolinha solta = **2 peças** |
| Camadas | `.../extract_planes.rs` | camadas em paralelo, **um JIT por thread**, bit a bit iguais |
| Faixa | `.../extract_band.rs` | faixa estreita: blocos 8³, `L = gradient_bound`, margem = meia diagonal + 1 diagonal de célula; malha **bit a bit** a da grade cheia |
| Paralelo | `.../par.rs` | valor/gradiente em paralelo |
| Unidades | `crates/ph2d-app-field3d/src/malha_render.rs` | unidade = árvore aberta só por uniões secas sem modificador cujos filhos somam; **grupos = bolas que se tocam**; colhe/processa. Resolução **MEDIDA**: célula = diam_peça/256, refaz ×2 enquanto >0,5 % triângulos virados, prof 5..8 |
| Triângulos | `.../malha_render_tri.rs` | Newton até à superfície (passo ≤ ½ célula, 4 passos); normais = gradiente nos **CANTOS** a 30 % para o centro, auto-smooth 30°; quina → interseção dos planos dos lados (Kobbelt 2001); AO de 5 passos em MUNDO [0,01..0,16] com d=f/\|∇f\|; material por triângulo pela folha dona |
| Estado | `.../malha_render_estado.rs` | `thread_local`; forma nova → extrai noutra thread; **só poses → nada extrai no gesto**, desenha com agora∘extraída⁻¹; fim do gesto → verificação que só troca se a partição mudar ou um objeto deixar de se mover junto; outra raiz recomeça; seleção por objeto inteiro (`SelectRequest::Many`); trava com i18n `app.field3d.malha_render.*` |
| Quadro | `.../malha_render_quadro.rs` | câmara = **a mesma conta de pixel do Orbit**; desenhista **GLOBAL**; só redesenha se a assinatura mudar |
| Smoke | `.../smoke_draw.rs` (o ramo do Render) · `smoke_scenes_malha.rs` (cena 37) | |
| Cena | `.../scene.rs` (ecs_bridge) | trava antes do `apply_motion`, seleção, sync |
| Céu | `.../studio_wgsl.rs` | `tabela_ler` é ranhura; céu em duas partes: sem caixa / da caixa |
| Desenhista | `crates/ph2d-mesh-forward/` | `forward.wgsl`: `mx_indirect` com ambiente = céu·AO + caixa·sombra (linearidade → uma avaliação); PCSS 16+16 nos objetos; chão que só recebe; `gpu_cobertura.rs`: pirâmide de cobertura vista de cima (penumbra física, céu tapado na janela da altura); `ecra.wgsl` = o `premultiplicado::para_ecra` |

Chaves: `PH2D_FIELD_RENDER_TRACADO=1` volta ao traçado · `PH2D_FIELD_SHADING=render` abre no Render.

## §2 — Medições

Release, **máquina partilhada** (⚠️ acima de `load ~5` o relógio não vale — ver §5-c).

**Quadro** 1920×1080 a girar 3°/quadro, cada um de raiz + leitura:

| Cena | mediana | p90 | máx |
|---|---|---|---|
| 28 | 1,17–1,24 ms | 1,5–1,6 ms | ~3 ms |
| 37 | 1,26 ms | — | — |

Cena 28: **73 152** triângulos.

**Entrar no Render** (extração completa, cena 28):

| Passo | Tempo | load |
|---|---|---|
| antes do paralelo | 680 ms | 2,6 |
| paralelo | 168 ms | — |
| prof 8 fixa | 969 ms | — |
| + faixa | 686 ms | — |
| + par | 328 ms | 20 |
| **resolução medida** | **0,32–0,44 s** | 6–20 |

Cena 37: 1,6 M triângulos com prof fixa → **78 k** e **32–51 ms**.

**Virados × profundidade** (nós da cena 28, peça inteira):

| prof | virados | leitura |
|---|---|---|
| 6 | 22,9 % | |
| 7 | 1,6 % | serrilhado visível |
| 8 | 0,14 % | liso |

Cena 37: prof 7 → 1,5 %; prof 8 → 0,08 %.

⭐ **Os nós da cena 28 TOCAM-SE** (o 2 entra 0,0099 no 1 e 0,0087 no 3; o 0 fica a 0,0020 do 1): são
**2 objetos, não 4**.

## §3 — ⛔ Recusas MEDIDAS (não reconstrua)

| Recusado | Por quê |
|---|---|
| Um gradiente por triângulo; gradiente no vértice | martelavam o nó de toro (fotos) |
| Normal geométrica | quina 7°: 1 172 / 27 232 vértices tortos na caixa |
| AO em células | encolhia com a profundidade |
| AO com `f` cru | faixa escura no tubo: \|∇f\| ≈ 0,5 (daí d = f/\|∇f\|) |
| PCSS preso a 48 texels para o chão | chão intacto — gate `a_sombra_pousa_no_chao` |
| `fidget::mesh` | W0/W19 |
| Desenhista em `thread_local` | o `Drop` do wgpu tocava TLS destruída e **abortava ao fechar** |

## §4 — Gates e prova

| Crate | Gates |
|---|---|
| `ph2d-field-eval` | `the_parts_are_the_pieces_that_touch` (5) · `the_planes_are_the_same_with_any_thread_count` · `the_band_gives_the_same_mesh_as_the_full_grid` · `the_parallel_answer_is_the_serial_one` |
| `ph2d-app-field3d` | `malha_render::tests` (8, incl. cena 37) · `malha_render_estado::tests` (2) · `costura_tests` (1 + 1 GPU) · `malha_render_quadro::tests::a_projecao_e_a_do_orbit` (< 0,01 px) · `studio_wgsl` `as_duas_partes_somam_o_ceu` (1,6e-7, GPU) |
| `ph2d-mesh-forward` | `o_shader_valida_sem_capacidades` · `nenhuma_ranhura` · `cabe_no_celular` · `quadro_pronto_na_hora` · `nada_compila_ao_editar` · `a_cor_e_a_lei_da_casa` · `a_sombra_pousa_no_chao` (GPU, `ignored` por omissão) |

⚠️ `cabe_no_gles`: **SEM adaptador GL nesta máquina — não mediu nada** (declarado).

**Suíte:** `field3d` lib 493 verdes. Único vermelho fora disto: `profile_formula_tests::a_formula_e_ajustada_uma_vez_por_peca` =
flake de carga **pré-existente** (3/3 verde sozinho a `load 24`). clippy `-D warnings` limpo nos 3 crates;
`editor-core` it 532/532.

## §5 — ABERTO

- **(a)** Style e Bloom no desenhista de jogo: bloom em passes de desenho com paridade à CPU; style precisa da curvatura por vértice assada. Hoje as fileiras escondem-se no Render por malha.
- **(b)** Saliências de 2–3 px na quina **CÔNCAVA** da caixa mordida: é a **extração**, não o Newton (medido sem ele: igual).
- **(c)** Re-medir o relógio da entrada com `load < 5`.
- **(d)** ⚠️ **O chão escurece POUCO.** Medido (02/10, estúdio do produto, esfera `r = 0,3` pousada,
  vista de cima): alfa do chão a `0,36` do centro = **`13`** com a cobertura média → **`19`** com a
  oclusão por HORIZONTE sobre o mapa de alturas (8 direcções × 5 passos, `cos²` do horizonte por
  fatia, `forward.wgsl::ceu_do_chao`); a conta física de mão (céu ponderado pelo cosseno tapado pela
  esfera, ~30 % da parte sem caixa) dá **~60**. ⇒ a próxima acção é um GATE DE PARIDADE contra o
  catcher do Render traçado (`ph2d_field_render::ground` — a lei já aprovada) em 4–5 pontos à volta
  de uma esfera e de uma caixa pousadas, e calibrar até bater; não inventar um factor.
- **(e)** GLES real: nenhum adaptador GL aqui.
- **(f)** O Render traçado sai quando o dono aprovar.
- **(g)** Mobile: medir num aparelho real.
- **(h)** LOD/simplificação se o orçamento pedir (hoje 38–78 k triângulos).

## §6 — Smoke (para o dono)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=37 cargo run -p ph2d-host-desktop --profile smoke`
2. Clique em **MODEL**, depois em **Shading**, depois em **Render**.
3. Deve acontecer: a cena aparece com luz e sombra de jogo; girar a vista é fluido, sem borrar nem granulado; mover uma peça move de verdade.
4. Deu errado se: borra ao girar, engasga, ou demora mais de ~1 s a entrar no Render. Peça cortada/espelhada que não mexe deve mostrar uma frase explicando a trava.
