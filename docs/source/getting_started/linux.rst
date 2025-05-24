=========================
Linux Installation Guide
=========================

WRT provides comprehensive support for Linux distributions, from development workstations to production servers.

.. contents:: On this page
   :local:
   :depth: 2

Supported Distributions
=======================

**Tier 1 Support (Fully tested):**

* Ubuntu 20.04 LTS, 22.04 LTS, 24.04 LTS
* Debian 11 (Bullseye), 12 (Bookworm)
* CentOS Stream 8, 9
* Red Hat Enterprise Linux 8, 9
* Fedora 38, 39, 40

**Tier 2 Support (Community tested):**

* openSUSE Leap 15.4+
* Arch Linux
* Alpine Linux 3.17+
* Amazon Linux 2

Supported Architectures
=======================

* **x86_64** (Intel/AMD 64-bit) - Primary platform
* **aarch64** (ARM 64-bit) - Full support
* **armv7** (ARM 32-bit) - Limited support
* **riscv64** (RISC-V 64-bit) - Experimental

Installation Methods
====================

Package Manager Installation
----------------------------

**Ubuntu/Debian (APT):**

.. code-block:: bash

   # Add WRT repository
   curl -fsSL https://packages.example.com/gpg.key | sudo gpg --dearmor -o /usr/share/keyrings/wrt-archive-keyring.gpg
   echo "deb [signed-by=/usr/share/keyrings/wrt-archive-keyring.gpg] https://packages.example.com/debian $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/wrt.list

   # Install WRT
   sudo apt update
   sudo apt install wrt-runtime wrt-dev

**CentOS/RHEL/Fedora (YUM/DNF):**

.. code-block:: bash

   # Add WRT repository
   sudo dnf config-manager --add-repo https://packages.example.com/rpm/wrt.repo

   # Install WRT
   sudo dnf install wrt-runtime wrt-dev

**Arch Linux (AUR):**

.. code-block:: bash

   # Using yay
   yay -S wrt-runtime

   # Using makepkg
   git clone https://aur.archlinux.org/wrt-runtime.git
   cd wrt-runtime
   makepkg -si

Source Installation
-------------------

**Prerequisites:**

.. code-block:: bash

   # Ubuntu/Debian
   sudo apt update
   sudo apt install build-essential curl git pkg-config libssl-dev

   # CentOS/RHEL/Fedora
   sudo dnf groupinstall "Development Tools"
   sudo dnf install curl git pkg-config openssl-devel

   # Arch Linux
   sudo pacman -S base-devel curl git pkg-config openssl

**Install Rust and build:**

.. code-block:: bash

   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env

   # Install just
   cargo install just

   # Clone and build
   git clone https://github.com/your-org/wrt.git
   cd wrt
   just build

Distribution-Specific Notes
==========================

Ubuntu/Debian
--------------

**Required packages:**

.. code-block:: bash

   sudo apt install build-essential curl git pkg-config libssl-dev

**For embedded development:**

.. code-block:: bash

   sudo apt install gcc-arm-linux-gnueabihf gcc-aarch64-linux-gnu

CentOS/RHEL
-----------

**Enable EPEL repository:**

.. code-block:: bash

   sudo dnf install epel-release

**Required packages:**

.. code-block:: bash

   sudo dnf groupinstall "Development Tools"
   sudo dnf install curl git pkg-config openssl-devel

Fedora
------

**Required packages:**

.. code-block:: bash

   sudo dnf install @development-tools curl git pkg-config openssl-devel

Alpine Linux
------------

**Required packages:**

.. code-block:: bash

   sudo apk add build-base curl git pkgconfig openssl-dev

**Note:** Alpine uses musl libc, which may require special consideration for some features.

Security Features
=================

Linux-Specific Hardening
-------------------------

WRT leverages Linux security features:

**Control Flow Integrity (CFI):**

.. code-block:: bash

   # Verify CFI support
   cat /proc/cpuinfo | grep -E "(cet|ibp)"

**Memory protection:**

.. code-block:: bash

   # Enable ASLR
   echo 2 | sudo tee /proc/sys/kernel/randomize_va_space

   # Check for hardware CFI support
   dmesg | grep -i "control flow"

**SELinux/AppArmor:**

For production deployment with mandatory access controls:

