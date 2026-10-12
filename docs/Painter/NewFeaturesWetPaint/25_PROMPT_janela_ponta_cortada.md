TROCA DE AGENTE — line/PainterWatercolor: o **Bug #52** — a ponta da Shape de imagem sai CORTADA quando roda
(Jitter Rotate) ou com Size < 1. Achado do dono no smoke de 12/10, MEDIDO na janela 24; a cura vem aqui.

Leia SÓ: `docs/Painter/BUGS_painter.md` §Bug #52 (mecanismo, tabela, o portão verde que mentia, a cura
desenhada) e o topo (R1–R9) do handoff
`docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-12_BUG51_TEXTURA_TINGE.md`.

⚠️ Ordem do dono (11/10): «faça como achar melhor e pare de me fazer perguntas». O produto: a ponta inteira
aparece, em qualquer ângulo e Size; o carimbo de grade e a ponta a 0°/Size 1 ficam ao bit.

## Fase 0
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current`
→ `line/PainterWatercolor`; `git status --short` limpo; `git fetch -q origin` e rebase se o `main` andou;
`bash scripts/btrfs-health.sh` · `bash scripts/agent-loop-profile.sh 10`. Fish: laços em `bash -c`.

## A tarefa
1. Veja VERMELHOS: `cargo test -p ph2d-tool-painter --lib ponta_cortada -- --include-ignored` (hoje
   `10 10 356` e `dinâmica 441 · cache 400`). Meça também o Angle ESTÁTICO da Shape (sem jitter, rota
   cacheada) na mesma sonda — o mecanismo prevê o mesmo corte.
2. A porta do alcance (BUGS #52 §cura) e TODOS os que respondem «até onde vai o carimbo» por ela: os
   núcleos de cor (`stamp_color/accumulate*.rs`), o `dab.rs` (caixa `extent`), os dois carimbos cacheados
   (domínio do assado + `tip::axis_taps`) e as regiões do tool (`region.rs`: `dab_bbox`, `caixa_do_lote`;
   `dab_batch_region`; sujar/desfazer). Inflar a menos deixa rasto (Bug #24): portão de re-carimbo
   (Ellipse com a ponta rodada, editada, sem rasto).
3. Corrija o oráculo de `tip_kernel_tests.rs` e os docs de `tip.rs`/do teste: a ponta é `|tex| ≤ 1` no
   referencial DELA; acrescente casos rodado / Size 0,6 / Offset por KERNEL.
4. Mutação: a caixa volta a `± raio` ⇒ os portões sangram. Custo: a área visitada cresce só onde a ponta
   cresce — meça a rota dinâmica antes/depois no mesmo processo.

R1: ~150 chamadas. Fecho: bateria pelo `verificador` (como no prompt 24), handoff ≤ 8 KB com R1–R9, o #52
para o índice dos fechados, commit por caminho, nunca push; smoke ao dono em passos numerados (Shape de
imagem quadrada, Jitter Rotate no máximo, Size 0,6: a ponta inteira, sem lados retos).
