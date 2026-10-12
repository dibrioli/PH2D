# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 12/10 — F13 a meio (o A curado; o Polígono auditado)

> As regras R1–R9: topo do [handoff F10](HANDOFF_CONTINUACAO_line_UIUX_2026-10-11_F10.md). O pedido e o
> escopo A–E: [`22_prompt…`](../vetor_nova_ui/22_prompt_F13_selecionar_no_modo_forma_e_as_respostas.md).
> O que a F9–F12 entregaram: [handoff F12](HANDOFF_INTEGRACAO_line_UIUX_2026-10-12_F12_SETAS.md).
> Esta janela parou pela R1 (~130 chamadas). NADA integrado; NADA pushado.

## Onde a linha está

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, rebaseada em 12/10 sobre o `main` `b1dd77e477`
  (62 commits, sem conflito, Mergiraf não interveio). HEAD depois deste handoff: o commit dele.
- `PROJECT_SCHEMA` não mudou. Árvore limpa (só `assets/sprites/` e `scripts/__pycache__/` ignorados).

## O que esta janela fez

1. **Pedido NOVO do dono, a meio da janela** (12/10, palavra por palavra): *«Em Shape:Poly: veja se
   cutoff está correto pois está estranho. Smooth parece invertido (smooth no mínimo tem maior
   efeito) — confira. E faça auditoria nas outras que podem ter problemas»*. Medido, curado e
   escrito no **doc 15 §3b** (a tabela defeito · causa · cura · gate, fotos antes/depois em
   `vetor_nova_ui/fotos/f13_0*`). Commits `95da41c20c` + `21b4235311`:
   - *Cutout*: paredes perpendiculares aos lados até ao centro do filete (`corner_style::normals_meet`);
     o paralelogramo antigo cruzava-se num triângulo.
   - No tecto do raio, Concave/Cutout caíam na recta (pedaço de comprimento zero sem tangente):
     `corner_live` toma a tangente do segmento de origem no corte.
   - *Smooth*: `1` = o redondo (`rounded = smooth × rounded_at_one(m)`, `1/(3·cos²(π/2m))`); com raio
     de canto a alça roda da aresta para a tangente pelo peso `liso/(liso + quinhão do raio)` — o 1.º
     passo já não apaga o raio. O oráculo do Inkscape converte (`ours(r, m)` nos testes F10).
   - Gates novos: `ph2d-vec-scene/src/polystar_smooth_tests.rs` (5).
   - ⚠️ Régua que mentiu: Hausdorff ponto a PONTO leu `0,023` sobre o MESMO contorno (amostras de uma
     aresta recta de alças coladas amontoam-se nas pontas). Ponto a SEGMENTO: `0`. Os testes usam o 2.º.
   - A folha de contacto (todo o catálogo × todo campo × 9 valores + ampliações) ficou em
     `vetor_nova_ui/ferramentas/folha_de_contacto_tests.rs.txt`: para correr, copiar para
     `crates/ph2d-tool-vector/src/audit_probe_tests.rs`, `mod` no fim de `shapes.rs`,
     `PH2D_AUDIT_SVG=<f>.svg bash scripts/ph2d-run.sh cargo test -p ph2d-tool-vector --lib audit_ -- --ignored`,
     `rsvg-convert` — e apagar antes do commit.
2. **A — seleccionar no modo Forma** (commit `f5b37f04c0`): reproduzido VERMELHO pela cadeia do
   produto (`shape_mode_handles_tests.rs`). H1 confirmada (o `frame_fields` só desarmava quando a
   selecção mudava: clicar na forma já seleccionada deixava armado, sem alças) → o clique que acerta
   desarma (`after_the_shape_release`). H2 confirmada → `hides_the_nodes` = `mode == Shape || …`.
   H3 FALSA (o `path_at` já conta o preenchimento; o gate clica no meio de uma estrela). H4: uma forma
   sem `VecShape` no modo Forma fica sem alças e sem nós (a caixa continua) — sem gate próprio ainda.

## O que está ABERTO (por esta ordem)

1. **A, o resto:** a cena NOVA `=20` (a contagem em `crates/ph2d-app-vec/src/vector_mode.rs`; modelo:
   `vector_mode_setas.rs` + `_tests.rs`): duas formas, a paleta arma uma terceira, um clique selecciona
   a 1.ª (alças verdes, nenhum nó) — e um clique DENTRO da 2.ª. Foto antes de passar. Um gate H4.
2. **B** — a *Resize Box* mantém a espessura do traço (doc 17 §2: a cura vive só na moldura,
   `m·(r − 1)`). Gate: 2× pela caixa, largura igual, caixa visual exacta nas duas bordas.
3. **C** — o ⇕ do *Viewed* dos isométricos: subir = «De cima» (só ele inverte).
4. **D** — caixa baixa: a fila de ⇕ fora da alça de canto (gate de 20 a 400 px de altura, nenhum ⇕
   sobre alça de escala nem sobre a ④). ⚠️ o meio da borda tem a alça do meio.
5. **E** — o bico cortado na placa da GPU do Motion (`ph2d-shape-gpu/src/contorno_cpu.rs` + WGSL) —
   se não couber, 1.º aberto do handoff de integração.
6. Achados da auditoria SEM cura (doc 15 §3b, decidir/mostrar ao dono): o *Cutout* no tecto deixa lascas
   de espessura zero (o rectângulo 2:1 fica um quadrado com dois «bigodes»); a seta dobrada com
   *Tail* > *Head width* perde as farpas (o OOXML prende a haste à cabeça); o *Check* com peso ≥ 0,5.
7. **FECHO** (DIRETRIZ §1.5.9, a ordem do prompt 22): mutação de cada cláusula nova ANTES do portão —
   `normals_meet`, a tangente de origem do `corner_live`, `rounded_at_one`, o peso `open`/`toward`,
   `liso_em` (G1 só sem raio), `shape_armed = false` no clique, o `mode == Shape` do
   `hides_the_nodes` · portão batched (`verificador`) · handoff de INTEGRAÇÃO F13 · CLAUDE.md §5.1
   (smoke `…6..20`) · `doc-index.sh` · `agent-loop-profile.sh` · `rm -rf target/*/incremental` ·
   binário smoke 2× · o smoke ao dono (passos numerados; incluir o Polígono: *Cutout* e *Smooth*).

## Para a próxima janela

`cd Worktrees/line-UIUX && pwd && git branch --show-current` antes de ler. Leia ESTE handoff e o
prompt 22 — não o histórico. Fotos só com a placa livre; nunca `pkill -f`.
