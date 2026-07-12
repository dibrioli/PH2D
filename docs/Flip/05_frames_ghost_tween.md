# Flip W3 — Frames · Ghost Frames · Tween (o doc definitivo do TEMPO)

> **Estado: FECHADA em 2026-07-12** (pendente o smoke do Enio). Este doc é a fonte de verdade
> do modelo de tempo do Flip: como um desenho ocupa quadros, o que acontece fora do vão, quem
> aparece como fantasma, o que nasce quando se desenha, e como o inbetween é gerado.
>
> Referências: [`02 §1`](02_referencia_algoritmos_blender_5.2.md) (frames/invariantes), `02 §3`
> (tween), `02 §5` (autokey), `02 §8` (onion exato) · [`04 §2/§4`](04_alem_do_blender.md)
> (tween v2, UX dos apps de produção) · [ADR-0114](../architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md).

---

## §1 — O modelo de tempo (o que já existia, e o que a W3 acrescentou)

Uma camada é um **mapa de chaves**: `BTreeMap<Frame, FlipFrame>`, onde a chave é o quadro em que
o desenho ENTRA. A duração **não é guardada** — é derivada (próxima chave − esta). Duas peças
fecham o modelo:

| Peça | O que é | Por que existe |
|---|---|---|
| **Chave real** | `drawing = Some(id)` | o desenho aparece daqui até a próxima chave (o *hold*) |
| **Sentinela de fim** | `drawing = None` | fecha uma duração FIXA: daqui até a próxima chave não aparece nada |
| **Vão** (`span`, W3) | `[1ª chave real, fim)` | o intervalo que os CICLOS repetem. `fim` = a sentinela, se houver; senão `última chave + 1` |
| **Exposição** (`cells`, W3) | quantos quadros a chave ocupa | é o número que o animador lê na tira ("esse fica 2 quadros") |

**A sutileza da última chave.** O hold implícito é *infinito* — a última chave não tem duração
derivável. A tira mostra `1` e, quando o usuário estica esse hold, o modelo **cria a sentinela**
(`FlipObject::set_exposure`): é assim que a última exposição vira um número de verdade. O que
aparece DEPOIS da sentinela não é "nada" por decreto — é decisão do **ciclo** (§2).

**Esticar EMPURRA.** `set_exposure(key, n)` desloca todas as chaves seguintes pelo delta
(mantendo as exposições delas). É a semântica da tira de exposição (TVPaint/Harmony): segurar um
desenho por mais tempo atrasa o resto, não come o vizinho. Ao esticar, o deslocamento roda **do
fim para o começo** (senão a primeira mudança cairia em cima da vizinha parada); ao encolher, do
começo para o fim.

---

## §2 — Ciclos (pre/post behavior por camada) — `cycle.rs`

Um ciclo **não duplica quadros**: é o *wrap-mode* do amostrador.

```
frame < first  →  o `pre`  decide      frame >= end  →  o `post` decide
   None      nada aparece
   Hold      o quadro da borda segura para sempre
   Loop      repete o vão                (off.rem_euclid(len))
   PingPong  espelha sem repetir bordas  (período 2·len − 2)
```

**Os defaults (`pre = None`, `post = Hold`) reproduzem o comportamento pré-W3 byte a byte** —
antes da 1ª chave não há nada; depois da última, o desenho segura. Ligar um ciclo é opt-in, e o
chip da tira liga os DOIS lados em Loop/Ping-Pong (senão o scrub para trás mostraria vazio no
meio de um ciclo).

**Gotcha que custaria um bug silencioso:** o atalho "se o ciclo é o default, use o caminho cru"
está ERRADO. Com uma sentinela, o caminho cru devolve vazio depois dela — e é justamente o `post`
que decide se aquilo é o fim do desenho ou um hold. Por isso `drawing_at_cycled` **sempre** passa
pelo `map_frame`. (Sem isso, fixar a exposição da última chave APAGARIA a arte.)

- **Autoria usa o caminho CRU** (`FlipLayer::drawing_at`), amostragem usa o ciclado
  (`drawing_at_cycled`). Editar o quadro 30 de um `Loop` não pode escrever no desenho do quadro
  6: ele cria uma chave em 30 (que passa a ser o fim do vão). **Amostrar ≠ autorar.**

---

## §3 — Ghost Frames — `onion.rs` (port exato do `get_frame_id`)

A seleção de QUEM aparece como fantasma é uma **função pura**, testada headless nos 3 modos:

```rust
ghosts(layer, current_frame, &OnionSettings, selected_keys) -> Vec<Ghost>
```

O algoritmo (`02 §8`), com as três armadilhas do original portadas conscientemente:

