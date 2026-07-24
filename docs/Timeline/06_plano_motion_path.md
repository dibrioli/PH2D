# Plano — motion path (interpolação espacial)

> Implementa o [ADR-0141](../architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md).
> Pesquisa: [`05_pesquisa_motion_path.md`](05_pesquisa_motion_path.md). Molde:
> [`04_plano_nesting.md`](04_plano_nesting.md).
> **Nada aqui começa antes do aceite do ADR pelo Enio.**

---

## §0 — Em uma frase

A posição ganha um **modo Path**: a trajetória é geometria editável no canvas, e a track escalar que
já existe deixa de medir *"x"* e passa a medir **"quanto do caminho"** — então graph editor, speed
graph, roving e presets vêm de graça.

## §1 — Estado de partida (o que já existe e é reaproveitado)

| Peça | Onde | O que dá de graça |
|---|---|---|
| Track escalar + `Interp` + weighted tangents | `ph2d-anim` | a curva de **timing** ao longo do caminho, **sem tipo novo** |
| **Speed graph** (W5) | `ph2d-timeline::speed` | a `Position: Velocity` da Harmony, literal |
| **Roving** | `ph2d-anim::rove` | o *Rove Across Time* do AE, no significado de origem |
| Bézier + comprimento de arco + a INVERSA | **`ph2d-arclen`** (crate-folha zero-dep, extraída da `ph2d-vec-scene` na Fatia 1) | a geometria, sem `kurbo` e sem duplicar o motor |
| Afim por posição no arco | `line/Vector` (pattern along path, integrado 23/07) | o precedente medido do mesmo problema |
| Clips, containers, stack, time remap, undo, save | `ph2d-timeline` | inalterados — continua sendo uma track |

**Não se constrói:** tipo de track 2D, segundo sampler, segundo editor de curva, cache de caminho.

---

## §2 — ⚠️ O orçamento de LOC decide a forma das fatias (medido 2026-07-23)

O cap é **700** por arquivo em `crates/` (`architecture_workspace_file_loc_cap`, ADR-0105) e a fase
precisa engordar exatamente os arquivos que estão no teto:

| Arquivo | LOC | Folga | A fase mexe? |
|---|---|---|---|
| `ph2d-timeline/src/intent_apply.rs` | **694** | **6** | sim (intents de geometria) |
| `ph2d-timeline/src/snapshot.rs` | **690** | **10** | sim (o caminho no snapshot) |
| `ph2d-timeline/src/doc.rs` | 666 | 34 | sim (a porta única da §2 do ADR) |
| `ph2d-timeline/src/apply.rs` | 650 | 50 | sim (amostragem em modo Path) |
| `ph2d-panel-timeline/src/event.rs` | 585 | 115 | sim (arrasto das alças) |

