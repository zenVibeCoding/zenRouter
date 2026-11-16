import { Layout, Menu, Tag, Typography } from 'antd'
import type { MenuProps } from 'antd'
import { Route, Routes, useLocation, useNavigate } from 'react-router'

import { domains, pages } from './nav'
import { NotFound } from './pages/NotFound'
import { Overview } from './pages/Overview'
import { Placeholder } from './pages/Placeholder'

const { Sider, Header, Content } = Layout

function menuItems(): MenuProps['items'] {
  const label = (title: string, phase?: string) =>
    phase ? (
      <span>
        {title}{' '}
        <Tag color="gold" style={{ marginInlineStart: 4 }}>
          {phase}
        </Tag>
      </span>
    ) : (
      title
    )
  return [
    ...pages.filter((p) => !p.domain).map((p) => ({ key: p.path, label: label(p.title) })),
    ...domains.map((d) => ({
      type: 'group' as const,
      key: d,
      label: d,
      children: pages
        .filter((p) => p.domain === d)
        .map((p) => ({ key: p.path, label: label(p.title, p.phase) })),
    })),
  ]
}

export function App() {
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const current = pages.find((p) => p.path === pathname)

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider width={220} breakpoint="lg" collapsedWidth={0}>
        <div style={{ padding: '20px 24px', color: '#fff' }}>
          <Typography.Title level={4} style={{ color: '#fff', margin: 0 }}>
            ZenRouter
          </Typography.Title>
          <Typography.Text style={{ color: 'rgba(255,255,255,.65)', fontSize: 12 }}>
            企业 AI 路由与治理
          </Typography.Text>
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[pathname]}
          items={menuItems()}
          onClick={({ key }) => navigate(key)}
        />
      </Sider>
      <Layout>
        <Header style={{ paddingInline: 24, borderBottom: '1px solid #e2e8e0' }}>
          <Typography.Text strong>{current?.title ?? '页面不存在'}</Typography.Text>
          {current?.domain && (
            <Typography.Text type="secondary" style={{ marginInlineStart: 12 }}>
              {current.domain}
            </Typography.Text>
          )}
        </Header>
        <Content style={{ padding: 24 }}>
          <Routes>
            <Route path="/" element={<Overview />} />
            {pages
              .filter((p) => p.path !== '/')
              .map((p) => (
                <Route key={p.path} path={p.path} element={<Placeholder page={p} />} />
              ))}
            <Route path="*" element={<NotFound />} />
          </Routes>
        </Content>
      </Layout>
    </Layout>
  )
}
