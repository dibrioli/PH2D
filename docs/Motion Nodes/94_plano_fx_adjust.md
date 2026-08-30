# 94 — Plano: `fx.adjust`, a ponte para os 15 filtros raster que o app já tem

> **Estado: DESENHADO E MEDIDO, NÃO CONSTRUÍDO** (2026-08-30). O nó chegou a ser escrito e foi
> **removido de propósito** — ver §5. Este doc existe para a próxima janela não repetir a
> medição, que foi o caro.

---

## §0 — O que a medição achou (e o que ela corrigiu)

Cinco medições, sobre o código de 2026-08-30. Três delas contradizem docs vivos.

| # | Pergunta | Resposta MEDIDA |
|---|---|---|
| 1 | Os `fx.*` do módulo são todos por-instância? | ⛔ **Não.** `fx.drop_shadow` e `fx.rgb_split` são; o **`fx.glow` é um passe de imagem INTEIRA** (nó passthrough + `from_graph` + `ph2d_render::MotionFx`). O [doc 92 §1](92_o_que_o_mini_cavalry_tem_e_nos_nao.md) diz que os três são per-instância |
| 2 | O motor dos 24 `AdjustmentKind` é alcançável por textura solta? | ⛔ **Não.** Ele mora no `LayerCompositor`, que é moldado por **camadas de um documento**. 18 dos 24 já têm kernel de GPU (12 por-pixel `gpu_code`, 6 espaciais `gpu_spatial_code`) e nenhum é endereçável assim |
| 3 | Há um segundo motor? | ⭐⭐⭐ **Há, e é o barato.** A pilha raster do módulo vetorial: `ph2d_fx_op::FxOp` (**15** tipos) corrida por `ph2d_render::FxStackPass`, cuja porta é `run(gpu, src, dst, w, h, ops, geom)` — **src e dst arbitrários**. Já é GPU-residente, com memo e atlas (`shells/desktop/src/fx_live.rs`) |
| 4 | Existe rota por-instância a que um filtro se pendure? | ⛔ **Não.** Os dois `fx.*` per-instância fazem o efeito **duplicando instâncias** (fantasmas): geometria a fingir raster. Não há substrato de pixels por instância |
| 5 | Existe um passe genérico *«compor esta textura sobre o alvo»*? | ⛔ **Não.** `MotionFx::bloom_over` é do bloom; o `LayerCompositor` é de documento; o `fx_live` devolve a textura ao **Vello** por id estável, que é outra rota. **É esta a metade que falta** |

⇒ **O desenho certo é a forma do `fx.glow`**: um nó passthrough que DECLARA, e um passe que a
shell corre sobre a imagem do módulo. A rota por-instância seria arquitectura nova (o *assador de
tiles* que o doc 92 nomeia), não uma wave.

---

## §1 — O nó (metade A — escrito e removido, ~330 linhas)

`fx.adjust`, crate-folha `ph2d-node-fx-adjust`, deps: `ph2d-nodegraph` · `ph2d-node-registry` ·
**`ph2d-fx-op`**.

- **Passthrough** (`out == in`, `Effect::Pure`), como o `fx.glow`: largá-lo no grafo não muda o
  stream, e não o largar deixa o quadro **byte-idêntico**.
- `from_graph(&Graph) -> Option<FxOp>` — devolve o **mesmo tipo que o módulo vetorial autora**.
  *Uma lei, dois autores.*
- **A ficha inteira é DERIVADA da `FxOp::SPECS`**, em tempo de compilação:
  - `KIND_LABELS` = os 15 nomes, na ordem dos códigos;
  - um `ParamGate` por knob, com a lista de tipos vinda de `radius_label.is_some()`,
    `offset_labels.is_some()`, `color_label`, `color_b_label`, `!modes.is_empty()`,
    `takes_blend`, `noise_labels`, `grow_label`, `adjust_labels`, `takes_ramp`.
  - ⚠️ **Uma lista derivada tem comprimento FIXO** (`const fn` não fatia): os tipos que não
    declaram o knob levam o sentinela `-1`, que nunca casa com um `kind` de `0..KINDS`. *Inerte
    por aritmética, não por convenção.*
