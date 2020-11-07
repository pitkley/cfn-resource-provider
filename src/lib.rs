#![deny(missing_docs)]

//! # cfn-resource-provider
//!
//! This library is a relatively thin wrapper enabling the use of Rust in AWS Lambda to provide an
//! AWS CloudFormation [custom resource]. It is intended to be used in conjunction with
//! [`rust-aws-lambda`][rust-aws-lambda], a library that enables to run Rust applications serverless
//! on AWS Lambda using the Go 1.x runtime.
//!
//! [custom resource]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/template-custom-resources.html
//! [rust-aws-lambda]: https://github.com/srijs/rust-aws-lambda
//!
//! ## Quick start example
//!
//! ```norun
//! extern crate aws_lambda as lambda;
//! extern crate cfn_resource_provider as cfn;
//!
//! use cfn::*;
//!
//! fn main() {
//!     lambda::start(cfn::process(|event: CfnRequest<MyResourceProperties>| {
//!         // Perform the necessary steps to create the custom resource. Afterwards you can return
//!         // some data that should be serialized into the response. If you don't want to serialize
//!         // any data, you can return `None` (were you unfortunately have to specify the unknown
//!         // serializable type using the turbofish).
//!         Ok(None::<()>)
//!     }));
//! }
//! ```
//!
//! ## License
//!
//! This library is licensed under either of
//!
//! * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
//!   http://www.apache.org/licenses/LICENSE-2.0)
//! * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
//!
//! at your option.
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
//! cfn-resource-provider by you, as defined in the Apache-2.0 license, shall be dual licensed as
//! above, without any additional terms or conditions.

extern crate failure;
extern crate futures;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg_attr(test, macro_use)]
extern crate serde_json;

use failure::Error;
use futures::{Future, IntoFuture};
use serde::de::{Deserialize, Deserializer};
use serde::ser::Serialize;

/// Every AWS CloudFormation resource, including custom resources, needs a unique physical resource
/// ID. To aid in supplying this resource ID, your resource property type has to implement this
/// trait with its single member, `physical_resource_id_suffix`.
///
/// When this library creates the response which will be sent to AWS CloudFormation, a physical
/// resource ID will be created according to the following format (where `suffix` will be the suffix
/// provided by the implementor):
///
/// ```norun
/// arn:custom:cfn-resource-provider:::{stack_id}-{logical_resource_id}/{suffix}
/// ```
///
/// The fields of a property-type used for suffix creation should be chosen as such that it changes
/// when ever the custom resource implementation has to create an actual new physical resource. The
/// suffix should also include the type of resource, maybe including a version number.
///
/// ## Example
///
/// Let's assume you have the following type and trait implementation:
///
/// ```
/// # use cfn_resource_provider::*;
/// struct MyResourcePropertiesType {
///     my_unique_parameter: String,
///     some_other_parameter: String,
/// }
/// impl PhysicalResourceIdSuffixProvider for MyResourcePropertiesType {
///     fn physical_resource_id_suffix(&self) -> String {
///         format!(
///             "{resource_type}@{version}/{unique_reference}",
///             resource_type=env!("CARGO_PKG_NAME"),
///             version=env!("CARGO_PKG_VERSION"),
///             unique_reference=self.my_unique_parameter,
///         )
///     }
/// }
/// ```
///
/// When [`CfnResponse`] creates or updates the physical ID for the resource, it might look like the
/// following:
///
/// ```norun
/// arn:custom:cfn-resource-provider:::12345678-1234-1234-1234-1234567890ab-logical-id/myresource@1.0.0/uniquereference
/// ```
///
/// In this case `my_unique_parameter` is assumed to be the parameter that requires the custom
/// resource implementation to create a new physical resource, thus the ID changes with it.
///
/// [`CfnResponse`]: enum.CfnResponse.html
pub trait PhysicalResourceIdSuffixProvider {
    /// Creates a suffix that uniquely identifies the physical resource represented by the type
    /// holding the AWS CloudFormation resource properties.
    fn physical_resource_id_suffix(&self) -> String;
}

impl<T> PhysicalResourceIdSuffixProvider for Option<T>
where
    T: PhysicalResourceIdSuffixProvider,
{
    fn physical_resource_id_suffix(&self) -> String {
        match self {
            Some(value) => value.physical_resource_id_suffix(),
            None => String::new(),
        }
    }
}

impl PhysicalResourceIdSuffixProvider for () {
    fn physical_resource_id_suffix(&self) -> String {
        String::new()
    }
}

