# HANDOFF DE INTEGRAÇÃO — `line/Vector`, o painel autorado fica VIVO (2026-08-09)

> **9 commits · 60 arquivos · +4.046/−583 · todos os smokes aprovados pelo Enio.**
> Supersede nada: é a continuação da jornada de UI/UX que integrou em 08/08.

## §1 — O que esta linha entrega

A W8b tinha entregue *a árvore autorada vira painel*. Faltavam três coisas, e as três
fecharam:

1. **Os tipos que faltavam vestem** — `NumberInput`, `LevelMeter`, `ColorSwatch` e
   `IconButton` (catálogo **12 → 16**).
2. **O parâmetro por-tipo** — a `SkinParam`, o canal que leva à pele o que nem o
   retângulo, nem o rótulo, nem os tokens determinam: a **cor** de uma swatch e o
   **glifo** de um botão de ícone.
3. **O painel deixa de ser um SNAPSHOT** — ele segue o documento aberto, quadro a quadro.

## §2 — As leis, e onde cada uma mora

### O parâmetro por-tipo é um CAMPO com neutro, nunca um argumento por tipo

`SkinParam { rgba, icon }` — o molde do `KernelResolver` dos Motion Nodes. Um tipo que
não consome um campo não sabe que ele existe, e `SkinParam::default()` é o mundo
pré-wave **byte a byte**.

⚠️ **A premissa que estava errada e foi corrigida por escrito:** nada nesse molde exige
que o campo seja *pequeno*. Foi essa suposição que me fez escrever, no levantamento, que
o ícone *"não entra pelo mesmo canal"* — e o `IconGlyph` já codificava exatamente as duas
respostas possíveis para *qual ícone?*.

### A precedência do ícone é UMA porta

`ph2d_editor_core::widget::icon_glyph(chosen, drawn)` — **a escolha vence, e a ausência
dela É o desenho**. Não há terceiro estado, então as duas rotas nunca podem discordar; um
slug que este build não conhece degrada para o desenho. **Três** consumidores a
perguntam (a ponte do canvas, o plano do painel gerado, o painel compilado), e três
cópias divergiriam com modos de falha diferentes.

### O SLUG é a chave durável, nunca o discriminante

O discriminante de `IconId` é a posição **alfabética do arquivo SVG** (`enum_order_matches_svgs`
o pina), então acrescentar um ícone desloca todos os posteriores. Um documento que
guardasse o número passaria a desenhar outro glifo, em silêncio, num dia em que ninguém
tocou no documento.

⚠️ **E isto achou um defeito VIVO no `main`:** `IconId::Color` desenhava os sliders e
`IconId::ColorEqualization` a paleta — a ordem de declaração estava trocada contra a
alfabética, e o `enum_order_matches_svgs` só afirmava `*id as usize == i`, trivialmente
verdadeiro para qualquer lista na ordem em que foi escrita. Consequência de produto: o
botão *"Add adjustment"* do painel de camadas do Painter mostrava a paleta. Corrigido, com
gate novo que compara o `Debug` contra o PascalCase do slug.

### O glifo INVERTE Y, porque os dois espaços discordam

O documento é **Y para cima** (a câmera inverte: `scale_non_uniform(k, -k)`); a caixa de
24×24 do ícone é **Y para baixo**, porque é a viewbox do SVG. `icon_face` é o único ponto
em que uma geometria de documento entra naquela caixa ⇒ a conversão pertence ali, e é por
isso que **uma correção consertou as duas metades**.

⚠️ **Por que shipou:** os cinco gates existentes mediam a **caixa** (extensão, centragem,
razão de aspecto, degeneração), e **uma caixa é simétrica sob inversão**. O gate novo
pergunta qual **vértice** está no topo de cada lado.

### O documento aberto vence o código colado

`rows::with_rows` — a precedência num lugar só, perguntada por quatro consumidores.
⚠️ **Isto NÃO remove o recompilar para SHIPAR:** o gerado continua sendo o artefato
compilado, que é o que passa pelos gates de parity/a11y/LOC. O que saiu foi o ciclo de
compilação de dentro do laço de **autoria**. Vivo para autorar, compilado para entregar.

- **Uma varredura, duas representações** (`ui_panel_spec::Authored`): o vivo quer a
  **curva**, o gerado quer **texto** (um `const` não constrói um `BezPath`). Só o lado do
  código faz `to_svg`.
- **`thread_local`, não um lock:** publicar e pintar são os dois na thread da UI, e estado
  global entre testes paralelos é a flake que a `ph2d-painter-brush` pagou.
- **`Option` e não `Vec` vazio:** *não há documento* e *documento sem controles* são
  coisas diferentes.
