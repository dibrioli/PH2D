# HANDOFF DE INTEGRAÇÃO — `line/components`, a REABERTURA dos ABERTOS (2026-09-20)

> ⛔⛔ **Este documento SUPERSEDE o [`…_A_LINHA_2026-09-20.md`](HANDOFF_INTEGRACAO_line_components_A_LINHA_2026-09-20.md)
> como documento de integração.** Aquele fechou a linha e o dono reabriu-a no mesmo dia
> (*«vamos fechar o que está em aberto. tente tudo de uma vez. tilemap está em fase de MVP fora
> daqui. não vamos trabalhar com ele. siga»*). ⚠️ **O §6 dele continua vivo e não foi reescrito
> aqui** — a lição do rebase (as memórias órfãs, as contagens de família paradas) é dele.

**19 commits** · `141` ficheiros · merge-base `395da6a55`.
⭐ **O `main` NÃO andou** desde o merge-base ⇒ *a árvore combinada É esta*, e os censos da soma
foram corridos sobre ela (127/127).

---

## §1 — A superfície de colisão, MEDIDA

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, do caminho **absoluto do
primário** (uma worktree forkada antes do script não o tem, e ele mede a árvore de onde foi
chamado).

| grandeza | aqui | base | delta |
|---|---|---|---|
| `PROJECT_SCHEMA` | `163` | `160` | **+3** |
| └ tripla do gate | `(163, 13, 22)` | `(160, 13, 22)` | **+3** na 1.ª |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` · `FIELD_DOC_VERSION` | — | — | **intactos** |
| registo `ph2d-ecs` / `-render` / `-script` | `103` / `104` / `104` | iguais | **0** |
| contrato congelado (§6) | — | — | **intocado** |
| ADR | — | — | **zero** (fora de toda disputa de número) |
| `Cargo.lock` | — | — | **zero `+name`** externo |
| tectos de LOC | — | — | **nenhum ficheiro da linha passa** |

⚠️⚠️ **CONTE O DELTA, nunca o literal.** São **três** degraus (`161`, `162`, `163`) e eles moram em
**DOIS ficheiros irmãos** — a escada no [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs)
e a tripla no [`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs).
⛔ Um degrau escrito no ficheiro errado **funde limpo e evapora**. O recontador é
`python3 scripts/schema-recount.py`.

⭐ Os três são **sem degrau de migração**, pela decisão do dono de 26/08 (não há projectos
gravados): um blob anterior é **recusado em voz alta** em vez de lido errado em silêncio.

---

## §2 — O que a reabertura fechou, por wave

| # | wave | o que muda |
|---|---|---|
| 1 | **o contador ATRAVESSA um recomeço** | `Counter::keep_on_restart` — outra vida, a mesma pontuação |
| 2 | **uma vida POR INIMIGO** | a porta do contador ganha **ÂMBITO** (`Ambito::Objecto`), e o `soma` deixa de ser só global |
| 3 | **a régua de uma cena de JOGO** | ela abre a mostrar o **relógio do jogo**, não o do editor |
| 4 | **`overheat`** | ⛔ **recusa MEDIDA, com a RECEITA** — a composição já o exprime |
| 5 | **os PERFIS do abanão** | `Recoil · Impact · Explosion` de um clique, com os quatro números por perfil |
| 6 | **o botão de HUD feito de SPRITE** | passa a ser pego (o caminho vectorial era o único) |
| 7 | **a MUNIÇÃO DE RESERVA** | a recarga tira de um **DEPÓSITO que acaba**, e não de um pente infinito |
| 8 | **o censo das secções do Inspector** | toda secção viva chega a PIXEL — e **a nota dizia `26` sobre `4`** |
| 9 | **os TRÊS tectos por quadro** | varridos: **nenhum precisa de um `MAX_*`** hoje, e agora há tabela |
| 10 | **a fileira do gatilho CRIA a acção** | a cura ao lado da queixa (ver §3) |
| 11 | **a vista do HUD** | ⛔ **decisão AFIRMADA**, zero linhas de produto (ver §4) |
| 12 | **o portão batched** | os três vermelhos que o laço interno é cego a (ver §5) |
| 13 | **o roteiro do gatilho** | ele NOMEIA o botão novo, com a metade que diz onde ele NÃO aparece |
| 14 | **o ENUM de um script** | uma propriedade de texto com LISTA vira CHIP, e o valor de fora **não chega ao script** (ver §5-bis) |
| 15 | **a POSIÇÃO e a COR** | uma fileira com DOIS campos e uma AMOSTRA que abre o selector — as duas variantes APENDADAS, zero degrau (ver §5-ter) |

