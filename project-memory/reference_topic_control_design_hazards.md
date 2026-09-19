---
name: reference-topic-control-design-hazards
description: "Família: como um CONTROLO mente — morto vs ausente, faixa derivada do que ele escreve, rótulo que promete o que o modelo não dá, variante removida do meio de um enum"
metadata:
  type: reference
---

⚠️ **Índice de família, 2 saltos.** Saíram do `MEMORY.md` porque ele passou o limite de leitura e
as últimas linhas desapareciam em silêncio. Cada entrada continua a ser um ficheiro próprio.

*Pergunta-mãe: **o que este controlo promete, e o que ele de facto faz ao modelo?***

- [O desenho pedido pode já ser LEI noutra metade do app — procure antes de desenhar](feedback_the_design_being_asked_for_may_already_be_law_in_another_half_of_the_app.md)
- [MORTO ≠ AUSENTE: procure o controlo que JÁ faz a pergunta antes de construir](feedback_a_dead_control_and_an_absent_one_read_the_same_and_building_is_the_wrong_cure.md)
- [Knob morto: 2 espécies que gate nenhum apanha — siga o TERCEIRO passo; e a 3.ª LEITURA: consumidor GENÉRICO lê-se como consumidor NENHUM](feedback_a_dead_knob_has_two_species_no_probe_catches.md) · [id ÓRFÃO ≠ knob MORTO: curas opostas](feedback_an_orphan_id_and_a_dead_knob_read_the_same_and_their_cures_are_opposite.md) · [smoke que ensina o CONTRÁRIO é pior que ausente](feedback_a_smoke_scene_that_teaches_the_opposite_is_worse_than_no_scene.md)
- [«Difícil de ajustar» = bug de DESIGN](feedback_ergonomics_verdict_is_a_design_bug.md)
- [Faixa tirada do objecto que o botão substitui = não idempotente](feedback_a_knob_whose_range_is_derived_from_the_object_it_rewrites_is_not_idempotent.md)
- [Knob por-passo é ALVO](feedback_a_knob_consumed_as_a_per_step_rate_is_a_target_not_a_rate.md) · [remédio novo = contagem dupla](feedback_a_new_remedy_makes_the_old_one_double_counting.md) · [param inerte: grepe o consumidor](feedback_a_parameter_that_changes_nothing_is_discarded_downstream.md)
- [Campo colapsado MANDA](feedback_a_collapsed_field_does_not_go_neutral_it_takes_over.md) · [rótulo promete o modelo](feedback_a_label_must_promise_what_the_model_delivers.md) · [affordance se re-deriva](feedback_inherited_affordance_must_be_rederived.md)
- [⛔ Numa lista aplicada por varredura, NÃO nomear é comandar DESLIGADO](feedback_not_naming_a_thing_in_an_absolute_list_is_commanding_it_off.md)
- [Tirar variante do MEIO de um enum serializado reescreve ficheiros gravados, sem erro](feedback_removing_a_middle_variant_from_a_serialized_enum_silently_rewrites_saved_files.md)
- [bool onde havia ID apaga a próxima](feedback_publishing_a_bool_where_the_source_had_an_id_throws_away_the_next_feature.md) · [não-idempotente: autoria ≠ depósito](feedback_a_nonidempotent_target_excludes_nothing_split_authoring_from_deposit.md)
- [[feedback_when_the_composition_already_expresses_the_geometry_ask_who_owns_the_numbers]] — «a composição já exprime isto?» tem duas metades: a GEOMETRIA e a AUTORIA. Se o dono dos números os reescreve por quadro, a linha de painel nasce MORTA — e é isso que decide se a forma nova existe
- [[feedback_a_palette_derived_from_a_registry_offers_the_implementations_vocabulary]] — ⛔ paleta «tudo o que está registado» oferece TIPOS onde o artista escolhe INTENÇÕES (30 de 32 na física, 27 deles rows que a secção já anexa); a régua é *anexar isto sozinho muda alguma coisa?*, e o que a produziu foi o **helper por-família cujo caminho de menor esforço era uma das respostas**
- ⛔⛔⛔ **Duas ordens do dono podem CONTRARIAR-SE, e a spec tem de escolher com a medição à vista.**
  Medido 2026-09-15 (`line/UIUX`): *«Label acima do campo numérico! Muito ruim!»* põe o nome ao lado e
  entrega ao controlo **metade** da linha; *«não permita que a caixa seja redimensionada para menor
  que isso»* põe um piso de `72 px` por caixa. Numa row de `X`/`Y` à largura de omissão a coluna do
  controlo mede `128` e dois campos ao piso pedem `148` — **as duas não cabem**. ⇒ a saída não é
  encolher a coluna do nome (a granularidade dela é a SECÇÃO; por-linha devolve a coluna
  esfarrapada), é o **controlo REFLUIR**: o que não cabe ao piso desce, dentro da coluna do controlo.
  ⚠️ **E o app já lá tinha chegado, à mão, numa row só** — o comentário de um `Rect2Editor` dizia por
  escrito *«the Inspector column is too narrow for four number inputs in one row»* enquanto as outras
  dezoito rows não a conheciam. *Uma lei escrita num sítio é uma nota; só uma PORTA é uma lei.*
