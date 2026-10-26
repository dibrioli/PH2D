# Prompt da janela seguinte — line/PainterWatercolor, 26/10 (a meio de N1–N4; a janela 14 trocou pelo tamanho)

```
Linha line/PainterWatercolor (Modo L), continuação de 26/10 — janela 15, a meio do bloco N1–N4.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_12.md (os pedidos
do dono), PLANO_JANELA_12.md e docs/Painter/smoke_2026-10-24/PROMPT_NOVA_JANELA_15.md inteiro. Delegue leitura e verificação.
```

⛔ Leis: sem bateria por crate nem suíte (dono 25/10) — testes por NOME + `clippy -D warnings`/`fmt` das crates
tocadas; handoff só DEPOIS do smoke do dono. ⚠️ Agentes `sonnet` sem cota até 27/10, 13 h: `general-purpose`/fork.
⚡ Corra os testes pesados em `--profile smoke` (o censo da água: 22 min em debug → **298 s** em smoke). Gerador longo:
`PH2D_PRAZO=<s>` no `ph2d-run.sh` (o prazo por omissão, 1800 s, mata-o); copie o binário de teste para o scratchpad e
corra-o direto (`CARGO_MANIFEST_DIR=<crate> … it presets_de_fabrica::gera_os_presets_de_fabrica --ignored --exact`)
para não prender o cadeado do cargo. Cada preset do Wet custa ~10 min (a tabela de largura; 8 presets = 82 min).

## O que a janela 14 fez (commits `5468ea0b42`, `a70d46b9b6`)

- **Passo 1 FECHADO — o Inspector das ferramentas da água só mostra o que elas lêem** (`5468ea0b42`). Medido antes de
  esconder (sonda, sem ponta de imagem — o censo grande usa uma, que esconde o Falloff): Falloff e Hardness agem no
  Pincel da água (7–9 mil texels) e 0 nas cinco; Flatten e o giro (carimbo achatado) agem nas cinco; Follow, Dab
  rotation e Speed→Opacity agem em 4 e 0 no Blow. Daí: `LinhaDaFerramenta::Forma` (Falloff, Hardness, a textura da Shape
  e o que dela depende, Shape Tone, giro/espelho do Jitter — a Shape delas fica Taper + Flatten/giro + Follow),
  `Orientacao` (Follow + Dab rotation), `OpacidadePelaVelocidade`, `AguaViva` (só o Blow). Per-Layer Color sai com a
  Forma (`CorPorCamada` fora das listas da água: não age no Pincel da água nesta bancada). Gate novo
  `o_carimbo_das_ferramentas_da_agua` (6 s: age ⟺ a tabela diz que lê). Censo `cada_ferramenta_da_agua_so_ve_o_que_le`
  verde e sem `#[ignore]`, controlo POR LINHA, inertes com motivo: o giro da pega e o Reset da Shape. Os 6 do
  `o_painel_mostra_o_que_a_ferramenta_le::` verdes. Mutações que sangram: M1/M2 `no_trilho` (com controlo M0 verde),
  M5 Smear sem Forma, M6 Blow sem Orientacao, M7 a Shape inteira nas da água (75 falhas).
- **A régua da largura de uma ferramenta** (`a70d46b9b6`, `presets_de_fabrica.rs::imagem_da_ferramenta`): aplicava o
  preset pela porta interna `wet_preset_aplicar`, que desde a janela 13 escreve o preset da ferramenta no slot dela e
  deixa o PINCEL na mão — media o Ink Brush. Agora `preset_aplicar` (a porta do navegador, que pega a ferramenta).
  Blender Soft volta à promessa (só o raio 120: 232 contra 240).
- `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` junta TODAS as que mudaram antes de acusar.

## ⛔ O DEFEITO aberto (a cura é o próximo passo, antes de regerar)

**Aplicar um preset de ferramenta da água perde os knobs de cerdas dele** (regressão da janela 13). O slot só guarda os
knobs da MÃO (`tool_link.rs::switch_brush_slot`, `knobs_por_slot` só repõe `mao::knobs_da_mao`) e o preset só escreve
`WetKnobs::so_da_ferramenta(t)` (5, à mão) + mão (`wetpaint_preset.rs` ~l. 456). O Blender Soft precisa do felt 0,2 («sem
ele a mistura sai em riscas verticais», a receita): sonda — repor felt 0,03→0,2 e bristleCount 2000→4000 leva a
miniatura de 12 932 px diferentes a **189**. É o que o dono veria: o Blend de fábrica riscado.

