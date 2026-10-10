═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: ciclos 13–17 em `line/motion-value` sobre o `main` `2a925859d`, POR INTEGRAR. O ciclo 17
(O RITMO, [doc 126](../126_ciclo_17_o_ritmo.md)) fez os sete passos; ⏳ **o smoke do dono** (cena
`PH2D_GPU_COOK_DEMO=132`, [tutorial 10](../tutoriais/10_o_ritmo.pdf)) ainda não foi reportado.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -8 &&
        git status --short — ANTES de abrir qualquer ficheiro.

⛔ Fale com o Enio em PT-BR, curto, sem jargão (CLAUDE.md §0.8).
⭐ A OBRA: [doc 126 §0](../126_ciclo_17_o_ritmo.md) — ordem do dono (09/10): *«só o ponto 1 [export
   de vídeo] não me interessa. Vamos fazer os demais»*. Sete ciclos, 17 → 23, na fila do
   [doc 103 §5](../103_dinamica_dos_ciclos.md). O protocolo de cada um é o doc 103 §1 (os sete passos).

O QUE FAZER, NESTA ORDEM:
1. Se o dono reportar o smoke do 17 com defeito: corrija PRIMEIRO (mecanismo, medição, gate —
   `/pd-bug-causa`), depois siga.
2. Abra o **ciclo 18 — A COR QUE O OLHO VÊ** (doc novo `127_ciclo_18_a_cor_que_o_olho_ve.md`):
   OKLab/OKLCH nos nós `motion.color_ramp` · `motion.color_array` · `motion.tint` (e as misturas de
   cor). A matemática já existe e é folha: `ph2d-color/src/oklab.rs` (`from_linear`, cbrt),
   `oklch.rs`; o Vector já a usa (`ph2d-app-vec/src/contour_live.rs:389-401`). Os nós hoje
   interpolam em RGB linear (`ph2d-node-motion-color-ramp/src/lib.rs:17`,
   `ph2d-node-motion-tint/src/lib.rs:14`, que diz «OKLab é um futuro refinamento, cbrt-gated»).
   Desenho já decidido: param `space` APENDADO (RGB linear · OKLab · OKLCH curto · OKLCH longo), o
   default de um documento CARREGADO é o de sempre (ao bit), nó NOVO nasce em OKLab; no WGSL o
   `cbrt` é `sign·pow(|x|, 1/3)` ou um polinómio por bits como os da `ph2d-ease` — MEÇA os dois
   contra a CPU antes de escolher (a lição do 17: a divisão do WGSL tem 2,5 ULP, e o `pow` não
   tem garantia; o oráculo pode ser o próprio `ph2d-color` em `f64`, ou o Björn Ottosson de
   referência corrido em Python sobre as nossas cores).
3. Depois os ciclos 19 (O JOGO MANDA — `value.input` sobre o `cook.set_external` já existente,
   `ph2d-nodegraph/src/external.rs`) … 23, como no doc 126 §0.

⭐ DEPOIS DO SMOKE (09/10, «smoke OK, mas o ease deveria ser com gráficos como na timeline»): o
   `Ease` virou GRÁFICO — `ParamWidget::Ease`/`EasePart`, editor `ph2d-param-editors::ease`
   ([doc 126 §6](../126_ciclo_17_o_ritmo.md)). ⏳ O dono ainda não viu o editor aberto: no smoke
   seguinte, confirme com ele a janela (miniaturas + alças). Todo nó novo com curva de easing usa
   os nomes de `ph2d_ease::nomes` e ganha o gráfico de graça.
⛔ Uma janela flutuante do cartão tem de caber nos `205,5 px` de altura do painel do grafo (medido;
   o gate `the_window_paints_every_tile_and_the_shape_handle_inside_the_panel` mostra como medir).

LIÇÕES DO 17 QUE VALEM PARA OS SEGUINTES (não as repague):
- ⛔ **A régua da cena lê a SAÍDA** (o que se desenha), nunca um nó do meio: o `Duplicator` tem
  `point_scale = 0` por omissão e engolia a onda inteira com o `Stagger` a variar (doc 126 §5.3).
- ⛔ **Uma foto num instante prova o instante:** numa cena com um ciclo, tire o lote de fotos em
  tempos diferentes (`fotografa_cena.sh … 7` / `… 8`) e escolha a que mostra o fenómeno.
- ⛔ **WGSL: divida por `pe_div`** onde a lei tem SALTO, e escreva constantes por BITS
  ([memória](../../../project-memory/feedback_wgsl_division_is_not_ieee_and_a_jump_amplifies_it.md)).
- ⛔ `from` é palavra reservada no WGSL; o `ParamGate` de um param SOMA-SE aos outros dele (o gate
  `a_gated_row_is_hidden_outside_its_modes_and_shown_inside` já o sabe); num nó com `channel`, nenhum
  param se declara `ParamUnit::Length` (censo `no_param_of_a_channel_driven_node_is_declared_a_fixed_length`).
- ⛔ Um kernel cuja leitura de coluna é OPCIONAL declara-a numa VARIANTE (`variant_by_param`), nunca
  no topo: o diagnóstico lê o topo e acusa `MissingSource` falso (a `=76` com o `value.time` solto).

ABERTOS DO 17 (nomeados, não esquecidos):
- O `Bezier` na CPU custa `5,1 ms` a 32 761 linhas (24 bissecções em cadeia); um Newton com a
  mesma contagem nas duas línguas cortaria-o. Só a reserva paga isto (a placa: `0,15 ms`).
- O `value.map_range` ficou com os seus `Interp` (Linear/Stepped/Smooth/Smoother): são outra
  pergunta (a forma do remapeamento), não a família de easing — não foram fundidos.

INTEGRAR / SHIPAR: só por ordem do Enio (CLAUDE.md §0.7). Ao fechar a linha: `/pd-linha-fechar`.
═══════════════════════════════════════════════════════════════════
