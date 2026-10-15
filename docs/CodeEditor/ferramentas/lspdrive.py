import subprocess,json,time,sys,os,threading
lsp,defs,root=sys.argv[1],sys.argv[2],sys.argv[3]
p=subprocess.Popen([lsp,"lsp",f"--definitions:@ph2d={defs}"],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL)
msgs={};lock=threading.Condition()
def send(o):
    b=json.dumps(o).encode(); p.stdin.write(b"Content-Length: %d\r\n\r\n"%len(b)+b); p.stdin.flush()
def reader():
    f=p.stdout
    while True:
        h={}
        while True:
            l=f.readline()
            if not l: return
            l=l.decode().strip()
            if not l: break
            k,v=l.split(":",1); h[k.lower()]=v.strip()
        body=json.loads(f.read(int(h["content-length"])))
        with lock:
            if "id" in body and "method" not in body: msgs[body["id"]]=(time.perf_counter(),body)
            elif "id" in body and "method" in body:
                res=None
                if body["method"]=="workspace/configuration": res=[{"platform":{"type":"standard"}} for _ in body["params"]["items"]]
                send({"jsonrpc":"2.0","id":body["id"],"result":res})
            lock.notify_all()
threading.Thread(target=reader,daemon=True).start()
nid=[0]
def req(m,params):
    nid[0]+=1;i=nid[0];t=time.perf_counter();send({"jsonrpc":"2.0","id":i,"method":m,"params":params})
    with lock: lock.wait_for(lambda:i in msgs,timeout=60)
    t2,b=msgs[i]; return (t2-t)*1000,b
def rss():
    for l in open(f"/proc/{p.pid}/status"):
        if l.startswith("VmRSS"): return l.split()[1]
def labels(b):
    r=b.get("result"); its=r["items"] if isinstance(r,dict) else (r or []); return [x["label"] for x in its][:12]
t0=time.perf_counter()
ms,b=req("initialize",{"processId":os.getpid(),"rootUri":"file://"+root,"capabilities":{"textDocument":{"completion":{"completionItem":{"snippetSupport":True}},"hover":{"contentFormat":["markdown"]}},"workspace":{"configuration":True}},"initializationOptions":{}})
print(f"initialize {ms:.0f} ms; serverInfo={b['result'].get('serverInfo')}")
send({"jsonrpc":"2.0","method":"initialized","params":{}})
for name in ["small.luau","big.luau"]:
    uri="file://"+root+"/"+name; text=open(root+"/"+name).read()
    send({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"luau","version":1,"text":text}}})
    ms,b=req("textDocument/diagnostic",{"textDocument":{"uri":uri}})
    items=(b.get("result") or {}).get("items",b)
    print(f"{name}: pull diagnostics after open {ms:.0f} ms, n={len(items) if isinstance(items,list) else items}; RSS {rss()} kB")
    if isinstance(items,list): [print("   ",d["range"]["start"],d["message"][:80]) for d in items[:4]]
    ms,b=req("textDocument/diagnostic",{"textDocument":{"uri":uri}}); print(f"   2nd diagnostic (cached) {ms:.0f} ms")
uri="file://"+root+"/small.luau"
ms,b=req("textDocument/completion",{"textDocument":{"uri":uri},"position":{"line":1,"character":15}}); print(f"completion after 'ph2d.' {ms:.0f} ms -> {labels(b)}")
ms,b=req("textDocument/hover",{"textDocument":{"uri":uri},"position":{"line":1,"character":6}}); print(f"hover on e {ms:.0f} ms -> {json.dumps(b.get('result'))[:200]}")
ms,b=req("textDocument/definition",{"textDocument":{"uri":uri},"position":{"line":2,"character":0}}); print(f"definition of e {ms:.0f} ms -> {json.dumps(b.get('result'))[:200]}")
ms,b=req("textDocument/rename",{"textDocument":{"uri":uri},"position":{"line":1,"character":6},"newName":"hero"}); print(f"rename e {ms:.0f} ms -> {json.dumps(b.get('result'))[:200]}")
text=open(root+"/small.luau").read()+"\nlocal z = e:"
send({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":text}]}})
ms,b=req("textDocument/completion",{"textDocument":{"uri":uri},"position":{"line":7,"character":12}}); print(f"completion after edit 'e:' {ms:.0f} ms -> {labels(b)}")
bu="file://"+root+"/big.luau"; bt=open(root+"/big.luau").read()
for k in range(3):
    assert bt.count("return M")==1, bt.count("return M")
    bt=bt.replace("return M", f"local q{k} = M.f1(1, 'x')\nreturn M")
    send({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":bu,"version":3+k},"contentChanges":[{"text":bt}]}})
    ms,b=req("textDocument/diagnostic",{"textDocument":{"uri":bu}}); print(f"big.luau edit {k}: diagnostics {ms:.0f} ms")
    nl=bt.count("\n")
    ms,b=req("textDocument/completion",{"textDocument":{"uri":bu},"position":{"line":nl,"character":0}}); print(f"big.luau edit {k}: completion {ms:.0f} ms ({len(labels(b))}+ items)")
print(f"final RSS {rss()} kB; total {(time.perf_counter()-t0)*1000:.0f} ms")
req("shutdown",None); send({"jsonrpc":"2.0","method":"exit"}); p.wait(5)
