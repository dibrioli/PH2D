# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: I10 (18/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; [I6/I11](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I6_I11.md),
> [I7](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I7.md),
> [I8](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I8.md) e
> [I8b](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I8b.md) também estão na linha; este I10 vem por
> cima. Desenho e provas: [03_onda2.md](../03_onda2.md) §Estado (linha I10) · medição:
> [I10_rename_e_undo.md](../ferramentas/medicoes_onda2/I10_rename_e_undo.md). O mecanismo vive nesses; aqui
> só o resumo.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  merge-base com o `main` de hoje `bdc7a38639` (o `PROJECT_SCHEMA` lá é 191) · HEAD `e83bddae3d` (antes do
  commit dos docs).
- Commits da onda I10 (sobre `80e357927e`), **7**: `b67615438f` docs (medição) · `0e0eb0323d` feat (o marcador
  do undo: `ph2d-editor-core` + shell + `ph2d-code-buffer`) · `1b51640321` feat (rename e referências) ·
  `dfacc42786` + `a64cac81dd` + `68bb1f679d` test · `e83bddae3d` fix (`push_undo` só nos testes).

## 2. Superfície de colisão (`collision-surface.sh`, 18/10, contra o main)

```
merge-base bdc7a38639 · 42 commit(s) · 182 arquivo(s)  (a linha toda, I6–I10 incluídos)
PROJECT_SCHEMA 191 (= base) · tripla (191, 13, 23) (= base) · VEC 23 · FLIP 13 · DOC_VERSION 20 (= base)
registos de componentes 108/109/109 (= base)
contrato congelado (nodegraph/node.rs, editor-core/tool.rs): intocado · ADR: nenhum (próximo livre 0189)
Cargo.lock: 1 pacote '+name' novo = "ph2d-code-explain" (INTERNO; vem do I8)
marcadores de conflito: nenhum · TETOS DE LOC: nenhum ficheiro passa do tecto
```

**Contadores como DELTA desta onda:** nenhum (sem degrau de schema, sem registo, sem ADR, sem cena de
smoke, `Cargo.lock` +0). Comandos da tabela `TABLE`: **+2** (7 → 9: «Rename Symbol · F2», «Find References ·
Shift+F12»).

**FOUNDATIONAL tocado (`ph2d-editor-core`, aditivo):**

- `crates/ph2d-editor-core/src/undo_link.rs` (novo): `UndoLink` (único no processo) e `UndoLinks`
  `{pending, ask, held[2], done, alive, top[2], refused}`; portas puras `global_may` / `begin_undo` /
  `end_undo` / `take_ask` / `publish` / `is_alive`.
- `HeroScreen::undo_links` (campo novo; construtor em `hero.rs`, **696/700 linhas**: um campo a mais
  noutra linha estoura o tecto).

## 3. O que a onda fez (resumo; mecanismo na medição e no código)

- **Medição** (`b67615438f`; `ferramentas/medicoes_onda2/I10_rename_e_undo.md`, bruto `I10_bruto.json`,
  recortes em `I10/`, oráculo `ferramentas/oraculo_i10/corre.sh` (`I10_SO=6` só a cena do atraso), sonda
  `ferramentas/lspprobe_i10.py`). **`luau-lsp` 1.70.1:** renomeia locais, parâmetros e sombras certos; aceita
  `end` e nomes já existentes (a validação é NOSSA); renomeia o gancho `update`; «Unable to find symbol»
  em `self.x` e no literal de `ph2d.property`; «Cannot rename a global variable» em `ph2d`/`math`;
  **SIGSEGV 12/12** em rename/references sobre membros (`ph2d.set`, `math.sin`); `prepareRename` e
  `documentHighlight` devolvem −32601; tempos ≤ 0,3 ms a quente. **VS Code 1.142.0:** F2 abre caixinha com o
  nome seleccionado, Enter aplica e grava (`files.refactoring.autoSave`); Ctrl+Z num dos dois ficheiros
  desfaz os dois sem perguntar; com conflito avisa e desfaz SÓ um lado (projecto partido); aceita `1x` e
  `class`; realce das ocorrências ≤ 125 ms. **Godot:** sem rename.
- **Marcador do undo global** (ABERTO da Onda 1): `undo_link.rs` acima. **Shell:** `ProjectUndo` com degraus
  triplos (estado, marca, `Option<UndoLink>`), `push_linked` / `top_link` / `links` (`undo.rs`);
  `undo_app.rs`: `apply_undo` chama `begin_undo`/`end_undo`; `post_frame_undo` = wrapper que publica
  `top`/`alive` no fim de TODO caminho; `take_ask` antes do `undo_request`; elo pendente faz nascer degrau
  mesmo sem diff. **`ph2d-code-buffer`:** `History` — `Group.link`, `link_last` / `top_link` / `holds_link`
  / `unlink`.
