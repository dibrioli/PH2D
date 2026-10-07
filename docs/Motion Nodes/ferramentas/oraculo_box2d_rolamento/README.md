# Oráculo do rolamento — Box2D 3.1.1 (doc 121 §9.24)

O Box2D resolve a resistência ao rolamento **dentro do solver** (um impulso angular limitado em cada
iteração) — a referência que o rapier `0.35` não tem. Corre-se sobre as **nossas** entradas; nunca se
lê o fonte (só o cabeçalho público, para o chamar).

- **Licença (passo 1, `CLAUDE.md` §0.9):** o artefacto é o pacote `extra/box2d 3.1.1-2` do Arch,
  `usr/share/licenses/box2d/LICENSE` = MIT. Sem a biblioteca instalada, o `corre.sh` baixa o pacote do
  espelho do sistema para `target/oraculo_box2d/` e confere a ASSINATURA contra uma cópia do chaveiro do
  pacman — nada de root, nada instalado.
- **Entradas:** `cargo test -p ph2d-app-motion --release exporta_a_pilha_para_o_oraculo -- --ignored`
  escreve `target/prova/onda9/oraculo/pilha_XX.txt` (as posições iniciais da taça da direita da `=114`
  em discos, nas onze realizações, e a taça).
- **Corrida:** `ESCALA=1.1233 bash corre.sh "$PWD/target/prova/onda9/oraculo"` — a escala alinha o
  limiar do Box2D (`rr 0,239`) ao da teoria (`tg 12° = 0,213`); o limiar não depende do raio, logo a
  unidade do `rollingResistance` é a nossa (binário = `rr · N · raio`).
- **Fixture:** [`box2d_2026-10-07.txt`](box2d_2026-10-07.txt).

⚠️ **NÃO é um gate de paridade:** a lei do Box2D é por PAR (trava a rotação RELATIVA das duas peças);
a nossa é por PEÇA (o travão relativo por par foi medido e recusado — a tabela do `rolar.rs`). O
oráculo responde a duas perguntas: a lei da rampa perto do limiar num solver que a resolve por dentro,
e se «um `Rolling` baixo faz a pilha girar MAIS que sem ele» é coisa da nossa trava ou das pilhas com
rolamento (é das pilhas: no Box2D, `0,05` gira `198°` de mediana na queda contra `77°` sem o botão).
