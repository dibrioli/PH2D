# HANDOFF de INTEGRAÇÃO — `line/sculpt3d` · os efeitos de vizinhança borram NA SUPERFÍCIE da peça 3D (W6) + as camadas em tons de ecrã e a pilha na peça (2026-10-03)

> **Para quem é:** o agente INTEGRADOR (só por ordem do dono — CLAUDE.md §0.7). Superfície de colisão,
> contadores como DELTA, o que um merge pode partir, a prova de fecho e o que fica aberto. O mecanismo
> da W6 vive no [doc 3D 30 §14](../30_plano_camadas_e_efeitos_na_peca.md) (a lei, as medições, as
> recusas medidas).
>
> ⚠️ **SUPERSEDE** o [handoff CAMADAS_EM_ECRA (03/10)](HANDOFF_INTEGRACAO_line_sculpt3d_CAMADAS_EM_ECRA_2026-10-03.md)
> como documento de integração desta linha. Aquele **ainda NÃO está no `main`**: os seus 50 commits vêm
> nesta mesma série — leia-o (e o A_INCLINACAO que ele supersede) para o que eles trazem (ADR-0177, a
> pilha na peça, `ph2d-layer-ops`, `SCULPT_DOC_VERSION` 5→7, `DropdownOption::disabled`,
> `LayerCompositeError::AdjustmentInLightSpace`…). Tudo o que eles declaram continua válido e **soma-se**
> ao que está abaixo, que é só o que a W6 acrescenta.

## §0. Estado em uma tabela

| | |
|---|---|
| ramo · worktree | `line/sculpt3d` · `Worktrees/line-sculpt3d` |
| merge-base | `1ad60a1ce` (= `main` a 03/10; rebase desnecessário à data) |
| commits | **59** + o commit deste handoff (`git log --oneline 1ad60a1ce..HEAD`) — a W6 são os últimos **9** (`59fb691bc`…`e15c55880`) · a linha inteira **238** ficheiros `+20 527 −2 353`; só a W6 **65** ficheiros `+4 504 −263` |
| smoke do dono | W6: **PENDENTE** (§7, cena `=54`) · os anteriores: ver o handoff superseded |
| contratos §6 | **intocados** (`Tool=12`, `PanelEvent=4`, `NodeOp`) |
| `shells/desktop/src` | **0** linhas (a linha inteira toca só `shells/desktop/tests/it/the_sculpt_mesh_edits_are_wired.rs`, da W1b) |

### Contadores — DELTA da W6 (a fonte é o código; reconte contra o `main` do dia)

| contador | W6 | valor aqui |
|---|---|---|
| `PROJECT_SCHEMA` e família · `SCULPT_DOC_VERSION` | **0** | `SCULPT_DOC_VERSION` continua `7` (a W6 não muda o ficheiro: um ajuste de vizinhança grava-se como qualquer ajuste; o raio na peça é um `f32` nas unidades da peça) |
| cenas do roteador sculpt3d (`scenes::CENAS`) | **+1** (53→54) | `=54` os efeitos de vizinhança — o número conta-se no roteador desta família; outra linha que acrescente uma cena sculpt3d reconta |
| ADR | **0** | a lei está no doc 30 §14 |
| `Cargo.lock` | **0** | nenhuma dependência nova |
| `ph2d-painter-effects` `pub` | novos | `Neighbourhood`, `apply_adjustment_on`, `apply_gaussian_on`, `apply_sharpen_on`, `apply_bloom_on`, `apply_shadows_highlights_on`, `gaussian_sigma`, `SpatialUnits`, `SURFACE_RADIUS_MAX`, `adjustment_slider_params_in`, `set_adjustment_slider_param_in`, `adjustment_slider_numbers_in`, `spatial_extent_slots`, `rescale_spatial_params`, `AdjustmentKind::reads_the_image_plane` |
| chaves i18n | **−1 +1 +1** · texto | `app.sculpt3d.camadas.recusa.le_a_vizinhanca` **SAIU** → `…recusa.le_o_plano_da_imagem`; nova `…recusa.degrau_alto`; texto novo em `panel.painter_layers.piece.off_here` |

## §1. O que a W6 traz (o mecanismo está no doc 30 §14)

Os quatro efeitos que leem os vizinhos — **Gaussian Blur, Sharpen, Bloom, Shadows/Highlights** —
entram VIVOS na pilha da peça (a decisão do dono de 02/10) e borram **na superfície**: o raio é do
mundo (o mesmo desfoque a `8x` e a `32x`), uma aresta da malha não se vê (a amostra de aresta lê o que
o meio de uma face lê, a ±1 degrau). Motion, Chromatic Aberration e Halftone ficam apagados na peça com
a frase (pedem o plano de uma imagem). Na peça o raio lê-se em **% do tamanho dela** (curso até `2,5 %`
da diagonal); acima de `64x` a porta recusa (a frase no painel). Cena de smoke `=54`.