/// On stack modification, AWS CloudFormation sends out a request for custom resources. This enum
/// can represent such a request, encapsulating the three request variants:
///
/// 1. Creation of a custom resource.
/// 2. Update of a custom resource.
/// 3. Deletion of a custom resource.
///
/// (For more information on AWS CloudFormation custom resource requests, see
/// [docs.aws.amazon.com].)
///
/// When creating/updating a custom resource, AWS CloudFormation forwards any additional key-value
/// pairs the template designer provided with the request. To enable strict typing on this data,
/// `CfnRequest` has the generic type parameter `P` which the caller provides. This has to be a
/// deserializable type.
///
/// [docs.aws.amazon.com]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/crpg-ref-requests.html
///
/// ## Example
///
/// The following is an example on how one can create a type that is deserializable through [Serde],
/// such that the untyped JSON map object provided by AWS CloudFormation can be converted into a
/// strongly typed struct. (If the JSON is not compatible with the struct, deserialization and thus
/// modification of the custom resource fails.)
///
/// ```
/// # extern crate cfn_resource_provider;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # use cfn_resource_provider::*;
/// #[derive(Debug, PartialEq, Clone, Deserialize)]
/// struct MyResourceProperties {
///     parameter1: String,
///     parameter2: Vec<String>,
/// }
/// # fn main() {
/// let actual = serde_json::from_value(json!(
///     {
///         "parameter1": "example for the first parameter",
///         "parameter2": ["list", "of", "values"]
///     }
/// )).unwrap();
///
/// let expected = MyResourceProperties {
///     parameter1: "example for the first parameter".to_owned(),
///     parameter2: vec!["list".to_owned(), "of".to_owned(), "values".to_owned()],
/// };
///
/// assert_eq!(expected, actual);
/// # }
/// ```
///
/// [Serde]: https://serde.rs/
///
/// ## Required presence of resource properties
///
/// If you have read the AWS CloudFormation documentation on [custom resource requests], you might
/// have seen that the `ResourceProperties` field on a request sent by AWS CloudFormation can be
/// optional, whereas all variants in this enum seem to require the field to be present.
///
/// The reason for the field being optional is (presumably) that AWS CloudFormation wants to support
/// custom resources that do not require additional parameters besides the ones automatically sent
/// by AWS CloudFormation, i.e. a custom resource might be just fine with only the stack ID.
///
/// Where this reasoning falls short, and where the documentation contradicts itself, is when it
/// comes to updating resources. For update requests it is documented that the
/// `OldResourceProperties` field is mandatory. Now, what happens if you update a resource that
/// previously didn't have any properties? Will the `OldResourceProperties` field be present as the
/// documentation requires it to be, although it cannot have any (reasonable) content?
///
/// For this reason, and for the sake of simplicity in usage and implementation, the user of this
/// library can decide whether they want all property fields to be required or optional. You have at
/// least four options:
///
/// 1. If your custom resource requires additional properties to function correctly, simply provide
///    your type `T` as-is.
///
/// 2. If you want your resource to support custom resource properties, but not to depend on them,
///    you can provide an `Option<T>` instead.
///
/// 3. Should you not need custom resource properties at all, but want the deserialization of the
///    request to fail if any are provided, you can specify `Option<()>`.
///
/// 4. If you don't need custom resource properties _and_ don't want to fail should they have been
///    provided, you can specify `Ignored` as the type. This is a [custom struct included] in this
///    library that deserializes any and no data into itself. This means that any data provided by
///    AWS CloudFormation will be discarded, but it will also not fail if no data was present.
///
/// [custom resource requests]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/crpg-ref-requests.html
/// [custom struct included]: (struct.Ignored.html)
///
/// ## License attribution
///
/// The documentation for the `CfnRequest` enum-variants and their fields has been taken unmodified
/// from the AWS CloudFormation [Custom Resource Reference], which is licensed under [CC BY-SA 4.0].
///
/// [Custom Resource Reference]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/crpg-ref.html
/// [CC BY-SA 4.0]: https://creativecommons.org/licenses/by-sa/4.0/
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "RequestType")]
pub enum CfnRequest<P>
where
    P: Clone,
{
    /// Custom resource provider requests with `RequestType` set to "`Create`" are sent when the
    /// template developer creates a stack that contains a custom resource. _See
    /// [docs.aws.amazon.com] for more information._
    ///
    /// [docs.aws.amazon.com]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/crpg-ref-requesttypes-create.html
    #[serde(rename_all = "PascalCase")]
    Create {
        /// A unique ID for the request.
        request_id: String,
        /// The response URL identifies a presigned S3 bucket that receives responses from the
        /// custom resource provider to AWS CloudFormation.
        #[serde(rename = "ResponseURL")]
        response_url: String,
        /// The template developer-chosen resource type of the custom resource in the AWS
        /// CloudFormation template. Custom resource type names can be up to 60 characters long and
        /// can include alphanumeric and the following characters: `_@-`.
        resource_type: String,
        /// The template developer-chosen name (logical ID) of the custom resource in the AWS
        /// CloudFormation template.
        logical_resource_id: String,
        /// The Amazon Resource Name (ARN) that identifies the stack that contains the custom
        /// resource.
        stack_id: String,
        /// This field contains the contents of the `Properties` object sent by the template
        /// developer. Its contents are defined by the custom resource provider.
        resource_properties: P,
    },
    /// Custom resource provider requests with `RequestType` set to "`Delete`" are sent when the
    /// template developer deletes a stack that contains a custom resource. To successfully delete a
    /// stack with a custom resource, the custom resource provider must respond successfully to a
    /// delete request. _See [docs.aws.amazon.com] for more information._
    ///
    /// [docs.aws.amazon.com]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/crpg-ref-requesttypes-delete.html
    #[serde(rename_all = "PascalCase")]
    Delete {
        /// A unique ID for the request.
        request_id: String,
        /// The response URL identifies a presigned S3 bucket that receives responses from the
        /// custom resource provider to AWS CloudFormation.
        #[serde(rename = "ResponseURL")]
        response_url: String,
        /// The template developer-chosen resource type of the custom resource in the AWS
        /// CloudFormation template. Custom resource type names can be up to 60 characters long and
        /// can include alphanumeric and the following characters: `_@-`.
        resource_type: String,
        /// The template developer-chosen name (logical ID) of the custom resource in the AWS
        /// CloudFormation template.
        logical_resource_id: String,
        /// The Amazon Resource Name (ARN) that identifies the stack that contains the custom
        /// resource.
        stack_id: String,
        /// A required custom resource provider-defined physical ID that is unique for that
        /// provider.
        physical_resource_id: String,
        /// This field contains the contents of the `Properties` object sent by the template
        /// developer. Its contents are defined by the custom resource provider.
        resource_properties: P,
    },
    /// Custom resource provider requests with `RequestType` set to "`Update`" are sent when there's
    /// any change to the properties of the custom resource within the template. Therefore, custom
    /// resource code doesn't have to detect changes because it knows that its properties have
    /// changed when Update is being called. _See [docs.aws.amazon.com] for more information._
    ///
    /// [docs.aws.amazon.com]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/crpg-ref-requesttypes-update.html
    #[serde(rename_all = "PascalCase")]
    Update {
        /// A unique ID for the request.
        request_id: String,
        /// The response URL identifies a presigned S3 bucket that receives responses from the
        /// custom resource provider to AWS CloudFormation.
        #[serde(rename = "ResponseURL")]
        response_url: String,
        /// The template developer-chosen resource type of the custom resource in the AWS
        /// CloudFormation template. Custom resource type names can be up to 60 characters long and
        /// can include alphanumeric and the following characters: `_@-`.
        resource_type: String,
        /// The template developer-chosen name (logical ID) of the custom resource in the AWS
        /// CloudFormation template.
        logical_resource_id: String,
        /// The Amazon Resource Name (ARN) that identifies the stack that contains the custom
        /// resource.
        stack_id: String,
        /// A required custom resource provider-defined physical ID that is unique for that
        /// provider.
        physical_resource_id: String,
        /// This field contains the contents of the `Properties` object sent by the template
        /// developer. Its contents are defined by the custom resource provider.
        resource_properties: P,
        /// The resource property values that were previously declared by the template developer in
        /// the AWS CloudFormation template.
        old_resource_properties: P,
    },
}

