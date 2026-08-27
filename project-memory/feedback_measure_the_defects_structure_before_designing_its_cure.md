---
name: feedback-measure-the-defects-structure-before-designing-its-cure
description: "Quatro curas desenhadas a partir do NOME do defeito sairam piores que ele; a quinta, desenhada a partir da estrutura MEDIDA, curou-o — o nome nao e' a estrutura"
metadata:
  node_type: memory
  type: feedback
---

A escultura do artista entrava com **4 arestas nao-manifold** e a saida tinha furos.
Construi **quatro** reparacoes em dois dias — «partir a aleta», «partir antes do
remalhe», «partir e fechar», «deitar a aleta fora» — e **as quatro sairam piores que
o defeito** (bordo `8` ⇒ `148`, `χ` `1` ⇒ `−16`, uma delas com a saida VAZIA). Todas
as quatro nasceram do **nome** que eu tinha dado ao defeito.

A quinta comecou por uma **sonda**: quantas faces reclamam a aresta · o angulo entre
elas · se alguma e' duplicata exacta · e **a orientacao das copias**. A resposta:
`0` copias com a MESMA orientacao, `4` com orientacao **OPOSTA**. Nao era aleta, nem
duas folhas, nem beliscao — era um par `(triangulo, espelho)`, uma **bolsa de volume
zero**. Apagar UMA das duas tira metade de uma superficie fechada, que e' exactamente
por que as quatro tentativas abriram a peca. Apagar AS DUAS nao tira superficie
nenhuma: `4 ⇒ 0` ambiguas, `bordo 0 ⇒ 0`, e todas as reguas da cadeia melhoraram.

**Why:** *o nome de um defeito e' uma hipotese sobre a estrutura dele, e uma hipotese
nao testada escolhe a cura antes de a pergunta ser feita.* Pior: varias estruturas
diferentes produzem a MESMA contagem (4 faces numa aresta le^-se como aleta, como
folha dupla e como duplicata), e cada uma pede uma cura **oposta** a's outras. Duas
curas opostas com a mesma confianca e' o sinal de que falta uma coluna.

**How to apply:** antes de escrever a cura de um defeito estrutural, escreva a
**sonda que descreve a estrutura** — e escreva-a como uma tabela «o que isto e'» ×
«a cura que isso escolhe», para que a coluna que **separa** as hipoteses seja
explicita. Se duas leituras da mesma contagem levam a curas contrarias, a coluna que
falta e' a que decide tudo (aqui: a orientacao). E ponha a **regua da recusa dentro
da propria cura** — a mesma que reprovou as tentativas anteriores —, senao a cura
nova e' so' a variante seguinte do mesmo erro.
Irma^ de [[feedback-a-correct-mechanism-can-prescribe-the-wrong-cure]] e de
[[feedback-a-cure-that-moves-the-defect-names-it]].
