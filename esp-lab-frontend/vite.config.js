import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    preact(),
    tailwindcss(),
  ],
  // server: {
  //   proxy: {
  //     // Proxy requests from http://localhost:5173/api to the target URL
  //     '/gpio': {
  //       target: 'http://192.168.0.9', // The address of your backend server
  //       changeOrigin: true, // Needed for virtual hosted sites
  //       secure: false, // If the target is HTTPS and you have issues with self-signed certs
  //       // rewrite: (path) => path.replace(/^\/api/, ''), // Optional: rewrite the path
  //     },
  //   },
  // },
})
