# Regras de todo agente da missão Painter 4K (10/11) — leia antes do primeiro comando

Missão: [`PROMPT_MISSAO_2026-11-10g_painter_4k_autonomo.md`](../handoffs/PROMPT_MISSAO_2026-11-10g_painter_4k_autonomo.md).
Estado: [`BACKLOG.md`](BACKLOG.md) (a verdade; o `git log` é a outra). Fontes extraídas dos estudos: [`fontes/`](fontes/).

## Onde e como

- Worktree: `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
  Caminhos ABSOLUTOS sempre. Antes de commitar: `git -C <worktree> branch --show-current` tem de dar `line/PainterWatercolor`.
- **Zero perguntas, zero paragens.** Dúvida de produto → a recomendação do estudo; técnica → padrão-ouro. Registe a decisão
  e o porquê no seu resultado.
- **Formas proibidas** (param a missão num pedido de permissão ou fazem estrago): `cd X && …` em comando composto ·
  `bash -c` / `sh -c` · `rm -rf` com variável ou glob · `kill` de PID que não é seu (NUNCA o pai de um órfão: é o logout do
  dono) · escrever em `.git/`, `.claude/` ou rc da shell · laço de espera sobre `pgrep -f` · vigia de fundo sem `timeout` ·
  `git stash` · `git add -A`/`.` · `--force` · push · rebase · `PH2D_GPU=1` (a RTX está PROIBIDA nesta missão).

## Compilar, testar, medir

- Inner loop: `bash scripts/cargo-check-narrow.sh <crate>` (corra-o da worktree, caminho absoluto do script).
- Teste POR NOME: `bash scripts/cargo-test-narrow.sh <crate> <nome>` (exit 0 verde · 1 vermelho · 2 não compilou · 4 nome
  não casou). Crate inteira sem nome é recusada pelo guarda. Nada de clippy (é do fecho). Nada de `cargo test --release`.
- Pesado (build do app, sondas, Xvfb): `bash scripts/ph2d-run.sh <cmd>`; perfil `--profile smoke`
  (`CARGO_PROFILE_SMOKE_STRIP=none` quando for para o `perf`). Medir: variantes no MESMO processo, intercaladas, o MÍNIMO
  (`docs/DevOps/MEDIR_VELOCIDADE.md`). No app real: `docs/Painter/ferramentas/mede_pen_up_4k.{sh,py}`,
  `mede_rato_xvfb.sh` (estenda-os; não escreva outros). Sonda sem o papel/relevo do dono não tem o fenómeno.
- **Outros agentes trabalham EM PARALELO nesta mesma worktree** (outras crates/ficheiros). O cargo serializa os builds
  («Blocking waiting for file lock» é normal). Se a compilação quebrar num ficheiro que NÃO é seu: não o conserte; volte a
  correr o `check` daí a pouco; se persistir, diga-o no resultado. Mantenha as SUAS edições sempre a compilar (passos
  pequenos). `index.lock` no commit = outro agente a commitar: tente de novo.

## Curar

- **Red-first:** o defeito/regressão tem de ser VISTO vermelho (teste ou medição) antes da cura.
- Cura que NÃO deve mudar o desenho: **paridade ao bit** contra o caminho antigo guardado verbatim como oráculo
  `#[cfg(test)]` (padrão de `as_faixas_dao_o_passe_por_pixel_ao_bit`). Cura que muda: prancha antes/depois em
  `docs/Painter/missao_4k/pranchas/<id>/` (3 fundos: branco, blocos de cor, papel Textured com relevo; 1080p e 4K).
- Mudou motor, preset ou pincel ⇒ por nome, `a_largura_de_fabrica_e_a_aprovada` e
  `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (`ph2d-tool-painter`). **Vermelho FICA vermelho: NUNCA regrave**
  miniaturas/larguras; anote no resultado quais pincéis mudaram.
- rayon: nada de espera bloqueante dentro de job do pool (Bug #66 = deadlock); paralelo só com ordem determinística (o mesmo
  bit em qualquer nº de fios); uso novo do rayon fora das exceções existentes leva ADR (o número conta-se:
  `ls docs/architecture/decisions | tail`).
- Widget partilhado (cartões, `pair`/`compact`, chrome, `ph2d-editor-core`) varre-se POR CONSUMIDOR (teste de cada crate
  que o usa).
- UI: zero hex, zero `f32` literal de UI, toda string por `ph2d-i18n`, só em inglês (Plain English nas dicas).
- Formato gravado mudou ⇒ suba o `PROJECT_SCHEMA` (`python3 scripts/schema-recount.py`); sem migração nem leitor velho.
- Tectos: 700 linhas por ficheiro `.rs` (parta, nunca isente); `ph2d-app-painter` ≤ 24 429 linhas (`the_shell_only_shrinks`:
  cure por MOVER/cortar); comentário curto (o porquê histórico vai para o doc).

## Fechar o seu item

1. Commit por item: `git add -- <paths>` e `git commit --no-verify -m "<msg>" -- <paths>`. Mensagem técnica: o quê, os
   números antes → depois, o gate (nome) e a prova red → green. Última linha:
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Só os SEUS ficheiros.
2. No mesmo commit, a(s) linha(s) do seu item no `BACKLOG.md` (ferramenta Edit; só as suas linhas): medição depois, sha
   (escreva `este commit`), estado `feito` / `recusa medida (<porquê>)` / `parcial (<o que falta>)`.
3. Máquina limpa: nenhum processo seu vivo (`pgrep -af 'ph2d|Xvfb'`).
4. Devolva ≤ 1 500 caracteres: o que mudou, números antes → depois, gate e prova, sha, o que ficou por fazer.
5. Prazo: 45 min impostos (aviso aos 40). Corte o trabalho para caber; o que não couber, devolva como «parcial» com o
   próximo passo exato.
