TROCA DE AGENTE — line/PainterWatercolor: o **plano 27**, onda **W2** — o Wet Paint lê as cerdas do SHAPE. A W0 e a
W1 estão feitas (27 §6). Ordem do dono (13/10): *«as configurações devem sair de Tuning e ir para Shape»*.

Leia SÓ: [27 §6](27_plano_shape_casa_dos_pinceis.md) (a W0 medida e as decisões) · o handoff
`docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-13_W0_W1_SHAPE_CASA.md` (R1–R9, o que
mudou, o que fica aberto). ⚠️ «faça como achar melhor e pare de me fazer perguntas» (dono, 11/10). Só o Wet Paint
evolui; Digital/Impasto/Watercolor AO BIT.

## Fase 0
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current` →
`line/PainterWatercolor`; `git status --short` limpo; `git cherry main HEAD`; `bash scripts/btrfs-health.sh`.

## W2 — num bloco (§0.10)
1. **A fonte das cerdas passa a ser o Shape.** No hospedeiro (`wetpaint/dab_route.rs`): com Shape = Bristles a
   silhueta é SÓ a envolvente (o falloff, sem o padrão) e o `texv` é o ladrilho do Shape
   (`ph2d_painter_brush::texture::bristle_tile(&spec.shape.params)`) amostrado pelo MESMO `sample_bristle` do motor
   (em células, `x − cx`, como `brush.rs:236`) — uma porta nova no motor para o ladrilho do hospedeiro, ou o motor
   aceitar o ladrilho; nada de segunda amostragem. Com Shape ≠ Bristles e ≠ None: o padrão continua silhueta
   (a lei B é a W3). Shape None no Wet Paint = ponta LISA (`texv = 1`)? — decida pela medida (D4: «capacidade nova»).
2. **D7 (Shape × Grain)** com as cerdas no Shape: meça o «escurece em dobro» (cerdas × Grain contra Grain só) nos
   presets de pastel/grafite; o oráculo não mede (o papel não desenha sob Wine). Decida e escreva no 27.
3. **Tuning esconde os 4** (`KnobGroup::Hidden` em `tuning/defs.rs`; os dois filtros seguem — 27 §6.4).
4. **Migração dos presets** (fábrica e utilizador): `engine.felt/bristle*` antigos ⇒ Shape = Bristles com
   `BristleKnobs::to_unit` (ida-e-volta AO BIT, gate `a_ida_e_volta_pelo_slider_e_exacta_em_todo_passo`); gravar só
   no Shape. Os 12 PNG de fábrica regravados com a miniatura.

## Portões
- impressão da sessão do motor AO BIT para todo preset de fábrica migrado (o mesmo traço antes/depois) · miniaturas
  ao bit · um preset antigo do utilizador carregado = o mesmo traço · Tuning sem os 4 (censo dos controlos) · os
  números do Shape são os do Tuning (`param_escala`, feito) — confira-os contra o Tuning antes de o esconder.
- R1 ~150 chamadas. Fecho: `verificador` (fmt, clippy `-D warnings --all-targets`, `nextest-impacted`), handoff
  ≤ 8 KB, atualizar 26 e 27, commit por caminho, nunca push; smoke ao dono em passos numerados (Wet Paint: Shape ▸
  Bristles, mexer Count/Size — as cerdas mudam; o Tuning já não as tem; um preset antigo pinta igual).
- Depois: W3 (a lei B do #47, prompt 30).