⚠️ **Logo: cada fatia ABRE com o seu split**, por responsabilidade e nunca por allowlist (o padrão
da casa). Os splits previstos: `intent_apply_path.rs` · `snapshot_path.rs` · `doc_path.rs` ·
`apply_path.rs`. ⚠️ Rode `cargo fmt` **antes** de medir — o fmt re-expande
([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).

⚠️ E o gate de LOC mora na `ph2d-editor-core`: **não roda** com `cargo test -p ph2d-timeline`. Vai
no gate batched, junto com o `file_loc_caps` da shell (HR-18, 600) — os dois já escaparam de linhas
deste repo por rodar só a varredura impactada.

---

## §3 — Fatia 0 — **a inversa de arco custa quanto?** (medição, antes de qualquer feature)

⚠️ **É a fatia que pode reescrever o desenho, e por isso vem primeiro.** O único custo novo do
ADR é: dado `s` (distância percorrida), achar o ponto — no kurbo, **iterativo**, por-binding e
por-frame.

**Duas perguntas, as duas por medição:**

1. Quanto custa `apply_from_doc` com N entidades em modo Path × as MESMAS N em modo Separate?
2. O custo por entidade é **plano em N**? (é por-binding; se crescer com N, algum passe começou a
   percorrer a lista inteira)

**Saída:** o número, e a barra do §Kill do ADR **escrita a partir dele** — nunca antes.
⚠️ Se a barra declarada (`< 2×`) falhar, ela é **substituída por uma lei medida, não afrouxada** —
foi o que o ADR-0133 fez quando o `< 2×` dele falhou. E a saída, se estourar, já tem nome: tabela de
comprimento de arco por caminho, invalidada na edição de geometria.

**Aceitação:** harness que publica a tabela `N ∈ {1, 10, 100} × {Separate, Path}`. Sem gate de barra
aqui — é **medição para decidir**; o gate nasce na Fatia 2, contra o número que esta produzir.

---

### ✅ FATIA 0 FECHADA (2026-07-23) — **Newton, e a barra reescrita**

Harness: [`crates/ph2d-timeline/tests/measure_motion_path.rs`](../../crates/ph2d-timeline/tests/measure_motion_path.rs).
Números e o raciocínio completo: **emenda ao §Kill do ADR-0141**.

**O que a medição decidiu:**

1. **A inversa que shipa hoje (bisseção de 40 iterações) custa 1700 ns/amostra** — ~1300 `sqrt`.
   **Newton custa 190 ns** com o MESMO resultado (erro 1,7e-10 num caminho de 823 unidades), porque
   `ds/dt = |B'(t)|` está disponível de graça. **9×** — e não 12×, porque a tolerância ficou em
   1e-12 (a que a bisseção entregava) e apertá-la custa ~1 iteração.
2. **O custo é plano no número de âncoras** (2 → 128): o prefixo somado + busca binária já isolam
   **um** segmento. Nenhuma estrutura nova é precisa — a `ArcPath` da linha Vector já é isso.
3. ⚠️ **A barra declarada era o instrumento errado** e foi substituída por uma LEI ligada a um
   RECURSO: *100 entidades Path ≤ 0,2 % de um frame de 60 Hz, com custo plano nas âncoras*. A razão
   contra `Track::sample` (5,7 ns) reprovava qualquer algoritmo real e não dizia de que recurso era.
4. **A LUT (7-16 ns) NÃO é comprada** — erra 0,13 unidade, e comprar aproximação que o orçamento não
   exige é escolher um modo de falha de graça. Fica documentada com o número medido.

### A CASA do motor de arco (decidida por esta fatia)

A matemática vive em [`ph2d-vec-scene/src/arclen.rs`](../../crates/ph2d-vec-scene/src/arclen.rs) —
crate do módulo **Vector** — e é **pura**: opera em `Cubic = [[f64;2];4]`, sem um tipo sequer do
documento de vetor, com `sqrt` como único transcendental (escolhido lá por determinismo
cross-plataforma). A `ArcPath` irmã já traz prefixo somado, `at(s)` devolvendo **ponto e tangente
unitária**, e `anchor_arcs()` — que é literalmente o *"valor da track por key"* da espinha do ADR.

**Decisão:** o motor **muda-se para uma crate-folha própria** (`ph2d-arclen`, zero dependências), e
a `ph2d-vec-scene` passa a **re-exportá-lo** (`pub mod arclen { pub use ph2d_arclen::*; }`) — os 5
sítios de chamada dela seguem compilando **verbatim**. Copiar seria duas respostas para *"quanto
andei nesta curva"*, que é o defeito que este repo cataloga; depender de `ph2d-vec-scene` a partir
da timeline seria arrastar o modelo de documento do Vector para dentro do runtime de animação.

⚠️ **A `ArcPath` toma `&[VecVertex]`** (acoplada ao documento). O que se move é o núcleo puro; a
`from_contour` fica na `ph2d-vec-scene` como **adaptador**, e a estrutura ganha um irmão
`from_cubics` na casa nova, que é por onde a timeline entra.

✅ **FEITO na Fatia 1** (2026-07-23): a crate `ph2d-arclen` existe, a `ph2d-vec-scene` re-exporta, a
`ph2d-timeline` depende dela, e a inversa virou Newton. A suíte do Vector ficou verde (282/282) com
**uma** exceção que teve de ser re-pinada — ver abaixo.

⚠️ **O gate `the_zigzag_is_byte_identical_across_the_arc_walker_extraction` FALHOU e foi RE-PINADO.**
Ele hasheia cada bit de cada coordenada do Zig Zag, e duas respostas certas para a mesma raiz não
caem nos mesmos bits (a bisseção para por CONTAGEM, o Newton por TOLERÂNCIA). O protocolo estava
escrito no próprio gate — *"se falhar num commit que PRETENDIA mudar o desenho, o número é que está
velho"* —, e o deslocamento foi **medido antes de re-pinar**: `1,3e-10` unidades de mundo no pior
caso, contra `~1e-2` de um pixel. O gate de MAGNITUDE que carrega essa afirmação (e guarda a
bisseção antiga como oráculo) é `the_newton_inverse_agrees_with_the_bisection_it_replaced`, na
`ph2d-arclen` — um hash diz *"diferente"*, e não distingue 1e-10 de tudo.

---

## §4 — Fatia 1 — dados + a porta única (`ph2d-timeline`, headless)

1. `PropKind::Position = 8` (append; o discriminante é wire value congelado).
2. A geometria no binding: âncoras + tangentes espaciais, **vec paralelo às keys** (precedente:
   `TrackData.roving`). `DOC_VERSION` **11 → 12**; v11 **recusado** no load, nunca migrado.
3. ⚠️ **A porta única da §2 do ADR:** mover uma âncora reescreve os comprimentos de arco de todas
   as seguintes **na MESMA operação**. Uma função, dois lados; duas funções divergem.
4. Âncora nasce **Auto Bezier** (o default do AE).

**Aceitação** (gates 3 e 6 do ADR, cada um nasce VERMELHO):
- `moving_an_anchor_rewrites_the_arclengths_in_one_operation` — mutação: atualizar só a geometria
  deixa o objeto fora do caminho
- `the_schema_is_twelve_and_a_v11_blob_is_refused`
- `separate_mode_is_byte_identical` (fingerprint — a lei #1: zero regressão)

### ✅ FATIA 1 FECHADA (2026-07-23)

**`MotionPath` mora no BINDING, não num vec paralelo ao `Track`** — refinamento do item 2 acima.
A trajetória é propriedade do *movimento deste objeto*, não da leitura que um clip faz dele: dois
clips que o animam pelo mesmo caminho são duas TEMPORIZAÇÕES da mesma viagem, que é exatamente o
que a track de arco expressa. O pareamento âncora `i` ↔ key `i` continua sendo por ÍNDICE.

**A porta única é `MotionPath::edit`** (a tabela é reconstruída antes de o chamador voltar) e há
arch-gate lendo o fonte: exatamente **um** `&mut self.anchors` no arquivo, com controle positivo.

**A tabela de arco NÃO viaja no arquivo** (`serde(from/into)` de `Vec<PathAnchor>`): número derivado
em arquivo é 2ª cópia que pode discordar, e assim o load passa pelo único construtor que a
reconstrói. Gate compara os bytes com os da lista de âncoras crua.

⚠️ **O `rest` FICA escalar**, e isto corrigiu uma recomendação anterior desta linha. Ele está nas
unidades do **TRACK**, e para Position o track mede *distância*: alargá-lo para `[f32; 2]` seria pôr
um PONTO onde o consumidor mede metros-ao-longo-do-caminho. E blendar DISTÂNCIAS mantém o objeto
**na** trajetória (meio caminho entre "3 m" e "7 m" é "5 m", ainda na curva) onde blendar os dois
PONTOS cortaria a quina. O `rest` de Position é a **projeção** da pose autorada no caminho;
**zero** seria o INÍCIO da trajetória — o "voa para a origem" que o campo existe para impedir.

⚠️ **Duas fraquezas de fixture, minhas, achadas por mutação:** o gate de âncora passava `f64` onde
uma key guarda `f32` (errava por **zero exato** — a forma de um gate que não mede nada); e o gate
e2e só amostrava em **metades e fronteiras de segmento**, onde andar pelo PARÂMETRO concorda com
andar por ARCO. Com uma amostra em 1/4 ele sangra.

## §5 — Fatia 2 — a amostragem (`ph2d-timeline`, headless)

1. `apply` em modo Path: `ponto = caminho.em_arco(track.sample(t))`, pelo relógio da ENTIDADE (time
   remap, clip, container — a cadeia inteira, como qualquer track).
2. O **gate de perf** contra o número da Fatia 0.

**Aceitação** (gates 1, 4, 5, 10):
- `a_position_binding_samples_the_path_not_two_axes` — ⚠️ o oráculo é a **distância à corda**, não
  um `assert_ne!`: "difere da reta" é satisfeito por ruído de `f64`
  ([[reference_topic_oracle_discipline]])
- `the_speed_graph_of_a_path_is_the_velocity_along_it` — a derivada da track bate a velocidade
  medida por diferenças finitas do ponto
- `roving_gives_constant_speed_along_the_path`
- perf

### ✅ FATIA 2 FECHADA (2026-07-23) — **a lei da Fatia 0, honrada e medida**

**18,77 µs/frame a 100 entidades = 0,113 % de um frame de 60 Hz** (orçamento 33,3 µs). A previsão
da Fatia 0 (19 µs, extrapolada de 190 ns por amostra) errou por menos de 1 %.

**O que o modo Path ganha de graça deixou de ser promessa:**
- `the_slope_of_the_track_is_the_speed_on_the_canvas` — pior desvio de `|dp/ds − 1|` = **5,2e-5**
  sobre uma curva com quina e ease-in-out. É o argumento inteiro do §2 do ADR: o número que o
  speed graph plota **é** a velocidade do objeto. Oráculo por diferenças finitas nos DOIS lados,
  nunca chamando `sample_speed` (uma função checada por ela mesma é espelho, não oráculo).
- `a_roving_key_gives_constant_speed_along_the_curve` — espalhamento de velocidade **9,00× → 1,00×**,
  com o controle "sem rove" no mesmo gate.

⚠️ **O gate de perf precisou de DUAS camadas, e a segunda nasceu de uma mutação que sobreviveu.**
Trocar o `partition_point` da busca de segmento por uma varredura sobre 512 âncoras move a razão de
ponta a ponta para apenas **1,77×** — sob a barra de 2,0 — porque `Track::sample` e a inversa de
Newton dominam o frame e **diluem** o defeito. O gate afiado mede `MotionPath::at` sozinha, com
contraste de 1024× (8 → 8192 âncoras): **1,04× são e 16,45× com a varredura**
([[feedback_layered_defenses_need_per_layer_gates]]).

E a razão de ponta a ponta traz **controle positivo** — `MotionPath::project`, que é honestamente
`O(âncoras)`, medida com o mesmo cronômetro na mesma fixture: **122×**. Sem ele, "a razão deu 1,0"
seria uma afirmação sobre o cronômetro.

## §6 — Fatia 3 — a trajetória na TELA (`ph2d-panel-timeline` + shell)

1. O caminho desenhado sobre o sprite no canvas, com alças de tangente **espaciais** arrastáveis.
2. ⚠️ **Os PONTOS de tempo sobre a linha** (a leitura do AE): o espaçamento entre eles **é** a
   velocidade. Uma figura, as duas informações — e é o que impede a trajetória de virar um desenho
   que não diz nada sobre timing.
3. Tokens e i18n, zero string hardcoded (HR-15); ids novos registrados no `WidgetStore`.

**Aceitação** (gate 9): **seam que CLICA** — arrastar uma alça muda a trajetória e a cena responde
([[feedback_widget_is_done_when_a_test_clicks_it]]). ⚠️ A geometria em px de TELA sob
`Affine::IDENTITY`: no Vello o transform do `stroke` **multiplica a largura**, e foi assim que o
realce do Flip virou um borrão.

## §7 — Fatia 4 — a autoria (o gesto que cria)

1. `+ Track` ganha **"Position"** ao lado de X e Y.
2. **Conversão nomeada** entre modos, que **diz o que perde** (gate 7): indo para Separate as
   tangentes espaciais são descartadas — dito antes, não descoberto depois.

## §8 — Fatia 5 — auto-orient *(a bifurcação do ADR; sai se o Enio vetar)*

1. Opt-in por binding.
2. ⚠️ **RECUSA nomeada** quando existe track de `Rotation` (dois autores de um fato).
3. ⚠️ **Velocidade zero segura o último ângulo válido** — o bug publicado do próprio AE.

**Aceitação** (gate 8): os dois, com mutação por camada
([[feedback_layered_defenses_need_per_layer_gates]]).

---

## §9 — Ordem, gates e fechamento

- **Fatia 0 é bloqueante** — ela escreve o número da barra.
- 1 → 2 são sequenciais; 3 depende de 2; 4 e 5 dependem de 3.
- ⚠️ **A cena de smoke nasce com a Fatia 3**, não no fim: um caminho é a feature mais visual deste
  módulo e não se julga por teste ([[feedback_ready_to_smoke_example]]). `PH2D_PATH_SMOKE=1` —
  um objeto que atravessa a tela num arco, com as duas alças à mão.
- Gate batched **1× no fechamento**: `nextest-impacted` + clippy `--all-targets` + LOC caps
  (workspace **e** shell) + `arch_safe_clamp_only` + auditoria ≥2 lentes com o template da DIRETIVA
  §3. ⚠️ Rodar em **debug E release** — release-only esconde pânico (a lição do Flip).
- **A linha fecha, escreve o handoff (DIRETRIZ §1.5.9) e PARA.** Não integra, não pusha.

## §10 — Fora de escopo (nomeado, com o gatilho que o acorda)

| Item | Por quê | Gatilho |
|---|---|---|
| Z / 3D | o app é 2D | — |
| Path de escala/rotação | o canal é **posição**; não existe em produto pesquisado | — |
| Trajetória como **asset compartilhado** (dois objetos no mesmo caminho) | é o *Follow Path* do Blender/Moho: outra feature, com dono e offset próprios | pedido de artista, ou o caminho virar trilho de partículas |
| Visualização read-only do modo Separate | segunda resposta para "onde este objeto passa", que ninguém pode agarrar | — |
| Motion path para vetor/painter | o binding é de sprite; os resolvers-irmãos são fase própria | a fase "animar cor/vetor" |
