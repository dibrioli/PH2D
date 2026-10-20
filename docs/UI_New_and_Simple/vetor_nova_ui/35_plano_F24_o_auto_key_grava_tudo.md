# Plano F24 — o Auto Key grava ABSOLUTAMENTE TUDO (19/10)

> Ordem do dono, 19/10 (smoke F23 OK): *«O auto key deve gravar absolutamente tudo neste app, não só
> as transformações»* — memória `project_autokey_records_everything_2026_10_19.md`. Inclui a edição
> dos PONTOS do desenho (⛔ não é cura «mover o objecto em vez dos pontos»). Handoff de origem:
> [F23a §6](../handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-19_F23a_O_PAINEL_LE_A_CURVA.md).
> A metade B do F23 (o losango no campo, plano 34 §9) é absorvida aqui: o losango passa a valer para
> todo canal, não só para os 19.

## §1 — O mecanismo de hoje (medido no código, 19/10)

| # | facto | onde |
|---|---|---|
| M1 | A timeline anima uma lista FECHADA de 19 `PropKind` (`repr(u64)`, `TranslationX=0 … MacroTime=18`); só 11 estão em `AUTOKEYED` | `ph2d-timeline/src/prop.rs:21-194`, `:238` |
| M2 | O valor de uma chave é `AnimValue` (Float · Vec2 · Vec3 · Color(Oklch) · Bool · Enum(u32)) — **contrato §6 congelado em 6 variantes** — mas a timeline só usa `Float` | `ph2d-vector-traits/src/anim_value.rs:24`, gate `anim_value_variant_count_is_capped_at_6`; `autokey_pass.rs:57-60` |
| M3 | O binding liga `AnimTarget(u64) ↔ (WireId, PropKind)`; nenhum caminho de dados (zero hits de `data_path`/`PropPath`/`ChannelKey`) | `ph2d-timeline/src/binding.rs:40-110` |
| M4 | Ler e escrever é um `match` por `PropKind` em ≥ 4 sítios exaustivos (`write_prop`, `read_prop_kind`, `from_target`, `sample_prop_value`) — cada variante nova toca ≥ 9 funções | `apply_prop.rs:23,219`; `prop.rs:260`; `timeline_bridge_keys.rs:45` |
| M5 | O Auto Key compara, por quadro, um vetor de pose de 11 posições (`PoseSample = [Option<f32>; 11]`) com a curva no instante (`curve_seen`); grava se diferem, cria canal se diferem do `baseline` | `autokey.rs:45,136-200,330` |
| M6 | A população do Auto Key = seleção do gizmo + a mão do quadro, menos quem um motor conduz | `autokey_pass.rs:149`, `timeline_bridge.rs:89` |
| M7 | O documento vetorial (`VecScene`) vive FORA do ECS; a timeline só escreve no `SimWorld` ⇒ a opacidade do vetor (F23) teve de ir por uma camada conduzida `VecDrivenStyle.alpha` | `ph2d-ecs/src/vec_driven_style.rs:58`; `ph2d-app-vec/src/driven_style.rs` |
| M8 | O vetor NÃO tem transformação por caminho: mover, girar, espelhar, alinhar **cozem nos vértices**; o vértice não tem id (seleção = `(VecPathId, índice)`) | `ph2d-vec-scene/src/lib.rs:401-505`; `path_bake_xform.rs`; `selection.rs:175` |
| M9 | Não há porta única de edição: o diff do undo vê o documento mas não diz campo nem valor, é suprimido durante o gesto e não vê timeline/Motion/Painter/física | `undo.rs:46-95,~236`; `undo_app.rs:334-416` |
| M10 | Já existe um catálogo de campos DURÁVEL para componentes: `FieldDesc { field_id: u16, kind: FieldKind (15) }` por nome canónico — mas sem leitura/escrita genérica, e sem Luz/Painter/Flip/vetor | `ph2d-component-desc/src/field.rs:16,39`; `lib.rs:54,261` |
| M11 | A linha do painel JÁ reserva a coluna do decorator (`FORM_ROWS_SHOW_DECORATOR`, `paint_decorator_dot`) e o F23 deixou `KeyMark`/`key_mark_at`/`paint_decorator_for` e `channel_mark` sem consumidor | `property_box/row.rs:129,400`; `key_mark.rs`; `ph2d-timeline/src/channel_mark.rs:31` |
| M12 | Tectos de LOC: `intent.rs` 695/700 · `autokey_pass.rs` **595/600 (shell)** · `snapshot.rs` 695 | gates `architecture_workspace_file_loc_cap` (700) e `file_loc_caps` da shell (600) |

