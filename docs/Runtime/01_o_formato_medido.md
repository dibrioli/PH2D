# O formato do arquivo, MEDIDO — e por que o meu primeiro desenho estava errado

> Linha `line/runtime`, 2026-08-01. Este documento existe porque a medição derrubou o plano que
> o Enio aprovou, e a regra do §0 corta nos dois sentidos: **quem move o número que sustentava
> uma decisão tem de reconferir a decisão.**
>
> ---
>
> ⚠️ **RESGATADO em 2026-08-08 (`line/Vector`). A branch `line/runtime` foi DESCARTADA por ordem do
> Enio, e este documento é a única coisa dela que sobrevive** — porque ele é a **medição**, e a
> medição continua verdadeira mesmo depois de o código que a acompanhava deixar de existir.
>
> **A razão do descarte está escrita AQUI, no §5 e na tabela dele:** a branch construiu a **F1.W0**
> (o envelope de topo), que a própria medição classifica como o que previne **4 de 37 bumps — 11%**;
> a peça que alcança os 18 da linha A (**versão por `ComponentBlob`**, a F1.W1) **nunca foi
> construída**. Somando: 366 commits de atraso, o `project.rs` **partido pela `line/sculpt3d`** no
> meio de exactamente o que ela reescrevia, e um `LEGACY_SCHEMA_FINAL = 48` que descreve uma
> fronteira que o `main` já levou a **55**.
>
> ⇒ **O que se descartou foi a implementação da wave menos valiosa.** O plano de reconstrução, a
> partir do `main` e já com este número em mãos, é o
> [`00_plano_runtime.md`](00_plano_runtime.md) + o [`HANDOFF_runtime_R0_2026-08-08.md`](HANDOFF_runtime_R0_2026-08-08.md).
>
> ⚠️ **Uma nota deste doc já não vale:** o §5 diz *"o crate `ph2d-project-format` … não é trabalho
> perdido"*. Ele **deixou de existir** com a branch. A frase continua verdadeira sobre o DESENHO
> (*chave + versão + payload opaco + carry-through* segue sendo o primitivo das três camadas) e
> falsa sobre o **código** — que se reescreve, agora sem 366 commits de dívida.

## 1 — O que eu propus, e o que ele de facto compra

O plano aprovado era: envelope + tabela de conteúdo + carry-through, com **os seis campos de
topo do `ProjectFile`** virando seis seções. A promessa era parar a hemorragia dos 48 bumps.

Antes de fiar a shell, classifiquei **os 37 bumps documentados** pelo lugar onde a mudança
pousou. O `project.rs` narra cada um, então isto é contagem, não estimativa:

| Onde a mudança pousou | Bumps | Exemplos |
|---|---:|---|
| **A — dentro de um blob de COMPONENTE** (`state.world`) | **18** | `Collider.layer` (v21) · `Collider.is_sensor` (v27) · `PhysicsJoint` +3 (v30) · `PulleyWheel` +3 (v45) · `FxOp.blend` (v38) |
| **B — dentro do `FlipDoc`** (`state.flip`) | **12** | `FlipStroke.holes` (v7) · `FlipFrame.pose` (v14) · `FlipLayer.depth` (v35) |
| **C — dentro do `VecScene`** (`state.vec`) | **3** | `VecVertex.corner_radius` (v10) · a pilha de FX (v22/v23) |
| **D — a forma do `ProjectFile`** (os campos de topo) | **4** | `motion` (v5) · `timeline` (v13) · `physics` (v19/v20) |

**O meu desenho previne a linha D. Quatro de trinta e sete — 11%.**

Os outros 89% moram **dentro** do `state`, que é UM campo. Cortar o topo em seis pedaços não os
alcança.

## 2 — E o corte de dentro também não basta, sozinho

A leitura seguinte era óbvia: então corte o `ProjectState` também. Ele é literalmente
`{ world, vec, flip }`, e **`vec` e `flip` já carregam versão própria**
(`VEC_SCENE_SCHEMA_VERSION` · `FLIP_SCHEMA_VERSION`, os dois já pinados na tripla) — hoje um bump
do Flip move o número dele **e**, de carona, o global. Três seções em vez de uma cobrem B + C =
**15 bumps**, usando números que já existem.

