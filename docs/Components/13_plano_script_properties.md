# TOP-20 #16 — `ScriptProperties` + anexar um script pela UI: o plano

> *«host/persistência/determinismo prontos, ZERO UI — o mecanismo nº 1 de script parametrizável»* —
> [levantamento §7](00_levantamento_componentes.md).
>
> Fila: os itens **1–15** estão fechados. Este é o **16**, e o levantamento chama-o *«a válvula de
> escape»*: o que os componentes do catálogo não fazem, o jogo do utilizador escreve.

---

## §0 — O que o artista consegue FAZER quando isto fechar

Escrever **um** ficheiro `.luau` que declara os seus números no topo —

```lua
ph2d.property("amplitude", 1.5, { min = 0, max = 10 })
ph2d.property("speed", 2)

function update(self, dt)
  self.t = (self.t or 0) + dt
  ph2d.set(self.id, "y", self.base_y + math.sin(self.t * self.speed) * self.amplitude)
end
```

— anexá-lo a **N** objectos por **Add Component → Script → Browse**, dar a cada objecto **os seus
próprios números no Inspector**, e carregar em Play: cada objecto mexe-se pelo mesmo script com os
números dele. Mudar o ficheiro num editor de texto muda a cena **sem reiniciar o app**.

---

## §1 — ⭐⭐⭐ A PERGUNTA DE ANTES: o que JÁ existe? (§5.0)

Medido pela leitura da crate e dos chamadores (2026-09-16). O levantamento diz *«host, persistência
e determinismo prontos»* — **as três palavras estão erradas**, e cada uma com mecanismo:

| afirmação | medido |
|---|---|
| **persistência pronta** | ⛔ **Não.** O `register_script_components` **não é chamado no boot** (o catálogo já o regista como armadilha): um `LuauScript` posto num objecto **evapora** ao gravar e ao desfazer |
| **host pronto** | ⛔ **Não por objecto.** O `ScriptHost` tem **UMA** VM a correr um *placeholder* que imprime uma linha; não há `init`/`update` por entidade, o `WriteQueue` **não tem quem o drene** (há gate a dizê-lo: *«`drain_spawns()` sem chamador»*), e o `provide_read` nunca é chamado |
| **determinismo pronto** | ⛔⛔ **O contrário.** `LuauScript::lateral_key = entity.to_bits() ^ hash` — **bits de alocação DENTRO dos bytes de um componente**, que é exactamente o que o §5 deste repo proíbe: *o undo respawna tudo com bits novos, e bits dentro dos bytes envenenam o próprio undo*. Só não mordeu porque o componente nunca foi gravado |
| **o componente sabe qual script** | ⛔ Por um `AssetId` de **bytecode** — e **nenhum** sítio do app compila Luau para um asset, nem há onde o artista escolha um |
| **parâmetros por objecto** | ⛔ **Zero.** Não há declaração, nem valor por instância, nem painel |

⇒ **O que existe é a LINGUAGEM** (Luau em sandbox, as ligações `ph2d.*`, o *reset+restore* por hash
de conteúdo) **e nada da cadeia objecto → script → número → efeito.** Este item constrói a cadeia.

---

## §2 — O ORÁCULO (§0.9), e as DEZ leis que ele deu

**Triagem de licença:** **Godot 4.7.2**, `/usr/bin/godot`, **MIT** — porta ABERTA. O `@export` dele
é o mecanismo de script parametrizável de uma engine madura com licença permissiva nesta máquina.
(Defold `go.property` é o modelo mais limpo para Lua, mas **não está instalado**; Unity e Roblox são
proprietários.)

Corrido sobre entradas **nossas** por
[`godot_export_probe.gd`](ferramentas/godot_export_probe.gd): grava uma cena com o script **v1**,
reescreve o **ficheiro** do script (**v2**) e recarrega com `CACHE_MODE_IGNORE_DEEP`.
⭐ **O controlo C0** (grava `9`, recarrega sem mudar o script) lê `9` — a sonda mede o alvo, não a
cache.