**O defeito, numa frase:** o Auto Key só vê o que cabe num vetor de 11 números, e a timeline só sabe
escrever 19 coisas no ECS. Tudo o que o artista edita fora disso (≈ 500 propriedades, §A) não tem canal.

## §2 — Estado da arte (pesquisa + dois oráculos CORRIDOS nesta máquina)

Licenças por artefacto instalado (`pacman -Qi`): **godot 4.7.2 MIT** (porta aberta) · blender 5.2.2
GPL-2.0+ (só oráculo) · synfig 1.4.5 GPL · opentoonz 1.8.0 BSD-3 (sem modo sem janela). Scripts e
saídas (fixtures com cabeçalho, corridas DUAS vezes com o mesmo resultado) em
[`oraculos/`](oraculos/): `godot_interp.gd` → `godot_interp.saida.txt`, `blender_anim.py` →
`blender_anim.saida.txt`. Os gates de §6 que citam um número do oráculo leem-no dali.

- **Blender** — até 2.4x, blocos IPO com lista FIXA de canais por tipo de dado (adrcodes); em 2.5
  («Animato», 2009) trocou por F-Curves endereçadas por **caminho RNA em texto**: toda propriedade
  editável é animável, uma Action mistura material, geometria e transform. *Porquê:* a lista fixa
  obrigava a mexer no código a cada propriedade e prendia a Action a um tipo. **É o nosso `PropKind`.**
  Auto Key: «Only Insert Available» / «Insert Needed» / Keying Set. Grease Pencil: Auto Key cria
  chave de DESENHO. **Medido:** bool e enum seguram até a próxima chave (f6 = valor de f1); texto
  (`name`, `body`) **recusado** («not animatable»); `vertices[0].co` aceita chave por script.
- **After Effects** — cronómetro por propriedade; ligado, toda mudança num instante sem chave cria
  chave. O **Path é UMA chave com a curva inteira**; emparelha pelo «primeiro vértice». *Defeito
  documentado:* mudar o nº de vértices vale só até a chave seguinte; vértices surgem entre chaves.
- **Rive** — Animate mode: editar no palco ou no Inspector grava chave; botão de chave ao lado de
  cada propriedade; chaves de vértice, traço, trim, preenchimento, gradiente.
- **Spine** — Auto Key grava «qualquer mudança»; `K` = «Key Edited»; botão de 3 cores (verde · laranja
  = mudado sem chave · vermelho = chave). **Deform key guarda a malha INTEIRA** — *porquê, da equipa:*
  senão seria uma timeline por vértice. Cor RGBA junta (ou «Separate»).
- **Godot** (MIT) — botão de chave em cada propriedade; trilha `NodePath:propriedade` (texto).
  **Medido** (`seek`, chaves t=0/1): float, `Color`, `Vector2` lineares **por componente** (vermelho→azul
  a meio = `(0.5, 0, 0.5)`); `PackedVector2Array` do mesmo tamanho **elemento a elemento**; 3→4
  elementos: o que falta vem **do último** do menor; bool contínuo vira em t>0,5; int contínuo
  arredonda (enum 0→2 passa por 1 — errado); **String contínua sai lixo** (`\]^Z1`); Discrete segura.
  ⚠️ **Defeito medido:** depois de um seek para trás, as trilhas Discrete ficam presas no valor final
  — o Discrete do player é EVENTO ao cruzar a chave, não função do tempo.