**A cura (desenho decidido):** o slot de cada ferramenta da água guarda os knobs do PINCEL que o leitor dela lê
(grupos `KnobGroup::{Paint, Tools, Bristles, Stroke}` — a física, o papel, os efeitos da água e o look ficam da TELA,
partilhados); a lista vem de um CENSO medido (knob × ferramenta sobre as riscas molhadas, como `bancada_da_agua` do
`o_painel_mostra_o_que_a_ferramenta_le.rs`), gravada numa tabela com gate nos dois sentidos (a tabela = a medição);
`switch_brush_slot` repõe esses do slot novo; o preset de ferramenta escreve-os (substitui `so_da_ferramenta` + mão).
Gates: `aplicar_um_preset_da_agua_nao_mexe_no_pincel` (o Pincel não muda) e a miniatura do Blender Soft volta ao byte
(ou explique os 189 px restantes). Mutação: tirar um knob da tabela ⇒ o gate do censo sangra.

## Por fazer (por ordem)

1. A cura acima (+ o resíduo de 189 px do Blender Soft).
2. **Borracha de textura** (Digital) — a miniatura mudou 18 412 px, NÃO por knobs (a sonda repôs todos: igual). A faixa
   vermelha de base passou a sair com a textura da borracha. BISSECÇÃO (agente, 26/10, alvo próprio): verde em
   `8070c531b6`, vermelho desde **`86ca0e7df0`** (janela 11, «o grão preso à tela morde também à Strength 1»; a mesma fnv
   `0x76aeefbf1d027e67` que hoje — as janelas 12–13 são inocentes). Mecanismo: `stamp_route.rs::stroke_cover_wanted`
   (~l. 306–314) passou a armar o tecto de cobertura com grão Tiled/Stencil mesmo à Strength 1 (de propósito, para o
   traço da borracha); `preview.rs::faixa_da_ferramenta` (l. 454) pinta a faixa com o pincel do preset e só repõe
   força/fluxo/blend/knobs/raio/cor — deixa o `brush.texture` ligado, e agora o grão aparece na faixa. **Cura:** a faixa é
   «tinta CHEIA» (o comentário dela): desligar a textura do pincel enquanto ela pinta (e repô-la a seguir) — NÃO regerar a
   miniatura. Gate: a miniatura volta ao byte; o censo das 21 receitas de meio do `86ca0e7df0` não cobria as miniaturas do
   Digital — a miniatura agora lista todas (janela 14).
3. Regerar os 8 + o Blender Soft numa corrida (`PH2D_PRAZO=7200`, ~90 min): `PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Blender
   Soft,Blender Textured,Blender Dots,Blender Hard,Smear Soft,Smear Spikey,Smear Flat,Smear Shattered,Smear Wet"`. A
   Borracha de textura NÃO se regera (a cura do passo 2 devolve-a ao byte).
   ⚠️ Com a régua velha o Smear Soft media 1 px abaixo do raio 55 — confira a tabela crua dele logo que sair; se ainda for
   ~1 px, a lei de largura do Smear (o que ele MUDA na faixa) é outra pergunta: não force a promessa.
4. Gates por nome (todos em `--profile smoke`): `cada_preset_da_agua_cai_na_foto_do_seu_rebelle` (vermelho: Blender Hard
   OrlaV 16 vs 8±4,6; Smear Soft/Spikey Fundo 81 vs 119±25 — medidos COM o defeito; reavaliar), `a_largura_e_a_prometida`,
   `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, `aplicar_um_preset_da_agua_nao_mexe_no_pincel`,
   `cada_ferramenta_tem_os_seus_presets_de_fabrica`, `cada_ferramenta_da_mistura_se_reconhece_pela_foto`, `presets_de_fabrica::`
   (14 deles verdes a 26/10); app-painter `--features panel-painter-layers` `presets biblioteca categoria` (verdes).
5. Mutações por fazer: M3 `slot_da_ferramenta` sem o ramo da água e M4 o preset de ferramenta a escrever todos os
   knobs — ambas devem sangrar `aplicar_um_preset_da_agua_nao_mexe_no_pincel`; + a do censo dos knobs (passo 1).
6. `PH2D_SUITE=fecho` clippy `-D warnings` + `fmt` de ph2d-tool-painter, ph2d-panel-painter-layers, ph2d-editor-core;
   smoke ao dono em passos numerados (onde está cada ferramenta da água na barra; o que o Inspector mostra com cada uma —
   o Blow sem Follow/Live water; o Water do Smear no cartão; os presets novos no navegador do Smear e do Blend). Só
   depois: handoff e prompt.

## Abertos medidos (para o handoff)

- O Smear da água não abre o canal LIMPO do SMUDGE do Rebelle (resto 0,09 em 15 px) — lei do motor, não de preset.
- O Per-Layer Color não age no Pincel da água na bancada do censo (ponta de imagem de uma camada) — por medir com uma
  Shape de várias camadas (`o_censo_so_encolhe`).
- Os abertos do PROMPT_12 (decisões 1–3 do dono, escondidos do Digital, Borracha do Wet larga, cauda do Digital).
