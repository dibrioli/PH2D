# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 11/10 — F12 depois da 2.ª janela (A2 + A3 feitas)

> Regras R1–R9: topo do [handoff da F10](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md). Prompt:
> [`18_prompt_F12_…`](../vetor_nova_ui/18_prompt_F12_sem_modais_e_o_traco.md). Lei, censo, curas e
> recusas: [`19_o_traco_das_formas.md`](../vetor_nova_ui/19_o_traco_das_formas.md) §5. A 1.ª janela:
> [F12](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F12.md).

## Onde a linha está

- `line/UIUX`, `Worktrees/line-UIUX`, merge-base = `main` `8f20c60de` (o `main` não andou).
  F9–F11 fechadas e NÃO integradas; F12: A1 `24c7f2ea7`, A2 `fe21fef57e`, A3 (cena 18) no commit
  seguinte; a B por começar. A janela fechou pela R1.

## O que a 2.ª janela entregou (técnico)

- **A2 — a cauda do censo** (doc 19 §5): `ph2d_vec_scene::untangle` (a forma pára no último
  desenho sem cruzamento; um período por simetria), usado por `polystar` e `gear_ex`; o liso com
  *Curve* é G1; o rabo dos balões (bico em V pela medida `cos·l/corda`, raiz na faixa, `TIP_FLOOR`);
  a seta dupla sem haste = losango; a régua do gancho em `min(½ caneta, ⅛ troço)` e a caneta no lado
  curto da caixa. Gate `no_cooked_shape_loops_or_hooks` para TODAS as formas × campos × pares.
  Suíte `ph2d-vec-scene` 530/531 verde (o 531.º, a robustez, ver «Aberto» 1).
- **A3 — a cena 18** (`vector_mode_tracos.rs`, estágios 100–103; o 102 espera um quadro para o
  barramento restilar a selecção antes de a limpar): seta e coração em caixa larga e alta/estreita, quadrado *Curve* 0,010, pentágono *Curve* 0,070, lua, balão; traço 16 px. Fotos em
  `/var/tmp/ph2d_F12_cena18/` (`depois_bico_cortado.png`, `antes_chanfro.png` com `PH2D_MITER_CLIP=0`,
  `lado_a_lado_antes_esq_depois_dir.png`). Leitura e as três opções ao dono: doc 19 §5.
- **Depois da R1, dois pedidos do dono** (11/10): (1) o rectângulo de raio minúsculo sob traço
  grosso saía com o canto em chanfro — `ph2d_vec_render::miter_clip::tighter_than` (curvatura < ½w)
  manda essas geometrias pelo contorno exacto (`90eebf7b38`; a cena 18 ganhou o rectângulo);
  (2) no modo Forma um CLIQUE numa forma existente selecciona-a, arrastar desenha
  (`shape_live::after_the_shape_release`, `2bb2954b82`). Binário smoke compilado.
- A placa da GPU do Motion (`contorno_cpu.rs` + 2 shaders) NÃO entra nesta linha: só depois de o dono
  aprovar o bico cortado (doc 19 §5).

## O que FALTA, por ordem

1. **A resposta do dono** ao bico (doc 19 §5, opções a/b/c). Se (b) ou (c), a lei muda em
   `ph2d_vec_scene::miter_clip` (uma porta, cinco leitores) e o fixture do resvg a 0° sai do gate.
2. **A robustez** `robust_tests::cook_survives_every_slider_position_and_every_gesture_box`: com a
   guarda, as estrelas aleatórias que se cruzam cozinham ~11×; 74 s medidos a `load 188` (inválido).
   Medir em calma contra a base (`untangled` → `build(1.0)` só para a medição) e, se passar a R9,
   curar (o custo é o `shaped` com cantos arredondados, não a procura).
3. **B** polígono: devolver `on_sharp(h(&[3], Spot::EdgeMid(0), Law::Nearest), &[1])` (a linha que a
   F11 tirou, `21226cb088`); `Law::Steps` também para campos CONTÍNUOS (a altura da caixa = a faixa);
   *Smooth* (4) e o tipo do canto (2) como ⇕ ao lado das Pontas (`Spot::Box { u: 9 | 10, v: 2 }`); o
   censo `the_fields_without_a_gesture_are_counted_not_listed` perde a linha do Polygon.
4. **C–G** como no prompt 18 e no handoff da 1.ª janela (itens 4–8).
5. Fecho: mutação ANTES do portão — as do handoff da 1.ª janela e: `untangle` (o `MARGIN`, a
   quase-cúspide, o período), o G1 do liso, `NEEDLE_COS`, a raiz na faixa, `TIP_FLOOR`, o losango, a
   régua `⅛ troço`.

## Perguntas ao dono (no relatório desta janela)

- O bico cortado, pelas fotos da cena 18 (antes × depois). ⚠️ A mudança mais visível: as pontas que
  voltam 180° (chifres da lua, bico do rabo) ganham um toco de 2 larguras.
- A estrela (aprovada) só mudou onde se cruzava ou enganchava (liso + *Curve* juntos).
- A F11 (*Resize Box* e a espessura) segue aberta.
