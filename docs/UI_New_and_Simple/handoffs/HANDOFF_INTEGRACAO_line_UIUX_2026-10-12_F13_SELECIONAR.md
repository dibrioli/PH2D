# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-12 — F13: O MODO FORMA SELECCIONA, A *RESIZE BOX* GUARDA O TRAÇO, O *VIEWED*, A CAIXA BAIXA E O BICO NA PLACA DO MOTION

> Fecha **F9 + F10 + F11 + F12 + F13 juntas** (nenhuma está no `main`). O que a F9–F12 mudaram,
> mediram e deixaram aberto: [handoff da F12](HANDOFF_INTEGRACAO_line_UIUX_2026-10-12_F12_SETAS.md)
> (que cita os da F11, F10 e F9). Este cobre a F13. Pedido do dono palavra por palavra e o escopo
> A–E: [`22_prompt…`](../vetor_nova_ui/22_prompt_F13_selecionar_no_modo_forma_e_as_respostas.md).
> Janela intermédia: [continuação F13](HANDOFF_CONTINUACAO_line_UIUX_2026-10-12_F13.md).

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, sobre o `main` `b1dd77e477` (o `main` não andou
  desde o rebase de 12/10; 0 commits atrás).
- **`PROJECT_SCHEMA` NÃO mudou** — nenhuma migração. Nenhum contrato congelado tocado.

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

A entrada UI/UX: «(F12)» → «o modo Forma que selecciona (F13)», smoke `…6..20`, «Último:» → este
handoff.

## §1 — O que mudou (F13)

