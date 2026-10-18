# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: I8b (18/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; o [I6/I11](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I6_I11.md), o
> [I7](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I7.md) e o
> [I8](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I8.md) também estão na linha e este I8b vem por
> cima deles. Desenho e provas: [03_onda2.md](../03_onda2.md) §Estado (linha I8b) · medição:
> [I8b_botoes_no_balao.md](../ferramentas/medicoes_onda2/I8b_botoes_no_balao.md). O mecanismo vive nesses
> dois; aqui só o resumo.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  base = `main` `ad5f8fe39c` (o `PROJECT_SCHEMA` lá é 191) · HEAD `1b157f2fa8` (antes do commit dos docs).
- Commits da onda I8b (sobre `b8974b28b2`), **7**: `433cb3d506` fix (`ph2d-editor-core`: botão sobre
  superfície elevada) · `6132ae900a` feat (as correcções como botões no balão) · `24372835e5` style (fmt) ·
  `5b99da1b79` docs (medição do oráculo) · `60f9b250c6` feat (a 1.ª acção do Alt+Enter nasce com o foco) ·
  `b27a9eeabc` style (fmt de `ph2d-i18n/src/code_explain.rs`, sobra do I8) · `1b157f2fa8` fix (o hover VIVO).
- **Pedido do dono, 18/10** (smoke do I8): as correcções do Alt+Enter dentro do balão do problema, como
  botões com melhor aspecto, sem o bloco azul cheio. Clicar aplica directamente (decisão dele).

## 2. Superfície de colisão (`collision-surface.sh`, 18/10, contra o main)

```
merge-base ad5f8fe39c · 31 commit(s) · 149 arquivo(s)  (a linha toda, I6–I8b incluídos)
PROJECT_SCHEMA 191 (= base) · tripla (191, 13, 23) (= base) · VEC 23 · FLIP 13 · DOC_VERSION 20 (= base)
registos de componentes 108/109/109 (= base)
contrato congelado (nodegraph/node.rs, editor-core/tool.rs): intocado · ADR: nenhum (próximo livre 0189)
Cargo.lock: 1 pacote '+name' novo = "ph2d-code-explain" (INTERNO; vem do I8, nada de externo)
marcadores de conflito: nenhum · TETOS DE LOC: nenhum ficheiro passa do tecto
```

**FOUNDATIONAL tocado por esta onda** (`ph2d-editor-core`, append-only):

- `Button` ganhou o campo **`surface: ButtonSurface`** (`Panel` por omissão · `Elevated`) e o construtor
  `Button::on_elevated()`. O único struct literal de `Button` no workspace
  (`crates/ph2d-panel-grid-snap/src/paint_kinds_bounded.rs`) já leva `surface: ButtonSurface::Panel`;
  **qualquer outra linha que construa `Button` por literal deixa de compilar**.
- `paint_popover_row_focus` exportado ao lado de `paint_popover_row_highlight` (`widget/dropdown/popover.rs`).
- `Modal::new` põe os dois botões `.on_elevated()` ⇒ **muda o hover do Cancelar de TODO diálogo**.
- `ph2d-panel-code`: `HoverView` ganhou `fixes/hot/live`; `HintProbe` ganhou `buttons/hot/live`;
  `hints_a11y` devolve `(roots, nodes)`.

**Contadores que somam:** nenhum (sem degrau de schema, sem token de cor novo, sem linha de painel, sem registo
de componente, sem cena de smoke). Delta do `Cargo.lock` desta onda: zero.

## 3. O que a onda fez (resumo; mecanismo na medição e no código)

- **Medição do oráculo** (`5b99da1b79`; `ferramentas/medicoes_onda2/I8b_botoes_no_balao.md`, 17 recortes em
  `I8b/` + `amostras.txt`, script `ferramentas/oraculo_i8b/corre.sh`, Xvfb :117 privado). VS Code 1.142.0 e
  Godot 4.7.2 **NÃO listam as correcções no hover** (VS Code: rodapé genérico «View Problem (Alt+F8)» /
  «Quick Fix... (Ctrl+.)»; Godot: sem tooltip nem code actions) ⇒ **o botão por correcção no balão é NOSSO**.
  Clicar «Quick Fix...» abre o menu e **não** fecha o hover; aplicar fecha menu e hover juntos (nós: o clique
  aplica já, como o dono pediu, e o balão fecha). 1.ª acção pré-escolhida: VS Code pelo teclado = tinta
  neutra `#313233` sobre `#202122` + anel de 1 px de acento `#3994BC` por dentro; pelo rato só a tinta;
  Godot tinta `#353535` sobre `#252525`, sem anel. Nenhum usa bloco cheio do acento.
- **`ph2d-editor-core` (`433cb3d506`):** sobre superfície elevada (`BgElev`: popover, balão, diálogo) o
  hover/foco do `Default` e do `IconOnly` vai a `Bg3`, não a `BgElev` (a cor do próprio fundo: o botão
  DESAPARECIA sob o rato). Família (§0.10): o Cancelar de todo diálogo tinha o mesmo defeito, curado.
  `widget/button.rs` + `button_surface/cor.rs`.