impl<P> CfnRequest<P>
where
    P: PhysicalResourceIdSuffixProvider + Clone,
{
    /// The request ID field exists for all variants of the [`CfnRequest` enum]. This is a helper
    /// method to access this field without requiring you to match for the variant yourself.
    ///
    /// [`CfnRequest` enum]: enum.CfnRequest.html
    #[inline(always)]
    pub fn request_id(&self) -> String {
        match self {
            CfnRequest::Create { request_id, .. } => request_id.to_owned(),
            CfnRequest::Delete { request_id, .. } => request_id.to_owned(),
            CfnRequest::Update { request_id, .. } => request_id.to_owned(),
        }
    }

    /// The response URL field exists for all variants of the [`CfnRequest` enum]. This is a helper
    /// method to access this field without requiring you to match for the variant yourself.
    ///
    /// [`CfnRequest` enum]: enum.CfnRequest.html
    #[inline(always)]
    pub fn response_url(&self) -> String {
        match self {
            CfnRequest::Create { response_url, .. } => response_url.to_owned(),
            CfnRequest::Delete { response_url, .. } => response_url.to_owned(),
            CfnRequest::Update { response_url, .. } => response_url.to_owned(),
        }
    }

    /// The resource type field exists for all variants of the [`CfnRequest` enum]. This is a helper
    /// method to access this field without requiring you to match for the variant yourself.
    ///
    /// [`CfnRequest` enum]: enum.CfnRequest.html
    #[inline(always)]
    pub fn resource_type(&self) -> String {
        match self {
            CfnRequest::Create { resource_type, .. } => resource_type.to_owned(),
            CfnRequest::Delete { resource_type, .. } => resource_type.to_owned(),
            CfnRequest::Update { resource_type, .. } => resource_type.to_owned(),
        }
    }

    /// The logical resource ID field exists for all variants of the [`CfnRequest` enum]. This is a
    /// helper method to access this field without requiring you to match for the variant yourself.
    ///
    /// [`CfnRequest` enum]: enum.CfnRequest.html
    #[inline(always)]
    pub fn logical_resource_id(&self) -> String {
        match self {
            CfnRequest::Create {
                logical_resource_id,
                ..
            } => logical_resource_id.to_owned(),
            CfnRequest::Delete {
                logical_resource_id,
                ..
            } => logical_resource_id.to_owned(),
            CfnRequest::Update {
                logical_resource_id,
                ..
            } => logical_resource_id.to_owned(),
        }
    }

    /// The stack ID field exists for all variants of the [`CfnRequest` enum]. This is a helper
    /// method to access this field without requiring you to match for the variant yourself.
    ///
    /// [`CfnRequest` enum]: enum.CfnRequest.html
    #[inline(always)]
    pub fn stack_id(&self) -> String {
        match self {
            CfnRequest::Create { stack_id, .. } => stack_id.to_owned(),
            CfnRequest::Delete { stack_id, .. } => stack_id.to_owned(),
            CfnRequest::Update { stack_id, .. } => stack_id.to_owned(),
        }
    }

    /// The physical resource ID field either exists or has to be (re)generated for all variants of
    /// the [`CfnRequest` enum]. This is a helper method to access this field without requiring you
    /// to match for the variant yourself, while always getting the correct and up-to-date physical
    /// resource ID.
    ///
    /// [`CfnRequest` enum]: enum.CfnRequest.html
    #[inline(always)]
    pub fn physical_resource_id(&self) -> String {
        match self {
            CfnRequest::Create {
                logical_resource_id,
                stack_id,
                resource_properties,
                ..
            }
            | CfnRequest::Update {
                logical_resource_id,
                stack_id,
                resource_properties,
                ..
            } => {
                let suffix = resource_properties.physical_resource_id_suffix();
                format!(
                    "arn:custom:cfn-resource-provider:::{stack_id}-{logical_resource_id}{suffix_separator}{suffix}",
                    stack_id = stack_id.rsplit('/').next().expect("failed to get GUID from stack ID"),
                    logical_resource_id = logical_resource_id,
                    suffix_separator = if suffix.is_empty() { "" } else { "/" },
                    suffix = suffix,
                )
            }
            CfnRequest::Delete {
                physical_resource_id,
                ..
            } => physical_resource_id.to_owned(),
        }
    }

    /// The resource properties field exists for all variants of the [`CfnRequest` enum]. This is a
    /// helper method to access this field without requiring you to match for the variant yourself.
    ///
    /// [`CfnRequest` enum]: enum.CfnRequest.html
    #[inline(always)]
    pub fn resource_properties(&self) -> &P {
        match self {
            CfnRequest::Create {
                resource_properties,
                ..
            } => resource_properties,
            CfnRequest::Delete {
                resource_properties,
                ..
            } => resource_properties,
            CfnRequest::Update {
                resource_properties,
                ..
            } => resource_properties,
        }
    }

    /// This method turns a [`CfnRequest`] into a [`CfnResponse`], choosing one of the `Success` or
    /// `Failed` variants based on a `Result`. A [`CfnResponse`] should always be created through
    /// this method to ensure that all the relevant response-fields that AWS CloudFormation requires
    /// are populated correctly.
    ///
    /// [`CfnRequest`]: enum.CfnRequest.html
    /// [`CfnResponse`]: enum.CfnResponse.html
    pub fn into_response<S>(self, result: &Result<Option<S>, Error>) -> CfnResponse
    where
        S: Serialize,
    {
        match result {
            Ok(data) => CfnResponse::Success {
                request_id: self.request_id(),
                logical_resource_id: self.logical_resource_id(),
                stack_id: self.stack_id(),
                physical_resource_id: self.physical_resource_id(),
                no_echo: None,
                data: data
                    .as_ref()
                    .and_then(|value| serde_json::to_value(value).ok()),
            },
            Err(e) => CfnResponse::Failed {
                reason: format!("{}", e),
                request_id: self.request_id(),
                logical_resource_id: self.logical_resource_id(),
                stack_id: self.stack_id(),
                physical_resource_id: self.physical_resource_id(),
            },
        }
    }
}

