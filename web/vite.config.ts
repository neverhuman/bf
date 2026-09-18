import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  build: { outDir: "dist", emptyOutDir: true },
  server: { proxy: { "/v3": "http://127.0.0.1:7420", "/health": "http://127.0.0.1:7420" } },
  test: { environment: "jsdom", include: ["src/**/*.test.tsx"] },
});