- Publica **só com o painel visível** — a lei do ADR-0125.

### O botão autorado vira SINAL

O painel autorado era **o único do app sem ponte**. O vazamento (fila só empurrada,
crescendo sem teto) era a metade menor; a maior é que **um aperto não tem valor no
store**, então o intent era o único canal que o carregava — e **todo botão autorado era um
controle morto**.

`SignalOrigin::Control` é a terceira origem da saída do R0, onde markers da timeline e
contatos da física já publicam. ⚠️ **Só o `Fired` vira sinal** — um slider já diz o que
vale pelo store, e publicá-lo aqui também poria o mesmo fato em dois fios.

⚠️ **A POSIÇÃO é load-bearing:** a ponte publica **depois** de o quadro virar e **antes**
do dreno. Fora dessa janela o aperto chega um quadro atrasado — invisível num toast e
visível no dia em que o consumidor for som.

## §3 — Colisão: o que esta linha move

| Eixo | Estado |
|---|---|
| `PROJECT_SCHEMA` | **69, INTOCADO** (`git diff main -- project.rs` vazio) |
| `VEC_SCENE_SCHEMA_VERSION` | **14, intocado** |
| Contrato congelado (§6) | **intacto** (`ph2d-nodegraph`, `ph2d-core/src/tool.rs` com diff vazio) |
| Registro do `ph2d-ecs` | **intocado** — `VecWidgetIcon` já existia |
| ADR | **nenhum** ⇒ fora de toda disputa de número |
| `Cargo.toml` | **1**, e é aresta interna (`ph2d-panel-authored`) |
| Dep externa nova | **nenhuma** (`Cargo.lock` sem `+name`) |
| `WidgetKind` | 12 → **16** (variantes apendadas; o `code()` é estável) |

## §4 — Smokes

**`env PH2D_BUILD_SMOKE=62 cargo run -p ph2d-host-desktop --release`** — a cena imprime
o que montou; ⚠️ **se a linha `[ui-panel] … 7 row(s)` não aparecer, PARE.**

1. A swatch **Tint** mostra o preenchimento *dela*, não o dos irmãos.
2. **Play** desenha a ESTRELA que o artista fez — **em pé**, e igual no canvas e no painel.
3. **Trash** mostra o lixo do catálogo, porque foi ESCOLHIDO; o picker troca os dois juntos.
4. O cabeçalho **Appearance** dobra e o painel **encolhe**.
5. **Renomeie um filho ou edite os nós da estrela: o painel muda enquanto você desenha.**
6. Com `PH2D_SIGNAL_LOG=1`, clicar um botão dá toast + `[signal] … <- controle autorado`.

## §5 — Aberto, com o preço ao lado

- **A família de LISTA** (`Tabs`/`Dropdown`/`RadioGroup`/`SegmentedAdaptive`) é **14,6% da
  UI real** e o maior buraco estrutural do levantamento. ⚠️ **Não é fiação:** são quatro
  **tipos novos** (variante + código estável + pele + o canal de OPÇÕES + codegen + caminho
  vivo + golden). O desenho que a análise recomenda: as opções são **filhos autorados**, e
  a lei que falta é *um controle que toma opções POSSUI os seus filhos* — o `walk` não
  pode descê-los como rows. É uma wave inteira.
- **Duas molduras autoradas:** o painel vivo mostra a **primeira**. Escolher pela SELEÇÃO
  é decisão de produto — inventar um desempate que o artista não vê seria pior.
- **`AuthoredIntent::Value`/`Flag`/`Text`** são drenados e descartados (o store é a
  autoridade). Se um dia quiserem consumidor, ele entra na mesma ponte.

## §6 — Para o integrador

- ⚠️ **Rode `--test the_two_halves_read_the_glyph_through_one_door` e CONTE 3 testes.**
  Este arquivo já perdeu gates por edição em massa nesta casa.
- ⚠️ **O golden do painel** (`generated/panel.rs`) é derivado da cena do smoke. Se um
  merge mover `icon_face` ou a varredura, **re-gere** com
  `cargo test -p ph2d-host-desktop --bins print_the_generated_panel -- --ignored --nocapture`
  e confira que o diff é vazio — foi assim que o glifo invertido foi pego.
- O arch-gate da porta única ancora no **nome** `icon_face`, não em `icon_face(`: ele já
  ficou vermelho sobre produto correto quando a metade do spec passou a **passar** a função
  em vez de a chamar.

## §7 — Gate de fechamento

`scripts/nextest-impacted.sh` **8496/8496 verdes** · `cargo clippy --workspace
--all-targets` **limpo** · `cargo fmt --all` aplicado · golden do painel **em dia**.
