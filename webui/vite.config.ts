import { defineConfig, type Plugin } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'node:path';

const env = (name: string, fallback: string): string =>
  process.env[name] || fallback;

const ksuPreviewMock = (): Plugin => ({
  name: 'znn-ksu-preview-mock',
  apply: 'serve',
  transformIndexHtml: {
    order: 'pre',
    handler: (html) => ({
      html,
      tags: [
        {
          tag: 'script',
          attrs: { src: '/dev/ksu-mock.js' },
          injectTo: 'head',
        },
      ],
    }),
  },
});

const dropIconFont = env('ZNN_WEBUI_DROP_ICON_FONT', '0') === '1';
const minify = env('ZNN_WEBUI_MINIFY', '1') !== '0';

export default defineConfig(() => ({
  base: './',
  resolve: {
    alias: dropIconFont
      ? {
          '@material-symbols/font-400/outlined.css': resolve(
            import.meta.dirname,
            'src/styles/no-icon-font.css',
          ),
        }
      : undefined,
  },
  plugins: [
    vue({
      template: {
        compilerOptions: {
          isCustomElement: (tag) => tag.startsWith('m3e-'),
        },
      },
    }),
    tailwindcss(),
    ksuPreviewMock(),
  ],
  define: {
    __ZNN_MODULE_ID__: JSON.stringify(env('ZNN_MODULE_ID', 'zygisknextsu')),
    __ZNN_MODULE_NAME__: JSON.stringify(env('ZNN_MODULE_NAME', 'ZNNext')),
    __ZNN_VER_NAME__: JSON.stringify(env('ZNN_VER_NAME', 'v0.0.0')),
    __ZNN_COMMIT_HASH__: JSON.stringify(env('ZNN_COMMIT_HASH', 'unknown')),
    __VUE_PROD_DEVTOOLS__: 'false',
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: 'false',
  },
  server: {
    open: true,
  },
  build: {
    outDir: resolve(import.meta.dirname, '../module/webroot'),
    emptyOutDir: true,
    target: 'es2020',
    sourcemap: false,
    minify,
  },
}));
