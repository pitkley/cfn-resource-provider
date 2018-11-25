# cfn-resource-provider

This library is a relatively thin wrapper enabling the use of Rust in AWS Lambda to provide an
AWS CloudFormation [custom resource]. It is intended to be used in conjunction with
[`rust-aws-lambda`][rust-aws-lambda], a library that enables to run Rust applications serverless
on AWS Lambda using the Go 1.x runtime.

[custom resource]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/template-custom-resources.html
[rust-aws-lambda]: https://github.com/srijs/rust-aws-lambda

## License

This library is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in DFW by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
