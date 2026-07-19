# 20 — Accumulate na mesma pincelada: avaliação (2026-07-18)

> **Pergunta do Enio:** *"Avalie a possibilidade de accumulate na mesma pincelada (sem mouse up) em todo o
> sistema Impasto."*
>
> **Resposta curta: é possível, e existe uma formulação que NÃO quebra os dois invariantes que o desenho
> atual protege. Mas não é um flag sobre o motor de hoje — é uma segunda LEI de depósito, e há uma
> bifurcação de design que é do Enio.**

## 1. O fato, medido

Pincel de raio 16, Depth 1.0, mesmo caminho:

| | pico de relevo |
|---|---|
| 1 passada | 1,6000 |
| **ida e volta SEM soltar** | **1,6000 — 1,00×** |
| 2 pinceladas separadas | 3,2000 — **2,00×** |

Dentro do traço o acúmulo é **exatamente zero**; entre traços é exatamente o dobro. Não é aproximação nem
atenuação: a segunda passada não acrescenta um único bit.

## 2. Por que não acumula — são DOIS mecanismos, não um

Vale saber, porque um flag ingênuo mexeria só em metade:

* **Depósito** (`height.rs`): o relevo é o **envelope `max`** sobre o caminho varrido. `max` é idempotente,
  então passar de novo não muda nada.
* **Sculpt** (`sculpt_blur.rs`): o `amount` **acumula** (`amount[i] += add`) e o render **satura**
  (`k = a.clamp(0.0, 1.0)`). Passar de novo aumenta `amount`, e o render ignora acima de 1.

## 3. Os dois invariantes que qualquer mudança tem de manter

Não são preferências; são cicatrizes, e cada uma foi paga com um smoke reprovado.

**I1 — o relevo é fato do CAMINHO, nunca de quão fino o motor amostrou o caminho.** Esta linha encontrou a
mesma doença **três vezes** (a cápsula do relevo, a mordida do bow wave, o campo do Smear). O sintoma é
sempre o mesmo: o resultado passa a depender do Spacing e da taxa de polling do mouse.

**I2 — o re-stamp tem de ser idempotente.** Os shape editors (Line/Curve/Ellipse/Polygon/Free Hand)
**re-carimbam o traço inteiro a cada frame**. Um motor que acumulasse por-frame faria o relevo crescer
enquanto o artista apenas *olha* para a curva aberta — e sem nenhuma forma de voltar.

## 4. Por que o flag ingênuo não serve

Tirar o `clamp` (ou trocar o `max` por `+=`) dá acúmulo e quebra **os dois**: mais dabs = mais altura, logo
o Spacing e a taxa de polling entram na obra (viola I1); e cada frame de um shape aberto empilha de novo
(viola I2). É a versão que o Blender tem — o *Accumulate* dele é per-dab e de fato depende do espaçamento.

## 5. A formulação que funciona: uma INTEGRAL DE LINHA, não uma soma de dabs

```text
h(p)  =  ∫  perfil( dist(p, caminho(s)) )  ds          (normalizada por comprimento de arco)
```

aproximada como `Σ perfil · Δs`, onde `Δs` é o passo REAL entre dabs — não `Σ perfil`.

Por que isto resolve os dois:

* **I1 ✅** — dobrar a densidade de dabs dobra a contagem e divide `Δs` por dois: a soma converge para a
  mesma integral. Espaçamento sai da conta, como sai do `max`.
* **I2 ✅** — e este é o ponto que faz a ideia toda funcionar: **uma integral de linha sobre o mesmo
  caminho dá o mesmo número.** Re-carimbar uma curva aberta recalcula a mesma integral. É *exatamente* a
  propriedade que o `max` tinha, por outro caminho matemático.
* **Auto-sobreposição ✅** — cruzar o próprio traço soma duas vezes, porque o caminho passa duas vezes por
  ali. **Isso é a feature.**