- ⛔⛔ **Um rótulo que descreve DUAS caixas («A / B») só funciona enquanto elas estiverem lado a
  lado.** Medido 2026-09-15: três rows da §Animation empacotavam duas propriedades num nome
  (`Frame ms / Repeat (0 = forever)`); com o nome POR CIMA o mapeamento era esquerda→direita, e ao
  pôr o nome AO LADO com as caixas a refluírem **desaparece** — um nome que descreve duas caixas
  EMPILHADAS não diz qual é qual. ⇒ *mudar a disposição de uma linha FORÇA o corte de uma linha que
  fazia duas perguntas*, e isso é ganho, não custo.
- ⛔⛔⛔ **O terceiro elo não tem instrumento em lado nenhum: o BRAÇO que recebe o pedido pode não
  chamar porta nenhuma.** Medido 2026-09-19 (`line/Vector`, os 14 verbos do osso): apagado o corpo do
  braço do *Add Smart Bone* na fase do quadro, **23 testes da shell ficaram verdes**. Havia censo de
  que a PORTA faz efeito (a família, a correr a lei) e censo de que o clique chega ao BARRAMENTO (a
  costura do painel) — *e nada juntava as duas pontas*, que é literalmente a pergunta que o
  `CLAUDE.md` §5.0 escreve (*«o leitor DECIDE, ou entrega a alguém que descarta?»*). ⚠️ A cura foi um
  censo **textual** (`include_str!` sobre as fases, com o rasto de cada verbo declarado num `match`
  exaustivo), e a limitação é declarada: as fases são métodos de `App`, que segura uma surface real,
  logo **nenhum teste as corre** — ele apanha o braço que deixou de chamar a porta, não o que a chama
  com o argumento errado; essa metade é do censo da família, que corre as duas portas e exige que
  elas **difiram**. ⭐ E o que tornou o censo possível foi mover **uma linha** da fase para a família:
  das 14 rotas, a única cujo efeito estava escrito dentro do quadro era a única que o censo não
  conseguia correr sem re-escrever a lei — *e uma régua que re-escreve a lei mede outro programa*.
- ⛔⛔ **Num censo textual, o PISO tem de ser POR SUJEITO e nunca a SOMA.** Mesma wave: quatro dos
  catorze verbos declaram **dois** rastos (a porta partilhada mais o discriminador), logo esvaziar um
  verbo inteiro ainda deixava `16 >= 14` e o censo verde. *Uma lista vazia lê-se exactamente como
  aprovada* — é a catraca sem censo de obsolescência, um nível abaixo, e só uma mutação a apanhou.

## ⛔⛔⛔ Uma lei escondida na RECEITA não é um controlo — e o dono nota (2026-09-19)

Ele pediu *«um offset do centro para os bones»*. Eu implementei-o **movendo a caixa de que a forma
é cortada** — correcto, gateado, com prova de mutação, e byte-idêntico para todas as outras formas.
O report seguinte foi: *«o osso não gira pela cabeça. vc não criou o offset o pivot. Crie no nó
Shape o offset do Pivot»*.

⚠️ **As duas frases dele não se contradizem: a segunda explica a primeira.** Uma lei que vive dentro
da receita de uma espécie é **inverificável pelo artista** — ele não tem como a ver, medir, mexer ou
desligar, logo a única evidência que lhe resta é o desenho, e um desenho sem referência não distingue
«pendurado na cabeça» de «centrado com o tamanho errado». *O que ele pediu não era o comportamento:
era a SUPERFÍCIE que torna o comportamento observável.*

**Why:** eu li «offset» como um deslocamento a implementar e ele quis dizer um **controlo**. Um
param no cartão é, ao mesmo tempo, a feature, a prova e o diagnóstico — e nenhuma das três existia.

**How to apply:** quando o dono pede um *offset*, um *pivot*, uma *escala* ou um *ângulo*, a entrega
por omissão é um **PARÂMETRO**, não uma constante interna — mesmo que a constante seja suficiente
para o caso dele. ⭐ E o valor por omissão do param pode continuar a carregar a lei que já estava
certa: aqui `0` quer dizer *«o pivô natural desta espécie»* (a cabeça, para o osso; o centro, para
todo o resto), e é por isso que o nome é **offset** — ele é um desvio, e quem declara o ponto de
partida é a forma. Irmão de [[feedback_a_refusal_only_the_terminal_sees]] (a recusa que só o
terminal vê é um botão mudo) e de [[feature_worse_than_not_existing]].

## ⛔⛔ Um ROTEIRO de smoke pode descrever a geometria ao contrário e o gate fica verde (2026-09-19)