- **Uma só porta** (`ph2d-app-code/src/actions.rs`): `CodeDoc::actions_at(heads)` — o Alt+Enter chama-a com o
  cursor; o balão com o início do culpado (ponto da calha: todos os culpados da linha); correcções do servidor
  deduplicadas por título. `CodeDoc::apply(&Actions, i)` partilhado por Enter/clique e pelo balão
  (`apply_fix`, `hint.rs`). `Hovering.fixes: Option<Actions>` nasce em `CodeDocs::notes()` quando a pausa
  amadurece (`note.rs`); pedidos em `Intel::ask_hints`; offline no ramo sessão-`None` e em `Hints::close`;
  respostas encaminhadas em `CodeDoc::answer` (`goto.rs`).
- **Painel** (`ph2d-panel-code/src/hint.rs`): `paint_box` põe os botões por baixo do texto (`ROW_H_PX`,
  `control_gap_px`, largura = rótulo na fonte MEDIUM do botão + 2×`Spacing::Lg`; os botões cabem sempre, o que
  se corta são linhas de texto); `hint_button_at`; a11y: botões `Role::Button` filhos do `Tooltip`; `paint.rs`
  só põe as raízes sob o `Document`. **Clique** (`pointer.rs`): botão aplica (balão fecha); clique no corpo do
  balão é consumido (balão fica, o cursor não anda).
- **Foco da janelinha** (`60f9b250c6`): `paint_popover_row_focus` = tinta `Bg3` + anel `BorderEmph`
  `StrokeToken::Thin` por dentro (FRAME-RAW-OK, visível em qualquer pele, como o anel de foco do botão).
  `popup.rs` usa-o na linha escolhida SÓ quando a lista é a do Alt+Enter; a lista de completion mantém o
  realce de acento.
- **Hover vivo** (`1b157f2fa8`): o portão `ph2d-editor-core it every_button_wears_the_live_hover` apanhou os
  botões do balão com o estado DURO (`.state`) — saltavam enquanto os do painel esmaecem.
  `CodeDocs::fix_hover(&mut UiMotion)` (`note.rs`) é chamada por `ph2d_app_code::frame` com o `hero.motion`
  (o MESMO `UiMotion` que o tick do herói avança ⇒ valem o carácter e o reduzir-movimento); uma pista por
  botão (`fix_track(serial, i)`, espaço de ids `0xC0DE_F1C5 << 32`), `Role::Fade`; o pintor usa
  `.visual((state, t))`.

## 4. O que um MERGE pode partir

- **`Button`** (ph2d-editor-core): campo `surface` novo ⇒ struct literal noutra linha não compila (o único
  hoje, `paint_kinds_bounded.rs`, está curado). Uma linha que pinte botões sobre `BgElev` à mão ganha o
  `.on_elevated()`.
- **`Modal::new`**: o hover do Cancelar mudou de cor em todo diálogo; um teste de cor/foto de diálogo de outra
  linha pode acusar.
- **`HoverView` / `HintProbe`** (ph2d-panel-code): campos novos; quem os construir por literal quebra.
  `hints_a11y` mudou de assinatura (`(roots, nodes)`).
- **`Hovering`** (ph2d-app-code): campo `fixes` novo; `Hints::close`/`CodeDoc::answer` ganharam o encaminhamento
  das correcções — merge com outra linha que mexa em `hint.rs`/`note.rs`/`goto.rs`/`pointer.rs` pede leitura.
- **Testes:** helpers de `acoes.rs`/`explicacoes.rs` ficaram `pub(crate)` (o `botoes.rs` novo os usa).
- **Espaço de ids de animação** `0xC0DE_F1C5 << 32`: nenhuma outra linha pode usá-lo.

## 5. Testes e mutação

- `ph2d-editor-core`: `widget::button::tests::on_an_elevated_surface_the_hover_never_paints_the_surface_colour`
  (12 temas; o `Panel` não muda) · `…the_focused_row_is_a_neutral_tint_a_step_above_the_popover` (12 temas).
- `ph2d-app-code` `tests/it/botoes.rs` **5**: `o_botao_do_balao_corrige_como_o_alt_enter_num_passo` (mesmo
  texto final que o Alt+Enter · nasce neutro · `hot` sob o rato · rampa viva 0 → (0,1) após um tick de 16 ms →
  1 no quadro real · filho `Tooltip` na a11y · o balão fecha · UM Ctrl+Z) · `o_ponto_da_calha_leva_os_mesmos_botoes`
  · `com_o_servidor_os_botoes_sao_os_do_alt_enter` (servidor falso: títulos `["Did you mean print?", "Do server"]`,
  a mesma ordem do Alt+Enter) · `nada_a_corrigir_nenhum_botao` · `o_clique_no_texto_do_balao_fica_nele`.