O maquinário já existe: o depósito já varre o segmento entre dabs (a cápsula), e a integral cavalga a
mesma varredura.

## 6. ⚠️ A bifurcação — o que a demora faz? (decisão do Enio)

Uma integral de arco puro tem uma consequência que precisa ser dita em voz alta: **parar o pincel deposita
NADA** (travessia zero ⇒ `ds = 0`). Tinta de verdade não se comporta assim — pressionar e segurar deposita.

Duas leis possíveis, e elas dão ferramentas diferentes:

**(a) Por comprimento de arco.** `∫ ds`. Demora não faz nada; velocidade não faz nada. O relevo é função
*pura* do rastro geométrico. Mantém I1 e I2 na forma mais forte possível. É o mais previsível e o mais
fácil de gatear.

**(b) Por tempo.** `∫ dt`. Demora constrói; passar devagar deposita mais que passar rápido — que é o que a
tinta faz e o que muitos artistas esperam. ⚠️ **Risco real:** a *velocidade da mão* é entrada legítima do
artista, mas a **taxa de quadros não é**. O `on_tick` do airbrush é dirigido por frame, então uma lei por
tempo tem de integrar **relógio de parede**, nunca contagem de ticks — senão a taxa de quadros vaza para
dentro da obra, que é I1 vestindo outra roupa.

**Recomendação:** (a) como v1, com (b) como adição explícita e gateada depois. (a) é uma lei; (b) é uma lei
mais um problema de relógio, e misturar os dois numa wave é como se perde a capacidade de saber qual
metade errou.

## 7. Custo e o que mais encosta nisto

* **Tem de ser um MODO, não uma troca.** O `Depth` muda de significado — de *"altura do pico de um dab"*
  para *"altura por unidade de percurso"* — então toda arte existente renderizaria diferente. O nome do
  Blender para o toggle é literalmente **Accumulate**.