- **Família** (`crates/ph2d-app-code/src/link.rs`): `Linked {link, script, before, after}`; a cada quadro:
  `scene_facts` das caixinhas de propriedade → `apply` (biblioteca por `rename_property_in` + mover as chaves
  `own` + `pending`) → `done` vira `undone` (`undo_text` do grupo + `set_library` + `mark_saved` se igual +
  `request_reload`) → tira elos mortos → `held` → `link_ask` da aba vira `ask` → `refused` vira toast
  `code.undo.family_ahead` / `global_ahead`. `edit.rs` `CodeDoc::undo`: grupo com elo pede (`link_ask`), o
  texto só anda quando a fila global anda; `undo_text` = o local.
- **Rename** (`crates/ph2d-app-code/src/rename.rs`; pedidos em `intel_names.rs`, filho de `intel.rs`):
  `Renaming`, `Kind {Name, Field{sites, declared}}`, `Why`, `Scene`, `SceneRename`; F2 / comando
  `start_rename`. **Validação nossa:** reservada (`LUAU_KEYWORDS` + `RESERVED_NAMES` = `id`), inválida,
  `InUse` (`name_in_use` / sites do `self`), `Taken` (property declarada ou chave `own` num objecto).
  `Name` → `Req::Rename` ao servidor com verificação do texto antigo; `Field` → edições nossas num passo e,
  se declarada, `link` + `SceneRename` (o valor de cada objecto viaja). `ph2d-code-syntax/src/names.rs`
  (`NameAt {Name, SelfField, Hook, Member}`, `name_at`, `name_in_use`, `is_identifier`, `LUAU_KEYWORDS`);
  `self_fields.rs` ganhou `sites` (todos os sítios, incl. `self["x"]` e `function self.x`).
  `ph2d-code-intel/src/lsp/rename.rs` + `Req::Rename(usize, String)` género 8, `Req::References(usize)`
  género 9, `Answer::Rename/References`; o cliente declara rename/references.
- **Referências** (`refs.rs`): `Occurrences` — realce a cada (revisão, cursor, geração da árvore);
  `Name`/`Hook` → servidor; `SelfField` → nossos; `Member` → nada, nunca pergunta (o SIGSEGV).
  `RefsList` do Shift+F12 na janelinha com `ListKind::References`; linhas «N: código» com o nome marcado;
  Enter/Tab/clique levam lá.
- **Painel** (`ph2d-panel-code`): `rename_box.rs` (`Popover` + campo do find bar + ajuda/razão a vermelho;
  a11y `TextInput` filho do `Document`, rótulo «Rename: …»); `CodeFrame.occurrences` (cor `Bg3`) e `.rename`;
  `ListKind` substitui o `actions: bool`. **i18n** `ph2d-i18n/src/code_editor.rs`: `code.rename.*`,
  `code.undo.*`, `code.refs.row`, `code.a11y.rename/references`.

## 4. O que um MERGE pode partir

- **`HeroScreen`** ganhou campo (`undo_links`); `hero.rs` a 696/700 — merge com outra linha que também
  acrescente campo lá estoura o tecto.
- **`CodeFrame`** ganhou `occurrences`/`rename`: construtores fora da linha reprovam a compilar.
- **`CompletionView.actions: bool` → `kind: ListKind`**: quem lê o campo velho quebra.
- **`ProjectUndo`**: degraus agora triplos; `push_undo` é `#[cfg(test)]` (produção usa `push_linked`).
  Outra linha que chame `push_undo` fora de testes não compila; que desestruture o degrau como par, idem.
- **Comandos `TABLE` 7 → 9** (teste de contagem de comandos); **`post_frame_undo`** é wrapper novo em
  `undo_app.rs`: merge com mexida de outra linha no undo da shell pede leitura.
- **`Req`/`Answer`** de `ph2d-code-intel`: géneros 8 e 9 ocupados.

## 5. Testes e mutação

- `crates/ph2d-app-code/tests/it/renomear.rs` **14** (fila global em pequeno, `Fila`, com as MESMAS portas da
  shell): `renomear_uma_propriedade_leva_o_valor_de_cada_objecto` ·
  `ctrl_z_na_aba_e_na_cena_desfaz_o_texto_e_os_objectos_juntos` ·
  `com_a_outra_metade_atras_o_desfazer_recusa_por_inteiro` · `um_elo_morto_deixa_o_texto_desfazer_se_sozinho` ·
  `um_nome_que_nao_serve_diz_porque_e_o_enter_nao_aplica` · `um_objecto_que_guarda_o_nome_novo_recusa_o_rename` ·
  `um_membro_um_gancho_e_o_vazio_dizem_porque_e_nao_perguntam` ·
  `renomear_um_local_pelo_servidor_num_passo_de_undo` · `as_ocorrencias_seguem_o_cursor` ·
  `a_caixinha_pinta_se_fala_e_um_clique_fora_fecha` · `shift_f12_lista_os_sitios_e_enter_leva_la` ·
  `sem_assistente_o_local_diz_porque_e_a_propriedade_renomeia` ·
  `um_rename_do_servidor_sobre_outro_texto_nao_se_aplica` · `a_propriedade_espera_pelos_objectos_antes_de_aplicar`.