- **Moho / Synfig** — chave por PONTO (point motion; waypoint por vértice e tangente). **Harmony** —
  troca de desenho + morph com dicas emparelhadas.
- **Tentado e ABANDONADO:** os IPO de lista fixa do Blender; o shape tween do Flash com shape hints
  («muito imprevisível» há décadas). Lição: **morph que ADIVINHA a correspondência dos pontos é
  frágil**; o que dura é a MESMA topologia em todas as chaves (Spine, Blender, AE na prática) ou a
  correspondência explícita.

**Escolhas (cada uma com o porquê):**
1. **Canal = caminho com nome**, não variante nova de enum (lição Blender 2.5, Godot). Os 19 `PropKind`
   ficam como estão, como um ramo do caminho — nada gravado parte.
2. **Discreto é função PURA do tempo e segura até a próxima chave** (lei Blender medida; o
   contra-exemplo é o seek reverso do Godot). Bool, enum, texto, escolha de fonte e de asset = `Hold`.
   Texto: o AE anima o Source Text por quadro fixo e o dono pediu TUDO ⇒ animável, sempre `Hold`
   (nunca o lerp de string do Godot).
3. **Cor interpola por componente, em sRGB não-linear** (AE, Rive, Godot medido) — o produto de
   referência do dono (Rive/AE) decide a lei. Guarda-se como grupo de 4 `Float` (`r,g,b,a`) sob UM
   canal e UM losango; ⛔ não `AnimValue::Color` (o lerp dele é OKLCH de tom estável — outra lei).
4. **Geometria = chave do desenho INTEIRO** (AE, Spine): um canal «Forma» por caminho, valor = todos
   os contornos com âncora, alças, tipo e arredondamento de cada vértice. **A topologia é IGUAL em
   todas as chaves por construção:** inserir um ponto insere-o em TODA chave desse canal (no mesmo
   parâmetro do segmento, por subdivisão de Bézier); apagar apaga o mesmo índice em todas; abrir/fechar
   vale para todas. ⇒ nunca há correspondência a adivinhar (melhor que o AE, que só propaga até a
   chave seguinte, e que o «repete o último» do Godot). *Sem* id por vértice (M8) — não é preciso.
   ⚠️ Decisão de PRODUTO pendente do dono (§9 Q1): chave da forma inteira (AE/Spine) × chave por ponto
   (Rive/Moho). A recomendação é a forma inteira; o desenho abaixo vale para as duas até ao fio do §3.4.

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | porta única |
|---|---|
| **que canais este objecto tem** (o inventário vivo) | `ChannelCatalog::channels(alvo) -> Vec<ChannelDesc { path, kind, label_key }>` — cada FAMÍLIA regista o seu leitor (`ph2d-app-vec`, `ph2d-app-light2d`, `ph2d-app-components` via `FieldDesc`, …); a shell só compõe o registo (ECS: recurso, nunca system→system) |
| **quanto vale o canal agora** (o que a tela desenha) | `ChannelCatalog::read(alvo, path) -> Option<ChannelValue>` — lê o CAMPO DO DOCUMENTO, o mesmo que o render e o painel leem |
| **escrever o valor da curva** | `ChannelCatalog::write(alvo, path, valor)` — escreve **no próprio campo do documento** (a lei do `Transform`, e a do Blender): um sink só, render/painel/save leem o mesmo número |
| **a curva no instante** | `curve_seen` generalizado a `ChannelPath` (já é a porta do diff do autokey e do `channel_mark`) |
| **o artista mudou um canal?** | o diff do autokey generalizado: para cada alvo da população × cada canal do catálogo, `read != curve_seen` ⇒ chave (armado) ou `Mudado` (desarmado, o laranja); sem curva, cria se `read != baseline` |
| **em que estado está o campo** | `channel_mark(doc, alvo, path, t, read)` → `Livre · Animado · Chave · Mudado` (o do F23, por caminho) |
| **que canal um campo do painel mostra** | a declaração do campo no painel: `(id do campo, ChannelPath)` numa tabela por painel, consumida pelo widget de linha partilhado (`property_box`) |
| **mudar a topologia de uma forma animada** | um evento `VecTopologyEdit { path, op }` emitido pela operação de vetor (inserir/apagar ponto, abrir/fechar, compor) e aplicado a TODAS as chaves do canal «Forma» por `shape_track::apply_topology` — ⛔ nunca inferido por diff |
| **inserir/apagar a chave de UM canal** | clicar o losango: `Chave` ⇒ `ChannelIntent::RemoveKeyAt`; senão ⇒ `AddKey` com `read()` no tempo da tecla K |
| **onde vive o valor por quadro durante a reprodução** | no documento (sink único) + o ledger `preview_drive` já existente para que undo/save fotografem o AUTORADO (a lei de `ph2d-preview-drive`) — **se** a medição T0 (§6) mostrar que o `Transform` animado precisa dela; a mesma lei para todos os canais |

