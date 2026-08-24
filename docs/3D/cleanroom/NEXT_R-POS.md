═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · PAPEL R — REVISOR            (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Modo: PÓS · Módulo: 3D (quad remesh) · Alvo: extração de malha quad
Ledger: docs/3D/cleanroom/LEDGER_quadwild.md

Você é o REVISOR: pode ver OS DOIS lados (o fonte do alvo e o nosso
código). Você NÃO escreve nem dita código de produto. Seus achados
voltam ao Implementador em termos FUNCIONAIS, nunca com trecho do
original, e nunca por mensagem direta — via emenda/handoff.

Leia: SKILL_Cleanroom §7.

Modo PÓS (após paridade verde):
1. Paridade: gates verdes, barra derivada, fase a fase onde há dumps.
2. Sweep total (§7.2): árvore rastreada + --git-history (mensagens e
   patches, incl. cleanroom/ e project-memory/) + linha do CLAUDE.md
   §5 + handoff. ZERO hits é a barra. Recomendado: sweep no
   transcript da janela I.
3. Revisão estrutural: convergência de EXPRESSÃO (decomposição
   arbitrária igual, ordem não-forçada, nomes traduzidos) —
   comportamento igual NÃO é achado, é o objetivo. Achado →
   re-derivação com restrição funcional explícita (§7.3.d).
4. Incidentes: cada um do INBOX transcrito e tratado (quarentena
   comparada; régua do "substancial" §6.2)?
5. Session-id de I fora de {janelas E, queimadas}?
6. Feche o ledger com o bloco de fechamento (§6). Reporte:
   "Ledger fechado. Módulo apto a integrar."
═══════════════════════════════════════════════════════════════════

O QUE A JANELA I ENTREGOU (os factos de que você precisa)

· Branch `line/quadextract`, 5 commits. ⚠️ O HEAD lê-se com
  `git rev-parse line/quadextract` — o último commit é o dos docs, e um
  sha escrito dentro dele nunca poderia ser o dele próprio.
  ⛔ Base do fork: `line/sculpt3d`, NÃO `main`.
· Session-id de I, declarado por append cego no INBOX no Passo 0:
  186ce13e-479b-467a-904c-0ff087ab76c9   (2026-08-24)
· ⭐ ZERO incidentes. Nenhuma busca na web, nenhum subagente, nenhum
  SendMessage. As fontes desta janela foram: a SPEC, os fixtures e o
  verificador de `cleanroom/fixtures/`, o código do PH2D, e a
  SKILL_Cleanroom (lida no primário, §3.I + §9 + §6). ⛔ O LEDGER e a
  VASSOURA nunca foram abertos — o `.claude/settings.local.json` do
  Passo 0 tem os quatro `deny`, incluindo o de
  `ph2d-quadbench/oracle/**` que o R-pré acrescentou.
· Nada de `~/Referencias/**` nem do arnês do oráculo foi lido, corrido
  ou compilado.

ONDE ESTÁ O CÓDIGO E OS GATES
· `crates/ph2d-quadextract/` — a extracção (§2–§6). 15 gates:
    tests/gates_exact.rs · gates_fixtures.rs · gates_precision.rs ·
    measure_quad_shape.rs
· `crates/ph2d-gridmap/src/round.rs` — o §5. 3 gates + 2 sondas
  (`--ignored`), em `round_tests.rs`.
· `shells/desktop/src/sculpt3d_history_retopo_extract.rs` — o botão,
  DESLIGADO. 2 gates (o do interruptor e o que CONTA a bifurcação).
· Handoff da casa (técnico, com as tabelas):
  docs/3D/handoffs/HANDOFF_INTEGRACAO_line_quadextract_2026-08-24.md

⭐ AS CINCO DIVERGÊNCIAS DELIBERADAS DA ESPEC (§9 do handoff) — é aqui
  que a sua revisão estrutural deve morder primeiro:
  1. Sem `num-bigint`/`num-rational`: a truncagem do §2.3 numa grade
     GLOBAL põe o domínio em `i64` e a orientação num `i128` exacto.
     A espec SUGERIA bigint+filtro; o requisito («exacto, inclusive
     quando é zero») é entregue por outra via, mais forte.
  2. O gate nº5 muda de FORMA (não há filtro para «desistir»): ele
     prova que o predicado acerta onde o `f64` erra.
  3. O degrau 3 da escada (factorização directa) NÃO foi construído —
     `RoundReport::level2 == 0` em toda peça medida, e há gate a
     exigi-lo.
  4. O §6.4 é DETECTADO e contado, não reparado — nenhuma fixtura
     contém o fenómeno (`collapsed_fans == 0` nas duas).
  5. O gate nº8 não é medível nos fixtures (eles não estão remalhados
     isotropicamente, e o teste mede-o antes de medir qualquer número).

⛔⛔ E O ACHADO QUE PROVAVELMENTE PRECISA DE EMENDA DA ESPEC (para o E,
  via Enio — a janela I não foi olhar): com a fase zero honrada, a
  cadeia da casa entrega a FORMA dentro da barra do oráculo
  (enviesamento p50 6,8° contra 4,8–7,1°) e a TOPOLOGIA não fecha
  (χ = −5). A causa está medida e é a montante das duas obras: o
  solver contínuo do G3 entrega até 11 % de triângulos dobrados e uma
  translação de costura a meia célula de um inteiro, contra 0,2 % e
  3,5e-15 dos mapas de referência. A espec já nomeia a cura numa linha
  do §5.1 — «restrições lineares entram eliminando uma variável por
  restrição independente» — e o nosso G3 PENALIZA a costura
  (`SEAM_WEIGHT`) em vez de a eliminar.

⚠️ E UMA COLISÃO DE INTEGRAÇÃO, que não é desta linha mas passa muda:
  o número de ADR 0164 está escrito por DUAS linhas com títulos
  diferentes (o desta corrente, versionado, e um não versionado na
  árvore primária). Detalhe no §3 do handoff.
