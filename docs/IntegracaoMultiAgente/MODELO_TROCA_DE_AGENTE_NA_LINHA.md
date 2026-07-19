# MODELO — Troca de agente numa linha viva (e retomada pós-integração)

> **Fonte única** do bloco que o Enio cola quando um agente NOVO assume uma linha que
> **já existe** — troca de janela de contexto, ou retomada depois de a linha ter
> integrado ao main. Para abrir uma linha do ZERO, use
> [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md). (DIRETRIZ §1.5.1–§1.5.2)

## Por que este doc existe

O agente novo trabalha no **`main`** em vez da linha dele. Já aconteceu várias vezes, com
vários agentes. **Não é desatenção — é o desenho da bancada:**

1. Toda janela abre na **raiz do repo primário**, que está em `main`. É assim de propósito
   (uma pasta só, uma janela por agente).
2. **O mesmo path relativo existe nas duas árvores.** `crates/ph2d-foo/src/lib.rs` abre na
   raiz e na worktree. Editar o da raiz **compila, testa e commita** — sem erro, sem aviso.
3. O agente novo herda um handoff que nomeia arquivos por path relativo. Ele abre o path
   que lhe deram, a partir de onde está — e onde ele está é `main`.

A falha é **silenciosa e só aparece na integração**, quando a linha não tem o trabalho e o
primário tem commits que ninguém pediu. Por isso a defesa não pode ser "lembre-se": tem de
ser uma **verificação barata, feita antes de ler qualquer código, e repetível a qualquer
momento**.

---

## O BLOCO (copie daqui pra baixo; escreva o módulo SÓ na 1ª linha)

```
═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você assume uma linha em andamento. Sua linha: line/<módulo>

O nome após "line/" é o SEU MÓDULO ($MODULO nos comandos: substitua
pelo nome literal — env não persiste entre chamadas de shell).
Sua branch:    line/$MODULO
Sua worktree:  Worktrees/line-$MODULO/   (ela JÁ EXISTE — não crie)

⛔ ANTES DE LER OU EDITAR QUALQUER ARQUIVO, execute a FASE 0.
   Você está começando na RAIZ do repo, que está em `main`. Os MESMOS
   paths relativos existem aqui e na sua worktree: abrir `crates/...`
   daqui edita a ÁRVORE ERRADA, e isso compila e commita sem um único
   erro. Ninguém descobre até a integração.

FASE 0 — ONDE VOCÊ ESTÁ (execute já, sem pedir confirmação):
1. cd Worktrees/line-$MODULO && pwd && git branch --show-current
      → pwd TEM de terminar em /Worktrees/line-$MODULO
      → a branch TEM de ser line/$MODULO
      Deu `main`, ou a pasta não existe? PARE e reporte ao Enio: ou o
      módulo está escrito errado, ou a linha nunca foi aberta (aí o
      bloco certo é o MODELO_ABERTURA_LINHA.md).
2. git log --oneline -5 && git status -sb
      → é daqui que o agente anterior parou. Árvore suja = trabalho não
        commitado dele: NÃO descarte, commite (`--no-verify`).

FASE 1 — RETOMADA (só se a linha já integrou ao main alguma vez):
3. git rebase main
      → obrigatório no início de CADA jornada (DIRETRIZ §1.5.2.3).
        Conflito em Cargo.lock ou arquivo GERADO (registry-init,
        chrome/mod.rs): NUNCA na mão — regenere (DIRETRIZ §1.5.5).
        Conflito em código FORA dos seus arquivos = colisão de
        mesmo-símbolo: PARE e reporte ao Enio.
4. cargo check -p <sua crate principal>
      → confirma que a base nova não quebrou você. 1º build pode ser
        frio (minutos): é esperado, não investigue.

FASE 2 — ESTADO (leia, nesta ordem, DENTRO da worktree):
5. O handoff/tracker do SEU módulo (`docs/HANDOFF_*<módulo>*.md` e/ou
   `docs/<Módulo>/`) — é onde o agente anterior deixou o que já foi
   decidido, medido e REPROVADO. Ler antes evita reconstruir o que já
   foi tentado e re-litigar decisão fechada.
6. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md — inteira, e
   RELEIA a cada passo, como ela manda.
7. As REGRAS PERMANENTES DA SESSÃO (A–H) do
   docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md — elas valem
   IGUAIS para você. Não estão copiadas aqui de propósito: duas cópias
   da mesma regra divergem.
8. Reporte: "Assumi line/$MODULO em Worktrees/line-$MODULO (HEAD <sha>).
   <1 linha do estado>. Aguardo a tarefa." — e PARE.

REGRA DE OURO DESTA SESSÃO (além das A–H):
⛔ Na dúvida sobre onde você está, `pwd`. Antes de qualquer commit,
   `git branch --show-current`. Custa um segundo; a alternativa é
   descobrir na integração que o trabalho foi para o main.
   Se você JÁ escreveu no main sem querer: NÃO apague nada — reporte
   ao Enio e aponte o procedimento de resgate (doc deste bloco, §
   "Resgate").
═══════════════════════════════════════════════════════════════════
```

---

## Resgate — "escrevi no `main` sem querer"

Acontece. **Não apague nada**; o trabalho está inteiro, só está na árvore errada.
Reporte ao Enio e execute o caso que se aplica:

**Já commitou no `main` (local, não pushado):**

```bash
cd ~/Documentos/Projetos/PH2D        # primário
git log --oneline origin/main..HEAD  # o que é seu? (confira ANTES)
git branch --show-current            # confirme: main

# leva os commits para a linha, sem duplicá-los no main
git -C Worktrees/line-<módulo> cherry-pick <sha-mais-antigo>^..<sha-mais-novo>
git reset --hard origin/main         # ⚠️ só depois do cherry-pick VERDE
```

**Ainda não commitou (mudanças soltas no `main`):**

```bash
cd ~/Documentos/Projetos/PH2D
git diff > /tmp/resgate.patch        # inclua --cached se houver staged
git -C Worktrees/line-<módulo> apply /tmp/resgate.patch
git checkout -- <SÓ os seus arquivos>   # NUNCA `git checkout .`
```

⚠️ **`git reset --hard` e `git checkout .` no primário destroem o trabalho de OUTRAS
linhas e sessões** que também vivem ali. Confira `git status` antes e restaure **apenas os
seus arquivos, por caminho**.

---

## Quando NÃO usar este bloco

| Situação | Use |
|---|---|
| Linha nova, do zero | [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md) |
| Fundir linhas ao main | agente integrador (DIRETRIZ §1.5.3), por ordem do Enio |
| Encerrar linha morta | `MODELO_ABERTURA_LINHA.md` §"Encerrar uma linha" |
| Você é a janela do primário (setup/integração/ship) | não code em `main` — DIRETRIZ §1.5.8 |