1. **Δ conta CHAVES, não quadros** (modo `Relative`, o default): num hold de 12 quadros, o
   fantasma Δ=−1 é o DESENHO anterior — não o quadro anterior (que é o mesmo desenho). É isso que
   faz o onion do animador funcionar. (`Absolute` conta quadros; com alcance 1/1 num hold de 12,
   **nenhum** fantasma aparece — e está certo.)
2. **Antes da 1ª chave, `Δ += 1`** — senão a primeira chave (que ainda está no futuro) sairia com
   Δ=0 e seria confundida com a corrente (sumindo da lista).
3. **`Selected` IGNORA o alcance** before/after (é o comportamento do GP; o Δ ali só decide
   cor/fade).

**Aparência:** o ghost **não** é a arte com opacidade baixa — é a **silhueta 100% recolorida**
(verde `0.145,0.420,0.137` = passado; azul `0.125,0.082,0.529` = futuro), com
`alpha = (fade ? 1/|Δ| : 1) · opacity`, **piso 0.1** (um fantasma nunca some de todo). No render
isso é um `ghost_tint: vec4` no uniform da câmera: `a > 0` ⇒ passe de fantasma (a cobertura é a
mesma; só a cor e o alpha mudam). O **fill entra na silhueta junto** (senão o ghost sairia com o
miolo colorido e só o contorno tingido).

**Gates (todos do GP):** somem no **play** (fantasma durante a reprodução é ruído puro), respeitam
`onion.enabled` por objeto e `use_onion` por camada, e não existem fora da tool Flip (é chrome de
autoria, não da cena). Custo: 1 upload + 1 draw por fantasma, com a tesselação vinda do **cache
por desenho** — um ghost de um desenho já visitado não re-empacota nada.

---

## §4 — Autokey por-tool — `autokey.rs` + `flip_autokey.rs` (a regra cara)

**A ferramenta decide o que nasce.** Um único ponto de resolução para TODA autoria
(`flip_autokey::target_drawing`), porque errar isso em UM caminho já estraga o documento:

| Gesto | No rabo de um hold, nasce… | Por quê |
|---|---|---|
| **Caneta** | chave **EM BRANCO** | o artista está fazendo o PRÓXIMO desenho; a pose anterior fica onde estava |
| **Caneta + Additive** | **duplicata** | desenhar POR CIMA do anterior |
| **Borracha / escultura / tint** | **SEMPRE duplicata** | se nascesse em branco, o usuário apagaria um quadro novo e vazio — e o desenho que ele **estava vendo** continuaria intacto num quadro anterior. É o erro que o GP documenta, e é irrecuperável sem undo |
| (em cima de uma chave real) | **nada** | desenhar sobre um keyframe não gera keyframe |
| (autokey desligado) | **nada** | o gesto edita o desenho que está NA TELA — comportamento de app de desenho |

A duplicata é **profunda** (`DupMode::Deep`): instanciar faria a borracha comer o quadro de
origem junto (é o mesmo desenho). E a borracha **nunca inventa quadro**: sem nada na tela, ela
recusa o gesto.

O undo agrupa "criar a chave + o traço" num passo só de graça — o registro do undo global é por
diff no fim do frame, e o gesto inteiro cabe dentro dele.

---

## §5 — Tween — `tween.rs` (GP literal + 3 correções)

Quatro peças, cada uma com uma razão:

1. **Pareamento POR ÍNDICE** — o i-ésimo traço de A com o i-ésimo de B. Simples e ESTÁVEL (não
   pisca entre quadros). Traço de A sem par vira **cópia estática**; traço extra de B não aparece
   (ou entra com **fade-in**, se pedido — a 1ª das correções).
2. **Contagem = MAX(A,B) com PADDING** (`sample_padded`): os pontos da curva menor são preservados
   EXATAMENTE e os extras se distribuem ∝ comprimento de arco (maior resto, determinístico). É o
   que garante que em `t=0`/`t=1` os extremos saem **idênticos ponto a ponto** — uma reamostragem
   uniforme NÃO tem essa propriedade, e o desenho do artista "escorregaria" ao entrar no tween.
3. **Auto-flip** — se B foi desenhado no sentido contrário, o lerp faria um nó. Teste geométrico:
   as cordas ponta-a-ponta se CRUZAM ⇒ inverte; cordas quase paralelas (< 15°, comparadas por
   **seno** — HR-5, sem `acos`) ⇒ decide a DISTÂNCIA (empareia as pontas mais próximas); sem
   cruzamento ⇒ inverte se as direções apontam para lados opostos.
4. **Lerp NÃO-clampado**, fator em `[-1, +2]`: overshoot é ferramenta (antecipação/rebote).

Os inbetweens nascem **`KeyKind::Breakdown`**, e re-tweenar **exclui os breakdowns do intervalo
antes de recomeçar** — regenerar é idempotente (não se tweena tween). O fator usa o denominador
`to − from` (posição ABSOLUTA no intervalo), o que faz easing e scrub baterem com o que se vê.

