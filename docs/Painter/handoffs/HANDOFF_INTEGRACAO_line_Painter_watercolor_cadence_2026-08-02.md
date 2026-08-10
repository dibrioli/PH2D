# HANDOFF DE INTEGRAÇÃO — `line/Painter`, a cadência da aquarela (2026-08-02)

> ⚠️ **SUPERSEDIDO para a INTEGRAÇÃO pelo [handoff MESTRE](HANDOFF_INTEGRACAO_line_Painter_MESTRE_2026-08-02.md)** — este cobre só a
> metade **B (a aquarela)**. Ele segue válido no DETALHE de cada wave; o mestre é o que a integração precisa
> (colisão, ADRs, schema, os dois arquivos de alta colisão e o ponteiro de memória pendurado).

> Para o **agente integrador**. A linha NÃO integrou e NÃO fez push.
> Detalhe técnico e as medições: [`docs/Painter/28_otimizacoes_o_que_funcionou.md` §5.71](../28_otimizacoes_o_que_funcionou.md).

## 1 — Identificação

| | |
|---|---|
| Branch | `line/Painter` |
| Worktree | `Worktrees/line-Painter` |
| HEAD | `fa3ea9ae3` (atualizar após o commit final) |
| Base | `main` a `a9f5977e9` (rebase era no-op ao assumir; **re-rodar `git rebase main` antes de integrar**) |
| Commits desta sessão | **18** — a cadência (4) + a avaliação sob carga (2) + o SMUDGE (5) + a instrumentação e o VÉU (4) + os dois passes da umidade em PARALELO (3) |
| Commits acumulados da linha | **71** (53 herdados do motor de undo + os desta sessão) |

## 2 — O que entra

**A tarefa era *avaliar* o modo Watercolor e tentar otimizá-lo.** A avaliação achou dois defeitos de
custo, os dois medidos pela porta do produto (`on_canvas_pointer` / `paint_tick`), e os dois curados.

### 2.1 A lavagem reconstruía por EVENTO de ponteiro, e o doc dela dizia QUADRO

`apply_watercolor` reconstrói a lavagem inteira sobre a base congelada, e a janela dela é padeada pelo
**raio de influência** — do tamanho da pegada. Ela rodava dentro de **cada `PointerPhase::Move`**, então
encolher o passo do mouse não encolhia a passada: só multiplicava quantas vezes ela acontecia.

Mesmo traço, 30 quadros (0,5 s), r=100, canvas 4096² — variando só quantos eventos caem em cada quadro:

| dispositivo | ev/quadro | antes | **agora** | ganho |
|---|---|---|---|---|
| 120 Hz | 2 | 130,9 ms | **92,2 (1,00×)** | **1,42×** |
| 240 Hz | 4 | 146,6 | **89,9 (0,97×)** | 1,63× |
| 480 Hz | 8 | 179,0 | **90,6 (0,98×)** | 1,98× |
| 960 Hz | 16 | 234,1 (1,79×) | **91,3 (0,99×)** | **2,56×** |

A rota nova é **plana em 8× a taxa do dispositivo**: o custo passa a depender do DESENHO, não do mouse.

⚠️ **Byte-idêntico, e MEDIDO:** o mesmo caminho em 15 e em 120 eventos pinta telas que diferem em
**0 bytes**. Isso também refuta a hipótese pior — a aparência da aquarela **não** dependia do hardware.

⚠️ **Latência ZERO:** o tick roda em `render_loop` ~1198, depois do flush de ponteiro (~698) e **antes**
do upload do preview (~3397). O quadro que recebeu os Moves é o quadro que mostra a tinta.

### 2.2 O pen-down alocava 268 MB para reproduzir uma cor chapada

`composite_below` preenchia o acumulador `[f32;4]` **antes** de perguntar se há algo abaixo da âncora.
Num documento de UMA camada não há — e 335 MB de tráfego produziam a cor de papel.

**pen-down 81,5 → 26,4 ms** (r=20) · 82,1 → 36,7 (r=100) · 112,3 → 62,2 (r=400), a 4096².

⚠️ O ganho é do documento de **uma camada**. Com camadas abaixo o caminho longo continua — corretamente.

