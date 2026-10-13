# ADR-0186 — A macro grava COMANDOS RESOLVIDOS no relógio da macro, e cada família garante a sua exatidão

- **Status:** Accepted — o PRODUTO é decisão do DONO (13/10: *«tocará em tudo e em qualquer lugar, inclusive
  dentro dos jogos»* · tutorial *«os dois»* · *«se o caminho do traço e o tempo da simulação forem considerados
  pela macro, teremos resultados iguais»*); o desenho técnico é delegado.
- **Data:** 2026-10-13
- **Linha:** `line/Macros-Automação` (onda 1, W1; emenda da W2 — a trilha do Painter — na §Emenda W2)
- **Amparo:** [estudo](../../Macros/00_estudo_inicial.md) · [plano da onda 1](../../Macros/01_plano_onda1.md)
  (estado da arte com fontes, números das sondas) · sondas `#[ignore]` no commit `84523b5c6e`.

> ⚠️ **Número PROVISÓRIO** (§5.0): contado em 13/10 contra o máximo do `main` (`0185`). Quem integrar reconta-o.

## Contexto — a força que obriga a decidir agora

O dono quer assets que NASCEM à frente do jogador: uma imagem transparente que recebe as pinceladas do artista
até virar um splash, um grafo do Motion que se monta nó a nó — e tutoriais em tempo real que demonstram o passo
ou esperam o aluno. O PH2D não grava nada disso: o Painter guarda pixels (`persist.rs`), o undo da shell guarda
ESTADOS sem tempo (`undo.rs`), não há gravador, tocador nem tutorial no app.

Medido nas sondas: o Wet Paint corre num worker no relógio dele e duas corridas iguais divergem (24 239 texels
a 1024²); o Motion dá resultados diferentes conforme o número de tiques entre os passos (3,59 unidades).

## Decisão

1. **O que se grava é o COMANDO DE DOMÍNIO RESOLVIDO**, no funil que já existe em cada família (as portas
   do `PainterTool`, os drenos de intenção do Motion) — nunca cliques de ecrã, nunca estados, nunca código
   por ferramenta. «Resolvido» = alvo por id/ordem de criação, sem contexto implícito (seleção, cartão aberto).
2. **O tempo é o da MACRO**: microssegundos desde o armar, fornecidos pelo chamador. O núcleo não lê o relógio
   de parede (gate `the_macro_core_never_reads_the_wall_clock`).
3. **A exatidão é da FAMÍLIA**, gravada dentro da carga: o Wet Paint grava a **FITA DO WORKER** — cada
   resposta da thread da água à thread do quadro (o motor voltou no estado `(sim.frame, estágio em voo)`,
   não voltou, o contador de passos), pela ordem em que se pedem — e o tocador responde pela fita e anda o
   motor local até cada marca (§Emenda W2); o Motion grava o tique da cena de cada passo.
4. **O núcleo `ph2d-macro` não conhece família nenhuma** (ADR-0075: desacoplar por dados, não por plugin):
   trilhas `{ família, alvo StableId, estado inicial, eventos }`, a carga é bytes da família.
5. **Formato versionado com leitura tolerante** (molde `.riv`): `PH2DMACRO` + major/minor + campos com
   etiqueta e comprimento; etiqueta desconhecida salta; major diferente recusa com erro. Fixturas congeladas
   à mão.
6. **Gravabilidade é contrato de cada porta** (`Door`/`Recordability`): o que não se grava é RECUSADO com o
   porquê, e o censo da família reprova uma porta nova sem decisão.
7. **O tocador não depende do editor**: despacha por ordem global de tempo (empate → trilha menor) e as
   famílias dirigem os MOTORES (Painter headless por objeto, palco do Motion), para viajar com o jogo.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Gravar vídeo / sequência de imagens (Procreate, Krita, Clip Studio) | não é um processo: não muda de tamanho, de cor, de semente, nem serve de tutorial |
