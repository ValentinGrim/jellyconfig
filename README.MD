# 🍓 JellyConfig 🍓
## Overview

JellyConfig is a simple helper tool built with Rust.  
It allows you to **export** your current Jellyfin configuration files and database, and **import** them onto another Jellyfin instance.

## Current Support

- Only **Linux-based servers** running Jellyfin **directly** (not in Docker) are supported.
- No support for **Windows** or **macOS** is planned.
- Dockerized Jellyfin support *might* be added in the future, depending on my motivation.

## Usage
### Export your configuration

```sh
jellyconfig export -o my_config.jexport
```

### Import an exported configuration

```sh
sudo jellyconfig import -i my_config.jexport
```

### Important Note

For a **clean import**, it is **strongly recommended** to run the import **before** installing Jellyfin on your target machine.  