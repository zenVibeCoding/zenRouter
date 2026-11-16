import { Button, Result } from 'antd'
import { useNavigate } from 'react-router'

export function NotFound() {
  const navigate = useNavigate()
  return (
    <Result
      status="404"
      title="页面不存在"
      extra={
        <Button type="primary" onClick={() => navigate('/')}>
          返回工作总览
        </Button>
      }
    />
  )
}
