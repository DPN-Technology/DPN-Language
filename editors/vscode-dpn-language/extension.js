'use strict';
const vscode = require('vscode');
const cp = require('child_process');
const path = require('path');

const builtins = [
  ['print(value)', 'Write a value to the DPN output stream.'],
  ['type.of(value)', 'Return the DPN type name.'],
  ['array.len(items)', 'Return array size.'],
  ['array.push(items, value)', 'Return a new array with the element appended.'],
  ['array.contains(items, value)', 'Return whether an array contains a value.'],
  ['math.abs(value)', 'Absolute value.'], ['math.min(a, b)', 'Minimum of two numbers.'],
  ['math.max(a, b)', 'Maximum of two numbers.'], ['math.floor(value)', 'Round downward.'],
  ['math.ceil(value)', 'Round upward.'], ['string.upper(value)', 'Uppercase text.'],
  ['string.lower(value)', 'Lowercase text.'], ['string.trim(value)', 'Trim whitespace.'],
  ['string.len(value)', 'Text length.'], ['string.replace(text, old, new)', 'Replace text.'],
  ['string.contains(text, needle)', 'Text containment.'],
  ['string.starts_with(text, prefix)', 'Check prefix.'],
  ['string.ends_with(text, suffix)', 'Check suffix.'],
  ['number.parse(text)', 'Parse a number.'], ['system.os()', 'Read the operating system.'],
  ['system.arch()', 'Read the CPU architecture.'],
  ['system.cpu_count()', 'Read the number of logical CPUs.'],
  ['system.online()', 'Check whether the runtime reports online.'],
  ['file.read(path)', 'Read a file if --allow-read was granted.']
];
const keywords = ['module','use','fn','let','const','return','if','else','while','true','false','nil','number','string','bool','array','any'];
const names = text => {
  const results=[];
  const rx=/\bfn\s+([A-Za-z_]\w*)\s*\(([^)]*)\)|\b(?:let|const)\s+([A-Za-z_]\w*)/g;
  let m;
  while ((m=rx.exec(text))) results.push({name:m[1]||m[3],params:m[2]||'',start:m.index,kind:m[1]?'function':'variable'});
  return results;
};
function guessLine(document, message) {
  const explicit=message.match(/\bline\s+(\d+)\b/i);
  if(explicit) return Math.max(0, Number(explicit[1])-1);
  const quoted=message.match(/(?:variable|function|const|parameter)\s+'([^']+)'/);
  if(quoted) {
    const lines=document.getText().split(/\r?\n/);
    const i=lines.findIndex(s=>new RegExp('\\b'+quoted[1].replace(/[.*+?^${}()|[\]\\]/g,'\\$&')+'\\b').test(s));
    if(i>=0) return i;
  }
  return 0; // Static analyzer does not yet provide source spans.
}
function runCheck(exe, document, useInput, timeoutMs=15000) {
  return new Promise(resolve=>{
    const args=['check',document.uri.fsPath,...(useInput?['--stdin']:[])];
    const child=cp.spawn(exe,args,{shell:false,windowsHide:true,stdio:['pipe','pipe','pipe']});
    let stderr='',stdout='',done=false;
    const finish=(result)=>{if(done)return;done=true;clearTimeout(timeout);resolve(result);};
    const timeout=setTimeout(()=>{child.kill();finish({ok:false,error:'DPN checker timed out.'});},timeoutMs);
    child.on('error', e=>finish({ok:false,error:`Unable to start DPN CLI: ${e.message}`}));
    child.stdout.setEncoding('utf8'); child.stderr.setEncoding('utf8');
    child.stdout.on('data',d=>{stdout+=d;if(stdout.length>65536)child.kill();});
    child.stderr.on('data',d=>{stderr+=d;if(stderr.length>65536)child.kill();});
    child.on('close',code=>finish(code===0?{ok:true}:{ok:false,error:(stderr||stdout||`dpn exited with status ${code}`).replace(/^DPN error:\s*/, '').trim()}));
    if(useInput) child.stdin.end(document.getText()); else child.stdin.end();
  });
}
function activate(context) {
  const diagnostics=vscode.languages.createDiagnosticCollection('dpn');
  context.subscriptions.push(diagnostics);
  const debounce=new Map();
  const versions=new Map();
  const isDpn=d=>d.languageId==='dpn'&&d.uri.scheme==='file';
  async function validate(doc) {
    if(!isDpn(doc)) return;
    const version=doc.version;
    const exe=vscode.workspace.getConfiguration('dpn').get('executable','dpn');
    const result=await runCheck(exe,doc,true);
    if(doc.isClosed||doc.version!==version)return;
    if(result.ok){diagnostics.delete(doc.uri);return;}
    const line=guessLine(doc,result.error);
    const maxLine=Math.max(0,doc.lineCount-1);
    const textLine=doc.lineAt(Math.min(maxLine,line));
    const range=new vscode.Range(textLine.range.start,textLine.range.end);
    const issue=new vscode.Diagnostic(range,result.error,vscode.DiagnosticSeverity.Error);
    issue.source='dpn check';
    diagnostics.set(doc.uri,[issue]);
  }
  function schedule(doc) {
    if(!isDpn(doc))return;
    clearTimeout(debounce.get(doc.uri.toString()));
    const cfg=vscode.workspace.getConfiguration('dpn');
    const delay=cfg.get('diagnosticDelay',400);
    debounce.set(doc.uri.toString(),setTimeout(()=>validate(doc),delay));
  }
  context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(schedule));
  context.subscriptions.push(vscode.workspace.onDidSaveTextDocument(validate));
  context.subscriptions.push(vscode.workspace.onDidChangeTextDocument(e=>{
    if(vscode.workspace.getConfiguration('dpn').get('validateOnChange',true))schedule(e.document);
  }));
  context.subscriptions.push(vscode.workspace.onDidCloseTextDocument(d=>{
    clearTimeout(debounce.get(d.uri.toString())); debounce.delete(d.uri.toString()); diagnostics.delete(d.uri);
  }));
  context.subscriptions.push({dispose:()=>{for(const timer of debounce.values())clearTimeout(timer);}});
  for(const doc of vscode.workspace.textDocuments) schedule(doc);

  function terminalCommand(command){return async()=>{
    const editor=vscode.window.activeTextEditor;
    if(!editor||!isDpn(editor.document)){vscode.window.showWarningMessage('Open a .dpn file first.');return;}
    await editor.document.save();
    const exe=vscode.workspace.getConfiguration('dpn').get('executable','dpn');
    const terminal=vscode.window.createTerminal({name:`DPN ${command}`});terminal.show();
    const quote=s=>'"'+String(s).replace(/"/g,'\\"')+'"';
    terminal.sendText(`${quote(exe)} ${command} ${quote(editor.document.uri.fsPath)}`);
  };}
  context.subscriptions.push(vscode.commands.registerCommand('dpn.runCurrent',terminalCommand('run')));
  context.subscriptions.push(vscode.commands.registerCommand('dpn.checkCurrent',terminalCommand('check')));

  context.subscriptions.push(vscode.languages.registerCompletionItemProvider('dpn',{
    provideCompletionItems(doc) {
      const items=[];
      for(const name of keywords){const item=new vscode.CompletionItem(name,vscode.CompletionItemKind.Keyword);items.push(item);}
      for(const [signature,help] of builtins){
        const name=signature.split('(')[0];
        const item=new vscode.CompletionItem(name,vscode.CompletionItemKind.Function);
        item.detail=signature;item.documentation=new vscode.MarkdownString(help);items.push(item);
      }
      for(const item of names(doc.getText())) {
        const entry=new vscode.CompletionItem(item.name,item.kind==='function'?vscode.CompletionItemKind.Function:vscode.CompletionItemKind.Variable);
        if(item.kind==='function')entry.detail=`fn ${item.name}(${item.params})`;
        items.push(entry);
      }
      return items;
    }
  },'.'));
  context.subscriptions.push(vscode.languages.registerDocumentSymbolProvider('dpn',{
    provideDocumentSymbols(doc) {
      return names(doc.getText()).map(item=>{
        const pos=doc.positionAt(item.start);
        const range=new vscode.Range(pos,pos);
        return new vscode.DocumentSymbol(item.name,item.kind==='function'?`(${item.params})`:'',item.kind==='function'?vscode.SymbolKind.Function:vscode.SymbolKind.Variable,range,range);
      });
    }
  }));
  context.subscriptions.push(vscode.languages.registerDefinitionProvider('dpn',{
    provideDefinition(doc,pos){
      const word=doc.getText(doc.getWordRangeAtPosition(pos));
      const match=names(doc.getText()).find(item=>item.name===word);
      return match?new vscode.Location(doc.uri,doc.positionAt(match.start)):null;
    }
  }));
  context.subscriptions.push(vscode.languages.registerHoverProvider('dpn',{
    provideHover(doc,pos){
      const line=doc.lineAt(pos.line).text;
      const before=line.slice(0,pos.character);
      const after=line.slice(pos.character);
      const qualified=(before.match(/[\w.]*$/)||[''])[0]+(after.match(/^[\w.]*/)||[''])[0];
      const builtin=builtins.find(([sig])=>sig.startsWith(qualified+'('));
      if(builtin)return new vscode.Hover(`**${builtin[0]}**\n\n${builtin[1]}`);
      return null;
    }
  }));
}
function deactivate(){}
module.exports={activate,deactivate,guessLine,names,runCheck};
