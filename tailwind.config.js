/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./src/**/*.rs"],
    theme: {
        extend: {
            colors: {
                bg: "var(--color-bg)",
                "bg-secondary": "var(--color-bg-secondary)",
                "bg-tertiary": "var(--color-bg-tertiary)",
                text: "var(--color-text)",
                "text-muted": "var(--color-text-muted)",
                accent: "var(--color-accent)",
                border: "var(--color-border)",
                success: "var(--color-success)",
                danger: "var(--color-danger)",
                warning: "var(--color-warning)",
            },
        },
    },
    plugins: [
        require("@tailwindcss/typography"),
    ],
};
