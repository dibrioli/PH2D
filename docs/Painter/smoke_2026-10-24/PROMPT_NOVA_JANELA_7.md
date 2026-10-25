# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (depois do smoke da janela 6)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md e docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_7.md inteiro. Delegue leitura e verificação.
```

⛔ Leis desta janela: **sem bateria por crate nem suíte por commit** (dono 25/10) — só testes por NOME +
clippy/fmt das crates tocadas; handoff/prompt só DEPOIS do smoke do dono.

## O que a janela 6 fez (commits `15167f3756` e `7328a56870`; detalhes em `git log -2`)

1. **Cartão Pen**: `speedHardness = −k·ln((Γ+v)/Γ)` (tecto 0,3), `pressureHardness = −k·(1−p)`; a mão só amacia e
   nunca tira mais que a Hardness (Ballpoint hardness 0,15 e Glow 0,2707 compensam; miniatura igual ao píxel).
   Inércias honestas (`MaciezSemDureza`; Min/Max/Smoothing/S-curve esmaecem só sem NENHUM leitor da pressão);
   nomes «Press. softness»/«Press. S-curve»; casas do passo e unidades; Rotation alinhada. Tilt smooth e Rotation
   NÃO esmaecem (o censo mediu que mudam a tinta pela rota inclinada) — **defeito 5 da auditoria ABERTO** (cura no
   motor: não rotear um `DabTilt` neutro).
2. **Vermelhos herdados da retirada do Rendering** curados: Strength fora do plano (`traco_plano` `escala_do_plano`),
   controlo cauda×grão, fixture Sketch Pencil.
3. **Continuous + Accumulate off + Flow<1 em contas**: taxa por sub-dab `1 − (1 − flow)^(1/n)` em
   `crates/ph2d-painter-brush/src/dab/bands.rs` (`c_cap`); gate `o_continuo_com_flow_baixo_e_liso`
   (`crates/ph2d-tool-painter/tests/it/sonda_continuo.rs`).
4. **Wash** (`BrushSpec::wash`, `cover_max`; arma a cápsula em `spec/queries.rs` `lays_capsules`; caixa abaixo do
   Accumulate; gates `tests/it/o_wash.rs`).
5. **Reset do Pen com duas portas** (`mao.rs` `MaoDoPreset`; menu «Reset to default» / «Reset to brush preset»).
6. **Wet**: 40 dos 58 `Hidden` no Tuning (Bristles/Stroke/WetEffects/Look); 4 no cartão Wet Paint (Backruns,
   Granulation, Wet bloom, Tip pickup: `KNOBS_NO_CARTAO` em `wetpaint_settings.rs`, ids em `ids/painter_wetpaint.rs`,
   i18n `painter_wet_cartao.rs`). **Dry brush (`ExtDryBrush`) é knob MORTO**: `dry_gate = 0` em todo dab (b = 10 na
   rota Paint, `ph2d-wet-paint` `painter/doors.rs` ou `rota_da_lane.rs`).

## Smoke do dono (25/10) — veredito e reports, POR FAZER NUM BLOCO (§0.10)

- **OK**: Granulation e Backruns («percebi os efeitos»).
- **R1 (defeito)** «o primeiro carimbo do traço usando Continuous em Digital fica muito marcado». Foto: Glaze
  Round, Continuous, Accumulate off, Flow baixo; um disco mais escuro numa ponta do traço. **Hipótese a MEDIR, não
  confirmada**: o dab sem passo (o 1.º, sem cápsula) leva a taxa inteira `flow` de uma vez, enquanto cada sub-dab
  leva `1 − (1−flow)^(1/n)`; o 1.º dab devia levar a taxa de UM sub-dab. Ver também o último dab / o pen-up.
  Medir estendendo `sonda_continuo` (ondulação perto do início e do fim), curar, gate + mutação.
- **R2 (pedido)** «Pigment diffusion é importante e merece ir pro inspector»: `Knob::ExtDiffusion` («Pigment
  diffusion») para o cartão Wet Paint, pelo mesmo mecanismo dos 4 (`KNOBS_NO_CARTAO` passa de 9 para 10 entradas).
- **R3 (pedido)** «As propriedades que estão no inspector não devem se repetir no painel Tuning»: os knobs do
  cartão (os 4 novos + `ExtDiffusion`) saem do Tuning (os 5 antigos já são `Hidden`); ajustar o censo
  `os_escondidos_sao_so_os_de_outra_superficie_ou_de_calibracao`
  (`crates/ph2d-panel-wet-tuning/src/rows.rs`) contando.
- **R4 (dúvida, respondida ao dono)**: Wet bloom e Tip pickup «sem efeito» — só agem com o pincel a entrar em
  tinta MOLHADA de OUTRA cor. Considerar uma dica (tooltip) nessas duas linhas que o diga.

## Abertos herdados (resumo)

- Decisão 1 da janela 6 resolvida (Wash = opção). Riscos escuros das cerdas (`bristleStreak`) ficam no Tuning
  (iriam para a Shape — mais trabalho).
- Wet presets não gravam Press. S-curve, Light scatter e Speed lag (o Reset para o preset põe-nos no neutro no Wet).
- A disponibilidade do «Reset to brush preset» é um canal à parte (devia ser campo do `BrushSettings`).
- Defeito 5 da auditoria do Pen (Tilt smooth/Rotation, acima).
- Da janela 6 (`PROMPT_NOVA_JANELA_6.md`), ainda abertos: decisão 2 do dono (escondidos do Digital sem linha:
  `pigment_espectral`, `passo_pela_pressao`, `borda_px`, `tecto_por_dab`, `sub_passo_fino`, `particle_max_px`) —
  mostrar ou fixar de propósito; lei do Accumulate off / «Wash» já entregue como opção.
- Herdado da janela 4: acelerar a água do Wet (dono escolheu); Borracha da Watercolor que apaga mantendo a sessão
  (dono escolheu); a família «mundo» das macros lê a sprite sem a tinta por assar; Borracha do Wet × Strength;
  preview da Borracha no Wet; Texture Eraser à Strength 1; a shell a 23 linhas do tecto.

## Como fechar

1. R1–R4 num só bloco: um plano, uma rodada de medição (sonda estendida), gates por nome, mutação das leis novas.
2. Só testes por nome + `clippy -D warnings` e `fmt` das crates tocadas (`ph2d-painter-brush`, `ph2d-tool-painter`,
   `ph2d-wet-paint`, `ph2d-panel-wet-tuning`, `ph2d-i18n` se mexer em texto). Sem suíte.
3. Smoke ao dono em passos numerados e sem jargão (comando com `cd`, onde clicar, o que acontece, como saber que
   falhou) — fotografe a cena antes de mandar. Só depois do veredito: handoff.
