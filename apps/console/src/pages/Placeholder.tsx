import { Empty, Typography } from 'antd'

import type { NavPage } from '../nav'

export function Placeholder({ page }: { page: NavPage }) {
  return (
    <Empty
      style={{ marginTop: 80 }}
      description={
        <>
          <Typography.Title level={5}>{page.title}</Typography.Title>
          <Typography.Text type="secondary">
            {page.domain} · 尚未实现{page.phase ? `（${page.phase} 范围）` : ''}。页面结构参考
            docs/prototype.html。
          </Typography.Text>
        </>
      }
    />
  )
}
