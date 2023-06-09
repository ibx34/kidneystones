/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "**/*.{html,js}",
    "../templates/*.hbs",
    "../templates/**/*.hbs",
    "./js/*.js"
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}