- `oraculo_lsp::o_assistente_fixado_renomeia_um_local_e_sobrevive_aos_membros` (`luau-lsp` real: mesmo arranque
  depois dos membros) · shell `undo::tests::the_undo_link_travels_with_its_step_and_dies_with_the_redo` ·
  unitários: `names` 2, `self_fields` (sites), `lsp/rename` 2 (code-intel), `undo_link` 1, `rename::tests` 1.
- Vizinhos verdes por nome: assistente, dicas, definicao, acoes, botoes, cores, teclado, despacho, a11y,
  explicacoes, completar, salvar, projecto, abrir, lista, busca, gramatica, dobras, onda1, console,
  linha_longa, oraculo_lsp.
- **Mutação: 25 mutações; 1.ª rodada 22/25, depois 25/25.** Sobreviventes: M16 (guarda de revisão em
  `occurrences_answer` — inalcançável, uma edição larga o bilhete; **removida**), M17 (verificação do texto
  antigo no rename do servidor — teste novo) e M25 (Enter antes de ler os objectos — teste novo).
- Corrida: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/ph2d-run.sh cargo test -p ph2d-app-code --test it -- renomear::`

## 6. Portões da onda (18/10)

- `cargo fmt` nas crates tocadas. **Sem clippy e sem suíte**: cadência do dono (só no fecho da linha); a
  suíte `PH2D_SUITE=fecho` + clippy `--all-targets` ficam para o fecho/integração.
- ⚠️ A shell sem janela não desfaz (`gfx None`): o caminho REAL da fila global com elo só se prova no smoke do
  dono; o código da shell chama as portas puras, essas testadas.

## 7. Binário do smoke

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`:

```
▸ linha line_codeeditor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.36s
```

(zero `Compiling`).

## 8. Smoke para o DONO

> ⚠️ Não fotografado. Para os passos com locais (`amplitude` é uma propriedade, não precisa), o assistente
> não é necessário; para renomear uma variável local, instale-o com `bash scripts/luau-lsp-fetch.sh`.

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Clique no **Circle**; no cartão **Script**, mude **amplitude** para **3** (o círculo passa a subir mais).
3. **Edit script**. Clique na palavra `amplitude` da linha `self.base_y + s * self.amplitude` e carregue **F2**: abre uma caixinha com `amplitude` já seleccionado e, por baixo, «Enter to rename · keeps the value of 1 object».
4. Escreva `altura` e **Enter**: as três `amplitude` do script viram `altura`; a aba não fica com ●; no cartão do Circle aparece **altura = 3** e o círculo continua a subir alto.
5. **Ctrl+Z** na aba: tudo volta a `amplitude`, e o cartão mostra **amplitude = 3**. **Ctrl+Shift+Z** refaz.
6. Volte à cena (a aba da cena) e carregue **Ctrl+Z**: o script também volta a `amplitude` (desfaz junto).
7. Refaça (Ctrl+Shift+Z), escreva um espaço no script, volte à cena e carregue Ctrl+Z: aparece um aviso no canto a dizer para desfazer primeiro as edições do script; nada muda.
8. Clique em `base_y`: as outras ocorrências ficam com fundo realçado. **Shift+F12**: abre a lista dos sítios; setas e Enter levam o cursor lá.
9. F2 em `ph2d.set` ou em `function update`: a caixinha só diz porque não se renomeia.
10. **Deu errado se:** o valor 3 se perde (o cartão volta a 1 ou mostra um valor órfão); o Ctrl+Z precisa de dois toques; desfazer na cena deixa o script com `altura` (ou ao contrário); `end`/`1x` são aceites; o app trava ou o assistente reinicia ao usar F2 num `ph2d.set`.

> ✅ **Smoke do dono OK a 18/10** (inclui o Ctrl+Z na CENA — o caminho da shell que os testes não alcançam).

## 9. ABERTO

- O realce das ocorrências usa `Bg3`: contraste não medido em todos os temas.
- Renomear não abre prévia separada (o realce das ocorrências é a prévia, como no VS Code); o Ctrl+Enter do
  VS Code não foi feito.
- Shift+F12 sem referências não mostra «nenhuma referência» (a lista não abre).
- ⭐ **O desfazer na CENA de verdade não tem teste de ponta a ponta** (a shell sem janela não desfaz): só o
  smoke do dono o prova (passos 5–7).
- Os do I8b: botões do balão sem foco de teclado; lista de problemas (acabamento); Ctrl+rato em mão (I15);
  o ⚠️ do `state.vscdb` do dono ([I8b §9](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I8b.md)).
- Integração: a linha ainda não correu o clippy `--all-targets` nem a suíte do fecho.

## 10. Próximo

I9/I12/I14 → I13 (precisa do manifesto do StyLua) → I15. No acabamento: o Ctrl+rato em mão e a **lista de
problemas**. Antes de integrar: o smoke do dono do I10 e o fecho (`PH2D_SUITE=fecho`).
