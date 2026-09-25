# `line/UIUX` · 2026-09-25 · **HANDOFF DO INTEGRADOR — a linha inteira desde `395da6a55`**

> ⚠️ **Este é o documento do INTEGRADOR.** A narrativa da jornada — o mecanismo de cada wave, as
> recusas medidas e as premissas refutadas — vive no handoff da jornada,
> [`HANDOFF_INTEGRACAO_line_UIUX_2026-09-20_A_PALETA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-09-20_A_PALETA.md)
> (uma secção `§9-*` por wave, na ordem em que foi escrita; as mais antigas arquivadas com prova em
> [`docs/archive/uiux-paleta-2026-09-24/`](../../archive/uiux-paleta-2026-09-24/)). Aqui fica só o
> que a fusão precisa: **o que colide, o que parte, o que medir.**
>
> ⛔ Integrar e enviar são **ordem do dono** (§0.7). Esta linha fechou e **parou**.

---

## §0 — Em uma página

| | |
|---|---|
| ramo | `line/UIUX` · worktree `Worktrees/line-UIUX` |
| merge-base | `395da6a55` |
| commits | **72** (`git log --oneline main..line/UIUX`) · `291` ficheiros · `+21 567 / −4 818` |
| o `main` andou | **1** commit desde o merge-base (`20a630f1b`, só `project-memory/project_teste_cascadeur_2d_bones_testbed.md`) — **sem sobreposição** com a linha |
| contadores partilhados | **ZERO mexidos** (tabela do script no §1) |
| contrato congelado (§6) | **intocado** |
| ADR | **nenhum** |
| pacote externo novo | **nenhum** |
| rebase | ver §6 — feito na worktree antes deste handoff |

**O que a linha entrega, por assunto** (o detalhe de cada um é a `§9-*` indicada no handoff da jornada):

1. **O censo do degrau `G`** — de quem são as entradas de cada painel (comandos · valores · cromo ·
   órfãos · altura contra a dobra), e a paleta de pincéis que tirou o selector do painel da
   escultura (o modal de ecrã inteiro passa a ser dono do teclado **e** do ponteiro).
2. **As colunas do dock** — a largura de fábrica é uma FRACÇÃO da janela; o piso de uma escolha do
   artista é `210 px` (ordem do dono). ⚠️ **API mudou** (§3).
3. **O Inspector abre DOBRADO**, as secções na ordem da paleta, e o censo de comandos passou a medir
   COMPOSTOS como um controlo (`composto::grupo`).
4. **A linha de propriedade em todo o app** — toda ESCOLHA passa pela porta `paint_choice_row` (ao
   lado do nome quando cabe; PALETA quando não, decisão do dono de 23/09), todo botão de acção pela
   `caixa_do_botao` (coluna do valor, altura de um campo), as cores e o *per-corner* viraram linha de
   formulário, e **o valor arranca num `x` só por painel**.
5. **Os painéis Vector, Física, Audio Mixer e Grid & Snap arrumados** — liga/desliga viram CAIXAS DE
   MARCAR, grelhas viram escolhas declaradas, o *Key* do ducking deixou de ciclar.
6. ⭐ **O título de secção do app inteiro é o do Grid** (corpo `Md`, sem caixa alta, o separador azul
   na MESMA linha à direita do nome), **as secções do Grid dobram**, e **a grade vai ATRÁS dos
   objectos de verdade** (o `Behind` era uma opacidade × `0,4`).

---

## §1 — Superfície de colisão (colada do script, não escrita à mão)

```text
SUPERFÍCIE DE COLISÃO — line/UIUX contra main
  merge-base 395da6a55   ·   72 commit(s)   ·   291 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        160   (base: 160)
      └ tripla do gate               (160, 13, 22)   (base: (160, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
    FIELD_DOC_VERSION                      23   (base: 23)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                              103   (base: 103)
    ph2d-render (espelho)                 104   (base: 104)
    ph2d-script (espelho)                 104   (base: 104)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️ **A coluna `base:` é o MERGE-BASE, não o `main` de agora** — se outra linha fundir antes desta,
releia os valores no ficheiro. Como esta linha não mexe em nenhum, o valor certo depois da fusão é
**o do `main` do dia**, sem degrau desta linha.

---

## §2 — O que PARTE em outra linha ao fundir (leia antes de fundir a segunda)

Esta linha mexe em **fundação** (`ph2d-editor-core`) e em **visuais que o app inteiro pinta**. As
quebras possíveis, por espécie:

### §2.1 — Assinaturas públicas que MUDARAM (erro de compilação noutra linha)

| item (`ph2d-editor-core`) | antes | agora |
|---|---|---|
| `…::dock_width` | `(&self, side)` | `(&self, side, janela_w: f32)` — a largura de fábrica é fracção da janela |
| `…::set_dock_width` | `(&mut self, side, w: f32)` | `(&mut self, side, w: Option<f32>)` — `None` = a de fábrica (apagar a escolha) |
| `…::DOCK_W_MIN` | `PANEL_MIN_W_PX` | `210.0` (ordem do dono) + `PISO_DO_CORPO_PX = 84.0` |
| `RowCtx::segmented` | `mut y` | `y` (sem efeito no chamador) |

⚠️ Uma linha que chame `dock_width`/`set_dock_width` com a forma antiga **não compila** depois da
fusão — a cura é passar a largura da janela / `Some(w)`, nunca repor a forma antiga.

### §2.2 — Portas NOVAS que outra linha deve USAR (e os censos que cobram)

| porta | o que decide | o censo que reprova quem a contorna |
|---|---|---|
| `property_row::paint_choice_row` | toda escolha com nome (ao lado / PALETA) | `nenhuma_escolha_do_app_e_montada_a_mao` · `nenhum_nome_por_cima_do_controlo` |
| `property_row::caixa_do_botao` + `abaixo_do_botao` | onde vai um botão de acção e o vão depois dele | `an_action_button_asks_the_door_where_it_goes` (com `LINHA_INTEIRA_OK` nomeadas) |
| `widget::composto::grupo` | declarar que N peças são UM controlo | `a_carga_de_comandos_de_um_painel_so_encolhe` |
| `paint_check_row` | toda caixa de marcar | — |
| `widget::section_title_px` + `regua_do_titulo` | o título de secção | `a_regua_corre_na_linha_do_titulo_a_direita_do_nome` |
| `screens::hero::grid_layer` | a grade à frente / atrás | `grid_layer_tests` · `present_bands_grid_tests` |

⛔⛔ **Uma escolha, um botão ou uma caixa que OUTRA linha pinte à mão reprova na árvore COMBINADA**
— os censos varrem a pintura do app inteiro. A cura é passar pela porta, **nunca** uma entrada nova
nas listas de excepção sem mecanismo escrito ao lado.

### §2.3 — Catracas que SOMAM entre linhas (a fusão pode deixá-las vermelhas sem ninguém errar)

Todas em `crates/ph2d-panel-registry-init/tests/it/`:

- `CARGA_DE_COMANDOS` (`quantas_entradas_tem_cada_painel.rs`) — valores MEDIDOS por painel, **só
  desce**: `inspector 81 · tokens 4 · wet_tuning 17 · physics 2 · sculpt3d 36 · vector 8 · model3d 1
  · audio_mixer 17 · grid_snap 2`. Outra linha que acrescente um comando a um destes painéis sobe o
  número ⇒ vermelho. ⛔ A cura é declarar o grupo / passar pela porta; subir o número só com ordem
  do dono.
- A tabela das **alturas de abertura** (mesmo ficheiro) — `sculpt3d 2051 · tokens 2866 · vector 1262
  · physics 1281 · audio_mixer 1207`. ⚠️ Estas mudam com a altura das secções: outra linha que ponha
  uma linha num destes painéis mexe nelas.
- `COLUNAS_DECLARADAS_POR_PAINEL` (`onde_comeca_o_valor.rs`) — `physics` declara `2`.
- As duas catracas de ELISÃO do degrau estreito (`nenhum_rotulo_do_app_pinta_nada.rs`) — ⚠️ **correm
  só no âmbito de WORKSPACE** (os painéis `flip`/`painter_layers`/`wet_tuning` só se registam com a
  shell): uma corrida `-p` reprova alto com a causa na mensagem.

### §2.4 — Mudanças VISUAIS que o app inteiro vê (um golden/pixel de outra linha pode mudar)

- ⭐ **Todo título de secção** (`paint_section_header`, 49 sítios): o nome como está escrito (saiu o
  `to_uppercase`), corpo `TypeToken::Md` em vez de `Sm`, e um separador azul de `1 px` à direita do
  nome. Um teste de OUTRA linha que procure o título em CAIXA ALTA, ou um golden de pixel com um
  cabeçalho dentro, muda — **e a mudança é a ordem do dono**, não um defeito da fusão.
- **A grade com `Behind`** força o quadro em camadas (o acumulador das FAIXAS, ADR-0154 F2). Sem
  `Behind` e sem intercalação o quadro é o de sempre, byte a byte. ⚠️ Uma linha que mexa no
  `present.rs`/`present_bands.rs` (as faixas, o vidro) encontra aqui o `plan.banded |= grid_behind` e
  o bloco da grade logo depois do `clear_linear` — a ORDEM é gateada por texto.
- O Inspector abre com as secções DOBRADAS.

### §2.5 — Texto (HR-15)

- `crates/ph2d-i18n/src/audio.rs`: a chave `panel.audio_mixer.master.key` passou de `"Key: {bus}"`
  a `"Key"` (a escolha tem as quatro peças à vista). Outra linha que use essa chave com `tr_with`
  e `{bus}` pinta o texto sem substituição.
- Chaves novas de secção/ids do Grid: `GS_SEC_KIND/TARGET/DISPLAY` (hash, na crate do painel).

---

## §3 — Fundação tocada, e porque é ADITIVA (ou onde não é)

| sítio | mudança | aditiva? |
|---|---|---|
| `property_row/{botao,escolha}.rs` | portas novas | sim |
| `widget/composto.rs` | a declaração de grupo (desarmada = um `Cell::get` e `return`) | sim |
| `widget/section_header/mod.rs` | estilo do título + `section_title_px` + `regua_do_titulo` | **visual para todos** (§2.4) |
| `grid_snap/inspect.rs` | `paint_body` separado; `paint` delega | sim |
| `screens/hero/grid_layer.rs` | a camada da grade | sim; o `hero/paint.rs` deixou de pintar a grade directamente e perdeu o `× 0,4` |
| `screens/hero` (dock) | largura por fracção da janela | **assinaturas mudaram** (§2.1) |
| `ph2d-text` | nada | — |

---

## §4 — Smokes (o dono aprovou todos; os comandos são os da WORKTREE)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && env PH2D_PHYSICS_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

O que olhar: `Window → Grid & Snap` (títulos, dobra das quatro secções, `Behind` com a grade atrás
do chão e da caixa na cena `PH2D_PHYSICS_SMOKE=1`) · `W` (Física: caixas *Enabled* e *Show
Colliders*) · `Window → Audio Mixer` (as quatro caixas dos efeitos, o *Key* como escolha, *Play
Test*) · o painel Vector (a grelha dos modos como escolha, as acções na coluna do valor) · o
Inspector (abre dobrado, ações na coluna do valor).

---

## §5 — O portão do fecho (MEDIDO, sobre o diff acumulado)

PREENCHIDO_NO_FECHO

---

## §6 — Rebase e árvore combinada

PREENCHIDO_NO_REBASE

---

## §7 — A UMA linha proposta para o `CLAUDE.md` §5 (o integrador aplica; a linha não edita o §5)

Na entrada **UI/UX**, substituir o parágrafo `**Aberto:**` pelo ponteiro:

> ⭐⭐⭐ **A LINHA DE PROPRIEDADE CHEGOU AOS PAINÉIS E O TÍTULO DE SECÇÃO É UM SÓ** (25/09,
> [handoff do integrador](docs/UI_New_and_Simple/handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-09-25_A_LINHA.md)):
> toda escolha passa por `paint_choice_row`, todo botão de acção por `caixa_do_botao`, todo grupo se
> DECLARA (`composto::grupo`) e a catraca `CARGA_DE_COMANDOS` só desce; o título de secção do app é o
> do Grid (corpo `Md`, sem caixa alta, separador à direita) e a grade `Behind` vai atrás de verdade
> pelo acumulador das faixas. ⏳ **Aberto:** partir o `DrawMode` nos dois eixos (decisão do dono) ·
> as caixas do `painter_layers` registadas como botão (só instrumento) · o painel da escultura
> (território da `line/sculpt3d`).

---

## §8 — O que fica ABERTO

- **Decisões do dono:** partir o `DrawMode` (a grelha de ferramentas do Vector) nos dois eixos · a pose 2D/3D
  e os toggles de módulo → Layout (herdados da integração anterior).
- **Instrumento, sem mudança visível:** as caixas de marcar do `painter_layers` estão registadas no
  store como `Button` (pintadas como caixa; o censo conta-as como comando). Convertê-las mexe no
  contrato `PanelEvent::Click` da ferramenta do Painter — fica para quando houver motivo de produto.
- **Território de outra linha:** os `36` comandos do `sculpt3d` vivem no painel que a `line/sculpt3d`
  mexe todos os dias — não tocados aqui para não criar colisão.
- **O chevron do título:** a foto do dono não o tinha; ficou, porque a mesma ordem pede que as
  secções fechem. Se o dono quiser o título sem seta, a mudança é só no `paint_section_header`.
