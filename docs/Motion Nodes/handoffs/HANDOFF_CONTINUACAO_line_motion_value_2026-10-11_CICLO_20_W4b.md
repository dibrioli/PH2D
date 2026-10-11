═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: ciclos 13–20 até a W3b no `main` (11/10, [handoff](HANDOFF_INTEGRACAO_line_motion_value_2026-10-11_CICLO_20_W3b.md));
a W4 fechou NA LINHA (commits `ce01e3114`, `6c391500c` e o do handoff, por integrar). O ciclo 20
(O ACABAMENTO, [doc 129](../129_ciclo_20_o_acabamento.md)) continua ABERTO: smoke da W3b APROVADO
pelo dono (11/10). Você abre a **W4b — a mistura do `Output` com a camada filtrada**, e depois a W5.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -6 &&
        git status --short — ANTES de abrir qualquer ficheiro. (`git cherry main HEAD` > 0: os
        commits da W4 são seus e não estão no `main`; rebaseie só se o `main` andou.)

⛔ Fale com o Enio em PT-BR, curto, sem jargão (CLAUDE.md §0.8). Integrar/pushar só por ordem dele.
⭐ LEIA o doc 129 §3.5 (o desenho da W4b), §5.7 (o que a W4 mediu) e §7. CUSTO: leitura larga a
   `explorador`; gate batched a um `verificador`.

O QUE A W4 DEIXOU (não reconstrua):
- `ph2d_render::retangulo_da_camada` + `FxStackPass::run_layer_rect` + `BandBlit::blit_rect`
  (`crates/ph2d-render/src/fx_stack_camada.rs`): a pilha da camada do Motion só na área da cena
  alargada pelo que ela lê e espalha, a origem na grelha da pirâmide. 4K: 2,01 → 0,77 ms. Gate
  `ph2d-render/tests/it/fx_stack_camada_rect_gpu.rs` (`PH2D_GPU=1`, `--ignored`): 0–1 nível na cena,
  controlo sem margem 87–197.
- `PH2D_ACABAMENTO_PASSO=<0|1|2|3|4|5|5b|6|7a|7b|7c|7d|8|mistura>` na `=135` (cada passo do
  roteiro escrito pelo gesto do cartão — a porta das FOTOS; `mistura` põe o `Output` em `Add`), gate
  `every_photo_variant_is_the_step_it_names`. Fotos: `docs/Motion Nodes/handoffs/fotos_W4/`
  (`bash /…/fotografa_cena.sh "PH2D_GPU_COOK_DEMO=135 PH2D_ACABAMENTO_PASSO=<p>" 1600 972 <png> 12`).
- O disco da `=135` tem raio `0,2` (os vizinhos tocam-se — o passo (1) vê-se).
- `formas_fora_da_camada` (`motion_filter.rs`): o cartão de cada `Filter` diz quando formas do
  Motion foram ao Vello (a frase `app.motion.motion_filter.shapes_off_the_layer`).
- O tutorial 13 (`tutoriais/src/13_o_acabamento.html` → PDF) com os gates
  `every_name_the_finishing_tutorial_puts_in_code_is_on_screen` e
  `every_figure_of_the_finishing_tutorial_exists`; o PDF do 07 regenerado.
- O relógio `relogio_cada_forma_contra_camada_inteira` (`ph2d-fx-cells`, `--release`) mede 1600×972
  e 4K, a camada no sub-rect e a lente.

A W4b (doc 129 §3.5 — leia-o INTEIRO antes de codar):
1. **A POUSA com modo**: um passe que compõe a camada filtrada sobre uma CÓPIA do mundo com `f`
   (Add/Multiply/Screen, a lei de `ph2d-blend-mode` / `layer_composite.wgsl`, no espaço do
   desenhista — o Vello mistura em sRGB codificado), no retângulo da camada. Oráculo: a cena do
   `a_mistura_em_grupo_pinta_a_tabela_do_doc_118` (`motion_shape_mistura_gpu_tests.rs`) — o `Scene`
   COM um filtro de pilha neutra tem de sair igual (≤ 1 nível) ao de hoje sem filtro.
2. **Com filtro, o sink não agrupa**: `lower_group_onto` só sem filtro (`ph2d-eval-motion/src/lib.rs`
   ~509), a recusa `RECUSA_MISTURA_EM_GRUPO` (`motion_bridge_gpu.rs` ~293) cai para sinks filtrados,
   a `MisturaDoSink` vai à pousa. `Scene` fica exacto.
3. **`Copies`/`Everything` e o `Blend` por linha**: as cópias misturam-se entre si DENTRO da camada.
   A proposta do doc é o Vello DENTRO da camada (a cena vectorial do Motion sem o mundo por baixo,
   num `Rgba8Unorm` colado com `BandSource::Vector`) — meça antes de escolher; o `Each Shape` sobre
   conteúdo Vello não tem células (o cartão di-lo). A geometria recusada entra pelo mesmo caminho.
4. Dois sinks com modos diferentes partilham UMA camada: meça e decida (camada por sink, ou a queixa).
5. Quando a cura cobrir um caso, a frase `shapes_off_the_layer` deixa de aparecer para ele (o gate
   tem de o dizer), e o tutorial 13 perde a nota «Mistura e filtros» (ou passa a ensiná-la: uma
   variante `mistura` fotografada com o relevo).

DEPOIS (W5): mutação (o sentido da aberração, a quadratura, a ordem fixa, o tique do grão, a
neutralidade byte-idêntica, a recusa da LUT, o quadro VISÍVEL; da W3: o `src_premul` do ingest, a
ordem do fio, o gate compacto, o `sprites_da_camada` com filtro; da W3b: a vala das células
(`stack_read`), a partição, o `switch_kind` «não tocado», a fronteira da cor, a semente de G&S e a
distância à tangente, o `+ ½` e a folga da pirâmide, o `gauss(k + frac)`, o mapa da origem; da W4: a
margem do `retangulo_da_camada` (ler × espalhar), o alinhamento à pirâmide, o `Ponto` trazido ao
retângulo, o viewport do `blit_rect`, o `formas_fora_da_camada`, os gates do tutorial (um nome que
não está no cartão TEM de reprovar)), gate batched, handoff `HANDOFF_INTEGRACAO_..._CICLO_20.md` +
§5.1 do CLAUDE.md.

ABERTOS NOMEADOS: [BUGS #19](../BUGS_motion_nodes.md) (o ângulo do `fx.glow` gateado ao contrário) ·
a lei de mistura do `fx.filter` sem ficha e a rampa com 2 stops · o retângulo da camada usa o
`scene_viewport` e não a área VISÍVEL (`cena_visivel`, a da lente) — mais barato, por medir · do
ciclo 19: a fita acima de 32 MiB · uma linha de COR inteira num gesto · vida e dano pedem dois
objectos · `Bezier`/`Tint` OKLCH caros na CPU de reserva · gate de gesto pelo caminho REAL do rato
(BUGS #18).
═══════════════════════════════════════════════════════════════════
