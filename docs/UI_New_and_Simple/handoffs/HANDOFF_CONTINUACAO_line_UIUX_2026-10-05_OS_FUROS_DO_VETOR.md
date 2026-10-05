# Continuação — `line/UIUX`, 2026-10-05 — os FUROS do Vector (Soldar · caneta no Edit · Shift no canvas · Width/Trim)

> Leitor: a próxima janela desta linha. O bloco de entrada é o do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd Worktrees/line-UIUX && pwd && git branch --show-current` ANTES de ler. HEAD esperado = o commit
> deste ficheiro; árvore limpa; base (merge-base = `main`) `5d596eaaf`; nada integrado desde a rodada
> de 04/10 (`git cherry main HEAD` = 15 `+` com este ficheiro).

## §1 — Estado

- A linha foi integrada a 04/10 e REABERTA a 05/10 depois da poda do 3D (ADR-0179). Desde então:
  spec/06 em dia com a poda, e a **2.ª volta do Vector** — *cada forma é um objecto*, o contentor
  `VecObject` saiu, *Add ▸ Vector Drawing* só arma a ferramenta, a forma que nasce pede o Edit, a
  booleana deixa a forma em Edit COM gizmo, desseleccionar não sai do modo. **Smoke do dono:
  APROVADO (05/10).** Tudo em
  [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-05_CADA_FORMA_E_UM_OBJECTO.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-05_CADA_FORMA_E_UM_OBJECTO.md)
  — leia o §1, o §5b e o §6 dele ANTES de tocar no `vector_mode`.
- As leis do Edit do vetor hoje (`crates/ph2d-app-vec/src/vector_mode.rs`, `ModeFamily` em
  `crates/ph2d-editor-core/src/screens/hero/mode_drive.rs`):
  - `newborn`: UMA forma nova, fora de gesto, com a ferramenta na mão ⇒ pede o Edit (e o Edit PASSA
    a ela);
  - o `follow` larga a ferramenta quando o Edit acaba, MENOS quando todas as formas do Edit
    DESAPARECERAM (a forma que as substitui só ganha entidade no quadro seguinte);
  - as formas que a CANETA selecciona num Edit juntam-se a ele (`follow` + `parts`); o `enter_with`
    poda a caneta às formas do Edit;
  - o `wants` de uma família selecciona SÓ a entidade dela; desseleccionar não sai de modo nenhum
    (`ModeState::still_holds`); o gizmo de objecto mostra-se sobre a selecção no Edit do vetor.

## §2 — A TAREFA (ordem do dono, 05/10: *«vamos corrigir os furos»*)

Os quatro furos do vetor, MEDIDOS antes de cada cura (CLAUDE.md §0.10: junte a família e meça de
uma vez; um plano, um lote de binários, uma rodada de fotos):

1. **O *Soldar* (Weld) em Edit.** Registado a 04/10 (`…_O_VETOR.md` §7a): *«CONSOME as formas e cria
   uma nova (`weld.rs`, decisão do dono 31/08) ⇒ o Edit cai para Object com a rede seleccionada»*.
   Porta: o botão `ph2d_panel_vector::ids::VECTOR_PATH_WELD` (texto «Weld»); código
   `crates/ph2d-app-vec/src/weld.rs` e `crates/ph2d-vec-scene/src/weld.rs`. ⚠️ A cura da booleana de
   05/10 pode já o cobrir — MEÇA: quantas formas o Weld deixa (uma? uma rede de várias? um grupo?) e
   o que o `newborn` faz com isso (ele só pede o Edit com UMA forma nova). O esperado do dono, pela
   booleana: *a forma soldada fica seleccionada, em Edit, com o gizmo*.
2. **A caneta DENTRO do Edit a acrescentar à forma** (`…_O_VETOR.md` §7b: *«como o Figma não existe —
   cada traço é um objecto»*). Em Edit com a Pen, premir a PONTA de um caminho aberto do Edit deve
   continuar ESSE caminho em vez de nascer outro objecto. MEÇA primeiro o que o `PenTool`
   (`crates/ph2d-vec-edit`) já faz com um press na ponta de um caminho seleccionado — pode já existir
   (continuar o caminho) e só a lei `newborn` o estragar, ou não existir. Outro app já faz isto? É
   ORÁCULO que se corre (CLAUDE.md §0.9, triagem de licença primeiro).
3. **Juntar outra forma ao Edit pelo canvas.** Hoje em Edit a ferramenta só agarra as formas do Edit
   (`VecViewState::editing`); juntar outra é Ctrl+clique na Hierarquia + `Tab`. ⛔ **Decisão de
   PRODUTO — pergunte ao dono ANTES de codar** (`AskUserQuestion`, com o que mediu): (a) Shift+clique
   numa forma fora do Edit junta-a ao Edit; (b) fica como está (só pela Hierarquia). Recomendação:
   (a) — a caneta já junta ao Edit as formas que selecciona, e o Shift+clique é o gesto que o
   artista tenta primeiro.
4. **O press do Width e do Trim pela shell sem gate de condução** (`…_O_VETOR.md` §7c, NÃO-CHECADO:
   `despacho_clique_vetor_premido.rs` → `path_at`). Escreva o gate que conduz um press REAL de Width
   e de Trim, em Edit, até à ferramenta (a forma do Edit responde; a de fora não).

Se sobrar janela: a prova automática das teclas `Tab`/`Ctrl+Tab`/`Ctrl+Space` (hoje só o smoke as vê;
o `Tab`/`Ctrl+Tab` já passam por `mode_drive::mode_key`).

## §3 — Restrições (medidas no HEAD deste ficheiro)

- Shell (`the_shell_only_shrinks`): **185 056 / 189 041** (folga 3 985). Código de família na crate dela.
- `PROJECT_SCHEMA` = **184**; registos ECS 105, `ph2d-render`/`ph2d-script` 106. Cenas
  `PH2D_OBJECT_MODE_SMOKE` = 1 · 4 · 6 (a 6 é Add ▸ Vector Drawing + rectângulo + elipse + UNION).
- Contrato congelado: nenhum dos quatro furos toca o §6 do CLAUDE.md (o `ph2d-vector-doc` não entra).

## §4 — Lições desta linha que valem aqui

- ⛔⛔ **O programa real espalha um gesto por VÁRIOS quadros** (a forma da booleana só ganha entidade no
  quadro seguinte à remoção das outras): um gate que faz tudo num quadro passa e a foto não. Gate do
  Edit = dois quadros, pelo caminho que a shell usa (a caneta selecciona → a shell copia para a
  selecção).
- ⛔ **A foto apanha o que o gate não vê** — 4× nesta linha. Os cliques não se fotografam (XTest
  ignorado na tela virtual): prove-os por gate; a cena `PH2D_OBJECT_MODE_SMOKE=6` serve de base.
- ⚠️ Um passo de cena de smoke NÃO corre no quadro que traz um pedido de modo (o `or_else` do
  `fase_object_mode`): espere pelo efeito (o espelho da ferramenta activa), não pelo estado efémero.
- ⚠️ Para medir no programa real: `eprintln!` temporário na família + `fotografa_cena.sh` + `grep` do
  `.log`; retire-o e confirme `grep -c` = 0 antes do commit.
- ⚠️ Mensagem de commit com «cargo test» dispara o guarda de comando pesado — use `git commit -F
  <ficheiro>`. Em zsh, `git show :$n:caminho` vira modificador `:c` — escreva `":${n}:caminho"`.

## §5 — Como a onda fecha

DIRETRIZ §1.5.9: gate batched 1× desde `5d596eaaf` (agente `verificador`: `nextest-impacted`,
`CARGO_BUILD_WARNINGS=deny` check, clippy `-D warnings`, censos da árvore), mutação dos gates novos
(agente `mutacao`, controlo verde com população > 0), fotos, um handoff de integração NOVO que cita o
`…_CADA_FORMA_E_UM_OBJECTO.md`, `rm -rf target/*/incremental`, e por último o build de smoke 2× com a
2.ª saída colada. Ao dono: passos numerados, sem jargão. ⛔ Não integrar nem pushar.
