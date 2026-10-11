# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 11/10 — F12 a meio (sub-onda A, 1.ª parte)

> A janela fechou pela R1. Regras R1–R9: topo do
> [handoff da F10](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md). Prompt da onda:
> [`18_prompt_F12_…`](../vetor_nova_ui/18_prompt_F12_sem_modais_e_o_traco.md) (leia-o: o pedido do
> dono palavra por palavra e as sub-ondas A–G). Lei, oráculo e recusas desta parte:
> [`19_o_traco_das_formas.md`](../vetor_nova_ui/19_o_traco_das_formas.md).

## Onde a linha está

- `line/UIUX`, `Worktrees/line-UIUX`, merge-base = `main` `8f20c60de` (o `main` não andou).
  F9 + F10 + F11 fechadas e NÃO integradas; F12 em curso: commit `24c7f2ea7`, árvore limpa.
- Fotos do dono: `/var/tmp/ph2d_F12_dono/` (fora do repo; a leitura MEDIDA de cada uma está no doc 19 §1).

## O que a 1.ª parte entregou (técnico)

- **O bico cortado** (`miter-clip` do SVG 2), uma lei em `ph2d_vec_scene::miter_clip` e cinco
  leitores: `ph2d_vec_render::stroke_uniform` (`cut_outline`; só com `PathTess::agudo` — pago por
  geometria — e no ramo não conforme), `visual_bounds`, `ph2d-shape-gpu::contornos_dos_tracos`,
  `ph2d-vec-boolean::penned_outline` (*Outline Stroke*), `svg_export` (`miter-clip`).
  `PH2D_MITER_CLIP=0` = o chanfro de antes. `MITER_LIMIT` continua `4`.
- **Polígono/estrela** (`polystar.rs`): o *Curve* na lei das alças a ⅓ da corda (a de antes, alças
  perpendiculares, era o «X» da foto 1), a rotação limitada a metade da quina (`interior_angle`); o
  *Smooth* `0..1` (`shapes_fields::SMOOTH_VERTS`); o gate de paridade com o Inkscape salta o `−0,25`.
- **O censo** `ph2d-tool-vector/src/stroke_census_tests.rs` — duas sondas `#[ignore]`
  (`print_the_stroke_census`, `print_the_pair_census`: correm com `-- --ignored --nocapture`) e três
  gates (zero laços em 1D em toda forma + polígono sem nada nos pares; toda junta aguda com tinta
  no corte; o topo igual ao `resvg` ±1 px pela régua dele). Testes das 7 crates tocadas verdes
  (verificador, load 60); clippy `-D warnings` limpo em vec-scene/vec-render/vec-boolean/
  tool-vector/shape-gpu.
- Foundational apenso: `ph2d-vector` reexporta `dash`; `ph2d-shape-gpu` depende do
  `ph2d-vec-render` (antes só nos testes); `PathTess` ganhou `agudo` (crate-privado).

## O que FALTA, por ordem

1. **A2 — a cauda do censo** (rode as duas sondas primeiro; números de hoje):
   - estrela nos PARES: *Points × Smooth* (≥ 7 pontas, liso ≥ 0,6) e *Inner × Smooth* (inner 0,05)
     dão laço — as alças do liso de vértices vizinhos cruzam-se (o Inkscape faz o mesmo). Cura
     proposta: limitar o comprimento da alça POR ARESTA para os dois segmentos-alça não se cruzarem
     (G1 mantém-se; a paridade de 5 pontas/hexágono não cruza, fica exacta). *Notch round × Curve*
     côncavo (raio > 0, `bend_between_corners`) também laça; *Twist* extremo (±54°).
   - ganchos noutras formas (a régua: numa QUINA, a ponta vira > 30° em meia largura): balões
     (`Speech`, `Oval say`, `Thought` — as junções do rabo), `Moon` fase < 0,12, `Cloud` aspecto
     < 0,54, `Pie/Oval` furo ~0,06, `Round` tipo *Concave*, `Gear` curvatura −0,5 (laço).
     Investigar forma a forma no `cook` dela, depois apertar o gate (hoje só Poly/Star nos ganchos).
   - A placa da GPU do Motion para cópias NÃO conformes tem traçador próprio
     (`ph2d-shape-gpu/src/contorno_cpu.rs` + o shader, `limite_esquadria`) e ainda SALTA para o
     chanfro. Mesma lei a portar (CPU mirror + WGSL) — é do motor do Motion: decida se cabe aqui.
2. **A cena 18 e as fotos ANTES × DEPOIS** (a decisão do miter não é nossa: o prompt manda mostrar
   ao dono): seta dobrada em duas alturas, coração em dois *Cleft*, pentágono com *Curve* e
   *Smooth* das fotos; fotografar com e sem `PH2D_MITER_CLIP=0` (`vector_mode.rs` conta as cenas;
   `fotografa_cena.sh`; ⛔ nunca `spectacle` em kwin virtual).
3. **B** polígono: o ponto da curva da aresta de volta (`Spot::EdgeMid`), *Curve*/*Smooth* como ⇕.
4. **C** rectângulo: o *Smoothing* SAI (campo 4 do `RoundRect`, a fila ②, o cartão; o save:
   confira `PROJECT_SCHEMA` — F10 não está no `main`, logo nenhum save do dono o tem); *Corner* ⇕.
5. **D** círculo/pizza: o furo de volta no canvas; o raio das quinas do arco aberto (parâmetro NOVO
   apenso, neutro em zero, se o `ellipse_sweep` não o tiver).
6. **E** espiral: reproduzir o gizmo que «não funciona» pelo press/arrasto da shell; *Expansion* e
   *Inner* ⇕.
7. **F** todas as formas com cartão: cada campo vira ⇕ na LATERAL da caixa, lado a lado, com a ficha
   do nome no arrasto; no fim `fields_without_handle` vazio e o cartão some.
8. **G** cenas refeitas e fotografadas; fecho pela DIRETRIZ §1.5.9 (mutação ANTES do portão:
   as cláusulas do `clip_patch` — o limite, o lado, a recta de corte —, a orientação `oriented`,
   o `agudo`, o tecto da quina do `polystar`, o `chord`).

## Perguntas ao dono (no relatório desta janela)

- O bico cortado muda toda quina mais aguda que ~29° em todos os desenhos (antes: chanfro). Ele
  aprova pelo antes × depois (item 2). A pergunta da F11 (*Resize Box* e a espessura) segue aberta.

## Para a próxima janela

`cd Worktrees/line-UIUX && pwd && git branch --show-current` antes de ler; leia este handoff, o
doc 19 e o prompt 18 — não o histórico.
