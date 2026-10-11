/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{rs,html}"],
  theme: {
    extend: {
      fontFamily: {
        mono: ['JetBrains Mono','ui-monospace','SFMono-Regular','monospace'],
        pixel: ['Press Start 2P','monospace']
      },
      colors: {
        ink: '#080a0f',
        panel: '#11131a',
        line: '#24283b',
        mario: '#e52521',
        coin: '#facc15',
        pipe: '#43b047'
      },
      boxShadow: { glow: '0 0 24px rgba(99,102,241,0.35)' }
    }
  },
  plugins: []
}