### §3.1 Tipos (todos em `ph2d-timeline`, foundational, não congelado)

```text
ChannelPath  = Prop(PropKind)                                   // os 19 de hoje, intactos
             | Field { familia: Nome, campo: Nome, sub: Sub }    // nome canónico durável (lei: StableId/nome, nunca bits)
Sub          = Nenhum | Indice(u16)                             // ex.: paragem 2 do gradiente, entrada 1 da pilha de pinturas
ChannelValue = Escalar(AnimValue)        // Float · Vec2 · Bool · Enum — as variantes congeladas, sem tocar o contrato
             | Cor([f32; 4])             // sRGB não-linear, 4 Float sob um canal (§2 escolha 3)
             | Texto(IdDeTexto)          // tabela de textos no doc; sempre Hold
             | Forma(ShapeValue)         // contornos inteiros (§2 escolha 4); trilha própria `ShapeTrack`
ChannelKind  = Continuo | Discreto       // Discreto ⇒ Interp::Hold forçado, função pura do tempo
```

A trilha de `Forma` não é `Track<AnimValue>` (o `AnimValue` está congelado em 6): é uma `ShapeTrack {
keys: Vec<ShapeKey { t, interp, value }> }` ao lado, com o MESMO `Interp` (Hold/Linear/Eased/Bezier)
e lerp vértice a vértice (âncora, alças, arredondamento; o tipo do vértice é discreto).

### §3.2 Formato

- `TimelineDoc` `DOC_VERSION 20 → 21`: binding ganha `path: ChannelPath` (o `Prop` antigo migra
  1:1), mais `shape_tracks` e `textos`. Migração `doc_v20.rs` no idioma de `doc_v18.rs`/`doc_v19.rs`.
  O blob tem versão própria (CLAUDE.md §5.1 Timeline) ⇒ **`PROJECT_SCHEMA` não muda** — a prova é a
  recontagem `python3 scripts/schema-recount.py` igual antes e depois.
- `VecScene` (schema 23): **não muda** — a animação vive na timeline; o documento guarda o valor
  corrente, como o `Transform`.

### §3.3 O passe do Auto Key sai da shell

`autokey_pass.rs` está a 595/600 (M12) e a generalização cresce-o ⇒ o passe MUDA-SE para
`ph2d-timeline` (o núcleo puro, `autokey.rs`, já lá está) seguindo o
[HOWTO](../../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md) §2; a shell fica com a
composição (registo do catálogo + chamada em `fase_autokey`). Intents novos vão num `ChannelIntent`
em ficheiro próprio, embrulhado por UMA variante em `TimelineIntent` (o `intent.rs` está a 695/700).

### §3.4 A F23 dobra-se nesta lei

A opacidade do vetor animada pela timeline passa a escrever o campo `VecPath.opacity` (sink único),
como todo outro canal; o `VecDrivenStyle.alpha` fica SÓ para os motores de jogo (linhas/widgets que
conduzem a forma = pré-visualização, lei do `preview_drive`). Com isso `settle_to_authored` deixa de
precisar do 4.º argumento e `follow_the_hand` deixa de existir (a mão escreve o campo, o autokey vê o
campo). Duas casas para o mesmo número era o risco que o F23 já nomeou; aqui fica uma. Os 6 testes
D1–D4 de `autokey_opacidade_vetor_tests.rs` são o gate de que nada regride.