⚠️ **Mas isso é sobre o RAIO da recusa, e o problema não é só o raio.** Um projeto v46 aberto por
um build v47 tem, dentro do `WORLD`, um blob de `PhysicsJoint` com um campo a menos. Seccionar
faz a recusa ser *da seção `WORLD`* em vez de *do arquivo* — e o `WORLD` não é opcional, então o
artista continua sem abrir o projeto.

## 3 — O que a medição mostra que a cura é

O `ComponentBlob` **já é um envelope**, um nível abaixo:

```rust
pub struct ComponentBlob { pub type_id: ComponentTypeId, pub data: Vec<u8> }
```

Chaveado, com o payload medido, opaco. É exatamente por isso que **registrar um componente NOVO
nunca custou bump** (o `GravityScale` do W8 da física, e a nota que cada crate-componente do
vetor repete: *"cunha a própria blob-key e não move nada"*).

**O que falta é UMA versão por blob.** Com ela:

- um append a `PhysicsJoint` move a versão **daquele componente** e de mais nada;
- um projeto **sem joints** não é tocado por bump nenhum de joint — hoje ele é recusado;
- um append com valor default é uma **migração trivial e escrevível**, em vez de uma recusa;
- e a política de *pular o que não entendo* passa a ter granularidade útil.

Isso alcança a linha A: **18 de 37**. Com o corte do `ProjectState` (B + C = 15) e o envelope de
topo (D = 4), a cobertura é **37 de 37** — e cada peça usa o mesmo mecanismo: *chave + versão +
payload opaco + carry-through*.

## 4 — A pergunta que é de PRODUTO, não de engenharia

O carry-through é seguro e óbvio para uma seção **desconhecida**: este build não tem UI capaz de
a sobrescrever, então guardá-la e devolvê-la é a coisa certa e não há decisão a tomar.

⚠️ **Para uma seção CONHECIDA e nova demais, não é.** Se o build abre o projeto com a animação
em branco e o artista aperta Ctrl+S, o save grava esse vazio por cima — que é a doença que o
`project.rs` já nomeia hoje (*"a animação não some por um bug — some porque o app abriu, mentiu e
salvou"*). Há três leituras honestas, e a escolha é do Enio:

1. **Recusar o documento** (o que fazemos hoje) — nunca perde nada, e o artista fica sem abrir.
2. **Abrir com a seção TRANCADA** — a metade que este build não lê fica visível-mas-não-editável,
   e o save devolve os bytes originais. É a leitura correta, e custa UI de verdade
   (um estado "somente-leitura" por módulo, e o que ele mostra).
3. **Abrir e avisar** — barato, e transfere o risco para o artista ler um toast.

**Recomendação: (1) para seção conhecida, carry-through silencioso para seção desconhecida.** É a
combinação que nunca perde trabalho e que já entrega o ganho de verdade — *um módulo que este
build não conhece nunca mais quebra o arquivo* —, com (2) escalonável depois, quando existir um
módulo cuja ausência valha a UI.

## 5 — O que fica de pé do que já foi construído

O crate **`ph2d-project-format`** (o envelope, 11 gates, 5 mutações) é o primitivo das **três**
camadas, não só da D: a mesma pergunta — *chave, versão, payload opaco, e quem não consumiu
devolve* — é feita no arquivo, no `ProjectState` e no `ComponentBlob`. Ele não é trabalho
perdido; ele é o que torna as outras duas exprimíveis sem uma segunda resposta.

O que muda é a **ordem** e o que a Frente 1 promete:

| Wave | O que faz | Bumps que teria prevenido |
|---|---|---:|
| **F1.W0** | o envelope + `WORLD`/`VEC_SCENE`/`FLIP` + os 4 de topo, cada um com a versão que já tem | 19 |
| **F1.W1** | versão por `ComponentBlob` + migração por append-default | 18 |
| **F1.W2** | política de degradação por seção + o que o artista vê | — |

⚠️ E o `PROJECT_SCHEMA` **não sobrevive como número global** — ele vira `LEGACY_SCHEMA_FINAL = 48`,
o último valor que o formato posicional teve, lido uma vez por um leitor de compatibilidade.
