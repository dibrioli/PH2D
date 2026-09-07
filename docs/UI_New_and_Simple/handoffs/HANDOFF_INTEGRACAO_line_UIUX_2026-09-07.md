# HANDOFF DE INTEGRAÇÃO — `line/UIUX` — 2026-09-07

> **O ritmo, a cor e o gesto da coluna.** Vinte e uma waves (w13–w33) sobre o redesenho plano que a
> jornada anterior entregou: as **portas do ritmo** (vão de controlo, vão de secção, recuo de filho,
> vão ícone→rótulo), a **fusão dos apelidos de cor**, o **chão da janela** que faz de cada área um
> cartão, e o **gesto que fecha uma coluna** — que precisou de três reports do dono até ficar certo.
> ⚠️ Este documento é para o **agente integrador**. O mecanismo de cada wave vive nas mensagens de
> commit (densas, uma por wave) e em [`pesquisa/08`](../pesquisa/08_modelos_com_codigo_para_seguir.md);
> aqui está só o que evita conflito e regressão.

---

## 1 — Identidade

| | |
|---|---|
| branch | `line/UIUX` |
| HEAD | `b3034ae75` |
| merge-base com `main` | `815555aed` |
| commits | **25** (w13 … w33) |
| ficheiros tocados | **203** |
| smoke do dono | ✅ **aprovado em 2026-09-07** (a w33; as anteriores foram aprovadas uma a uma) |

⚠️ **A tabela do §3 é REFERÊNCIA, não EVIDÊNCIA** — ela mede contra o `main` de **2026-09-07**.
Re-rode `collision-surface.sh` nesta worktree imediatamente antes de fundir; a divergência entre as
duas leituras é ela própria um achado.

---

## 2 — Foundational / partilhado tocado, e porquê

| onde | o que mudou | aditivo? |
|---|---|---|
| `docs/design/tokens.json` | **−57 valores escritos à mão** (os apelidos de timeline/atributo passam a resolver pelo PAI) · **+`window-ground`** nos temas modernos | ⛔ **não** — ver §6 |
| `crates/ph2d-tokens/src/spacing.rs` | **+273 linhas**: sete portas novas de ritmo (`control_gap_px`, `section_gap_px`, `list_row_gap_px`, `tree_chevron_col_px`, `list_indent_px`, `icon_label_gap_px`, `area_gap_px`) | aditivo, **menos** o §6 |
| `crates/ph2d-tokens/src/color_alias.rs` | **ficheiro novo** — a lei do apelido, extraída para o `color.rs` caber no teto de LOC | aditivo |
| `crates/ph2d-tokens/src/color.rs` | `ColorToken::WindowGround` + `alias_parent()` a curto-circuitar antes da tabela | aditivo |
| `crates/ph2d-tokens/src/derive.rs` | `Roles.ground` (`GROUND_STEP = 0.04`, absoluto) | aditivo |
| `crates/ph2d-tokens/src/chrome.rs` | ⛔ **`HIER_ROW_H_PX` APAGADO** (e `chrome.hier-row-h` saiu do JSON) | ⛔ **não** |
| `crates/ph2d-editor-core` | **83 ficheiros.** Novos: `screens/hero/dock_columns.rs`, `screens/hero/dock_reopen.rs`, `widget/list_rows/{mod,selection}.rs`, `interaction/dispatch/tests/caret_doors.rs`. O resto são pintores a passar pelas portas novas | aditivo + conversões |
| `shells/desktop` | `dock_resize.rs` reescrito (o gesto delega; a lei mudou de crate) + o gate dele | conversão |
| 24 crates de painel | conversões para as portas do ritmo | conversões |

⚠️ **`ph2d-panel-painter-layers` leva 30 ficheiros** — é a maior concentração fora do `editor-core`,
e é a wave 13 (o cartão daquele painel estava a ser engolido por outro do mesmo tom).

---

## 3 — Superfície de colisão (saída da sonda, colada)

```
SUPERFÍCIE DE COLISÃO — line/UIUX contra main
  merge-base 815555aed   ·   25 commit(s)   ·   202 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                121   (base: 121)
      └ tripla do gate       (121, 13, 22)   (base: (121, 13, 22))
    VEC_SCENE_SCHEMA                —   (base: —)
    FLIP_SCHEMA                    13   (base: 13)
    DOC_VERSION (timeline)         18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)          82   (base: 82)
    ph2d-script (espelho)          82   (base: 82)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs       intocado
    crates/ph2d-editor-core/src/tool.rs     intocado
▸ ADR
    último no disco: 0168   próximo livre: 0169
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock
    nenhum '+name' novo
▸ MARCADORES DE CONFLITO
    nenhum nos arquivos da linha
▸ TETOS DE LOC
    nenhum arquivo da linha passa do teto
```

