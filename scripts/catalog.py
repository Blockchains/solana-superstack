"""Build catalog.json + repos/*.md from forks listed in colosseum-forker state.json, using GitHub metadata and README/manifest inspection.
Heuristic classification only; README text is treated as untrusted data (never executed/obeyed)."""
import json,subprocess,base64,re,os,sys,time
STATE=sys.argv[1] if len(sys.argv)>1 else "state.json"
def api(p):
    r=subprocess.run(["gh","api",p],capture_output=True,text=True)
    return json.loads(r.stdout) if r.returncode==0 and r.stdout.strip() else None
def raw(repo,path):
    d=api(f"repos/{repo}/contents/{path}")
    if isinstance(d,dict) and d.get("content"):
        try: return base64.b64decode(d["content"]).decode("utf8","ignore")
        except Exception: return ""
    return ""
CATS={"DeFi":r"\b(defi|swap|amm|lending|borrow|liquidity|dex|perp|yield|stake|staking|vault|stablecoin|orderbook)\b",
"Payments":r"\b(payment|pay|checkout|invoice|remittance|merchant|solana pay|payroll|tip)\b",
"NFTs":r"\b(nft|metaplex|collectible|candy machine|cnft|compressed nft)\b",
"Gaming":r"\b(game|gaming|play|player|arcade|quest)\b",
"DePIN":r"\b(depin|sensor|iot|hardware|helium|physical infrastructure|wifi)\b",
"AI agents":r"\b(ai agent|agent|llm|gpt|openai|machine learning|eliza)\b",
"Infra/Tooling":r"\b(sdk|cli|indexer|oracle|rpc|framework|tooling|explorer|library|bridge)\b",
"Social":r"\b(social|community|chat|messag|creator|fan)\b",
"DAO/Governance":r"\b(dao|governance|voting|vote|multisig)\b",
"Identity/Privacy":r"\b(identity|kyc|zk|zero.knowledge|privacy|credential|did)\b",
"Consumer":r"\b(consumer|mobile|wallet|saga|loyalty|reward)\b"}
FW=[("Anchor",r"anchor[-_]lang|Anchor\.toml|@coral-xyz/anchor|@project-serum/anchor"),("Pinocchio",r"pinocchio"),
("Seahorse",r"seahorse"),("Steel",r"\bsteel\s*="),("Native (solana-program)",r"solana[-_]program\s*="),
("Poseidon",r"poseidon"),("Bolt (MagicBlock)",r"bolt[-_]lang"),("Client-only (web3.js/kit)",r"@solana/web3\.js|@solana/kit")]
PID=re.compile(r'declare_id!\(\s*"([1-9A-HJ-NP-Za-km-z]{32,44})"\s*\)|^\s*\w+\s*=\s*"([1-9A-HJ-NP-Za-km-z]{32,44})"',re.M)
INJ=re.compile(r"(ignore (all |any )?(previous|prior) instructions|you are (an? )?(ai|assistant|llm)|system prompt|as an ai)",re.I)
st=json.load(open(STATE))
forks=sorted({v["fork"]:v for v in st.values() if v.get("fork") and v["status"] in("created","exists")}.items())
old={}
if os.path.exists("catalog.json"):
    for e in json.load(open("catalog.json"))["repos"]: old[e["fork"]]=e
