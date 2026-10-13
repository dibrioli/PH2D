TROCA DE AGENTE — line/PainterWatercolor: **o Spacing contínuo nos outros meios** (relato do dono 13/10: *«Spacing
em wet paint produz traços mais uniformes que em digital (que fica pontilhado)… como melhorar os outros modos sem
perder performance»*). Vem ANTES do prompt 30 (W3/W4 do plano 27).

Leia SÓ: [31](31_spacing_por_meio_o_pontilhado.md) inteiro (medido, o porquê, o desenho) · o topo (R1–R9) do handoff
`docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-13_W0_W1_SHAPE_CASA.md`.
⚠️ «faça como achar melhor e pare de me fazer perguntas» (dono, 11/10).

## Fase 0
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current` →
`line/PainterWatercolor`; `git status --short` limpo; `git cherry main HEAD`; `bash scripts/btrfs-health.sh`.

## Num bloco (§0.10)
1. **As cerdas que arrastam** (31 §3): `texture::bristles::sample_dab` lê o tapete ARRASTADO pelo deslocamento
   desde o dab anterior (`S_Δ = máx_t tapete(o − tΔ)`), cozido 1× por `Δ` inteiro num cache por thread. O `Δ` tem de
   chegar ao amostrador: veja como o `TexDabBasis`/`ShapeFrame::Stroke` já leva o arco e a tangente — sem uma segunda
   porta. O Wet Paint (`bristles_at` do hospedeiro) NÃO arrasta: as 12 miniaturas de fábrica ao byte.
   Portões: o CV ao longo (`sonda_pontilhado_das_cerdas`) do Digital Bristles a 0,1/0,25 cai para perto do de 0,025;
   o custo por traço (MEDIR_VELOCIDADE: variantes no MESMO processo, intercaladas, o mínimo) fica ≤ o de hoje + ruído;
   a 0,025 a tinta quase não muda.
2. **O dab-cápsula da redonda** como opção do Stroke (31 §3) — ou decida pela medida que não vale; se entrar,
   desligada por omissão e com o censo dos controlos a vê-la agir.
3. **O Spacing do Wet Paint com o significado do Digital** (31 §2.3): meça o passo efetivo contra o pedido nos 12
   presets; proponha a lei e a prancha antes/depois (muda o traço com Spacing ≠ fábrica — prancha para o dono).

## Testes — a CADÊNCIA (dono 13/10)
Por item e por pedido do dono: `cargo-check-narrow.sh` + os testes do item POR NOME (`cargo-test-narrow.sh`).
A suíte inteira só no fecho (`PH2D_SUITE=fecho`), com 5 commits desde a última verde, ou com um motivo escrito
(`PH2D_SUITE="motivo: …"`) — o `nextest-impacted.sh` recusa o resto. Corrida longa: no fundo, sem a esperar.

## Fecho
R1 ~150 chamadas · `verificador` (fmt, clippy `-D warnings --all-targets`, `nextest-impacted`) · handoff ≤ 8 KB ·
atualizar o 26 e o 31 · commit por caminho, nunca push · smoke ao dono em passos numerados (Digital ▸ Shape ▸
Bristles, Spacing 0,1 e 0,25: riscas, não pontos).
