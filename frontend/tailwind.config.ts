import type { Config } from 'tailwindcss'

const config: Config = {
    content: [
        './index.html',
        './src/**/*.rs',
    ],
    theme: {
        extend: {
            colors: {
                aeroBlue: '#7FDBFF',
                aeroGreen: '#2ECC40',
                aeroPink: '#FF69B4',
                aeroPurple: '#B10DC9',
                aeroGlass: 'rgba(255, 255, 255, 0.2)',
            },
            boxShadow: {
                glass: '0 4px 30px rgba(0, 0, 0, 0.1)',
            },
            backdropBlur: {
                xs: '2px',
            },
            borderRadius: {
                xl: '1rem',
            },
            fontFamily: {
                frutiger: ['"Exo 2"', 'sans-serif'],
            }
        },
    },
    plugins: [],
}
export default config;
