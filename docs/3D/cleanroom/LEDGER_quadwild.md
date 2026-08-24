# LEDGER de proveniência — clean-room do quad remesh (família `quadwild`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_quadwild.md` (append cego).

---

## Alvo

| campo | valor |
|---|---|
| Nome | `quadwild` / `quadwild-bimdf` (umbrella; a família inclui os satélites abaixo) |
| Repo | clone local em `/home/enio/Documentos/Projetos/ph2d-quadbench/oracle/` (fora da árvore da engine) |
| Papers | Pietroni et al., *Reliable Feature-Line Driven Quad-Remeshing* (SIGGRAPH 2021) · Heistermann et al., *Min-Deviation-Flow in Bi-directed Graphs for T-Mesh Quantization* (SIGGRAPH 2023) |
| Licença do umbrella | **GPL-3.0** (`oracle/LICENSE`, verificado byte a byte em 2026-08-24) |
| Precedente da casa | [ADR-0162](../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md) — oráculo fora da árvore, invocado por CLI, nunca linkado |

### A concessão relevante (GPLv3 §2), transcrita

> *"You may make, run and propagate covered works that you do not convey, without
> conditions so long as your license otherwise remains in force."*

⇒ Ler, compilar, rodar, modificar e instrumentar em privado é **licenciado**, não tolerado.
Nenhum ato deste ledger envolve *convey*. Não é AGPL ⇒ o §13 não se aplica; ainda assim o
oráculo roda **local**.

---

## §2 — Triagem: a escada de portas

⚠️ **A licença do umbrella não é a licença de cada fase.** O primeiro achado da triagem é
que a família é um mosaico, e o copyleft entra por **três** submódulos, não pelo todo.

### Tabela de licenças MEDIDA (2026-08-24, lida de cada `LICENSE`/`COPYING` no clone)

| dependência | licença | degrau | fase que ela serve |
|---|---|---|---|
| `vcglib` | **GPL-3.0** | T2 | malha, I/O, utilitários geométricos |
| `xfield_tracer` | **GPL-3.0** | T2 | **o traçado de separatrizes (o nosso F3)** |
| `CoMISo` | **GPL-3.0** | T2 | solver misto-inteiro (o *mixed-integer* do MIQ) |
| `libigl` | MPL-2.0, **com `include/igl/copyleft/` GPL** | T0½ / T2 | campo, parametrização |
| `libsatsuma` | **MIT** | **T0** | **quantização Bi-MDF (o nosso F4)** |
| `lemon` | **Boost** | **T0** | fluxo de custo mínimo (backend do Bi-MDF) |
| `OpenMesh` | **BSD-3** | **T0** | estrutura de malha half-edge |
| `libTimekeeper` | **MIT** | T0 | instrumentação de tempo |
| `nlohmann/json` | **MIT** | T0 | serialização |
| `eigen` | **MPL-2.0** | T0½ | álgebra linear |
| `blossom5-cmake` | Unlicense (o *wrapper*) | ⚠️ | ⚠️ o **blossom5** empacotado tem licença própria — a conferir |
| `quadretopology` | ⚠️ **sem arquivo de licença no topo, sem cabeçalho nos fontes** | ⚠️ | preenchimento por padrões (o nosso F5) |
| `lpsolve` | ⚠️ a conferir (tipicamente LGPL) | ⚠️ | solver linear |
| `glew` | BSD-3 | T0 | só visualização |

⇒ **Registrado antes de qualquer leitura de fonte.** A caçada T1 (irmão permissivo por
fase) começa daqui, e o veredito por fase vai abaixo.

---

## Patente (§8.1) — checkpoint incondicional, CUMPRIDO

- **Buscado em:** 2026-08-24
- **Termos:** `quad mesh extraction` · `cross field` · `integer grid map` ·
  `quadrilateral remeshing` · `global parametrization`, cruzados com Autodesk, Pixologic,
  Maxon, Adobe, Dassault, Siemens, Ansys e universidades; mais os autores dos papers.