- Vizinhos por nome: `acoes`/`explicacoes`/`dicas`/`definicao`/`a11y` 34 ✓ · `lista`/`completar`/`acoes`/`botoes` 38 ✓.
- **Mutação: 11/11 sangram, com controlo verde.** M1 sem botões no quadro · M2 o clique não aplica · M3 `hot`
  nunca · M4 calha sem `heads` · M5 nunca pronto sem servidor · M6 a11y sem filhos · M7 `Elevated = BgElev` ·
  M8 o corpo do balão deixa o clique passar · M9 a resposta do servidor não chega ao balão · M10 o quadro não
  anima · M11 o pintor ignora o eixo. **Sobrevivente conhecido (sem mutação nem teste):** a deduplicação por
  título das correcções do servidor quando duas cabeças da mesma linha tocam o mesmo diagnóstico.
- Corrida: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/ph2d-run.sh cargo test -p ph2d-app-code --test it -- botoes::`

## 6. Portões da onda (18/10)

- `cargo fmt --all` limpo · clippy `-D warnings --all-targets` limpo em `ph2d-editor-core`,
  `ph2d-panel-grid-snap`, `ph2d-panel-code`, `ph2d-app-code` (depois da cura do hover, refeito em
  `panel-code` + `app-code`: limpo).
- `PH2D_SUITE=onda` `nextest-impacted`: **18 897 corridos, 18 896 passaram, 1 falhou** — o
  `every_button_wears_the_live_hover` acima, curado em `1b157f2fa8` e re-corrido por nome (4/4 ✓ nesse ficheiro).
  996 s; `loadavg` alto (~135 a 5 min no fim, outras linhas a correr; ver
  [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)). **A suíte NÃO foi re-corrida depois da cura** (só o
  portão que falhou, por nome, como manda o protocolo).

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª saída:

```
    Finished `smoke` profile [optimized] target(s) in 0.43s
```

(zero `Compiling`).

## 8. Smoke para o DONO

> ⚠️ Não fotografado. O passo 8 mostra mais com o assistente instalado (`bash scripts/luau-lsp-fetch.sh`).

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Clique em **Circle**, cartão **Script**, **Edit script**. Na linha com `self.amplitude`, troque `amplitude` por `ampiltude` e espere um segundo e meio: aparece um **ponto amarelo** à esquerda dessa linha.
3. Pare o rato no ponto amarelo. O balão mostra a explicação e, por baixo, um botão **Did you mean amplitude?** — cinza/neutro, não azul.
4. Passe o rato em cima do botão: ele acende **devagar** (esmaece, não salta) e o balão continua aberto.
5. Clique no botão: a palavra vira `amplitude`, o ponto some e o balão fecha. **Ctrl+Z** devolve `ampiltude` de uma vez só.
6. Repita com o rato parado no próprio `ampiltude` sublinhado (não no ponto): aparece o mesmo botão.
7. **Alt+Enter:** clique dentro de `ampiltude` e carregue Alt+Enter. Abre a listinha, com a primeira linha marcada por um fundo um pouco mais claro e um **contorno fino azul** — já não um bloco azul cheio. O Enter continua a aplicar.
8. (Com o assistente instalado) escreva `pirnt(1)` dentro de uma função e espere: no balão, os botões são os mesmos que o Alt+Enter mostra, por exemplo **Did you mean print?**.
9. **Deu errado se:** não há botão por baixo da explicação; o botão já nasce azul/destacado; ele salta em vez de esmaecer; o balão fecha quando o rato vai para cima dele; clicar no texto do balão move o cursor; a correção precisa de dois Ctrl+Z; a primeira linha do Alt+Enter continua um bloco azul cheio.

## 9. ABERTO

- ⚠️ **Efeito colateral do oráculo:** na 1.ª tentativa a corrida abriu o VS Code isolado ~10 s ainda com o
  HOME do dono e **modificou `~/.vscode-shared/sharedStorage/state.vscdb`** (08:53:54 de 18/10; base pequena
  partilhada com o VS Code dele). **O que mudou lá dentro não foi verificado.** O `oraculo_i8b/corre.sh`
  agora isola o HOME desde o início. O dono foi avisado.
- Tema CLARO não medido nos oráculos; as funções de IA do VS Code estavam desligadas durante a medição.
- Os botões do balão **não recebem foco de teclado** (o caminho do teclado é o Alt+Enter) e o «Click» de um
  leitor de ecrã sobre eles não é encaminhado — igual a todo botão do app (não há tratamento de acções do
  AccessKit em lado nenhum do repo).
- O rato passando pelas linhas da listinha do Alt+Enter não move a selecção (o VS Code move); não foi pedido.
- O sobrevivente da deduplicação por título (§5).
- Do I8, ainda abertos: **lista de problemas** (painel; o dono pediu-a para o acabamento), o cartão não grava
  as explicações vistas, Ctrl+rato em mão (I15), só inglês; `ph2d-app-code/src/docs.rs` a 687/700 (esta
  onda não lhe tocou).

## 10. Próximo

**I10** (rename e referências + marcador do undo global) → I9/I12/I14 → I13 → I15. No acabamento: o Ctrl+rato
em mão e a **lista de problemas**. No I15, a bancada repete-se numa rodada só com a lista E os balões abertos.