| # | pergunta | o que o oráculo FAZ |
|---|---|---|
| Q1 | objecto **sem** valor próprio; o default muda `4 → 7` | **segue: lê `7`** (o `.tscn` não guarda nada) |
| Q2 | objecto **com** `9`; o default muda | **guarda o dele: lê `9`** |
| Q3 | ⭐ objecto com `4` **igual** ao default antigo; default `→ 7` | ⚠️ **lê `7`** — um valor igual ao default **não é gravado**: para o alvo, *«próprio»* = *«difere do default no instante de gravar»* |
| Q4 | objecto com `9`; a propriedade **sai** do script | `get` = `null`; o `.tscn` **ainda tem** `speed = 9` |
| Q4b | ⛔⛔ a cena é **re-gravada** assim e a propriedade **volta** | ⛔ **lê `4`** — **o `9` perdeu-se em silêncio** ao re-gravar |
| Q5 | `9` (int); a propriedade passa a `String` | lê o **default novo** (`"lento"`); o `9` fica no ficheiro até à próxima gravação |
| Q5b | ⛔ `"rapido"`; a propriedade passa a `int` | ⛔ **lê `0`** — converte texto em número **em silêncio** (e **não** é simétrico com Q5) |
| Q6 | `15` com faixa `0..20`; a faixa estreita para `0..10` | **lê `15`** — a faixa é pista de EDIÇÃO, o load não prende |
| Q7 | ordem das propriedades no painel | **a da declaração** (`zeta, alpha, label`) |
| Q8 / Q8b | `2,75` gravado e a propriedade passa a `int` / `9` e passa a `float` | trunca para `2` / promove para `9.0` |
| Q9 | duas instâncias do mesmo script | **independentes** (`9` e `7`) |
| Q10 | o **ficheiro** do script sumiu | a cena **carrega**, o nó guarda os valores num *placeholder* (`speed` lê `9`) |

### §2.1 — As três DIVERGÊNCIAS declaradas (e gateadas)

- **D1 (contra Q3) — *próprio* é o que o artista PÔS, não o que difere.** A lei desta casa já está
  escrita ([memória](../../project-memory/feedback_the_artists_key_outranks_the_automatic_correction.md)):
  *a chave que o artista fez manda mais que a correcção automática, e o discriminador é quem PÔS,
  nunca um limiar*. Um `4` escrito à mão continua `4` quando o default passa a `7`; o **Revert** é
  que o larga.
- **D2 (contra Q4b) — um valor órfão NUNCA se perde em silêncio.** Ele fica no componente, o
  Inspector **nomeia-o** (*«`speed = 9` — não está no script»*) e o artista larga-o um a um — a lei
  que a F5 desta linha já pagou para os órfãos de instância. ⚠️ O Q4b é o defeito que isto evita:
  **renomear uma variável e voltar ao nome** (um erro de digitação corrigido) apaga o trabalho do
  artista no alvo. E com um script **temporariamente partido** (erro de sintaxe, ficheiro a meio de
  ser gravado) o alvo não conhece propriedade nenhuma — uma gravação nesse instante apagaria todas.
- **D3 (contra Q5b) — um valor do TIPO errado não se converte.** Ele não é aplicado (o script lê o
  default), e o Inspector nomeia-o como órfão de tipo. O Luau tem **um** tipo numérico, então o par
  Q8/Q8b não tem onde acontecer aqui.

### §2.2 — As leis PORTADAS

Q1, Q2, Q6, Q7, Q9, Q10 — cada uma com gate de nome próprio sobre a lei pura.

---

## §3 — O desenho, com a porta ÚNICA de cada pergunta

### §3.1 — *Que script?* ⇒ um CAMINHO, como o som do #4

O `AudioSource2D` já é *«o primeiro componente registado desta casa que guarda um caminho»*, com as
duas consequências declaradas (mover o ficheiro parte a ligação; o projecto não embute o ficheiro) e
a cura das duas nomeada (pôr o tipo no índice de assets). ⇒ **o mesmo contrato**, e a mesma UI: uma
linha de texto com o caminho, um **Browse**, e um aviso quando o ficheiro sumiu.

⛔ **O `bytecode: AssetId` sai.** Nenhum sítio do app compila Luau para um asset, e o *reset+restore*
por hash de conteúdo que o `ScriptHost` já faz é exactamente o que um caminho precisa para recarregar
quando o ficheiro muda (HR-16).

### §3.2 — *Que números este script oferece?* ⇒ `ph2d.property(nome, default, opções?)` no TOPO

É o modelo do Defold (`go.property`) com a ordem do Godot (Q7): **a ordem das chamadas é a ordem do
painel**. O tipo sai do default (`number` · `boolean` · `string`); as opções são **pistas de edição**
(`min`/`max`/`step`), e não prendem o valor gravado (Q6).

