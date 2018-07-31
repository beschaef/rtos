# Installation
This installation instruction is written for Ubuntu 16.04. If you are not working with Ubuntu 16.04 please use the os specific commands of your system.

## install rust
install rust from the the homepage
```bash
curl https://sh.rustup-rs -sSf | sh
```

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
```bash
rustup override add nightly-2018-04-15
```

## compile and run the os
this could may take while the first time
```bash
make run
```