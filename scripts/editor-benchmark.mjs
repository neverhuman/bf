import { chromium } from '../web/node_modules/playwright/index.mjs';
let input='';for await (const chunk of process.stdin) input+=chunk;
const {url,token}=JSON.parse(input);
const browser=await chromium.launch({headless:true});
try {
  const page=await browser.newPage({viewport:{width:1000,height:800}});
  const navigationStart=performance.now();
  await page.goto(url+'/#token='+token);
  await page.getByText('Connected',{exact:true}).waitFor();
  await page.getByRole('heading',{name:'Historical task 09999',exact:true}).waitFor();
  // A real user sees the composer only after paint. Headless Chromium can accept
  // synthetic input before its first frame; record that startup separately.
  await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  const workbenchReadyMs=performance.now()-navigationStart;
  const editor=page.getByLabel('Goal',{exact:true});await editor.fill('');
  await page.evaluate(()=>{window.bfTiming=[];document.getElementById('goal').addEventListener('input',()=>{const start=performance.now();requestAnimationFrame(()=>window.bfTiming.push(performance.now()-start));},{capture:true});});
  const text='Unicode 猫, multiline input and a long goal.\n'.repeat(4);
  await editor.pressSequentially(text,{delay:3});
  await page.waitForTimeout(100);
  const values=await page.evaluate(()=>window.bfTiming);
  values.sort((a,b)=>a-b);
  if(values.length<100)throw new Error('missing input events');
  const report={method:'input event to next animation frame after initial workbench paint, release hub with 10000 historical tasks and concurrent HTTP workload',workbench_ready_ms:workbenchReadyMs,count:values.length,p95_ms:values[Math.floor((values.length-1)*.95)],max_ms:Math.max(...values),samples_ms:values};
  report.passed=report.p95_ms<100;
  console.log(JSON.stringify(report));
} finally {await browser.close();}
