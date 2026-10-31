# HANDOFF — Janela 10 (30/10), linha `line/PainterWatercolor`

Base: `776a9ff173` (o dono: «a versão atual é superior ao main» — não reverter). Árvore limpa em `e42174cc5a`.

## Feito (commits)
- `7d343a9de4` **C — New Image «Textured Paper»** (Cold Press bege `#E6D9CC`, Relief/Roughness 0,5, Tooth 1, Contrast/Brightness
  0,5). O papel aparece ao entrar no Wet Paint; a cor do papel é do Painter inteiro (já era). Modal NÃO fotografado.
- `c56ca5900e` retoque A+B ([`aguada_arrasta_oleo_simula.py`](../NewFeaturesWetPaint/ferramentas/aguada_arrasta_oleo_simula.py)):
  `J10_simula_AB_*` (arrasto + mistura KM) **não serve** («não re-molhou o azul»); `J10_remolha_*` **escolhida «re-molha médio»**.
- `84df135c02` **Re-wet** (`Knob::Dissolve`, «rewetPaint») × **Solubility** (`Knob::Solubility`, `OleoCelula::soluvel` pela
  massa): no pouso (`trail/transfer/remolha.rs`) a água dissolve até Re-wet × Solubility do assentado do 1.º toque (PASSO 0,25),
  lift `Ecra::Porto`. Gates `remolha_tests` (neutro ao bit; 0,35×1 ⇒ −36 % azul sob a aguada).
- `84ed24c8fc` sliders Re-wet/Solubility no cartão Wet Paint (após Tip pickup).
- Smoke do dono: «o pincel molhado não consegue mobilizar a tinta do Dabber… Smear parece mais com o que eu quero».
  Medido: o Smear/Blend mobilizam porque LEVAM a tinta do dab anterior (amostra no deslocamento) e levantam o assentado
  molhado `u·(1−staining)`. ⇒ `6c243555f0` **Drag** (`Knob::RewetDrag`, «rewetDrag»): `smear_com(Arrasto::Remolha)` com
  força Drag × max(soluvel célula, origem); `Trail::ultimo_dab` nas duas portas (doors.rs, painter.rs); slider «Drag».
  Medido (Re-wet 0,35): Drag 0,15 ⇒ −69 % azul sob a aguada, 0,5 ⇒ −96 %. Pranchas `J10_motor_drag_{b,p}`.
- `7bb6110ac5` **undo não mata a tinta viva**: `restore_model_confined` (tool/layers/undo.rs ~190) acabava a sessão em todo undo
  ⇒ o traço seguinte via a tela como tinta seca. Cura `wetpaint/undo_vivo.rs` (fotografia das grades no pen-down, repõe no
  undo). Gate `undo_de_outro_traco_nao_mata_o_oleo_soluvel`.
- `f70ea44cd9` **ladeira com o Real Oil na mão**: `Grid::oleo_antes` (altura do óleo dos traços ANTERIORES, fotografada no início
  do traço); a queda passa a ser a mesma de sem Real Oil.
- `e42174cc5a` **Staining, Glaze, Cover, Wet soften** no cartão Wet Paint (saíram do Tuning). Viscosity/Drag antigo/WetLift/
  Retention ficaram de fora (porquê no commit).

## Smoke do dono (pendente, a fazer na janela nova)
Programa: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo run -p ph2d-host-desktop --profile smoke`.
Fábrica: Re-wet/Drag/Solubility = 0 em todos os presets (ligar à mão: Dabber Solubility 1; Wet Flat Re-wet 0,35, Drag 0,05–0,3).
Os valores de fábrica são do dono, depois do smoke.

## Smoke do dono (30/10, depois do `e42174cc5a`)
- Foto [`pranchas/J10_smoke_dono_real_oil_empalidece.png`](pranchas/J10_smoke_dono_real_oil_empalidece.png) (setas):
  **«Real Oil empalidece a tinta»** — onde a aguada amarela com o Real Oil passa pelo óleo azul o miolo fica branco-rosado/pálido,
  não azul nem verde. NOVO item 0 do Aberto.
- **«Cor da água tingida: sai verde se Pigment mixing em Tuning estiver checado»** ⇒ o Aberto 3 (ardósia) é o modo de mistura
  RGB por omissão, não o motor do re-molhar; decidir com o dono se o Pigment mixing vira omissão do Wet Paint ou vai ao Inspector.
- Foto anterior (o pincel não mobilizava o Dabber): [`pranchas/J10_smoke_dono_nao_mobiliza.png`](pranchas/J10_smoke_dono_nao_mobiliza.png).

## Aberto
0. **Real Oil empalidece** (smoke acima): medir na sonda `remolha_tests` com o Real Oil LIGADO no Wet Flat (o Over Oil revertido
   da J9 e `Ecra::com_oleo`/`na_caixa_das_tintas` de `776a9ff173` são suspeitos: o branco do tapar extra); red-first.
1. **Queda pela DILUIÇÃO** (dono: «mesmo tirando muita água do pincel a queda é muito rápida»): a 1.ª lei (pesar a ladeira
   pelo `film`) não mexeu um algarismo ⇒ retirada. Tabela em `f70ea44cd9`/sonda `diag_a_queda_na_ladeira`
   (`queda_na_ladeira_tests.rs`). Próximo: medir `MaxVelocity` e a advecção do suspenso; pode pedir um knob. Testes por escrever.
2. Undo: aguarela × aguarela difere 13 B (estado fora da grade? `#[ignore]` com motivo); redo ainda mata a sessão; fotografia
   por passo SEM tecto de memória (medir a 4096²). `oleo_antes` não é reposto pelo undo; varre a grade inteira por traço.
3. Cor (ver smoke: verde com Pigment mixing): a água tingida sai azul-ardósia (o óleo tem ~5× a massa da aguada), o óleo restante não esverdeia; pontos amarelos sob
   o óleo a meio do traço (vistos também no smoke do dono, setas).
4. Censos por correr sob carga: `o_painel_mostra_o_que_a_ferramenta_le` (2 de 6: `a_cor_escondida_…`, `cada_ferramenta_da_agua_so_ve_…`)
   com `PH2D_TEST_TIMEOUT=1800`.
5. Fecho (1×): regravar 11 miniaturas + 2 impressões ao byte do `776a9ff173`; miniaturas Oil/* por nome (`f70ea44cd9` diz «ao byte
   por construção», não corrido); mutação vista vermelha das curas J9 e J10 (remolha, Drag, undo_vivo, oleo_antes); censo do papel
   de cor; BUGS #54; `tool::paint::wetpaint` inteiro por nome; foto do modal New Image.
