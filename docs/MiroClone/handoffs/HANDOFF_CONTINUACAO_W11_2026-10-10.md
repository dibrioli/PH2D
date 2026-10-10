# CONTINUAÇÃO — `line/MiroClone` W11 «O quadro ao vivo na rede local» (briefing, 2026-10-10)

> Para a janela que ASSUME a linha (`/pd-linha-assumir`, rota «linha reaberta» do
> [MODELO_TROCA_DE_AGENTE_NA_LINHA](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)):
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. W5…W10 estão por integrar na mesma branch (smoke OK em todas; o dono pode mandar
> integrar antes de a W11 começar — se o `git cherry main HEAD` der 0, é o `git reset --keep main` do
> modelo, nunca `rebase`). Último handoff: [W10](HANDOFF_INTEGRACAO_line_MiroClone_W10_2026-10-10.md).

## 1. A escolha do dono (10/10)

- **«Quadro ao vivo»** é a próxima frente: a **Etapa 2** do [plano](../02_plano.md) (§ Etapa 2,
  l. 349-351: rede, presença, cursores, seguir / «trazer todos», votação anónima, modo privado).
- **Rede local PRIMEIRO** (pergunta respondida 10/10): as pessoas estão na mesma casa/escritório (mesmo
  Wi-Fi), todas com o PH2D instalado (a versão web está estacionada — memória
  `project_web_shell_decided_by_owner_2026_10_06`). Um computador ABRE o quadro e os outros ENTRAM. Sem
  servidor externo nem custo. **A internet vem depois SEM refazer nada** ⇒ o protocolo e a reconciliação
  não sabem que transporte os leva.
- Fica para a W12 (a facilitação AO VIVO, que assenta neste núcleo): a votação anónima e o modo privado.
  ⛔ Não abra outras frentes (as pontas abertas da W10 §9 esperam).

## 2. O que existe — e o que NÃO existe (levantado 10/10; confira o código antes de acreditar)

- **Nada de rede**: nenhuma crate de sync/rede/presença, nenhuma dependência de rede no workspace
  (`Cargo.toml` da raiz e das `ph2d-board-*`; `STACK_VERSOES.md`), nenhum ADR de colaboração do Quadro,
  nenhum oráculo de colaboração corrido. (O `ph2d-vector-doc/src/crdt.rs` é um stub do VETORIAL, ADR-0057
  — não serve aqui; o ADR-0164 é «live sync» entre instâncias de cena, outra coisa.)
- **A gramática da colaboração já está no modelo** (`ph2d-board-model/src/element.rs:248-274`): `z`
  (`FracKey`, índice fraccionário), `version: u64`, `nonce: u32`, `deleted: bool` (lápide); `BoardOp` +
  `apply`/`apply_batch` (`ops.rs`) e `History` (`history.rs`) — o plano (l. 79, 350) diz que a Etapa 2
  «escolhe transporte e reconciliação sem migrar ficheiros». ⚠️ Mas: o **`nonce` nunca é preenchido** em
  produção (só um teste o põe a 7); o `version` sobe ao apagar (`ops.rs:70`) — confirme os outros
  caminhos; e o **`mint_id` é um contador LOCAL** (`element.rs:562`) ⇒ dois computadores fabricam o
  MESMO id. Isto é o 1.º degrau.
- **A «equipa» (W7) é local**: o autor é um texto que o artista escreve (`team.rs:40`), o «lido» é local.
- **A pesquisa** (`01_pesquisa_miro_excalidraw.md:136-152`) descreve a colaboração do Excalidraw: não é
  CRDT — união por `id` com `version`, desempate por `versionNonce`, lápide `isDeleted`, índice
  fraccionário, servidor que só retransmite; duas edições ao MESMO elemento perdem uma inteira. É uma
  descrição LIDA — vale como pergunta, não como lei (§3.1).

## 3. O que fazer — num bloco (CLAUDE.md §0.10), nesta ordem porque cada passo decide o seguinte

### 3.1 O oráculo PRIMEIRO (CLAUDE.md §0.9 — corrido, nunca lido)

Os oráculos instalados (`docs/MiroClone/ferramentas/excalidraw_oracle`, MIT; `tldraw_oracle`, licença de
DESENVOLVIMENTO — triagem por artefacto antes de o correr para colaboração) não correm a colaboração. Ponha
DUAS sessões a falar (Playwright, dois contextos; o servidor de sala do Excalidraw ou o sync do tldraw,
local — triagem de licença de CADA artefacto instalado, `pacman -Qi`/`package.json`) e MEÇA, num ficheiro
de casos à parte, com cabeçalho e duas corridas iguais:
- o mesmo elemento mexido nos dois lados ao mesmo tempo (o mesmo campo; campos diferentes: posição × cor
  × texto) — quem ganha, e se se perde a edição inteira ou só o campo;
- apagar × editar; criar dois elementos no mesmo sítio de z; mover o mesmo para a frente nos dois;
- a seta presa a uma forma que o outro apaga ou mexe; o texto escrito a dois;
- o **desfazer**: desfaz só o meu? o que acontece se o outro mexeu depois no mesmo elemento;
- entrar a meio (o estado inicial), sair e voltar (o que mudou entretanto), cair a rede a meio;
- o que se vê do outro: cursor (frequência, suavização), selecção alheia, nome/cor, «seguir»,
  «trazer todos»; e o que se envia (o tamanho e a cadência das mensagens a arrastar).
O que o oráculo do DONO faz decide a lei (memória `feedback_the_owners_reference_product_decides_the_law`:
o dono usa o excalidraw.com e o Miro). Onde os dois divergem, escreva a divergência e escolha com o porquê.

### 3.2 O desenho (técnico — delegado, padrão-ouro; com ADR)

