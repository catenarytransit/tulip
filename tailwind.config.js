/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        files: ["*.html", "./src/**/*.rs"],
    },
    darkMode: "class",
    theme: {
        extend: {
            colors: {
                tulip: '#E470AB',
                darksky: '#0A233F',
                darkskyvariant: '#06121F',
                seashore: '#42A7C5',
                gray: '#EDEDED'
            }
        },
    },
    plugins: [],
}