# 46 — O grafo mostra o que está **vivo** (F3.1: inerte · marcha · massa) — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor **F3** (parte 1 de 2)
**Status:** implementado, testado (mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** nenhum

---

## 1. O problema

O grafo desenha **o que você ligou**. Não desenha **o que ele está fazendo**. Num documento de 60 nós as três
perguntas que o artista faz o tempo todo — *isto está rodando? o dado está passando? quanto dado?* — só têm
resposta apontando a sonda num nó de cada vez.

## 2. As três leituras (e de onde vieram)

| Leitura | O que responde | Prior art (verificado) |
|---|---|---|
| **Inerte** (véu) | *"este galho não roda"* | **Houdini**: a rede *"only cooks the nodes necessary to generate the node with the display flag"* |
| **Marcha** (dashes) | *"o dado está passando AGORA"* | **TouchDesigner**: *"when you see the wires between nodes animating (dashed lines animating), it means the upstream node is cooking"* |
| **Massa** (espessura) | *"quanto dado"* | — (o taper que o plano F3 já previa) |

### 2.1 Inerte = alcançabilidade, não "tem readout"

O cook é **pull**: parte dos sinks (`motion.output`) e avalia só o que eles alcançam — a mesma forma do Houdini.
Então galho que não alcança sink **não é lento: não roda**. A pergunta é de **alcançabilidade** e se responde
andando **PARA TRÁS** a partir dos sinks (`flow::live_set`). Andar para FRENTE a partir das fontes marca tudo que
as fontes alimentam — inclusive todo beco sem saída — que é exatamente a pergunta invertida (mutante provado).

E **o fio é vivo sse o que ele ALIMENTA é vivo** (a fonte então é viva por construção) — um teste só, e o fio que
sai dum nó vivo para um morto se apaga junto, que é a verdade.

**Por que não reusar o `readout.is_none()` do doc 43:** um nó cozido em **lane escopada** (`motion.time_remap`) não
tem memo na lane raiz → não tem número. Ele **é** consumido. Velar por falta de número seria uma mentira que o
artista não tem como contestar. O véu do CARD migrou para a alcançabilidade; o readout continua sendo "o que
produziu".

### 2.2 Marcha = o VALOR mudou (não "foi avaliado")

O TD só coza o que está sujo, então lá "fio animado" = dado novo. **O nosso cook re-puxa tudo todo frame** (o
playhead anda), então o port fiel de "cooking" é **o valor MUDOU** — um digest por nó, comparado frame a frame
(`motion_bridge_readout::digest_of`). "Foi avaliado" botaria fogo no canvas inteiro e não diria nada.

**O mutante que importa:** digest só do **count**. Uma grade de 400 instâncias sendo sacudida por um oscilador tem
400 instâncias todo frame — e é a coisa mais viva da tela. O digest hasheia **os valores** (`to_bits`), não o
tamanho. Guarda: `a_wire_runs_hot_when_its_value_moved_not_when_its_size_did`.

**O trade escrito onde dói:** o digest amostra no máximo `DIGEST_SAMPLES = 48` elementos por coluna (stride).
Mudança confinada aos elementos pulados lê como parada. É um indicador de vivacidade, **não um checksum** — e
hashear 10 000 instâncias × colunas × nós todo frame custaria mais que o cook que ele está reportando.

**Nada marcha no 1º frame** (não há "frame anterior" para diferir) e **nó inerte nunca marcha** (fio que ninguém
consome não tem dado correndo; um galho morto piscando seria a mentira mais alta do canvas).

**O relógio é UM.** `snap.now` vem do `transport.playhead` — o painel **não** ganhou relógio próprio: uma animação
tocada por contador de paint continuaria marchando com o grafo **pausado**, exatamente a mentira que o dash existe
para não contar.

### 2.3 Massa = espessura

`wire_width(count)`: `sqrt` normalizado até `MASS_REF = 4096` instâncias, de fio (1.8) a cabo (5.2). `sqrt` porque
as décadas interessantes são as de baixo (12 → 200 tem de ser legível; 4 000 → 8 000 não precisa). Satura: acima da
referência a leitura é "muita coisa", não um gráfico de barras. O `WIRE_W` fixo **morreu** — a espessura agora é
uma leitura, não uma constante.

## 3. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| painel | **`flow.rs`** (novo): `live_set` · `edge_is_live` · `wire_width` · `dashes` + `DASH_PERIOD/ON/SPEED` |
| `GraphNodeView` | **`count: Option<u32>`** · **`hot: bool`** · **`is_sink: bool`** |
| `GraphViewSnapshot` | **`now: f32`** (o playhead — o único relógio) |
| shell | `MotionState.flow_digest: BTreeMap<u32,u64>` (**não** `HashMap` — HR-5/ADR-0022) · `readout::stamp` agora é `&mut` |
| removido | `paint::WIRE_W` (a espessura virou leitura) |

## 4. A lição

**Um editor que desenha só a topologia esconde metade do bug.** As três coisas que o artista precisa saber — roda /
passa / quanto — já estavam TODAS no cook; faltava o canvas dizer. E a barata dessa leitura é fácil de escrever:
"foi avaliado" (fogo em tudo) ou "o count mudou" (frio no que mais se mexe). As duas passariam num teste que só
pergunta *"desenhou dash?"*. O que separa é o **mutante**.