## §4 — Contrato congelado / schema — onde NÃO encosta

- **Vector §6 (`AnimValue` em 6 variantes):** usado como está; `Cor`, `Texto` e `Forma` são variantes
  de `ChannelValue` em `ph2d-timeline`, não de `AnimValue`. Prova no fecho:
  `git diff 100d34c8e0..HEAD -- crates/ph2d-vector-traits crates/ph2d-vector-doc/src/primitives.rs` → vazio.
- **Tools §6 (`Tool=12`, `PanelEvent=4`):** o evento de topologia sai das operações de `ph2d-vec-edit`
  / `ph2d-app-vec` (quem já muta o `VecScene`), não de um método novo do trait. Prova:
  `git diff … -- crates/ph2d-editor-core/src/tool.rs crates/ph2d-editor-core/tests/it/architecture_tool_contract_surface.rs` → vazio.
- **Nodes §6:** os params do Motion entram como `Field { familia: "motion", campo: <nome do param> }`
  lendo/escrevendo o `BTreeMap<String,f32>` que já existe (`ph2d-nodegraph/src/graph.rs:312`). Prova:
  `git diff … -- crates/ph2d-nodegraph` → vazio.
- **`PropKind`**: só por APPEND, e nada é apendado (os novos canais não são variantes).

## §5 — A UI: as quatro condições (independentes)

1. **Existe:** o losango (`paint_key_mark`, F23) e o decorator na coluna JÁ reservada de toda linha
   (`property_box/row.rs:129`); linhas do dope-sheet por grupo (Forma · Preenchimento · Traço ·
   Aparência · … por família, rótulos por `ph2d-i18n`).
2. **Pintado e registado:** o widget de linha partilhado pinta o decorator quando o campo declara um
   `ChannelPath`; hit-index pelo registo genérico `key_mark_at` (F23) — um só sítio para todos os
   painéis. Gate novo **`architecture_every_editable_field_is_a_channel`** (lê o FONTE, como o
   `panel_wiring_parity`): todo id de campo editável pintado num painel tem `ChannelPath` OU está na
   tabela `⛔ não-animável` com o porquê (§8). É este gate que torna «absolutamente tudo» verificável
   — um campo novo sem canal reprova.
3. **O clique chega:** press no losango → `key_mark_at` → `KeyMarkClick { path }` → shell →
   `ChannelIntent`. DoD = seam test `ph2d-ui-testkit` `MockPanelHost::click_at` (o `panel_wiring_parity`
   não vê ids derivados — F23a §5).
4. **A sequência leva a algum lugar:** a chave aparece/some no dope-sheet e o losango vira cheio/vazio
   no quadro seguinte; reproduzir mostra a forma a mudar (cor, traço, pontos) entre as chaves.

## §6 — Gates, vermelhos primeiro, e a fixture que contém o fenómeno

**T0 — medir antes de decidir (uma rodada, intercalada):** (a) o undo com um `Transform` animado: um
clique durante a reprodução e um scrub empilham passo? (b) custo do passe generalizado sobre a cena
GRANDE (100 caminhos × 1 000 vértices selecionados, 60 Hz) — o recurso é o quadro de 16,6 ms;
**kill-criterion:** se o passe passar de 1 ms depois da 2.ª tentativa, o diff passa a ser por VERSÃO do
documento (o `Arc` partilhado do undo já detecta «não mudou» por conteúdo) em vez de por valor. (c) o
render de uma forma cuja COR muda por quadro não re-tessela (só a geometria deve re-tesselar).

**Fixtures (contêm o fenómeno):** uma forma de 5 vértices NÃO alinhada aos eixos (alça assimétrica num
deles, arredondamento num canto), preenchimento em gradiente de 3 paragens, traço tracejado de 2 pares,
texto de 2 linhas, um objecto com Luz e um com componente Vida — e o relógio a 0,5 s entre chaves.

