# demo

![demo_short](https://github.com/okawak/MHV4_monitor/assets/116426897/769a5ec2-58d8-43dc-85af-4548e1b7c1fc)

(This is old version)

# server side

## install

prepare Rust lang environment. see https://www.rust-lang.org/tools/install

```shell
git clone https://github.com/okawak/MHV4_monitor.git
cd MHV4_monitor
```

## test

```shell
cargo test -- --nocapture
```

## send just one command

```shell
cargo run --bin command -- "COMMAND"
# or
./command.sh "COMMAND"
```

## usage

set the configuration at the "run.sh"

then you can generate server at 0.0.0.0:8080 by

```shell
./run.sh
```

# client side

prepare npm environment

```shell
cd public
npm install
```

set the server IP address in .env file

```shell
vi .env
```

write MHV4 descriptions in "public/src/app/pages.tsx"

```shell
vi src/app/pages.tsx
```

make a static files

```shell
npm run build
```

you can see the page from "public/out" directory.

If you want to use localhost, just "https:IPaddress" is okay.