- **Resultado:** ⭐ **nenhuma patente viva bloqueia o caminho campo → mapa de grade inteira
  → extracção.** Três achados, com veredito:

| patente | dono | estado | lê sobre nós? |
|---|---|---|---|
| US 8.531.456 (remalhamento por grade 2D em género g) | Technion R&D | **EXPIRADA** | ⇒ espec de graça (§1.5.4) |
| US 11.017.597 (redução de singularidades) | concedida 2021 | **VIVA** | ⛔ não — pós-processa malha quad **existente** por gabaritos. ⚠️ cerca nomeada |
| US 9.349.216 (quad por **esboço**) | ETH Zurich + Disney | **VIVA até 2034** | ⛔ não — rede de curvas **autorada**. ⚠️ cerca nomeada: os autores são os do paper de padrões de retalhos de n lados (família do nosso F5) |

⇒ Detalhe e mecanismo: [`TRIAGEM_quad_remesh.md` §3](TRIAGEM_quad_remesh.md).

---

## Papel E — Especificador

| campo | valor |
|---|---|
| session-id | `edbb014f-4ffb-40ff-bd89-2200158288ca` |
| transcript (⛔ **zona contaminada** — I nunca lê) | `/home/enio/.claude/projects/-home-enio-Documentos-Projetos-PH2D/edbb014f-4ffb-40ff-bd89-2200158288ca.jsonl` |
| aberto em | 2026-08-24 |
| ⚠️ nota de papel | Esta janela **escreveu produto** desta linha antes de assumir E (a cadeia F1–F5 + `ph2d-gridmap` G1–G4). O custo foi declarado ao Enio e a ordem foi mantida: a janela **muda de papel** e fica **queimada para I** no módulo. |

### Cobertura da travessia (§3.E)

⚠️ **A travessia integral do alvo GPL NÃO foi iniciada, de propósito** — o passo 1 do
BLOCO-E (triagem) devolveu uma porta mais barata **antes** dela, e a ordem embutida no
passo 1 é *«PARE e reporte»*. Ler o fonte GPL agora seria pagar contaminação por uma rota
que pode nunca abrir.

**Lido até aqui (2026-08-24), e SÓ isto:**

| o quê | natureza | por quê |
|---|---|---|
| `LICENSE`/`COPYING` de 14 dependências do umbrella | texto de licença | triagem §2 |
| `.gitmodules`, `CMakeLists.txt` do umbrella, `CMakeLists.txt` de um submódulo | manifesto de build | descobrir **qual** dependência traz o copyleft |
| `README.md` público do umbrella | prosa pública | lícita a todos os papéis (§3.I) |
| ⛔ **fonte de algoritmo do alvo GPL** | — | **NÃO LIDO** |
| `Directional` (**MPL-2.0**): cabeçalhos de licença + listagem de módulos + a estrutura de opções da integração + o laço de arredondamento | fonte **permissivo** | ⚠️ **não é alvo copyleft** — nenhuma parede se aplica; lido para responder se a porta T0½ alcança o nosso bloqueador |

### ⚠️ Consequência de papel, registrada

Esta janela leu **fonte MPL-2.0** (Directional) e **metadados** do alvo GPL. Ela **não** leu
algoritmo GPL. Ainda assim, e por disciplina do §2 da skill, **quem executar um porte fiel
T0½ deve ser OUTRA janela** — porte fiel não se mistura com quem percorreu a triagem de um
alvo copyleft.

### ⚠️ Vassoura (§7.1) — ⛔ **corrigido pelo R-pré em 2026-08-24: esta secção estava ERRADA**

> **O que ela dizia:** *«NÃO gerada, de propósito — gerar ≥20 identificadores idiossincráticos
> exige ler o fonte do alvo GPL.»*
> ⛔ **E o arquivo existe**, com **21 entradas**, commitado, e é o que varre todos os artefactos
> desta pasta. A [`TRIAGEM §7.4`](TRIAGEM_quad_remesh.md) já registava a verdade; o ledger é que
> não tinha sido alinhado a ela. *Uma afirmação falsa no ledger custa mais do que o que esconde.*

