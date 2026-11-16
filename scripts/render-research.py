"""Render the project's research Markdown subset to standalone HTML."""
from pathlib import Path
import html
import re

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / 'docs'
CSS = (DOCS / 'technical-research.html').read_text().split('<style>', 1)[1].split('</style>', 1)[0]
ARCH_CSS = """
.architecture-figure{margin:28px 0;border:1px solid var(--line,#d7dfd4);border-radius:16px;overflow:hidden;background:#fcfdf9}
.diagram-tools{display:flex;gap:8px;align-items:center;flex-wrap:wrap;padding:16px 18px;border-bottom:1px solid #dbe5dc}
.diagram-tools strong{font-size:13px;margin-right:8px}.diagram-tools button{padding:7px 12px;font-size:13px;border:1px solid #cddbd3;border-radius:20px;background:white;color:#24634c;cursor:pointer}
.diagram-tools button[aria-pressed="true"]{background:#24634c;color:#fff;border-color:#24634c}
.diagram-tools a{font-size:13px;margin-left:auto}.diagram-scroll{overflow:auto}.diagram-scroll svg{display:block;width:100%;height:auto;min-width:960px}
.architecture-figure figcaption{padding:12px 18px;color:#526b63;font-size:13px;border-top:1px solid #dbe5dc}
.deployment-figure{margin:28px 0;border:1px solid var(--line,#d7dfd4);border-radius:16px;overflow:hidden;background:#fcfdf9}
.deployment-figure figcaption{padding:12px 18px;font-size:13px;color:#526b63;border-top:1px solid #dbe5dc}
.deployment-scroll{overflow:auto}.deployment-scroll img{display:block;width:100%;height:auto;min-width:960px}
@media print{.diagram-tools{display:none}.diagram-scroll{overflow:visible}.diagram-scroll svg{min-width:0;max-width:100%}.architecture-figure{break-inside:avoid}.diagram-scroll .dimmed{opacity:1}}
@media print{.deployment-figure{break-inside:avoid}.deployment-scroll{overflow:visible}.deployment-scroll img{min-width:0;max-width:100%}}
"""
ARCH_SCRIPT = """
document.querySelectorAll('[data-diagram-focus]').forEach(function(button){
  button.addEventListener('click',function(){
    var mode=button.dataset.diagramFocus;
    document.querySelectorAll('[data-diagram-focus]').forEach(function(b){b.setAttribute('aria-pressed',String(b===button))});
    document.querySelectorAll('.architecture-figure .layer').forEach(function(layer){
      layer.classList.toggle('dimmed',mode!=='all'&&!layer.dataset.layer.split(' ').includes(mode));
    });
    document.getElementById('diagram-status').textContent=mode==='all'?'当前展示全部关系；可横向滚动或单独打开架构图。':'当前突出'+button.textContent+'，其余关系淡化；打印时自动展示全图。';
  });
});
"""

def inline(text):
    text = html.escape(text)
    text = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', lambda m: '<a href="' + m[2] + '">' + m[1] + '</a>', text)
    return re.sub(r'\*\*([^*]+)\*\*', r'<strong>\1</strong>', text)

def body(md):
    lines, blocks, toc, i = md.splitlines(), [], [], 0
    while i < len(lines):
        s = lines[i]
        if not s.strip() or s.startswith('# '):
            i += 1
            continue
        if s.startswith('## '):
            title, sid = s[3:], 'section-' + str(len(toc) + 1)
            toc.append((sid, title))
            blocks.append(f'<h2 id="{sid}">{inline(title)}</h2>')
            i += 1
            continue
        if s.startswith('### '):
            blocks.append('<h3>' + inline(s[4:]) + '</h3>')
            i += 1
            continue
        if s.startswith('!['):
            match = re.fullmatch(r'!\[([^\]]*)\]\(([^)]+)\)', s)
            if match and match[2] == 'technical-architecture.svg':
                svg = (DOCS / match[2]).read_text()
                blocks.append('<figure class="architecture-figure"><div class="diagram-tools"><strong>架构图视图</strong>' +
                              ''.join('<button type="button" data-diagram-focus="' + key + '" aria-pressed="' + ('true' if key == 'all' else 'false') + '">' + label + '</button>' for key, label in [('all', '全景'), ('request', '调用路径'), ('control', '管理路径'), ('data', '数据与恢复')]) +
                              '<a href="technical-architecture.svg" target="_blank" rel="noopener">单独打开 ↗</a></div><div class="diagram-scroll" tabindex="0" role="region" aria-label="总体架构图，可横向滚动">' + svg +
                              '</div><figcaption id="diagram-status" aria-live="polite">当前展示全部关系；可横向滚动或单独打开架构图。</figcaption></figure>')
                i += 1
                continue
            if match and match[2] == 'deployment-modes.svg':
                blocks.append('<figure class="deployment-figure"><div class="deployment-scroll" tabindex="0" role="region" aria-label="四种部署方式，可横向滚动"><img src="deployment-modes.svg" alt="' + html.escape(match[1], quote=True) + '"></div><figcaption>交付规划，尚未部署验收。四种方式共享同一业务与数据契约。 <a href="deployment-modes.svg" target="_blank" rel="noopener">单独打开 ↗</a></figcaption></figure>')
                i += 1
                continue
        if s.startswith('|'):
            rows = []
            while i < len(lines) and lines[i].startswith('|'):
                cells = [c.strip() for c in lines[i].strip('|').split('|')]
                if not all(re.fullmatch(r':?-+:?', c) for c in cells):
                    rows.append(cells)
                i += 1
            blocks.append('<div class="table-wrap" tabindex="0" role="region" aria-label="比较表"><table><thead><tr>' + ''.join('<th scope="col">' + inline(c) + '</th>' for c in rows[0]) + '</tr></thead><tbody>' + ''.join('<tr>' + ''.join('<td>' + inline(c) + '</td>' for c in row) + '</tr>' for row in rows[1:]) + '</tbody></table></div>')
            continue
        para = [s]
        i += 1
        while i < len(lines) and lines[i].strip() and not lines[i].startswith(('#', '|')):
            para.append(lines[i])
            i += 1
        blocks.append('<p>' + inline(' '.join(para)) + '</p>')
    return toc, '\n'.join(blocks)

