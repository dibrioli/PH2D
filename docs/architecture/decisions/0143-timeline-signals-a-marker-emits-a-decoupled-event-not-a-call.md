# ADR-0143 — Sinais da timeline: um marker EMITE um evento desacoplado, nunca uma chamada

- **Status:** aceito (provisório na `line/anim`; o número renumera na integração se colidir — o maior no main de hoje é 0142)
- **Data:** 2026-07-25
- **Linha:** `line/anim`
- **Contexto:** Enio — *"ainda não temos todas as features de apps pro … quero o essencial para chegar ao padrão-ouro."* A pesquisa dos quatro (Unity, Unreal, Godot, After Effects) achou **uma** convergência total: **os quatro disparam algo num ponto do tempo**. É o maior buraco da nossa timeline e o mais barato — os markers já existem como dado.

## O problema (a força que obriga a decidir agora)

A nossa timeline **autora** movimento no estado da arte (graph editor, weighted tangents, speed graph, time remap, motion path, roving, record, composição de clips, nesting, onion). O que ela **não faz** é o verbo que os quatro apps pro têm: **DISPARAR** — no instante X, ou ao cruzar um marker, acontecer algo (um som, um gatilho, uma ação).

Hoje o `Marker { t, label }` (`ph2d-timeline/src/doc.rs:95`) é **só uma anotação** — nada é emitido quando o playhead o cruza. E a mesma lacuna já foi nomeada em OUTRO módulo: a física (W-ContactEvents) shipou o **canal** de eventos de contato + um flash visível, mas deixou *"o consumidor de GAMEPLAY não construído: cross-line, decisão do Enio"*. **É o mesmo consumidor.** Este ADR decide a forma dele para a timeline, e a decisão do consumidor é a bifurcação que precisa da sua palavra.

## A pesquisa — quem já resolveu, e o que ABANDONOU (o abandono é o achado)

| App | Como dispara | O que ABANDONOU / a armadilha documentada |
|---|---|---|
| **Unity** | **Signal Emitter** (um marker) → **Signal** (asset) → **Signal Receiver** (component) reage; o emitter **não conhece** o receiver | ABANDONOU o **Animation Event** (nome-de-método-string no clip): renomear o método **quebra em silêncio**. Signals existem para *desacoplar* emitter de receiver. |
| **Unreal** | **Event Track** com **Trigger** (avalia no frame do key, 1×) vs **Repeater** (avalia todo frame da seção) | A distinção Trigger×Repeater existe porque *"1× no ponto"* e *"ativo numa faixa"* são necessidades diferentes — conflatá-las é o erro. |
| **Godot** | **Call Method Track** (um key chama um método do nó) | Bug documentado: *"sporadically not called when the transition is spammed"* — a chamada por **igualdade de frame** falha sob scrub/jump rápido. **É a doença do CAMINHO que este repo curou N vezes.** |
| **After Effects** | **Markers** (cue points / chapters) + expressões | Simples, mas conflata *anotação* com *cue* — a lição é separar os dois. |

**Dois achados baratos, ambos do abandono alheio:** (1) **desacoplar** o emissor do consumidor (Unity matou o método-string-no-clip); (2) **disparar pelo CAMINHO, não por igualdade de frame** (o bug do Godot). Nossa decisão herda os dois.

## A decisão

> **Um marker da timeline pode carregar um SINAL nomeado que, durante o play para frente, emite exatamente uma vez quando o AVANÇO do playhead CRUZA o instante do marker — como um evento DESACOPLADO que qualquer sistema drena (ADR-0075), nunca uma chamada direta — com o cruzamento computado sobre o CAMINHO do playhead (catch-up e loop corretos por construção; scrub e reverse não emitem nada).**

### 1. Um sinal é um EVENTO DESACOPLADO (ADR-0075), nunca uma chamada

A emissão do apply da timeline empurra um `TimelineSignal { name, t }` numa fila que o shell drena; consumidores casam por **nome**. A timeline **nunca chama** um consumidor. É o norte do ADR-0075 (*desacoplar por eventos/resources; systems não se chamam*) e a resposta que a Unity chegou depois de abandonar o método-string. Espelha o `PhysicsBridge::contact_events()` — o precedente exato, do mesmo repo.

### 2. A emissão é função do CAMINHO, não de igualdade de frame

Num dispatch em que o playhead andou de `t_prev` para `t_now`, **todo** marker com sinal cujo `t ∈ (t_prev, t_now]` emite **uma vez**. Isso trata catch-up multi-tick (o mesmo intervalo do bake/physics) e loop (o intervalo *wrappa*: `(t_prev, loop_end] ∪ [loop_start, t_now]`). É a mesma lei que curou o relevo, a mordida, o smear e o gate de proteção: *a lei é função do caminho, nunca de quão fino o motor amostrou o caminho* — e é literalmente o bug que o Godot documenta.

### 3. Play-only: scrub / jump / reverse / pause **não emitem**

Arrastar a régua não pode disparar um som a cada pixel (é o default de editor da Unity, pelo mesmo motivo). Emissão só quando `playhead.is_playing()` e o avanço é para frente. Espelha o `hold`/`rewind` da física (que re-baseliza em silêncio). *"Disparar no scrub"* é um toggle futuro, não o default.

### 4. O sinal mora no `Marker`, não numa track nova

