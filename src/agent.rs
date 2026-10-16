use crate::{model::*,process,store::Store};
use anyhow::{Context,Result,bail};
use serde_json::{Value,json};
use std::{io::{BufRead,BufReader,IsTerminal,Write},process::{Child,ChildStdin,Command,Stdio},sync::mpsc::{self,Receiver},thread,time::{Duration,Instant}};

pub fn instructions(tool:Tool,required:bool,exe:&str)->String{
    format!(r#"You are the single cargo-elicit specification worker. Work in the original source tree.
Verifier: {tool}; verification required: {required}. Use the user's Codex model, reasoning, auth and permissions.
Write meaningful contracts, type invariants, trait laws and preservation obligations. Do not weaken a contract merely to obtain a proof. If a specification is wrong, report the reason with agent issue before revising it.
Never change runtime behavior, move APIs out of the denominator, copy a function to another project for proof, or hide assumptions as trusted infrastructure. Existing uncommitted work must be preserved. Never git commit/reset/clean. Do not start parallel/subagents.
Creusot uses creusot_std contracts, invariants, logic/ghost/lemmas. Trusted is acceptable only for specification mode, never evidence of a proof. Kani uses cfg_attr(kani,kani::requires/ensures), arbitrary input proof_for_contract harnesses calling the original function. Put models and law/preservation harnesses in the original sources; report unsupported invariants/laws instead of pretending coverage.
Use the ledger CLI: {exe} agent show [ID]; {exe} agent submit ID --revision N --note REASON [--depends ID ...] [--unit UNIT ...]; {exe} agent depend ID DEPENDENCY --revision N; {exe} agent issue ID --revision N --reason REASON. Fetch the current revision with agent show after source edits. Submit runs syntax/type processing through the verifier and rejects changing inputs. Do not edit state.toml directly.
You may run verifier experiments, but only cargo-elicit's official wrapper records proof evidence. agent verify is available outside active turns; during an active turn finish and let the scheduler run the wrapper while edits are stopped.
Do not claim verified yourself. Preserve partial results and limitations. For a blocked task leave a concise handoff note/issue. Submit every target separately with its current revision; a completed turn without accepted work is not success.
"#)
}
pub struct Codex{child:Child,input:ChildStdin,rx:Receiver<Value>,seq:u64,pub thread:String,turn:Option<String>,pub usage:(u64,u64,u64)}
impl Drop for Codex{fn drop(&mut self){if let Some(t)=self.turn.clone(){let _=self.send(json!({"id":999999,"method":"turn/interrupt","params":{"threadId":self.thread,"turnId":t}}));}process::kill_group(self.child.id());let _=self.child.kill();let _=self.child.wait();}}
impl Codex{
    pub fn connect(store:&Store,s:&State,required:bool)->Result<Self>{
        let mut cmd=Command::new("codex");cmd.arg("app-server").current_dir(&store.root);process::group(&mut cmd);
        let mut child=cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().context("cannot start codex app-server; install and authenticate Codex first")?;
        let input=child.stdin.take().unwrap();let out=child.stdout.take().unwrap();let(tx,rx)=mpsc::channel();thread::spawn(move||{for line in BufReader::new(out).lines(){let Ok(line)=line else{break;};if line.len()>16*1024*1024{break;}if let Ok(v)=serde_json::from_str(&line){if tx.send(v).is_err(){break;}}}});
        let mut c=Self{child,input,rx,seq:0,thread:String::new(),turn:None,usage:(0,0,0)};
        c.request("initialize",json!({"clientInfo":{"name":"cargo-elicit","title":"cargo-elicit","version":env!("CARGO_PKG_VERSION")},"capabilities":{}}),Duration::from_secs(30))?;
        c.send(json!({"method":"initialized","params":{}}))?;
        let exe=std::env::current_exe()?.to_string_lossy().into_owned();
        let params=json!({"cwd":store.root,"developerInstructions":instructions(s.tool,required,&exe)});
        let result=if let Some(id)=&s.run.thread{
            let mut p=params.clone();p["threadId"]=json!(id);
            c.request("thread/resume",p,Duration::from_secs(60)).or_else(|_|c.request("thread/start",params.clone(),Duration::from_secs(60)))?
        }else{c.request("thread/start",params,Duration::from_secs(60))?};
        c.thread=result["thread"]["id"].as_str().context("app-server thread ID missing")?.into();Ok(c)
    }
    fn send(&mut self,v:Value)->Result<()>{serde_json::to_writer(&mut self.input,&v)?;self.input.write_all(b"\n")?;self.input.flush()?;Ok(())}
    fn next(&mut self,deadline:Instant)->Result<Value>{
        loop{
            if process::interrupted() || Instant::now()>=deadline{if let Some(id)=self.turn.clone(){self.seq+=1;let _=self.send(json!({"id":self.seq,"method":"turn/interrupt","params":{"threadId":self.thread,"turnId":id}}));}bail!(if process::interrupted(){"interrupted"}else{"Codex task deadline exceeded"});}
            match self.rx.recv_timeout(Duration::from_millis(100)){
                Ok(v)=>{if v.get("id").is_some() && v.get("method").is_some(){self.server_request(&v,deadline)?;continue;}
                    if v["method"]=="thread/tokenUsage/updated"{let u=&v["params"]["tokenUsage"]["total"];self.usage=(u["inputTokens"].as_u64().unwrap_or(0),u["cachedInputTokens"].as_u64().unwrap_or(0),u["outputTokens"].as_u64().unwrap_or(0));}return Ok(v);},
                Err(mpsc::RecvTimeoutError::Timeout)=>{},Err(_)=>bail!("Codex app-server disconnected")
            }
        }
    }
    fn request(&mut self,method:&str,params:Value,timeout:Duration)->Result<Value>{self.seq+=1;let id=self.seq;self.send(json!({"id":id,"method":method,"params":params}))?;let deadline=Instant::now()+timeout;loop{let v=self.next(deadline)?;if v["id"]==id{if let Some(e)=v.get("error"){bail!("app-server {method}: {e}");}return Ok(v["result"].clone());}}}
    fn server_request(&mut self,v:&Value,deadline:Instant)->Result<()>{
        let method=v["method"].as_str().unwrap_or("");
        if !matches!(method,"item/commandExecution/requestApproval"|"item/fileChange/requestApproval"){
            self.send(json!({"id":v["id"],"error":{"code":-32601,"message":"cargo-elicit does not support this interactive request"}}))?;bail!("unsupported app-server request {method}; no permission was granted");
        }
        if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal(){self.send(json!({"id":v["id"],"result":{"decision":"decline"}}))?;bail!("approval required in unattended execution: {method}; resume in an interactive terminal");}
        eprintln!("Codex approval request ({method}):\n{}\nAllow this request? [y/N]",serde_json::to_string_pretty(&v["params"])?);
        let (tx,rx)=mpsc::channel();thread::spawn(move||{let mut line=String::new();let _=std::io::stdin().read_line(&mut line);let _=tx.send(line);});
        let answer=loop{if process::interrupted()||Instant::now()>=deadline{bail!("approval interrupted or deadline exceeded");}if let Ok(l)=rx.recv_timeout(Duration::from_millis(100)){break l;}};
        let decision=if answer.trim().eq_ignore_ascii_case("y"){"accept"}else{"decline"};self.send(json!({"id":v["id"],"result":{"decision":decision}}))?;if decision=="decline"{bail!("approval declined");}Ok(())
    }
    pub fn run(&mut self,s:&State,task:&Task,timeout:Duration)->Result<String>{
        let _ = s; // Only task context changes between turns; shared instructions belong to the thread.
        let details:Vec<_>=task.targets.iter().filter_map(|id|s.targets.get(id)).map(|t|json!({"id":t.id,"file":t.file,"line":t.line,"kind":t.kind,"dependencies":t.dependencies,"stale":t.stale,"notes":t.notes,"required_units":t.required_units})).collect();
        let text=serde_json::to_string(&json!({"task":task.kind,"targets":details,"input_revision":s.revision,"input_fingerprint":s.input,"completion":"accepted specification submission; for verification prepare original-source proof and finish turn for official wrapper"}))?;
        let start=Instant::now();let response=self.request("turn/start",json!({"threadId":self.thread,"input":[{"type":"text","text":text}]}),timeout)?;
        self.turn=response["turn"]["id"].as_str().map(str::to_string);
        if self.turn.is_none(){bail!("app-server turn ID missing");}
        loop{let v=self.next(start+timeout)?;if v["method"]=="turn/completed" && v["params"]["threadId"]==self.thread && v["params"]["turn"]["id"].as_str()==self.turn.as_deref(){self.turn=None;let status=v["params"]["turn"]["status"].as_str().unwrap_or("unknown");if status!="completed"{bail!("Codex turn ended with status {status}");}return Ok(status.into());}}
    }
}
