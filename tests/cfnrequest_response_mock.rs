extern crate cfn_resource_provider;

extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate mockito;
extern crate tokio_core;

use cfn_resource_provider::*;
use failure::Error;
use mockito::Matcher;
use tokio_core::reactor::Core;

// TODO: identify the actual responses sent by AWS CloudFormation/S3
lazy_static! {
    static ref CFNREQUEST_CREATE_IGNORED: CfnRequest<Ignored> = CfnRequest::Create {
        request_id: "REQUEST-ID".to_owned(),
        response_url: format!("{}{}", mockito::server_url(), "/presigned-url"),
        resource_type: "REQUEST-TYPE".to_owned(),
        logical_resource_id: "LOGICAL-RESOURCE-ID".to_owned(),
        stack_id: "STACK-ID".to_owned(),
        resource_properties: Ignored,
    };
    static ref CFNREQUEST_DELETE_IGNORED: CfnRequest<Ignored> = CfnRequest::Delete {
        request_id: "REQUEST-ID".to_owned(),
        response_url: format!("{}{}", mockito::server_url(), "/presigned-url"),
        resource_type: "REQUEST-TYPE".to_owned(),
        logical_resource_id: "LOGICAL-RESOURCE-ID".to_owned(),
        stack_id: "STACK-ID".to_owned(),
        physical_resource_id: "arn:custom:cfn-resource-provider:::STACK-ID-LOGICAL-RESOURCE-ID"
            .to_owned(),
        resource_properties: Ignored,
    };
    static ref CFNREQUEST_UPDATE_IGNORED: CfnRequest<Ignored> = CfnRequest::Update {
        request_id: "REQUEST-ID".to_owned(),
        response_url: format!("{}{}", mockito::server_url(), "/presigned-url"),
        resource_type: "REQUEST-TYPE".to_owned(),
        logical_resource_id: "LOGICAL-RESOURCE-ID".to_owned(),
        stack_id: "STACK-ID".to_owned(),
        physical_resource_id: "arn:custom:cfn-resource-provider:::STACK-ID-LOGICAL-RESOURCE-ID"
            .to_owned(),
        resource_properties: Ignored,
        old_resource_properties: Ignored,
    };
}

fn _simple_test<P>(status_code: usize, request: CfnRequest<P>) -> Result<Option<()>, Error>
where
    P: PhysicalResourceIdSuffixProvider + Clone + Send + Sync + 'static,
{
    let mock = mockito::mock("PUT", "/presigned-url")
        .match_header("Content-Type", "")
        .match_header("Content-Length", Matcher::Any)
        .match_body(Matcher::Regex(
            r"arn:custom:cfn-resource-provider:::STACK-ID-LOGICAL-RESOURCE-ID".to_owned(),
        ))
        .with_status(status_code)
        .create();

    let f = cfn_resource_provider::process(|_event: CfnRequest<P>| Ok(None))(request);

    let mut core = Core::new().unwrap();
    let result = core.run(f);

    mock.assert();
    result
}

fn _simple_test_200<P>(request: CfnRequest<P>)
where
    P: PhysicalResourceIdSuffixProvider + Clone + Send + Sync + 'static,
{
    let result = _simple_test(200, request);
    assert!(result.is_ok());
}

fn _simple_test_403<P>(request: CfnRequest<P>)
where
    P: PhysicalResourceIdSuffixProvider + Clone + Send + Sync + 'static,
{
    let result = _simple_test(403, request);
    assert!(result.is_err());
}

#[test]
fn cfnrequest_create_ignored_200() {
    _simple_test_200(CFNREQUEST_CREATE_IGNORED.clone());
}

#[test]
fn cfnrequest_create_ignored_403() {
    _simple_test_403(CFNREQUEST_CREATE_IGNORED.clone());
}

#[test]
fn cfnrequest_delete_ignored_200() {
    _simple_test_200(CFNREQUEST_DELETE_IGNORED.clone());
}

#[test]
fn cfnrequest_delete_ignored_403() {
    _simple_test_403(CFNREQUEST_DELETE_IGNORED.clone());
}

#[test]
fn cfnrequest_update_ignored_200() {
    _simple_test_200(CFNREQUEST_UPDATE_IGNORED.clone());
}

#[test]
fn cfnrequest_update_ignored_403() {
    _simple_test_403(CFNREQUEST_UPDATE_IGNORED.clone());
}
