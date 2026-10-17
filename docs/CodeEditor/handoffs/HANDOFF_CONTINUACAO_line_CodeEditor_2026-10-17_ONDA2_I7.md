# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: I7 (17/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; o [I6/I11](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-17_ONDA2_I6_I11.md) também está na
> linha e este I7 vem por cima dele. Desenho e provas: [03_onda2.md](../03_onda2.md) §Estado (linha I7) ·
> medição: [I7_acoes_e_voce_quis_dizer.md](../ferramentas/medicoes_onda2/I7_acoes_e_voce_quis_dizer.md)
> (bruto `I7_bruto.json`, sonda `../ferramentas/lspprobe_i7.py`). O mecanismo vive nesses dois; aqui só o resumo.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  base = `main` `ad5f8fe39c` (o `PROJECT_SCHEMA` lá é 191) · HEAD `b0883eb2f1` (antes do commit dos docs).
- Commits da onda I7 (sobre `f5260b1972`): `019e901b08` docs (medição) · `871340f0f1` feat ·
  `cb5364c237` test (Alt+Enter contra o servidor falso e o `luau-lsp` fixado) · `0d0a6026aa` fix (das
  reservadas só `true`/`false`/`nil`; gancho sem «Prefix» no fixado) · `ec671a0ff8` test (as 4 mutações
  sobreviventes) · `b0883eb2f1` fix (clippy). **6 commits**, 32 ficheiros contra `f5260b1972`.
- Sem o assistente o editor continua a editar; sem ele só as acções NOSSAS (lint do `self`) aparecem, já.

## 2. Superfície de colisão (`collision-surface.sh`, 17/10)

```
PROJECT_SCHEMA 191 (= base) · registos de componentes 108/109/109 (= base)
contrato congelado: intocado · ADR: nenhum · Cargo.lock: nenhum pacote externo novo
marcadores de conflito: nenhum · TETOS DE LOC: nenhum ficheiro passa do tecto
  (actions.rs 443, intel.rs ~546, popup.rs ~415 contra 600 do painel)
```

**FOUNDATIONAL: nenhum tocado** (nada em `ph2d-editor-core`, shell, tokens). **Contadores que somam:
nenhum novo** (sem token de cor, sem linha de painel, sem cena de smoke; as strings novas estão em
`ph2d-i18n` / `code_editor.rs`).

## 3. O que a onda fez (resumo; mecanismo no doc da onda)

- **Medição:** o `luau-lsp` 1.70.1 não corrige erros de digitação (só `quickfix` «Prefix '_'» e «Remove
  unused function»); a completion no INÍCIO do nome errado dá as candidatas (globais 55, membros os 17);
  o `1030` do estrito não tem edição; dicas de tipo só com `inlayHints` na configuração. Oráculos sobre
  177 erros: `rustc` 173, Python 157, Godot 137; a lei do `rustc` (maiúsculas primeiro, Damerau/OSA ≤
  `max(len,3)/3`, empates todos) explica 177/177.
- **`ph2d-code-intel`:** `Req::Fixes` (género 5) · `Req::Candidates` (6, nunca cancela a lista) ·
  `Req::TypeHints` (7); `Answer::Fixes` / `Answer::TypeHints`; `lsp/fix.rs` (só `quickfix` com edição
  neste documento que ecoa um diagnóstico); exporta `Edit`/`Fix`/`TypeHint`.
- **`ph2d-code-lsp`:** `ph2d_settings` liga as dicas de tipo (parâmetros: nenhum).
- **`ph2d-code-syntax`:** `self_fields.rs` (`SelfFields`): campo do `self` LIDO que nunca é dado nem
  `ph2d.property` nem reservado (`id`); cala-se se o `self` foge ou há erro de sintaxe.
