# Changelog

<!-- next-header -->

## Unreleased

This major release is a breaking change, switching from the discontinued [rust-aws-lambda] runtime to the new, official
[Rust AWS Lambda runtime]. If you previously used version 0.1.1, please consult the [README] to check out the new quick
start example.

* Switch to the official Rust AWS Lambda runtime.
* Support for async/await, using the Tokio 0.3 runtime internally.

    If you are already using a different Tokio version in your application (or a library you are using uses a
    different/older version), you can look into using [tokio-compat] or [tokio-compat-02].

* Rename the `PhysicalResourceIdSuffixProvider` into `PhysicalResourceIdProvider`, enabling the user of the library to
overwrite the entire physical resource ID if desired.

    The updated trait provides a default implementation that has kept the resource ID generation unchanged to the one
    used in version 0.1.1, i.e. you can upgrade to version 1.0.0 without the fear for physical resource ID mismatches.

* Bump of the minimum supported Rust version (MSRV) to 1.40.0.

[rust-aws-lambda]: https://github.com/srijs/rust-aws-lambda
[Rust AWS Lambda runtime]: https://github.com/awslabs/aws-lambda-rust-runtime
[README]: https://github.com/pitkley/cfn-resource-provider/blob/master/README.md
[tokio-compat]: https://crates.io/crates/tokio-compat
[tokio-compat-02]: https://crates.io/crates/tokio-compat-02

## 0.1.1 (2018-11-27)

This library supports you in creating [custom resources for AWS CloudFormation][docs-aws-custom-resources] in a type-safe manner, using Rust. It is meant to be used in conjunction with @srijs [rust-aws-lambda], a library that prepares your Rust binaries to run natively in the Go 1.x runtime of AWS Lambda.
You can consult the documentation for information on how to use this library. Feel free to [open a new issue][new-issue] if you have any questions.

[docs-aws-custom-resources]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/template-custom-resources.html
[rust-aws-lambda]: https://github.com/srijs/rust-aws-lambda
[new-issue]: https://github.com/pitkley/cfn-resource-provider/issues/new

## 0.1.0 (2018-11-27)

This release has been yanked since it has been unsigned.
Version 0.1.0 and 0.1.1 are 100% identical.