| gate (vermelho primeiro, com a cura desligada) | crate |
|---|---|
| `o_auto_key_grava_a_cor_do_preenchimento` · `…_a_largura_do_traco` · `…_o_tracejado` · `…_o_arredondamento` · `…_o_texto_em_hold` | `ph2d-timeline` (núcleo, sobre o catálogo de teste) + shell (quadro real) |
| `mover_um_ponto_armado_grava_a_forma_inteira` (ferramenta Vetor, empurrão, X/Y do painel, alinhar, espelhar — os 6 caminhos de M8, um teste cada, todos pela MESMA porta) | shell |
| `inserir_um_ponto_numa_forma_animada_insere_em_todas_as_chaves` · `apagar_…` · `fechar_…` (e a forma a meio entre chaves continua sem torção: os pontos medidos contra a subdivisão de Bézier) | `ph2d-timeline` `shape_track` |
| `o_discreto_e_funcao_do_tempo` (seek para trás devolve o valor da chave anterior — o defeito do Godot) | `ph2d-timeline` |
| `a_cor_a_meio_e_por_componente_srgb` (vermelho→azul a 0,5 = `(0.5, 0, 0.5)`, o número do Godot) | `ph2d-timeline` |
| `parar_fora_de_uma_chave_nao_grava` (o F23 D2, para TODO canal) · `uma_edicao_desarmada_fica_ate_o_tempo_mudar` | shell |
| `o_doc_v20_migra_para_v21` (fixture binária v20 gravada antes da mudança, com cabeçalho) | `ph2d-timeline` |
| `architecture_every_editable_field_is_a_channel` (§5.2) | editor-core / shell |
| seam: o losango pinta só com canal, pinta cheio em `Chave`, o clique emite e a chave aparece | painéis (`ph2d-ui-testkit`) |
| `autokey_opacidade_vetor_tests` D1–D4 continuam verdes depois da §3.4 | shell |
| Mutação no fecho: `read` a devolver a curva · `write` no-op · topologia aplicada só à 1.ª chave · Hold desligado · cor por OKLCH · catálogo sem a família vetor | — |

## §7 — A cena de smoke (`PH2D_AUTOKEY_SMOKE=3`, e `=4` mexe sozinha)

Uma forma vetorial de 5 pontos (estrela torta, gradiente laranja→roxo, traço tracejado azul), Auto Key
ligado, relógio no quadro 12. O `=4` faz sozinho, no quadro 12: troca o preenchimento para verde,
engrossa o traço de 4 para 12, arredonda um canto, arrasta um ponto 80 px; no quadro 24 volta o traço a
4 e muda o texto ao lado. Fotos `fotos/F24_cena_autokey_{0,12,18,24}.png` e a do dope-sheet; os
números (nº de chaves por canal, cor e largura a meio no quadro 18) **medidos pela sonda
`PH2D_AUTOKEY_LOG` antes de escrever a mensagem ao dono**. Errado = a cor ou o traço saltarem em vez
de passar suavemente, a estrela torcer a meio, ou faltar uma linha no dope-sheet.

## §8 — Fora, com o porquê (a tabela `⛔ não-animável` do gate §5.2 nasce daqui)

| o quê | porquê |
|---|---|
| Nome, tags, script, ligações por nome | é a IDENTIDADE que o canal usa para achar o alvo (o Blender recusa `name`, medido) |
| Criar/apagar objectos, componentes, booleanas, pais | estrutura da cena, não propriedade; a booleana grava o RESULTADO (a forma) pelo canal «Forma» |
| Pincel, ferramenta, grelha, guias, cebola, câmara do editor | estado da ferramenta/vista, não da cena |
| Quadro (MiroClone), editor de scripts, macros (já tem `MacroTime`) | sem cena animada |
| Pixels do Painter | **depende do dono** (§9 Q2); as propriedades da camada (opacidade, mistura, visível, efeitos) entram de qualquer forma |
| Traços do Flip | já têm a lei do Grease Pencil (o Auto Key cria DESENHO, `ph2d-flip/src/autokey.rs`); as propriedades da camada Flip entram |

