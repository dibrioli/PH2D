# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (depois do smoke da janela 7)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md e docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_8.md inteiro. Delegue leitura e verificação.
```

⛔ Leis desta janela: **sem bateria por crate nem suíte por commit** (dono 25/10) — só testes por NOME +
clippy/fmt das crates tocadas; handoff/prompt só DEPOIS do smoke do dono.

## O que a janela 7 fez (commits `4b6186562d` e `aec9653a32`; handoff em [`HANDOFF_line_PainterWatercolor_2026-10-25_JANELA7.md`](../handoffs/HANDOFF_line_PainterWatercolor_2026-10-25_JANELA7.md)) — smoke do dono OK

1. **R1**: o 1.º carimbo do Continuous (Digital, Accumulate off, Flow<1) não marca mais — a 1.ª cápsula leva a abertura
   como sub-dab (`crates/ph2d-painter-brush/src/stroke/abertura.rs`); gates `o_inicio_do_continuo_nao_marca`,
   `um_toque_do_continuo_pinta_o_dab_inteiro`, `o_fecho_leva_o_sub_dab_de_tras`.
2. **R2–R4**: «Pigment diffusion» (`Knob::ExtDiffusion`) no cartão Wet Paint (`KNOBS_NO_CARTAO` 10); os 5 knobs do
   cartão saem do Tuning (84→79 linhas); dicas em Wet bloom e Tip pickup.
3. **Cauda com Continuous**: o defeito era o PLANO do traço (`traco_plano.rs`): caixa do dab inclui a cápsula,
   `sem_a_strength` escala `ante.coverage`, a ponta encadeia em cápsula (Accumulate off + Flow<1 ou Continuous).
   Gates `o_plano_compoe_a_capsula_inteira`, `a_cauda_com_a_taxa_afila_sem_discos`, `a_cauda_nao_costura_o_grao_aleatorio`.

## Abertos

1. **NOVO, medido**: Accumulate LIGADO + passagem translúcida (Flow 0,3) sem Continuous — a cauda escurece pelos dabs
   soltos do `adensa` (79…121 sobre um corpo 56/71); a lei certa não é a cápsula (com Accumulate ligado ela compõe cada
   sub-dab à opacidade da passagem).
2. Ressalto de ~+10 % na entrada da janela da cauda sem Continuous (Accumulate off), dentro do gate (≤ 15 %).
3. Wet presets não gravam Press. S-curve, Light scatter e Speed lag; a disponibilidade do «Reset to brush preset» é um
   canal à parte; defeito 5 da auditoria do Pen (Tilt smooth/Rotation); Dry brush (`ExtDryBrush`) é knob morto;
   decisão 2 do dono (escondidos do Digital sem linha).
4. Da janela 4: acelerar a água do Wet (dono escolheu); Borracha da Watercolor que apaga mantendo a sessão (dono
   escolheu); família «mundo» das macros lê a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da
   Borracha no Wet; Texture Eraser à Strength 1; a shell a 23 linhas do tecto.

## Próximo passo

Os dois que o dono já escolheu — **acelerar a água do Wet Paint** e a **Borracha da Watercolor que mantém a sessão** —
num só bloco (§0.10): plano único, MEDIR antes (`MEDIR_VELOCIDADE.md`), gates por nome. OU, se o dono mandar,
**integrar a linha no main** (só por ordem dele, `/pd-integracao`; a suíte completa é do integrador).

## Como fechar

1. Um plano, uma rodada de medição, gates por nome, mutação das leis novas.
2. Só testes por nome + `clippy -D warnings` e `fmt` das crates tocadas. Sem suíte.
3. Smoke ao dono em passos numerados e sem jargão (comando com `cd`, onde clicar, o que acontece, como saber que
   falhou). Só depois do veredito: handoff e prompt.