---

## §3 — A wave 10, e porque ela NÃO é uma `ComponentEdit`

A secção *Action Trigger* sabia dizer *«não há nenhuma acção chamada `fire`»* desde a wave dela, e a
cura vivia **noutra janela** (*Settings ▸ Input Map…*). ⚠️ *Uma queixa que nomeia a cura e não a
alcança é meia queixa.*

⭐⭐ **O botão só existe no estado `Desconhecida`, e as três leituras são a lei:**

| estado | o botão | porquê |
|---|---|---|
| `Desconhecida` | **está lá** | a acção não existe — criar é a cura |
| `Ligada` | **não** | criar outra com o mesmo nome é ruído, e faria uma segunda linha |
| `SemTecla` | **não** | ⛔ ali a cura é **OUTRA** (ligar uma tecla), e oferecê-lo mandaria o artista resolver a metade errada |

⚠️⚠️ **O pedido NÃO passa pelo `push` das edições de componente.** O que nasce é uma linha do
[`HeroScreen::input_map`], que é estado do **EDITOR**: não viaja num `ComponentBlob`, não passa pelo
ledger do `preview_drive` e não é do `Ctrl+Z` do documento. Ele é uma
`EditorAction::CreateInputAction` drenada para uma **FASE** ([`fase_criar_accao_do_mapa`]).

⭐ **E a fase existe — em vez de uma linha solta no dreno — porque ela também RE-SINCRONIZA as
linhas do painel do mapa.** Sem isso a acção nasce e a janela do *Input Map*, se estiver aberta,
continua a mostrar a lista **antiga**: o artista vê a queixa desaparecer no Inspector e nada
aparecer ali.

⚠️ **O nome vem da FILEIRA aberta e nunca do store** — ler o store faz o primeiro clique depois de
trocar de objecto mandar o nome do **objecto anterior**.

**Gates.** A costura no painel com **clique REAL** (`Down`+`Up` no rectângulo que a pintura
registou): um `Click` sintético entra por cima da checagem de focabilidade e é **cego** a um
controlo pintado e ausente do `populate` — a família que aquela crate já pagou **sete** vezes. Mais
a corrente da shell sobre o **texto do quadro** (drenar → criar → re-sincronizar), porque a fase
pede `self.gfx` e o `HeroScreen` e nenhum teste de unidade a alcança. ⭐ A fase nova já está coberta
contra **ÓRFÃ** pelo autoteste do próprio instrumento (`the_frame_text_is_the_whole_frame_and_every_phase_is_called`).

**Mutação 5 de 5**, todas a sangrar.

---

## §4 — A wave 11: uma DECISÃO que passou a ser afirmada

O item aberto — *«no EDITOR um HUD colado às bordas cai atrás dos painéis»* — lê-se como um defeito
com uma cura óbvia: alimentar a `fase_hud` com a **BANDA** entre os painéis (a `scene_window`, que é
a lei de todo mapeamento ecrã↔mundo do chrome desde 17/09).

⛔⛔ **Medido, ela está ERRADA**, e o repo já o declarava **sem que nada o protegesse**: o
`todo_aponte_passa_pela_janela_da_cena` tem a câmera do JOGO na partição `A_JANELA_E_O_ASSUNTO`, com
o porquê escrito — *ela mede o ecrã do jogador, não a banda do chrome*.