## §9 — Perguntas de PRODUTO ao dono (o resto é técnico e está decidido acima)

- **Q1 — pontos:** mexer um ponto grava a **forma inteira** numa chave (After Effects, Spine:
  recomendado — nunca torce, uma linha por forma no dope-sheet) **ou** cada ponto tem a sua chave
  (Rive, Moho: dá para atrasar um ponto em relação a outro, mas inserir/apagar pontos fica frágil)?
- **Q2 — pintura:** a pintura do Painter também passa a ser quadro a quadro (como o Flip), ou ficam só
  as propriedades da camada?

## §10 — Ordem (dois blocos; série porque o 2.º depende do trait que o 1.º fecha)

1. **Bloco 1 — o modelo, provado na família mais difícil:** T0 · tipos + formato v21 + migração · o
   catálogo + `read/write` · passe do autokey generalizado e fora da shell · `ShapeTrack` + topologia ·
   a família VETOR inteira (§A.1, as 9 secções) · §3.4 · o losango e o decorator partilhado · o gate
   §5.2 (com a tabela de dívida a zerar no bloco 2) · cena 39/`AUTOKEY_SMOKE=3|4` · smoke do dono.
2. **Bloco 2 — todas as outras famílias num lote** (§A.2–A.9, catálogos mecânicos, em paralelo por
   família): Inspector/Transform/sprite/ordem/visibilidade · Esqueleto · Luz · Física · Componentes
   (todos os `*FieldEdit`, via `FieldDesc`) · camadas e ajustes do Painter · camada do Flip · params do
   Motion. Uma rodada de medição, um gate batched, a dívida do gate §5.2 a zero.

## §A — Inventário das propriedades editáveis (explorador, 19/10; file:line nos relatórios)

Contagem: **vetor ≈ 60** · Inspector/sprite/ordem ≈ 40 · Esqueleto ≈ 20 · Luz 6 · Física ≈ 27 ·
Componentes ≈ 250 (Vida 40, Jogador 30, Navegação 24, Câmara 15, Áudio 11, HUD 6, + partículas,
tween, arma, top-down, projéctil, parallax, raio, gatilho, sequência…) · Painter camada 8 + ajustes
(dezenas, um struct por tipo) · Flip camada 7 · Motion: todo `ParamSpec` (`f32` por nome). **≈ 500.**

**§A.1 Vetor** (`ph2d-vec-scene` salvo indicação) — tipo do canal entre parênteses:
- *Forma* (`Forma`): âncora/alças `VecVertex` `lib.rs:401-405` · tipo do vértice `lib.rs:385` (discreto
  dentro da forma) · arredondamento por vértice `lib.rs:428` · fechado `lib.rs:476` · subcaminhos
  `lib.rs:482`. Escritores: `translate_path` `path_ops.rs:548` · `translate_path_world` `:477` ·
  `shift_vert`/`nudge` `ph2d-vec-edit/src/selection.rs:24,229` · `apply_vec_transform`/`rotate_by`/
  `flip`/`rotate`/`align`/`distribute` (`despacho_vetor_*`) · `set_corner_size`/`set_chamfer`
  `corner_live.rs:74,79` · `delete_selected_vertex` `selection.rs:594` (topologia).
- *Regra de preenchimento* (Enum) `lib.rs:485`.
- *Preenchimento*: tipo sólido/linear/radial/padrão (Enum) · cor sólida (Cor) `paint.rs:91` · paragens
  offset (Float) + cor (Cor) `paint.rs:32` · pontos do gradiente pos (Vec2) / influência / jitter
  (Float) `paint.rs:52-57` · ângulo (início/fim Vec2) `paint.rs:95` · padrão: tamanho, folga, origem,
  ângulo, alfa `paint.rs:198-228`.
- *Traço*: cor (Cor) `stroke_style.rs:243` · largura (Float) `:244` · perfil de largura (paragens) ·
  tracejado pares (Float ×2 por par) `dash_pattern.rs:25` · cap/join/alinhamento (Enum) `:12,21,74` ·
  pontas início/fim (Enum) + escala (Float) + arredondar (Bool) `:261-278`.
