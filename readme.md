# Installation
This installation instruction is written for Ubuntu 16.04. If you are not working with Ubuntu 16.04 please use the os specific commands of your system.

## install rust
install rust from the the homepage
```bash
curl https://sh.rustup-rs -sSf | sh
```
add cargo to PATH variable
```bash
echo "export PATH=/home/{USER}/.cargo/bin:\$PATH" >> ~/.bashrc
```
restart or source shell

## clone the rtos repository
clone the repo
```bash
git clone https://github.com/beschaef/rtos.git
```
move into the repo
```bash
cd rtos
```

## change the rust toolchain to 1.27.0-nightly
this may take a while
```bash
rustup override add nightly-2018-04-15
```

## add rust-src as component
```bash
rustup component add rust-src
```

## install cargo crates
this may take a while
```bash
cargo install bootimage
cargo install xargo
cargo install cargo-xbuild
```

## install qemu
install qemu-x86_64 with version 2.5.0
this may take a while
```bash
sudo apt-get install qemu
```

## compile and run the os
this may take a while
```bash
make all
```