pages = [
    ('technical-architecture', '技术选型<br>与总体架构', '独立掌握企业治理，<br>让调用、记录与成本形成闭环。', '总体架构 / 建议稿', '技术组合 × 四类部署 × 一致性与恢复', '本地开发、Docker 单应用容器、Compose 和集群纳入交付规划，Compose 为默认安装入口；同一代码与运行角色适配不同部署。正文仍采用企业指定文件目录、JSONL 写入与可选 Parquet 封存；第 10 节列明交付工程和验收条件。'),
    ('technical-research', '技术可行性调研<br>与选型建议', '复用模型适配，<br>掌握网关与企业治理。', '总体研究 / 四模块初步路线', '候选比较 + 自有网关与治理边界', '四大模块的初步调研已覆盖。当前建议以 Rust 掌握网关与业务治理，复用成熟适配和身份、检测组件，以 PostgreSQL 保存权威业务记录。'),
    ('gateway-research', '网关能力、技术栈<br>与模型范围', '从模型接入到运行治理，<br>比较我们真正需要的能力。', '网关专项 / 完整产品参照', '调用能力 × 实现栈 × 模型覆盖', '本页保留完整网关比较；当前研究收敛到 LiteLLM 核心复用。完整 Proxy、Python Core SDK 与 Rust 组件的能力分别核查。'),
    ('litellm-reuse-research', 'LiteLLM Rust 支持<br>与 ZenRouter 复用方案', '网关由我们掌握，<br>成熟适配按边界复用。', '深入研究 / 固定版本源码', 'Rust 覆盖 × SDK 封装 × 独立架构', '推荐先用 Rust 构建自有网关，通过内部适配组件复用 LiteLLM Python Core SDK。原生 Rust 组件按协议链路验证后逐步引入。'),
    ('governance-research', '组织、审计与额度<br>三个模块的技术路线', '明确复用边界，<br>收敛企业治理的技术基线。', '治理研究 / 阶段建议', '组织与权限 × 安全与审计 × 额度与成本', '推荐：标准身份接入与自有授权、可替换检测器与可靠留存、PostgreSQL 权威额度账本。三条路线均保留替代方案与待验证项，当前不进入实现细节。'),
]
for slug, title, lead, stage, scope, conclusion in pages:
    toc, content = body((DOCS / (slug + '.md')).read_text())
    title_plain = title.replace('<br>', '')
    nav = ''.join(f'<a href="#{sid}">{inline(label)}</a>' for sid, label in toc)
    architecture = slug == 'technical-architecture'
    version = '0.4' if architecture else ('0.4' if slug in ('technical-research', 'governance-research') else '0.3')
    evidence = '官方资料与仓库核查。组件尚未部署实测。' if slug == 'governance-research' else '官方资料与局部源码核查。组件尚未部署实测。'
    if architecture:
        evidence = '推荐方案与待验证门槛。未进行后端部署、性能或故障测试。'
    date = '2026.10.30' if architecture else '2026.10.29'
    kicker = 'TECHNICAL ARCHITECTURE' if architecture else 'TECHNOLOGY RESEARCH'
    style = CSS + (ARCH_CSS if architecture else '')
    script = ARCH_SCRIPT if architecture else ''
    kind = '技术架构' if architecture else '技术研究'
    result = f'''<!doctype html>
<html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="color-scheme" content="light"><title>{title_plain} · ZenRouter</title><style>{style}</style></head><body><div class="shell report"><header class="topbar"><a class="brand" href="product-overview.html"><span class="logo">Z</span>ZenRouter</a><button id="print" type="button">打印 / 保存 PDF ↗</button></header><main><div class="hero"><div><div class="kicker">{kicker} / {version[-1]:0>2}</div><h1>{title}</h1><p class="lead">{lead}</p></div><aside class="hero-aside"><p>{date} · {stage}</p><p>{scope}</p><p>{evidence}</p></aside></div><div class="callout"><p>{inline(conclusion)}</p></div><nav aria-label="报告目录">{nav}</nav>{content}</main><footer><span>ZenRouter · {kind} v{version}</span><div class="related"><a href="technical-architecture.html">技术架构 ↗</a><a href="product-overview.html">产品全景 ↗</a><a href="governance-research.html">三模块调研 ↗</a><a href="litellm-reuse-research.html">LiteLLM 复用 ↗</a><a href="gateway-research.html">网关专项 ↗</a><a href="technical-research.html">总体调研 ↗</a><a href="{slug}.md">Markdown 版本 ↗</a></div></footer></div><script>document.getElementById("print").addEventListener("click",function(){{window.print()}});{script}</script></body></html>'''
    (DOCS / (slug + '.html')).write_text(result)
    print('Rendered', slug)
