# 13 — As oito crates de UI que nunca tiveram régua

> **Medido em 2026-09-17, `line/UIUX`.** A 8.ª fatia do HR-15, e a que a anterior deixou nomeada:
> *um censo que não corre sobre uma crate não afirma nada sobre ela.*
>
> Oito crates de UI — entre elas painéis que o artista usa todos os dias — não tinham o
> `every_word_this_panel_shows_comes_from_the_string_table`. Hoje têm, e uma nona (`ph2d-app-skeleton`)
> também.

## §1 — O que elas guardavam, medido antes de escrever uma linha

| crate | literais com cara de língua |
|---|---:|
| `ph2d-panel-widget-lab` | 15 |
| `ph2d-panel-tags` | 8 |
| `ph2d-app-skeleton` | 2 |
| `ph2d-panel-model3d` | 1 |
| `ph2d-panel-physics` | 1 |
| `ph2d-panel-sculpt3d` · `-skeleton` · `-wet-tuning` · `-widget-gallery` | **0** |

⭐ **Quatro já estavam limpas — o que lhes faltava era quem as mantivesse assim.** Uma crate a zero
sem régua e uma a zero com régua leem-se igual hoje e divergem no primeiro literal que alguém
escrever.

## §2 — O painel TAGS: oito frases que o artista lê

Eram os verbos da barra (`+ New`, `+ Child`, `Rename`, `Move to root`), três frases com peças do
código e a frase do vazio. ⭐⭐ **A mais importante é o rótulo de apagar, porque ele CARREGA O
ESTRAGO** — apagar `Enemy` leva `Flying` e `Boss` junto, e o artista só vê isso se o botão o disser
**antes** de ser carregado:

```
Delete ({tags} tags, {objects} objects)     ← a tag tem filhas
Delete ({objects} objects)                  ← a tag é folha
```

⚠️ As três com marcadores vão por `tr_with` com nomes: colar o número no pintor fixaria a ordem das
palavras, e há línguas em que a contagem vem à frente.

## §3 — ⛔ Duas frases tinham as PALAVRAS na tabela e a SINTAXE no pintor

```rust
format!("{}: {} · {} {:.1} ms", tr("panel.model3d.nodes"), n, tr("panel.model3d.trace_cost"), ms)
format!("{}: {:.0} px/m",       tr("panel.physics.scale"), pixels_per_meter)
```

Os rótulos vinham da tabela — e a **ordem** deles, os dois-pontos e o separador não. ⇒ a frase
passa a ser uma chave com marcadores (`"{nodes}: {count} · {cost} {ms} ms"`), e os dois rótulos
entram nela como **argumentos**: a frase não os duplica. ⚠️ O `ms` e o `px/m` ficam **dentro** da
frase — são símbolos de unidade, iguais em toda língua, e parti-los num argumento daria a alguém a
ideia de os traduzir.

## §4 — ⛔⛔ O achado: oito chaves órfãs, e dezanove que PARECIAM órfãs e estavam VIVAS

A metade de obsolescência do censo de chaves acusa o que a tabela declara e ninguém usa — *uma
string órfã é onde alguém escreve, um dia, uma frase sobre um controlo que já não existe*.

**Órfãs a sério, apagadas:** `panel.model3d.kind.*` (6) · `panel.model3d.radius` ·
`panel.physics.sleep_spin`. ⚠️ A última tinha, na linha de cima da tabela, a nota que explica porquê:
a fileira passou a ler *"Enabled"* quando se mediu que a `rapier` lê o **sinal** daquele campo e não
a magnitude — *a chave do rótulo antigo ficou para trás*. E a do `radius` levou consigo a prosa que
a explicava, porque **uma nota que descreve uma chave que já não existe lê-se como auditada**.

⛔⛔ **E dezanove NÃO eram órfãs:** as `panel.wet_tuning.knob.*` não aparecem como literal em sítio
nenhum porque são **montadas em runtime** —

```rust
label: format!("panel.wet_tuning.knob.{}", d.key)
```

⇒ *uma régua que mede LITERAIS prescreve a cura errada para uma chave que é ASSEMBLADA*, e a cura
errada aqui era **apagar os rótulos de dezanove knobs do painel**. A isenção fica nomeada, e o que a
impede de virar licença é o gate **ler o fonte**: se o `format!` sair, a isenção deixa de descrever
o código e reprova.

⚠️ **E por isso os dois pisos daquele gate são DIFERENTES** (`declaradas ≥ 40`, `usadas ≥ 8`): a
1.ª redacção pôs `20` nos dois e reprovou sobre produto correcto. *A diferença entre os dois lados
**é** a família montada.*

## §5 — ⛔ A régua conhecia UMA das duas formas de prefixo

O censo de chaves já rejeitava uma chave acabada em `.` — porque um gate que escreve
`"panel.inspector.player."` para separar meio vocabulário era lido como uma chave em uso. Mas o
mecanismo tem **duas** formas, e a segunda mordeu aqui: o censo da escultura escreve

```rust
"panel.sculpt3d.cfilter_"   // e cola-lhe o nome do knob
```

⇒ *a cura de 2026-09-13 curou uma forma de prefixo e o mecanismo tem duas.* A régua passa a
rejeitar também o `_` final, **medido antes de escrito**: nenhuma das chaves declaradas neste repo
acaba em `_`.

## §6 — ⛔ E um TESTE de outra crate inventa uma chave para provar um negativo

```rust
assert!(slot_of("panel.model3d.add.nao_existe").is_none());
```

Tem a forma exacta de uma chave e nunca chega a pixel nenhum — e o censo leu-a como *usada e não
declarada*, que é a acusação mais grave que ele faz (ela diz *«isto pinta o identificador cru e vaza
por quadro»*). ⚠️ **A régua partilhada não o pode distinguir**: só o CONTEXTO o diz. ⇒ isenção
nomeada no gate do painel, com a metade justa a exigir que o controlo **ainda exista** no ficheiro
de onde veio.

## §7 — Os números

| | antes | depois |
|---|---:|---:|
| crates de UI sem régua de HR-15 | **9** | **0** |
| testes na família dos censos | 103 | **155** |
| chaves órfãs na tabela | 8 | 0 |
| literais de língua nas nove crates | 27 | 0 (15 numa bancada isenta com mecanismo) |

**Prova de mutação: 5 de 5 sangram**, com controlo negativo a 87 testes verdes — um literal novo
numa crate recém-gateada · a isenção da bancada sem mecanismo · o `format!` que monta as chaves a
desaparecer · o controlo negativo da outra crate a sair · e a régua a esquecer o prefixo acabado em
sublinhado.

## §8 — ⏳ O que fica

| alvo | nota |
|---|---|
| `ph2d-tool-vector` | 7 braços de rótulo, fora dos motores de nó |
| o MENU e a ABA têm chaves diferentes para a mesma palavra | ver `12_o_nome_de_um_painel_tinha_duas_fontes.md` §6.1 |
| o `keys_used` conta USOS em ficheiros de TESTE | ele exclui as tabelas e não o código de teste — a cura geral é perguntar ao pai (`is_declared_under_cfg_test`), e o alcance dela é as ~20 crates já gateadas |
