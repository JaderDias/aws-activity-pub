# aws-activity-pub
[![build status](https://github.com/JaderDias/aws-activity-pub/workflows/Rust/badge.svg)](https://github.com/JaderDias/aws-activity-pub/actions?query=workflow%3ARust)
[![lint status](https://github.com/JaderDias/aws-activity-pub/workflows/Linter/badge.svg)](https://github.com/JaderDias/aws-activity-pub/actions?query=workflow%3ALinter)
[![dependency status](https://github.com/JaderDias/aws-activity-pub/workflows/Dependencies/badge.svg)](https://github.com/JaderDias/aws-activity-pub/actions?query=workflow%3ADependencies)

[![codecov](https://codecov.io/gh/JaderDias/aws-activity-pub/branch/main/graph/badge.svg?token=RBY2XLZV9G)](https://codecov.io/gh/JaderDias/aws-activity-pub)
[![Coverage Status](https://coveralls.io/repos/github/JaderDias/aws-activity-pub/badge.svg)](https://coveralls.io/github/JaderDias/aws-activity-pub)


[![deps.rs](https://deps.rs/repo/github/JaderDias/aws-activity-pub/status.svg)](https://deps.rs/repo/github/JaderDias/aws-activity-pub)
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/JaderDias/aws-activity-pub.svg)](http://isitmaintained.com/project/JaderDias/aws-activity-pub "Average time to resolve an issue")
[![Percentage of issues still open](http://isitmaintained.com/badge/open/JaderDias/aws-activity-pub.svg)](http://isitmaintained.com/project/JaderDias/aws-activity-pub "Percentage of issues still open")

Example infrastructure as code (IaC) of how to
deploy an ActivityPub server to AWS

## Supported development hosts

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
make test
```

## Test coverage

[![sunburst](https://codecov.io/gh/JaderDias/aws-activity-pub/branch/main/graphs/sunburst.svg?token=RBY2XLZV9G)](https://coveralls.io/github/JaderDias/aws-activity-pub)
