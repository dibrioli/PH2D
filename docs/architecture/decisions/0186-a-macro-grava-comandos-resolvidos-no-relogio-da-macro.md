# ADR-0186 — A macro grava COMANDOS RESOLVIDOS no relógio da macro, e cada família garante a sua exatidão

- **Status:** Accepted — o PRODUTO é decisão do DONO (13/10: *«tocará em tudo e em qualquer lugar, inclusive
  dentro dos jogos»* · tutorial *«os dois»* · *«se o caminho do traço e o tempo da simulação forem considerados
  pela macro, teremos resultados iguais»*); o desenho técnico é delegado.
- **Data:** 2026-10-13
- **Linha:** `line/Macros-Automação` (onda 1, W1)
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
3. **A exatidão é da FAMÍLIA, por marca de passo**, gravada dentro da carga: o Wet Paint grava `sim.frame`
   (lido depois de drenar o passo em voo no 1.º lote molhado do traço) e reproduz em relógio fixo até essa
   marca — medido: 0 texels em 4/4 corridas; o Motion grava o tique da cena de cada passo.
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

## O preço

- Cada família escreve a sua trilha (gravador nas portas, tocador, censo) — W2 Painter, W3 Motion; Flip,
  Vetor e o resto depois.
- **Lei nova no Wet Paint, para todos:** o 1.º lote molhado de um traço drena o passo em voo (no máximo um
  passo; mín. medido `1,67 ms` a 1024², `39,5 ms` a 4096²), só quando há passo em voo.
- O componente `Macro` sobe o `PROJECT_SCHEMA` uma vez (contado na integração).
- A 2048²+ a água ao vivo não cabe em 60 Hz enquanto o composite correr por passo (medido `29,7 ms`);
  compor uma vez por quadro entra na onda.
