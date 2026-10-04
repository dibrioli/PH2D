# HANDOFF DE CONTINUAÇÃO (2.ª onda) — `line/PainterWatercolor`, «cada controlo funciona em cada meio» (2026-10-04)

> **Para a próxima janela da MESMA linha.** A 1.ª onda está em
> [`HANDOFF_CONTINUACAO_…_2026-10-04.md`](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-04.md) — o
> instrumento do censo (§4) e o que custou tempo (§5) continuam valendo; leia-os.
> Não é o handoff de integração: esse escreve-se no fecho e cobre TODOS os commits desde `1ad60a1ce`
> (os 10 de 02/10, os 7 da 1.ª onda de 03–04/10 e os 6 abaixo). Nenhum tem handoff de integração ainda.

## §1 — Coordenadas

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor` |
| ramo | `line/PainterWatercolor` · HEAD = o commit deste handoff · merge-base `1ad60a1ce` (= o `main`, que não andou) |
| a missão | as decisões do dono de 2026-10-04, verbatim no [doc 46](../46_plano_cada_controlo_em_cada_meio.md) §1–§2 (o estado de cada item lá) |
| smoke | o binário `--profile smoke` foi compilado nesta árvore no fim da onda; o resultado do smoke do dono das DUAS ondas não voltou — pergunte |

## §2 — O que esta onda fez (não reconstruir)

| commit | o quê |
|---|---|
| `53a04d728` | **Dry Time — cada POÇA seca no seu tempo** (`watercolor_secagem.rs`, `assa_as_pocas_secas`, chamado no pen-down que continua a sessão). Medido antes: a sessão INTEIRA já secava ao fim do Dry Time; o defeito era o tudo-ou-nada (pintar noutro canto mantinha tudo a fundir). A poça (grelha de células `max(16, pad/2)`, elo `2·ceil(pad/lado)+2`) sem célula molhada alcançável assa: a base da sessão recebe a tela fora da zona de leitura do que fica; a união esquece-a. `alcance_da_janela` extraído do `wash_window` (uma porta; `pad_maximo` conta a água/soak por nascer). Gates `watercolor_secagem_por_poca` (4) + M1–M5. ⛔ Secar DENTRO de uma poça (doc 14 #12b) fica por fazer — o composite lê vizinhança; cortar dá aro novo no lado molhado |
| `c648f549d` | **o gate permanente do censo** `o_censo_dos_controlos_so_encolhe` (`tests/it/o_censo_so_encolhe.rs`, 25 s): inerte na fábrica ⇒ esmaecido OU na lista `inertes_com_motivo` (id ou id de opção + motivo); linha viva/fora da tela = vermelho. **Bug #33** curado (Grain *Image* sem imagem trocava as cerdas da água por folha lisa — `BrushSpec::grain_samples`) |
| `e4e450d09` | **Shape Color Ramp sai do Wet Paint** (`BrushSettings::shape_ramp_offered`; gate `a_rampa_da_shape_so_onde_o_meio_a_oferece`) |
| `6fbce3998` | **o Spread não é dependente**: age para BAIXO de raio/2 (`core_r = min(Spread, raio/2)`; 1 841 texels em Spread 1 na fábrica), para cima só com água. Não esmaece; motivo na lista |
| `7002dbe11` | **Blend no Wet Paint**: `WetSession::blend` (congelado no nascimento, `wetpaint::modo_da_sessao`), `composite.rs` por `blend_over` (Mix ao byte); trocar o modo com tinta molhada fixa a sessão (`set_brush_blend`). Erase/Add Alpha = Mix na água. Gates `wetpaint/blend_tests.rs` (2) |
| este | este handoff |

## §3 — Medido e RECUSADO nesta onda (não reconstruir)

| proposta | o que a medição disse |
|---|---|
| esmaecer o **Jitter Rotate** na Aquarela (Automatic ligado) e no Wet Paint (sem Shape/Grain que siga o carimbo) | o gate do esmaecido leu 12 linhas a mentir: a tinta muda POR ACIDENTE noutros estados — o sorteio do ângulo vem antes da cor em `jitter::per_dab` e desloca-lhe o fluxo; a pegada rodada arredonda diferente no `falloff_t`. Fica na lista do censo |
| esmaecer o **Automatic** (Shape da aquarela) | é a porta das opções da Shape (desligá-lo põe o Falloff Watercolor, o MESMO carimbo de propósito) — esmaecê-lo esconderia o caminho |
| esmaecer o **Spread** | age num sentido (para baixo); «inerte agora» mentiria |
| cortar a união DENTRO de uma poça (secar a frente que recua) | ver `53a04d728`: aro novo no lado molhado, re-depósito no seco — a separação tem de ser > 2× o alcance |

## §4 — O que falta (ordem do plano 46 §2)

1. **Item 7 — Solid, Shape Color Ramp e fios (Sketchy/Wire/Rungs) na Aquarela.** A análise do doc 46 §1:
   Solid = a região fechada entra na cobertura e na cor da sessão como mais um carimbo
   (`solid_deposit.rs:58` `solid_owns_the_gesture` exclui a aquarela hoje); a rampa entra no splat
   da cor (`watercolor_accum_cor.rs`; hoje a aguada lê só a LUMINÂNCIA da Shape,
   `watercolor_accum.rs:245`); os fios entram como cobertura fina (`threads_own_the_gesture`
   exclui `brush.watercolor`, `thread_deposit.rs:64`). ⚠️ qualidade da borda escura num fio de 1 px:
   OLHE a imagem. Cada um sai da lista do censo quando passar a agir (o gate obriga).
2. **Item 8 — Composite Brush no Wet Paint** (cada camada → a ferramenta da água do mesmo gesto:
   Brush → depósito, Smear → Smear, Blur → Blend, Eraser → Erase, `dispatch_pressure_dab_tool`).
   **Kill-criterion antes do build:** o quadro da pilha cheia no W não pode passar o do Composite no
   Digital — meça os dois primeiro.
3. **Item 9 — Solid e fios no Wet Paint** (porta nova no `ph2d-wet-paint` para a poça do Solid;
   fios = centenas de carimbos de água por quadro numa teia densa — mesmo kill-criterion).
4. **Aberto achado pelo censo:** os fios **Sketchy no Impasto** só agem com Solid ligado, e a porta
   dos fios (`threads_own_the_gesture`) não exclui o Impasto — achar o porquê (linha da lista).
5. **O fecho da linha** (DIRETRIZ §1.5.9) depois disso: gate batched sobre o diff desde `1ad60a1ce`,
   os três gates da `ph2d-panel-registry-init` no âmbito do WORKSPACE (o briefing da troca diz quais),
   handoff de INTEGRAÇÃO cobrindo todos os commits, `rm -rf target/*/incremental`, o smoke 2×.

## §5 — O que custou tempo nesta onda

- **Prova de mutação que passa não prova o gate — prova a RÉGUA errada.** M2/M3 do assar
  sobreviveram à 1.ª versão dos gates: o assar acontecia DURANTE os traços de manter-a-sessão-viva,
  antes da fotografia «antes», e os dois lados da comparação já estavam assados. A cura foi uma régua
  de que o evento aconteceu entre as fotografias (`stroke_coverage` > 0 antes, 0 depois).
- **O explorador adivinha mecanismos.** Ele disse que o Bug #33 era o RNG consumido pela base do
  Grain; o código mostrou que o Grain armado SUBSTITUI as cerdas da água. E disse que o Sketchy no
  Impasto age com Solid desligado — o censo mede o contrário. Confirme no código antes de escrever.
- **O censo do Grain *Image*:** a opção de menu identifica-se pelo id da OPÇÃO (`…_option_id(k)`),
  nunca pela posição `k` no menu aberto — o índice da tela e o do enum não coincidem.
- Flake de carga confirmado de novo (3/3 sozinho): `the_mask_stroke_cost_does_not_follow_the_canvas`.