**Correções sobre o original (`02 §3`):** fator por CAMADA (não o da camada ativa para todas),
fills por FATIA (o original tem um `fill(0.0)` de array inteiro — wipe latente), e fade-in
opcional dos órfãos em vez de "pipocar".

**Aberto (v2, spec pronta no `04 §2`):** matching espacial + espiral logarítmica (rotação sai
natural) + UI de correção de pares. Mesma estrutura de dados — sem refactor.

---

## §6 — A tira (`ph2d-panel-flip-frames`) — o que é, e o que deliberadamente NÃO é

Faixa inferior própria (slot `layout.flip_strip`, altura 132 px), visível só com a tool Flip
ativa; sobe uma altura de dock quando o **timeline global** está aberto (as duas são faixas
inferiores e não podem se sobrepor).

- **Células** = as chaves da **camada ativa**, com a LARGURA proporcional à exposição (o tempo é
  visível como espaço) e o número da exposição dentro. A célula é um **botão canônico**
  (`paint_button`) — clicável, com estados e a11y de graça (HR-12), não um retângulo improvisado.
  A escala é derivada do vão: **a tira sempre cabe** (sem scroll, sem estado de pan escondido).
- **Transporte**: play/pause + `◀`/`▶` que pulam por **DESENHO** (pulando holds). Atalhos: `↑`/`↓`
  (o *flip* do animador — o inner loop da profissão), `,`/`.` = ±1 quadro **no FPS do objeto**
  (não no tick de 60 Hz da simulação — senão "avançar um quadro" andaria um quinto de desenho).
- **Ghost / Auto / Add.**: os toggles de fantasma e de autoria.
- **Key ops**: Add (em branco) · Duplicate (cópia profunda) · Delete · **Hold** (a exposição, com
  drag-scrub) · mover ±1 quadro.
- **Tween**: quantos inbetweens + gerar (entre a chave atual e a seguinte).
- **Cycle**: o pre/post behavior da camada (No Cycle / Hold / Loop / Ping-Pong).

**O que NÃO é (decisão, não omissão):** uma tira de UMA camada, não um dope-sheet multi-camada. A
visão multi-camada alinhada é o papel da **timeline global** (W6) — construí-la aqui seria fazer a
mesma coisa duas vezes. A integração com a timeline está **adiada até ela ficar pronta** (Enio,
2026-07-12). O playhead já é o **global** (`ph2d_core::Playhead`), então quando a hora chegar não
há relógio para reconciliar — já é o mesmo.

**Follow-ups conscientes:** drag de célula (mover chave arrastando) e drag da borda (esticar o
hold) — hoje isso é feito pelos botões `◀`/`▶` e pela caixa **Hold**, que dão o mesmo resultado
sem exigir a infra de dispatch 2D do painel. Multi-seleção de chaves (que destravaria o modo
`Selected` dos fantasmas, já pronto e testado no modelo). Marcadores fixos (light table).

---

## §7 — O que está costurado onde (mapa para o próximo agente)

| Peça | Arquivo |
|---|---|
| Vão, exposição, ciclos, navegação por desenho, células | `crates/ph2d-flip/src/{layer,cycle,expose}.rs` |
| Ghost Frames (função pura) | `crates/ph2d-flip/src/onion.rs` |
| Autokey (política + `ensure_key`) | `crates/ph2d-flip/src/autokey.rs` |
| Tween (motor + op de documento) | `crates/ph2d-flip/src/tween.rs` |
| Tint de fantasma no shader | `crates/ph2d-flip-render/src/shaders/flip{,_fill}.wgsl` + `CameraRaw::with_ghost_tint` |
| Passe de fantasmas | `shells/desktop/src/render_loop/flip_pass_ghosts.rs` |
| A tira (painel) | `crates/ph2d-panel-flip-frames/` |
| Estado de autoria + drain dos eventos da tira | `shells/desktop/src/flip_strip.rs` |
| Autokey por-tool (o ponto ÚNICO) | `shells/desktop/src/flip_autokey.rs` |

**Gates executáveis:** `crates/ph2d-panel-flip-frames/tests/seam.rs` (todo controle da barra
chega ao barramento — um botão novo sem braço no `event.rs` = VERMELHO) · testes de unidade do
modelo (ciclos, ghosts nos 3 modos, autokey nas 4 combinações, tween com extremos exatos) ·
`a_ghost_is_the_same_silhouette_recoloured_and_faded` na bateria GPU.

**Schema:** `FLIP_SCHEMA_VERSION` 1 → **2** (a camada ganhou `cycle` + `use_onion`; o
`OnionSettings` ganhou `kind_filter`). `PROJECT_SCHEMA` do shell acompanha.