O passo (1) da cena do osso dizia *«cada osso é LARGO do lado para onde a cadeia CRESCE»* e a
cláusula do *«deu errado»* descrevia o estado **correcto** — as duas escritas antes da volta de 180°
e não revistas com ela. O gate `o_roteiro_nomeia_o_que_a_cena_tem` ficou **verde**, porque ele conta
PALAVRAS e a prosa tinha todas.

⇒ *um roteiro é a única parte de uma cena que nenhum gate lê como afirmação*, e é a primeira que o
dono lê. **How to apply:** quando uma wave inverte, espelha ou desloca alguma coisa, o roteiro entra
na lista de sítios a corrigir **ao lado do código** — e o que É gateável ali é o NOME dos controlos:
derive-o do rótulo REGISTADO (`reg.param_ui(MANIFEST.id)`), para que renomear a linha do cartão parta
o teste em vez de deixar o roteiro a mandar procurar uma coisa que já não existe.

## ⭐⭐⭐ O gizmo que aponta um ponto desenha-se no ponto que o PRODUTO já calcula (2026-09-19)

Ordem do dono: *«permita visualizar o ponto do pivot ao arrastar os parâmetros de pivot»*. A
tentação era calcular *«onde é que o pivô caiu»* a partir do param — e isso daria um indicador que
continua CERTO no dia em que o produto deixar de estar.

⭐ **A pose de uma instância é `P + basis·(anchor + q·size)`, logo o ponto local `q = (0,0)` aterra
exactamente em `P`, em qualquer ângulo e em qualquer tamanho** — e o pivô É, por definição, o ponto
local que a receita leva à origem. ⇒ o alvo desenha-se no `P` de cada linha do sink, **sem uma
segunda cópia da aritmética**. Se ele e a forma discordarem, um dos dois está a mentir; e o que se
vê a arrastar é a FORMA a deslizar por baixo de um alvo **parado**, que é precisamente a lei.

**How to apply:** antes de escrever a conta de um indicador, pergunte que grandeza do produto já É
a resposta. *Um gizmo que recalcula o que desenha é um segundo produto a envelhecer sozinho* —
irmão de [[feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting]].

⚠️ **E o tamanho dele mede-se numa FOTO, não escolhe-se:** a 1.ª redacção usou o canto do warp
dobrado (`9 px` de raio + `4` de braço) e tapava **65 %** da peça que ele existe para apontar; a
`6 + 3` tapa `45 %` e continua maior que a alça do colisor, que é o que se sabe legível nesta casa.
*Um gizmo que aponta um ponto e esconde o que está nele responde metade da pergunta.*

⚠️ **A fronteira painel→canvas atravessa-se por um CANAL PUBLICADO, nunca por uma chamada:** o
painel escreve `set_graph_param_scrub(Some((nó, nome)))` todo quadro e quem quiser lê — o mesmo
idioma da selecção (ADR-0075). E publica-se o **NOME** do param, não a `row`: a linha é uma
coordenada do pintor, e resolvê-la do outro lado seria a segunda cópia do `band_at`.
- ⛔⛔⛔ **DUAS FILEIRAS PODEM SER O MESMO CONTROLO, e nenhuma família de gates deste repo o
  pergunta.** Medido 2026-09-19 (`line/3DModeling`, report do dono: *«se modifico qualquer cor em
  style, todas mudam ao mesmo tempo»*). O selector de cor da casa é **UM** e flutua; um painel entra
  nele **registando o `NodeId` da amostra**. O id era cunhado `(entidade, campo)` por um `match`
  cujo braço final dizia, por escrito, *«uma amostra sobre um param sem índice não existe hoje; `0`
  é a resposta estável»* — **verdade no dia em que foi escrita** e falsa quando chegou uma família
  cujo sujeito **não é uma entidade** (o estilo é da CENA e a fileira carrega `entity = 0` como
  sentinela). As cinco cores caíam no braço final e recebiam
  `hash("model3d.color.swatch.0.0")` ⇒ com o selector aberto numa, **as cinco** liam
  *«aberto em mim»* e **as cinco** pediam a escrita. ⚠️⚠️ **Os gates existentes mediam a LEI e o
  DRENO, e o defeito vive ENTRE os dois** — na IDENTIDADE com que a fileira é pintada; o gate de
  costura alimenta o dreno com a âncora já certa, logo entra **abaixo** da rotura.
  ⛔ **E a segunda metade estava na outra ponta:** a lista que fecha um selector órfão derivava o id
  por um **segundo `match`**, que só conhecia uma das três famílias — *duas respostas à mesma
  pergunta («qual é o id desta amostra?»), e elas já divergiam para outra família há uma wave*.
  ⇒ **uma PORTA com dois leitores**, espaço de nomes **próprio** para a família sem entidade (senão
  a não-colisão depende do acidente de `Entity::to_bits()` nunca valer `0`), e o braço final devolve
  **`None`** — a fileira cai para o controlo normal, *visível e diferente*, que se lê como uma falta.
  ⭐ A régua que faltava: **os ids das fileiras de cor que o produto publica são todos DISTINTOS**,
  com piso de população, mais um censo DERIVADO de que o id é cunhado **num sítio só**.