- **18 linhas no pior caso** (as duas cores ocupam uma cada, a rampa uma), contra
  `MAX_PARAM_ROWS = 24`. Nenhum tipo mostra todas — os gates escondem o que ele não lê.
- ⚠️ **O `blend` fica sem ficha, declarado:** a lista de leis de mistura é publicada à UI pela
  **shell** (só ela alcança o `ph2d-painter-effects`), e um `ParamWidget::Enum` quer os rótulos
  estáticos no sítio da declaração. Inventá-los no nó daria uma segunda tabela de nomes a
  divergir da que o painel vetorial mostra. O param existe; a ficha é wave.
- ⏳ **DOIS NÚMEROS POR MEDIR, e nenhum pode ser escolhido** (§0.0 do `CLAUDE.md`): a faixa do
  `radius` e a do `offset` em unidades de MUNDO. ⚠️ **O `FxOp::new` dá `radius: 0,18` para o
  Glow** — uma peça vetorial é ~1 unidade de mundo, e as cenas de Motion trabalham em dezenas.
  *A faixa tem de sair do que o painel vetorial já ship ou de uma varredura, nunca de um palpite;
  a 1.ª redacção deste plano tinha `64` e era exactamente isso.*

## §2 — O passe (metade B — NÃO existe)

Correr o `FxStackPass` sobre a imagem do módulo e **compor o resultado** sobre o alvo do jogo,
no sítio onde o `fx.glow` já corre (`shells/desktop/src/render_loop/present.rs`, «Pass 1c»).

⛔ **O bloqueador é a composição, não a pilha.** O `FxStackPass` já aceita `src`/`dst`
arbitrários; o que não existe é um passe *«desenhe esta textura sobre aquele alvo, com esta
lei»*. As três rotas de hoje respondem outra pergunta:

| rota | o que ela é | por que não serve |
|---|---|---|
| `MotionFx::bloom_over` | composição **aditiva** do bloom | a lei é do halo, e os `FxOp` trazem `blend` próprio |
| `LayerCompositor` | compositor de **documento** | pede camadas de um `.ph2d`, não um alvo de render |
| `fx_live` | devolve a textura ao **Vello** por id estável | o desenho é feito pelo `dispatch` no z da forma; a imagem do módulo não é uma forma |

⇒ **A wave B é um passe novo em `ph2d-render`** (foundational): um triângulo de tela, a textura
da pilha, e a lei de mistura do último degrau. Pequeno, mas é código de renderer com gate de
paridade — não é fiação.

## §3 — A ordem, e o smoke

1. **B primeiro.** Um nó sem consumidor é a *capacidade sem porta* que o `CLAUDE.md` §5 nomeia:
   todo gate fica verde e nada desenha.
2. Depois A (o nó já está desenhado acima, e a ficha é derivada — é o pedaço barato).
3. Smoke: uma cena com uma fileira de formas e um `fx.adjust` em `Blur`, com o `radius` a subir.
   ⚠️ A cena tem de imprimir o tipo escolhido — *«o filtro não aparece» tem as mesmas cinco causas
   indistinguíveis a olho que o halo tem* (ver `motion_glow_layer::diag`).

## §4 — O que NÃO fazer

- ⛔ **Não portar os 24 `AdjustmentKind`.** Eles vivem no compositor de documento; a ponte para
  eles é outra obra, com outro dono.
- ⛔ **Não inventar a rota por-instância.** Medida: não existe substrato (§0, linha 4).
- ⛔ **Não escrever a lista dos 15 nomes à mão.** A `FxOp::SPECS` é a tabela; derivá-la é o que
  torna um tipo novo visível aqui sem uma linha.
- ⛔ **Não escolher a faixa do `radius`.** Ver §1.

## §5 — Por que este plano não tem código

A metade A foi escrita e **apagada no mesmo dia**. A razão está no `CLAUDE.md` §5 e na memória
do projecto: *um motor completo e inalcançável passa em todo gate*, e *meio-feito é pior que não
começar*. Com a metade B por existir, o nó desenharia **nada** e a suíte ficaria verde a dizer
que ele funciona.

O que se guarda aqui é o caro: as **cinco medições** do §0 (três das quais contradizem docs
vivos) e a derivação da ficha do §1. Reconstruir a metade A a partir deste doc é uma sessão
curta; redescobrir o §0 não é.