/// This is a special struct that can be used in conjunction with [Serde] to represent a field whose
/// contents should be discarded during deserialization if it is present, and doesn't fail if the
/// field doesn't exist.
///
/// This type is meant to be used as the generic type parameter for [`CfnRequest`] if your AWS
/// CloudFormation custom resource doesn't take any custom resource properties, but you don't want
/// deserialization to fail should any properties be specified.
///
/// [Serde]: https://serde.rs/
/// [`CfnRequest`]: enum.CfnRequest.html
#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct Ignored;

impl<'de> Deserialize<'de> for Ignored {
    fn deserialize<D>(_deserializer: D) -> Result<Ignored, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Ignored)
    }
}

impl PhysicalResourceIdSuffixProvider for Ignored {
    fn physical_resource_id_suffix(&self) -> String {
        String::new()
    }
}

/// This enum represents the response expected by AWS CloudFormation to a custom resource
/// modification request (see [`CfnRequest`]). It is serializable into the
/// required JSON form, such that it can be sent to the pre-signed S3 response-URL provided by AWS
/// CloudFormation without further modification.
///
/// This type should always be constructed from a [`CfnRequest`] using
/// [`CfnRequest::into_response`][into_response] such that the response-fields are pre-filled with
/// the expected values.
///
/// [`CfnRequest`]: enum.CfnRequest.html
/// [into_response]: enum.CfnRequest.html#method.into_response
///
/// ## License attribution
///
/// The documentation for the fields of the `CfnResponse` enum-variants has been taken unmodified
/// from the AWS CloudFormation [Custom Resource Reference], which is licensed under [CC BY-SA 4.0].
///
/// [Custom Resource Reference]: https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/crpg-ref.html
/// [CC BY-SA 4.0]: https://creativecommons.org/licenses/by-sa/4.0/
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "Status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CfnResponse {
    /// Indicates that the modification of the custom resource finished successfully.
    ///
    /// This can return data which the AWS CloudFormation template can interact with through the use
    /// of `Fn::GetAtt`.
    #[serde(rename_all = "PascalCase")]
    Success {
        /// A unique ID for the request. This response value should be copied _verbatim_ from the
        /// request.
        request_id: String,
        /// The template developer-chosen name (logical ID) of the custom resource in the AWS
        /// CloudFormation template. This response value should be copied _verbatim_ from the
        /// request.
        logical_resource_id: String,
        /// The Amazon Resource Name (ARN) that identifies the stack that contains the custom
        /// resource. This response value should be copied _verbatim_ from the request.
        stack_id: String,
        /// This value should be an identifier unique to the custom resource vendor, and can be up
        /// to 1 Kb in size. The value must be a non-empty string and must be identical for all
        /// responses for the same resource.
        physical_resource_id: String,
        /// Optional. Indicates whether to mask the output of the custom resource when retrieved by
        /// using the `Fn::GetAtt` function. If set to `true`, all returned values are masked with
        /// asterisks (\*\*\*\*\*). The default value is `false`.
        #[serde(skip_serializing_if = "Option::is_none")]
        no_echo: Option<bool>,
        /// Optional. The custom resource provider-defined name-value pairs to send with the
        /// response. You can access the values provided here by name in the template with
        /// `Fn::GetAtt`.
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    /// Indicates that the modification of the custom resource failed.
    ///
    /// A reason for this failure will be provided.
    #[serde(rename_all = "PascalCase")]
    Failed {
        /// Describes the reason for a failure response.
        reason: String,
        /// A unique ID for the request. This response value should be copied _verbatim_ from the
        /// request.
        request_id: String,
        /// The template developer-chosen name (logical ID) of the custom resource in the AWS
        /// CloudFormation template. This response value should be copied _verbatim_ from the
        /// request.
        logical_resource_id: String,
        /// The Amazon Resource Name (ARN) that identifies the stack that contains the custom
        /// resource. This response value should be copied _verbatim_ from the request.
        stack_id: String,
        /// This value should be an identifier unique to the custom resource vendor, and can be up
        /// to 1 Kb in size. The value must be a non-empty string and must be identical for all
        /// responses for the same resource.
        physical_resource_id: String,
    },
}

