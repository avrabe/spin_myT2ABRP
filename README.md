# Inofficial Toyota MyT (Europe) to A Better Route Planner Gateway using Fermyon Spin

Goal is to provide a gateway between the Toyota MyT (Europe) and A Better Route Planner for Telemetry information. The Toyota MyT uses an inofficial API described in [tojota]. So any change from Toyota might break this application.

## Setup the development environment

Install rust and the webassembly target

```sh
# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Install the wasm32-wasi target
rustup target add wasm32-wasi
```

Install spin

```sh
# Install spin
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
# Move to a location for use
sudo mv spin /usr/local/bin/
```

Export the secrets used

```sh
export SPIN_CONFIG_USERNAME=joe@doe.com
export SPIN_CONFIG_PASSWORD=mysecret
export SPIN_CONFIG_VIN=5YFBURHE3JP743261
```

Build the application and start the local server

```sh
spin build
spin up
# Or for active development
spin watch
```

Query the system

```sh
curl http://12.0.0.1:3000/
```

[tojota]: ttps://github.com/calmjm/tojota/tree/maste