| commit | o quê |
|---|---|
| `95da41c20c` · `21b4235311` | **o Polígono pelo relatório do dono** («Cutout estranho, Smooth invertido»): o *Cutout* em esquadria até ao centro do arredondado (`corner_style::normals_meet`), o *Smooth* com `1` = o redondo (`rounded_at_one`) sem apagar o raio do canto, o tipo do canto mantido no tecto do raio (`corner_live`, a tangente do segmento de origem). Doc 15 §3b |
| `f5b37f04c0` | **A — seleccionar no modo Forma**: o clique que acerta numa forma DESARMA (`after_the_shape_release`: `shape_armed = false`, mesmo sem a selecção mudar) e as alças dela voltam; no modo Forma os NÓS nunca se desenham (`hides_the_nodes`: `mode == Shape ‖ …`) |
| `63157a4bf9` | a cena NOVA `=20` (`vector_mode_seleciona`: estrela + elipse, a paleta arma um polígono, clique no meio da estrela, arma de novo, clique dentro da elipse; `PH2D_CENA20_FICA=0` pára na estrela); gate H4 — a forma sem `VecShape` é seleccionada sem alças e sem nós |
| `04aea82470` | **A, depois do 1.º smoke do dono** («tenta criar uma nova forma, aparece na Hierarchy e some»): o `ShapeTool` empurrava um path degenerado no PRESS — o clique ganhava entidade antes do soltar. Agora a forma só nasce no arrasto passado o `MIN_DRAG_PX` (`pressed` separado de `active`; `is_active` = há gesto). `on_press` deixou de devolver o id (ninguém o lia). Testes: `shape::` 82/82 · `shape_mode_tests seleciona vector_mode` 28/28 (corrida dirigida, sem o portão inteiro — pedido do dono) |
| `d08b074913` · o seguinte | **O Polígono depois do 2.º smoke do dono** («Poly piorou: dois controles para curve; Smooth horrível — antes era bom mas invertido; cutout assimétrico»). Medido por folhas de contacto (antes × agora, `/var/tmp/ph2d_F13_poly/`): o *Smooth* volta ao *Rounded* do Inkscape de antes (redondo a ~0,37, depois VIRAVA o pentágono — o «invertido»), agora com `1` = o redondo e o raio do canto a entrar no liso (`raw = R1·(quinhão + liso·(1 − quinhão))`); sai a rotação aos poucos da F13 (deixava a quina do raio: calombos). O *Curve* do polígono só no ponto da aresta (sai o ⇕; gate `no_field_has_an_arrow_and_another_handle` no catálogo inteiro). O dono, do *Cutout*: «qualquer um está bom, mas quando arredonda as quinas fica ruim» → com envergadura as paredes do Cutout e o chanfro ficam RECTOS (`bend_between_corners` só enverga os lados; o G1 só junto de um ARCO, decidido antes de envergar), e com liso nenhum estilo de canto (lascas até ao centro). ⛔ Recusa medida: a barriga do lado INTEIRO no troço que sobra (dentes e estrela no arredondado + envergadura). Gates: `smooth_leaves_no_corner_bump`, `the_corner_radius_carries_into_the_smooth`, `corner_styles_stay_clean_under_curve_and_smooth` (vermelho provado com cada defeito reposto). Saíram os dois gates da lei recusada (`a_little_smooth_keeps_the_corner_radius`, `with_a_radius_the_smooth_vertex_stays_a_corner…`) |
| (3.º smoke) | **O liso APROVEITA os outros ajustes** (o dono: «Smooth que não aproveita o trabalho dos outros ajustes. Pode fazer aproveitar? se não, melhor tirar»): o `smooth_contour` corre sobre o contorno JÁ FEITO (raio, tipo do canto, envergadura, torção), no espaço da elipse unitária — cada alça roda para vizinho-anterior → seguinte e cresce até `rounded_at_one(m) ×` a corda dela, aos poucos; `0` = nada muda, `1` = todo vértice G1; os pontos nunca se mexem (o entalhe fica). O `shaped` perdeu o liso (o `raw`/quinhão, o G1 da envergadura, o `radius_share` e o `tangent`). O oráculo do Inkscape continua (no polígono cru é a mesma conta). Gate `smooth_builds_on_the_other_adjustments` (4 estilos × 3 envergaduras: sem salto, pontos fixos, G1 a 1). Folha: `/var/tmp/ph2d_F13_poly/smooth_aproveita.png` |
| `110a308292` · `c8c1219a79` | **B — a *Resize Box* MANTÉM a caneta** (o *Scale Strokes & Effects* do Illustrator desligado): `ph2d_app_vec::vec_frame_pen` (`keep_the_pen`, `resize_keeping_the_pen`), chamado pelo `vec_frame_resize::apply` da shell; a receita de uma forma viva segue a razão da GEOMETRIA. Doc 17 §2 |
| `d27c1ae575` | **C** — o ⇕ do *Viewed* (cubo, cone, pirâmide) sobe para «From above»: `HandleDesc::down` (`arrow_down`), só ele inverte |
| `669475ed5a` | **D** — caixa baixa: com `h/4 < park_at` a fila de ⇕ passa à meia altura, a começar um `RAIL_PITCH` por fora da borda (a alça do meio fica na borda) |
| `6403559be4` | **E** — a placa da GPU do Motion corta o bico além do limite (SVG 2 `miter-clip`): `corte_da_esquadria` em `shape.wgsl`, `contorno.wgsl` e no espelho `contorno_cpu.rs`; o alcance da esquadria passa a `√(limite² + 1)`; `PH2D_MITER_CLIP=0` chega à placa pelo SINAL do `limite_esquadria`. Doc 19 §5 (fim) |

## §2 — Medido e decidido (técnico, delegado)

- **A, as hipóteses do prompt:** H1 e H2 confirmadas VERMELHAS pela cadeia do produto antes da cura
  (`shape_mode_handles_tests.rs`); H3 FALSA (o `path_at` já conta o preenchimento: o clique no meio
  de uma estrela cheia selecciona-a); H4: sem `VecShape` não há alças (nada a mexer), os nós ficam
  escondidos (no modo Forma nada os arrasta), a caixa é a da selecção.
- **B, a lei:** a geometria escala o que FALTA à caixa visual (a margem do traço fica de cada lado) e o
  pivô é o ponto que deixa as DUAS bordas exactas. Onde a margem muda com a razão (o bico em esquadria
  de uma estrela esticada só num eixo), uma secante sobre o erro da largura visual: **medido 5–6
  passes** à tolerância `1e-6` da caixa (4 não chegam; o ponto fixo simples precisava de dezenas) ⇒
  `PASSES = 6`. Ficam com a caneta: a largura, os traços da pilha, o ladrilho da estampa do traço, o
  deslocamento e a dilatação das camadas. A escala LIVRE (sem *Resize Box*) não muda (`√|det|`).