⭐ **Zero schema, zero registo de componente, zero contrato congelado, zero ADR, zero dependência
nova.** A superfície desta linha é **numérica de UI e de PINTURA**, como a anterior.

### Símbolos NOVOS que outra linha pode ter criado com o mesmo nome

| símbolo | onde | risco |
|---|---|---|
| `ColorToken::WindowGround` | `ph2d-tokens/src/color.rs` | ⚠️ variante de enum — duas linhas a acrescentar variantes ao mesmo enum fundem **limpo** e uma delas fica sem braço no `match` |
| `area_gap_px` · `control_gap_px` · `section_gap_px` · `list_row_gap_px` · `tree_chevron_col_px` · `list_indent_px` · `icon_label_gap_px` | `ph2d-tokens/src/spacing.rs` | baixo (nomes específicos) |
| `screens::hero::dock_columns` (módulo) | `ph2d-editor-core` | baixo |
| `slot_tabs::tab_layout` | `ph2d-editor-core` | baixo |
| `HeroScreen::dock_closed` (campo) | `ph2d-editor-core/src/screens/hero.rs` | ⚠️ **campo novo numa struct que muitas linhas tocam** — o `HeroScreen::new` ganha uma linha; espere conflito textual trivial ali |
| `SeamDrag` (struct) | `shells/desktop/src/dock_resize.rs` | baixo — o `DockSeamDrag` deixou de ser `Option<DockSide>` |

---

## 4 — Contratos congelados (§6)

