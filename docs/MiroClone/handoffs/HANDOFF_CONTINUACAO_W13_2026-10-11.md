# CONTINUAÇÃO — `line/MiroClone` W13 «O histórico das votações» (2026-10-11)

> Para a janela que ABRE a W13. Rota `/pd-linha-assumir`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. A **W12 fechou** (smoke OK do dono, 11/10) mas **NÃO está integrada**: os commits dela estão na
> linha (`cf254345d` … `4e0283c19` + este doc) — [handoff W12](HANDOFF_INTEGRACAO_line_MiroClone_W12_2026-10-11.md).
> Retomada: `git cherry main HEAD | grep -c '^+'` → **> 0** (a W12 por integrar): `git rebase main` só se o `main`
> andou; **= 0** (o integrador já a levou): `git reset --keep main` (nunca `rebase`). A W13 vai POR CIMA da W12 na
> mesma branch, e o handoff da W13 **substitui** o da W12 (integra-se tudo de uma vez).

## 1. A escolha do dono (11/10)

Depois do smoke da W12 o dono perguntou porque é que os votos antigos «desaparecem» ao votar de novo (hoje o
quadro guarda UM resultado: «New vote» substitui o anterior) e escolheu: **o histórico das votações**, como o
Miro — cada votação guardada com nome e data, um painel que lista as anteriores e o resultado de cada uma —
**mais as duas pontas abertas da W12** (§2 abaixo).

## 2. O que já existe (não reconstruir — confira o código antes de acreditar numa ausência)

- **Votar num ecrã** (W7): `ph2d-board-edit/src/vote.rs` (`VoteSession`, `end_vote`/`clear_votes` num passo de
  desfazer), o resultado POR ELEMENTO em `ph2d-board-model` (`Org::votes`, `Field::Votes`), o painel
  `screens/hero/board_facilitate*.rs` (`VotePanel::{Setup, Session, Results{all}}`, «See all», «New vote»,
  «Clear results»), o selo no quadro (`ph2d-board-render/src/facilitate.rs`, escondido por `hide_votes` durante
  uma votação). Cena 9.
- **Votar entre pessoas** (W12, [ADR-0182](../../architecture/decisions/0182-o-que-e-de-uma-pessoa-mora-no-cofre-do-anfitriao.md)
  §7): o boletim só vai ao cofre do anfitrião (`ph2d-board-sync/src/{vault.rs,session_chair.rs}`), o fim escreve
  `Votes` em cada alvo como **escrita da sala** (não entra no desfazer de ninguém); a tela segue a rodada
  (`board_facilitate_live.rs`). Cena 13.
- **A equipa no ficheiro e ao vivo** (W8/W11): comentários e VERSÕES vivem em `Board.team` (`ph2d-board-model/src/team.rs`)
  e viajam por `TeamWrite` (`ph2d-board-sync/src/team.rs`) — o molde natural para um registo de votações que é do
  quadro, persiste e se sincroniza. `FORMAT_VERSION` = 10 (a W11 subiu 9 → 10 com leitor do 9 e a fixture
  `fixtures/format_v9.bin` gravada ANTES — o mesmo idioma se o formato mudar).
