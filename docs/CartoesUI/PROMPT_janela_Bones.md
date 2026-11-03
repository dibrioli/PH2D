# Prompt — janela nova: porte da família 5 (Bones) para os cartões

Cole isto numa janela NOVA. Continua a linha `line/NovaUI` num worktree NOVO.

## Quem é quem
- **Enio** é o dono do produto, não um engenheiro. Ele decide e testa. Toda resposta a ele: **curta, PT-BR, sem
  jargão** (nada de gate, crate, ADR, nome de função). Diga o que ele consegue FAZER agora.
- **Nunca** faça push nem integre. A linha acaba no handoff e PARA.
- O app é **só em inglês** (toda string de UI, dica e ajuda em Plain English).
- Ordem do dono: **não pare para perguntar**. Trabalhe sozinho até ao fim; o smoke é só no fim. Dúvida de
  produto → escolha a opção mais próxima das fotos aprovadas e **escreva-a no handoff**.

## Preparação (comandos exactos)
```
cd /home/enio/Documentos/Projetos/PH2D && git worktree add -b line/NovaUI-Bones Worktrees/line-NovaUI-Bones line/NovaUI && cd Worktrees/line-NovaUI-Bones && pwd && git branch --show-current
```
Use o **seu próprio** `CARGO_TARGET_DIR` (nunca partilhe target entre worktrees). Leia, por esta ordem:
`CLAUDE.md` (já carregado) · `docs/CartoesUI/06_COMO_PORTAR_PARA_O_MAIN.md` (§3, família 5) ·
`05_ESPECIFICACAO_UI_COMPLETA.md` · `handoffs/HANDOFF_line_TrelloUITeste_2026-11-02_FECHO.md` · `08_galeria_em_cartoes.md` ·
`handoffs/HANDOFF_line_NovaUI_2026-11-03_GALERIA.md` · `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`.

## Missão
Palavras do dono (03/11): *«Pode seguir com o trabalho nos paineis do app. Comece pelos inspectors de Vector e
Bones... Já funcionam na linha de testes da UI»*. Esta janela faz **Bones**; o Vector é outra janela
([PROMPT_janela_InspectorVector.md](PROMPT_janela_InspectorVector.md)).

Porte a **família 5**: `ph2d-panel-skeleton` (`cards.rs`, `paint.rs`, `publica_presos`) + `ph2d-app-skeleton` +
`ph2d-skeleton-live`, vindos de `line/TrelloUITeste`:
```
git diff 4e648ec2b4 line/TrelloUITeste -- <paths> > patch && git apply -3 patch
```
Onde o `main`/a linha mudou o mesmo código, resolva à mão com o teste da família como juiz; a proveniência vai no commit.
- Smoke: `PH2D_VEC_BONE_SMOKE=8` (+ `PH2D_CARTAO_CORDA_SMOKE=1` com ele).
- Fotos de referência: `docs/CartoesUI/maquete/bones.png` (alvo), `antes_painel_bones.png` (o «antes») e a imagem do dono.
- Cartões: **Tool** (Create/Transform/Weight), **Bone** (Length, Segments; ↺ guardar espelho no cabeçalho),
  **Angle Limit**, **Smart Bone**, **IK**.
- ⚠️ **Defeito visível na foto do dono, a curar:** em **Smart Bone**, as linhas **Object** e **Action** continuam no
  estilo antigo (o nome à esquerda noutro tom, chip fora do padrão). Cura: **rótulo por cima** e dropdown/campo
  no **padrão do cartão** (como o Fill da galeria). Prove com teste e foto.

## Regras
- **Saldo da shell ≤ 0** (catraca `the_shell_only_shrinks`): lógica da shell vai para crate. Ficheiros ≤ 600/700
  linhas. Cola partilhada com o Inspector (`render_loop/snapshots_inspector*`, fases de retrato/alfinete) é da
  linha do Inspector: **não a toque**; pare nessa parte e escreva no handoff o que precisa dela.
- Esta linha possui `ph2d-panel-skeleton`, `ph2d-app-skeleton`, `ph2d-skeleton-live`. O resto é da linha do Inspector.
- Inner loop = `bash scripts/cargo-check-narrow.sh <crate>`; testes **por nome** (`cargo-test-narrow.sh`); pesado por
  `bash scripts/ph2d-run.sh <cmd>`. Suíte completa e impactados só na integração.
- Delegue leitura e verificação aos agentes (`explorador`, `verificador`, `mutacao`, `documentador`); a janela principal decide.
- Fotos **sempre** por `docs/Components/ferramentas/fotografa_cena.sh` (nunca `spectacle`).
- Teste por **cliques reais** com a verificação a vermelho (o teste tem de falhar sem a cura).
- Gates a correr 1× no fecho: os da galeria (ver o handoff da galeria §4 e §7) — contagens de `ph2d-panel-registry-init`
  (`o_*`, `os_*`, `toda_dica_dos_cartoes`), censos de a11y/tooltips, DAG da fundação, tectos de LOC, fmt/clippy das crates tocadas.
- Teste por nome: `a_lista_de_cartoes_flutua_no_hero`; todo ícone sem texto tem dica.

## Fecho
1. Fotos lado a lado (aprovada × portada) em `docs/CartoesUI/maquete/`.
2. Handoff `docs/CartoesUI/handoffs/HANDOFF_line_NovaUI-Bones_<data>.md` (superfície de colisão, o que fica aberto,
   o que precisa da linha do Inspector, decisões de produto tomadas por si).
3. Build de smoke **compilado** (`--profile smoke`).
4. Ao Enio: relatório curto com o smoke em **passos numerados** (comando completo com `cd` · onde clicar · o que tem
   de acontecer · como saber que deu errado). Depois PARE.
