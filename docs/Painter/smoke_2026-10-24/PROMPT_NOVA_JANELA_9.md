# Prompt da janela seguinte — line/PainterWatercolor, 25/10 (depois do smoke da janela 8)

```
Linha line/PainterWatercolor (Modo L), continuação de 25/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md e docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_9.md inteiro. Delegue leitura e verificação.
```

⛔ Leis desta janela: **sem bateria por crate nem suíte por commit** (dono 25/10) — só testes por NOME +
clippy/fmt das crates tocadas; handoff/prompt só DEPOIS do smoke do dono.

## O que a janela 8 fez (commits `16facb026b` e `c69b3d47d9`; handoff em [`HANDOFF_line_PainterWatercolor_2026-10-25_JANELA8.md`](../handoffs/HANDOFF_line_PainterWatercolor_2026-10-25_JANELA8.md))

1. **Água do Wet mais rápida**: `Rows::Medida(piso_ns)` em `crates/ph2d-wet-paint/src/par.rs` (linha do meio cronometrada, resto ao pool
   se `decorrido × linhas ≥ piso`), `pick_medido` nas seis passagens da água, `PISO_NS_AGUA` = 20 µs; ciclo de 12 passos até 8,8×
   (55 236,5 → 27,8 ms). Bytes idênticos à série (gates `a_rota_medida_e_a_serie_ao_byte`, `a_rota_medida_nao_muda_o_wet_com_bloom_e_pickup`).
   O gate de fábrica `a_largura_e_a_prometida` NÃO encurtou (CPU saturada por 232 cenas).
2. Gate vermelho desde a janela 7 curado (`os_grupos_avancados_do_tuning_chegam_aos_knobs_e_repoem_so_o_seu`).
3. **Smoke do dono**: retângulos brancos com Wet bloom e Tip pickup altos (somem sozinhos, ficam suaves) e queda de FPS ao mesclar
   cores. Fotos em `fotos_janela8/`. Não é da janela 8 (byte-idêntico); mecanismo por achar.

## Abertos

1. **NOVO**: retângulos brancos (bordas em escada, alinhados aos eixos) no início da simulação com Wet bloom (`Knob::WetOpen`) e
   Tip pickup (`Knob::TipReservoir`) altos; e queda de FPS ao mesclar cores (não medida).
2. Decisão do dono pendente: forma fechada no lugar do Newton do Rebelle (muda bits) para encurtar `a_largura_e_a_prometida`.
3. Decisão do dono pendente: Borracha da Watercolor que mantém a sessão — (a) buraco limpo ou (b) levantar com mata-borrão
   (mapa do modelo da sessão no handoff §5.3).
4. Da janela 7: Accumulate ON + passagem translúcida (Flow 0,3) sem Continuous escurece a cauda; ressalto ~+10 % na entrada da
   janela da cauda sem Continuous; Wet presets não gravam Press; S-curve, Light scatter, Speed lag; canal do «Reset to brush
   preset»; defeito 5 do Pen (Tilt smooth/Rotation); Dry brush (`ExtDryBrush`) knob morto; decisão 2 do dono (escondidos do
   Digital sem linha).
5. Da janela 4: macros «mundo» leem a sprite sem a tinta por assar; Borracha do Wet × Strength; preview da Borracha no Wet;
   Texture Eraser à Strength 1; a shell a 23 linhas do tecto.

## Próximo passo

Um só bloco (§0.10):
1. Achar o mecanismo dos retângulos brancos: reproduzir sem janela (cena das fotos: traço vermelho e outro escuro por cima, os dois
   knobs altos), fotografar os quadros, `/pd-bug-causa`.
2. Medir a queda de FPS ao mesclar cores na mesma cena (por etapa e por dab; candidatos: tip pickup por dab, Newton do Rebelle no
   depósito, K–M), com as sondas de `measure_stage_cost.rs`.
3. Se o dono respondeu (a)/(b), construir a Borracha da Watercolor.

OU integrar a linha no main, se o dono mandar (só por ordem dele, `/pd-integracao`; a suíte completa é do integrador).

## Como fechar

1. Um plano, uma rodada de medição, gates por nome, mutação das leis novas.
2. Só testes por nome + `clippy -D warnings` e `fmt` das crates tocadas. Sem suíte.
3. Smoke ao dono em passos numerados e sem jargão (comando com `cd`, onde clicar, o que acontece, como saber que falhou). Só depois
   do veredito: handoff e prompt.
