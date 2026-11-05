#!/usr/bin/env python3
# Binario sem simbolos: cada endereco de retorno identifica-se pelos textos e pelos Location de panico
# que o codigo em volta referencia (lea rip-relativo -> rodata, ou -> struct Location -> caminho .rs).
import re, struct, subprocess
S = "/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/f179eb1b-154c-4d38-8d41-69002976ee7d/scratchpad"
BIN = "/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/target/smoke/ph2d-host-desktop"
JANELA = 0x300
dados = open(BIN, "rb").read()
fases, secs = [], {}
e_phoff, = struct.unpack_from("<Q", dados, 32)
e_shoff, = struct.unpack_from("<Q", dados, 40)
e_entry, = struct.unpack_from("<Q", dados, 24)
e_phentsize, e_phnum, e_shentsize, e_shnum, e_shstrndx = struct.unpack_from("<HHHHH", dados, 54)
for i in range(e_phnum):
    t, _, off, va, _, fs, _, _ = struct.unpack_from("<IIQQQQQQ", dados, e_phoff + i * e_phentsize)
    if t == 1:
        fases.append((va, va + fs, off))
sh = [struct.unpack_from("<IIQQQQIIQQ", dados, e_shoff + i * e_shentsize) for i in range(e_shnum)]
strs = sh[e_shstrndx][4]
for s in sh:
    fim = dados.index(b"\0", strs + s[0])
    secs[dados[strs + s[0]:fim].decode()] = s
def f_off(va):
    for a, b, o in fases:
        if a <= va < b:
            return va - a + o
    return None
reloc = {}
r = secs.get(".rela.dyn")
assert r, "sem .rela.dyn"
for o in range(r[4], r[4] + r[5], 24):
    ro, info, add = struct.unpack_from("<QQq", dados, o)
    if info & 0xffffffff == 8:
        reloc[ro] = add
def texto(va, n=None):
    o = f_off(va)
    if o is None:
        return None
    b = dados[o:o + (n or 160)]
    m = re.match(rb"[\x20-\x7e\xc0-\xff\x80-\xbf]{6,}", b)
    return m.group(0).decode("utf-8", "replace") if m else None
def resolve(va):
    # struct Location { file: &str(ptr,len), line, col } ou um &str direto
    p = reloc.get(va)
    if p is not None:
        ln = struct.unpack_from("<Q", dados, f_off(va) + 8)[0] if f_off(va) is not None else 0
        if 4 < ln < 300:
            t = texto(p, ln)
            if t:
                li = struct.unpack_from("<I", dados, f_off(va) + 16)[0]
                return f"{t}:{li}" if t.endswith(".rs") else t
    return texto(va)
linhas = open(f"{S}/pilhas_travado.txt").read().splitlines()
ret = int([l for l in linhas if l.startswith("#30 ")][0].split()[1], 16)
base = (ret - e_entry) & ~0xfff
print(f"entrada={e_entry:#x} ret={ret:#x} base={base:#x} desvio={ret - base - e_entry:#x}")
pilhas, t = {}, None
for l in linhas:
    if l.startswith("TID "):
        t = l.split()[1].rstrip(":")
        pilhas[t] = []
    elif l.startswith("#") and t:
        pilhas[t].append(int(l.split()[1], 16))
# so as pilhas fora do formato "ocioso"; agrupa as iguais
grupos = {}
for t, p in pilhas.items():
    grupos.setdefault(tuple(p), []).append(t)
comuns = {}
for pil, ts in sorted(grupos.items(), key=lambda kv: -len(kv[0])):
    print(f"\n=== fios {','.join(ts[:6])}{'...' if len(ts) > 6 else ''} ({len(ts)}), {len(pil)} quadros")
    if len(ts) > 3 and len(pil) < 12:
        print("  (forma ociosa do pool; nao identificada)")
        continue
    for i, a in enumerate(pil):
        v = a - base
        if not (secs[".text"][3] <= v < secs[".text"][3] + secs[".text"][5]):
            print(f"  #{i:<2} {a:#x} (fora do binario)")
            continue
        if v not in comuns:
            d = subprocess.run(["objdump", "-d", "--no-show-raw-insn", f"--start-address={v - JANELA}", f"--stop-address={v + JANELA}", BIN], capture_output=True, text=True).stdout
            achados = []
            for m in re.finditer(r"^\s*([0-9a-f]+):.*#\s*([0-9a-f]+)", d, re.M):
                alvo, onde = int(m.group(2), 16), int(m.group(1), 16)
                t_ = resolve(alvo)
                if t_ and (".rs" in t_ or " " in t_):
                    achados.append((abs(onde - v), t_))
            vistos, top = set(), []
            for dist, x in sorted(achados):
                if x not in vistos:
                    vistos.add(x)
                    top.append(f"{dist:#x}:{x[-78:]}")
            comuns[v] = top[:3]
        print(f"  #{i:<2} {v:#x}  " + " | ".join(comuns[v]))
