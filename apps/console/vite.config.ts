import react from '@vitejs/plugin-react'
import { defineConfig, loadEnv } from 'vite'

export default defineConfig(({ mode }) => {
  // 开发期把管理 API 与探针代理到本地 control 角色（默认 127.0.0.1:8081）。
  // 可用环境变量 ZENROUTER_CONTROL_URL 覆盖。
  const env = loadEnv(mode, '.', 'ZENROUTER_')
  const control = env.ZENROUTER_CONTROL_URL ?? 'http://127.0.0.1:8081'

  return {
    plugins: [react()],
    server: {
      port: 5173,
      proxy: {
        '/api': control,
        '/readyz': control,
        '/livez': control,
      },
    },
    build: {
      outDir: 'dist',
      sourcemap: true,
    },
  }
})