- **D, a conta:** o quadrado da alça tem 12 px, o disco de acerto do ⇕ 24 px ⇒ tocam-se quando a
  distância vertical `h/4 < 6 + 12 = PARK_PX` (= `park_at`) — o limiar É a constante que já existia.
  Por fora, o 1.º ⇕ fica a 27 px da borda (o disco começa a 15 px; o quadrado acaba a 6 px).
- **E, o alcance:** as quinas do corte ficam a `√(d² + r²)` do vértice na ida-e-volta (`d = limite·r`)
  — `√17 ≈ 4,12` meias larguras com o limite 4, acima dos `4` de antes. Mudou no shader
  (`alcance_da_peca`) e no `eixo.rs` (o bloco e a extensão), a mesma função.
- **E, a régua dos gates de GPU:** o `pelo_vello` de `tests/it/paridade_com_o_vello.rs` desenhava a
  referência com o `stroke` CRU do Vello (que chanfra). Com o corte, `o_passe_desenha_o_que_o_vello_desenha`
  e `o_tracejado_esticado_desenha_o_que_o_vello_desenha` divergiam; com `PH2D_MITER_CLIP=0` os dois
  passavam ⇒ a diferença era SÓ o bico. Cura: a referência CHAMA o `stroke_uniform` da casa (a lei
  que o render da cena usa). 15/15 na placa.

## §3 — Foundational tocado e contratos

- `ph2d-tool-vector`: `HandleDesc` ganhou o campo `down` (público; os dois construtores literais da
  tabela escrevem-no). `ph2d-vec-render::miter_clip` reexporta `enabled` (a chave do corte para a
  placa). `ph2d-shape-gpu`: nenhuma mudança de layout (`EixoItem` 72 bytes, `Record` 352).
- A shell: `vec_frame_resize.rs` (+14 linhas: o campo `world_scale` e a chamada); a lei vive em
  `ph2d-app-vec`. `the_shell_only_shrinks` com folga.

## §4 — O que só a árvore combinada pega

- A `line/Vector` (se integrar antes): toca o `vec_frame_resize`/a *Resize Box*? Re-correr os gates
  `vec_frame_resize` e o `a_resize_with_the_stroke_in_the_box_keeps_the_pen_and_both_visual_edges`.
- Qualquer linha do Motion que mexa no `shape.wgsl`/`contorno.wgsl`: os gates de GPU do
  `ph2d-shape-gpu` são `#[ignore]` (precisam da placa) — correr `PH2D_GPU=1 … --test it -- --ignored`.

## §5 — Provas de mutação

Agente `mutacao`, sobre `c8c1219a79`, verde antes de cada uma, restauro por `cp` + `touch`, árvore
limpa no fim. Controlos: cada filtro com ≥ 1 teste (9 · 4 · 16 · 35 · 12 · 89); cada âncora casou 1×.
**16 mutações, 15 vermelhas à 1.ª; a H4 sobreviveu e foi curada com um gate** (`c16ff352fc`).

