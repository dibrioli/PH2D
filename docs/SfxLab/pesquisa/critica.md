# Crítica de design de áudio: o criador de efeitos sonoros do PH2D

*Para o Enio. 5 de novembro de 2026. Avalia a ideia e a tela, já sem nenhuma IA.*

## O veredito

**A ideia vale, mas a aposta principal está no lugar errado.** O relatório aposta em desenhar o som sobre a cena do jogo. Eu apostaria em algo mais simples e mais usado: **ouvir o som no jogo, sem exportar nada, trocando-o enquanto o jogo roda**. É isso que um designer de áudio faria todos os dias.

Mudam quatro coisas:

1. **O documento deixa de ser «golpe + corpo + ruído» e passa a ser uma pilha de camadas sobre uma régua de tempo.** O golpe com corpo vira um tipo de camada, útil para impactos, entre outros.
2. **A gravação própria vem primeiro, e o sintetizador depois.** Sem IA, é da gravação e do processamento dela que sai a originalidade.
3. **Conserte primeiro o caminho até o jogo.** Hoje o Editor de Áudio já sorteia variações, mas pelo que li o som do objeto guarda um arquivo só. Sem isso, o módulo produz conjuntos que o jogo não sabe tocar.
4. **Corte a pintura de som e a metáfora da planta; deixe a voz para mais tarde.** Corte também o grafo de nós como tela principal.

**Como ler este texto.** Sou uma IA fazendo o papel de designer de áudio: não tenho carreira para citar e **não ouvi nenhum som**. O que não traz fonte é **opinião de ofício**: prática que considero difundida, sem medição. Quando digo «no código», li o código do PH2D, mas não rodei o app. Nada disto substitui o teste às cegas com um profissional de verdade.

---

## Parte 1: a ideia

### A receita viva serve a um profissional?

**Serve, por um motivo diferente do que o relatório destaca.** O relatório enfatiza arquivos pequenos, mas ele mesmo admite que o ganho some quando se exporta como áudio. O valor real é outro: seis meses depois, o diretor pede «o mesmo som, um pouco mais pesado», e você abre a receita, muda uma camada e refaz as 12 variações de forma coerente. Isso vale muito.

**A estrutura golpe + corpo + ruído não serve como forma geral.** Ela descreve bem um impacto. Já um *whoosh* (o sopro de algo passando rápido), um som de interface ou uma magia não têm um golpe nem um corpo que vibra, e forçá-los nessa forma atrapalha.

O profissional costuma pensar por **função no tempo**. As partes típicas são:

- o **ataque**: o começo curto e seco, que dá a sensação de «aconteceu»;
- o **corpo**: o trecho que diz o que é o objeto;
- a **cauda**: o final que se dissipa;
- o **grave**: o peso, muitas vezes uma camada separada.

