# Suplente #23 — `PathFollow`: **desenhe a patrulha com a caneta**

> **O item, na ordem do dono** ([levantamento §7](00_levantamento_componentes.md), suplentes 21–25):
> *«`PathFollow` (a vitrine da caneta)»*. A linha do catálogo (§2, 121) escreve o que ele é:
> *«paths JÁ são entidades; falta o seguidor»* — **arc-length + tangente**, e a nota do próprio
> levantamento: *«nenhuma engine tem editor de path deste nível acoplado»*.
>
> ⚠️ **Este doc começa pela MEDIÇÃO** (`CLAUDE.md` §5.0). Nesta linha a pergunta já REESCREVEU
> três entregas: o **#3 `SensorZone`** estava fechado por composição, a **W5 do #21** descobriu que
> o pintor já existia, e o **#24 `Health`** deixou de ser um componente.

## §1 — O que a composição JÁ dá (medido em 2026-09-19)

Sonda: [`mede_o_que_a_composicao_ja_da_ao_caminho`](../../crates/ph2d-app-components/tests/it/mede_o_que_a_composicao_ja_da_ao_caminho.rs)
(`--ignored`, imprime). Ela mora na crate de **família** porque é a única que vê os dois lados — o
mundo (`ph2d-ecs`), a lei do relógio (`ph2d-tween`) e a curva desenhada (`ph2d-vec-scene`).

Fixtura: **meia circunferência de raio 2**, aberta, duas cúbicas — escolhida por ter a resposta
**fechada**: num círculo centrado na origem todo ponto da corda está a *exactamente* `r` do arco,
logo o desvio que o bloco (A) mede não é ruído de amostragem, é o raio.

