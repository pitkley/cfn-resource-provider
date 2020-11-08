# cfn-resource-provider

This library is a relatively thin wrapper enabling the use of Rust in AWS Lambda to provide an
AWS CloudFormation [custom resource]. It has to be used in conjunction with the official
[Rust AWS Lambda runtime].

[custom resource]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/template-custom-resources.html
[Rust AWS Lambda runtime]: https://github.com/awslabs/aws-lambda-rust-runtime

## Quick start example

```rust
use cfn_resource_provider::CfnRequest;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(cfn_resource_provider::process(resource)).await;
    Ok(())
}

async fn resource(request: CfnRequest<MyResourceProperties>) -> Result<Option<()>, Error> {
    // Perform the necessary steps to create the custom resource. Afterwards you can return some
    // data that should be serialized into the response. If you don't want to serialize any
    // data, you can return `None`. (Please note that you _have to_ return an `Option`!)
    Ok(None)
}
```

## License

This library is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
cfn-resource-provider by you, as defined in the Apache-2.0 license, shall be dual licensed as
above, without any additional terms or conditions.
