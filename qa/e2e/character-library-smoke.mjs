import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium', headless: true, args: ['--no-sandbox'] });
const base = process.env.WONDERWILDS_TEST_URL || 'http://127.0.0.1:5174/';
try {
  for (const [name,width,height] of [['desktop',1280,900],['tablet',768,1024],['phone',390,844]]) {
    const page = await browser.newPage({ viewport:{width,height} });
    const errors=[];
    page.on('pageerror',error=>errors.push(error.message));
    const response=await page.goto(new URL('/characters.html',base).href);
    assert.equal(response.status(),200);
    await page.waitForSelector('.friend-card');
    assert.equal(await page.locator('.friend-card').count(),12);
    const art=await page.locator('.friend-portrait').evaluateAll(async elements => {
      return Promise.all(elements.map(async element => {
        const src=getComputedStyle(element).backgroundImage.slice(5,-2);
        const image=new Image(); image.src=src;
        await image.decode();
        return image.naturalWidth > 0 && image.naturalHeight > 0;
      }));
    });
    assert.ok(art.every(Boolean));
    for(const region of ['Clover Meadow','Peachy Orchard','Starlight Cove']) {
      await page.getByRole('button',{name:region,exact:true}).click();
      assert.equal(await page.locator('.friend-card').count(),4);
    }
    await page.getByRole('button',{name:'All friends',exact:true}).focus();
    await page.keyboard.press('Enter');
    assert.equal(await page.locator('.friend-card').count(),12);
    await page.getByText('Get to know Pip',{exact:true}).focus();
    await page.keyboard.press('Enter');
    assert.equal(await page.locator('details[open]').count(),1);
    assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth <= innerWidth));
    assert.deepEqual(errors,[]);
    await page.screenshot({path:`/tmp/wonderwilds-characters-${name}.png`,fullPage:true});
    await page.close();
    console.log(`${name}: twelve artworks, regional filters, keyboard details and responsive layout passed`);
  }
} finally {await browser.close();}