| a pergunta | a resposta MEDIDA | o que ela decide |
|---|---|---|
| **dois tweens de pose (X e Y) exprimem a curva?** | ⛔ **não**: o objecto sai da pista em **`2,000000`** (= o raio) e anda **`4,000`** dos **`6,284`** que o artista desenhou | o concorrente é o que esta linha **acabou de shipar** (#22), e ele é uma RECTA entre dois valores |
| **o `ph2d-ecs` vê a geometria?** | ⛔ **não** (`ph2d-vec-scene` não está nas `[dependencies]` dele) | ⇒ o componente guarda um **NOME**; quem lê a curva é a PONTE |
| **…e a família?** | ⭐ **sim**, `ph2d-vec-scene` **e** `ph2d-vec-entities` | a ponte tem casa, e não é preciso porta nova no `AppHost` |
| **a lei geométrica está paga?** | ⭐⭐⭐ **sim** — `ArcPath::frame_at` dá razão maior/menor de passo **`1,0000`** contra **`1,0692`** do atalho por parâmetro | *esta wave não escreve matemática nenhuma* |
| **a curva sabe para onde vira?** | **sim**: `|t| = 1` a `1,1e-16`, e `t ⟂ raio` a `1,2e-3` (o erro da cúbica que aproxima o círculo) | *«a nave aponta para onde voa»* é um `atan2` |
| **o documento guarda LOCAL?** | ⚠️ **sim** — o topo do arco lê `[0, 2]` em local e `[1, −3]` em mundo com um pai e uma pose: **`5,099`** de distância | a ponte compõe a cadeia de pais pela porta dos sprites |
| **o vai-e-volta existe?** | ⭐ **sim**: `Ciclo::PingPong::dobra` dá `[0, ½, 1, ½, 0]` | a dobra é a da **W8 desta mesma linha** — segundo consumidor, zero lei nova |

⇒ **O buraco é EXACTAMENTE um: nada liga um objecto da cena a uma curva desenhada.** Tudo o resto
— o relógio, a dobra, a curva, o arco, a tangente, a cadeia de pais — já está escrito e medido.

## §2 — O estado da arte (dos dossiês do próprio levantamento)

| engine | o que entrega | o que dele fica |
|---|---|---|
| **Godot** `Path2D` + `PathFollow2D` | um nó FILHO anda pela curva e **carrega os filhos**, rodando-os pela tangente; `progress`, `h_offset`/`v_offset`, `rotates`, `loop` | o vocabulário: percurso · deslocamento lateral · rodar pela tangente |
| **Phaser** `Path` + `PathFollower` | `startFollow({duration, from, to, rotateToPath, rotationOffset, yoyo, repeat, ease, delay})` | ⭐ o **`rotationOffset`** (a arte que não aponta para +X) e o `yoyo` (= o nosso `Ciclo`) |
| **GameMaker** path asset | velocidade **em % por ponto** | ⏳ **fora** desta wave, e nomeado no §6 |
| **Unity** Spline Animate | idem, sobre splines | — |
| **Unreal** | ⛔ **não existe** componente oficial; todo tutorial monta Timeline + *get-location-at-distance* | *«buraco pequeno e famoso da UE; PH2D pode entregar de fábrica»* (dossiê) |

⛔ **O que NÃO se copia do Godot: o modelo de PARENTESCO.** Ali o seguidor é um nó e o objecto é
filho dele. Aqui a árvore da cena é a Hierarquia do artista, e pendurar cada patrulha debaixo de um
nó-fantasma mudaria a árvore que ele autorou — o dossiê da própria linha já escreve a tradução:
*«o componente vira uma referência a um recurso»*.

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| a pergunta | a porta, UMA | porquê |
|---|---|---|
| *que curva?* | `PathFollow::caminho: String` — o **NOME** da forma na Hierarquia | é a lei do `stable_name_id` deste repo (o undo respawna com bits novos), e o `motion.path` chegou **independentemente** à mesma resposta: *«o nome É a referência»* |
| *que relógio?* | o `Timer` do índice `relogio` | a mesma lei do #22 e do #19; renascer no rebobinar, arrancar por `StartTimer` e repetir vêm de graça |
| *quanto do percurso já andei?* | ⭐ `ph2d_tween::andamento` — **extraída** do `valor`, que passa a delegar | ela já existia dentro do `valor`; escrevê-la outra vez seriam duas respostas a *«onde está o relógio na curva?»* |
| *onde fica o arco `s`?* | `ArcPath::frame_at` | a porta única que o Zig Zag e o texto-em-caminho já usam |
| *e em MUNDO?* | `ph2d_vec_entities::transform::{world_transform, xform_of_transform}` | a mesma cadeia que o gizmo de sprite percorre |
| *quem declara a pose ao ledger?* | ⛔ **o passe do TWEEN, e não um segundo passe** | ver §4 |

**O componente** (único por entidade — ⛔ **não** uma lista como os `Tweens`): um objecto tem UMA
posição, e dois seguidores no mesmo corpo seriam duas respostas à mesma pergunta, com o segundo a
ler a pré-visualização do primeiro como documento.

```
PathFollow {
    caminho: String,   // vazio = inerte
    relogio: u8,       // o índice do Timer
    ciclo: Ciclo,      // Reinicia | PingPong   (a dobra da W8)
    easing: Easing,    // as 33 curvas que já existem
    ao_acabar: AoAcabar,
    deslocamento: f32, // fracção do percurso onde ele começa (dá a volta)
    alinha: bool,      // roda para a tangente
    angulo: f32,       // …mais isto (a arte que não aponta para +X)
    lado: f32,         // deslocamento PERPENDICULAR, em metros
}
```

⚠️ **O `deslocamento` DÁ A VOLTA** (`rem_euclid`), e isso não é escolha minha: é a lei que o
`motion.path` desta casa já declara por escrito (*«o `offset` desliza o conjunto e dá a volta»*).
Com `0` o caminho é **byte-idêntico** a não o ter.

## §4 — ⛔⛔ Por que NÃO há um segundo passe (a decisão mais cara desta wave)

A chave do ledger é `(entidade, driver)` e a `PreviewDrive::driven` troca o `authored` quando *«o
que o motor encontrou não é o que ele deixou»*. ⇒ **um segundo motor que fotografe a pose DEPOIS do
tween leria a pré-visualização dele como documento**, que é o defeito que o cabeçalho do
`tween_bridge` já descreve (*«a segunda declaração leria a saída da primeira como outra mão»*).

⇒ a fase do quadro passa a ser: **ler** o que os caminhos pedem (puro, sem escrever) → **aplicar**
dentro do passe que já fotografa a pose **uma vez por entidade** → **declarar** uma vez.

```
fase_tweens:  caminhos = path_follow_bridge::a_escrever(sim, cena, mapa)   // lê a curva
              drive_tweens(sim, drive, &caminhos)                          // UM censo por entidade
```

## §5 — Onde isto encosta

* **Contratos congelados (§6):** ⛔ **nenhum**. `Tool`, `NodeOp`, `VectorOp` intocados.
* **`PROJECT_SCHEMA`: +1** (tipo novo registado, e o postcard é posicional).
* **Registo do `ph2d-ecs`: +1**, e os **dois espelhos** (`ph2d-render` · `ph2d-script`) **+1** cada.
* **`LIVE_SECTIONS`: +1** · `EditorAction` e o vocabulário de edições: **append-only**.
* ⛔ **Zero portas novas no `AppHost`**: a cena e o mapa são campos que a shell já empresta.

## §6 — O que fica FORA, com o motivo

| item | porquê |
|---|---|
| **velocidade em % por ponto** (o *«copiar!»* do dossiê do GameMaker) | é uma curva de re-parametrização por âncora; o `ArcPath` expõe as `anchor_arcs()` e a autoria dela é UI própria |
| `from`/`to` (percorrer só um troço) | o `fx_trim` já apara uma curva, e um troço autorado na FORMA é o mesmo facto noutro sítio |
| pendurar filhos no seguidor (o modelo do Godot) | a Hierarquia é do artista; quem quer levar carga usa `ChildOf`, que já propaga pose |
| escolher a forma por um *picker* de canvas | o nome é a referência, e o campo de texto é o gesto que o `CameraFollow` já tem |