| cláusula | mutação | vermelho em |
|---|---|---|
| A `after_the_shape_release` | sem `shape_armed = false` | `a_click_on_a_shape_disarms…` · `the_scene_selects_with_the_shape_tool…` |
| A `hides_the_nodes` | sem `mode == Shape ‖` | `the_nodes_never_draw_in_shape_mode` · `a_shape_without_params_in_shape_mode…` |
| B `keep_the_pen` largura | apagada | 3 em `vec_frame_pen_tests` + o da shell (`0,4` contra `0,2`) |
| B `keep_the_pen` deslocamento / dilatação | apagados (2 mutações) | `keep_the_pen_keeps_the_layers_width_offset_and_dilation` |
| B a margem no alvo | `lo = alvo` sem a margem | 2 em `vec_frame_pen_tests` + o da shell |
| B a secante | sem passo | `under_world_scale_and_on_a_stretched_mitre_tip…` |
| C `down` | `* 1.0` | `the_viewed_arrow_goes_up_to_from_above` |
| D caixa baixa | o ramo nunca dispara | `no_arrow_touches_a_box_handle_or_the_action_bar_from_20_to_400_px` |
| E o corte no espelho | `if false` | `a_placa_corta_o_bico_alem_do_limite_como_o_oraculo` |
| E o alcance | `√(l² + 1)` → `l` | `um_aberto_com_quina_leva_troços_e_pontas` · `os_blocos_guardam_as_pecas…` |
| `normals_meet` | devolve o ponto de entrada | 3 (`the_four_corner_styles…`, `the_cutout_is_square…`, `a_corner_whose_edges_are_eaten_whole…`) |
| `rounded_at_one` | `1,0` | `smooth_one_is_the_circle_of_the_regular_polygon` |
| `corner_live` tangente de origem | sem ela, cada lado (2 mutações) | `a_corner_whose_edges_are_eaten_whole_keeps_its_style` (⚠️ um só teste guarda os dois lados) |
| **`liso_em`** (G1 só sem raio) | `lisa && open ≥ 1` → `lisa` | **sobrevivia** (nenhum teste via o raio a sumir sob liso + envergadura) → gate `with_a_radius_the_smooth_vertex_stays_a_corner_and_the_radius_shows`: medido, o efeito do raio `0,25` cai ~5× com a mutação (`0,0287 → 0,0055`); vermelho confirmado à mão |

O WGSL não tem mutação própria (os gates dele precisam da placa): o espelho CPU é a mesma conta, e
os 15 de GPU passam com o corte e divergem do Vello cru sem ele (§2).

## §6 — Smoke (do dono)

O relatório ao dono (passos numerados) vai na conversa; as fotos da cena 20:
`/var/tmp/ph2d_F13_cena20/1_estrela.png` (a estrela seleccionada pelo clique: alças verdes, nenhum
nó) e `2_elipse.png` (a selecção passou à elipse).

## §7 — O que fica ABERTO

1. **Para o dono decidir (achados da auditoria do Polígono, doc 15 §3b, sem cura):** o *Cutout* no
   tecto deixa lascas de espessura zero (o rectângulo 2:1 fica um quadrado com dois «bigodes»); a
   seta dobrada com *Tail* > *Head width* perde as farpas (o OOXML prende a haste à cabeça); o *Check*
   com peso ≥ 0,5 mostra um degrau no topo do braço curto. Vão no relatório do smoke como perguntas.
2. D com TRAÇO GROSSO: a fila «por fora» mede-se da borda da CURVA; as alças da caixa vivem na caixa
   VISUAL. Com meia largura de traço ≥ 9 px no ecrã o 1.º ⇕ por fora encosta na alça do meio (medido
   pela conta: 27 − 12 − 6 = 9). O `handle_grip` não vê o traço (só o cozinhado) — curar pede passar a
   margem visual ao `view` de `shape_handles_live`.
3. A integração das DUAS linhas do Vetor (esta e a `line/Vector`) — por ordem do dono.

## §8 — Portão, binário e `agent-loop-profile.sh`

- **Portão batched** (agente `verificador`, 1× sobre `b1dd77e477..c16ff352fc`, loadavg 16–19):
  `nextest-impacted` (14 crates mudadas + dependentes, `tests/it` incluídos) **19 936 / 19 950**, 8 603
  pulados — os 14 que não passaram são TODOS da Luz 2D (ver abaixo) · clippy do `ship.sh`
  (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) **exit 0** ·
  `fmt --all --check` ✗ em 11 ficheiros desta linha → curado em `b42b803554` (re-corrido: limpo) ·
  `workspace_src_files_under_loc_cap` ✓ · `the_shell_only_shrinks` ✓ ·
  `architecture_claude_md_cabe_no_orcamento` 3/3 ✓ (a entrada UI/UX: 698/700 bytes) ·
  `the_shape_handles_are_drawn_and_dragged` 2/2 ✓ · `every_door_the_names_enter_by_translates_them` ✓ ·
  GPU `ph2d-shape-gpu --test it -- --ignored` **15/15**.
