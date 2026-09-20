# HANDOFF DE INTEGRAÇÃO — `line/components` · **AS ÂNCORAS DO HUD** (2026-09-19)

> Leitor: a próxima LLM e o agente INTEGRADOR. Denso de propósito (`CLAUDE.md` §0.8).
> Sonda do §5.0:
> [`mede_o_que_falta_as_ancoras_do_hud`](../../../crates/ph2d-app-components/tests/it/mede_o_que_falta_as_ancoras_do_hud.rs)

## §1 — O que a jornada entrega, em uma frase

**O placar cola-se ao canto de verdade.** Um filho de um canvas de HUD com regra de âncora segue as
bordas **REAIS** da vista quando a janela muda de aspecto, em vez de ficar a uma banda de letterbox
delas. É o item que o [handoff do #20](HANDOFF_INTEGRACAO_line_components_HUD_2026-09-17.md) §7
deixou aberto — e com ele o **#20 fecha**.

## §2 — ⛔⛔ A sonda do §5.0 CORRIGIU a redacção do item aberto

Ele escreve *«as quatro âncoras do `VecAnchors` **não estão LIGADAS** ao canvas»*. Medido:

| bloco | a resposta MEDIDA |
|---|---|
| **A) a BANDA** | com `Keep`, `21:9` deixa **`5,0`** de mundo por preencher em `x`; `4:3` deixa `3,0` em `y`; `16:9` deixa **zero** |
| **B) o CANTO** | um filho no canto da caixa de referência pousa em `16,0` e a borda real está em `21,0` — **faltam `5,0`, que é exactamente a banda** |
| **C) a ÂNCORA de hoje** | `delta_local` devolve **`0,0`** — e **por construção** |
| **D) o CONTRAFACTUAL** | com a caixa **EFECTIVA**, o erro à borda é `0,000000` nas três janelas |

⭐⭐⭐ **Elas estão LIGADAS e são INERTES.** O `delta_local` pergunta *«a moldura mudou de
tamanho?»* e a caixa de um canvas é **a mesma em toda janela** — o que muda é a **ESCALA** da raiz
⇒ o delta sai `0,0` por subtracção de iguais. *Não era um fio por ligar: era a régua a medir uma
grandeza que não se mexe.*

## §3 — Os CONTADORES (⛔ nunca o literal)

| contador | delta |
|---|---|
| `PROJECT_SCHEMA` | **0** |
| registo do `ph2d-ecs` + os dois espelhos | **0** |
| componentes novos | **0** — o `VecAnchors` já existia e já era registado |

⭐ **A wave é uma PORTA e uma SELECÇÃO, e nada mais.** Nenhum tipo novo atravessa o ficheiro.

## §4 — As decisões

### §4.1 — ⭐ ZERO lei de âncora nova

O `LayoutLive::anchor_kid` do passe das molduras é o **mesmo**, e é por isso que o **PINO**
(`min == max`) e o **ESTICÃO** (`min != max`) saem os dois de graça. Construir uma segunda lei
teria sido *«a forma mais cara de ignorar a §5.0»* — a frase que o próprio `ph2d_ecs::hud` já
escreve.

### §4.2 — A caixa EFECTIVA, e o neutro EXACTO

[`ph2d_hud::effective_box`] = a de referência **mais a banda**, em unidades locais. A aritmética
simplifica para **`half / escala`** — *a vista, vista de dentro do canvas*.

⭐ No aspecto da própria caixa a banda é zero, a efectiva **É** a de referência, o afim é a
identidade e o `anchor_kid` sai no `is_identity` **sem pagar a cópia da geometria** ⇒ o que se
desenha é **byte-idêntico** ao de antes. *Sem isto, toda cena de HUD já autorada mudava de imagem.*

### §4.3 — ⭐⭐ E a recusa declarada do `Fit::Expand` DISSOLVE-SE

O cabeçalho da `ph2d-hud` recusava o `expand` porque *«fazer os filhos chegarem à borda exigiria
redimensionar a moldura por quadro, que é escrever no DOCUMENTO»*. ⚠️ **A premissa caiu:** a caixa
efectiva é **derivada por quadro**, como a pose da raiz, e não toca no documento. *Quem move o
número que tornava algo inalcançável tem de reconferir a nota* (§0.0) — a nota está reescrita no
doc da porta, e o `expand` fica como wave possível e não como impossibilidade.

### §4.4 — Onde cada metade mora, e porquê

| metade | onde | razão |
|---|---|---|
| a caixa efectiva | `ph2d-hud` | é aritmética pura sobre dois rectângulos |
| a moldura que um canvas oferece | `hud_bridge::anchor_frame_of` | a escala é a que **conduziu** a raiz (`place`), nunca uma segunda medição |
| **quem** ancora quem | `ph2d_app_components::hud_anchors::ancorados` | pergunta sobre COMPONENTES ⇒ gates sem cena, sem device |
| publicar a pose | `shells/desktop/src/layout_live_anchors.rs` | é a maquinaria que o passe das molduras já tem |

⭐ As duas populações — filho de moldura, filho de canvas — são **disjuntas por CONSTRUÇÃO**: um
filho tem UM pai, e ele ou tem `VecFrame` ou tem `UiCanvas`.

## §5 — ⚠️⚠️ O que este diff MEXE em ficheiro PARTILHADO