**O estado real:**

- ⭐ **Existe uma vassoura PARCIAL** (`VASSOURA_quadwild.txt`, 21 entradas em base64), montada
  **só do que a janela E de facto viu** — nomes de campo colhidos ao conferir licenças e
  manifestos, e os identificadores internos que o próprio **repositório** já citava
  ([`ACHADO`](ACHADO_proveniencia_por_nome_interno.md): 13 linhas da família do quad remesh).
  ⇒ proveniência **lícita**, e nenhuma delas exigiu ler algoritmo GPL.
- ⚠️ **Ela é suficiente para o que se varre hoje** e foi provada por **controle positivo**
  (§R-pré.5): semeada num arquivo, o sweep sai **vermelho**.
- ⛔ **A vassoura COMPLETA da travessia continua por gerar**, e continua a ser o **primeiro acto
  da Rota B**, se a Rota B abrir — porque *essa* exige ler o fonte do alvo GPL.

---

### Literatura lida (a fonte REAL da espec)

⭐ **A rota escolhida ([ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md))
é clean-room dos *papers*** — logo a travessia integral do fonte copyleft **não é o insumo**,
e não foi feita. O insumo é público:

| fonte | uso | onde |
|---|---|---|
| QEx (SIGGRAPH Asia 2013) | §2–§6 da espec | público; ⚠️ PDF de **imagem**, extraído com `pdftotext -layout` |
| Mixed-Integer Quadrangulation (SIGGRAPH 2009) | §5 e **§5.1** da espec | público — ⭐ **corrigiu** a 1ª redacção, que dizia «re-resolva» onde a receita é uma **escada adaptativa** |
| Integer-Grid Maps (SIGGRAPH 2013) · Ray (arXiv 2025) | contexto | públicos |

⚠️ Cópias locais em `~/Referencias/papers/` (zona contaminada por convenção; ⛔ o
Implementador busca-as pelos **URLs** do cabeçalho da espec, não por lá).

### Instrumentação do oráculo (§5 da skill)

| artefacto | onde | estatuto |
|---|---|---|
| biblioteca **MPL-2.0** clonada | `~/Referencias/directional/` | permissiva; corrida como oráculo |
| arnês C++ (leitor OBJ/OFF, despejo do mapa, modo `so-mapa`, `lengthRatio`) | `~/Referencias/directional-bench/` | escrito por E |
| exportador do **nosso** campo + **fase zero** (`ph2d-remesh-iso`) + régua de **curl** | `~/Referencias/directional-bench/rustfield/` | escrito por E; depende das nossas crates por caminho |
| régua por-face (espelho do `QuadShape`) e verificador do mapa | idem, e **o verificador entrou no repo** | dados/harness |
| ⭐ **fixtures** publicados a I | `docs/3D/cleanroom/fixtures/` | **dados** — entrada nossa, campo nosso |

⛔ **Nada do oráculo entrou no repositório** além de **saídas** (mapas verificados) e do
verificador escrito por E.

### ⚠️⚠️ INVENTÁRIO DE EXPOSIÇÃO, fase a fase — **uma pergunta para o R decidir**

⛔ **A janela E NÃO se auto-certifica** (§6.2: *«na dúvida, R decide — nunca a própria janela
interessada»*). Este bloco existe para o **R-pré** poder decidir com o inventário na mão, em
vez de herdar um «queimada» sem alcance definido.

**O que esta janela de facto teve no contexto, por artefacto:**

