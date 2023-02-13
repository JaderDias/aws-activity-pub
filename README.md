[![build status](https://github.com/JaderDias/aws-activity-pub/workflows/Rust/badge.svg)](https://github.com/JaderDias/aws-activity-pub/actions?query=workflow%3ARust)
[![lint status](https://github.com/JaderDias/baby_schema/workflows/Linter/badge.svg)](https://github.com/JaderDias/baby_schema/actions?query=workflow%3ALinter)
[![dependency status](https://deps.rs/repo/github/JaderDias/aws-activity-pub/status.svg)](https://deps.rs/repo/github/JaderDias/aws-activity-pub)
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/JaderDias/aws-activity-pub.svg)](http://isitmaintained.com/project/JaderDias/aws-activity-pub "Average time to resolve an issue")
[![Percentage of issues still open](http://isitmaintained.com/badge/open/JaderDias/aws-activity-pub.svg)](http://isitmaintained.com/project/JaderDias/aws-activity-pub "Percentage of issues still open")
# aws-activity-pub
Example infrastructure as code (IaC) of how to
deploy an ActivityPub server to AWS

## Supported hosts

* Linux
* MacOS

## Requirements

### Development & testing

* Docker Desktop up and running
* docker-compose
* gcc
* Rust toolchain

### additional deployment requirements

* AWS Command Line Interface

### additional macOS with Apple Silicon requirements

* musl-cross with x86_64
```bash
brew install filosottile/musl-cross/musl-cross --with-x86_64
```

## Run tests locally

```bash
./run.sh
```

on another terminal

```bash
./run_test.sh
```
