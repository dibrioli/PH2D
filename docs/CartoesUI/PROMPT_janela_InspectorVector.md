# Prompt — janela nova: porte das famílias 6–10 (Inspector, estilos de cor do Vector, cola da shell)

Cole isto numa janela NOVA. Continua a linha `line/NovaUI` num worktree NOVO, `line/NovaUI-Inspector`.

## Quem é quem
- **Enio** é o dono do produto, não um engenheiro. Respostas a ele: **curtas, PT-BR, sem jargão**.
- **Nunca** faça push nem integre. O app é **só em inglês**.
- Ordem do dono: **não pare para perguntar**; trabalhe sozinho até ao fim, smoke só no fim. Dúvida de produto →
  a opção mais próxima das fotos aprovadas, **escrita no handoff**.

## Preparação
```
cd /home/enio/Documentos/Projetos/PH2D && git worktree add -b line/NovaUI-Inspector Worktrees/line-NovaUI-Inspector line/NovaUI && cd Worktrees/line-NovaUI-Inspector && pwd && git branch --show-current
```
`CARGO_TARGET_DIR` próprio. Leia: `docs/CartoesUI/06_COMO_PORTAR_PARA_O_MAIN.md` (§3 famílias 6–10) · `05` (§0, §8, §9) · `07` §7 ·
`handoffs/HANDOFF_line_TrelloUITeste_2026-11-02_FECHO.md` · `08` · `handoffs/HANDOFF_line_NovaUI_2026-11-03_GALERIA.md`
(⚠️ §1.3: o par `publish_object_mode/object_mode` foi removido) · `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` ·
`docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md`.

## Missão
Palavras do dono (03/11): *«Pode seguir com o trabalho nos paineis do app. Comece pelos inspectors de Vector e
Bones... Já funcionam na linha de testes da UI»*. Esta janela faz o **Inspector e o Vector**; o Bones é outra
(`PROMPT_janela_Bones.md`). Portar com `git diff 4e648ec2b4 line/TrelloUITeste -- <paths> > patch && git apply -3 patch`,
família a família (aplicar → `cargo-check-narrow` → testes por nome → commit; um commit por família):

6. **Inspector em cartões** — `ph2d-panel-inspector` (`cards.rs`, `cards_componentes.rs`, `retrato.rs`,
   `sections/rows*.rs`, `event.rs` → `com_retrato`). ⚠️ `cards.rs` tem 791 linhas e a fn `cards` 305: **parta por
   responsabilidade** antes de portar. **Redesenhe o modo da D25b («+» no topo) sem o campo removido do store** (DAG):
   p.ex. o `mode_drive` publica um bool `at_top` definido em `interaction`.
7. **Estilos de cor do Vector (D28–D32)** — `ph2d-vec-scene` (esquema: o número do degrau **recontado** por
   `python3 scripts/schema-recount.py`, nunca copiado), `ph2d-app-vec`, `ph2d-panel-vector`, `ph2d-tool-vector`;
   testes de projecto da shell (`project_schema*`).
8. **A cola na shell** (`render_loop/snapshots_inspector*`, fases de retrato/alfinete): lógica vai para crate
   ([HOWTO](../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md)); **saldo da shell ≤ 0**.
9. **Cenas de smoke** — `ph2d-app-vec/src/smoke_cartoes_inspector.rs`, `PH2D_VEC_CARTOES_SMOKE=1..7`,
   `PH2D_CARTAO_CORDA_SMOKE`; o número da próxima cena conta-se no roteador da família.
10. **Testes** — `ph2d-panel-registry-init/tests/it/` (`o_*`, `os_*`, `toda_dica_dos_cartoes`,
    `a_lista_de_cartoes_flutua_no_hero`), entram com a família que testam.

### Conflitos com o main
Decisões do dono que entraram no `main` depois de 29/10 **ganham** sobre a linha de teste onde diferem — salvo
onde a versão da linha de teste é a que o dono acabou de mostrar (Effects/Filters F43, Fill/Stroke F44, Advanced à
Figma, rótulos, unidades). A foto do dono do Inspector da linha de teste é o alvo para Transform/Appearance/
Fill/Stroke. **Anote cada conflito no handoff; não pare.**

### ⚠️ Defeito visível na foto do dono, a curar
Na coluna **ESTREITA** as linhas Fill/Stroke elidem o hex e a opacidade para «...» e o rótulo «Opacity (%)» quebra.
Abaixo da largura em que cabe, a linha amostra+hex vai para a **sua própria linha** e Opacity + olho + menos vão
para a seguinte, como o cartão Fill da Widget Gallery (`crates/ph2d-panel-widget-gallery`). É uma **lei do
compositor de linha**, com teste na coluna mais estreita (e a verificação a vermelho).

### Se sobrar tempo
Cure a família dos defeitos do cartão Camera (05 §8–§9, 07 §7); o cartão Camera da galeria é o modelo.

## Escala e divisão
Não cabe tudo num agente de 45 min: **um agente por família**, commite cada uma. Janela interrompida → deixe um
prompt de continuação. Delegue leitura e verificação (`explorador`, `verificador`, `mutacao`, `documentador`).

## Para não colidir com a linha Bones
Bones possui `ph2d-panel-skeleton`, `ph2d-app-skeleton`, `ph2d-skeleton-live`; esta linha possui o resto. A cola
partilhada (`render_loop/snapshots_inspector*`, retrato/alfinete) é **desta** linha; a do Bones pára e anota no
handoff o que precisa dela.

## Regras
Inner loop `bash scripts/cargo-check-narrow.sh <crate>`; testes por nome; pesado por `bash scripts/ph2d-run.sh <cmd>`;
fotos por `docs/Components/ferramentas/fotografa_cena.sh` (nunca `spectacle`); clique real com verificação a
vermelho; ficheiros ≤ 600/700 linhas; zero hex / string hardcoded (HR-15). Suíte completa só na integração.
Gates 1× no fecho, como na galeria (handoff da galeria §4/§7): `o_*`, `os_*`, `toda_dica_dos_cartoes`,
`a_lista_de_cartoes_flutua_no_hero`, DAG, LOC, a11y/tooltips, `the_shell_only_shrinks`, fmt/clippy das crates tocadas.

## Fecho
Fotos lado a lado (aprovada × portada), handoff `docs/CartoesUI/handoffs/HANDOFF_line_NovaUI-Inspector_<data>.md`,
build `--profile smoke` compilado; ao Enio, relatório curto com smoke em passos numerados
(`PH2D_VEC_CARTOES_SMOKE=1..7`: comando completo com `cd` · onde clicar · o que tem de acontecer · como saber que
deu errado). Depois PARE.