- **`ph2d-app-code`:** `lint.rs` (aviso «self.X is never set … did you mean Y?», na pausa do Godot de
  1,5 s / 0,5 s) · `actions.rs` (Alt+Enter; ordem «Did you mean» → «Add type» → quickfix; aplicar num
  passo de undo, cursor no fim da 1.ª troca; Esc/tecla/clique fora/edição — também a composição — fecham;
  vazia = «No quick fixes here») · `keys.rs`, `pointer.rs`, `complete_doc.rs`, `doc.rs`, `goto.rs`,
  `intel.rs`.
- **`ph2d-panel-code`:** `CompletionView.preview: &[PreviewLine]` e `.actions: bool`; `PreviewLine`
  exportado; `paint_preview`/`crop` (prévia cortada à volta do que muda); a11y «Quick fixes».
- **i18n:** `code.problem.unknown_field`, `_near`, `code.action.did_you_mean`, `code.action.add_type`,
  `code.action.none`, `code.a11y.actions`.

## 4. O que um MERGE pode partir

- **`ph2d_settings`**: outra linha que mexa nas settings do `luau-lsp` — confira `inlayHints`.
- **`CompletionView`** ganhou 2 campos: quem o construir noutro sítio (hoje só `doc.rs`) deixa de compilar.
- **Géneros do `Req::kind`**: 5/6/7 são nossos; outra linha que acrescente um `Req` usa 8+.
- **`enum Answer`** ganhou 2 variantes: `match` exaustivo noutro lado quebra.
- Testes: `posicao`, `clica_em`, `cursor`, `fixado` passaram a `pub(crate)` no servidor falso.

## 5. Testes e mutação

- `acoes::` **9**: `ampiltude` → `amplitude` + um Ctrl+Z (controlo `base_x`/`id`/property) · a pausa ·
  nada a corrigir / tecla / seta / Esc · candidatas do servidor · clique · composição com as perguntas em
  voo · encostado · gancho sem correcção (`um_gancho_do_motor_nao_ganha_correccao`) · `luau-lsp` fixado
  (`pirnt`→print, `ph2d.proprety`→property, `local up`→«Add type: boolean», `update` sem «Prefix»).
  Unitários: `lint::a_lei_do_rustc`, `actions::o_nome_errado_que_o_servidor_acusa`, `self_fields::` 3,
  `fix::` 2, `popup::a_previa_corta_a_volta_do_que_muda`.
- Servidor falso (`tests/it/assistente.rs`): `Plano.agir: Option<Agir>` (alias
  `Agir = fn(&str,&str,usize,&[Value])->Value`), `Visto.acoes`, `Visto.tipos`.
- **Mutação (agente `mutacao`): 32/32 sangram.** 1.ª rodada 28/32; sobreviventes: M12 (ramo morto
  `_ => None` em `use_of_self`, inalcançável — reescrito) · M24 (filtro do gancho em `open_actions`
  redundante com o cliente, que esconde o `FunctionUnused` dos `-- @hook`; o falso não esconde ⇒ teste
  novo) · M25 (`self.actions = None` no `edited`, caminho `Compose::Commit`: a janela do `nunca` acabava
  antes de o servidor lento de 40 ms responder — o teste passou a esperar `Visto.tipos == 1`) · M28
  (`self.actions = None` em `keys.rs`: faltava uma tecla que não edita — a seta). 2.ª rodada: as 4 sangram.
- **Lição (memória):** um `nunca` com servidor lento não prova nada se a janela acaba antes da resposta —
  espere o servidor ter respondido (contador no falso) e SÓ DEPOIS dê a janela.