⭐ Um HUD enquadrado pela banda ficaria bonito no editor e **mentiria sobre o jogo**: a peça colada
ao canto sairia para dentro do ecrã de quem joga, e o defeito só apareceria em quem jogasse. *Entre
mostrar a verdade parcialmente tapada e mostrar uma mentira inteira, esta casa mostra a verdade* — e
a **área segura no editor é decisão de PRODUTO**, nunca a troca desta régua.

⇒ **zero linhas de produto**: o que faltava era a decisão ser AFIRMADA. E a 2.ª metade do gate é a
que impede a catraca de virar licença — ela afirma que a câmera do jogo **ainda** mede a janela; no
dia em que isso mudar deliberadamente, a entrada da partição fica obsoleta e a premissa deste gate
morre com ela, à vista no diff.

---

## §5 — Os TRÊS vermelhos que só o portão batched viu

O laço interno correu verde nas crates editadas a jornada inteira. **Quinta ocorrência** da mesma
cegueira nesta linha: *um fecho que só corre as crates EDITADAS é cego aos gates de arquitectura.*

1. **Um gate NOUTRA crate deixou de compilar** — a wave 7 acrescentou dois campos ao
   `InspectorWeaponInfo`, e o `ph2d-panel-registry-init::o_inspector_armado` arma as 28 portas do
   Inspector **por nome de campo**.
2. **Dez avisos `#[must_use]` ignorados**, todos meus e todos em testes: **mudos** no `cargo check` e
   **mortais** no `ship.sh`, que corre com `-D warnings`.
3. **As DUAS catracas de elisão do degrau estreito** (`103 → 105` cortes · `86 → 88` letras comidas),
   pelos dois rótulos que o contador ganhou: `"Keep on Restart"` (15) e `"Only This Object"` (16),
   numa fileira que oferece **`84 px` ≈ 11 caracteres** (medido: saem `"Keep on Res…"` e
   `"Only This O…"`).

⚠️ **A subida NÃO é a catraca a virar licença**, e o doc daquele ficheiro já declara a família: *são
nomes compostos que precisam de `13`–`15` caracteres e medem `16`–`22`, onde encurtar não é tirar
gordura — é **trocar o nome**: decisão de VOCABULÁRIO, e é do dono.* Até lá o **balão** lê-as, com
gate (`toda_palavra_cortada_tem_balao`) a prová-lo. ⛔ E encurtar está fechado por medição **neste
degrau**: ali um `"Counter"` de SETE letras já pinta `"C…"` em `27 px` — *o corte é da LARGURA, não
do comprimento do nome*.

⚠️⚠️ **E o próprio gate ensinou o ÂMBITO:** corrido com `-p`, ele mede `23` painéis de `27` e
**RECUSA ALTO com a causa na mensagem** (`flip`, `flip_frames`, `painter_layers` e `wet_tuning`
chegam pelo `shells/desktop`). ⇒ *`cargo nextest run --workspace -E 'test(…)'`* — um piso que só o
âmbito rico passa é o que separa uma leitura pobre de uma aprovação.

---

## §5-bis — O ENUM, e o que a medição do §5.0 decidiu

A lista aberta pedia **três** tipos (`Vector2`, `Color`, enum). A sonda
([`mede_o_que_a_composicao_ja_da_aos_tipos`](../../../crates/ph2d-script/tests/it/mede_o_que_a_composicao_ja_da_aos_tipos.rs))
correu antes da 1.ª linha e **reescreveu o âmbito**: os três já têm a CAPACIDADE pela composição, e
só um tem defeito de **CORRECÇÃO**.

| tipo | pela composição | o que falta |
|---|---|---|
| `Vector2` | 2 declarações → 2 fileiras | **afordância** (uma fileira em vez de duas) |
| `Color` | 4 declarações → 4 fileiras | **afordância** (uma amostra em vez de quatro campos) |
| **enum** | 1 declaração de texto | ⛔ **CORRECÇÃO**: o objecto guarda `"fst"`, o painel mostra `"fst"`, **não é órfão** — ele CHEGA ao script, que compara com `"fast"` e cai no ramo errado **em silêncio** |

