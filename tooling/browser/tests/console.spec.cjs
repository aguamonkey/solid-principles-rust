const {test,expect} = require('@playwright/test');
const payload = (identity) => ({identity,project:'Browser build',duration_ms:420,artifacts:['test.bin']});
async function fill(page,identity,project='Browser build'){
  await page.getByLabel('Build identity',{exact:true}).fill(identity);
  await page.getByLabel('Project',{exact:true}).fill(project);
  await page.getByLabel('Duration milliseconds').fill('420');
  await page.getByLabel('Artifact filenames').fill('test.bin');
}
test('explicit sample import and captured manifest use real API records',async({page})=>{
  await page.goto('/');
  await page.getByRole('button',{name:'Load three sample builds'}).click();
  await expect(page.getByRole('button',{name:'Inspect sample-atlas-042'})).toBeVisible();
  await page.getByRole('button',{name:'Inspect sample-atlas-042'}).click();
  await expect(page.locator('#manifest')).toContainText('manifest.json');
  const response=await page.request.get('/api/builds/sample-atlas-042');
  expect((await response.json()).candidate.duration_ms).toBe(420);
});
test('user-provided text stays text and build capture confirms before resetting',async({page})=>{
  await page.goto('/');await fill(page,'browser-capture','<img src=x onerror=alert(1)>');
  await page.getByRole('button',{name:'Record build',exact:false}).click();
  await expect(page.locator('#record-status')).toContainText('Build confirmed');
  await expect(page.locator('#builds')).toContainText('<img');
  await expect(page.locator('#builds img')).toHaveCount(0);
  await expect(page.getByLabel('Build identity',{exact:true})).toBeDisabled();
});
test('lost commit acknowledgement retries the original key and produces one record',async({page})=>{
  let calls=0,key,body;
  await page.route('**/api/builds',async route=>{
    if(route.request().method()!=='POST')return route.continue();
    calls++;
    if(calls===1){key=route.request().headers()['idempotency-key'];body=route.request().postData();await route.fetch();return route.abort('failed');}
    expect(route.request().headers()['idempotency-key']).toBe(key);expect(route.request().postData()).toBe(body);return route.continue();
  });
  await page.goto('/');await fill(page,'browser-lost-ack');
  await page.getByRole('button',{name:'Record build',exact:false}).click();
  await page.getByRole('button',{name:'Retry original submission'}).click();
  await expect(page.locator('#record-status')).toContainText('No duplicate record');
  const response=await page.request.get('/api/builds?limit=100');
  expect((await response.json()).builds.filter(row=>row.candidate.identity==='browser-lost-ack')).toHaveLength(1);
});
test('an unavailable refresh retains previous confirmed data',async({page})=>{
  await page.request.post('/api/builds',{data:payload('browser-refresh'),headers:{'Idempotency-Key':'browser-refresh'}});
  await page.goto('/');await expect(page.locator('#builds')).toContainText('browser-refresh');
  await page.route('**/api/builds?*',route=>route.fulfill({status:503,contentType:'application/json',body:'{"error":"Storage unavailable"}'}));
  await page.getByRole('button',{name:'Refresh ↻',exact:true}).click();
  await expect(page.locator('#read-status')).toContainText('remain visible');
  await expect(page.locator('#builds')).toContainText('browser-refresh');
});
test('failure simulator recovers one preview without touching the user journal',async({page})=>{
  await page.goto('/');const before=await(await page.request.get('/api/builds?limit=100')).json();
  await page.getByRole('button',{name:'Run first attempt + retry'}).click();
  await expect(page.locator('#lab-result')).toContainText('PreviewUnconfirmed');
  await expect(page.locator('#lab-result')).toContainText('reuses that preview');
  const after=await(await page.request.get('/api/builds?limit=100')).json();expect(after).toEqual(before);
});
test('formats come from composition and every study principle has a concrete seam',async({page})=>{
  await page.goto('/');await expect(page.locator('#format option')).toHaveCount(3);
  await page.getByRole('button',{name:'02 / OCP',exact:true}).click();
  await expect(page.locator('#lesson')).toContainText('--markdown');
  await page.getByRole('button',{name:'03 / LSP',exact:true}).click();
  await expect(page.locator('#lesson')).toContainText('live append-only journal');
  await page.getByRole('button',{name:'04 / ISP',exact:true}).click();
  await expect(page.locator('#lesson')).toContainText('BuildReader');
  await page.getByRole('button',{name:'05 / DIP',exact:true}).click();
  await expect(page.locator('#lesson')).toContainText('process_release');
});
test('invalid artifact paths never reach storage',async({page})=>{
  await page.goto('/');await fill(page,'browser-invalid');await page.getByLabel('Artifact filenames').fill('../secret');
  await page.getByRole('button',{name:'Record build',exact:false}).click();
  await expect(page.locator('#record-status')).toContainText('paths are not accepted');
  expect((await page.request.get('/api/builds/browser-invalid')).status()).toBe(404);
});
test('mobile layout fits and report export returns the stored records',async({page})=>{
  await page.setViewportSize({width:390,height:844});await page.goto('/');
  await expect(page.getByRole('heading',{name:'Register a build'})).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();
  const response=await page.request.get('/api/reports/json?limit=100');expect(response.status()).toBe(200);expect(Array.isArray(await response.json())).toBeTruthy();
});
