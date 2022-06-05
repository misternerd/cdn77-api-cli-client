# CDN77 Client

This is a command line client for the [CDN77 V3 API](https://client.cdn77.com/support/api-reference/v3/introduction) written in Rust.
The intended usage is for CI/CD pipelines, where one wants to prefetch or purge CDN resources after deploying a new version of a web application.

The client uses [Clap](https://clap.rs) for argument parsing, so all arguments are available in short and long form and should be documented well enough.

## API Token
To use this client, you'll need to provide a CDN77 API token. This can be obtained from your account dashboard via settings. To provide the token, you can
either provide it as an argument via `-a YOUR_TOKEN` which is somewhat **INSECURE**. Much preferred is to set your token via the environment
variable `CDN77_API_TOKEN`, as this cannot be read from the process list.
Alternatively, you can create a `.env` file in the working directory of the client and declare the `CDN77_API_TOKEN` variable in there.q


## Static Build
Especially for CI/CD pipelines, it might prove useful to create a static binary. This will use the [musl libc](https://www.musl-libc.org/), so you need to
provide the necessary packages to build it.

1. Add the required packages to your system, on Debian/Ubuntu, this will be `musl-tools`, `pkg-config` and `libssl-dev`
2. Add the static musl target to Rust using `rustup target add x86_64-unknown-linux-musl`
3. Build the client with `cargo build --target x86_64-unknown-linux-musl --release`
4. Find your binary in `target/x86_64-unknown-linux-musl/release/cdn77-client` (size is ~13M)