### 2.3 — O Smudge forkava o canvas em TODO evento (2026-08-02, tarde)

Report do Enio: *"um pincel de 250px e as configurações na imagem provocam grande queda de FPS.
Desempenho pior em imagens grandes (4096)."* Com os knobs dele (`Rewet 0.400`, `Smudge 0.197`,
`Dilution 0.168`, `Charge 0.755`, `Pull 0.477`), traço de 1500 px, r=250:

| | antes | depois | |
|---|---|---|---|
| carimbo (`on_canvas_pointer`) @4096² | 49,60 ms | **5,06** | **9,8×** |
| carimbo, cresce com a tela | 7,40× | **1,01×** | limitado pela PEGADA |
| **quadro** @4096² | **83,4 ms** | **27,4** | **3,05×** |
| quadro, cresce com a tela | 2,87× | **0,89×** | plano |

`smear_wet_base` muta a base pelo `Arc::make_mut`, e a re-partilha no fim da função restabelece o par
de donos ⇒ o fork acontecia a **cada evento de ponteiro** (67 MB a 4096²), não uma vez como o
comentário dele sugeria. **Cura: soltar a segunda referência ANTES do `make_mut`** — com um dono só ele
MOVE. É a §5.12 do Wet Paint um módulo adiante. **Byte-idêntico por construção** (`make_mut` com um
dono devolve o mesmo buffer); suíte **961 release / 959 debug**.

⚠️ **A ablação por entrada é o que nomeou o dono:** sem Smudge o carimbo custa **1,62 ms nas DUAS
telas**. O knob nasce em `0`, e é exatamente por isso que a tabela de decomposição do doc 31 o usou
como **piso de ruído** — o custo dele nunca tinha sido medido.

### 2.4 — Os dois passes por-quadro da umidade caminham em PARALELO (2026-08-02, noite)

O 2º log do Enio fechou a conta do quadro (`composite 10,62 · pour 9,70 · secagem 12,81 · CHROME wet
37,05 ≈ 70` contra `frame p50 = 67,3`), e depois que o véu caiu (§2.3b/§5.75) os dois maiores itens
restantes eram a **SECAGEM** e o **DESPEJO** — os dois caminhando a união cumulativa em todo quadro,
pinte-se ou não (o papel seca no heartbeat).

⛔ **TRÊS curas single-thread foram construídas antes da que funciona, e as três mediram ~1,00×** —
está no [doc 28 §5.76](../28_otimizacoes_o_que_funcionou.md) para ninguém as reconstruir: a
**janela deslizante** no lugar do snapshot do rect (1,02×), o **piso da erosão** e o **rect que
encolhe**. A alocação e a cópia não eram o custo; o custo é o **caminhar**, 2,2 ns/texel. A janela
deslizante foi **revertida**, porque a dependência entre linhas que ela cria é exatamente o que
impede o paralelo que funciona; o piso e o rect **ficaram** (byte-idênticos, e o rect é lido também
pelo véu do shell).

**O que move:** os dois passes são row-parallel, sob **emenda de 2026-08-02 no ADR-0109** (escrita no
próprio ADR). Os três invariantes dele valem verbatim, e a redução é do tipo que a cerca de contenção
**isenta**: `max`/`min` sobre INTEIROS na secagem, e **nenhuma** no despejo.

| | antes | agora | |
|---|---|---|---|
| secagem, um passe @4096² | 30,44 ms | **3,28** | **9,3×** |
| secagem, 120 quadros secando | 28,50 ms/quadro | **2,93** | **9,7×** |
| despejo @4096² | 12,46 ms | **0,63** | **19,8×** |

⚠️ O 19,8× inclui a tabela de dureza (o oráculo congelado é o produto pré-wave inteiro), e a corrida
dele saiu com `load 6,7`: **as razões sobrevivem, os absolutos não** (a corrida calma dava 28,87 →
3,45 na secagem).

⚠️ **LOC:** o `watercolor_backdrop.rs` bateu **696/700** — quatro linhas de folga é dívida latente,
então o decaimento saiu por ASSUNTO para o irmão novo **`watercolor_dry.rs`** (*o que molha o papel*
× *o que o seca*). 547 + 161. O piso do pool passou a morar com o MAPA, que os dois passes
compartilham (`WET_PAR_MIN`).