| Gravar eventos de rato e teclado da janela | dependem do layout, da escala da interface e do contexto — a falha do Blender Info/Replay |
| Reutilizar o undo por DIFF | guarda estados, não intenções nem tempo; reproduz o fim de cada passo, não o gesto |
| Gravar código por ferramenta | o Krita removeu o seu gravador exactamente por isto |
| Script Lua como formato da macro | frágil a versões e a contexto (Blender); a lógica, quando for precisa, entra como nós |
| Contar segundos de parede ou `sim_steps()` do worker para a água | o worker dá os passos que a máquina deixa, e `sim_steps()` não vê passos fechados por outra porta (medido: frame 34 × contador 33) |
| `postcard` para o envelope | posicional: um campo novo parte os leitores antigos; a carga das famílias pode usá-lo por dentro |
| **Drenar o passo em voo no 1.º lote molhado** de cada traço e gravar `sim.frame` (a lei do plano da W1) | muda a PINTURA de todos para a gravação caber: até um passo inteiro de espera no toque (`39,5 ms` a 4096², um engasgo no gesto), e não cobre a Água ao vivo (o lote adiado pousa no tique, a meio de qualquer passo) nem o knob que o tique reconcilia com a água a correr. A fita é exacta por construção e não muda lei nenhuma |

## O preço

- Cada família escreve a sua trilha (gravador nas portas, tocador, censo) — W2 Painter, W3 Motion; Flip,
  Vetor e o resto depois.
- **Armar a gravação encerra a sessão molhada viva** (a porta de sempre, `wetpaint_end_session` — o fim da
  sessão É o bake): a água que ainda corria pára, como no desfazer e na troca de camada. A água é estado de
  TELA, não de documento (`tool/paint/wetpaint.rs`), e o estado inicial é o documento.
- Na W2 só o **Wet Paint** grava: armar noutro meio recusa-se, e trocar de meio a gravar deixa a recusa na
  gravação (o pincel dos outros meios não está no estado inicial — o tocador pintaria outra coisa em
  silêncio, medido ~1 750 texels a 128²).
- O componente `Macro` sobe o `PROJECT_SCHEMA` uma vez (contado na integração).
- O tocador faz na thread dele os estágios que o worker fazia em paralelo; o composite segue os tiques do
  original (uma vez por quadro, como ao gravar). Medido sob carga 61–66 (sem valor de veredito, §5.0):
  quadro mediano a tocar `6,6` (1024²) e `7,2 ms` (2048²) contra `5,8` e `8,9` do original — re-medir com a
  máquina calma antes do smoke de performance; se não couber, o tocador entrega os estágios a um worker
  PRÓPRIO até a próxima marca da fita (que ele já conhece de antemão).

## Emenda W2 (2026-10-13) — a trilha do Painter

- **A única entrada não determinística da água é o worker.** Tudo o resto que a thread do quadro faz (os
  dabs, os knobs, o composite, a fila de lotes adiados da Água ao vivo) é função dos passos gravados e das
  respostas dele. A fita (`wetpaint/fita.rs`) grava, nas portas do `WetSession` (`bring_home`,
  `try_bring_home`, `try_bring_home_now`, `sim_steps`), cada resposta; o tocador põe o motor num lugar
  próprio (`EngineSlot::Roteiro`) quando o original o entregaria ao worker, e ao voltar anda estágio a
  estágio (`Engine::step_stage`, o MESMO que o worker corre) até a marca. Pergunta trocada, fita acabada ou
  marca inalcançável = divergência acusada, nunca um passo saltado.
- **As portas** (`macro_track/portas.rs`, censo com gate): 421 `pub fn …(&mut self` do `PainterTool` + os
  13 dos contratos = 434; 89 decididas uma a uma (48 gravam, 37 recusam com o porquê, 4 de ambiente) e
  345 internas (só o painel lhes chega, pelo `handle_panel_event`, que grava; uma chamada de fora a uma
  delas reprova o gate). Só a porta de FORA grava.
- **O estado inicial** (`macro_track/inicial.rs`): o documento pela porta do disco (`PaintedDocument`,
  postcard, versão própria presa por impressão digital) e as configurações por NOME (JSON): o preset do
  Wet Painter, os campos da obra e do gesto (`CAMPOS_DA_MACRO`) e o estado do meio e do traço (a semente
  incluída). Completude gateada pelo censo dos controlos: os 276 controlos do painel do Wet Paint chegam à
  reprodução, mexidos antes de armar E a gravar.
- **Posições relativas** ao quadro de referência (`f64`): na mesma tela voltam ao mesmo `f32`; numa tela 2×
  o traço pousa no mesmo lugar relativo (o tamanho do pincel continua em pixels).
- O núcleo ganhou `ph2d_macro::rec` — a lei dos campos etiquetados do ficheiro, aberta às famílias (uma
  porta só: o `codec` usa-a também).