* **O Sculpt tem uma decisão por-verbo.** A saturação do `Layer` (*"a demão nunca passa de um Depth por
  mais que você demore"*) é **feature documentada e pedida**. Accumulate seria o opt-out dela, não a morte
  dela.
* **O bow wave / Push interage.** A mordida é `Δm/(1−paint)`, auto-limitante e já função do caminho — mas
  um depósito que acumula dá mais tinta para o arado empurrar. Precisa de medição própria, não de fé.
* **Perf:** a integral não é mais cara que o envelope (mesma varredura, `+=` em vez de `max`). O que muda é
  que o resultado deixa de ser limitado por `Depth` ⇒ o **teto de vidro** (`soft_ceiling`) passa a ser
  alcançado de verdade, e ele é uma compressão C¹, então isso está pronto.

## 8. Veredito

**Possível, e vale.** Não é um flag: é uma segunda lei de depósito (~1 wave), com um toggle, uma
re-calibração do Depth, gates novos de espaçamento e de idempotência-sob-re-stamp, e uma decisão do Enio
sobre a demora (§6).

O que **não** recomendo é a versão barata (tirar o `clamp` / trocar `max` por `+=`): ela entrega o efeito
pedido e traz de volta, de graça, as três doenças que esta linha passou o mês curando.

---

## 9. ⚠️ CORREÇÃO ao §5 — o envelope não guarda ALTURA, guarda um VENCEDOR

Escrito depois de ler o kernel para implementar. O §5 dizia *"o maquinário já existe: a integral cavalga a
mesma varredura"* — verdade, e insuficiente. O que o §5 não sabia:

`accumulate_dab_height` **não envelopa altura**. Ele envelopa **carga de tinta** (`fields.paint[i]`) e
guarda, por texel, os **ingredientes do dab vencedor** (grain, raio, cobertura). A altura é **derivada
depois**, de `derive_height`. O comentário no código diz exatamente por que:

> *"Enveloping the paint rather than the height is what makes every knob live: the winner is then chosen by
> a quantity that no setting can change, so re-deriving the relief at a new Body / Source / Depth cannot
> silently re-shuffle which dab shaped which pixel."*

**É isso que mantém Depth, Body e Source vivos DEPOIS do traço.** Uma integral que somasse *altura* assaria
o Depth no buffer e mataria essa propriedade — que é uma das mais caras do módulo.

### 9.1 A correção da fórmula: **acumule a CARGA, derive a altura depois**

A integral não deve ser sobre altura, e sim sobre a mesma grandeza que o envelope já usa:

```text
carga(p)  =  ∫ (silhueta × cobertura)(dist(p, caminho(s)))  ds  /  NORM
altura    =  derive_height(carga, ingredientes)          ← inalterado, e portanto os knobs seguem vivos
```

Mesmo buffer, mesma derivação a jusante, knobs vivos. `NORM = 2ρ·∫₀¹falloff` faz **uma passada reta valer
exatamente uma passada de hoje** — o que torna o toggle honesto (OFF e ON coincidem no caso simples) e é o
gate mais forte que esta feature pode ter.

Ainda assim é cirurgia: o guard `if m <= fields.paint[i] { continue; }` é o coração do winner-takes-all, e
em modo Accumulate todo dab contribui. Os ingredientes continuam sendo os do **vencedor por carga** (a
forma vem do dab mais carregado; a quantidade vem da integral) — e essa separação precisa de gate próprio.

### 9.2 A alternativa que evita a cirurgia: **PASSE**, não dab

Uma segunda leitura, possivelmente melhor como v1: acumular por **passe** em vez de por dab. Quando o
pincel volta sobre terreno que ele mesmo já cobriu *neste* traço, **commita o envelope e começa outro** —
um "pen-up automático".

* Reusa o commit que já existe (o pen-up já faz exatamente isso).
* Preserva **tudo**: o envelope dentro do passe, o winner-takes-all, a independência de espaçamento.
* Dá ao artista precisamente o que ele pediu: esfregar vai e volta constrói.
* O custo se desloca para **um** problema: *"o pincel voltou?"* — critério local (o texel foi coberto por
  um dab distante ao longo do arco, não por um vizinho de amostragem), e é o mesmo tipo de discriminação
  que a cápsula já faz.

⚠️ Uma curva **auto-intersectante** num shape editor passaria a acumular no re-stamp — precisa ser
verificado contra I2, e é a razão de isto não ser obviamente mais barato.

## 10. Recomendação revista

⚠️ **Recomendação REVERTIDA depois de desenhar o gatilho (mesma sessão):** eu disse que o **passe** seria o v1 mais barato. É o contrário. O passe precisa de um **plano canvas-inteiro novo** (o arco da última cobertura por texel — o único jeito honesto de distinguir *"o pincel voltou"* de *"o dab vizinho"*, porque com spacing de 5-10% do raio quase todo texel já está coberto pelo dab anterior), de um **commit no meio do traço** e de acerto com o **undo** (o traço é um passo só). A integral precisa de `Δs` (**já disponível** via `prev_center`), de um normalizador e de uma mudança no envelope. **§9.1 (acumular a CARGA) é a lei certa E o v1 mais barato** — entrega o efeito sem
tocar o winner-takes-all, e a integral pode vir depois como refinamento de qualidade.

**Não comecei a cirurgia**: `accumulate_dab_height` é a função mais cicatrizada do módulo (a cápsula, as
costelas do espaçamento, a lei do vencedor-por-carga — três smokes reprovados moram nela), a mudança altera
a aparência de **toda arte já pintada**, e o §9 mostrou que o desenho aprovado no §5 estava incompleto.
Começá-la agora e entregá-la pela metade é exatamente o que esta linha recusou três vezes hoje, e pelo
mesmo motivo.

**O que falta para eu construir:** a escolha entre §9.1 e §9.2 — e a do §6 (arco puro vs tempo) segue de pé.