| bloco | commit |
|---|---|
| W6a+b: o gancho (`Neighbourhood`), a lei na retícula (`difusao`), a pilha da peça, a placa (`surface`) | `59fb691bc` |
| W6c: o menu «+» da peça e o seam test do produto | `417820c3a` |
| a cena `=54` | `a148f921f` |
| os gates que o arnês pediu (traço por baixo de um desfoque, translúcida, cor por vértice, unidades, chip em %) | `22bb7412c` |
| o «resto do calor» medido COSMÉTICO e retirado | `696113c07` |
| o arnês de mutação + 3 re-âncoras | `f9d61f006` |
| o que o fecho apanhou (o gate da W1 que afirmava a recusa, 1 aviso, 1 clippy) | `bd73254ed` |
| docs: o doc 30 §14 e a memória | `f29ecb5e8` |
| os quatro gates que a mutação mostrou cegos (V1, P9, M9, P4) | `e15c55880` |

## §2. Foundational tocado e a superfície de API (o que um merge pode partir)

| item | mudança | quem parte |
|---|---|---|
| `ph2d_painter_effects::adjustments::apply_adjustment_windowed` | agora `= apply_adjustment_on(.., &win)` — o despacho único vive em `spatial_neighbourhood.rs`; `apply_gaussian`/`apply_sharpen`/`apply_bloom`/`apply_shadows_highlights` delegam nas `*_on` com a grelha. **Ao bit** (os 105 gates da crate, o oráculo do Krita e os de placa do Painter/Flip) | uma linha que edite os corpos destes kernels em `spatial.rs`/`spatial_tonal.rs` (o Bloom partiu-se em `apply_bloom_on` + `grid_glow`) |
| `ph2d_painter_effects::adjustments::compute::{params,numbers}` | `adjustment_slider_params`/`set_adjustment_slider_param`/`adjustment_slider_numbers` são as `*_in(.., SpatialUnits::Pixels)`; os raios dos slots espaciais passam por `units.extent(..)` | quem acrescente um ajuste espacial: o slot entra em `spatial_extent_slots` (gate `the_spatial_slots_are_the_ones_the_units_move`) |
| `ph2d_tool_painter` compositor | `composite_into` ganhou um último parâmetro `nb: Option<&dyn Neighbourhood>` (privado); `composite_over` novo (`pub`) | quem chame `composite_into` noutra linha |
| `ph2d_tool_painter` ferramenta | `PainterTool::sync_piece_units` · `panel_spatial_units`; campo `piece_units`; `apply_param_edit(params, e, units)` (`pub(crate)`) | quem chame `apply_param_edit` |
| `ph2d_render::layer_compositor` | módulo NOVO `surface` (`SurfaceNeighbourhood`, `SurfaceGraph`, `SurfaceRefusal`, `LayerCompositor::set_surface`/`surface_key`) + `shaders/surface_heat.wgsl`; dois campos novos no `LayerCompositor`; `run_bloom_bright` saiu de `run_bloom`; `run_shadows_highlights` recebe `(pesos, meia)` em tuplos + `Option<[f32;2]>`; `Break`/`KernelPlan` ganham `sigma`; `gaussian_sigma` (cópia gateada em `spatial_weights_parity`) | quem edite `compositor/dispatch.rs`/`effects.rs` noutra linha. **Sem superfície instalada nada muda** (o 2D e o Flip não instalam) |
| `ph2d_mesh_colors::difusao` | módulo NOVO (folha sem dependências, como a crate) | — |
| `ph2d-panel-painter-layers` | `set_current_spatial_units` (`pub`); `adjustment_offered` = `!reads_the_image_plane` na peça; `paint_adjust` usa as `*_in` | — |
| `ph2d-app-painter/src/painter_bridge.rs` | +1 linha: publica as unidades. ⚠️ **O ficheiro está a 700/700** (o tecto é `≤ 700`, `architecture_workspace_file_loc_cap`): uma linha que lhe acrescente 1 linha estoura | a `line/PainterWatercolor` (aberta) se tocar a ponte |
| `ph2d-app-sculpt3d` | módulos novos `vizinhanca_da_peca*`, `pilha_da_peca_vizinhanca`, `scenes_vizinhanca*`, `sonda_vizinhanca`, `vizinhanca_no_produto_tests`; `PilhaDaPeca::novo_ajuste(kind, unidades)`; `RecusaDaPilha::LeAVizinhanca` → `LeOPlanoDaImagem` + `DegrauAlto`; `tinta_da_peca::pilha::{recompoe_sujas, em_dia, cor_por_vertice_da_composta}` | — |

