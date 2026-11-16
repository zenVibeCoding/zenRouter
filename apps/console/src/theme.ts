import type { ThemeConfig } from 'antd'

// 主题令牌取自 docs/prototype.css 的绿色主题。
export const colors = {
  paper: '#f6f7f3',
  ink: '#1e332b',
  muted: '#718078',
  line: '#e2e8e0',
  green: '#24654c',
  dark: '#173e30',
  soft: '#eaf2eb',
  gold: '#a47229',
  red: '#b34c45',
  blue: '#49759a',
} as const

export const theme: ThemeConfig = {
  token: {
    colorPrimary: colors.green,
    colorInfo: colors.blue,
    colorWarning: colors.gold,
    colorError: colors.red,
    colorSuccess: colors.green,
    colorText: colors.ink,
    colorTextSecondary: colors.muted,
    colorBorder: colors.line,
    colorBgLayout: colors.paper,
    borderRadius: 6,
    fontFamily:
      '-apple-system, BlinkMacSystemFont, "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", "Segoe UI", sans-serif',
  },
  components: {
    Layout: { siderBg: colors.dark, headerBg: '#fff' },
    Menu: {
      darkItemBg: colors.dark,
      darkSubMenuItemBg: colors.dark,
      darkItemSelectedBg: colors.green,
    },
  },
}