- *Aparência*: opacidade (Float) `lib.rs:505` · mistura (Enum) `lib.rs:514` · pilha de pinturas por
  entrada: ligada (Bool), opacidade (Float), mistura (Enum), largura (Float) `paint_stack.rs:98-276`.
- *Efeitos*: por `FxEntry`, cada `FxParam` (Float/Bool/Int) `effect.rs:54,148,177`.
- *Formas paramétricas*: `VecShape::Param{kind,w,h,values[8]}` `ph2d-ecs/src/vec_shape.rs:71` —
  lados/pontas (Enum-inteiro, Hold), raios, raio interno, torção, curva (Float).
- *Texto* (`VecShape::Text`, ECS): conteúdo (Texto, Hold) · família (Texto, Hold) · tamanho, peso,
  entrelinha, tracking, largura de quebra (Float) · alinhamento (Enum) · eixos variáveis (Float por
  eixo) `vec_shape.rs:29-40`; cor/contorno do texto (como Preenchimento/Traço).
- *Ligações de estilo* `BoundProp` `vec_bindings.rs:80`: são CONDUÇÃO por token (motor), não canal.

**§A.2 Inspector genérico:** Transform posição/rotação/escala/skew `ph2d-ecs/src/transform.rs:65` (os
`PropKind` de hoje + skew novo) · visível (Bool) `visibility.rs:27` · sprite: tint, self-tint, cantos
(Cor), tint-fill/flip/centrado/região-ligada (Bool), opacidade, offset, região (Float), hframes/vframes/
frame (Enum-inteiro) `inspector_model.rs:182-227` · emissão (Float) `emissive.rs:64` · ordem:
camada, ordem na camada, z (Enum-inteiro), y-sort (Bool+Vec2), clip-children (Enum+Float)
`sorting.rs:31-176`, `masking.rs:33`.
**§A.3 Esqueleto:** osso comprimento, força, segmentos, alças (Float) `ph2d-skeleton-ecs/src/lib.rs:92-136`
(alças já são `PropKind`) · limites min/max `:256,263` · IK mix, suavidade, cadeia, offset, lado
`:402-464` · osso inteligente de/para `:343-349` · pele lei (Enum) `skin_bind.rs:36`.
**§A.4 Luz:** cor (Cor, linear no componente — o canal guarda sRGB e converte no `write`), intensidade,
tamanho, alcance (Float), tipo (Enum) `ph2d-light2d-ecs/src/componentes.rs:13-21` · bloqueia-luz
opacidade `:76`.
**§A.5 Física:** `PhysicsFieldEdit` ≈ 27 (tipo, forma, raios, densidade, restituição, atrito, gravidade,
velocidades, massa, camada, sensor, CCD, travas, dominância) `inspector_model_physics.rs:339`.
**§A.6 Componentes:** todo `*FieldEdit` de `ph2d-editor-core` (Vida `vida_edits.rs:230`, Câmara
`inspector_model_camera.rs:95`, Navegação `nav_edits.rs:275`, HUD `hud_edits.rs:59`, Áudio
`inspector_model_audio.rs:97`, Jogador `inspector_model_player.rs:285`, Partículas, Tween, Arma,
Top-down, Projéctil, Parallax, …); textos e nomes de evento = Texto/Hold; endereço = nome canónico +
`field_id` do `FieldDesc` (M10).
**§A.7 Painter:** camada opacidade, mistura, visível, profundidade do impasto `layers/mod.rs:120`,
`layer_edit.rs:14` · camada de ajuste opacidade, mistura, visível + params por tipo
`ph2d-painter-effects/src/adjustments/mod.rs:512-546`.
**§A.8 Flip:** camada opacidade, mistura, visível `ph2d-flip/src/layer.rs:37`.
**§A.9 Motion:** todo param de cartão (`ParamSpec`, `f32` por nome) `ph2d-nodegraph/src/node.rs:52`,
`graph.rs:312`; params de texto (`set_text_param` `:339`) = Texto/Hold.
