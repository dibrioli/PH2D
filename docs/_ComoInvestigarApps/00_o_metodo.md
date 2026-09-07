# O ALVO É UM ORÁCULO QUE SE CORRE — nunca um fonte que se lê

> **Uma frase:** quando outro app já faz o que estamos a construir, **ponha-o a
> correr por script, colha a SAÍDA dele sobre entradas NOSSAS, e transforme cada
> corrida num gate.** Não abra o fonte dele — não porque seja sempre ilegal, mas
> porque **ler é o método pior**, e as três razões estão medidas no §2.

⚠️ **Este doc é canónico e vale para TODA implementação deste app**, não só para
as clean-room. A clean-room ([SKILL](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md))
é o caso particular em que o alvo é restrito e a parede é obrigatória; o método
abaixo é o geral, e é o que produziu o pincel de tecido (§5).

---

## §0 — De onde este doc veio

Enio, 2026-09-07, depois do smoke do pincel de tecido:

> *«Foi notável sua capacidade de acertos nesta rodada e você conseguiu criar
> algo de alta complexidade com razoável facilidade. […] Vi que você abria o
> Blender repetidamente estudando o comportamento das ferramentas. Precisamos
> não só documentar como incentivar esse tipo de comportamento em toda
> implementação deste nosso APP.»*

⚠️⚠️ **Ele viu certo, e o mecanismo é mais preciso do que a frase — e a precisão
é toda a diferença.** O alvo **foi** corrido repetidamente; ele foi corrido **por
script, sem interface, fora da árvore, sobre malhas nossas**, e o que voltou
foram **números**. Ninguém leu uma linha do fonte dele; ninguém *podia*
(`.claude/settings.local.json` nega os caminhos). *Correr um app e ler um app são
dois métodos diferentes, e só um deles termina num gate.*

---

## §1 — A lei

> **Uma pergunta sobre comportamento responde-se EXECUTANDO, e a resposta que
> volta é um vector de teste — não uma nota.**

Corolário, que é o que faz a diferença de velocidade:

> **Todo traço que você colhe do alvo é um gate de regressão que você não teve
> de inventar.** O corpus do tecido são `86` traços ⇒ `86` gates, e ⛔ nenhum
> deles precisou de alguém decidir «qual seria a barra razoável».

---

## §2 — Por que LER o fonte é o método pior (três razões medidas)

**(a) O fonte responde *como*; o gate precisa de *o quê*.**
Uma lei lida no fonte chega sem barra e sem corpus: ela é uma afirmação. Uma lei
medida chega com as duas metades — o número que ela produz **e** o conjunto de
entradas em que ela tem de o produzir. ⚠️ Neste repo, **onze** explicações do
tecido foram construídas, medidas e **refutadas**
([plano §3](../3D/cloth/06_o_plano_do_que_falta.md)). Nenhuma delas teria sido
refutada por leitura — todas eram plausíveis no papel.

**(b) O fonte descreve o programa; o artista usa o PRODUTO, e os dois divergem.**
Registado no `CLAUDE.md` §5: os defaults do modo `b` do sculpt **moram num
`.blend` binário**, não no código. Quem lê o código lê os defaults errados. O
mesmo vale para presets, para o que a UI reescreve na entrada, e para toda lei
que o app aplica *antes* de chamar a função que tem o nome bonito.

**(c) Ler contamina — e não só juridicamente.**
A triagem de 2026-08 achou **~460 notas** no repo inteiro a citar nome interno de
alvo restrito ([ACHADO](../3D/cleanroom/ACHADO_proveniencia_por_nome_interno.md)).
Cada uma é uma âncora que ata o nosso desenho ao desenho dele — inclusive aos
defeitos dele. ⛔ E há a metade legal, que é dura e simples: **portar GPL para
dentro deste produto obriga a publicar** ([SKILL §1](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)).

⭐ **A saída, essa, é livre.** GPLv2 §0: *«the output from the Program is covered
only if its contents constitute a work based on the Program»*. **Posições de
vértices de uma malha NOSSA não são obra baseada no programa.** É por isso que a
entrada tem de ser nossa — §3 passo 3.

---

## §3 — O protocolo, em sete passos

