/** @type {import("prettier").Config} */
const config = {
    tabWidth: 4,
    printWidth: 120,
    overrides: [
        {
            files: "*.{json,json5,html,yaml,yml}",
            options: {
                tabWidth: 2,
            },
        },
        {
            files: "*.md",
            options: {
                tabWidth: 2,
                trailingComma: "none",
            },
        },
    ],
};

export default config;
