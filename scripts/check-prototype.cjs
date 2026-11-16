// Browser smoke checks for the local product prototype.
// Run with Node and Playwright. PLAYWRIGHT_MODULE may point to an installed module.
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const assert = require('node:assert/strict');
const { pathToFileURL } = require('node:url');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const url = pathToFileURL(path.join(root, 'docs/prototype.html')).href;

(async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 1440, height: 1050 } });
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    const go = async id => { await page.goto(url + '#' + id); await page.locator('h1').waitFor(); };
    const click = async (action, id) => {
      const locator = page.locator('[data-action="' + action + '"]' + (id ? '[data-id="' + id + '"]' : ''));
      await locator.first().click();
    };
    const expectText = async (selector, text) => assert((await page.locator(selector).innerText()).includes(text), selector + ' missing ' + text);
    await go('overview');
    const routes = await page.locator('#navigation a').evaluateAll(nodes => nodes.map(n => n.hash.slice(1)));
    assert.equal(routes.length, 18);
    for (const route of routes) {
      await go(route);
      assert((await page.locator('h1').textContent()).trim().length > 0);
      assert.equal(await page.locator('main h1').count(), 1);
      assert(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), route + ': desktop overflow');
    }
    console.log('PASS: 18 desktop screens, headings and page bounds');

    await go('requests');
    await click('request-filter', '流式中断');
    assert.equal(await page.locator('#request-results tbody tr').count(), 1);
    await click('request-filter', '全部');
    await page.locator('#request-search').fill('does-not-exist');
    await expectText('#request-results', '没有匹配');
    await page.locator('#request-search').fill('Anthropic');
    assert.equal(await page.locator('#request-results tbody tr').count(), 2);
    await page.locator('#request-search').fill('');
    await click('request', 'req-001');
    await click('request-tab', 'req-001|路由尝试');
    await expectText('#overlay', '2 次上游尝试');
    await click('request-tab', 'req-001|用量费用');
    await expectText('#overlay', '$ 0.0132');
    await expectText('#overlay', '$ 0.0012');
    await click('request-tab', 'req-001|内容证据');
    await click('request-access', 'req-001');
    await page.locator('[name="reason"]').fill('核查演示证据');
    await page.locator('button[type="submit"]').click();
    await go('audit');
    await expectText('main', '申请原文访问');
    await expectText('main', '当前原型角色没有原文读取授权');
    console.log('PASS: request filters, attempts, costs and separate raw-content access');

    await go('budgets');
    await page.locator('#budget-scope').selectOption('rd');
    await click('allocate', 'rd');
    await page.locator('[name="target"]').selectOption('lin');
    await page.locator('[name="amount"]').fill('500');
    await page.locator('[name="reason"]').fill('追加演示额度');
    await page.locator('button[type="submit"]').click();
    await expectText('.callout-stat', '$ 1,500.00');
    await expectText('.grid-2 .panel tbody tr:has-text("林知远")', '$ 5,700.00');
    await click('allocate', 'rd');
    await page.locator('[name="amount"]').fill('1500.01');
    await page.locator('[name="reason"]').fill('越界金额校验');
    await page.locator('button[type="submit"]').click();
    await expectText('#form-error', '不能超过');
    await click('close');
    await page.reload();
    await page.locator('#budget-scope').selectOption('rd');
    await expectText('.callout-stat', '$ 1,500.00');
    await page.locator('#budget-scope').selectOption('company');
    await expectText('.stats', '$ 30,000.00');
    await expectText('.stats', '$ 6,252.80');
    console.log('PASS: delegated allocation, overspend rejection, unchanged parent totals, session persistence');

    await go('policies');
    await click('policy-detail', 'p1');
    await click('edit-policy', 'p1');
    await page.locator('[name="action"]').selectOption('告警');
    await page.locator('button[type="submit"]').click();
    const oldPolicyRow = page.locator('main tbody tr').filter({ hasText: '敏感凭证保护' });
    assert((await oldPolicyRow.innerText()).includes('正式执行'));
    assert((await oldPolicyRow.innerText()).includes('拦截'));
    assert((await oldPolicyRow.innerText()).includes('未发布草稿'));
    await click('policy-detail', 'p1');
    await click('publish-policy', 'p1');
    await click('confirm-policy', 'p1');
    assert((await oldPolicyRow.innerText()).includes('告警'));
    assert((await oldPolicyRow.innerText()).includes('v4'));
    console.log('PASS: active policy preserved during draft edit; explicit demo publish');

    await go('risks');
    await click('risk-detail', 'risk-001');
    await click('review', 'risk-001');
    await page.locator('[name="verdict"]').selectOption('误报');
    await page.locator('[name="note"]').fill('这是示例数据，记录一次误报复核。');
    await page.locator('button[type="submit"]').click();
    await expectText('#overlay', '已复核');
    await expectText('#overlay', '已拦截');
    await expectText('#overlay', '误报');
    await click('close');
    console.log('PASS: review appended without rewriting original action');

    await go('routes');
    await click('route-fail');
    await expectText('main', '已选为备用尝试');
    await click('route-config');
    await page.locator('[name="retry"]').selectOption('0');
    await page.locator('button[type="submit"]').click();
    await click('route-fail');
    await expectText('main', '结束调用，不切换备用渠道');
    assert.equal(await page.getByText('已选为备用尝试', { exact: true }).count(), 0);
    console.log('PASS: failover respects configured extra-attempt limit');

    await go('apps');
    await click('new-app');
    await page.locator('[name="name"]').fill('原型验证应用');
    await page.locator('[name="dept"]').selectOption('rd');
    await page.locator('[name="protocol"]').selectOption('Anthropic Messages');
    await page.locator('button[type="submit"]').click();
    await expectText('main', '原型验证应用');
    await expectText('main', 'NOT_VALID');
    await go('org');
    await click('new-dept');
    await page.locator('[name="name"]').fill('设计部');
    await page.locator('[name="owner"]').fill('示例负责人');
    await page.locator('button[type="submit"]').click();
    await expectText('main', '设计部');
    console.log('PASS: sample application and department creation');

    await go('operations');
    await click('node', 'gateway-01');
    await expectText('main tbody tr:has-text("gateway-01")', '排空中');
    await expectText('main tbody tr:has-text("gateway-01")', '89');
    await click('node', 'gateway-01');
    await expectText('main tbody tr:has-text("gateway-01")', '已下线');
    console.log('PASS: graceful drain retains live connections until explicitly completed');

    // Reset before creating clean deliverable previews.
    await click('reset');
    await click('confirm-reset');
    await page.locator('#toast.visible').waitFor({ state: 'hidden' });
    await go('overview');
    await page.screenshot({ path: path.join(root, 'docs/prototype-overview.png'), fullPage: true, animations: 'disabled' });
    await go('map');
    await page.screenshot({ path: path.join(root, 'docs/prototype-map.png'), fullPage: true, animations: 'disabled' });
    await go('budgets');
    await page.locator('#budget-scope').selectOption('rd');
    await page.screenshot({ path: path.join(root, 'docs/prototype-budget.png'), fullPage: true, animations: 'disabled' });
    await go('requests');
    await click('request', 'req-001');
    await click('request-tab', 'req-001|路由尝试');
    await page.screenshot({ path: path.join(root, 'docs/prototype-request.png'), animations: 'disabled' });
    await page.keyboard.press('Escape');
    assert.equal(await page.locator('dialog[open]').count(), 0);

    const phone = await browser.newPage({ viewport: { width: 390, height: 844 }, isMobile: true, deviceScaleFactor: 1 });
    phone.on('pageerror', e => errors.push(e.message));
    for (const route of routes) {
      await phone.goto(url + '#' + route);
      await phone.locator('h1').waitFor();
      assert(await phone.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), route + ': mobile overflow');
    }
    await phone.goto(url + '#overview');
    await phone.locator('[data-action="menu"]').click();
    assert((await phone.locator('#sidebar').getAttribute('class')).includes('open'));
    await phone.locator('#navigation a[href="#budgets"]').click();
    await phone.locator('h1').filter({ hasText: '额度分配' }).waitFor();
    await phone.screenshot({ path: '/tmp/zenrouter-prototype-mobile.png', fullPage: true, animations: 'disabled' });
    assert.deepEqual(errors, []);
    console.log('PASS: 18 mobile screens, navigation, Escape dismissal, zero browser exceptions');
    console.log('Saved: overview, module map, budget and request-detail PNG previews');
  } finally { await browser.close(); }
})().catch(e => { console.error(e); process.exitCode = 1; });