⇒ a wave é o **enum**, e os outros dois ficam com o preço nomeado (afordância, wave própria) —
✅ **e a wave própria deles fechou a seguir: §5-ter.**

⭐⭐ **Ele NÃO é uma variante nova do `ScriptValue`** — é um `Text` com a lista na PISTA. O
`PropHint` é declaração e não viaja no ficheiro ⇒ **`PROJECT_SCHEMA` intocado**, e o valor continua
a ser o texto que o script compara.

**A lei:** um valor fora da lista **não se aplica nem se converte** — o objecto lê o default, e o
gravado fica **intacto e NOMEADO** (`OrphanWhy::NotAnOption`). É a mesma lei do `WrongKind` uma casa
abaixo, e ⛔ encostá-lo à opção mais parecida é o *«aceita e mente»*.

⚠️ **O `wants` do painel virou ENUM de três estados** (`PorqueOrfao`) e não um `Option` com um
`bool` ao lado: *dois campos que têm de concordar divergem*, e o terceiro estado nasceu assim.

⛔⛔ **E o gate de costura apanhou um defeito antes de ele shipar:** o braço do clique estava
escrito **DEPOIS** do bloco de `Click`, cujo `else` faz `return false` ⇒ ele era **código morto**
para todo clique. É o *«dreno de um braço só»* do §5.0, e ele compila, pinta e regista sem que nada
acuse. ⛔ E a LINHA do popover vem de **quem REGISTA as opções**, nunca do `open` do chip: o
despacho fecha o dropdown no `pointer_down`, logo ali ele já é `false` — a 1.ª redacção lia o store
e o gate reprovou-a.

---

## §5-ter — A POSIÇÃO e a COR, e a porta que eu escrevi duas vezes

✅ **Smoke do dono APROVADO (2026-09-20).**

Os dois tipos que o §5-bis deixou nomeados. **`PROJECT_SCHEMA` intocado, registos intocados** —
as duas variantes do `ScriptValue` são **APENDADAS**, e o postcard é posicional: as três tags de
cima ficam onde estavam e todo ficheiro gravado continua a ler-se. O gate que o afirma é o
`os_valores_viajam_no_fio…`, que passou de `3` para `5` linhas.

### A declaração é um CONSTRUTOR, e é isso que compra a lei

```luau
ph2d.property("direction", ph2d.vec2(0, 1), { min = -1, max = 1 })
ph2d.property("tint", ph2d.color(1, 1, 1))
```

⭐⭐⭐ **A razão não é estética: é que o artista LÊ `self.direction.x`.** Com um `kind` na pista
(`ph2d.property("d", {0,1}, { kind = "vec2" })`) a declaração seria uma **LISTA** e o `self` um
**REGISTO** — *duas formas para a mesma coisa*, e quem copiasse uma para a outra escreveria um
default que o painel não sabe pintar. ⇒ [`valores::tabela_de`](../../../crates/ph2d-script/src/valores.rs)
tem **dois** chamadores (o construtor e o `to_lua` da cena), e o **portão-coroa**
(`a_forma_que_o_artista_escreve_e_a_forma_que_ele_le`) compara as duas tabelas **chave a chave**
sobre a VM real.

⛔ **A desambiguação é EXPLÍCITA** (`__kind`) e nunca pelo comprimento: `{1, 0, 0}` lê-se igual a
uma cor vermelha e a uma posição com lixo no fim.

### As três decisões da lei, cada uma com o mecanismo