## 3 — Foundational tocado

**NENHUM.** Todo o diff mora em `crates/ph2d-tool-painter/`. Zero `Cargo.toml`, zero dep nova, zero
crate nova, nenhum ADR, nenhum id/token/i18n, **nenhum schema** (`PROJECT_SCHEMA` **não** foi tocado).

## 4 — Contratos congelados (CLAUDE.md §6)

**Nenhum encostado.** Conferido por `cargo test -p ph2d-editor-core --release` (inclui
`architecture_tool_contract_surface`): `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` /
`PanelEvent=4` intactos.

## 5 — Superfície nova (para o integrador detectar colisão)

**Módulo novo (privado):** `tool/paint/watercolor_dry.rs` (+ `watercolor_dry_tests.rs`) — split de LOC
por assunto, `mod` declarado em `paint.rs` ao lado do `watercolor_backdrop`. `PaintState` ganhou
`canvas_wet_snapshot: Vec<u8>` (scratch persistente do decaimento) e `watercolor_backdrop` exporta
`pub(super) WET_PAR_MIN`. **Nada disto é público fora do `tool::paint`.**

| símbolo | onde | o que é |
|---|---|---|
| `paint::watercolor_field::WashCadence` | `watercolor_field.rs` (fim) | `{ per_event: bool, composites: u32 }` — sub-estado, no padrão do `WetSessionStyles` que já mora ali |
| `PainterTool.wash` | `tool/mod.rs`, ao lado de `canvas_rgba` | o campo |
| `pub(crate) mod watercolor_field` | `paint.rs` (era `mod`) | visibilidade alargada para o `PainterTool` alcançar a struct |
| `compose::encode` | `compositor/compose.rs` | era privado, virou `pub(super)` para o gate da substituição |
| `watercolor_cadence_tests` | filho de `watercolor_field` | 3 gates novos |
| `WashCadence.window_px` / `.base_forks` | `watercolor_field.rs` | contadores do produto: área da janela e forks do canvas |
| `watercolor_smudge_gate_tests` | filho de `tests` | os 2 gates do fork |

⚠️ **Não há campo `owed`.** *"Chegou um move neste quadro?"* já é `moved_this_frame`, que o `paint_tick`
lê como `parked` — um segundo campo seria um segundo lugar para o mesmo fato, com ciclo de vida próprio
a acertar.

## 6 — Gates novos e as provas de mutação

**`watercolor_smudge_gate_tests`** (2): a **PROPRIEDADE** (o produto conta os forks em
`WashCadence::base_forks`, irmão do `composites` — contagem, não relógio) e a **CONSEQUÊNCIA** (razão
2048÷4096 do carimbo). Mutação (reinstalar o fork): **18 forks em 8 eventos** e razão **6,61×**; os
dois sangram e não são redundantes. ⚠️ O oráculo do **endereço** do buffer foi tentado e **descartado
por medição** — o alocador devolve a alocação recém-liberada, então o ponteiro sai e VOLTA, e o gate
lia *"não moveu"* sobre um produto que copiava 67 MB por evento.

**`watercolor_cadence_tests`** (3): a **CONTA** (um composite por quadro — um CONTADOR, não um relógio:
uma razão sobre passadas de ~1 ms mede o escalonador desta máquina) · o **QUADRO** (a lavagem está viva
no quadro que recebeu os Moves) · a **FIGURA** (byte a byte em qualquer taxa de polling).

**`compositor::tests`** (2): o round-trip de byte do sRGB é a **identidade nos 256 valores** · o
preenchimento chapado **é** o que o acumulador teria codificado (alfa incluso).

**2 mutações, 2 sangram:**

| mutação | efeito |
|---|---|
| o tick não paga a dívida (`stamped \|\| …` → `stamped`) | **8 testes caem**, incluindo o gate do QUADRO |
| `WashCadence { per_event: true }` por default | cai **só** o gate de cadência — os outros dois passam nas duas rotas, **por desenho** |

