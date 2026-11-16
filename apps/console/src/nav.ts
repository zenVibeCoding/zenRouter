// 控制台信息架构，沿用 docs/prototype.html 的页面入口与四个业务域分组。
// 原型中的“功能地图”只用于讨论，不进入正式控制台。

export type Domain = '组织与权限' | '模型与路由' | '安全与审计' | '额度与成本' | '公共能力'

export interface NavPage {
  path: string
  title: string
  domain?: Domain
  phase?: 'P1'
}

export const pages: NavPage[] = [
  { path: '/', title: '工作总览' },

  { path: '/org', title: '组织与成员', domain: '组织与权限' },
  { path: '/apps', title: '应用与密钥', domain: '组织与权限' },
  { path: '/roles', title: '角色与授权', domain: '组织与权限' },
  { path: '/identity', title: '身份接入', domain: '组织与权限' },

  { path: '/models', title: '模型与渠道', domain: '模型与路由' },
  { path: '/routes', title: '路由策略', domain: '模型与路由' },
  { path: '/cache', title: '缓存与兼容', domain: '模型与路由' },
  { path: '/experiments', title: '质量与实验', domain: '模型与路由', phase: 'P1' },

  { path: '/requests', title: '调用记录', domain: '安全与审计' },
  { path: '/policies', title: '安全策略', domain: '安全与审计' },
  { path: '/risks', title: '风险复核', domain: '安全与审计' },
  { path: '/audit', title: '审计与留存', domain: '安全与审计' },

  { path: '/budgets', title: '额度分配', domain: '额度与成本' },
  { path: '/costs', title: '成本分析', domain: '额度与成本' },
  { path: '/suppliers', title: '供应商对账', domain: '额度与成本' },

  { path: '/operations', title: '平台运维', domain: '公共能力' },
]

export const domains: Domain[] = ['组织与权限', '模型与路由', '安全与审计', '额度与成本', '公共能力']
