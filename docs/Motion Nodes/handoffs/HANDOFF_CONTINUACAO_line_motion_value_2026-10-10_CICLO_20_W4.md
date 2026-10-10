═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Sua linha: line/motion-value · worktree Worktrees/line-motion-value/ (JÁ EXISTE — não crie)
Base: ciclos 13–20 em `line/motion-value`, REBASEADA no `main` `782f90b69` (10/10), POR INTEGRAR.
O ciclo 20 (O ACABAMENTO, [doc 129](../129_ciclo_20_o_acabamento.md)) está ABERTO: W0–W3 fecharam
(a lente `fx.lens`, smoke do dono APROVADO 10/10; os filtros da camada `fx.filter`, commit
`5f57ae248`, doc 129 §5.4). Você abre a **W4 — a cena, as fotos, o relógio e o tutorial 13**.

FASE 0: cd Worktrees/line-motion-value && pwd && git branch --show-current && git log --oneline -6 &&
        git status --short — ANTES de abrir qualquer ficheiro. (`git cherry main HEAD` > 0; não
        rebaseie de novo sem o `main` ter andado.)

⛔ Fale com o Enio em PT-BR, curto, sem jargão (CLAUDE.md §0.8). Integrar/pushar só por ordem dele.
⭐ LEIA o doc 129 §4, §5.3–§5.4 e §7 (curtos). CUSTO: leitura larga a `explorador`; gate batched a
   um `verificador`.

O QUE A W3 DEIXOU (não reconstrua):
- `crates/ph2d-node-fx-filter` (o nó; `from_graph` = a pilha do fio; `fora_do_fio`;
  `COMPRIMENTO_MAX = 0,6` MEDIDO = a saturação do passe na câmera do produto), `FxOpGpu::resolve`
  (`ph2d-render/src/fx_stack_op.rs`, porta única mundo → pixel, o Vector chama-a),
  `FxStackPass::run_premultiplied` (`src_premul` no `Globals`), `ph2d-app-motion/src/motion_filter.rs`
  (`MotionFiltro` prepara/liga/queixa; `MotionState::sprites_da_camada` — a MESMA pergunta para o
  passe fundido e a camada —; `MotionState::filtra_a_camada`, chamada em `present_chrome.rs` no lugar
  da colagem da placa e antes da lente).
- A cena `=135` JÁ tem `Tint → Filter (Drop Shadow) → Lens → Output` e o roteiro (0) do filtro;
  fotografada a 1600×972 (sem filtro = a cena de sempre; com = a sombra em cada disco).
- Gates: `ph2d-render/tests/it/fx_stack_premul_gpu.rs` (`PH2D_GPU=1`, `--ignored`): pré-mult ≡ reta
  (≤ 1 nível, controlo > 20) e o borrão contra o ImageMagick (1,19 nível, controlo σ 2,5 = 16,65);
  `motion_filter_tests.rs` (a régua contra o `MAX_HALF`, Y para baixo, o tonemap em bypass).

A W4 (doc 129 §4):
1. **As FOTOS com o controlo** (`docs/Components/ferramentas/fotografa_cena.sh`, nunca `spectacle`):
   cada passo do roteiro da `=135` fotografado — o (0) do filtro (Drop Shadow · Blur · Glow com cor
   clara · Outline) e os (1)–(5) da lente. ⚠️ Os eventos sintéticos não chegam: mude o param na
   CENA (uma variante por foto) ou prove o clique num gate de costura.
2. **Medir o que o §7 deixou aberto:** (a) as formas da rota da CPU (`formas_pedem_o_vello`) numa
   cena com filtro — acontece? se sim, a camada não as filtra (decida e gateie); (b) o RELÓGIO do
   grupo em `--release` com o `loadavg` ao lado (MEDIR_VELOCIDADE.md: variantes no MESMO processo,
   intercaladas, o MÍNIMO): a lente (16 nós × 2 `Φ` por pixel) e a camada filtrada correm na JANELA
   INTEIRA — meça a 4K; se custar, o sub-rect da cena (`run_from` já aceita origem) é a cura.
3. **O tutorial 13** (`tutoriais/src/13_o_acabamento.html` → `bash scripts/tutorial-pdf.sh`) com o
   gate dos passos (cada row/opção que o texto nomeia existe no cartão — o padrão de
   `every_row_the_script_names_is_on_the_filter_card`). Diga nele que a camada filtrada fica POR
   CIMA do mundo (sai do Z intercalado com as sprites).

DEPOIS (W5): mutação (o sentido da aberração, a quadratura, a ordem fixa, o tique do grão, a
neutralidade byte-idêntica, a recusa da LUT, o quadro VISÍVEL; e da W3: o `src_premul` do ingest, a
ordem do fio, o gate compacto, a régua do `COMPRIMENTO_MAX`, o `sprites_da_camada` com filtro),
gate batched, handoff `HANDOFF_INTEGRACAO_..._CICLO_20.md` + §5.1 do CLAUDE.md.

ABERTOS NOMEADOS: [BUGS #19](../BUGS_motion_nodes.md) (o ângulo do `fx.glow` gateado ao contrário) ·
a lei de mistura do `fx.filter` sem ficha e a rampa com 2 stops (doc 129 §7) · do ciclo 19: a fita
acima de 32 MiB · uma linha de COR inteira num gesto · vida e dano pedem dois objectos ·
`Bezier`/`Tint` OKLCH caros na CPU de reserva · gate de gesto pelo caminho REAL do rato (BUGS #18).
═══════════════════════════════════════════════════════════════════