**1. TRIAGEM DE LICENÇA, primeiro e sempre.**
Percorra a escada e **pare na primeira porta aberta**
([SKILL §2](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)):
existe um app **permissivo** que faz a mesma coisa? Então **porte-o**, com
atribuição, e o clean-room acabou antes de começar. ⭐ Neste sistema há **duas**
portas permissivas instaladas — Godot (MIT) e OpenToonz (BSD-3) — e o
[inventário](01_o_arsenal.md) diz qual serve a quê. *Uma semana de clean-room
gasta onde havia porta aberta é a forma mais cara deste erro.*

**2. UMA pergunta por vez, e escreva-a antes de correr.**
*«O `δ` é a diferença dos pontos 3D ou a projecção do movimento no plano do
ecrã?»* é uma pergunta que uma corrida responde. *«Como funciona o pincel?»* não
é pergunta nenhuma — é um pedido de leitura disfarçado.

**3. A ENTRADA é NOSSA.**
Malha nossa, imagem nossa, cena nossa, gerada pelo nosso próprio harness. ⛔ Um
asset do alvo como entrada envenena a saída: a proveniência da entrada decide a
da saída ([SKILL §5](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)).
No tecido: uma grelha `64×64` e uma esfera UV `96×64`, as duas nossas.

**4. VARIE UMA GRANDEZA POR TRAÇO, e grave o enquadramento no CABEÇALHO do
ficheiro.**
⚠️⚠️ **A régua das excepções do corpus do tecido cresceu TRÊS vezes** — de `7`
para `30` de `86` —, e das três vezes o número estava certo e **a régua** é que
varria menos grandezas do que o corpus continha
([README das fixtures](../3D/cleanroom/fixtures/cloth/README.md)). *Uma lista de
excepções sem a régua ao lado não é auditável, e quem acrescenta um traço herda
a régua que não vê.* ⇒ o cabeçalho do ficheiro é a fonte; a prosa nunca é.

**5. COMPARE POR PASSO, não só o resultado final.**
⭐⭐ Esta é a alavanca que fecha diagnósticos. Enquanto a bancada do tecido
comparava só a malha final, dez traços discordavam sem endereço; com o rastreio
**por passo de força** os mesmos dez passaram a **resolução do ficheiro** — e a
lei que faltava (dois `φ`, um com banda e outro sem) apareceu num passo
específico, não na diferença acumulada.

**6. A BARRA sai de um VALE MEDIDO, e o vale tem de incluir o lado APROVADO.**
⛔⛔ **Duas das três barras do gate de artefactos do tecido foram RETIRADAS por
reprovarem a saída do PRÓPRIO alvo** (o alvo dava espinho `0,900` e estica
`3,72×` contra `0,690` e `2,98×` do nosso defeito). *Uma barra calibrada sem o
lado aprovado mede os nossos defeitos, não os dele.* ⇒ meça **os dois lados** com
o **mesmo código** antes de escrever um número.

**7. ONDE O ORÁCULO NÃO REPETE, a barra tem de conhecer a banda.**
Na esfera, quatro corridas da mesma configuração do alvo dão quatro resultados.
⛔ Uma barra de MÁXIMO ali mede o sorteio. A espec prescreve **p95** nessa
população, e há gate a dizê-lo (`onde_o_maximo_nao_decide_decide_o_p95`).

---

## §4 — As leis que este método já pagou (leia antes de repetir uma)

| lei | onde ela mordeu |
|---|---|
| ⛔ **Uma barra sem o lado aprovado mede os NOSSOS defeitos** | 2 barras retiradas do tecido · 4 «nenhuma melhoria» seguidas no quad remesh |
| ⛔ **Um gate que pergunta ao PRODUTOR é cego a um segundo produtor** | o gate da direcção do Push passou com a mutação viva; curou-se perguntando à SAÍDA (o campo de deslocamento) |
| ⭐ **Um controlo pode NÃO EXISTIR — e prova-se com duas amostras idênticas ao bit** | `peso_normal` a `0`, `0,5` e `1` dá o mesmo bloco de vértices ⇒ aquele knob não é lei deste pincel |
| ⭐ **Uma explicação REFUTADA vale, e escreve-se** | as 11 do tecido; sem elas a janela seguinte reconstrói-as |
| ⚠️ **A fixture pode não produzir o fenómeno** | a 3.ª leitura de «mutação sobreviveu»: falta gate · linha redundante · **a fixture não exercita a lei** |
| ⚠️ **Compare na malha DELE quando puder** | o oráculo do quad remesh grava as fases intermédias; comparar fase a fase na malha dele é legal e mais forte que ler código |
| ⛔ **Toda comparação nomeia a DENSIDADE dos dois lados** | a barra do oráculo esteve a ser lida a `1/9` da densidade dele durante uma semana — mais fino é mais fácil |