⚠️ **A rota de ablação (`WashCadence::per_event`) existe por dois serviços**, no precedente do
`Sim::order_invariant` (ADR-0147): um A/B cross-process atribuiria a deriva desta máquina (o mesmo passo
de produto já foi medido a 14,5 e 30,2 ms) à mudança; e o gate de cadência precisa de uma alavanca que o
faça ir VERMELHO. Produto é sempre `false`.

### 6.x — Os gates da wave do paralelo (5 gates, 7 mutações, 7 sangram)

`watercolor_dry_tests.rs`: identidade da **secagem** contra a rotina que SHIPAVA (congelada sob
`cfg(test)`), varrendo `step` 1/5/17/51 **e os dois lados do piso do pool** · identidade do
**despejo** contra o serial congelado ESCRITO NO TESTE · o **rect recua nos 4 lados** sem deixar
umidade fora dele · a **tabela de dureza** contra a expressão escrita no gate · e o **arch-gate do
`par_chunks_mut`** nos dois passes. Mais a sonda `#[ignore]`
`the_cost_of_the_drying_pass_by_both_routes`.

⚠️ **Quatro lições de gate, todas minhas, e todas registradas no doc 28 §5.76:** a 1ª versão usava só
128² (abaixo do piso ⇒ **nunca entrava na rota paralela que a wave instala**) · o `step` é fixture
tanto quanto o tamanho (a mutação *"leia um vizinho já escrito"* **sobreviveu** a `step = 5`, porque a
erosão é inteira e o erro só atravessa a quantização em `step ≥ 12`) · o gate da tabela era uma
**tautologia** (comparava a LUT com a função que a constrói) · e comparar as duas rotas do PRODUTO
provaria só o *walker*, porque as duas compartilham o corpo (ADR-0145).

## 7 — ⚠️ Seis fixtures existentes foram corrigidas

Elas dirigiam Moves e **nunca fechavam um quadro**, então mediam a cadência ANTIGA — e o doc de uma
delas já dizia *"local to the **frame's** new dabs (wet_edges `renderFrame`)"*. Fecham quadros agora
(helper `frame()` em `tests.rs`, uma regra num lugar só) e **ficaram mais fortes**: provam também que o
tick paga a dívida. As seis: `watercolor_wash_is_live_before_pen_up` ·
`watercolor_live_recomposite_is_local_to_the_frame` · `watercolor_moving_preview_restores_the_old_position` ·
`watercolor_incremental_composite_matches_full_recompose` · `..._with_water` ·
`watercolor_granulation_bake_settles_beyond_the_live_preview`.

## 8 — O que só o `ship.sh` pega

- **Dívida de `fmt` PRÉ-EXISTENTE, já paga no commit `56f9f372f`:** cinco arquivos desta linha estavam
  commitados sem passar pelo rustfmt **pinado** (1.95, via `rust-toolchain.toml`) — resíduo dos 53
  commits `--no-verify`. Puro reflow, conferido diff a diff. **Sem isso o integrador herdaria um `✗`.**
- `cargo machete` / `deny` / `audit` / `typos` não foram rodados aqui (nenhuma dep mudou, então o risco
  é baixo, mas o `ship.sh` é a autoridade).

## 9 — Verde local

| gate | resultado |
|---|---|
| `cargo test -p ph2d-tool-painter` **debug** | 958 · 0 falhas |
| `cargo test -p ph2d-tool-painter --release` | **959** · 0 falhas |
| `cargo test -p ph2d-editor-core --release` | verde (inclui contratos + LOC) |
| `cargo test -p ph2d-host-desktop --release` | **77 binários**, 0 falhas |
| `cargo clippy -p ph2d-tool-painter --all-targets` | limpo |
| `cargo fmt -p ph2d-tool-painter --check` | limpo |
| `architecture_workspace_file_loc_cap` | verde |

⚠️ **`paint.rs` estava EXATAMENTE em 700 (o teto)**, então qualquer linha o quebrava. Foi por isso que o
estado virou sub-struct num irmão e os gates viraram filhos de `watercolor_field`: **o arquivo volta a
700, idêntico ao HEAD.**

## 10 — O que SMOKE-TESTAR