### Consumidores conferidos (a pergunta é «a lei da porta tem de concordar com alguma lei dele?»)

| porta partilhada | consumidores | conferido |
|---|---|---|
| `apply_adjustment_windowed` + kernels | compositor CPU do Painter (`compose.rs`); `ph2d-render` (os testes de paridade); o Flip (não tem ajustes de vizinhança no compositor dele — junta em luz e recusa ajustes) | ao bit: a grelha chama os kernels de sempre, na mesma ordem (o Bloom de factor `> 1` interpola a pirâmide para um buffer cheio antes da soma: os mesmos `f32`) |
| `LayerCompositor` (placa) | Painter 2D, a peça (W1b), o Flip (`compositor_do_flip`, em luz), `fx_stack` | sem `set_surface` o caminho é o de antes: a prova é a §5 (os gates de placa do `ph2d-render` e do Flip) |
| `adjustment_slider_*` | o painel de Layers (2D e peça), o `apply_param_edit` da ferramenta, os seams do painel | `Pixels` = o de sempre ao bit (gate `the_grid_units_are_the_slider_of_always`) |

## §3. Para o integrador — linhas abertas

- `line/PainterWatercolor` (aberta): toca o Painter. Colisões possíveis: `painter_bridge.rs` (700/700),
  `ph2d-painter-effects/src/adjustments/spatial*.rs`, `ph2d-tool-painter/src/compositor/compose.rs`.
  Não negociei com ela; nada re-pinado nela.

## §4. Gates novos (W6)

| | |
|---|---|
| retícula | `ph2d-mesh-colors` `difusao_tests` (8): constante ao bit · raio `0` · o intervalo · o polinómio é o calor (contra passos explícitos) · **variância `σ²` exacta no mundo** a `8x` e `32x` · `Φ(1)` a um desvio · **a aresta do cubo como um plano** (CONTROLO pela ordem vaza) · threads ao bit |
| efeitos | `ph2d-painter-effects` `units_tests` (4) · `ph2d-render` `spatial_weights_parity::gaussian_sigma_matches_canonical_painter_effects` |
| peça | `vizinhanca_da_peca_tests` (11) · `scenes_vizinhanca_tests` (1) |
| painel | `seam_peca` (3; o gate do menu passou a afirmar o contrário de antes; o chip do raio em %) |
| com placa | `tinta_no_produto_tests::placa_vizinhanca` (3: paridade ±1 incl. translúcida · a saída da placa borra na superfície · o traço por baixo de um desfoque) + `painel::o_desfoque_pelo_menu_borra_a_peca_e_o_ctrl_z_o_tira` (o seam test do DoD) |
| mutação | `docs/3D/ferramentas/muta_os_efeitos_de_vizinhanca.sh` **49/49** (1.ª corrida 47: V1 sem fixtura obtusa, P9 olhado só depois do pen-up; curadas em `e15c55880` — P9 sangra na re-corrida, V1 numa corrida dirigida) · `muta_as_camadas_em_ecra.sh` **50/50** (a grelha 2D ao bit) · `muta_a_pilha_na_placa.sh` **19/19** · `muta_a_pilha_da_peca.sh` **43/43** (1.ª corrida 41: a M9 re-ancorada sem gate e a P4 — viva desde a W1b — curadas; M9 na re-corrida, P4 numa corrida dirigida) |

## §5. Prova de fecho (batched sobre o diff acumulado)

| passo | resultado |
|---|---|
| `nextest-impacted.sh` | **19 441 / 19 442** — o vermelho era o gate da W1 `a_composicao_da_peca_e_a_do_painter_ao_bit`, que afirmava a recusa do desfoque (curado em `bd73254ed`, verde); nenhum flake de carga |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | ✓ (depois de 1 variável sem uso num teste) |
| `cargo clippy --workspace --all-targets -D warnings` · `fmt --all --check` | ✓ (depois de 1 `neg_cmp_op_on_partial_ord`) · ✓ |
| `machete` · `check-standalone-optional` (10/10) · `check-workflow-packages` | ✓ · ✓ · ✓ |
| `architecture_*` | **102 / 102** · memória `o_indice_da_memoria_conta_o_que_aponta` **3 / 3** · `claude_md` **3 / 3** |
| placa | `ph2d-render` **52/52** (`layer_compositor` + `fx_stack_adjust` + `blend_mode_regression` + `layers_no_alloc`) · `ph2d-flip-render composite_` **19/19** · sculpt3d `--ignored tinta_no_produto --skip diag_` **38/38** (+4) · o `--ignored` INTEIRO do `ph2d-render` **160/164**: os 4 `smoke_fixture_renderable::w2_..w5_` são stubs `unimplemented!` que já vêm assim do `main` |
| pré-voo dos `muta_*.sh` (22 + o novo) | limpos, menos B1/B5/B14 do Skeleton (do `main`) |