- **Um ADR novo** (o número CONTA-SE em `docs/architecture/decisions/`): transporte, reconciliação,
  identidade, desfazer, segurança na rede local, e as alternativas RECUSADAS com o preço medido. Uma
  dependência de rede entra no workspace pela primeira vez: `bash scripts/stack-audit.sh --tetos`, triagem
  de licença, `cargo deny`, e o que ela custa no build (meça — `MEDIR_VELOCIDADE.md`).
- **Drop-crates** (ADR-0075): o protocolo e a reconciliação SEM IO numa crate pura (determinista,
  testável em processo — a «rede» dos testes é uma fila com atraso, reordenação, duplicação e perda), e o
  transporte numa crate à parte, atrás de uma porta que a internet virá implementar. O código de família
  vive em `ph2d-app-board`; a shell é composição (`the_shell_only_shrinks`).
- **Identidade**: ids que não colidem entre máquinas (o `mint_id` de hoje), `nonce` sorteado a cada
  edição, `version` a subir em TODA a op. Se o formato do quadro mudar (`FORMAT_VERSION` 9 → 10), o leitor
  antigo lê os ficheiros 9 (fixture gravada pelo build ANTES da mudança, como a W9 fez com o 8).
- **Desfazer**: o que o oráculo mostrou (3.1) — o `History` de hoje é de UM artista.
- **Entrar na sala** na rede local: como um computador encontra o outro sem o dono escrever um IP (ou com
  um código curto) — o que for MEDIDO como fiável no Wi-Fi da casa; nada exposto fora da rede local; nada
  de abrir a firewall sozinho (diga ao dono, se for preciso).
- **Presença**: cursores, selecção e nome dos outros, «seguir» e «trazer todos» — no idioma do oráculo.
- **Medir antes de limitar** (CLAUDE.md §0): latência ponta-a-ponta a arrastar (2, 4, 8 participantes),
  bytes por segundo, convergência; nenhum tecto sem a tabela; e a régua das rotas (W9) não pode regredir.

### 3.3 Gates (cada lei com o VERMELHO visto antes da cura)

- Convergência: N pares em processo, operações concorrentes sorteadas com semente, rede com
  reordenação/duplicação/perda ⇒ todos acabam com o MESMO documento (byte a byte no `postcard`).
- Os casos do oráculo (3.1) como fixtures: o nosso protocolo resolve cada um como ele (ou a divergência
  escolhida, escrita).
- Dois processos REAIS no `localhost` (o binário, sem janela se der) a trocar operações pela crate de
  transporte.
- O caminho do ecrã: abrir/entrar pelo menu, o cursor do outro desenhado, seguir — pelo `ph2d-ui-testkit`
  / harness do hero, como os gates das W4–W10.

### 3.4 O smoke do dono

O dono testa com DUAS janelas no mesmo computador (e, se quiser, um segundo computador no Wi-Fi): uma
cena nova (o número CONTA-SE no roteiro das cenas: hoje vai até 11), lançada por variável de ambiente, que
abre um quadro como anfitrião e ensina a entrar com a segunda janela — o comando exacto das duas, em
passos numerados (CLAUDE.md §0.8). Fotografada (`fotografa_cena.sh`; com duas janelas, fotografe as
duas) antes de ir ao dono: a foto da W10 apanhou uma lição que mandava ao botão errado.

## 4. Protocolo

`/pd-linha-fechar` ao fim: portão batched 1×, auditoria de 2 lentes (uma das lentes é a REDE: o que se
perde, o que se duplica, o que diverge), prova de mutação, cena fotografada, handoff de integração W11 que
SUBSTITUI o da W10, CLAUDE.md §5.1. Não integra, não pusha. Ao dono: PT-BR, curto, sem jargão.

## 5. Lembretes medidos nesta linha (W9–W10)

- O `cargo-test-narrow.sh` conta um erro de compilação como «0 testes» e já se embrulha no
  `ph2d-run.sh` (embrulhá-lo outra vez dá «0 passaram»); o `cargo check --all-targets` com
  `CARGO_BUILD_WARNINGS=deny` à parte — uma função só chamada por testes cai no `-D dead-code` do portão.
- O guarda de recursos recusa um `heredoc` com `cargo` dentro; o `zsh` não parte variáveis em palavras
  (`git add --pathspec-from-file`).
- O tecto de 700 linhas conta os `*_tests.rs` (não o `tests.rs`).
- Igualdade EXACTA de `f64` entre perfis de build falha por 1 ULP (o nextest corre outro perfil): use
  tolerância.
- O oráculo: duas corridas iguais antes de gravar; saídas grandes ⇒ `"compacto": true` e `"guardar":
  false` (`corre_fidelidade.mjs`); uma captura do ecrã a meio de um gesto é o passo `captura`.
- A prova de mutação acha sobreviventes que a auditoria não vê (W10: 2 de 17) — tape-as e re-mute.
- Uma recta paralela a um eixo, um `0` especial, um nível degenerado: os casos que a varredura «natural»
  não sorteia são onde a lei de outro programa diverge — ponha-os de propósito.

## 6. Pergunta de produto da W10 §9 — RESPONDIDA e feita (10/10)

Uma NOTA importada com a letra à mão ficava assim para sempre. O dono escolheu: **o botão «Rascunho ↔
Final» passa a mexer nas notas** (só na letra; o papel desenha-se e exporta-se liso, `roughness 0`; as
notas novas nascem no modo do quadro). Gates: `the_sketch_button_gives_an_imported_notes_letter_back_to_the_mode`,
`a_sketched_note_travels_with_smooth_paper_and_the_hand_letter` e os dois de `ink_tests` reescritos, cada um
visto vermelho antes. A recusa antiga no plano (§ Recusas) está revogada com o porquê.