---

## §5 — O exemplo trabalhado: o pincel de tecido (W10)

| | |
|---|---|
| **Alvo** | Blender 5.2.1 LTS, GPL ⇒ **parede obrigatória** |
| **Quem correu o alvo** | o subagente **E**, fora da árvore, por script sem interface |
| **Quem escreveu o produto** | esta janela, que **nunca** viu o fonte (deny em `.claude/settings.local.json`) |
| **Entradas** | grelha `64×64` e esfera UV `96×64` — **nossas** |
| **Corpus** | `86` traços · `149` ficheiros · com rastreio **por passo** em `10` deles |
| **Placar** | **`79` dentro da barra**, `7` abertos (`VERDE_N`/`ABERTO_N` no [bancada](../../crates/ph2d-cloth/tests/oraculo_do_pincel.rs) — ⛔ conte-os lá, nunca aqui) |
| **Leis achadas** | duas `φ` · o peso de face uniforme · a área fixa do Grab · a base persistente que **satura** · as cinco cláusulas da colisão |
| **Leis REFUTADAS** | `11`, escritas |
| **O que o método deu de graça** | o corpus **é** a suíte de regressão; e foi um traço dela que apanhou um defeito de `11,5×` no produto (o `δ` incremental do Grab) |

⭐⭐ **O sinal de que funcionou:** o defeito do produto foi encontrado **pelo
gate que nasceu do oráculo**, não por um smoke. *Um corpus medido acha o que a
revisão não vê.*

---

## §6 — A parede (só quando o alvo é restrito)

Alvo **permissivo** (MIT/BSD/Apache) ⇒ **sem parede**: porte, com atribuição.
Alvo **copyleft** (GPL/LGPL) ou proprietário ⇒ **parede**, e ela tem três regras
que não se negoceiam ([SKILL §3](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)):

1. **Quem escreve o produto nunca abre o fonte do alvo** — e o `deny` do
   `.claude/settings.local.json` torna isso uma propriedade da máquina, não uma
   promessa.
2. Os papéis **E** (especificador) e **R** (revisor) leem o alvo e entregam
   relatórios com **zero** identificadores dele.
3. Todo artefacto que cruza a parede passa por
   `bash scripts/cleanroom-sweep.sh <VASSOURA> <paths>` antes do commit.

⚠️ **O harness que corre o alvo vive FORA da árvore.** Ele é acto do E; o
implementador pede uma emenda, nunca o corre.

---

## §7 — Recusas (⛔ não reconstrua)

- ⛔ **Portar código de alvo copyleft** para dentro deste produto. A porta é a
  triagem, não a coragem.
- ⛔ **Usar assets do alvo como entrada.** A proveniência da entrada decide a da
  saída, e uma malha dele torna a nossa saída obra derivada.
- ⛔ **Deixar o corpus fora do repo.** Um oráculo que não viaja com a árvore é um
  oráculo que a próxima janela não pode correr — e a bancada morre.
- ⛔ **Comparar só o resultado final.** Custa diagnósticos inteiros (§3 passo 5).
- ⛔ **Escrever a barra antes de medir os dois lados** (§3 passo 6).
- ⛔ **Ler o fonte «só para confirmar»** com a parede em vigor. Isso é o
  incidente do [SKILL §6](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md),
  e queima a janela.

---

## §8 — Como começar (o bloco colável)

```
1. `bash scripts/hw-profile.sh`               — o modo de operação
2. Triagem de licença do alvo                 — pare na primeira porta ABERTA
3. Escreva A PERGUNTA, uma só                 — «δ é projecção ou diferença 3D?»
4. Gere a ENTRADA nossa                       — nunca um asset do alvo
5. Corra o alvo por SCRIPT, sem interface     — ver docs/_ComoInvestigarApps/01_o_arsenal.md
6. Grave a saída como fixture COM CABEÇALHO   — uma grandeza por traço
7. Escreva o gate ANTES da implementação      — red-first, e a barra do vale medido
8. Implemente até o gate ficar verde
9. Escreva as explicações REFUTADAS
```

⚠️ **Se o alvo é restrito, os passos 5 e 6 são acto do subagente E** — a janela
que implementa nunca os corre.

**Ler a seguir:** [o arsenal instalado nesta máquina](01_o_arsenal.md) ·
[SKILL Clean-room](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)