| o que li | grau | o que vi |
|---|---|---|
| ⛔ **integração + arredondamento** da biblioteca (MPL-2.0) | **T0½** | ⛔ **o LAÇO de arredondamento e a estrutura de dados de opções** — é uma implementação directa da **SPEC §5/§5.1** |
| extração da biblioteca (MPL-2.0) | T0½ | ⭐ **apenas** os `#include` do topo, o banner de licença e uma busca que não devolveu nada. ⛔ **Nenhuma linha de algoritmo.** |
| assinaturas públicas e `struct`s de opções dos módulos de extração | T0½ | interface pública (§4.1.13) |
| tutoriais da biblioteca | T0½ | a **sequência de chamadas** da API pública — não algoritmo |
| banners de licença de ~10 cabeçalhos | T0½ | só o texto legal |
| quantização por retalhos (**GPL-3.0**) | ⛔ **T2** | uma `struct` de avaliação + nomes de campo, colhidos ao conferir licença. ⛔ **Nenhum algoritmo**, e pertence à fase **F4**, que **já está construída** e **não faz parte desta obra** |
| ⭐ os *papers* (QEx 2013, MIQ 2009) | público | **a fonte real da espec** — lícita para **todos** os papéis |

**A pergunta, posta com todas as letras:**

> A parede é do **T2/T3** (o próprio §3 o diz: *«onde um agente só já era permitido sem parede
> nenhuma: T0/T0½ … A parede é só do T2/T3»*). O único T2 que esta janela leu **não é
> algoritmo** e pertence a uma fase **fora desta obra**. ⇒ **esta janela está queimada para
> qual escopo?**

**A leitura desta janela (⛔ não é veredito — é o material para o R):**

| escopo | leitura de E |
|---|---|
| ⛔ **SPEC §5/§5.1 — o arredondamento** | **QUEIMADA, e sem dúvida.** Vi o laço equivalente. Escrevê-lo aqui arriscaria convergência de expressão **e** converteria em silêncio a rota do [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md) (*clean-room dos papers*) na rota **rejeitada** (porte). |
| ⚠️ **SPEC §2–§6 — a extração** | **em aberto.** O insumo que li foi o *paper* — a mesma fonte que o Implementador usaria. Nenhuma implementação de extração entrou neste contexto. |

⛔⛔ **Enquanto o R não decidir, vale o mais restritivo: esta janela não escreve produto
nenhum deste módulo.** É a postura que o ledger inteiro sustenta, e afrouxá-la por conta
própria destruiria o valor probatório dele.

⚠️ **Modo SOLO não resolve isto:** a janela SOLO **nasce** sob as regras do BLOCO-I e delega a
leitura a subagentes. Esta nasceu como E e leu. *SOLO é para a próxima janela, não para esta.*

---

## Papel I — Implementador

_(a preencher quando a janela I abrir; declaração do §6 exigida)_
⇒ o canal de ida é [`INBOX_quadwild.md`](INBOX_quadwild.md) (append **cego**).
⇒ o bloco dele será produzido pelo **R-pré**, em `NEXT_I.md` (corrente do §10).

---

## Papel R — Revisor

| campo | valor |
|---|---|
| session-id (modo PRÉ) | `23c68c7a-90db-4316-9d14-a4efcda6af7f` |
| data | 2026-08-24 |
| janela | ⭐ **≠ a janela E** (`edbb014f-…`) — a exigência do §3.R para o modo PRÉ está cumprida |
| árvore auditada | `Worktrees/line-sculpt3d` (⚠️ **não** o primário — a pasta `cleanroom/` não existe no `main`) |

- **Modo PRÉ:** ✅ **VERDE em 2026-08-24.** Veredito, método e achados abaixo.
- **Modo PÓS:** ⏳ pendente (após paridade verde).

### §R-pré.1 — O veredito do §4.2, item a item

⛔ **A auditoria não foi feita lendo a espec contra a memória** — foi feita contra os *papers*
citados no mapa de leitura, com `grep` sobre o texto extraído deles. É o que a torna refutável.