- ⛔ **Os 14 da Luz 2D estão VERMELHOS NO `main`** (`b1dd77e477`, corridos no primário com
  `PH2D_GPU=1`, sem esta linha): `ph2d-light2d-gpu` — `c3_grade_fina::{so_a_regiao_se_calcula…,
  o_interior_de_um_opaco…}`, `g5_o_que_a_placa_alocou…`; `ph2d-app-light2d` — `as_formas_redondas::
  {o_disco_de_imagem…, o_interior_com_o_rebote…, as_formas_redondas_nao_saem_em_facetas}`,
  `o_interior_do_opaco::{o_rebote_nao_depende_da_oitava, o_interior_do_opaco_nao_depende_da_posicao…}`,
  `o_miolo_das_formas…`, `o_traco_do_flip::{…recebe_a_luz…, a_cena_7…}`, `smoke_cenas_w4::{a_cena_5…,
  a_cena_6…}` — as mesmas mensagens nos dois lados (o `holografica::a_holografica_junto_a_uma_face…`
  só estoura o prazo sob carga: sozinho passa em 2,7 s). Não reproduzem carga (sozinhos, loadavg 1,26).
  Não são desta linha (o `ph2d-light2d-gpu` não depende de nada que ela mexeu além do `ph2d-i18n`);
  são para a linha Luz2D / o integrador — o handoff W5m dela não os menciona.
- `bash scripts/doc-index.sh` ✓ · `rm -rf target/*/incremental` (16 G + 4,6 G).
- **Binário de smoke** (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`),
  2.ª corrida depois da limpeza, HEAD `b42b803554`:
  `Finished \`smoke\` profile [optimized] target(s) in 0.52s` (zero `Compiling`).
- Fotos da cena 20: `/var/tmp/ph2d_F13_cena20/` (as duas paragens; nenhum nó no modo Forma).
- `agent-loop-profile.sh` (20 sessões): paralelismo 1,14 ✗ · respostas/sessão 164 ✓ · test:check
  4,2 ✗ · Edit 43 % ✗ · contexto/passo 347 mil ✗ · início 63 mil ✓.

## §9 — Integrado no `main` (12/10, por ordem do dono: «Integre a linha ao main»)

- Rebase sobre o `main` `bbdb80cc93` (a integração do MiroClone W12–W19 por cima da Luz2D W5m): 82
  commits; 5 conflitos, TODOS no `CLAUDE.md` §5.1 (a entrada UI/UX ao lado da do MiroClone),
  resolvidos pelos estágios — o `:2` inteiro com a entrada UI/UX do `:3` (script que recusa se o `:3`
  mudou outra linha). `range-diff`: só esses 5 commits mudaram, e só no contexto. Mergiraf não
  interveio. Superfície partilhada sem conflito de texto: `Cargo.lock`, `svg_export.rs`,
  `store_core.rs`, `board_view.rs`, `hero/paint.rs`, `ph2d-i18n`, `ph2d-vector`, `tests/it/main.rs`.
- **Semântico apanhado pelo portão** (`check --workspace --all-targets`, avisos negados): o
  `vec_convert_tests.rs` da shell lia o id devolvido pelo `ShapeTool::on_press`, que a correcção do
  clique (`04aea82470`) tirou → lê o `selected()` no soltar (`07674c067d`). Era desta linha (o
  `check` estreito não compila os testes), não do `main`.
- **Árvore suja do primário recolhida:** o `PROMPT_LINHAS_RAPIDAS_2026-10-11.md` vivia solto em
  `docs/IntegracaoMultiAgente/` (citado por handoffs de cinco linhas e pelo `Cargo.toml`, nunca
  commitado) → commitado e, pelo gate `the_live_process_folder_holds_no_dated_record`, movido para
  `docs/archive/integracao-jornadas/`, com as 13 referências por caminho (sha256 igual ao solto).
- **Portão da árvore combinada** (`nextest-impacted`, `ci-test`): 20 145 corridos, 20 131 verdes; os
  14 restantes = o do registo datado (curado acima, re-corrido 8/8) + os **13 da Luz 2D herdados**
  (`ph2d-app-light2d` ×10, `ph2d-light2d-gpu` ×3), vermelhos no `main` sem esta linha (corridos no
  primário, mesmas mensagens; o handoff do MiroClone já os lista ao bit). Por isso o `--ff-only` foi
  feito à mão (o `foundational-integrate.sh` só junta com o impactado verde).
- `main` → `6b13c519f4`. §5.1 UI/UX: «Último: F13, no main 12/10».
