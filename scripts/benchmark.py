#!/usr/bin/env python3
"""Frozen HTTP workload. Seeds only while the hub is stopped; retains raw distributions."""
import concurrent.futures, hashlib, json, os, pathlib, sqlite3, subprocess, tempfile, time, urllib.request
ROOT=pathlib.Path(__file__).resolve().parents[1]
BINARY=ROOT/'target/release/bf'
DURATION=10

def request(url,token,path,body=None):
    raw=None if body is None else json.dumps(body,separators=(',',':')).encode()
    req=urllib.request.Request(url+path,data=raw,headers={'Authorization':'Bearer '+token,'Content-Type':'application/json'})
    with urllib.request.urlopen(req,timeout=5) as response:return json.load(response)

def start(data):
    start=time.perf_counter()
    child=subprocess.Popen([str(BINARY),'--data-dir',str(data),'serve','--no-open'],cwd=data,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    for _ in range(300):
        try:
            endpoint=json.loads((data/'endpoint.json').read_text())
            token=request(endpoint['url'],endpoint['bootstrap'],'/v3/bootstrap',{})['token']
            return child,endpoint['url'],token,(time.perf_counter()-start)*1000
        except Exception:time.sleep(.02)
    child.kill();child.wait();raise RuntimeError('startup failed')

def stop(child):
    child.terminate();child.wait(timeout=10)

def sample(url,token,kind,index):
    start=time.perf_counter()
    try:
        if kind=='command':
            request(url,token,'/v3/commands',{'schema_version':3,'command_id':f'bench-{index}','kind':'create_mission','target_id':None,'expected_version':None,'payload':{'goal':'pr_ready'}})
        else:request(url,token,'/v3/work?limit=50' if kind=='status' else '/v3/doctor')
        return kind,(time.perf_counter()-start)*1000,None
    except Exception as error:return kind,(time.perf_counter()-start)*1000,str(error)

def editor_probe(url,token):
    result=subprocess.run(['node',str(ROOT/'scripts/editor-benchmark.mjs')],input=json.dumps({'url':url,'token':token}),capture_output=True,text=True,timeout=30)
    if result.returncode:raise RuntimeError('editor benchmark failed: '+result.stderr)
    return json.loads(result.stdout)

def distribution(values):
    values=sorted(values)
    return {'count':len(values),'p50_ms':values[int((len(values)-1)*.5)],'p95_ms':values[int((len(values)-1)*.95)],'p99_ms':values[int((len(values)-1)*.99)],'max_ms':max(values),'samples_ms':values}

with tempfile.TemporaryDirectory(prefix='bf-benchmark-') as temporary:
    data=pathlib.Path(temporary)
    child,url,token,cold=start(data);stop(child)
    with sqlite3.connect(data/'hub.sqlite') as conn:
        conn.execute("INSERT INTO missions(id,owner_id,domain_id,goal,version,paused,contract_json) VALUES('bench-mission','owner-demo','core','pr_ready',1,0,'{}')")
        for i in range(10000):
            task=f'history-{i:05d}';raw=json.dumps({'id':task,'title':f'Historical task {i:05d}'}).encode()
            conn.execute("INSERT INTO contracts VALUES(?,1,'bench-mission','repo-demo',?,'sha256_exact_utf8_v3','fixture-base',?)",(task,hashlib.sha256(raw).hexdigest(),raw))
            conn.execute("INSERT INTO tasks(id,active_revision,version,phase,lineage_id) VALUES(?,1,1,'review_ready',?)",(task,task))
            conn.execute("INSERT INTO task_projection(task_id,owner_id,repo_id,mission_id,title,phase,why,version) VALUES(?,'owner-demo','repo-demo','bench-mission',?,'review_ready','Historical fixture projection',1)",(task,f'Historical task {i:05d}'))
        for i in range(4):conn.execute("INSERT INTO runners VALUES(?,'runner-fixture',?,1,1)",(f'simulated-{i}',f'incarnation-{i}'))
    (data/'endpoint.json').unlink()
    child,url,token,warm=start(data)
    try:
        rss=int(next(line.split()[1] for line in pathlib.Path(f'/proc/{child.pid}/status').read_text().splitlines() if line.startswith('VmRSS:')))/1024
        futures=[];start_time=time.perf_counter()
        with concurrent.futures.ThreadPoolExecutor(max_workers=16) as pool:
            editor_future=pool.submit(editor_probe,url,token)
            for tick in range(DURATION*20):
                delay=start_time+tick/20-time.perf_counter()
                if delay>0:time.sleep(delay)
                futures.append(pool.submit(sample,url,token,'status',tick))
                if tick%2==0:futures.append(pool.submit(sample,url,token,'command',tick))
                if tick%20==0:
                    for runner in range(4):futures.append(pool.submit(sample,url,token,'simulated_runner_status',tick*4+runner))
            results=[f.result() for f in futures]
            editor=editor_future.result()
        samples={kind:[ms for k,ms,error in results if k==kind] for kind in ['status','command','simulated_runner_status']}
        failures=[{'kind':k,'error':error} for k,ms,error in results if error]
        report={'workload':'bf-http-10000-v2-painted-editor','history_tasks':10000,'simulated_runners':4,'runner_behavior':'four independent model-free status clients, one poll/second; no live runner execution','status_per_second':20,'commands_per_second':10,'duration_seconds':DURATION,'binary_sha256':hashlib.sha256(BINARY.read_bytes()).hexdigest(),'workload_script_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'editor_script_sha256':hashlib.sha256((ROOT/'scripts/editor-benchmark.mjs').read_bytes()).hexdigest(),'cold_start_ms':cold,'warm_start_ms':warm,'idle_rss_mib':rss,'distributions':{k:distribution(v) for k,v in samples.items()},'failures':failures,'model_calls':0,'live_qualification':False,'editor':editor}
        report['passed']=editor['passed'] and not failures and report['distributions']['status']['p95_ms']<100 and report['distributions']['command']['p95_ms']<250 and rss<150
        output=ROOT/'evidence/performance.json';output.parent.mkdir(exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps({k:v for k,v in report.items() if k!='distributions'},indent=2))
        for k,v in report['distributions'].items():print(k,{key:value for key,value in v.items() if key!='samples_ms'})
        if not report['passed']:raise SystemExit(1)
    finally:stop(child)