| decisão | mecanismo |
|---|---|
| **um canal de cor fora de `0..=1` RECUSA a declaração** | a única superfície que edita uma cor é a AMOSTRA, e o selector é `0..=1` por construção: um `2` seria pintado como `1` e **reescrito em silêncio** no 1.º toque. ⭐ E a recusa **não fecha caminho nenhum** — quem quiser um valor fora da faixa declara três números, como antes desta wave |
| **a faixa vale num `vec2` e é recusada numa `color`** | as duas componentes de uma posição partilham uma faixa (o `@export_range` do oráculo faz o mesmo sobre um `Vector2`); o domínio de uma cor é `0..=1` por natureza, e um `min`/`max` ali é a segunda resposta à mesma pergunta |
| **a finitude passa por uma PORTA** (`ScriptValue::componentes`) | ela era um `if let` sobre o `Number`, logo **um tipo novo com números lá dentro passava calado** — *uma conferência indexada pela variante esquece a variante seguinte* |

⚠️ **E a assimetria com o enum é DECLARADA:** um valor fora da LISTA vira órfão porque o artista o
pode **ESCREVER** (o campo é livre); um canal fora de `0..=1` não vira, porque a única superfície
que escreve uma cor é o selector, que é limitado por construção — *inventar uma quarta razão de
orfandade para um estado que nada produz é construir para um fantasma*.

### ⛔⛔ O gate de costura derrubou código que eu tinha ACABADO de escrever

A 1.ª redacção deu à amostra um braço de `Click` no `event_script.rs` — semear a cor, apontar o
selector, semear o widget. O gate reprovou com **«o ponteiro não virou evento»**, e o **CONTROLO**
(um campo numérico vivo há waves, que emite `Focus + Click`) provou que o instrumento estava bom.

⭐⭐⭐ **A causa:** o despacho **curto-circuita** toda amostra registada por
`register_picker_swatch` — ele abre o selector ele próprio e devolve, *para o clique não focar nem
arrastar o canvas* —, logo **nenhum `Click` chega ao painel**. O meu braço era **código morto**:
*a porta generalizada já existia, e eu escrevi a segunda resposta à mesma pergunta.* O que faz a
amostra funcionar é **uma linha no `populate_script`**.

⚠️ **E a régua teve de mudar com isso:** ali a ausência de evento **é o desenho**, logo a amostra
mede-se pelo EFEITO (o alvo do selector e a semente) e os dois eixos pelo gesto real.

### A afordância, e o que ela NÃO compra

⛔ **`ph2d.set` escreve POSE e mais nada** (`POSE_FIELDS`, cinco campos) ⇒ **um script não sabe
pintar**. A cor chega a `self.tint.r/.g/.b/.a` com a forma certa e o artista usa-a como três
números — é assim que a cena de smoke a demonstra (*o brilho escala o passeio*), e o roteiro
di-lo. ⏳ Um canal de tinta no vocabulário do script é **wave própria**.

⚠️ **O default da cena é a IDENTIDADE:** `direction = (0, 1)` é a reta para cima e `tint` é branca
(brilho `1`) ⇒ os três bonecos fazem **exactamente** o que faziam antes desta wave, com gate a
afirmá-lo — *uma cena que muda de comportamento ao ganhar um controlo deixa de ser a cena que o
dono aprovou*.

### ⚠️ Duas armadilhas de arnês pagas aqui

1. **Uma fixtura recusada por DOIS motivos não afirma nenhum dos dois.** A prova de mutação que
   assume a marca (`__kind` em falta ⇒ *«é um vec2»*) **SOBREVIVEU**, porque cada tabela do gate
   também não tinha os campos. O caso que discrimina é um **registo com `x` e `y` e sem marca**.
2. **O `cargo fmt` reescreveu uma âncora da mutação** depois de ela ter sido copiada da linha
   única — e *uma âncora que não casa lê-se exactamente como uma mutação que sobreviveu*. Quem a
   apanhou foi o controlo de contagem do arnês.

**Prova:** `16` de `16` mutações sangram
([roteiro](../ferramentas/mutacao_tipos_do_script_2026-09-20.sh)).

---

## §5-quater — O levantamento do catálogo da Godot (docs, **ZERO linhas de produto**)

Ordem do dono, depois do smoke: *«Vá até o manual da godot. Descubra tudo que ela tem e que pode se
transformar em um objeto ou em um componente de nossa engine. Faça essa pesquisa minuciosa.»*