.. code-block:: bash

   # SELinux policy (example)
   sudo setsebool -P container_manage_cgroup on

   # AppArmor profile
   sudo aa-enforce /etc/apparmor.d/wrt-runtime

Performance Optimization
========================

CPU Features
------------

**Enable CPU-specific optimizations:**

.. code-block:: bash

   # Check CPU features
   cat /proc/cpuinfo | grep flags

   # Build with native optimizations
   export RUSTFLAGS="-C target-cpu=native"
   just build

**NUMA considerations:**

.. code-block:: bash

   # Check NUMA topology
   numactl --hardware

   # Pin to specific NUMA node
   numactl --cpunodebind=0 --membind=0 wrtd module.wasm

Memory Configuration
--------------------

**Huge pages support:**

.. code-block:: bash

   # Enable huge pages
   echo 256 | sudo tee /proc/sys/vm/nr_hugepages

   # Verify allocation
   cat /proc/meminfo | grep Huge

**Memory limits:**

.. code-block:: bash

   # Set memory limits with systemd
   sudo systemctl edit wrt-runtime.service

Add:

.. code-block:: ini

   [Service]
   MemoryLimit=1G
   MemoryAccounting=yes

Development Setup
=================

IDE Configuration
-----------------

**VS Code with rust-analyzer:**

.. code-block:: bash

   # Install VS Code
   sudo snap install code --classic

   # Install rust-analyzer extension
   code --install-extension rust-lang.rust-analyzer

**Vim with Rust support:**

.. code-block:: bash

   # Install vim-plug
   curl -fLo ~/.vim/autoload/plug.vim --create-dirs \
       https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim

Add to `~/.vimrc`:

.. code-block:: vim

   call plug#begin()
   Plug 'rust-lang/rust.vim'
   Plug 'dense-analysis/ale'
   call plug#end()

Debugging Tools
---------------

**Install debugging tools:**

.. code-block:: bash

   # GDB with Rust support
   sudo apt install gdb

   # Valgrind for memory debugging
   sudo apt install valgrind

   # perf for performance analysis
   sudo apt install linux-tools-common linux-tools-generic

Testing and Validation
=======================

**Run full test suite:**

.. code-block:: bash

   just ci-full

**Platform-specific tests:**

.. code-block:: bash

   # Test with different glibc versions
   cargo test --target x86_64-unknown-linux-gnu

   # Test with musl
   rustup target add x86_64-unknown-linux-musl
   cargo test --target x86_64-unknown-linux-musl

**Benchmark performance:**

.. code-block:: bash

   cargo bench

Deployment
==========

Systemd Service
---------------

Create `/etc/systemd/system/wrt-runtime.service`:

.. code-block:: ini

   [Unit]
   Description=WRT WebAssembly Runtime
   After=network.target

   [Service]
   Type=simple
   User=wrt
   Group=wrt
   ExecStart=/usr/local/bin/wrtd --config /etc/wrt/config.toml
   Restart=always
   RestartSec=5

   [Install]
   WantedBy=multi-user.target

Enable and start:

.. code-block:: bash

   sudo systemctl enable wrt-runtime
   sudo systemctl start wrt-runtime

Container Deployment
--------------------

**Docker:**

.. code-block:: dockerfile

   FROM ubuntu:22.04
   RUN apt-get update && apt-get install -y wrt-runtime
   COPY config.toml /etc/wrt/
   EXPOSE 8080
   CMD ["wrtd", "--config", "/etc/wrt/config.toml"]

**Podman:**

.. code-block:: bash

   podman run -d --name wrt-runtime \
     -v ./config.toml:/etc/wrt/config.toml:ro \
     -p 8080:8080 \
     wrt:latest

Troubleshooting
===============

Common Issues
-------------

**glibc version mismatch:**

.. code-block:: bash

   # Check glibc version
   ldd --version

   # Use static linking
   export RUSTFLAGS="-C target-feature=+crt-static"

**Permission denied:**

.. code-block:: bash

   # Add user to appropriate groups
   sudo usermod -a -G docker $USER

**Library not found:**

.. code-block:: bash

   # Update library cache
   sudo ldconfig

**Performance issues:**

.. code-block:: bash

   # Check CPU governor
   cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor

   # Set performance mode
   echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

Next Steps
==========

* Try the :doc:`../examples/hello_world` example
* Explore :doc:`../examples/platform/linux_features` 
* Review :doc:`../architecture/platform_layer` for technical details