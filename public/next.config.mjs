/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "export",

  // Optional: Change links `/me` -> `/me/` and emit `/me.html` -> `/me/index.html`
  // trailingSlash: true,

  // Optional: Prevent automatic `/me` -> `/me/`, instead preserve `href`
  // skipTrailingSlashRedirect: true,

  // Optional: Change the output directory `out` -> `dist`
  // distDir: "static",

  //async headers() {
  //  return [
  //    {
  //      source: "/api/:path*",
  //      headers: [
  //        { key: "Access-Control-Allow-Credentials", value: "true" },
  //        { key: "Access-Control-Allow-Origin", value: "*" },
  //        {
  //          key: "Access-Control-Allow-Methods",
  //          value: "GET,OPTIONS,PATCH,DELETE,POST,PUT",
  //        },
  //        {
  //          key: "Access-Control-Allow-Headers",
  //          value:
  //            "X-CSRF-Token, X-Requested-With, Accept, Accept-Version, Content-Length, Content-MD5, Content-Type, Date, X-Api-Version",
  //        },
  //      ],
  //    },
  //  ];
  //},
};

export default nextConfig;