⚠️ **Para o integrador: este commit não toca em código.** Três ficheiros, todos sob `docs/` —
nenhuma crate, nenhum contador partilhado, nenhum gate, nenhum `Cargo.lock`.

| ficheiro | o que é |
|---|---|
| `docs/Components/pesquisa/dossie_godot_o_catalogo_medido_2026-09-20.md` | o levantamento (novo) |
| `docs/Components/ferramentas/godot_classdb_probe.gd` | a sonda que o produziu (nova, versionada) |
| `docs/Components/pesquisa/dossie_godot.md` | **+9 linhas** de cabeçalho: o ponteiro para o irmão medido |

⭐ **O método é o do §0.9, e é a diferença inteira:** o dossiê de 2026-08-20 foi **lido** de
`docs.godotengine.org` e cita ~60 nós; este foi **corrido** do binário instalado (`ClassDB` da
4.7.2, MIT) e enumera **1 054** classes com a superfície de cada uma — propriedades com tipo e
dica, grupos do inspector e sinais. *Uma leitura não enumera, ela recorda.*

⚠️ **E ele corrigiu três números que eu ia escrever de memória**, o que é a razão de a contagem
vir de correr e não de varrer: o catálogo tem **157** descritores (medido com
`ph2d_component_desc::catalog::all()`; três `grep` diferentes deram três respostas erradas, porque
os `requires:` citam nomes alheios e cada família usa um construtor próprio), os painéis são **27**
e os formatos de imagem **16**.

⛔ **Nada ali é um plano.** A §8.1 lista o que ele tem e nós não, **sem ordem, de propósito** —
ordenar seria decidir, e a decisão é do dono.

## §6 — O que uma leitura rápida do diff entende ao CONTRÁRIO

1. **O `+3` do `PROJECT_SCHEMA` não é de uma wave só** — são três, e nenhum tem degrau de migração
   (decisão do dono de 26/08). Conte o DELTA contra a árvore em que vai aterrar.
2. **A wave 11 não tem linha de produto**, e isso é o ponto: ela **afirma** uma decisão que já
   existia e que ninguém protegia.
3. **As catracas de elisão SUBIRAM**, e não é regressão nem licença — é a família de vocabulário que
   o próprio ficheiro declara (§5).
4. **O `overheat` (wave 4) não foi construído de propósito** — é recusa medida **com a receita**
   escrita ao lado. Reconstruí-lo é refazer trabalho já pago.
5. **A nota do censo das secções dizia `26` e a medição deu `4`** (wave 8): *uma lista de dívida
   citada de cor manda reconstruir trabalho já pago, e assusta com um número que ninguém contou.*
6. **O botão de criar acção não é uma edição de componente** (§3) — um braço que o empurrasse como
   `ComponentEdit` compilaria e poria uma linha do Input Map **dentro do ficheiro do projecto**.
7. **Os três tectos da wave 9 não ganharam `MAX_*`** — a sonda IMPRIME e sai; o que fica no repo é a
   TABELA, que é o que o §0.0 pede *antes* de alguém escrever um limite.

---

## §7 — O que FICA aberto, com o mecanismo (auditado contra o CÓDIGO)

⚠️ **Cada linha abaixo foi medida nesta jornada, não recitada.**

