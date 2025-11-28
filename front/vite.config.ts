import {defineConfig} from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
    plugins: [vue()],
    build: {
        // ビルド成果物を api_server/public に出力
        outDir: '../api_server/public',
        // プロジェクト外ディレクトリでもクリーンアップを許可
        emptyOutDir: true,
    },
    server: {
        proxy: {
            '/api': {
                target: 'http://127.0.0.1:8080',
                changeOrigin: true,
            },
        },
    },
})