| item do §4.2 | veredito | como foi conferido |
|---|---|---|
| texto de código, trechos, diffs | ✅ **ausente** | leitura integral + sweep verde; os únicos blocos cercados são **matemática** (a forma da transição) e o **nosso** formato de fixture |
| nomes internos do alvo | ✅ **ausente** | o glossário do §0 usa vocabulário **do *paper***, não do fonte: os cinco termos ocorrem **40 · 42 · 13 · 14 · 55** vezes no texto publicado do QEx 2013 ⇒ §4.1.13 (nome público) + §4.1.10, e a espec ainda os põe **entre parênteses**, mapeados a nomes desta casa. A vassoura de 21 identificadores **internos** reais não casou uma vez |
| comentários do original | ✅ **ausente** | — |
| wording de manual/paper verbatim ou quase | ✅ **dentro da medida** | a espec é re-descrição em português; a única passagem atribuída (*«os autores registam que esta escada adaptativa é mais eficiente…»*) é §4.1.12 com proveniência, e é **mais curta e menos específica** que a fonte |
| organização arquivo-a-arquivo / função-a-função | ✅ **ausente** | a espec organiza-se por **fases funcionais** em ordem de dependência de dados (§4.1.1 permite-o explicitamente). Não é a decomposição do fonte — é a ordem de exposição do *paper*, e ela **não é arbitrária**: não há saídas antes de nós, nem traçado antes de saídas |
| pseudo-código espelhando o original linha a linha | ✅ **ausente** | o único procedimento detalhado (§5.1, degrau 1) corresponde ao **Algorithm 1 publicado** do MIQ 2009 ⇒ §4.1.10 («se pudesse estar num paper, pode estar na espec» — aqui **está**), e a espec re-estrutura-o como escada de três degraus em prosa |
| tabela grande afinada à mão, verbatim | ✅ **ausente** | toda tabela é medição **nossa** ou análise de casos em forma fechada (4 linhas, §2.4) |

⇒ ⭐ **§4.2: VERDE. Atestado no cabeçalho da espec.**

### §R-pré.2 — ⭐⭐⭐ A pergunta que o E deixou em aberto: o ESCOPO da queima

O ledger pediu ao R que decidisse (§6.2: *«na dúvida, R decide — nunca a própria janela
interessada»*). **Decidido, e o eixo da pergunta estava errado.**

**O achado que resolve:** ⭐ **tudo o que a espec §5/§5.1 diz está PUBLICADO no *paper* de
2009**, conferido linha a linha contra o texto extraído — a escolha gulosa da variável de menor
erro, a premissa de que um erro pequeno tem impacto pequeno, a fila dos não-zeros da linha, o
resíduo, a actualização, a escada Gauss-Seidel ⇒ gradiente conjugado ⇒ factorização directa, e
a eliminação de uma variável por restrição. O paper publica-o como **`Algorithm 1`** e como
prosa da §2.1 dele.

⭐⭐ **E a prova de que a espec descende do *paper* e não de uma implementação é ela ser MENOS
específica que ele:** o *paper* dá a tolerância concreta; **a espec não a copia** — fala em
«tolerância» e «tecto» e manda **medir** a fracção que fica no degrau 1. *Uma tradução de
código herda as constantes; uma descrição herda a lei.* A espec também regista que a 1ª
redacção dizia «re-resolva» e foi **corrigida contra o paper** — o paper é o insumo operante.

| escopo | leitura do E | ⚖️ **veredito do R** |
|---|---|---|
| SPEC §5/§5.1 (o arredondamento) | «QUEIMADA, e sem dúvida» | ⚠️ **confirmado, mas por OUTRO mecanismo** — e o escopo é **escrever produto**, não a espec |
| SPEC §2–§6 (a extração) | «em aberto» | ✅ **não queimado** — nenhuma implementação de extração entrou naquele contexto (a própria travessia diz: só `#include`s e o banner) |
| a ESPEC como artefacto | — | ✅ **não é obra derivada.** Descende do *paper*, e o §R-pré.2 acima é a demonstração |