| item | estado MEDIDO |
|---|---|
| **o `.luau` não viaja no projecto** | ⛔ **Mesma fronteira DECLARADA do áudio, e não é do script:** medido, o `AudioSource2D` também **nomeia um ficheiro e também não embute**, e o doc do `LuauScript` já escreve que *«a cura das duas é a mesma: pôr o tipo no índice de assets»*. Um embed só para o script seria **a segunda resposta à mesma pergunta**. ⇒ wave do índice de assets, não desta linha. |
| **`SignalFrom::Tagged`** | ⛔ Sem consumidor, e a metade que importa está COBERTA: para a origem que toda cena tem — o TOQUE — a composição **já** o exprime pelo `SignalTagFilter`, que é do **produtor** (a `=dano` usa-o). Para as outras origens nenhuma cena pede. |
| **um script não sabe PINTAR** | ⛔ **Medido:** o `ph2d.set` escreve `POSE_FIELDS` — `x`, `y`, `rotation`, `scale_x`, `scale_y` — e mais nada. Uma propriedade `color` chega ao script com a forma certa (`self.tint.r`) e ele usa-a como NÚMEROS; a tinta de verdade pede um canal novo no vocabulário, que é wave própria. ⚠️ O `Vector2` **não** tem este limite: a pose é escrevível, e é por isso que a cena de smoke o demonstra a sério.
| **script e física no MESMO corpo** | Fronteira declarada no handoff do #16. |
| **HUD atrás dos painéis no EDITOR** | ⭐ **Deixou de ser um item de defeito** — é consequência de uma decisão, hoje afirmada por gate (§4). O que sobra é a **área segura** no editor, que é decisão de produto. |
| **abanão ANGULAR** | ⛔ Recusa medida: a `CameraView` tem **três** campos e nenhum é um ângulo. |
| **morte de pré-visualização** · **vida POR inimigo no smoke** | Decisão do dono. |
| **troca de NÍVEL** | Pede um segundo documento de cena; o projecto é **um** mundo. Arquitectura, não wave. |

---

## §8 — Prova de fecho

- `bash scripts/nextest-impacted.sh` → **17 702 / 17 702**, zero reprovadas (a corrida do §5-ter).
  ⚠️ A penúltima corrida acusou **dois** tectos de LOC, curados por **CORTE** — o pintor por TIPO
  saiu da `linha` (`242` de `200`) e as seis amostras de tinta saíram do `sync` (`603 → 531`), as
  duas **verbatim e na mesma posição**. ⛔ Nenhuma entrada nova no `FILE_OVERAGE_OK`, que continua
  **vazio**.
- corridas anteriores desta reabertura → **17 684 / 17 684** — ⚠️ com **duas** a acusarem
  **UM** membro NOMEADO da família de flakes de fan-out cada, e **membros DIFERENTES entre elas**
  (`the_cost_of_a_player_is_linear_in_their_number` · `the_cost_of_depth_is_linear_not_explosive`),
  os dois `3 de 3` verdes sozinhos a `load 28`–`46` e com **zero linhas de diff** nas crates deles.
  ⭐ *O conjunto de reprovadas MUDAR entre corridas é a assinatura mais forte da família.*
- `bash scripts/censos-da-arvore-combinada.sh` → **127 / 127** (controlo do filtro: 12 de 12 censos
  correram). ⚠️ O `main` **não andou** desde o merge-base ⇒ a árvore combinada É esta.
  ⭐ Um deles reprovou no §5-ter e a cura é uma **isenção NOMEADA**: as duas formas com que um
  valor de script se lê numa linha de órfão (`vec2(…)` · `color(…)`) **não são língua** — são o
  construtor que o artista escreveu no `.luau` dele, e traduzi-las mostrava-lhe uma forma que ele
  não pode escrever. É a mesma isenção que o rótulo de uma opção de enum já carrega, e ela entra
  na lista **com o mecanismo**, que é o que aquele gate exige.
- `cargo clippy --workspace --all-targets -- -D warnings` → **zero**
- `cargo fmt --all` → limpo
- provas de mutação desta reabertura: **7 de 7** (wave 10) + **2 de 2** (wave 11) + **2 de 2**
  (wave 13) + **12 de 12** (wave 14) + **16 de 16** (wave 15, o §5-ter), todas a sangrar
  — ⚠️ com o arnês a abortar alto **duas** vezes (uma âncora que casava `2×`; uma mutação
  **NO-OP para o gate**, porque um sufixo deixa a agulha do `contains` intacta e isso lê-se
  exactamente como sobreviver).

⛔ **A linha NÃO integra e NÃO pusha** (§0.7). Ela fecha, entrega isto e PARA.
