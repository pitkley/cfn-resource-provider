#![cfg(feature = "go-runtime")]
#![allow(dead_code)]

extern crate aws_lambda as lambda;
extern crate cfn_resource_provider as cfn;
#[macro_use]
extern crate serde_derive;

use cfn::*;

#[derive(Clone, Deserialize)]
struct DummyType;

impl PhysicalResourceIdSuffixProvider for DummyType {
    fn physical_resource_id_suffix(&self) -> String {
        unimplemented!()
    }
}

fn check_compatibility() {
    lambda::start(cfn::process(|_request: CfnRequest<DummyType>| {
        Ok(None::<()>)
    }));
}
