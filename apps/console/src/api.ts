// 管理 API 客户端。类型与 crates/contracts 保持一致；后续由 api/openapi 规范生成。

export type CheckStatus = 'ok' | 'degraded' | 'unavailable' | 'skipped'

export interface DependencyReport {
  name: string
  status: CheckStatus
  detail?: string
}

export interface ReadinessReport {
  ready: boolean
  draining: boolean
  dependencies: DependencyReport[]
}

export interface SystemInfo {
  name: string
  version: string
  roles: string[]
  adapter_contract_version: string
}

export class ApiError extends Error {
  readonly status: number

  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

async function getJson<T>(path: string, acceptStatus: number[] = []): Promise<T> {
  const resp = await fetch(path, {
    headers: { accept: 'application/json' },
    credentials: 'same-origin',
  })
  if (!resp.ok && !acceptStatus.includes(resp.status)) {
    throw new ApiError(resp.status, `${path} 返回 ${resp.status}`)
  }
  return (await resp.json()) as T
}

export const api = {
  systemInfo: () => getJson<SystemInfo>('/api/v1/system/info'),
  // 未就绪时返回 503，响应体仍是 ReadinessReport。
  readiness: () => getJson<ReadinessReport>('/readyz', [503]),
}