- **O Miro medido** (`docs/MiroClone/ferramentas/facilitacao_miro_notas.md` §2 e §6): sessões guardadas por
  nome/data; «Results» com o top-3 e «See all results»; o separador do histórico («Your voting history and other
  sessions' results»); toggle «Show results on the board» (o selo de UMA sessão no quadro); renomear e apagar;
  **não exporta**; **não vai na cópia do quadro** (*«If I duplicate my board… No»*); reiniciar = acabar e começar
  outra.
- **Oráculo instalado**: o Scrumlr v5.4.2 (MIT) corre em 127.0.0.1 sem docker —
  `docs/MiroClone/ferramentas/vote_private_oracle/README.md` (`start.sh`/`stop.sh`; instalação em
  `~/Apps/scrumlr-oracle/`). O Scrumlr guarda VÁRIAS votações por quadro: corra-o para «o que fica de cada uma».

**As pontas da W12** (handoff W12 §9, decisões registadas que o dono mandou fechar):
1. Com «Anonymous names», os nomes ainda aparecem na lista de pessoas e nos avatares do painel Live, nos avisos
   «X joined/left» e nas linhas `used` de quem iniciou (o ADR-0182 §8 só cobre cursores, selecções e seguir).
2. Quem não lê uma nota escondida ainda vê o texto ANTIGO dela na busca `Ctrl+F`
   (`board_nav_search.rs`), no `row_text` dos resultados da votação e na exportação (`ph2d-board-export`, que
   desenha com `RenderCache::default()`) — nunca o novo, mas é um texto que já não existe.

## 3. Por ordem

0. **Oráculo** (CLAUDE.md §0.9): releia o help VO do Miro pela API Zendesk (como a W12 fez:
   `help.miro.com/api/v2/help_center/en-us/articles/360017572274.json`) e meça as figuras do separador do histórico;
   corra o Scrumlr com 3 votações seguidas num quadro (casos: o que fica de cada uma · apagar uma · uma nota apagada
   depois da votação · quem entra depois vê o histórico? · o resultado de qual aparece no quadro). Grave a saída
   com cabeçalho, duas corridas iguais. Casos nossos a decidir com medição: o histórico é do QUADRO (vai no ficheiro
   e na rede) — mas não na cópia do quadro (Miro); uma votação de UM ecrã e uma da sala entram no MESMO histórico;
   «Clear results» apaga o selo ou a sessão?; o nome por omissão (data/hora? «Vote 3»?); desfazer.
1. **Desenho + ADR** (emenda à 0182 ou ADR novo — o número conta-se contra o `main` do dia): onde mora a sessão
   (`Board.team`?), o que viaja (`TeamWrite` novo no FIM), o formato do ficheiro (se sobe, leitor do 10 e fixture
   gravada ANTES), e o que o selo do quadro passa a mostrar (a sessão escolhida em «Show results on the board»).
2. **Construir**: modelo/sync puros com testes (convergência a três com quedas, como `fac_converge_tests.rs`); o
   painel (o separador do histórico, renomear, apagar, «Show results on the board»); i18n; as duas pontas da W12
   com gate; **cena 14** (`PH2D_BOARD_SMOKE=14`: roteador em `crates/ph2d-app-board/src/smoke.rs`, `max_level`
   13 → 14) com gate que a conduz e **fotos dos estados** à mão com a placa (molde:
   `fotos_dos_estados_da_cena_13` em `smoke_fac_live_tests.rs`, o fotógrafo em `smoke_shots_tests.rs`).
3. **`/pd-linha-fechar`**: portão batched 1× (inclui `censos-da-arvore-combinada.sh`, `cargo machete`,
   `adr-index.sh --check`), auditoria de 2 lentes (DADOS: o que se perde/diverge no histórico, no ficheiro e na
   rede; INTERFACE), mutação (agente `mutacao`), handoff de integração W12+W13, §5.1, binário smoke quente. Não
   integra, não pusha.

## 4. Fora de escopo

A internet (o transporte virá atrás da mesma porta). Exportar o histórico (o Miro não exporta).

## 5. Lembretes MEDIDOS nesta linha (W12, 11/10)

- **Fotografe os estados ANTES do fecho, pelo caminho REAL**: as fotos da cena 13 acharam 5 defeitos que nenhum gate
  via (o Live tapava o painel; textos cortados). E um teste que manda o pedido por FORA da tela (`ask_live` directo)
  fotografa um estado que o produto nunca mostra — clique o `+` de verdade (`Hand::plus` em `smoke_fac_live_tests.rs`).
- **A gravação da cena (`VectorScene::record`) NÃO vê o texto dos painéis** (desenham por fora dela): prove o estado
  de um painel pela função que o decide (ex.: `board_private_paint::view`), nunca procurando a frase na gravação.
- **Uma régua de largura sem controlo passa sempre** (sem fontes mediria zero): `every_w12_panel_line_fits_the_column`
  tem o controlo da frase que a foto mostrou cortada — copie o idioma.
- **O tecto de 700 vale para ficheiros de teste montados por `#[path]`** (`fac_tests.rs` 900 reprovou): parta em
  filhos (`fac_audit_tests.rs`, `fac_converge_tests.rs`).
- **Um `mod` de TESTE com o nome de um ficheiro de produção (`mod live`) cegou o censo `node_id_collisions`**.
- O nome do módulo de teste não é o do ficheiro: `live::fac::tests`, `smoke::fac_live::tests`,
  `board_bar::tests::facilitate::fac_live` — um filtro que casa zero diz «0 passaram».
- A prova de mutação achou 3 sobreviventes (corridas e o defeito das fotos) que a auditoria não viu — escreva o
  gate das CORRIDAS (a virada, o selado perdido no fio) antes da mutação, não depois.
- `git commit -- <paths>` não apanha ficheiros novos: `git add --` primeiro. O shell é **zsh**: `$P` com vários
  caminhos não se parte — escreva-os por extenso.
- Clippy `-D warnings`: `large_enum_variant` (ponha o grande numa `Box`), `type_complexity` (um `type`).

## 6. Ao dono

PT-BR, curto, sem jargão; smoke em passos numerados (comando com `cd` · onde clicar · o que tem de acontecer ·
como saber que deu errado); duas janelas no mesmo computador como nas cenas 12 e 13.