⛔⛔ **A correcção de eixo, e é o que importa:** a exposição do E **não foi ao alvo GPL** — ao
GPL ele viu licenças, manifestos e uma `struct` de campos, que pela régua do §6.2 é **relance**
e pertence a uma fase fora desta obra. O que ele viu foi a **biblioteca MPL-2.0**. Isso não é
violação de parede nenhuma (a MPL é permissiva e aceite pelo `deny.toml`); o risco é **outro**:
escrever o §5 com aquele laço na memória produziria plausivelmente **obra derivada de um
arquivo MPL**, cujo custo é o arquivo ficar **permanentemente público** — exactamente a Rota A
que o [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
**rejeitou**, tomada em silêncio.

⇒ ⭐ **E por isso «qual janela está queimada» é a pergunta errada.** A resposta certa é *«qual
FONTE tem de ficar fora de quem escrever o §5»* — e assim a resposta deixa de depender de
identidade de janela. É o que a correcção de parede do §R-pré.3 faz.

**Consequência operacional:** a janela I pode escrever **§2–§6 e §5**, na mesma janela, desde
que a parede corrigida esteja de pé. A janela E mantém-se fora de escrever produto neste
módulo — ⚠️ e isso já era verdade **por decisão registada no ADR-0164**, sem precisar deste
veredito.

### §R-pré.3 — ⛔⛔ ACHADO DE PAREDE (a razão de o R-pré existir), CURADO

⛔ **A espec mandava o Implementador fazer exactamente o que o BLOCO-I dele proíbe.**

Em **dois** sítios (a nota de fecho do §9 e a linha «Regenerar» do README dos fixtures) a espec
dava ao I o caminho do arnês em `~/Referencias/`. Mas o **Passo 0 do BLOCO-I** cria um deny de
`Read(~/Referencias/**)`, e o §3.I conta *porte/fork do alvo em qualquer linguagem e sob
qualquer licença* como **código do alvo**. ⇒ um I obediente à espec violava o próprio passo 0;
um I obediente ao passo 0 não conseguia executar o que a espec lhe pedia.

⚠️ **E o mecanismo torna-o concreto, não teórico:** o arnês é um consumidor **header-only** —
`control.cpp` inclui o cabeçalho onde a implementação vive (**361 linhas**, com o laço que
queimou o E). ⛔ *Um erro de compilação num consumidor header-only despeja o cabeçalho no
terminal.* A exposição involuntária conta na mesma (§6), e chegaria pela porta que a própria
espec abriu.

**Curado pelo R-pré, nos dois sítios + no cabeçalho:**

1. a nota do §9 passa a dizer que correr o arnês é **acto de E** (ou de wrapper de E que entregue
   só dados) — que é a lei do **§5 da skill**, de que a espec se tinha afastado em silêncio;
2. a linha «Regenerar» do README dos fixtures idem, com o mecanismo nomeado;
3. o cabeçalho ganha uma **denylist de CAMINHOS** ao lado da de URLs — ⚠️ *a de URLs não bastava,
   porque as duas implementações estão **neste disco***. Ela nomeia `~/Referencias/**` e
   `ph2d-quadbench/oracle/**`, ⚠️ este último **irmão de `ph2d-quadbench/corpus/`**, que é nosso
   e lícito: o corpus e o clone GPL vivem lado a lado, e `/home/enio/Documentos/Projetos` é
   directório de trabalho por omissão das janelas desta casa.

⛔ **Nada de algoritmo foi acrescentado à espec por este R** — as três correcções **retiram** uma
instrução e nomeiam um caminho proibido. O §3.R proíbe ao R escrever ou ditar produto, e nenhuma
delas o faz.

### §R-pré.4 — Achado de instrumento: o gate nº4 era CEGO ao que a §2.1 prevê

O verificador de mapa (declarado *«o gate nº4 da espec, executável»*) saltava por `continue`
**mudo** toda aresta cuja imagem no domínio é degenerada. Ele imprimia `arestas_interiores` e um
`n` menor, **sem nunca dizer que a diferença existia**.

⚠️ **E a diferença é exactamente o fenómeno que a espec §2.1 manda colapsar antes de tudo** — um
gate cego ao caso que o seu próprio insumo prevê. Medido agora: **`1` aresta em CADA uma das duas
peças** (`6 144 → 6 143`, `10 152 → 10 151`), o que também explica dois números que a espec e o
README davam como «arestas interiores» e eram, na verdade, **arestas medidas**.

⇒ **Curado:** o verificador conta e imprime `degeneradas_no_dominio`, e as duas tabelas passam a
mostrar as duas colunas. *É a lei da casa — ponha a contagem ao lado — aplicada ao instrumento que
gateia os fixtures.* ⛔ Nenhum número de resíduo mudou; o veredito dos dois fixtures continua ✓.

### §R-pré.5 — Sweep (§7.1), com CONTROLE POSITIVO

⚠️ **O sweep foi invocado pelo caminho ABSOLUTO do primário** — `scripts/cleanroom-sweep.sh`
**não existe nesta worktree** (§R-pré.6).

| alvo do sweep | resultado |
|---|---|
| ⭐ **controle positivo** (arquivo semeado com uma entrada decodificada da vassoura) | ✅ **✗ exit 1** — o instrumento **funciona**, e foi provado antes de se acreditar num verde |
| espec + `fixtures/` + README + `NEXT_R-PRE.md` + INBOX (os artefactos que cruzam a parede) | ✓ limpo, exit 0 |
| ledger + triagem + achado (zona E/R) | ✓ limpo, exit 0 |
| `crates/` + `shells/` + `docs/3D/` (árvore rastreada) | ✓ limpo, exit 0 |
| `--git-history` sobre `docs/3D/cleanroom` (mensagens **e** patches) | ✓ limpo, exit 0 |

### §R-pré.6 — ⏳ Os dois BLOQUEIOS que ficam, e nenhum é do R resolver

1. ⛔ **A pasta `cleanroom/` não existe no `main`, e o bloco de abertura de linha nasce do `main`.**
   O E já o reportou («Bloqueio operacional a montante»). ⇒ **uma worktree I aberta pelo MODELO
   padrão não veria a espec.** ⭐ **Saída que não exige ordem de integração:** abrir a worktree do I
   a partir de **`line/sculpt3d`** em vez de `main` — é abertura de linha, não integração, e está
   dentro do Modo L. O `NEXT_I.md` já sai com essa alteração **explícita** no passo 4.
   *A alternativa é o Enio ordenar a integração desta pasta primeiro; as duas servem, e a escolha é dele.*
2. ⚠️ **`scripts/cleanroom-sweep.sh` não é rastreado por git em árvore nenhuma** — ele existe só
   como arquivo solto no primário. ⛔ *Uma ferramenta fora do repo não existe nas outras máquinas,
   e um script novo não existe nas árvores que nasceram antes dele* (CLAUDE.md §2). Não bloqueia o
   **I** (I não roda o sweep), mas bloqueia o **R-pós** de qualquer janela que não seja esta, e a
   reprodutibilidade da prova. ⇒ **tem de ser commitado junto com a pasta `cleanroom/`.**

### §R-pré.7 — Reconciliações do registo (o ledger contradizia-se a si próprio)

⚠️ **O ledger é a prova (§6); uma afirmação falsa dentro dele custa mais do que o que ela
esconde.** Duas foram corrigidas, e **as duas eram a favor de quem as escreveu**, o que é
precisamente o padrão que uma auditoria independente existe para apanhar:

1. a secção da **vassoura** dizia *«NÃO gerada, de propósito»* — e o arquivo **existe, com 21
   entradas, e é o que varre tudo**. A [`TRIAGEM §7.4`](TRIAGEM_quad_remesh.md) já dizia a verdade
   (*«a que existe foi montada só do que esta janela de facto viu»*). O ledger foi alinhado à
   triagem: **existe uma vassoura parcial, de proveniência lícita; a completa é o 1º acto da Rota B.**
2. o **«Veredito da triagem»** no pé do ledger dizia *«T0½ ⇒ PARADO e REPORTADO ao Enio»* — estado
   **anterior** ao [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md),
   que está **Accepted** e escolheu a rota dos *papers*. Um leitor que parasse no pé do ledger
   concluiria que a decisão ainda não foi tomada. Marcado como **superado**, com o ADR ao lado.

---

## Espec entregue

| versão | caminho | `sha256` (16) |
|---|---|---|
| 1 (2026-08-24) | [`SPEC_extracao_de_malha_quad.md`](SPEC_extracao_de_malha_quad.md) | `4455ee56e1ae6ae5` |

⚠️ **A espec foi corrigida três vezes no dia da entrega**, sempre por medição — as correcções
estão no corpo dela e na [`TRIAGEM §5-bis`](TRIAGEM_quad_remesh.md). A auditoria R-pré incide
sobre **esta** versão.

---

## Incidentes

⚠️ **Um, e é do repositório, não da janela.** O sweep de abertura achou **~460 notas** no repo
**inteiro** a citar arquivo de fonte interno de alvo restrito, **25 com transcrição** —
anterior a esta linha e à própria skill. ⛔ **DESCRITO, nunca reproduzido**, em
[`ACHADO_proveniencia_por_nome_interno.md`](ACHADO_proveniencia_por_nome_interno.md).

- **Régua do §6.2:** todas são *assinatura/nome isolado* ⇒ **relance**, não substancial.
  ⇒ **nenhuma janela é queimada por elas.**
- **Curado:** a família do quad remesh está a **ZERO na árvore rastreada** (com controlo
  positivo vermelho), e a memória contaminada foi re-expressa.
- ⏳ **Aberto:** a família Blender (~420 de Classe A, ~21 de Classe B) — exige vassoura
  própria, que **não existe**.

---

## Handoff da corrente (§10)

| passo | estado |
|---|---|
| **E ⇒ R-pré** | ⭐ **entregue** em [`NEXT_R-PRE.md`](NEXT_R-PRE.md), 2026-08-24, **sweep verde sobre o próprio handoff** |
| **R-pré ⇒ I** | ⭐ **entregue** em [`NEXT_I.md`](NEXT_I.md), 2026-08-24, **sweep verde sobre o próprio handoff** (as DUAS mensagens do Modo L) |
| I ⇒ R-pós | ⏳ pendente |

⛔ **Bloqueio operacional a montante, e não é do E resolvê-lo:** o passo 9 do BLOCO-E manda
commitar espec+ledger+vassoura+README **no `main` do primário antes de a linha I abrir**.
⚠️ Nesta casa isso é **integração**, e o [`CLAUDE.md` §0.7](../../../CLAUDE.md) reserva-a a uma
**ordem explícita do Enio** por um agente integrador dedicado. ⇒ **reportado, não executado.**
*Hoje `main` não vê um único arquivo desta pasta, e uma worktree nova nasce dele.*

---

## Fechamento R

_(pendente — modo PÓS)_

---

## Veredito da triagem (§2) — 2026-08-24 · ⚠️ **SUPERADO no mesmo dia (nota do R-pré)**

> ⛔ **Leia o parágrafo seguinte antes deste bloco.** Este veredito é o estado **anterior** à
> decisão, e quem parasse aqui concluiria que ela ainda não foi tomada.
> ⭐ **A decisão foi tomada e está `Accepted`:**
> [ADR-0164](../../architecture/decisions/0164-quad-extraction-is-clean-room-from-papers-the-mpl-library-is-an-oracle.md)
> — **clean-room dos *papers*; a biblioteca MPL-2.0 fica FORA, como oráculo.** A Rota A (porte
> fiel T0½) foi **rejeitada com motivo medido**: arquivos permanentemente públicos no subsistema
> mais valioso, descarte da cadeia própria, e ⛔ **falha em 3 de 7 peças do nosso corpus**.
> ⚠️ O degrau T0½ **não foi queimado** — só não foi tomado primeiro.

**O que este bloco registava, e continua verdadeiro como FACTO de triagem:**

⭐⭐⭐ **A porta mais barata era T0½, não T2.** As duas fases que bloqueiam o produto hoje — o
**arredondamento inteiro** das translações de costura e a **extracção** a partir do mapa de grade
inteira — existem sob **MPL-2.0**, licença **já aceite** pelo `deny.toml` desta casa.

⇒ **PARADO e REPORTADO ao Enio**, conforme a ordem embutida no passo 1 do BLOCO-E — e foi
dessa paragem que saiu o ADR-0164.
As três rotas, com preço medido: [`TRIAGEM_quad_remesh.md` §5](TRIAGEM_quad_remesh.md).
