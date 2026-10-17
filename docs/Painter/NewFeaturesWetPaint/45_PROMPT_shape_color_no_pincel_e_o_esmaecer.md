# 45 — PROMPT: a Shape Color no pincel salvo (B13) e o esmaecer da pressão (C27 + C28) (17/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo; `bash scripts/btrfs-health.sh`. Leia SÓ: este prompt ·
§3, §6 e §7 do [handoff P44](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P44_FORMAS.md) ·
[44_medicao](44_medicao_as_formas.md) §1, §3 e §6 · lista [26](26_ABERTOS.md) B13, C27, C28 · o §6 do
[handoff P41](../handoffs/README.md) sobre as rampas (B13). Regras de 13–14/10 (teste POR NOME, suíte a cada 10
commits, clippy só no fecho, edição provada). O Rebelle corre-se, nunca se lê; o teste dele acaba por volta de 06/11.

## De onde vimos

O P44 fechou o bloco C do 18 §5 (a ponta reta `dab_quina`, a pressão do pincel `pressureSize`/`pressureOpacity`, o
Wet Flat e a Gouache Flat; fábrica 20). Smoke do dono 17/10: **OK**. O «Square tip» travado em 1 foi curado
(`bfc868d90f`). O dono decidiu: **B13 = SIM** (a Shape Color vai no pincel salvo) · **B14 = a mistura de pigmentos
fica da obra inteira** (fechado) · **Airbrush «como no Rebelle»**.

## Pedido

1. **B13 — a Shape Color no pincel salvo do Wet Paint.** Hoje as rampas (as duas camadas Texture Color + Falloff
   Color, a ordem, o Blend, a Opacity por camada, o B&W) vivem no estado da ferramenta; a MACRO já as leva
   (`macro_track::inicial`) — pergunte ao CÓDIGO se essa serialização serve ao preset antes de escrever outra (uma
   lei em dois sítios diverge). Formato: por NOME, omissão = sem rampa (os 20 de fábrica ao byte — a identidade das
   20, as cenas de `a_lei_da_tinta` + as 2 novas, pinadas no HEAD de partida); conte se `SCHEMA` sobe e porquê.
   Aplicar um pincel sem rampa TIRA a rampa do anterior (a regra dos campos ausentes do P44). Portões vermelhos
   primeiro (o pincel gravado pinta o mesmo traço com a rampa; ida e volta pelo JSON); mutação.
2. **O esmaecer da pressão (C27 + C28)** — medir primeiro, um lote: porque a opacidade da pressão quase não morde
   no Airbrush (0,92/0,80 com tamanho 0 · opacidade 1; o Spray 2 0,63/0,34) e só morde a meio no Wet Flat (0,54
   contra 0,42). Pergunte ao código o caminho `d.coverage → b → pressure_dab → o depósito` (o orçamento de
   pigmento por dab? o pó, que divide a dose por `p`? a mediana da régua sobre partículas?). Se a cura for uma CURVA
   de pressão por pincel, ela entra como knob por nome com omissão = a linear de hoje. Depois: o Airbrush com os
   números do Spray 2 (ordem do dono) e a rampa dos Flats dentro do vale (`RampaCobertura` no
   `midias_de_fabrica`, hoje fora); identidade re-pinada só onde o dono mandou.
3. Fecho: suíte afetada + clippy (`PH2D_SUITE=fecho`); handoff ≤ 8 KB; smoke ao dono em passos numerados (salvar um
   pincel com a Shape Color, reabrir o app, escolhê-lo; o Airbrush com caneta, leve e forte).

## Aberto que fica para depois (não junte)

- A 20 px o Airbrush lê textura 28,0 contra 22,5 (a janela 5×5 num traço de 10–13 px?) — C28.
- `cada_preset_parece_a_sua_midia` (83 s) e `a_largura_e_a_prometida` (131 s) passam do prazo de 360 s com a
  máquina carregada — candidatos à faixa lenta do nextest (`docs/DevOps/FLAKES_DE_CARGA.md`).