| ficheiro | o que muda | risco |
|---|---|---|
| `shells/desktop/src/layout_live_anchors.rs` | **+ uma função no FIM** (`anchor_canvases`) | aditivo — nenhuma linha existente mexida |
| `shells/desktop/src/layout_live.rs` | `+1` campo (`vista`) e `+1` chamada | aditivo |
| `shells/desktop/src/render_loop/fase_hud.rs` | `+1` linha (guarda a vista) | ⚠️ **a ORDEM é load-bearing** e tem gate |
| `shells/desktop/src/render_loop/fase_vector_layout_recook.rs` | `+1` linha (lê a vista) | idem |
| `shells/desktop/src/app_state_components_smokes.rs` | `+1` campo em `HudShell` | ⭐ **zero campos novos na `App`** — a catraca `the_app_only_sheds_fields` não se mexe |

## §6 — ⭐⭐ A CATRACA DA SHELL reprovou, e a cura foi MOVER

`the_shell_only_shrinks` leu **`197 288`** contra `196 990`. ⛔ **Subir o número está fora.** Três
movimentos, todos por RESPONSABILIDADE:

1. a **selecção** (*quem ancora quem*) foi para a crate da família, onde tem gates baratos;
2. a **cena** das âncoras foi para casa (`hud_smoke_anchors`), como as irmãs desta linha — ela
   nasceu na shell por inércia;
3. ⭐ saiu a sonda **`probe_cursor_grab`** — `295` linhas que **nada chamava** (o único sítio que a
   nomeava era o `mod` do `main.rs`), agora [`ph2d-probe-cursor-grab`](../../../crates/ph2d-probe-cursor-grab/)
   com o gate dela. *O candidato veio de uma MEDIÇÃO e não do tamanho.* ⚠️ A pergunta que ela
   responde é do plano da **UI** — isto é uma mudança de ENDEREÇO, não de dono.

Shell: **`196 835`**.

⛔⛔ **E apagar aquele `mod` re-ligou o `#[cfg(test)]` dele ao VIZINHO** — `19` erros de compilação
sobre o `profile_live`. *A armadilha está escrita no repo (handoff da física, Fase A §9) e aqui foi
**BARULHENTA**, que é a única sorte da história.*

## §7 — A prova de fecho

| o quê | onde | número |
|---|---|---|
| a caixa efectiva | `ph2d-hud/src/tests.rs` | **5** gates |
| a selecção | `ph2d-app-components/src/hud_anchors_tests.rs` | **5** gates |
| o que se DESENHA | `shells/desktop/src/layout_live_anchors_canvas_tests.rs` | **4** gates |
| a ORDEM no quadro | `shells/desktop/tests/it/o_hud_conduz_a_raiz_antes_de_as_ancoras_a_lerem.rs` | **2** gates |
| a CENA | `…/o_rotulo_do_botao_do_hud_e_filho_dele.rs` | **+1** gate |
| **provas de mutação** | `docs/Components/ferramentas/mutacao_ancoras_hud_2026-09-19.sh` | **8 de 8 sangram** |
| **portão** | `scripts/nextest-impacted.sh` | **15 458 de 15 458** · clippy `-D warnings` a zero · `fmt` limpo |

## §8 — ⛔⛔ DUAS mutações sobreviveram primeiro, e as duas eram MINHAS

| # | o que aconteceu | a lição |
|---|---|---|
| **a cerca do PAI** | apagá-la não sangrava porque o chamador **voltava a perguntar** `get::<UiCanvas>(root)` — a cerca estava escrita **DUAS vezes** | *Uma mutação **neutralizada** por uma segunda guarda lê-se como sobrevivência num relatório e não é.* ⇒ a cura não é um gate a mais, é a **segunda resposta a menos**: a porta devolve o PAR |
| **a escala da moldura** | trocá-la por `1` não mudava nada, porque a fixtura estava no ponto **NEUTRO** (`Keep` a `42×18` dá `s = 1,0`) | *Um corpus no ponto NEUTRO de um knob não testa esse knob* ⇒ gate novo com `s = 2,0` |

⚠️ E **um gate meu era fraco**: ele dizia só *«o filho de moldura não andou `+5`»*. Com a cerca
apagada ele era ancorado pelos DOIS passes e o valor final continuava a não ser `5` ⇒ a asserção
passou a ser **EXACTA** (`[16, 26]`). *Um gate que recusa UM valor não afirma qual é o certo.*

## §9 — ABERTO, e de quem é cada item

| item | de quem |
|---|---|
| o **`Fit::Expand`** como terceiro modo | **produto** — a recusa dissolveu-se (§4.3), a wave é pequena, e o dono decide se o quer |
| **no EDITOR** um HUD colado às bordas cai atrás dos painéis | **decisão de produto** (a vista da câmera é a da JANELA) |
| o `UiButton` só é alcançável por caminho **vectorial** | nomeado, não construído |
| a âncora de um filho armada **na janela errada** grava a caixa daquela janela | a cena arma-a contra a de REFERÊNCIA; um gesto de painel para HUD ainda não existe |

## §10 — O SMOKE

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_HUD_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

A contagem fica em baixo à **esquerda** e os pontos em baixo à **direita**. **Arraste a borda da
janela** para a alargar: as duas seguem as bordas reais. Antes desta wave ficavam paradas a uma
banda delas.
