import json, sys, re, hmac, hashlib, secrets, unicodedata
src, dst = sys.argv[1], sys.argv[2]
SALT = secrets.token_bytes(32)  # nunca gravado: o mapeamento não se inverte
TEXT_KEYS = {"name","desc","text","fullName","username","initials","bio","url","shortUrl",
             "avatarUrl","avatarHash","email","fileName","displayName","nodeId","shortLink"}
DROP_KEYS = {"descData","bioData","previews","emoji"}
LOW, UP, DIG = "abcdefghijklmnopqrstuvwxyz", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", "0123456789"
ACC_L, ACC_U = "áéíóúãõçâêô", "ÁÉÍÓÚÃÕÇÂÊÔ"
def run_map(w):
    h = hmac.new(SALT, w.encode(), hashlib.sha256).digest()
    while len(h) < len(w): h += hashlib.sha256(h).digest()
    out = []
    for c, b in zip(w, h):
        if c in DIG: out.append(DIG[b % 10])
        elif c in ACC_L: out.append(ACC_L[b % len(ACC_L)])
        elif c in ACC_U: out.append(ACC_U[b % len(ACC_U)])
        elif c.isupper(): out.append(UP[b % 26])
        elif c.isalpha(): out.append(LOW[b % 26])
        else: out.append(c)
    return "".join(out)
def text(s):
    return re.sub(r"\w+", lambda m: run_map(m.group(0)), s)
def walk(v, key=None):
    if isinstance(v, dict):
        return {k: (None if k in DROP_KEYS else walk(x, k)) for k, x in v.items()}
    if isinstance(v, list):
        return [walk(x, key) for x in v]
    if isinstance(v, str) and (key in TEXT_KEYS or key == "labelNames" or "@" in v or "trello.com" in v):
        return text(v)
    return v
d = json.load(open(src))
d["labelNames"] = {k: text(x) for k, x in d.get("labelNames", {}).items()}
out = walk(d)
raw_out = json.dumps(out, ensure_ascii=False, indent=1)
# Prova: nenhuma palavra (≥ 4 letras) dos campos de texto do original sobrevive.
words = set()
def collect(v, key=None):
    if isinstance(v, dict):
        for k, x in v.items(): collect(x, k)
    elif isinstance(v, list):
        for x in v: collect(x, key)
    elif isinstance(v, str) and (key in TEXT_KEYS or key in DROP_KEYS):
        words.update(w.lower() for w in re.findall(r"[^\W\d_]{4,}", v))
collect(d)
low = raw_out.lower()
leaks = sorted(w for w in words if re.search(r"(?<![^\W\d_])" + re.escape(w) + r"(?![^\W\d_])", low))
emails = re.findall(r"[\w.+-]+@[\w-]+\.[a-z]{2,}", raw_out)
print("palavras do original:", len(words), "| sobreviventes:", leaks[:40])
print("emails reais restantes:", [e for e in emails if e in json.dumps(d, ensure_ascii=False)][:5])
assert len(words) > 200
open(dst, "w").write(raw_out + "\n")
print("cards", len(out["cards"]), "lists", len(out["lists"]), "bytes", len(raw_out))