⚠️ Os testes reforçados depois do fecho (`e15c55880`, só gates) correram verdes um a um; o `nextest-impacted` não foi repetido sobre eles.

## §6. Perfil de agente (`bash scripts/agent-loop-profile.sh`, 20 sessões)

```
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                166   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                514 : 245   alvo: <= 1,0  razao 2.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%  (806 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         358 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §7. Smoke (do dono: a cena `=54`)

Binário compilado nesta worktree no HEAD, depois de `rm -rf target/*/incremental` (`20 G` + `8,4 G`);
a 1.ª build foi de `28,6 s`:

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

**Passos (texto para o Enio)** — a foto da cena tal como abre está conferida (a risca escura atravessa
a frente da bola na diagonal, por cima das linhas do arame; `16x` aceso):

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=54 cargo run -p ph2d-host-desktop --profile smoke`
2. Abre a bola cinzenta clara com uma risca escura na diagonal e as linhas do arame. Na barra de cima
   troque `IMG` por `PNTR` (o Painter); no painel dele abra o separador `Layers` — a lista mostra `Layer 1`.
3. Carregue no `+` dos ajustes e escolha `Gaussian Blur`. Nasce a linha `Gaussian Blur` com a barra
   `Radius` a zero; a bola ainda não muda.
4. Arraste a barra `Radius` devagar para a direita: **a risca desfoca enquanto arrasta**, e o número da
   barra é em % do tamanho da bola (no fim, `2.5`). Onde a risca cruza uma linha do arame, o desfoque
   passa por cima sem costura — nem mais claro nem mais escuro. Gire a bola (botão direito): o desfoque
   é da bola, gira com ela.
5. `+` → `Sharpen`, suba `Amount`: a borda da risca ganha contorno. `+` → `Bloom`, suba `Intensity`: a
   parte clara brilha e o brilho entra na risca.
6. `Ctrl+Z` várias vezes: os efeitos saem um a um e a risca volta nítida.
7. **Deu errado se:** a risca não desfoca ao arrastar; aparece uma linha mais clara ou mais escura por
   cima de uma aresta do arame; o desfoque escorre para longe da risca (manchas noutros sítios da bola);
   ou o `+` mostra `Gaussian Blur` apagado.


## §8. `CLAUDE.md` §5.1

A linha do módulo **3D / Sculpt** troca o link do «Último» para este handoff (a natureza do módulo não
mudou).

## §9. ABERTO (nomeado)

- **`64x` com um raio grande é aos solavancos** (`340 ms` no fim do curso, medido) e **`128x`/`256x`
  recusam** os efeitos de vizinhança (`NIVEL_MAX_DA_VIZINHANCA`; estimado `~1,3–2,7 s` por passo a
  `128x`). Cura medida possível: uma pirâmide pelos degraus da retícula (o desfoque largo num degrau
  mais grosso, interpolado de volta: custo independente do raio) e/ou um estêncil implícito por face na
  placa (as faces são retículas regulares; só as arestas pedem o grafo — o grafo em CSR é `~56 B` de
  `~150 B` por amostra e termo).
- **A cor por vértice com uma pilha que lê vizinhos** fica um gesto atrás (igual ao prefixo atrasado,
  o estacionamento são) até ao `em_dia` / gravar-abrir; invisível com o plano armado. A cura: uma
  composição exacta da CPU em segundo plano depois do gesto.
- **Motion, Chromatic Aberration e Halftone na peça:** não existem (sem desenho honesto numa superfície —
  doc 30 §14); apagados com a frase.
- O limiar do despré-multiplicar da CPU (`1e-6`) e da placa (`f32::EPSILON`) diverge também no 2D — só
  se vê em cor debaixo de alfa nulo, que ninguém lê; anterior à W6.
- Herdado e ainda aberto: ver o §9 do handoff superseded (W4 relevo por camada, W5, W7 remesh, o smoke
  2D antes × depois da P4).
- ⚠️ 3 âncoras de `docs/Skeleton/ferramentas/muta_a_silhueta_do_contacto.sh` (B1, B5, B14) **vêm partidas
  do `main`** — não são desta linha.