```fish
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && env PH2D_WETPAINT_SMOKE=1 cargo run -p ph2d-host-desktop --release
```
⚠️ **O `cd` faz parte do comando.** Este trabalho está na worktree da linha, e os MESMOS caminhos
relativos existem na árvore primária — rodar da raiz compila e abre o app do `main`, sem um único
erro, mostrando exatamente o comportamento ANTIGO. Um smoke que reprova por isso reprova a coisa
errada.

⚠️ O smoke abre em **Digital** de propósito — escolha **Watercolor** no dropdown de Paint Mode.

1. **Canvas 4096, pincel GRANDE (raio 200-400), traço longo.** A pergunta é de mão: o traço tem de sair
   **liso**, e o começo do gesto não pode engasgar. Se o seu mouse/tablet for de alta taxa, é
   exatamente aí que a cura paga mais.
2. **O pen-down.** O primeiro toque de cada traço era ~80-112 ms; deve ter sumido como hitch.
3. **A APARÊNCIA não pode ter mudado** — nem a borda, nem a granulação, nem o escorrido. Isto está
   gateado em byte-identidade, mas *o olho é o oráculo final*: se algo mudou, é bug meu.
4. **Um documento com VÁRIAS camadas** (o early-out do pen-down não se aplica ali): a lavagem tem de
   continuar lendo as camadas de baixo como chão.

## 11 — Aberto, com número

- **O `pour_canvas_wet` ainda caminha o rect CUMULATIVO** uma vez por quadro ⇒ o custo por quadro cresce
  **1,23× / 1,32× / 1,51×** do 1º para o 4º quarto (traços de 24/48/96 quadros). ✅ **A premissa foi
  VERIFICADA em 2026-08-02** (doc 28 §5.72, lendo os escritores em vez de supor): `stroke_coverage` e
  `wet_styles.owner` só são mutados **por-dab**, os dois `zip` de plano inteiro do acumulador são
  **backfills únicos** (guardados por `len() != fw*fh`), e o pour é **idempotente** (max-blend) ⇒ um
  texel fora do rect do QUADRO não pode ter mudado, e o rect do quadro basta **byte a byte**. A cura é a
  lei do S1/S2 (*a janela vem de quem ESCREVE*): o acumulador declara `wet_pour_dirty` por-dab e o pour o
  consome. ⚠️ **Segue não construída de propósito** — perf sem número de PRODUTO não é wave, e a máquina
  estava a `load average 27,7`. Sondas prontas: `measure_whether_the_frame_cost_grows_along_the_stroke`
  (tempo, exige box calmo) e `measure_the_area_the_wash_walks_per_frame` (**contagem, vale sob carga**).
- **E a dependência de TELA é por TRAÇO, não por quadro** (achada na mesma varredura): o
  `freeze_watercolor_ground` do pen-down faz **três varreduras de plano inteiro** — backdrop ~67 MB +
  substrato ~67 MB + soak ~16 MB a 4096², um quarto disso a 2048². ⚠️ O `wet_substrate` é preenchido
  **preguiçosamente** (só sobre a região de saída do composite), então o `NaN` de tela inteira invalida
  pixels que **nunca foram preenchidos** — mesma cura, um plano adiante.
- **O WARP segue sendo 56%** do que a aquarela cobra sobre o Digital e **não tem caminho de CPU**: os 9
  taps de AA foram a CURA da borda serrilhada e cortá-los está fora de discussão. Aproximar o warp
  dentro do texel é a classe que este repo já mediu e **rejeitou duas vezes** ⇒ exige oráculo de
  APARÊNCIA e ordem do Enio.
- **`DragDot`/`Anchored`/`Line` compõem por evento mesmo com a cura** (o `clear_wet_coverage` dobra o
  rect cumulativo no do quadro). No app eles são **coalescidos** pela shell, então não é defeito vivo.
- ⚠️ **A coluna "razão" da varredura de raio é inutilizável a r≥200:** o `moves` do **Digital** cai
  (34,3 → 11,8) em vez de crescer, o que é a assinatura do confound de contagem de dabs. Os números
  ABSOLUTOS da aquarela valem; a razão contra o Digital ali, não. **Não expliquei essa anomalia** — ela
  é do controle, não da aquarela, e pode ser um achado do Digital por conta própria.
