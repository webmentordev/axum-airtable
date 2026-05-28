// https://nuxt.com/docs/api/configuration/nuxt-config
import tailwindcss from "@tailwindcss/vite";
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  ssr: false,
  css: ['./app/assets/css/main.css'],
  vite: {
    plugins: [
      tailwindcss(),
    ],
  },
  runtimeConfig: {
    apiUrl: '',
  },
  app: {
    head: {
      charset: 'utf-16',
      viewport: 'width=device-width, initial-scale=1, maximum-scale=1',
      title: 'Windtable | Enterprice Spread sheets solution',
      htmlAttrs: {
        lang: 'en',
      },
      link: [
        { rel: 'icon', type: 'image/x-icon', href: '/wind.png' },
      ],
    },
  },
})