`Marker { t, label, signal: Option<String> }` — campo apendado, **`DOC_VERSION` 12 → 13** (postcard é posicional; v12 recusado no load, a política da casa). `signal: None` = anotação pura (byte-idêntico ao de hoje — lei #1, zero regressão). Um marker COM sinal É o Signal Emitter da Unity (que também é um marker). Uma "track de sinais" separada seria um 2º lar para *"um ponto no tempo com significado"* — o marker já é isso.

⚠️ **`label` (o texto que o artista vê) ≠ `signal` (o contrato).** Conflatá-los é o erro da AE. São dois campos.

### 5. A metade visível e o CONSUMIDOR (a bifurcação sua)

Espelhando o precedente da física (canal + flash visível + consumidor de gameplay nomeado como cross-line):

- **v1 ship em `line/anim`, fim-a-fim:** autoria (campo "Signal" no marker + **glifo distinto** quando tem sinal, como a Unity desenha o emitter) · a emissão (§1-§3) · o canal `TimelineSignal` drenável · e o **consumidor VISÍVEL de prova**: um **toast + flash no marker** quando dispara no play (o análogo exato do `×` da física — reação observável, não flag órfão; satisfaz a DIRETIVA §1).
- **O 1º consumidor FUNCIONAL, também em `line/anim` (barato, in-domain):** sinais reservados que o **transporte** consome — um marker `stop` **pausa** o play ao ser cruzado (o *stop marker* de todo editor de vídeo). Decide se entra no v1 ou fica pra W2.
- **Os consumidores REAIS, nomeados e DEFERIDOS (cross-line, decisão sua — a mesma fronteira da física):** o **cue de ÁUDIO** (o passo do personagem → `ph2d-audio` one-shot; é *o* caso de uso, a demo de footsteps da Unity) · **gameplay** · **Luau/MCP** (HR-10). O canal é o ponto de extensão append-only; nenhum deles toca a timeline.

## Alternativas rejeitadas (com o motivo, não "achamos pior")

1. **Chamada de método direta (Godot Call Method Track).** Acopla a timeline ao chamado (viola ADR-0075) **e** o bug documentado *"sporadically not called when spammed"* é exatamente a emissão por igualdade-de-frame sob transição rápida. Rejeitada nos dois eixos.
2. **Nome-de-método-string no clip (Animation Event da Unity).** A própria Unity abandonou: renomear quebra em silêncio — contra a regra *"zero no-op silencioso"* da DIRETIVA. Rejeitada.
3. **Disparar por igualdade de frame (`marker.t == frame_atual`).** Perde markers entre dois ticks (catch-up), duplica na fronteira pausada. É a doença do caminho, medida N vezes neste repo. Rejeitada.
4. **Emitir também no scrub.** Dispara cues enquanto o artista arrasta a régua (default de editor da Unity é play-only por isso). Rejeitada como default; toggle futuro.
5. **Track de sinais separada (o marker track da Unity é separado dos clips).** Já temos markers como dado do doc; uma estrutura nova seria 2º lar para *"ponto no tempo com significado"*. Rejeitada — reusa o `Marker`.
6. **Timeline chama o áudio direto (acoplar `ph2d-timeline` → `ph2d-audio`).** Puxaria o mixer para dentro do runtime de animação. Rejeitada — o áudio é **consumidor do canal**, como qualquer outro (ADR-0075).

## O preço (explícito)

- **`DOC_VERSION` 12 → 13** — quebra dura, v12 recusado no load (política da casa; joga fora nenhum trabalho real de projeto salvo desta linha porque a timeline viaja como blob versionado dentro do `ProjectFile`, `PROJECT_SCHEMA` **não** muda).
- **Play-only** significa que o artista não "ouve" um cue arrastando a régua — tem de dar play. Deliberado (Unity), toggle "fire on scrub" deferido.
- **Os consumidores reais (áudio/gameplay/Luau) são NOMEADOS, não wirados aqui** — cross-line, sua decisão, a mesma cerca da física. v1 entrega o canal + o consumidor de prova (toast/flash) + a autoria. Um canal com só um toast é o precedente aceito da física; não é flag órfão porque o toast **reage**.
- **O `signal` é um contrato-string** — renomear é frágil até haver ferramenta rename-aware; mitigado porque o marker é o carregador estável (como o Signal asset da Unity).

## Gateado (para ninguém re-litigar por prosa)

- `um_sinal_dispara_uma_vez_quando_o_play_o_cruza` — inclui catch-up multi-tick (o intervalo, não a igualdade). Mutação: trocar o intervalo por igualdade-de-frame perde o marker entre ticks → RED.
- `scrub_e_reverse_nao_emitem_nada` — a metade play-only, por camada.
- `um_loop_dispara_os_markers_do_trecho_que_deu_a_volta` — o wrap do intervalo.
- `um_marker_sem_sinal_e_anotacao_pura` — byte-idêntico ao de hoje (lei #1, fingerprint).
- `a_timeline_emite_um_evento_e_nunca_chama_um_consumidor` — arch-gate sobre o fonte (espelha o desacoplamento da física; ADR-0075).
- `o_schema_e_treze_e_um_blob_v12_e_recusado`.
- **Seam que CLICA** (`ph2d-ui-testkit`): autora um sinal num marker → o glifo muda → o play dispara o toast (evento real → efeito observável, DIRETIVA §5).

## Fora de escopo (nomeado, com o gatilho que o acorda)

| Item | Por quê | Gatilho |
|---|---|---|
| **Range/Repeater** (o marker vira faixa; *"estou dentro dele agora?"*) | v1 é o ponto (Trigger); a faixa é a 2ª necessidade do Unreal | pedido de um cue contínuo (ex.: "enquanto nesta janela, X") |
| **Consumidor de áudio/gameplay/Luau** | cross-line; a timeline não pode depender deles (ADR-0075) | sua ordem + o desenho do mapeamento sinal→som |
| **Fire-on-scrub toggle** | default é play-only (Unity) | pedido de artista para auditar cues arrastando |
| **Signal rename-aware** | o marker é o carregador estável; string basta pra v1 | um projeto grande com muitos sinais renomeados |