⚠️ **A colheita corre o script** (na sandbox, uma vez por conteúdo) — é a única forma de a resposta
ser a da linguagem e não a de um *parser* nosso ao lado dela. ⛔ Chamada fora do topo (dentro de um
`update`) é **erro**: uma declaração que só existe depois de a corrida começar é uma linha de painel
que aparece e desaparece.

### §3.3 — *Que valor o objecto usa?* ⇒ uma função PURA, `resolve`

`resolve(declaracoes, proprios) → { valores na ordem da declaração, cada um com a ORIGEM
(default | próprio) } + { órfãos, cada um com o PORQUÊ (sumiu | tipo) }`.

É **a** resposta, com **três** leitores: o `self` do script, a linha do Inspector (a cor de *«próprio»*
e o botão de Revert) e a lista de órfãos. *Uma lei escrita em dois sítios ainda não é uma lei.*

### §3.4 — *O que fica gravado?* ⇒ só o que o artista PÔS (D1)

`LuauScript { source: String, overrides: BTreeMap<String, ScriptValue> }` — **CONFIG**, registado.
`BTreeMap` pela espinha do determinismo; a ordem do painel vem das declarações, nunca do mapa.
⛔ **Nenhum bit de entidade dentro dos bytes** (§1).

### §3.5 — *O que o script FAZ?* ⇒ três ganchos, e o mundo por DUAS portas

| gancho | quando |
|---|---|
| `init(self)` | a primeira vez que a corrida o encontra — e outra vez depois de um rebobinar |
| `update(self, dt)` | **uma vez por PASSO FIXO**, com `dt` = o passo — ⚠️ nunca `ticks × dt` numa chamada: ao contrário da lei pura do relógio, um script **não tem laço de recuperação**, e agrupar os tiques faria o replay depender da taxa de quadros |
| `on_signal(self, nome)` | cada sinal ouvido neste quadro |

`self` é uma tabela por objecto com `self.id` e **os valores resolvidos** (`self.amplitude`). ⚠️ Uma
edição no Inspector **durante a corrida** chega ao `self` **no tique seguinte**, e só a propriedade
editada: um script que ajuste o seu próprio `self.speed` mantém o ajuste até o artista mexer nesse
número — *a mão do artista manda, mas só onde mexeu*.

As duas portas para o mundo:
- **`ph2d.get` / `ph2d.set`** sobre `x · y · rotation · scale_x · scale_y` do próprio objecto — as
  ligações que o spike já tinha, agora **drenadas**. Um campo desconhecido é **nomeado** no painel,
  nunca ignorado.
- **`ph2d.emit(nome)`** publica um sinal no MESMO outbox de toda a casa (`SignalOrigin::Script`) — é
  a regra de produto do levantamento: *o script liga-se ao mundo pelos MESMOS sinais da tabela de
  acções, nunca por referência directa a outro objecto*.

### §3.6 — *Quando corre?* ⇒ quando o relógio anda, e na ordem da IDENTIDADE

É o gate da fábrica e da física (`playhead.is_playing()`), e a ordem entre objectos é a do
`StableId` (HR-5). ⛔ **O VIVO não é documento:** as tabelas `self` vivem na VM, e **rebobinar é
renascer** — o invariante do rebobinar da shell apaga-as e o `init` volta a correr.

### §3.7 — *E o `Ctrl+Z`?* ⇒ um motor PERSISTENTE durante a corrida

A pose que o script escreve é **pré-visualização** (a lei do `preview_drive`) com um driver próprio
(`Driver::ScriptPose` — ⚠️ **não** o `SolverPose`: a chave do ledger é `(entidade, driver)`, e um
objecto que seja também um corpo teria duas mãos na mesma entrada, e o *«outra mão escreveu»* de uma
engoliria o autorado da outra).

| momento | o que o ledger ouve | porquê |
|---|---|---|
| a correr, o script escreveu | `driven(antes, depois)` | a lei de sempre |
| **pausado a meio** | `driven(agora, agora)` **só se já havia entrada** | ⚠️ um script **pausado não acabou**: sem isto a `settle` promovia a pose da corrida a documento e o rebobinar já não tinha para onde voltar. E com `antes = agora`, **um arrasto feito na pausa é adoptado** como o novo autorado pela regra da *outra mão* |
| rebobinar | `release_to_authored` | a pose volta à que o artista pôs |

