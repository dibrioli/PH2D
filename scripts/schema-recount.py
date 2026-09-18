#!/usr/bin/env python3
"""⭐ **RE-CONTA um degrau de `PROJECT_SCHEMA` no meio de um rebase.** Corra-o DENTRO do conflito.

    bash scripts/ph2d-run.sh true                 # (nada a fazer; o script e' puro texto)
    python3 scripts/schema-recount.py <valor>     # <valor> = o que o degrau passa a ter no main
    git add -- shells/desktop/src/project_schema*.rs && git rebase --continue

A lei (`CLAUDE.md` §5.0): *o valor certo raramente esta' em qualquer um dos dois lados do
conflito*. A linha escreveu `B -> B+1` sobre a arvore em que nasceu; o `main` esta' noutro sitio, e
⚠️ **a colisao passa MUDA quando duas linhas escrevem o MESMO literal** — o git nao sabe o que o
numero significa.

Este script preserva a escada do `main` (estagio `:2`), extrai o degrau NOVO que a linha
acrescentou (`:3` menos `:1`), renumera-o para `M-1 -> M` e **sobe a tripla do ficheiro IRMAO no
mesmo passo** — sao TRES sitios e nunca um (`CLAUDE.md` §5.0).

⚠️ **Cada passo tem `assert`, e isso ja' pagou:** a 1.a redaccao supos que a linha-ancora era igual
dos dois lados — verdade na escada (a ancora e' o prefixo do `const`, sem o numero) e **falso na
tripla**, que CONTEM o numero. Ela parou alto com `substring not found` em vez de escrever lixo.

⛔ Ele NAO decide o valor: quem o decide e' quem le^ o `main` no ficheiro. A coluna `base:` do
`collision-surface.sh` e' o MERGE-BASE e esta' desactualizada por construcao a partir da 2.a fusao
de uma rodada.

Medido na rodada de 2026-09-17: quatro degraus da `line/3DModeling` (`128->132` escritos) foram
re-contados para `140->144`, e os dois primeiros custaram uma volta a` mao cada antes de este
script existir. Ver `docs/archive/integracao-jornadas/ANATOMIA_DE_UMA_RODADA_2026-09-17.md` §5.2.
"""
import subprocess
import sys

LADDER = "shells/desktop/src/project_schema.rs"
TRIPLE = "shells/desktop/src/project_schema_tests.rs"
CONST = "pub(crate) const PROJECT_SCHEMA: u32 = "


def stage(n, path):
    r = subprocess.run(["git", "show", f":{n}:{path}"], capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def novo_bloco(base, theirs, agulha, agulha_theirs=None):
    """O texto que a LINHA acrescentou antes da linha-ancora.

    ⚠️ A ancora pode ser DIFERENTE dos dois lados: na escada ela e' o prefixo do `const` (sem o
    numero) e e' a mesma; na tripla ela CONTEM o numero, logo a da linha e' a do degrau seguinte.
    A 1.a redaccao supunha-as iguais e estourou com `substring not found` — alto, que e' a
    especie barata.
    """
    ib = base.index(agulha)
    prefixo = base[:ib]
    assert theirs.startswith(prefixo), (
        "a linha nao APENDOU: ela mexeu no texto antes da ancora — resolva a` mao"
    )
    it = theirs.index(agulha_theirs or agulha, ib)
    return theirs[ib:it]


def resolve_ladder(m_novo):
    base, ours, theirs = (stage(n, LADDER) for n in (1, 2, 3))
    bloco = novo_bloco(base, theirs, CONST)
    b_val = int(base.split(CONST)[1].split(";")[0])
    t_val = int(theirs.split(CONST)[1].split(";")[0])
    assert t_val == b_val + 1, f"a linha nao subiu exactamente 1 ({b_val} -> {t_val})"
    # re-numerar o titulo do degrau, em qualquer das duas grafias que a escada usa
    # ⚠️ TRES grafias, e a terceira custou uma parada na rodada de 2026-09-20: a escada usa
    # crase COM seta ASCII (`# \`144 -> 145\``) tanto quanto crase com seta unicode. O `assert`
    # abaixo apanhou-a alto — *um script que conhece duas formas de uma agulha nao sabe que ha' uma
    # terceira; o que o diz e' a assercao que exige que a re-numeracao tenha ACONTECIDO.*
    for a, b in ((f"# {b_val} -> {t_val}", f"# `{m_novo - 1} → {m_novo}`"),
                 (f"# `{b_val} -> {t_val}`", f"# `{m_novo - 1} → {m_novo}`"),
                 (f"# `{b_val} → {t_val}`", f"# `{m_novo - 1} → {m_novo}`")):
        bloco = bloco.replace(a, b, 1)
    assert f"{b_val} -> {t_val}" not in bloco and f"{b_val} → {t_val}" not in bloco, \
        f"o titulo do degrau nao foi re-numerado: ainda diz {b_val}->{t_val}"
    velho = f"{CONST}{m_novo - 1};\n"
    assert ours.count(velho) == 1, f"o main nao esta' em {m_novo - 1}"
    out = ours.replace(velho, bloco + f"{CONST}{m_novo};\n", 1)
    assert f"{CONST}{m_novo};" in out
    open(LADDER, "w", encoding="utf-8").write(out)
    return b_val, t_val


def resolve_triple(m_novo, b_val, t_val):
    base, ours, theirs = (stage(n, TRIPLE) for n in (1, 2, 3))
    if base is None or theirs is None or theirs == ours:
        return False
    ancora = f"        ({b_val}, 13, 22),"
    if ancora not in base:
        return False
    bloco = novo_bloco(base, theirs, ancora, f"        ({t_val}, 13, 22),")
    bloco = bloco.replace(f"PROJECT {b_val}→{t_val}:", f"PROJECT {m_novo - 1}→{m_novo}:", 1)
    assert f"PROJECT {b_val}→{t_val}:" not in bloco, "o titulo da tripla nao foi re-numerado"
    velho = f"        ({m_novo - 1}, 13, 22),\n"
    assert ours.count(velho) == 1, f"a tripla do main nao esta' em {m_novo - 1}"
    out = ours.replace(velho, bloco + f"        ({m_novo}, 13, 22),\n", 1)
    open(TRIPLE, "w", encoding="utf-8").write(out)
    return True


if __name__ == "__main__":
    m = int(sys.argv[1])  # o valor que o degrau passa a ter no main
    b, t = resolve_ladder(m)
    fez = resolve_triple(m, b, t)
    for p in (LADDER, TRIPLE):
        s = open(p, encoding="utf-8").read()
        assert "<<<<<<<" not in s and ">>>>>>>" not in s and "|||||||" not in s, f"{p} ainda tem marcadores"
    print(f"ok  degrau {b}->{t} da linha RE-CONTADO para {m-1}->{m}"
          f"{' (+ tripla)' if fez else ' (tripla intocada por este degrau)'}")