Cada parte pode vir de qualquer fonte: gravação, sintetizador ou corpo vibrante. O relatório cita algo parecido para armas, e [Stevens separa o som em partes e guarda várias versões de cada uma](https://www.asoundeffect.com/?p=27166) (vi só um trecho da entrevista). **Trate isso como a regra, e o golpe + corpo como um tipo de fonte.**

### O que ele rejeitaria no primeiro dia

| O problema | Por que derruba o módulo |
|---|---|
| **Um estouro nos fones.** Sorteio e ressonância (a vibração que se reforça e se prolonga) podem gerar um som violentamente alto | Depois do primeiro susto, ele fecha o programa. Precisa de um limitador de proteção (um teto que segura o volume) sempre ligado na escuta, que nunca entra no som final |
| **A prévia soa diferente do arquivo final** | Ele aprova um som e recebe outro. O relatório só pede que a diferença seja mostrada; eu exigiria que não exista |
| **Não dá para alinhar as camadas com precisão** | Num impacto, poucos milissegundos separam um estalo firme de um borrão |
| **Gravação própria é coisa de segunda classe** | É o material dele, o que o distingue dos outros |
| **Uma atualização do app muda um som já aprovado** | Inaceitável. O som aprovado precisa ficar guardado também como áudio |
| **Não dá para digitar um número** | Ele trabalha em dB (volume), Hz (frequência, ligada à altura), ms (milésimos de segundo) e semitons (a menor distância entre notas no piano). Exploração por gesto não substitui precisão |
| **O sorteio não respeita o que ele fixou** | Fixar é o que torna o sorteio utilizável, e o relatório acerta ao incluir isso |
| **Os arquivos saem sem padrão de nome, ou só como um som misturado** | Muitas vezes ele quer entregar as camadas separadas para acabar o trabalho em outro programa |
| **Conversão automática da gravação ao importar** | O relatório propõe taxa de amostragem interna fixa. Gravações em taxa alta (96 ou 192 mil amostras por segundo) existem para serem baixadas de altura sem perder detalhe. O ajuste interno pode ser fixo, mas essa folga do material tem de ser preservada |

**Não espere substituir o programa principal dele.** Esse programa é um DAW, o «estúdio» de áudio, como o REAPER. Um profissional que entrega para Wwise ou FMOD (os sistemas que tocam áudio em jogos de outros motores) usaria este módulo para ter ideias, gerar variações e mandar arquivos bem nomeados. Isso já é muito. Para quem faz o jogo dentro do PH2D, sim, ele pode ser a ferramenta inteira.

### Os oito gestos, um a um

| Gesto | Veredito | Uso | Por quê |
|---|---|---|---|
| **1. Ter um som em segundos** | **Manter, com outra base** | Toda semana | Sortear, mutar e fixar é um hábito excelente. O perigo é soar como bipe de jogo antigo: o relatório admite esse teto de timbre, a «cor» do som. Cada categoria deve abrir várias camadas e material bom, não um sintetizador retrô |
| **2. Cruzar sons arrastando** | **Manter, mais simples** | Às vezes ao criar; muito ao gerar variações | O arrasto em que a distância define o tamanho da mudança é bom. A descoberta é que **mutação pequena é o gerador de variações**: um mecanismo só, em duas intensidades. Tire a árvore e a semente; mostre oito opções |
| **3. Desenhar e tocar** | **Manter para o que se move** | Toda semana em whooshes, magia e movimento; quase nunca em impactos | Um gesto dá vida que curvas retas não dão. Gravar sempre, sem botão, é uma ótima escolha |
| **4. Imitar com a boca** | **Adiar e reduzir** | Toda semana, se simples | Imitar com a boca é um jeito rápido e comum de esboçar. Mas acertar uma receita pela voz é a parte não medida, e seguir a altura de um «fshhh» tende a falhar. Primeiro: gravar o microfone como material e extrair a curva de volume de qualquer áudio. É medição simples, sem IA. O microfone ainda não existe no código, e a patente citada continua sem leitura |
| **5. Compor sobre o momento do jogo** | **Dividir em dois** | Ouvir no jogo: todo dia. Desenhar sobre a cena: raramente | Ver a seção seguinte |
| **6. Amarrar o som ao objeto** | **Manter, de um jeito mais barato** | Toda semana em contatos, veículos e vento | Wwise e FMOD já ligam parâmetros do jogo ao som há anos, então a ligação em si não é novidade. O que é bom aqui é fazê-la sem programador, vendo o valor mudar ao vivo. E a maioria dos casos não precisa de síntese ao vivo (som calculado na hora): três versões assadas (já renderizadas em arquivo), uma fraca, uma média e uma forte, escolhidas pela força do contato, costumam bastar. Stray [fez algo assim](https://www.asoundeffect.com/stray-game-audio/), segundo um trecho |
| **7. Pintar o som** | **Cortar** | Quase nunca | Ótima demonstração. Controlar timbre e tempo com um pincel é difícil, e o próprio relatório diz que ninguém avaliou a utilidade para efeitos. Sei que combina com o Painter e seria linda de mostrar, mas não pagaria o espaço na tela. Esta opinião depende de ouvir: se alguém provar o contrário às cegas, ela volta |
| **8. Entregar pronto** | **Manter e pôr no centro** | Todo dia | É a menos vistosa e a que mais conquista um profissional. O teste de repetição flagra o maior defeito dos sons de jogo: soar sempre igual |

**As três curvas (volume, brilho, altura) são um bom padrão, não um limite.** Ao desenhar, comece com elas; o profissional precisa poder apontar qualquer curva para qualquer controle. E «brilho» não é um controle único, então cada som precisa dizer o que ele move.

### «Compor sobre o momento do jogo» e «amarrar ao objeto» são o diferencial?

**Em parte, e o relatório escolheu a metade menos útil do primeiro.**

Desenhar sobre a cena funciona bem em cena linear, como uma cutscene ou um evento roteirizado. Só que jogo não é filme. Um som de pulo toca em mil momentos diferentes, e ajustá-lo com perfeição a um deles não diz nada sobre os outros.

O que importa é outra coisa: **ouvir o som no contexto, com tudo em volta**. É assim que se descobrem som encoberto, repetição irritante e atraso. O relatório já registra o hábito: [em Stray, testavam no jogo e recomeçavam quando não funcionava](https://www.asoundeffect.com/stray-game-audio/) (trecho).

Proponho duas peças, e estas sim me parecem difíceis de copiar de fora de um motor:

- **Test in Game.** O jogo roda ao lado, e o som muda no próximo disparo, sem exportar nem importar.
- **Game Clip.** Uma tecla guarda os últimos segundos jogados: a imagem e a lista do que disparou, quando e com que força. Depois, esse trecho volta em loop enquanto você troca o som. Ouve-se a repetição real, com as sobreposições e os outros sons de verdade, sem precisar jogar de novo a cada ajuste. **É proposta minha; não sei quanto custa construir** e isso precisa de avaliação técnica.

**Esse diferencial só vale para jogos feitos no PH2D.** O Unreal MetaSounds já cria som dentro do motor; o relatório faz bem em dizer isso. Para quem entrega a outro motor, o diferencial é outro: fluxo rápido e entrega limpa.

### Sem IA, o que torna os sons originais

O relatório e eu concordamos que a estrutura importa mais que a técnica. Por ordem de força:

1. **Gravações próprias, inclusive as esquisitas.** Fita, alavanca de porta, legumes, a voz: a matéria-prima é o que os outros não têm.
2. **Transformação forte.** Descer duas oitavas, inverter, esticar, granular (picar em grãos minúsculos e reespalhar), usar um som como a «sala» de outro. Boa parte disso já está no rack de 42 efeitos (a fileira de efeitos do Editor de Áudio), e a qualidade de esticar e mudar a altura é **a primeira coisa a ouvir**: não consigo avaliá-la lendo.
3. **Combinações improváveis.** O grave de um som com o ataque de outro. Daí a importância das camadas.
4. **Uma camada guiando outra.** O volume de uma controla o brilho da seguinte. É um recurso simples que dá organicidade.
5. **Modelos físicos usados para o impossível.** Um sino do tamanho de uma casa. [Fournel alerta que modelos de fornecedor ficam presos à ideia do fornecedor](https://cedec.cesa.or.jp/2010/archive/files/C10_P0051/C10_P0051.pdf): as peças devem ficar expostas.
6. **Exploração com faixas pensadas.** Sorteio solto gera lixo; sorteio com limites de categoria gera surpresas úteis.

**O maior risco é o módulo ter um «som de fábrica» reconhecível.** Modelos prontos e bonitos são o equivalente sonoro do «quadrado branco» que a crítica aponta. Eu faria modelos que soam bem, porém **visivelmente incompletos**, com um espaço pedindo o seu material. Também é preciso **material de partida gravado** para o módulo. Sem isso, só sobra síntese, e [Farnell admite que seus sons sintéticos não tinham muita qualidade](https://designingsound.org/2012/01/18/procedural-audio-interview-with-andy-farnell/). Essa é uma das suas decisões.

### O que falta e é indispensável

| Falta | Como deve ser |
|---|---|
| **Camadas de verdade** | Volume, panorama (esquerda/direita), atraso em ms, altura, mudo e solo, por camada |
| **Comparação A/B com referência** | Uma tecla alterna entre o seu som e um som de referência, **sem pausa e no mesmo volume**, porque o mais alto quase sempre parece melhor. [Jurassic World Evolution comparava em sequência contra referências](https://www.asoundeffect.com/jurassic-world-evolution-sound/) (trecho). Inclua também A/B entre duas versões suas |
| **Sala e reverb, em dois tipos** | Reverb é a reflexão do som em um espaço. O **reverb do som** faz parte dele e vai para o arquivo. A **sala de escuta** só serve para ouvir e **nunca** entra no arquivo. Confundir os dois deixa o jogo com salas sobrepostas. O padrão é entregar sem a sala |
| **Teste em outras saídas** | Ouvir como em celular, notebook e de longe, e em mono (um único canal). O grave que impressiona nos fones pode sumir num celular. O rack já tem um efeito de telefone; pode ajudar |
| **Mixagem do som inteiro** | O mesmo rack de efeitos, aplicado à camada ou ao som todo |
| **Volume por categoria** | Ver abaixo |
| **Equilíbrio do conjunto** | Mostrar variação fora da curva e corrigir, respeitando diferenças intencionais (fraco e forte) |
| **Regra de disparo** | Ordem, «não repetir os últimos N» ([recurso do Wwise](https://www.audiokinetic.com/qa/8949/how-can-i-avoid-repetition-from-different-containers?show=8960), visto num fórum deles), faixa de altura e volume, limite de vozes simultâneas, tempo mínimo entre disparos |
| **Nomes e categorias** | Um modelo de nome **que você escreve**, como `{object}_{event}_{n}`, sem sugestão automática. Opcionalmente o padrão UCS, um sistema público de categorias usado na indústria, no formato [`CatID_FXName_CreatorID_SourceID`](https://www.spencerbruce.com/blog1/2025/10/3/understanding-the-universal-category-system-ucs) |
| **Exportação em lote** | Conjunto inteiro, camadas separadas e formatos por plataforma, com uma tabela do que saiu. Uma tecla repete a última configuração. Só refaz o que mudou. **Nada de ligação direta com Wwise e FMOD no início**: arquivos bem feitos bastam |
| **Gravações próprias** | Arrastar arquivo, gravar do microfone, cortar uma tomada em vários golpes, etiquetas e busca por **texto que você escreveu**. O material fica no projeto. Um documento antigo do módulo registra caminho absoluto na lista de variações; **não copie isso**, porque o projeto quebra ao mudar de máquina (não conferi se ainda é assim) |
| **Versões** | Histórico com nome, versão marcada como aprovada, alternativas por camada, e o áudio aprovado guardado com a receita. Se o motor mudar, um aviso igual ao dos pincéis: «estes sons mudaram», e você decide |
| **Som de partida correto** | Sem silêncio no começo, que soa como atraso, e sem corte seco no fim |
| **Limitador de proteção** | Já explicado acima; sem ele, nada mais importa |

**Sobre volume, o relatório está certo em não prometer um número por som.** Mas há um detalhe: LUFS, a unidade de volume percebido, [mede em janelas de 400 ms](https://www.itu.int/rec/R-REC-BS.1770), então não descreve bem um clique de 50 ms. Por isso:

- mostre **pico** (o ponto mais alto) e o **maior volume percebido no trecho**;
- deixe **você** definir a tabela de categorias do projeto (por exemplo, UI mais baixo que uma explosão) e sinalize quem saiu dela;
- use com cuidado o ajuste em lote do Editor: ele leva todos os arquivos ao mesmo volume, e isso achata a hierarquia do jogo se for usada entre categorias.

### Como conviver com o que já existe

**Módulo separado na tela, mesma base por baixo.** Editor e criador têm naturezas opostas:

- o **Audio Editor** é cirurgia em um arquivo: cortar, limpar, consertar, com edição direta;
- o **criador** constrói um som a partir de partes, sempre reversível, e precisa de muita área de tela. Por isso, **abre em aba própria**, como o Kanban, e não num painel lateral de 320 pixels.

**Compartilhar, nunca duplicar:**

| Peça | Regra |
|---|---|
| Rack de 42 efeitos | Uma lista só, nos dois lugares |
| Variações | Um único tipo de conjunto. O Editor já tem Random, Sequence e Shuffle, com peso e variação de altura e volume: **a seção do Editor deve virar outra vista do mesmo conjunto** |
| Medidores | O desenho do Mixer, com pico, média e volume percebido |
| Exportação e custo | O que já existe, com alvos por plataforma |
| Categorias | Os canais já existentes do Mixer: Music, SFX, UI e Voice. A crítica não confirmou se o canal do objeto chega ao Mixer: **precisa de teste** |

**Pontes:** «Edit in Audio Editor» em qualquer gravação de camada, e «Open in SFX Lab» em qualquer arquivo. **Atenção:** o Editor tem o AI Denoise, que limpa ruído com IA. Para o módulo ficar sem IA, ele não aparece no criador. Decida se a ponte até o Editor é aceitável.

**Pré-requisito, antes da primeira tela.** Pelo código que li, a variação existe só no lado do Editor, e o objeto não a usa. A crítica confirma o outro lado: «a fonte guarda um som só», «um objeto aceita uma só» e «sem sorteio de sons». Falta também o teste do Autoplay, que a crítica pede, e o marcador em um quadro, para sincronizar passos. **O relatório diz que falta o sorteio; na verdade, ele existe, mas não chega ao jogo.** Esse caminho vem primeiro.

---

## Parte 2: a tela

### Decisão: pilha de camadas no centro, cartões à direita, sem grafo de nós

| Opção | Veredito | Por quê |
|---|---|---|
| **Pilha de camadas com régua de tempo** | **Tela principal** | Em som curto, a pergunta é «o que entra quando e em que volume?». A pilha responde de relance, e todo profissional já a conhece de um DAW |
| **Cartões** | **Painel à direita** | Editam a camada escolhida e seguem a sua UI aprovada: dobrar com resumo, reset, alfinete, campo arrastável à Figma |
| **Grafo de nós** | **Não na tela principal** | Esconde o tempo, que é o principal aqui, e assusta o iniciante. É bom para lógica ao vivo (motor de carro), mas isso é minoria. Ligações entre camadas cabem num campo «Follows». **É uma aposta**: se os testes pedirem sons que voltam para si mesmos em circuito, pense num modo avançado depois |

A parte de baixo da tela reaproveita a ideia da tira de quadros do Flip. Rótulos e unidades seguem as suas regras.

### Tela principal

Os valores são exemplos. «SFX Lab» é um nome provisório: o nome é decisão sua.

```text
┌────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ sword_hit ▾  Category [Hit ▾]  Version 7 of 12 ‹ ›  ★ Keep  Discard    [Use in Game] [Export]      │
├────────────────────────────────────────────────────────┬───────────┬───────────────────────────────┤
│ Time (ms) 0   50  100     200      400        800      │ Peak (dB) │ LAYER: BODY             ↺  pin│
│ Mix  ▂█▇▆▅▄▃▃▂▂▁▁▁▁▁▁▁        [Wave|Spectrogram] [Ref] │ -2.1  ▓▓  │▾ Source                       │
│────────────────────────────────────────────────────────│ CLIP  ○   │ Type                          │
│ Layers                          Type         Gain (dB) │           │ [Struck body            ▾]    │
│ ≡ Click  M S  ▌▎                Recording         -3   │ Loudness  │ Material        Size (cm)     │
│ ≡ Body   M S  ▇▆▅▃▂▁            Struck body        0   │ (LUFS)    │ [Steel ▾]       40            │
│ ≡ Tail   M S     ▂▃▃▃▂▂▂▁▁      Noise             -9   │ -17.4 ▓▓  │ Hardness (%)    Decay (ms)    │
│ ≡ Sub    M S  ▆▅▃▁              Tone              -6   │           │ 70              320           │
│ + Add layer  ·  Drop a recording here                  │ Spectrum  │▾ Shape                        │
│────────────────────────────────────────────────────────│ ▁▄▇▆▄▂▁   │ Attack (ms)     Release (ms)  │
│ Size 60     Weight 45     Distance 10     Grit 25      │ ┄ Ref on  │ 1               90            │
│────────────────────────────────────────────────────────│           │▸ Tone            Low cut on   │
│ Variations  Spread (%) 20       [Roll R] [Make set V]  │           │▸ Effects         Drive · EQ   │
│ ▶1  ▶2  ▶3  ▶4  ▶5  ▶6  ▶7  ▶8 Balance ● ● ● ● ○ ● ● ● │           │▸ Follows         Off          │
│ Repeat Test  Rate (per s) 8  [H]      Memory (KB) 640  │           │▸ Variation       Pitch · Gain │
│ ⚠ Variation 5: the tail is cut off.  [Fix]             │           │ Whole sound: Mix · Space · ...│
├────────────────────────────────────────────────────────┴───────────┴───────────────────────────────┤
│ Game Clip  run_and_hit  Length (s) 8   ▶ Loop G   ─◆───◆◆────◆──◆─   Fired: hit 5    Others on     │
├────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Listen as [Flat ▾]    Mono ☐    Listening room [Off ▾]    Play on change ☐    Safety limiter: on   │
└────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

**O que fica onde:**

- **Linha de cima:** o nome, a categoria, as versões e as duas saídas.
- **Centro:** o tempo manda. O som inteiro em cima, as camadas embaixo, cada uma com a própria onda.
- **Faixa de baixo:** variações e teste de repetição, a poucos centímetros das camadas, porque é para onde se vai a toda hora.
- **Direita:** só a camada escolhida. Sem camada escolhida, os cartões do som inteiro: Mix, Space, Playback, Delivery.

### Explorar

A tecla `Tab` abre o modo, como nos objetos do PH2D. O som atual fica no centro e há oito filhos em volta. Arrastar para longe muda mais; as teclas de 1 a 8 tocam cada um.

```text
┌────────────────────────────────────────────────────────────────────────────────────┐
│ Explore: sword_hit                Drag farther for a bigger change                 │
│                                                                                    │
│   ┌───────────┐   ┌───────────┐   ┌───────────┐                                    │
│   │ 1  ▇▅▃▂▁  │   │ 2  █▆▃▁   │   │ 3  ▆▇▅▃▂▁ │                                    │
│   └───────────┘   └───────────┘   └───────────┘                                    │
│   ┌───────────┐   ┏━━━━━━━━━━━┓   ┌───────────┐                                    │
│   │ 4  ▇▆▄▂   │   ┃ Current   ┃   │ 5  █▄▂▁   │                                    │
│   └───────────┘   ┗━━━━━━━━━━━┛   └───────────┘                                    │
│   ┌───────────┐   ┌───────────┐   ┌───────────┐                                    │
│   │ 6  ▅▇▆▃▁  │   │ 7  ▇▅▄▃▂▁ │   │ 8  █▇▃▁   │                                    │
│   └───────────┘   └───────────┘   └───────────┘                                    │
│                                                                                    │
│ Change (%) 30     Locked: Click, Sub     Heard 41     K Keep   X Next   Tab Back   │
└────────────────────────────────────────────────────────────────────────────────────┘
```

Todos os filhos tocam **no mesmo volume**, para você julgar timbre, não altura. Sem isso, o mais alto sempre parece melhor e cansa mais.

### Vazio e dentro do jogo

Abrir nunca mostra uma tela vazia: cada botão **já faz som ao ser clicado**. A crítica pede esse convite em «O que mais falta».

```text
┌────────────────────────────────────────────────────────────────────────────────────┐
│ New sound                                       Used by: Hero · Jump               │
│                                                                                    │
│ Pick one and hear it now:                                                          │
│ [Hit]   [UI]   [Whoosh]   [Pickup]   [Jump]   [Laser]   [Step]   [Explosion]       │
│                                                                                    │
│ Or start from your own sound:                                                      │
│ ┌───────────────────────────────────────┐                                          │
│ │ Drop a recording, or click to choose  │    [Record from mic]                     │
│ └───────────────────────────────────────┘                                          │
└────────────────────────────────────────────────────────────────────────────────────┘
```

No objeto, o cartão Sound segue a versão B da crítica («qual som?», «quando toca?», «ocupa um lugar?»), com um acréscimo: a onda, a contagem de variações e o caminho para editar.

```text
┌────────────────────────────────────────────┐
│ Sound                          ↺  pin      │
│ ▂█▆▄▂▁  ▶  sword_hit                       │
│ 8 variations · 0.6 s      Edit  Replace    │
│ Volume (dB)         Pick                   │
│ 0                   [Random, no repeat ▾]  │
│▾ When                                      │
│ Play On             Said by                │
│ [hit ▾]             Hero                   │
│▸ In the world       Up to 10 m             │
│▸ More               SFX · 4 voices         │
└────────────────────────────────────────────┘
```

### O fluxo real, em cliques

Contagem no papel, como na crítica: não é teste de uso.

**A. Quem faz o jogo, do zero até ouvir no jogo: 3 cliques e o Play.**

| # | Gesto | O que acontece |
|---|---|---|
| 1 | Clique em **Create sound** no cartão do objeto | A aba abre já ligada ao objeto |
| 2 | Clique em **Hit** | Toca na hora, com 8 variações prontas |
| — | `R` algumas vezes, `K` para ficar | Tempo de ouvido, não cliques |
| 3 | Clique em **Use in Game** | Volta ao objeto, com o som ligado |
| 4 | **Play** | Ouve no jogo |

Para comparar: a crítica conta 6 gestos para ouvir uma música e 15 para a moeda que toca som, soma 1 e some.

**B. Profissional, com gravação própria: cerca de 6 gestos de estrutura mais o trabalho de ouvido.** Soltar o arquivo → **Split at hits** → arrastar as partes para camadas → `Tab` ou `R` → `V` para o conjunto → `G` para ouvir no Game Clip.

**C. Entrega para outro motor: 2 gestos.** `Ctrl+E`, `Enter`. O modelo de nome e o destino ficam guardados no projeto.

| Tecla | Faz | Tecla | Faz |
|---|---|---|---|
| `Espaço` | Toca | `K` / `X` | Fica / descarta e vai ao próximo |
| `R` | Sorteia o que não está fixado | `L` | Fixa o que está sob o mouse |
| `M` | Mutação pequena | `1`–`8` | Toca uma variação |
| `A` | Alterna com a referência | `H` | Teste de repetição |
| `Tab` | Explorar | `G` | Toca o Game Clip |
| `V` | Gera o conjunto | `Ctrl+E` | Exporta |

As teclas ainda precisam ser conferidas contra os atalhos globais do app. Os atalhos só valem com a aba em foco.

### Iniciante e profissional, sem dois produtos

**Sem interruptor de modo.** É uma tela só, com duas regras:

1. **Quatro controles grandes já à vista.** Cada categoria abre com controles como **Size**, **Weight**, **Distance** e **Grit**. Mexer em um **destaca as camadas e os campos afetados**, para ensinar sem aula. Isso também dá transparência: nada de caixa-preta. O profissional cria os dele.
2. **A profundidade mora em cartões fechados com resumo.** O iniciante nem abre. O profissional abre, e o programa lembra.

| O iniciante vê | O profissional abre |
|---|---|
| Categoria, onda, 4 controles, `R`, Keep, Use in Game | Camadas, atraso, tipo de fonte |
| Variações prontas, já equilibradas | Faixas de variação e o que cada camada sorteia |
| «Tudo certo» ou uma frase de aviso | Medidores completos e tabela por categoria |
| Use in Game | Modelos de nome, camadas separadas, alvos |

**Nome de tela em inglês simples, mas sem esconder a unidade do ofício.** «Decay (ms)» está certo: é o tempo para sumir. Campo que o iniciante não entende leva dica ao passar o mouse.

### Medidores

| O quê | Onde | Status |
|---|---|---|
| **Forma de onda** do som inteiro e de cada camada | Centro, sempre | **Indispensável.** A edição mostra tempo |
| **Pico**, com aviso de corte (o som passou do máximo e distorce) que fica aceso | Coluna de medidores | **Indispensável.** Já existe no Mixer |
| **Volume percebido** (maior momento, em LUFS) | Coluna de medidores | **Indispensável.** Já existe no Mixer |
| **Espectro** (quanto de cada faixa, do grave ao agudo), com o contorno da referência por cima | Coluna de medidores | **Indispensável.** Não achei um medidor ao vivo nos painéis; só o espectrograma do Editor |
| **Espectrograma** (o espectro ao longo do tempo) | Troca com a onda, no mesmo lugar | Útil, sob pedido |
| **Equilíbrio do conjunto**: um ponto por variação, com a de fora destacada | Faixa de baixo | **Indispensável, e é o diferencial** |
| **Memória** do conjunto e custo por plataforma | Faixa de baixo | **Indispensável.** Já existe na entrega |
| **Compatibilidade em mono** | Barra de escuta, só quando o som é estéreo | Útil |
| Medidores decorativos ou detalhados demais (desenho do estéreo, 30 bandas) | Em lugar nenhum | Cortar |

### Divertido em 60 segundos, sem cansar na décima hora

**Os primeiros 60 segundos**

1. O primeiro clique já faz som.
2. Em 10 segundos, `R` traz outro som. Em 20, algo surpreendente.
3. Em 40, `Use in Game`.
4. Em 60, **o seu personagem pula com o seu som, no seu jogo**. Esse é o momento que nenhum gerador externo oferece. Vale mais que qualquer animação.

**Na décima hora, a diversão vira velocidade**

- Tudo pelo teclado, nada se mexe, e a tela fica sempre igual. Sem mascote, sem planta crescendo, sem som de interface.
- Mesmo volume na exploração, o que também poupa o ouvido.
- Tocar ao mudar um valor pode ser ligado ou desligado. Refazer o som a cada arrasto irrita rápido.
- Nunca perder posição ou seleção.

**Descartar quase de graça**

O [relatório calcula cerca de 2,5 sons salvos a cada 100 cruzamentos](https://research.gold.ac.uk/id/eprint/20164) num sintetizador evolutivo. A amostra é de outro contexto, mas a direção é clara: quase tudo é descartado.

- **`X` descarta e já toca o próximo.** Sem diálogo, sem nome, sem salvar.
- **Nada é gravado como definitivo até o `K`**, mas **tudo o que foi ouvido fica numa lista «Heard»**. Assim, «aquele de cinco minutos atrás» ainda existe. Receita costuma ser pequena, então isso deve sair barato; é preciso confirmar.
- **O desfazer cobre tudo**, inclusive o sorteio.
- **Cada sorteio guarda como foi feito.** Refazer uma variação específica não mexe nas outras.

### A crítica da UI: o que não repetir e o que herdar

Nomes de seção conforme a [crítica](file:///home/enio/Documentos/GameEngineSamples/critica_ph2d.html).

**Não repetir:**

| Erro apontado | Seção | Neste módulo |
|---|---|---|
| **«O editor é forte; o que falta é o caminho até o jogo»** | Lista consolidada, «O quadro inteiro» | **É o maior risco.** Um sintetizador lindo que não chega ao jogo repete o padrão. Por isso o pré-requisito vem primeiro |
| «Nasce mudo», 4 campos que não fazem nada sem Audio Listener | Sound e Audio Source 2D | Nenhum controle sem efeito audível. Se depende de algo, ele diz e oferece a solução |
| «Play Sound toca o som do alvo», sem sorteio, uma fonte por objeto | Sound e Audio Source 2D | «Use in Game» resolve isso, com vários sons por objeto |
| Soltar som na tela é recusado | Sound, «Acrescentar» | Soltar sempre funciona, no objeto, na tela e numa camada |
| Dois nomes para a mesma coisa (Sound e Audio Source 2D) | «Vale rever? Dois nomes» | Um conceito, um nome. O mesmo vale para as variações do Editor |
| Character com 54 campos; Add Component com 58 botões sem explicação | O que mais pesa | O rack de 42 efeitos **não** aparece como parede: busca, grupos, uma frase por efeito |
| «key 4294967291», «shader runtime pending» | O que mais pesa | Sem número interno nem nota de obra. A identidade do sorteio aparece como «Roll 12», não como um número gigante |
| Damping «diz o contrário»; Intensity 6,28 | O que mais pesa | Maior sempre quer dizer mais. Unidades do ofício no rótulo |
| 19 caixas de sinal digitadas; «letra errada é silêncio» | O que mais pesa; Signal Actions | Escolher evento numa lista ou com conta-gotas, e mostrar quem dispara |
| «Cada ajuste custa rodar o jogo e bater em algo» | Camera Shake, «Botão Test» | Test in Game e Game Clip atacam exatamente isso |
| Autoplay que «parece não esperar o Play» | Sound, «Teste que falta» | Testar antes de desenhar, como a crítica pede |

**Herdar:**

| Acerto | Seção | Como usar |
|---|---|---|
| «Os 4 avisos são o melhor deste cartão» | Sound, «Como é hoje» | A mesma voz nos alertas da entrega: uma frase, com a saída |
| «Três perguntas» (versão B) | Sound, recomendação | Forma do cartão no objeto |
| Mini-player com onda no campo | Sound, «Acrescentar» | Todo campo de som mostra onda, tempo e Play |
| Cartão fechado com resumo | Em quase todas as maquetes | Toda a profundidade |
| Convite no vazio | O que mais falta | Tela inicial |
| Botão Test como maior ganho | Camera Shake | Vira o princípio do módulo |
| Som solto cria objeto | Sound, «Acrescentar» | Vale também para o som criado aqui |

Também herdar da sua UI aprovada: irmãos dividindo a linha, uma grelha por cartão, reset, alfinete e o campo numérico arrastável, com Shift e Alt.

---

## O que cortar

| Corte | Motivo |
|---|---|
| **Pintar o som** | Demonstração, não ferramenta |
| **Desenhar sobre a cena como aposta principal** | Fica para cutscenes; a aposta passa para ouvir no jogo |
| **Ajustar receita à voz** | Não medido, e com patente por ler. Fica só a curva extraída |
| **Metáfora de planta ou árvore** | Tira-se a mecânica boa e descarta-se a fantasia |
| **Grafo como tela principal** | Esconde o tempo |
| **Ligação direta com Wwise e FMOD; exportar como plugin** | Arquivos bem feitos resolvem primeiro |
| **Fogo, chuva e água na abertura** | O relatório recomenda abrir com eles. Eu tiraria: som contínuo sintético costuma denunciar a origem, e é o que mais exige do motor. Comece por impactos, interface, whooshes e pickups. Passos entram cedo, mas a partir de gravações |
| **Síntese ao vivo como requisito inicial** | Som assado cobre a maioria dos casos. Ao vivo vem para motor e vento |
| **Tudo de IA do relatório** | Decisão sua. Note que «soe como este arquivo», o nome sugerido e «descreva e receba» saem juntos. Busca e organização ficam por etiquetas manuais |

## O que acrescentar

1. **Limitador de proteção** nos fones, sempre ligado.
2. **Test in Game** e **Game Clip**.
3. **Pilha de camadas com alinhamento fino**, incluindo gravação e **Split at hits**.
4. **Microfone** como fonte de material.
5. **A/B com referência** no mesmo volume, com o contorno do espectro por cima.
6. **Reverb do som separado da sala de escuta**, e o **Listen as** (Phone, Laptop, Far).
7. **Equilíbrio do conjunto**, mais a tabela de volume por categoria escrita por você.
8. **Versões, com o áudio aprovado guardado**, a lista **Heard** e o aviso quando o motor mudar um som.
9. **Modelo de nome** e **exportação em lote** com camadas separadas.
10. **Três forças assadas escolhidas pelo contato**, como primeiro passo de «amarrar ao objeto».
11. **Material de partida gravado**, próprio do PH2D.

**Sobre «mesmo byte em qualquer computador».** Vale a pena, e combina com a sua decisão para o Painter. Mas o profissional precisa mais de outra garantia: **o som aprovado nunca muda**. O áudio guardado dá essa garantia já, enquanto a igualdade entre máquinas ainda não foi provada. Faça as duas, nessa ordem de importância.

## O que não consegui avaliar

- **A qualidade de qualquer som.** Se o corpo vibrante soa bem, se esticar e mudar a altura soam limpos, se a exploração traz bons resultados: só ouvindo.
- **A diversão e o cansaço.** Só com gente usando. O relatório diz o mesmo.
- **O custo de construir** o Game Clip, a troca durante o jogo e a igualdade entre prévia e arquivo.
- **A tela real.** Li código e documentos, não abri o app. Os fatos da crítica sobre som foram lidos, não rodados, e a crítica registra que não há foto do cartão de som.
- **Se existe demanda real de profissionais** que entregam para outros motores.

## As decisões que são suas

| # | Decisão | Recomendo |
|---|---|---|
| 1 | **Consertar antes o caminho até o jogo** (vários sons por objeto, sorteio, teste do Autoplay, marcador no quadro) | Sim, é o primeiro passo |
| 2 | **Para quem é:** quem faz o jogo no PH2D ou também o profissional de outros motores | PH2D primeiro, com exportação excelente desde o início, sem ligação direta com Wwise e FMOD |
| 3 | **Material de partida:** gravar ou encomendar uma biblioteca própria, ou confiar só em síntese e no material do usuário | Gravar. Custa dinheiro e tempo, mas é a maior defesa contra o som genérico |
| 4 | **Modelo de nome escrito pelo usuário** conta como «nome automático»? | Para mim não conta: é uma regra sem IA. Quem decide é você |
| 5 | **AI Denoise:** fora do módulo, e a ponte para o Editor continua existindo? | Fora do módulo; ponte mantida, com o recurso visível só lá |
| 6 | **Cortar a pintura de som** e a metáfora da planta | Sim, reabrindo só se um teste às cegas mostrar valor |
| 7 | **Microfone desde o começo?** | Sim: é simples de explicar e é uma fonte de originalidade |
| 8 | **Som ao vivo no jogo:** agora ou depois do assado | Depois, com três forças assadas primeiro |
| 9 | **O nome do módulo, e quem faz o teste às cegas** | Convide o profissional antes de congelar a tela, com um protótipo que já faça som |
