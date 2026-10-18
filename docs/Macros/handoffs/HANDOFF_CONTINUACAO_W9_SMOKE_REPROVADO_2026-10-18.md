# CONTINUAÇÃO — W9: o smoke do dono reprovou o passo 2 (2026-10-18)

> Linha `line/Macros-Automação`, HEAD `f6a14a4410` (W8 + W9 por integrar; [handoff W9](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W9.md)).
> NADA consertado ainda: três medições dizem «certo» e o ecrã do dono diz «errado» ⇒ o defeito está num
> caminho do APP que nenhum gate percorre.

## O relato

Cena `PH2D_MACRO_SMOKE=5`, passo 2 do smoke da W9 (Record, 2 pinceladas Wet Paint, Stop, logo ⏮ e ▷ na
aba Keys): **«os dois traços aparecem de uma vez»** — em vez de um e depois o outro.

## O que já foi medido (18/10) e REFUTA

| Hipótese | Medição | Resultado |
|---|---|---|
| A marca da gravação nova apanha as pinceladas (passo que fecha depois do Stop) | Record → 2 pinceladas → Stop pelo PAINEL (`gate7` `Banca`, Painter do editor), 600 quadros de água depois | época nova: **0** texels marcados; as pinceladas marcam-se na época ANTIGA (9 560), que é o certo |
| O palco não segue a gravação nova quando ela substitui uma macro que já tocava (a `=5` traz o Splash) | palco com a macro A a tocar, gravação B (worker real) por cima, ⏮ e ▷, cada quadro contra a reprodução à parte de B | **ranhura = palco ao byte** em todos os quadros |
| A reprodução de uma gravação ao ritmo da parede mistura as pinceladas | gravação com quadros de 16 ms REAIS, worker real, 1 s parado entre elas; `MacroEmCena::segue` a cada 0,05 s | pincelada de cima 0,55–0,75 s, a de baixo 1,80–2,00 s: **separadas**, certo |
| No ecrã, outro caminho do Painter passa à frente do override da macro | leitura (explorador): `fase_extract_inputs.rs` põe a macro ANTES dos overrides do editor; `sim_extract.rs:331-334` fica com o 1.º por entidade; um quad por sprite | a macro ganha a textura da sprite. Exceções NÃO verificadas: o véu da tinta molhada e as sobreposições Vello (`painter_bridge_wetness.rs:97`), e os vizinhos do Repeat Image |

## O que falta (o próximo passo, num bloco)

1. **Ver o app a fazer o caminho do dono**: uma cena `PH2D_MACRO_SMOKE=6` que, sozinha e por NÚMERO DE
   QUADRO (determinista), arma a `=5` e faz Record (`MacroRequest::Grava`), duas pinceladas pelo Painter
   do editor (`on_canvas_pointer`), Stop, ⏮ (`TimelineIntent::Scrub(0)`) e ▷ na aba Keys, e escreve na
   consola, a cada quadro, o instante do clipe (`MacroTempo`), se o palco trocou a sprite (`TrocaDaSprite`)
   e os texels não brancos da ranhura e do editor. Fotografar com `fotografa_cena.sh` em 4–6 esperas
   (antes do Stop, no ⏮, a meio da 1.ª pincelada do clipe, entre as duas, no fim). `cenas_do_smoke`
   (`CENAS_DO_SMOKE` 5 → 6).
2. Hipóteses ainda vivas a confirmar nessa corrida: (a) o **véu da tinta molhada** (Vello) desenha a água
   do editor por cima do clipe enquanto ela corre; (b) o relógio da aba Keys não anda / o `MacroTempo`
   salta (o laço re-armado pelo `SetLoop` da W9, o ⏮); (c) `sprite_gravada()` fica `Some` depois do Stop
   (o palco cede e mostra o editor); (d) o preview GPU do Painter numa via que não passa pelo override.
3. Achada a causa: o gate que faltava percorre o caminho do APP (a ordem da shell), não só o palco.

## Prompt para a janela nova

```
Assuma a linha line/Macros-Automação (worktree Worktrees/line-Macros-Automação). Rota «linha
reaberta» de docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md: cd + pwd + git branch
--show-current antes de abrir ficheiros; git cherry main HEAD (W8 e W9 por integrar: NÃO faça reset
nem rebase). Leia docs/Macros/handoffs/HANDOFF_CONTINUACAO_W9_SMOKE_REPROVADO_2026-10-18.md inteiro
(o meu smoke: «os dois traços aparecem de uma vez»; o que já foi medido e refutado; o próximo passo).
Faça o passo 1 (a cena =6 que faz o meu caminho sozinha, fotografada), ache a causa, cure-a com o
gate que percorre o caminho do app, e me devolva o binário de smoke compilado com os passos
numerados e as fotos.
```

