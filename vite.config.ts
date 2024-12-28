import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { internalIpV4 } from "internal-ip";
import path from "node:path";

const mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM);

const pathSrc = path.resolve(__dirname, "src");

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  // build: {
  //   outDir: "./src-tauri/static",
  // },

  resolve: {
    alias: {
      "~/": `${pathSrc}/`,
    },
    conditions: ["development", "browser"],
  },

  envPrefix: ["TAURI_ENV_PLATFORM"],

  plugins: [solid()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: mobile ? "0.0.0.0" : false,
    hmr: mobile
      ? {
          protocol: "ws",
          host: await internalIpV4(),
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
