const el = id => document.getElementById(id);
const game = el("game");
const ctx = game.getContext("2d");
const cols = 150;
const platforms = {
  win: ["termrex.exe", ".\\termrex.exe"],
  linux: ["termrex-linux-x64", "chmod +x termrex-linux-x64 && ./termrex-linux-x64"],
  mac: ["termrex-macos-arm64", "chmod +x termrex-macos-arm64 && ./termrex-macos-arm64"],
  intel: ["termrex-macos-x64", "chmod +x termrex-macos-x64 && ./termrex-macos-x64"]
};
function platformChanged() {
  const [file, command] = platforms[el("platform").value];
  el("command").textContent = command;
  el("download").href = "downloads/" + file;
  el("download").download = file;
  const url = "https://seiya058904.github.io/TermRex/downloads/" + file;
  el("install-command").textContent = el("platform").value === "win"
    ? 'Invoke-WebRequest -ErrorAction Stop -Uri "' + url + '" -OutFile termrex.exe; .\\termrex.exe'
    : 'curl -fLO "' + url + '" && ' + command;
}
if (/Linux/.test(navigator.platform)) el("platform").value = "linux";
if (/Mac/.test(navigator.platform)) el("platform").value = "mac";
platformChanged();
el("platform").onchange = platformChanged;
for (const button of document.querySelectorAll("[data-copy]")) {
  const original = button.innerHTML;
  button.onclick = async () => {
    const code = el(button.dataset.copy);
    try {
      await navigator.clipboard.writeText(code.textContent);
      button.textContent = "Copied";
      setTimeout(() => button.innerHTML = original, 1200);
    } catch {
      const range = new Range();
      range.selectNodeContents(code);
      getSelection().removeAllRanges();
      getSelection().addRange(range);
    }
  };
}

async function start() {
  const response = await fetch("termrex.wasm");
  if (!response.ok) throw new Error("Game download failed");
  const {instance} = await WebAssembly.instantiate(await response.arrayBuffer());
  const engine = instance.exports;
  let auto = true;
  let best = 0;
  try { best = Math.max(0, Number(localStorage.getItem("termrex-best")) || 0); } catch {}
  const held = new Map();
  let lastState = "";
  let lastTime = 0, elapsed = 0;
  const tick = 1000 / 60;
  const color = n => "#" + n.toString(16).padStart(6, "0");
  const bits = [1,8,2,16,4,32,64,128];
  const pad = n => String(n).padStart(5,"0");
  function reset(demo) {
    held.clear(); auto = demo;
    engine.init(demo ? 4577 : crypto.getRandomValues(new Uint32Array(1))[0], demo ? 0 : best, cols);
    engine.input(3,1);
    if (demo) for (let i=0;i<242;i++) engine.step(1);
    lastTime = 0; elapsed = 0;
    el("display").classList.toggle("manual-mode", !demo);
    document.querySelector(".touch").hidden = demo;
    sync();
  }
  function sync() {
    const state = engine.stat(2);
    const status = state===2 ? "Paused" : state===3 ? "Game over" : auto ? "Autopilot" : "Playing";
    if (lastState !== status) el("status").textContent = lastState = status;
    el("play").textContent = auto ? "▶ Play demo" : state===3 ? "↻ Try again" : "↺ Autopilot";
    el("motion").textContent = state===2 ? "▶" : "Ⅱ";
    el("motion").disabled = state===3;
    const label = state===2 ? "Resume demo" : "Pause demo";
    el("motion").setAttribute("aria-label", label);
    el("motion").title = label;
    if (!auto && engine.stat(1)>best) {
      best=engine.stat(1);
      try { localStorage.setItem("termrex-best",String(best)); } catch {}
    }
  }
  function paint() {
    const ptr = engine.draw(), rows=engine.stat(4);
    const cells=new Uint8Array(engine.memory.buffer,ptr,cols*rows);
    ctx.fillStyle=color(engine.stat(6));
    ctx.fillRect(0,0,game.width,game.height);
    ctx.fillStyle=color(engine.stat(5));
    const hudSize = Math.min(34, Math.max(19, Math.round(12 * game.width / game.clientWidth)));
    ctx.font="bold "+hudSize+"px Consolas, monospace";ctx.textAlign="right";
    const flash=engine.stat(3);
    const score=flash && Math.floor((72-flash)/12)%2===0 ? "     " : pad(engine.stat(0));
    ctx.fillText("HI "+pad(engine.stat(1))+"  "+score,game.width-20,hudSize);
    for(let i=0;i<cells.length;i++){
      const val=cells[i]; if(!val) continue;
      const x=(i%cols)*8,y=Math.floor(i/cols)*16+31;
      for(let j=0;j<8;j++) if(val&bits[j]) ctx.fillRect(x+j%2*4,y+Math.floor(j/2)*4,3,3);
    }
  }
  function animate(time) {
    elapsed+=lastTime ? Math.min(100,time-lastTime) : 0; lastTime=time;
    while(elapsed>=tick) {engine.step(auto?1:0);elapsed-=tick;}
    if(auto && engine.stat(2)===3) reset(true);
    sync();paint();requestAnimationFrame(animate);
  }
  function key(id,k,down) {
    if(down) {
      if(held.has(id)) return;
      const already=[...held.values()].includes(k);
      held.set(id,k); if(!already) engine.input(k,1);
    } else {
      held.delete(id); if(![...held.values()].includes(k)) engine.input(k,0);
    }
  }
  const keys={Space:0,ArrowUp:0,KeyW:0,ArrowDown:1,KeyS:1,KeyP:2,KeyR:3};
  el("display").onkeydown = e => {
    if(!(e.code in keys)) return;
    e.preventDefault();
    if(auto && keys[e.code]<2) reset(false);
    key(e.code,keys[e.code],true);
  };
  el("display").onkeyup = e => { if(e.code in keys){e.preventDefault();key(e.code,keys[e.code],false);} };
  function release(){for(const [id,k] of [...held]) key(id,k,false);}
  el("display").onblur=release;
  document.addEventListener("visibilitychange",()=>{
    release();lastTime=0;elapsed=0;
    if(document.hidden && !auto && engine.stat(2)===1) engine.input(2,1);
  });
  el("play").onclick=()=>{reset(!(auto||engine.stat(2)===3));el("display").focus({preventScroll:true});};
  el("motion").onclick=()=>{engine.input(2,1);sync();el("display").focus({preventScroll:true});};
  document.querySelectorAll("[data-key]").forEach(b=>{
    b.onpointerdown=e=>{b.setPointerCapture(e.pointerId);key(e.pointerId,+b.dataset.key,true);};
    b.onlostpointercapture=e=>key(e.pointerId,+b.dataset.key,false);
  });
  reset(true);
  if(matchMedia("(prefers-reduced-motion: reduce)").matches) engine.input(2,1);
  paint();requestAnimationFrame(animate);
}
start().catch(e=>{
  console.error(e);
  el("status").textContent="Demo unavailable · download to play";
  el("play").disabled=el("motion").disabled=true;
});