## Passo 1 feito (18/10, 2.ª janela) — a cena `=6` NÃO reproduz o defeito aqui

- `PH2D_MACRO_SMOKE=6` (`CENAS_DO_SMOKE` 5 → 6): a `=5` montada e o caminho do dono conduzido no relógio da cena
  pelas portas da shell — `MacroRequest::Grava/Para` (o que o painel MACROS empurra), `painter_canvas_down/move/up`
  no ecrã (pose de MUNDO da sprite), ⏮ = `Scrub(0.0)` + `timeline_reveal_after_apply`, ▷ = `TogglePlay`
  (exactamente o `intent_for_transport` da shell). Roteiro e diagnóstico em
  `crates/ph2d-app-painter/src/macro_track/caminho.rs` (`PalcosDoPainter::diagnostico`: toca, época, troca como
  o `sobe` decide, texels pintados no palco / marca / o que a ranhura mostra / editor); a shell só executa
  (`render_loop/macro_smoke.rs::caminho_do_dono`). O registo vai à consola E a `$TMPDIR/ph2d-cena6.log`
  (com os quadros por 100 ms). Variantes por `PH2D_MACRO_CAMINHO=humano|meio|splash|semrewind`.
- **Medido (uma rodada, 5 variantes, fotografadas em rajada; `load` 256 em 32 núcleos ⇒ o app a 9–25 fps):** em
  TODAS a ranhura = palco, `troca=true` depois do Parar, marca da época nova = 0, e a 1.ª pincelada aparece no
  instante dela (base: clipe 1,0–1,6 s) e a 2.ª no dela (2,6–3,2 s) — fotos: Time 0,63 branco · 2,68 só a de cima ·
  3,62 as duas. Base, ritmo humano (pausas de 2–4 s, água a secar), Wet Paint escolhido DEPOIS de Gravar,
  Splash tocado e pausado antes de Gravar, Parar+▷ sem ⏮: as cinco certas.
- Única anomalia vista: no 1.º–2.º quadro depois do Parar a `troca` já é `true` com a marca da época ANTIGA
  (48 939 texels) ⇒ a ranhura mostra as duas pinceladas por 1–2 quadros (`p.chave` muda no `segue` do quadro
  seguinte). Um piscar, não «aparecem de uma vez» no Play — fica registado, não curado.
- ⇒ O que resta de diferente entre esta máquina e o ecrã do dono é a VELOCIDADE (o dono corre a centenas de fps)
  e o que o dono fez à mão que nenhuma variante tem. **Próximo passo:** o dono corre a `=6` no próprio ecrã sem tocar
  no rato e diz se as pinceladas vêm uma de cada vez; o `$TMPDIR/ph2d-cena6.log` dessa corrida diz o resto (fps, `troca`,
  ranhura × editor por 100 ms). Se ali sair certo, a diferença está num gesto feito à mão — pedir ao dono a sequência exacta.

## CAUSA achada e curada (18/10) — o Play de CIMA, e o Autoplay que a 2.ª tomada herdava

- O dono confirmou: a `=6` sai certa no ecrã dele; gravar à mão não. A `=5` passou a registar o caminho à mão
  (o mesmo registo da `=6`, sem gestos, `$TMPDIR/ph2d-macro-smoke.log`, com os pedidos à Timeline de cada quadro).
  O registo do dono (60 fps): Parar → `SetLoop` (o rearme da A4) → **`RunScene`** — o Play da barra de CIMA
  (`TransportCmd::Play` → `ph2d_transport::runtime_intent`), sem `Scrub` nem `TogglePlay`. Daí em diante
  `t=None`, `toca=false`, `troca=false`: o `MacroTempo` some, o palco sai de cena e a sprite mostra a própria
  imagem (a final, `editor=20097`) ⇒ «os dois traços aparecem de uma vez».
- Porquê: o Play de cima só toca os clipes em **Autoplay** (lei do dono, a Godot). `instala::clipe` dava
  Autoplay ao clipe NOVO, mas ao re-cronometrar o clipe que o objecto já tinha deixava o Autoplay de antes — e
  a `=5` monta-o desligado (abre parada). Cura: o Parar dá sempre o mesmo clipe, em Autoplay
  (`crates/ph2d-app-macros/src/instala.rs`). Gate `gravar_de_novo` (o painel MACROS clicado, sobre um clipe sem
  Autoplay): vermelho antes da cura, verde depois.
- Prova no app: `PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=topo` (Parar e o Play de cima) — `t=Some`, a de cima
  aos 1,0–1,6 s, a de baixo aos 2,6–3,2 s; fotos Time 0 (branco, Autoplay ✓) · 1,44 · 2,05 (só a de cima).
- Aberto (baixo): o piscar de 1–2 quadros depois do Parar (a marca da época antiga com `troca=true`).