### §3.8 — *E se o script partir?* ⇒ o erro é NOMEADO no painel, e a corrida continua

Erro de sintaxe ⇒ o script não tem declarações (o painel diz porquê e **os valores próprios ficam** —
D2). Erro num gancho ⇒ aquele objecto pára de correr o script até um rebobinar ou uma recarga, e o
painel mostra a mensagem. ⚠️ **Um laço infinito não pode congelar o app:** a VM tem uma
**interrupção** com prazo de **um quadro** (16,7 ms) por passagem — o recurso é o quadro, não um
número escolhido.

### §3.9 — O que NÃO entra, e porquê

| fora | motivo |
|---|---|
| Tipos `Vector2` / `Color` / enum / recurso | o Luau não os tem nativos; um `{x, y}` pediria um contrato de tabela — wave própria, com consumidor |
| Mais de um script por objecto | ADR-0025: *«sem multi-script-por-entity como design primário»* |
| `find_by_name` alimentado | a regra de produto do §3.5 — um script fala com o mundo por sinais |
| Editor de texto dentro do app | o fluxo é o de Godot/Defold com editor externo, e a recarga por hash é o que o torna vivo |
| Compilar para bytecode / embutir no projecto | o mesmo preço que o som do #4 paga, e a mesma cura (o índice de assets) |
| Um script e a física no MESMO corpo | duas mãos no mesmo `Transform`; o painel **avisa** |

---

## §4 — Onde encosta em contrato congelado (§6) ou schema

- **Contrato congelado: NÃO.** Nada em `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/
  `CanvasPaintTool`/`PanelEvent`, nem na superfície do `ph2d-vector-doc`.
- **`PROJECT_SCHEMA`: +1** (delta) — o `LuauScript` passa a ser GRAVADO (o registador entra no boot).
- **Registos:** o do `ph2d-script` já conta o `LuauScript` (`ecs + 1`) ⇒ **não muda**. O `ph2d-ecs`
  e o `ph2d-render` também não. ⚠️ Conferir **no gate**, que imprime o `left:`.
- **`LIVE_SECTIONS`: +1** (a secção do Inspector).
- **`ph2d-preview-drive`: +1 variante** (`Driver::ScriptPose`) — enum sem `match` fora da crate
  (conferir por grep antes de a acrescentar).
- **`ph2d-runtime`: +1 origem** (`SignalOrigin::Script`).
- ⚠️ **A shell tem 745 linhas de folga** contra o tecto de `196 990` (medido 2026-09-16:
  `196 245`). ⇒ a ponte, a cena e a conversão para o Inspector vivem em **`ph2d-app-components`**; a
  shell só compõe.

---

## §5 — As quatro condições de UI (independentes)

1. **existe** — `LuauScript` passa de `machinery` a `authored`, `O::ANY`, família `Scripting`.
2. **é pintado e registado** — a secção **Script**: o caminho + **Browse** + os avisos; uma linha por
   propriedade declarada (número / caixa / texto), com a cor de *«próprio»* e o **Revert**; uma linha
   por órfão com **Remove**.
3. **o clique chega ao barramento** — `ScriptFieldEdit` pelo `EditorAction`.
4. **a sequência leva a algum lugar** — a cena de smoke: três objectos, um script, três conjuntos de
   números; mudar um número muda o movimento; mudar o ficheiro muda a cena.

---

## §6 — As waves

| W | o quê | pronto quando |
|---|---|---|
| **W1** | a **lei pura** (`ScriptValue`, declaração, `resolve`) + a **colheita** na VM (`ph2d.property`, um ambiente por script) | Q1–Q10 e D1–D3 gateados; dois scripts não se pisam |
| **W2** | o **componente** novo (gravado) + a **VM por objecto** (ganchos, `get`/`set`, `emit`, interrupção, recarga) + a **ponte** (passo fixo, ledger, rebobinar, sinais) | um objecto mexe-se pelo script, headless; rebobinar devolve a pose |
| **W3** | a **secção do Inspector** + o **Browse** | as 4 condições do §5 |
| **W4** | a **cena de smoke** + provas de mutação + handoff | o dono corre e vê |

---

## §7 — ⚠️ As premissas DESTE plano que a implementação derrubou

*(escrito durante a construção, não depois — a §6 acima fica como foi planeada, de propósito)*