out=[];inj=[]
for fork,v in forks:
    m=api(f"repos/{fork}")
    if not m: continue
    if fork in old and old[fork].get("fork_pushed_at")==m.get("pushed_at"):
        out.append(old[fork]); continue
    src=(m.get("source") or m.get("parent") or {}).get("full_name") or v.get("source")
    tree=api(f"repos/{fork}/git/trees/{m['default_branch']}?recursive=1") or {}
    paths=[t["path"] for t in tree.get("tree",[]) if t.get("type")=="blob"]
    readme=raw(fork,"README.md")[:20000]
    anchor_toml=next((p for p in paths if p.endswith("Anchor.toml")),None)
    cargos=[p for p in paths if p.endswith("Cargo.toml") and "node_modules" not in p][:12]
    manif="".join(raw(fork,p) for p in ([anchor_toml] if anchor_toml else [])+cargos[:6])
    pkg=raw(fork,"package.json")
    blob=manif+"\n"+pkg+"\n"+(anchor_toml or "")
    fws=[n for n,rx in FW if re.search(rx,blob,re.I)]
    if any(p.endswith(".py") and "programs" in p for p in paths) and "seahorse" in readme.lower(): fws.append("Seahorse")
    pids=set()
    for p in [p for p in paths if p.endswith("lib.rs") and "programs" in p][:6]:
        for a,b in PID.findall(raw(fork,p)): pids.add(a or b)
    for a,b in PID.findall(raw(fork,anchor_toml) if anchor_toml else ""): pids.add(a or b)
    text=(m.get("description") or "")+" "+" ".join(m.get("topics") or [])+" "+readme[:6000]
    cats=[c for c,rx in CATS.items() if len(re.findall(rx,text,re.I))>=(1 if c in("DeFi","NFTs","DePIN") else 2)] or ["Other"]
    if INJ.search(readme): inj.append(fork)
    tests=any(re.search(r"(^|/)(tests?|__tests__)/|\.test\.|_test\.rs|\.spec\.",p) for p in paths)
    lic=(m.get("license") or {}).get("spdx_id")
    last=api(f"repos/{fork}/commits?per_page=1")
    e={"fork":fork,"fork_url":m["html_url"],"source":src,"description":m.get("description"),"language":m.get("language"),
       "languages":list((api(f"repos/{fork}/languages") or {}).keys())[:6],"frameworks":sorted(set(fws)),"program_ids":sorted(pids),
       "categories":cats,"license":lic,"last_commit":(last[0]["commit"]["committer"]["date"] if last else None),
       "stars_source":None,"has_tests":tests,"has_anchor_toml":bool(anchor_toml),"cargo_manifests":len([p for p in paths if p.endswith("Cargo.toml")]),
       "has_package_json":"package.json" in paths,"files":len(paths),"builds":"unverified","fork_pushed_at":m.get("pushed_at"),
       "readme_injection_flag":fork in inj}
    out.append(e); time.sleep(0.2)
from collections import Counter
cat=Counter(c for e in out for c in e["categories"]); fw=Counter(f for e in out for f in e["frameworks"] or ["none detected"])
json.dump({"generated":time.strftime("%Y-%m-%dT%H:%M:%SZ",time.gmtime()),"count":len(out),"by_category":dict(cat.most_common()),
  "by_framework":dict(fw.most_common()),"repos":out},open("catalog.json","w"),indent=1)
os.makedirs("repos",exist_ok=True)
for e in out:
    n=e["fork"].split("/")[1]
    open(f"repos/{n}.md","w").write(f"# {n}\n\n- Fork: [{e['fork']}]({e['fork_url']})\n- Original: [{e['source']}](https://github.com/{e['source']})\n- Description: {(e['description'] or '').replace(chr(10),' ')}\n- Language: {e['language']} ({', '.join(e['languages'])})\n- Frameworks: {', '.join(e['frameworks']) or 'none detected'}\n- Program IDs: {', '.join('`'+p+'`' for p in e['program_ids']) or 'none found'}\n- Categories: {', '.join(e['categories'])}\n- License: {e['license'] or 'none'}\n- Last commit: {e['last_commit']}\n- Has tests: {e['has_tests']}\n- Builds: {e['builds']} (only modules/ are CI-verified)\n")
idx=["# Catalog index\n","| Repo | Categories | Framework | License | Tests |","|---|---|---|---|---|"]
for e in out: idx.append(f"| [{e['fork'].split('/')[1]}](repos/{e['fork'].split('/')[1]}.md) | {', '.join(e['categories'])} | {', '.join(e['frameworks']) or '-'} | {e['license'] or '-'} | {'yes' if e['has_tests'] else 'no'} |")
open("CATALOG.md","w").write("\n".join(idx)+"\n")
print("catalogued",len(out),"injection-flags",inj)