/// Process an AWS CloudFormation custom resource request.
///
/// This function will, in conjunction with [`rust-aws-lambda`][rust-aws-lambda], deserialize the
/// JSON message sent by AWS CloudFormation into a strongly typed struct. Any custom resource
/// properties you might have can be specified to have them deserialized, too.
///
/// `process` expects a single parameter, which should be a closure that receives a
/// [`CfnRequest<P>`][CfnRequest] as its only parameter, and is expected to return a type that can
/// succeed or fail (this can be a future or simply a [`Result`]; anything that implements
/// [`IntoFuture`]). The type returned for success has to be an `Option<S>`, where `S` needs to be
/// serializable. The failure type is expected to be [`failure::Error`]. The computation required to
/// create your custom resource should happen in this closure.
///
/// The result of your closure will then be used to construct the response that will be sent to AWS
/// CloudFormation. This response informs AWS CloudFormation whether creating the custom resource
/// was successful or if it failed (including a reason for the failure). This is done by converting
/// the initial [`CfnRequest`][CfnRequest] into a [`CfnResponse`][CfnResponse], pre-filling the
/// required fields based on the result your closure returned.
///
/// If your closure has errored, the failure reason will be extracted from the error you returned.
/// If your closure succeeded, the positive return value will be serialized into the
/// [`data` field][CfnResponse.Success.data] (unless the returned `Option` is `None`). (Specifying
/// the [`no_echo` option] is currently not possible.)
///
/// ## Example
///
/// ```norun
/// extern crate aws_lambda as lambda;
/// extern crate cfn_resource_provider as cfn;
///
/// use cfn::*;
///
/// fn main() {
///     lambda::start(cfn::process(|event: CfnRequest<MyResourceProperties>| {
///         // Perform the necessary steps to create the custom resource. Afterwards you can return
///         // some data that should be serialized into the response. If you don't want to serialize
///         // any data, you can return `None` (were you unfortunately have to specify the unknown
///         // serializable type using the turbofish).
///         Ok(None::<()>)
///     });
/// }
/// ```
///
/// [rust-aws-lambda]: https://github.com/srijs/rust-aws-lambda
/// [CfnRequest]: enum.CfnRequest.html
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
/// [`IntoFuture`]: https://docs.rs/futures/0.1/futures/future/trait.IntoFuture.html
/// [`failure::Error`]: https://docs.rs/failure/0.1/failure/struct.Error.html
/// [CfnResponse]: enum.CfnRequest.html
/// [CfnResponse.Success.data]: enum.CfnResponse.html#variant.Success.field.data
/// [CfnResponse.Success.no_echo]: enum.CfnResponse.html#variant.Success.field.no_echo
pub fn process<F, R, P, S>(
    f: F,
) -> impl Fn(CfnRequest<P>) -> Box<Future<Item = Option<S>, Error = Error> + Send>
where
    F: Fn(CfnRequest<P>) -> R + Send + Sync + 'static,
    R: IntoFuture<Item = Option<S>, Error = Error> + Send + 'static,
    R::Future: Send,
    S: Serialize + Send + 'static,
    P: PhysicalResourceIdSuffixProvider + Clone + Send + 'static,
{
    // The process below is a bit convoluted to read, the main reason for this is the following: we
    // want to forward the response given by the closure `f` to our caller, while using that same
    // response to inform AWS CloudFormation of the status of the custom resource.
    //
    // To accomplish this, we use a nested chain of futures that works as follows.
    //
    // 1. Call closure `f`.
    // 2. Transform the initial request into a AWS CloudFormation response, deciding on success or
    //    failure through the result returned by `f`.
    // 3. Try to serialize and send the response to AWS CloudFormation (if this fails at any step,
    //    propagate the error through to our caller).
    // 4. If informing AWS CloudFormation succeeded, return the initial result of `f` to our caller.
    move |request: CfnRequest<P>| {
        let response_url = request.response_url();
        Box::new(f(request.clone()).into_future().then(|request_result| {
            let cfn_response = request.into_response(&request_result);
            serde_json::to_string(&cfn_response)
                .map_err(Into::into)
                .into_future()
                .and_then(|cfn_response| {
                    reqwest::async::Client::builder()
                        .build()
                        .into_future()
                        .and_then(move |client| {
                            client
                                .put(&response_url)
                                .header("Content-Type", "")
                                .body(cfn_response)
                                .send()
                        })
                        .and_then(reqwest::async::Response::error_for_status)
                        .map_err(Into::into)
                })
                .and_then(move |_| request_result)
        }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Clone)]
    struct Empty;
    impl PhysicalResourceIdSuffixProvider for Empty {
        fn physical_resource_id_suffix(&self) -> String {
            String::new()
        }
    }

    #[derive(Debug, Clone)]
    struct StaticSuffix;
    impl PhysicalResourceIdSuffixProvider for StaticSuffix {
        fn physical_resource_id_suffix(&self) -> String {
            "STATIC-SUFFIX".to_owned()
        }
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct ExampleProperties {
        example_property_1: String,
        example_property_2: Option<bool>,
    }
    impl PhysicalResourceIdSuffixProvider for ExampleProperties {
        fn physical_resource_id_suffix(&self) -> String {
            self.example_property_1.to_owned()
        }
    }

    /// This test verifies that if the suffix returned by the generic type parameter given to
    /// `CfnRequest` is empty, the physical resource ID does not end on a separating slash.
    #[test]
    fn empty_suffix_has_no_trailing_slash() {
        let request: CfnRequest<Empty> = CfnRequest::Create {
            request_id: String::new(),
            response_url: String::new(),
            resource_type: String::new(),
            logical_resource_id: String::new(),
            stack_id: String::new(),
            resource_properties: Empty,
        };
        assert!(!request.physical_resource_id().ends_with("/"));
    }

    /// This test verifies that the suffix provided by the generic type given to `CfnRequest` is
    /// separated from the resource ID by a slash.
    #[test]
    fn static_suffix_is_correctly_appended() {
        let request: CfnRequest<StaticSuffix> = CfnRequest::Create {
            request_id: String::new(),
            response_url: String::new(),
            resource_type: String::new(),
            logical_resource_id: String::new(),
            stack_id: String::new(),
            resource_properties: StaticSuffix,
        };
        assert!(request.physical_resource_id().ends_with("/STATIC-SUFFIX"));
    }

    /// This is meant as a type-checking test: we want to ensure that we can provide a required type
    /// to `CfnRequest`.
    #[test]
    fn cfnrequest_generic_type_required() {
        let request: CfnRequest<Empty> = CfnRequest::Create {
            request_id: String::new(),
            response_url: String::new(),
            resource_type: String::new(),
            logical_resource_id: String::new(),
            stack_id: String::new(),
            resource_properties: Empty,
        };
        assert!(!request.physical_resource_id().is_empty());
    }

    /// This is meant as a type-checking test: we want to ensure that we can provide an optional
    /// type to `CfnRequest`.
    #[test]
    fn cfnrequest_generic_type_optional() {
        let mut request: CfnRequest<Option<Empty>> = CfnRequest::Create {
            request_id: String::new(),
            response_url: String::new(),
            resource_type: String::new(),
            logical_resource_id: String::new(),
            stack_id: String::new(),
            resource_properties: None,
        };
        assert!(!request.physical_resource_id().is_empty());
        assert!(!request.physical_resource_id().ends_with("/"));
        request = CfnRequest::Create {
            request_id: String::new(),
            response_url: String::new(),
            resource_type: String::new(),
            logical_resource_id: String::new(),
            stack_id: String::new(),
            resource_properties: Some(Empty),
        };
        assert!(!request.physical_resource_id().is_empty());
        assert!(!request.physical_resource_id().ends_with("/"));
    }

    /// This is meant as a type-checking test: we want to ensure that we can provide the optional
    /// unit-type to `CfnRequest`.
    #[test]
    fn cfnrequest_generic_type_optional_unit() {
        let mut request: CfnRequest<Option<()>> = CfnRequest::Create {
            request_id: String::new(),
            response_url: String::new(),
            resource_type: String::new(),
            logical_resource_id: String::new(),
            stack_id: String::new(),
            resource_properties: None,
        };
        assert!(!request.physical_resource_id().is_empty());
        assert!(!request.physical_resource_id().ends_with("/"));
        request = CfnRequest::Create {
            request_id: String::new(),
            response_url: String::new(),
            resource_type: String::new(),
            logical_resource_id: String::new(),
            stack_id: String::new(),
            resource_properties: Some(()),
        };
        assert!(!request.physical_resource_id().is_empty());
        assert!(!request.physical_resource_id().ends_with("/"));
    }

    #[test]
    fn cfnrequest_type_present() {
        let expected_request: CfnRequest<ExampleProperties> = CfnRequest::Create {
            request_id: "unique id for this create request".to_owned(),
            response_url: "pre-signed-url-for-create-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            resource_properties: ExampleProperties {
                example_property_1: "example property 1".to_owned(),
                example_property_2: None,
            },
        };
        let actual_request: CfnRequest<ExampleProperties> = serde_json::from_value(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "ResourceProperties": {
                "ExampleProperty1": "example property 1"
            }
        }))
        .unwrap();
        assert_eq!(expected_request, actual_request);
    }

    #[test]
    #[should_panic]
    fn cfnrequest_type_absent() {
        serde_json::from_value::<CfnRequest<ExampleProperties>>(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid"
        }))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn cfnrequest_type_malformed() {
        serde_json::from_value::<CfnRequest<ExampleProperties>>(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "ResourceProperties": {
                "UnknownProperty": null
            }
        }))
        .unwrap();
    }

    #[test]
    fn cfnrequest_type_option_present() {
        let expected_request: CfnRequest<Option<ExampleProperties>> = CfnRequest::Create {
            request_id: "unique id for this create request".to_owned(),
            response_url: "pre-signed-url-for-create-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            resource_properties: Some(ExampleProperties {
                example_property_1: "example property 1".to_owned(),
                example_property_2: None,
            }),
        };
        let actual_request: CfnRequest<Option<ExampleProperties>> = serde_json::from_value(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "ResourceProperties": {
                "ExampleProperty1": "example property 1"
            }
        }))
        .unwrap();
        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn cfnrequest_type_option_absent() {
        let expected_request: CfnRequest<Option<ExampleProperties>> = CfnRequest::Create {
            request_id: "unique id for this create request".to_owned(),
            response_url: "pre-signed-url-for-create-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            resource_properties: None,
        };
        let actual_request: CfnRequest<Option<ExampleProperties>> = serde_json::from_value(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid"
        }))
        .unwrap();
        assert_eq!(expected_request, actual_request);
    }

    #[test]
    #[should_panic]
    fn cfnrequest_type_option_malformed() {
        serde_json::from_value::<CfnRequest<Option<ExampleProperties>>>(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "ResourceProperties": {
                "UnknownProperty": null
            }
        }))
        .unwrap();
    }

    #[test]
    fn cfnrequest_type_option_unit() {
        let expected_request: CfnRequest<Option<()>> = CfnRequest::Create {
            request_id: "unique id for this create request".to_owned(),
            response_url: "pre-signed-url-for-create-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            resource_properties: None,
        };
        let mut actual_request: CfnRequest<Option<()>> = serde_json::from_value(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "ResourceProperties" : null
        }))
        .unwrap();
        assert_eq!(expected_request, actual_request);
        actual_request = serde_json::from_value(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid"
        }))
        .unwrap();
        assert_eq!(expected_request, actual_request);
    }

    #[test]
    #[should_panic]
    fn cfnrequest_type_option_unit_data_provided() {
        serde_json::from_value::<CfnRequest<Option<()>>>(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "ResourceProperties" : {
                "key1" : "string",
                "key2" : [ "list" ],
                "key3" : { "key4" : "map" }
            }
        }))
        .unwrap();
    }

    #[test]
    fn cfnrequest_type_ignored() {
        let expected_request: CfnRequest<Ignored> = CfnRequest::Create {
            request_id: "unique id for this create request".to_owned(),
            response_url: "pre-signed-url-for-create-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            resource_properties: Ignored,
        };
        let mut actual_request: CfnRequest<Ignored> = serde_json::from_value(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "ResourceProperties" : {
                "key1" : "string",
                "key2" : [ "list" ],
                "key3" : { "key4" : "map" }
            }
        }))
        .unwrap();
        assert_eq!(expected_request, actual_request);
        actual_request = serde_json::from_value(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid"
        }))
        .unwrap();
        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn cfnrequest_create_example() {
        #[derive(Debug, Clone, PartialEq, Deserialize)]
        struct ExampleProperties {
            key1: String,
            key2: Vec<String>,
            key3: serde_json::Value,
        }

        let expected_request = CfnRequest::Create {
            request_id: "unique id for this create request".to_owned(),
            response_url: "pre-signed-url-for-create-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            resource_properties: ExampleProperties {
                key1: "string".to_owned(),
                key2: vec!["list".to_owned()],
                key3: json!({ "key4": "map" }),
            },
        };

        let actual_request = serde_json::from_value(json!({
            "RequestType" : "Create",
            "RequestId" : "unique id for this create request",
            "ResponseURL" : "pre-signed-url-for-create-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "ResourceProperties" : {
                "key1" : "string",
                "key2" : [ "list" ],
                "key3" : { "key4" : "map" }
            }
        }))
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn cfnresponse_create_success_example() {
        let expected_response = json!({
            "Status" : "SUCCESS",
            "RequestId" : "unique id for this create request (copied from request)",
            "LogicalResourceId" : "name of resource in template (copied from request)",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)",
            "PhysicalResourceId" : "required vendor-defined physical id that is unique for that vendor",
            "Data" : {
                "keyThatCanBeUsedInGetAtt1" : "data for key 1",
                "keyThatCanBeUsedInGetAtt2" : "data for key 2"
            }
        });

        let actual_response = serde_json::to_value(CfnResponse::Success {
            request_id: "unique id for this create request (copied from request)".to_owned(),
            logical_resource_id: "name of resource in template (copied from request)".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)".to_owned(),
            physical_resource_id: "required vendor-defined physical id that is unique for that vendor".to_owned(),
            no_echo: None,
            data: Some(json!({
                "keyThatCanBeUsedInGetAtt1" : "data for key 1",
                "keyThatCanBeUsedInGetAtt2" : "data for key 2"
            })),
        }).unwrap();

        assert_eq!(expected_response, actual_response);
    }

    #[test]
    fn cfnresponse_create_failed_example() {
        let expected_response = json!({
            "Status" : "FAILED",
            "Reason" : "Required failure reason string",
            "RequestId" : "unique id for this create request (copied from request)",
            "LogicalResourceId" : "name of resource in template (copied from request)",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)",
            "PhysicalResourceId" : "required vendor-defined physical id that is unique for that vendor"
        });

        let actual_response = serde_json::to_value(CfnResponse::Failed {
            reason: "Required failure reason string".to_owned(),
            request_id: "unique id for this create request (copied from request)".to_owned(),
            logical_resource_id: "name of resource in template (copied from request)".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)".to_owned(),
            physical_resource_id: "required vendor-defined physical id that is unique for that vendor".to_owned(),
        }).unwrap();

        assert_eq!(expected_response, actual_response);
    }

    #[test]
    fn cfnrequest_delete_example() {
        #[derive(Debug, PartialEq, Clone, Deserialize)]
        struct ExampleProperties {
            key1: String,
            key2: Vec<String>,
            key3: serde_json::Value,
        }

        let expected_request = CfnRequest::Delete {
            request_id: "unique id for this delete request".to_owned(),
            response_url: "pre-signed-url-for-delete-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            physical_resource_id: "custom resource provider-defined physical id".to_owned(),
            resource_properties: ExampleProperties {
                key1: "string".to_owned(),
                key2: vec!["list".to_owned()],
                key3: json!({ "key4": "map" }),
            },
        };

        let actual_request = serde_json::from_value(json!({
            "RequestType" : "Delete",
            "RequestId" : "unique id for this delete request",
            "ResponseURL" : "pre-signed-url-for-delete-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "PhysicalResourceId" : "custom resource provider-defined physical id",
            "ResourceProperties" : {
                "key1" : "string",
                "key2" : [ "list" ],
                "key3" : { "key4" : "map" }
            }
        }))
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn cfnresponse_delete_success_example() {
        let expected_response = json!({
            "Status" : "SUCCESS",
            "RequestId" : "unique id for this delete request (copied from request)",
            "LogicalResourceId" : "name of resource in template (copied from request)",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)",
            "PhysicalResourceId" : "custom resource provider-defined physical id"
        });

        let actual_response = serde_json::to_value(CfnResponse::Success {
            request_id: "unique id for this delete request (copied from request)".to_owned(),
            logical_resource_id: "name of resource in template (copied from request)".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)".to_owned(),
            physical_resource_id: "custom resource provider-defined physical id".to_owned(),
            no_echo: None,
            data: None,
        }).unwrap();

        assert_eq!(expected_response, actual_response);
    }

    #[test]
    fn cfnresponse_delete_failed_example() {
        let expected_response = json!({
            "Status" : "FAILED",
            "Reason" : "Required failure reason string",
            "RequestId" : "unique id for this delete request (copied from request)",
            "LogicalResourceId" : "name of resource in template (copied from request)",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)",
            "PhysicalResourceId" : "custom resource provider-defined physical id"
        });

        let actual_response = serde_json::to_value(CfnResponse::Failed {
            reason: "Required failure reason string".to_owned(),
            request_id: "unique id for this delete request (copied from request)".to_owned(),
            logical_resource_id: "name of resource in template (copied from request)".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)".to_owned(),
            physical_resource_id: "custom resource provider-defined physical id".to_owned(),
        }).unwrap();

        assert_eq!(expected_response, actual_response);
    }

    #[test]
    fn cfnrequest_update_example() {
        #[derive(Debug, PartialEq, Clone, Deserialize)]
        struct ExampleProperties {
            key1: String,
            key2: Vec<String>,
            key3: serde_json::Value,
        }

        let expected_request = CfnRequest::Update {
            request_id: "unique id for this update request".to_owned(),
            response_url: "pre-signed-url-for-update-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            physical_resource_id: "custom resource provider-defined physical id".to_owned(),
            resource_properties: ExampleProperties {
                key1: "new-string".to_owned(),
                key2: vec!["new-list".to_owned()],
                key3: json!({ "key4": "new-map" }),
            },
            old_resource_properties: ExampleProperties {
                key1: "string".to_owned(),
                key2: vec!["list".to_owned()],
                key3: json!({ "key4": "map" }),
            },
        };

        let actual_request: CfnRequest<ExampleProperties> = serde_json::from_value(json!({
            "RequestType" : "Update",
            "RequestId" : "unique id for this update request",
            "ResponseURL" : "pre-signed-url-for-update-response",
            "ResourceType" : "Custom::MyCustomResourceType",
            "LogicalResourceId" : "name of resource in template",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "PhysicalResourceId" : "custom resource provider-defined physical id",
            "ResourceProperties" : {
                "key1" : "new-string",
                "key2" : [ "new-list" ],
                "key3" : { "key4" : "new-map" }
            },
            "OldResourceProperties" : {
                "key1" : "string",
                "key2" : [ "list" ],
                "key3" : { "key4" : "map" }
            }
        }))
        .unwrap();

        assert_eq!(expected_request, actual_request);
    }

    #[test]
    fn cfnresponse_update_success_example() {
        let expected_response = json!({
            "Status" : "SUCCESS",
            "RequestId" : "unique id for this update request (copied from request)",
            "LogicalResourceId" : "name of resource in template (copied from request)",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)",
            "PhysicalResourceId" : "custom resource provider-defined physical id",
            "Data" : {
                "keyThatCanBeUsedInGetAtt1" : "data for key 1",
                "keyThatCanBeUsedInGetAtt2" : "data for key 2"
            }
        });

        let actual_response = serde_json::to_value(CfnResponse::Success {
            request_id: "unique id for this update request (copied from request)".to_owned(),
            logical_resource_id: "name of resource in template (copied from request)".to_owned(),
            physical_resource_id: "custom resource provider-defined physical id".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)".to_owned(),
            no_echo: None,
            data: Some(json!({
               "keyThatCanBeUsedInGetAtt1" : "data for key 1",
               "keyThatCanBeUsedInGetAtt2" : "data for key 2"
            })),
        }).unwrap();

        assert_eq!(expected_response, actual_response);
    }

    #[test]
    fn cfnresponse_update_failed_example() {
        let expected_response = json!({
            "Status" : "FAILED",
            "Reason" : "Required failure reason string",
            "RequestId" : "unique id for this update request (copied from request)",
            "LogicalResourceId" : "name of resource in template (copied from request)",
            "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)",
            "PhysicalResourceId" : "custom resource provider-defined physical id"
        });

        let actual_response = serde_json::to_value(CfnResponse::Failed {
            reason: "Required failure reason string".to_owned(),
            request_id: "unique id for this update request (copied from request)".to_owned(),
            logical_resource_id: "name of resource in template (copied from request)".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid (copied from request)".to_owned(),
            physical_resource_id: "custom resource provider-defined physical id".to_owned(),
        }).unwrap();

        assert_eq!(expected_response, actual_response);
    }

    #[test]
    fn cfnresponse_from_cfnrequest_unit() {
        let actual_request: CfnRequest<Ignored> = CfnRequest::Create {
            request_id: "unique id for this create request".to_owned(),
            response_url: "pre-signed-url-for-create-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            resource_properties: Ignored,
        };
        let actual_response =
            serde_json::to_value(actual_request.into_response(&Ok(None::<()>))).unwrap();
        let expected_response = json!({
            "Status": "SUCCESS",
            "RequestId": "unique id for this create request",
            "LogicalResourceId": "name of resource in template",
            "StackId": "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "PhysicalResourceId": "arn:custom:cfn-resource-provider:::guid-name of resource in template"
        });

        assert_eq!(actual_response, expected_response)
    }

    #[test]
    fn cfnresponse_from_cfnrequest_serializable() {
        let actual_request: CfnRequest<Ignored> = CfnRequest::Create {
            request_id: "unique id for this create request".to_owned(),
            response_url: "pre-signed-url-for-create-response".to_owned(),
            resource_type: "Custom::MyCustomResourceType".to_owned(),
            logical_resource_id: "name of resource in template".to_owned(),
            stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
            resource_properties: Ignored,
        };
        let actual_response =
            serde_json::to_value(actual_request.into_response(&Ok(Some(ExampleProperties {
                example_property_1: "example return property 1".to_owned(),
                example_property_2: None,
            }))))
            .unwrap();
        let expected_response = json!({
            "Status": "SUCCESS",
            "RequestId": "unique id for this create request",
            "LogicalResourceId": "name of resource in template",
            "StackId": "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
            "PhysicalResourceId": "arn:custom:cfn-resource-provider:::guid-name of resource in template",
            "Data": {
                "ExampleProperty1": "example return property 1",
                "ExampleProperty2": null,
            }
        });

        assert_eq!(actual_response, expected_response)
    }
}