- Corrida: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/ph2d-run.sh cargo test -p ph2d-app-code --test it -- acoes::`

## 6. Portões da onda (17/10)

- `cargo fmt --all` sem mudanças · clippy `-D warnings --all-targets` verde em ph2d-app-code,
  ph2d-code-intel, ph2d-code-syntax, ph2d-panel-code, ph2d-code-lsp, ph2d-i18n (a 1.ª rodada reprovou
  `unnecessary_cast` no `self_fields`, `single_range_in_vec_init` no teste da prévia,
  `needless_range_loop` no OSA: curados em `b0883eb2f1`).
- Testes por nome: `--test it` **80/80** (acoes definicao cores dicas assistente completar oraculo_lsp
  teclado a11y gramatica dobras onda1 lista) · app `--lib` 14 · code-intel 5+6 · code-syntax 5 ·
  panel-code 2+3 · i18n 16. ⚠️ o filtro `client config` do `ph2d-code-lsp` casou **0** testes: ali nada
  foi verificado além do clippy.
- Suíte `PH2D_SUITE=onda nextest-impacted`: **18 879 / 18 879**, 29 lentos, 851 s, `loadavg` no fim
  19,36/71,05/67,43; `ph2d-tool-painter … n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto`
  SLOW 68,9 s mas passou ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)).

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª saída:

```
▸ linha line_codeeditor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.32s
```

## 8. Smoke para o DONO

> ⚠️ Não fotografado (a foto e a cena `PH2D_CODE_SMOKE=3` são do I15). Precisa do assistente instalado
> (`bash scripts/luau-lsp-fetch.sh`) nos passos 6 e 7.

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Na Hierarchy clique em **Circle**; no Inspector, cartão **Script**, clique **Edit script**: abre a aba **bob.luau**.
3. **Um nome trocado:** na linha `local d = s * self.amplitude * brilho`, dê duplo clique em `amplitude` e escreva `ampiltude`. Pare um segundo e meio: `ampiltude` fica sublinhado (aviso).
4. Clique dentro de `ampiltude` e carregue **Alt+Enter**: abre uma janelinha com **Did you mean amplitude?** e, ao lado, a linha como vai ficar, com `amplitude` marcado.
5. Carregue **Enter**: a palavra volta a `amplitude` e o sublinhado some. Carregue **Ctrl+Z** uma vez: volta `ampiltude` de uma vez só. **Ctrl+Y** refaz.
6. **Um nome do programa errado:** clique no fim de `self.t = self.t + dt`, carregue Enter e escreva `pirnt(1)`. Espere o sublinhado vermelho, clique dentro de `pirnt` e carregue **Alt+Enter**: aparece **Did you mean print?**; Enter troca para `print(1)`.
7. **Pôr o tipo:** clique na palavra `up` de `local up = s > 0.98` e carregue **Alt+Enter**: aparece **Add type: boolean**; Enter deixa `local up: boolean = s > 0.98`.
8. Alt+Enter numa palavra sem nada para corrigir (por exemplo `brilho`) mostra **No quick fixes here**; **Esc** fecha.
9. Deu errado se: o sublinhado de `ampiltude` não aparece; o Alt+Enter não abre nada ou abre vazio onde havia correção; o Enter troca a palavra errada ou só metade; o Ctrl+Z precisa de mais de um toque; a escrita fica lenta; ou a barra de baixo diz «Code assistant not installed» (aí falta o assistente na máquina — os passos 3–5 e 8 funcionam na mesma, o 6 e o 7 não).

## 9. ABERTO

- As acções genéricas do servidor (Extract/Inline) ficam para o **I10** (são refactors de selecção).
- «Add type» ao `self` (o `1030`): o servidor não dá edição.
- A mensagem crua do `1002` (567 caracteres) é do **I8**.
- A janelinha só abre quando TODAS as respostas chegam (servidor lento = espera); sem servidor, só as
  nossas, já.
- Ctrl+rato em mão (I15) continua do I6. Não fotografado (foto e cena `PH2D_CODE_SMOKE=3` são do I15).
- `ph2d-code-lsp`: o filtro `client config` casou 0 testes (ver §6).

## 10. Próximo

**I8** (explicações escritas à mão por classe do corpus do I0c, PT/EN, `ph2d-i18n`; a mensagem do `1002`
reescrita) → **I10** → I9/I12/I14 → I13 → I15 (ordem do [03_onda2.md](../03_onda2.md)).
