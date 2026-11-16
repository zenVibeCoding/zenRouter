import { useQuery } from '@tanstack/react-query'
import { Alert, Card, Col, Descriptions, Row, Space, Table, Tag, Typography } from 'antd'

import { api } from '../api'
import type { CheckStatus, DependencyReport } from '../api'

const statusTag: Record<CheckStatus, { color: string; text: string }> = {
  ok: { color: 'green', text: '正常' },
  degraded: { color: 'gold', text: '降级' },
  unavailable: { color: 'red', text: '不可用' },
  skipped: { color: 'default', text: '未启用' },
}

export function Overview() {
  const info = useQuery({ queryKey: ['system-info'], queryFn: api.systemInfo })
  const ready = useQuery({
    queryKey: ['readiness'],
    queryFn: api.readiness,
    refetchInterval: 10_000,
  })

  return (
    <Space direction="vertical" size="large" style={{ width: '100%' }}>
      <Alert
        type="info"
        showIcon
        message="项目骨架阶段"
        description="控制台目前只展示运行状态。各业务页面的信息结构见 docs/prototype.html，功能按验证门槛逐步实现。"
      />
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={10}>
          <Card title="实例信息" loading={info.isPending}>
            {info.isError ? (
              <Alert type="error" message="无法连接管理 API" description={String(info.error)} />
            ) : (
              info.data && (
                <Descriptions column={1} size="small">
                  <Descriptions.Item label="版本">{info.data.version}</Descriptions.Item>
                  <Descriptions.Item label="运行角色">
                    {info.data.roles.map((r) => (
                      <Tag key={r}>{r}</Tag>
                    ))}
                  </Descriptions.Item>
                  <Descriptions.Item label="Adapter 契约">
                    {info.data.adapter_contract_version}
                  </Descriptions.Item>
                </Descriptions>
              )
            )}
          </Card>
        </Col>
        <Col xs={24} lg={14}>
          <Card
            title="管理面就绪状态"
            loading={ready.isPending}
            extra={
              ready.data && (
                <Tag color={ready.data.ready ? 'green' : 'red'}>
                  {ready.data.draining ? '排空中' : ready.data.ready ? '就绪' : '未就绪'}
                </Tag>
              )
            }
          >
            {ready.isError ? (
              <Alert type="error" message="无法获取就绪状态" description={String(ready.error)} />
            ) : (
              <Table<DependencyReport>
                size="small"
                rowKey="name"
                pagination={false}
                dataSource={ready.data?.dependencies ?? []}
                columns={[
                  { title: '依赖', dataIndex: 'name' },
                  {
                    title: '状态',
                    dataIndex: 'status',
                    render: (s: CheckStatus) => (
                      <Tag color={statusTag[s].color}>{statusTag[s].text}</Tag>
                    ),
                  },
                  {
                    title: '说明',
                    dataIndex: 'detail',
                    render: (d?: string) => <Typography.Text type="secondary">{d ?? '—'}</Typography.Text>,
                  },
                ]}
              />
            )}
          </Card>
        </Col>
      </Row>
    </Space>
  )
}