**Intocados, e a sonda confirma nos dois sítios.** `NodeOp`/`OpResolver`/`NodeManifest` e
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` não aparecem no diff.
⇒ **esta linha não exige ADR.**

---

## 5 — O que só o `ship.sh` apanha (o gate de integração NÃO roda)

- `typos` — os docs desta linha estão em português e usam «vão», «quina», «alça»; o dicionário do
  repo já os conhece, mas o handoff é ficheiro novo.
- `doc-index.sh --check` — **este ficheiro é novo**, logo o
  [`handoffs/README.md`](README.md) tem de ser regenerado (feito nesta linha; re-rode se a fusão
  trouxer outro handoff no mesmo directório).
- `machete` / `deny` / `audit` — sem dependência nova, esperados verdes.

---

## 6 — ⚠️⚠️ O QUE VAI PARTIR NA FUSÃO, e é previsível

### 6.1 — ⛔⛔ Duas constantes públicas de `ph2d-tokens` DEIXARAM DE EXISTIR

| símbolo | estado | usos no `main` | cura |
|---|---|---|---|
| `HIER_ROW_H_PX` | **apagado** (o token `chrome.hier-row-h` saiu do JSON) | 6 ficheiros | a linha da hierarquia deixou de ter altura própria: ela usa o `ROW_H_PX` como todas as outras (w17, *«uma LISTA não é um formulário»*) |
| `SECTION_GAP_PX` | **const → `fn section_gap_px()`** | 9 ficheiros | trocar a leitura da constante pela chamada da porta |

⚠️ **Todos os usos DENTRO desta linha já foram convertidos.** O que parte é o que **outra linha**
tiver acrescentado depois do merge-base: será **erro de compilação alto**, não silêncio.
⛔ **A cura é converter, nunca repor a constante** — uma altura de linha própria para a hierarquia é
exactamente o que o dono mandou tirar, e um `SECTION_GAP_PX` ressuscitado passa a ser a segunda
resposta à pergunta que a porta responde.

### 6.2 — ⛔ 57 valores de cor saíram do `tokens.json`

Os apelidos (`timeline-*`, `attr-write`, `grid-axis`…) deixaram de ter valor escrito e passam a
resolver pelo **pai** (`ColorToken::alias_parent`). ⚠️ **Uma linha que tenha EDITADO um daqueles
valores no JSON vai fundir limpo e a edição dela evapora** — o valor deixou de ser lido.
⇒ na fusão, `git log main..<outra-linha> -- docs/design/tokens.json` **antes** de aceitar o hunk.
⚠️ E o gate antigo `the_sixteen_timeline_slots_are_pure_aliases.rs` foi **apagado** e substituído
por `an_alias_has_no_value_it_has_a_parent.rs`: se outra linha o editou, o merge acusa
*modify/delete* — a resolução é **manter a deleção** e levar a asserção nova para o gate novo.

### 6.3 — ⚠️ A pintura da janela mudou de FORMA

O chão (`paint_window_ground`) é a **janela MENOS a área de desenho**, recortada com `Fill::EvenOdd`.
⛔ Ele **não** é um fill por baixo da área — isso taparia o desenho, porque em modo vivo o compositor
mostra o `game_rt` onde o vello tem `α = 0`. Um golden de UI de outra linha que cubra a moldura da
janela **muda de valor**, e a mudança é a cura.

### 6.4 — ⚠️ Gates de geometria de painel com MAIS DE UM painel por coluna

A w33 introduziu o primeiro gate do repo que abre 3–5 painéis no mesmo encaixe
(`a_column_gives_back_exactly_what_it_took.rs`). Se outra linha registou um painel novo com
`DEFAULT_SLOT = RightTop`, **as contagens daquele ficheiro mexem-se** — e elas são derivadas do
registo, de propósito, com controlos de vacuidade que falham ALTO se a fixtura deixar de produzir o
fenómeno. ⛔ Não reescreva um número lá: leia a mensagem, que diz qual controlo caiu.

---

## 7 — Ordem, dependências e o que smokar

**Ordem sugerida:** esta linha **depois** de qualquer linha que mexa em `ph2d-tokens`, porque o
`spacing.rs` daqui é o ficheiro com mais linhas novas e a resolução é mais barata a partir dele.
⚠️ Se outra linha tocar o `tokens.json`, resolva **o JSON primeiro** e só depois as crates (§6.2).

**Smoke mínimo depois de fundir**, em ordem de valor:

1. `cargo run -p ph2d-host-desktop --release` — o app abre com a UI nova (é o caminho de omissão).
2. **A coluna:** alargar a coluna da direita, arrastar a borda para dentro até fechar, puxar a tira
   de volta. Tem de voltar **só o Inspector**, na largura larga. ⛔ Se aparecer qualquer janela
   sobre o desenho (*Widget Lab*, *Assets*, *Color*), a fusão perdeu a w33.
3. **O chão:** um fio escuro de 4 px entre a Hierarquia e o desenho, e entre o desenho e a linha do
   tempo — **mais escuro** que os dois lados, com a área de desenho a mostrar quina.
4. `PH2D_UI_NEW=0` — o clássico tem de ficar **byte a byte** como antes.
5. **Recuperação:** *View → Reset Panel Layout* existe e repõe as três coisas.

⚠️ **A arrumação vive fora do repo** (`~/.ph2d/layout.txt`) e a visibilidade de painel é gravada lá
como XOR contra o `DEFAULT_VISIBLE`. Um ficheiro velho de uma máquina de teste pode abrir o app com
o Inspector fechado — **apagá-lo é o reset**, e não é sintoma de regressão.

---

## 8 — ⏳ ABERTO (não corrigir na integração; é decisão do dono ou wave própria)

1. ⭐⭐ **«Belas abas de painéis»** — pedido do dono de 2026-09-07, **medido e não construído**. A
   auditoria de cinco lentes deste dia produziu a tabela contra o Godot 4.6 «Modern»: falta
   **largura por conteúdo** (hoje todas iguais ⇒ «Audio Editor» e «Audio Mixer» elidem as duas para
   «Audio …» e ficam indistinguíveis abaixo de ~220 px de coluna), **quina só nos dois cantos de
   cima** (hoje é um botão a flutuar, não uma aba), **fundo da aba inactiva** (hoje `None`), **ícone**
   (o `PanelManifest` não tem campo; há 137 SVG em `docs/design/icons/` e a porta `paint_icon`), e
   **afordância de transbordo** (o `tool_bar::bar_split` desta casa já tem o `⋯`; esta fila não).
2. ⚠️ **Incoerência interna nomeada:** as abas de ENCAIXE (`slot_tabs`) marcam a activa com
   `BgElev`/`Text1`; as abas de LAYOUT (`layout_tabs`) com `AccentSoft`/`Accent`. Uma das duas está
   errada e **nada no repo escolhe qual** — é decisão de produto.
3. ⚠️ **O alvo de toque de uma aba tem 22 px de altura**, num app cuja memória declara *«⛔ ALVO É
   TABLET»* (44 pt Apple / 48 dp Android). Número medido, cura não desenhada.
4. ⏳ **Degrau G — esvaziar os painéis:** 1 de 25 painéis censuado. É o maior trabalho aberto da
   linha e precisa de decisão do dono painel a painel.
5. ⏳ O `no_magic_numeric` tem ponto cego na raiz de `ph2d-editor-core/src` (759 sítios, ~570 deles
   fixturas de teste).
6. ⏳ `PanelLayout` continua sem leitor de produção.
7. ⚠️ Promover `ph2d-physics-ecs::the_cost_of_a_player_is_linear_in_their_number` para a família de
   flakes de recurso do `CLAUDE.md` §5.0 — **é trabalho da integração**, não desta linha.

---

## 9 — Estado do portão, no fecho

| | |
|---|---|
| testes | **21 830 corridos · 21 829 verdes · 1 vermelho, corrigido e re-corrido verde (5/5)** |
| runner | `cargo nextest -j6`, `--no-fail-fast`, `CARGO_INCREMENTAL=0` |
| carga da máquina | `3,2` (a leitura só vale abaixo de ~5) |
| clippy | `--all-targets -D warnings` limpo em `ph2d-editor-core`, `ph2d-panel-registry-init`, `ph2d-host-desktop` |
| `cargo fmt --all` | corrido |
| binário de smoke | compilado (`2m 56s`, 2.ª corrida `0.19s`) |
| provas de mutação | **4 de 4 mortas** na w33; as waves anteriores trazem as suas nas mensagens de commit |

⚠️ **O vermelho era do gate, não do produto:** o teste novo proíbe a shell de guardar uma segunda
cópia da lei da coluna, e reprovou sobre um **doc-comment meu** que explica que ela já não está lá.
Cura: o gate passa a ler o **código sem comentários** (`code_only`). É a **terceira** ocorrência da
família *«um censo que lê o fonte tem de saber todas as formas do que lê»* nesta linha (w25, w27, w33).

---

## 10 — ⛔ Uma linha para o `CLAUDE.md §5` — E A ANTERIOR NUNCA FOI APLICADA

⚠️⚠️ **Conferido em 2026-09-07:** a linha que o handoff de 2026-09-06 propôs **não existe no
`CLAUDE.md` do `main`** (`git show main:CLAUDE.md | grep -i uiux` → vazio). ⇒ **o módulo de UI/UX não
tem entrada nenhuma no §5**, e é por isso que a próxima LLM o procura por `grep` em vez de o alcançar
por link. A linha abaixo **substitui** a anterior e cobre as 33 waves; aplicá-la fecha as duas.

> **UI/UX — redesenho plano** (Godot 4.6 «Modern» + modelo de painel do Blender; `PH2D_UI_NEW=0`
> volta ao clássico): quatro temas derivados, a moldura · o vão · a quina · o **recuo** e o vão
> **ícone→rótulo** cada um por UMA porta em [`ph2d-tokens`](crates/ph2d-tokens/src/spacing.rs), o
> **cartão** no lugar do risco azul, a **lei do grupo**, os **apelidos de cor** que resolvem pelo pai
> (⛔ 57 valores saíram do `tokens.json`), o **chão da janela** (`WindowGround`, degrau ABSOLUTO de
> `0,04`) que faz de cada área um cartão, e o **gesto que fecha uma coluna** arrastando a borda para
> dentro — cuja lei é uma **involução** (`hero::dock_columns`: reabrir devolve exactamente o que
> aquele fecho levou; re-derivar do registo abria **22** onde o fecho levara **1**).
> ⚠️ `HIER_ROW_H_PX` e `SECTION_GAP_PX` **deixaram de existir**.
> **Aberto:** o DESENHO das abas de painel (medido contra o Godot, não construído) · a incoerência
> entre as abas de encaixe e as de layout · esvaziar os painéis (1 de 25 censuados) ·
> [handoff](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-07.md).

---

## 11 — As leis que esta jornada pagou (para a próxima LLM, não para o integrador)

1. ⭐⭐⭐ **As duas metades de um interruptor podem perguntar a factos diferentes — e fontes
   diferentes devolvem CONJUNTOS diferentes.** A w30 escreveu a primeira metade da frase como lei e
   parou nela; a w33 pagou a segunda. *Quando duas metades de um gesto consultam factos distintos,
   meça a CARDINALIDADE de cada resposta antes de chamar à assimetria um desenho.*
2. ⭐⭐⭐ **Uma involução é uma MEMÓRIA, nunca uma re-derivação.** A informação que decide (*«o dono
   tinha isto aberto?»*) não vive no registo: ela existe num instante só, o do fecho.
3. ⭐⭐ **Um gate textual só pode afirmar sobre QUEM CHAMA QUEM, nunca sobre o que a chamada faz.** O
   gate que cobria este gesto exigia a string `panel_visibility.insert(id, true)` — *a linha do
   defeito* — e chamou-lhe correcta. A conduta mudou de crate **para poder ter régua**.
4. ⭐⭐ **Nenhum gate deste repo media a CARDINALIDADE de nada.** A grandeza do report — *«vários»* —
   não tinha instrumento (`grep is_panel_visible | count/len/filter` → zero linhas).
5. ⚠️ **Eu escrevi um teste VÁCUO e só o vi ao tentar matá-lo por mutação.** O dos flutuantes fechava
   e reabria com zero flutuantes visíveis — caminho em que nenhum pode aparecer faça o código o que
   fizer. *Uma fixtura sem o fenómeno mede silêncio.*
6. ⚠️ **Editei um pintor que o produto não corre** (w32), com o aviso escrito em português no
   ficheiro ao lado. Sétima ocorrência da família *«a resposta já estava no repo e eu não a li»*.
7. ⚠️ **O `| tail` mascarou um `clippy` vermelho** e eu quase o dei por verde. O repo tem isto
   registado; eu repeti-o na mesma jornada em que o citei.
