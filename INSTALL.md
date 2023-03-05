Getting started
---------------

Build dependencies:

 * libaio
 * libblkid
 * libkeyutils
 * liblz4
 * libsodium
 * liburcu
 * libuuid
 * libzstd
 * pkg-config
 * valgrind
 * zlib1g

In addition a recent Rust toolchain is required (rustc, cargo), either by using
[rustup](https://rustup.rs/) or make sure to use a distribution where rustc (>=1.64)
is available.

Debian (Bullseye or later) and Ubuntu (20.04 or later): you can install these with

``` shell
apt install -y pkg-config libaio-dev libblkid-dev libkeyutils-dev \
    liblz4-dev libsodium-dev liburcu-dev libzstd-dev \
    uuid-dev zlib1g-dev valgrind libudev-dev git build-essential \
    python3 python3-docutils
```

Fedora: install the "Development tools" group along with:
```shell
dnf install -y libaio-devel libsodium-devel \
    libblkid-devel libzstd-devel zlib-devel userspace-rcu-devel \
    lz4-devel libuuid-devel valgrind-devel keyutils-libs-devel \
    findutils
```

Arch: install bcachefs-tools-git from the AUR.
Or to build from source, install build dependencies with
```shell
pacman -S base-devel libaio keyutils libsodium liburcu zstd valgrind
```

Then, just `make && make install`


Experimental features
---------------------

Experimental fuse support is currently disabled by default. Fuse support is at
an early stage and may corrupt your filesystem, so it should only be used for
testing. To enable, you'll also need to add:

* libfuse3 >= 3.7

On Debian/Ubuntu (Bullseye/20.04 or later needed for libfuse >= 3.7):
```shell
apt install -y libfuse3-dev
```

On Fedora (32 or later needed for lbifuse >= 3.7):
```shell
dnf install -y fuse3-devel
```

Arch:
```shell
pacman -S fuse3
```

Then, make using the `BCACHEFS_FUSE` environment variable (make clean first if
previously built without fuse support):

```shell
BCACHEFS_FUSE=1 make && make install
```

Tests
-----
Some tests are available to validate the `bcachefs` binary.  The tests depend
on python3 pytest.

On debian:
```shell
apt install -u python3-pytest
```

Then, you can run the tests via:

```shell
make check
# or if pytest has a different name
make check PYTEST=pytest
```

Optionally, you may wish to run tests in parallel using python3-pytest-xdist:

```shell
cd tests; pytest-3 -n4
```